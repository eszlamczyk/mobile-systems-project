import AVFoundation
import Atomics
import DspCore

/// Called on the main thread, plotDidUpdate ~30 times per second.
@MainActor protocol PlotObserver: AnyObject {
    func plotDidUpdate()
    func ghostDidChange()
}

nonisolated final class TunerEngine: @unchecked Sendable {
    // ~1 s of silence at 48 kHz with the default hop
    static let noiseHops = 47
    
    private let engine = AVAudioEngine()
    private let ring = SPSCRing(capacity: 1 << 14 ) // ~340 ms at 48kHz
    
    private let wake = DispatchSemaphore(value: 0)
    private let finished = DispatchSemaphore(value: 0)
    
    private let published = ManagedAtomic<Snapshot>(.empty)
    private let learnRequested = ManagedAtomic<Bool>(false)
    private let calibrated = ManagedAtomic<Bool>(false)
    private var dspThread: Thread?
    private var tuner: Tuner?
    private var tunerRate: UInt32 = 0
    private let targetMidi = ManagedAtomic<Int32>(69)
    private var configObserver: NSObjectProtocol?
    
    //plot related
    static let plotPeriodNs: UInt64 = 1_000_000_000 / 30
    private let plot = TripleBuffer(size: SpectrumGrid.points)
    
    ///Important: this is main thread ONLY.
    @MainActor weak var plotObserver: PlotObserver?
    
    @MainActor private(set) var ghost: [Float]?
    @MainActor private(set) var grid: SpectrumGrid?
    
    @MainActor var onCalibrationLost: (@MainActor () -> Void)?
    
    @MainActor func latestPlot() -> UnsafeBufferPointer<Float> { plot.latest() }
    
    var latest: Snapshot {
        published.load(ordering: .acquiring)
    }
    
    var isLearningNoise: Bool {
        learnRequested.load(ordering: .acquiring)
    }

    init() {
        configObserver = NotificationCenter.default.addObserver(forName: .AVAudioEngineConfigurationChange, object: engine, queue: .main) {
            [weak self] _ in
            guard let self, dspThread != nil else { return }
            stop()
            start()
        }
    }

    deinit {
        if let configObserver {
            NotificationCenter.default.removeObserver(configObserver)
        }
    }

    func start() {
        guard dspThread == nil else { return }

        let session = AVAudioSession.sharedInstance()

        do {
            try session.setCategory(.record, mode: .measurement)
            try? session.setPreferredSampleRate(48_000)
            try session.setActive(true)
        } catch {
            publishError("microphone unavailable: \(error.localizedDescription)")
            return
        }
        
        let input = engine.inputNode
        let format = input.outputFormat(forBus: 0) // native rate
        
        // According to API ref 0 Hz when there is no input device
        guard format.sampleRate > 0 else {
            publishError("microphone unavailable")
            return
        }
        
        // Analysis parameters default in dsp-core (TunerConfig); only the rate is ours.
        let config = TunerConfig(sampleRate: UInt32(format.sampleRate))
        let hop = Int(config.hopSize)
        
        let grid = SpectrumGrid(sampleRate: Float(format.sampleRate), fftSize: Int(config.fftSize), minHz: config.minHz, maxHz: config.maxHz)
        
        // start() is only called from UO, so this is main thread, hence this hack is just telling the compiler that
        MainActor.assumeIsolated { self.grid = grid }
        
        var coreError: String?
        if tuner == nil || tunerRate != config.sampleRate {
            if calibrated.exchange(false, ordering: .relaxed) {
                MainActor.assumeIsolated { onCalibrationLost?() }
            }
            tuner = nil
            do {
                let t = try Tuner(config: config)
                try t.setTarget(midiNote: targetMidi.load(ordering: .relaxed))
                tuner = t
                tunerRate = config.sampleRate
            } catch {
                coreError = error.localizedDescription
            }
        }
        
        MainActor.assumeIsolated {
            ghost = tuner.flatMap { ghostFor($0, grid) }
            plotObserver?.ghostDidChange()
        }

        input.installTap(onBus: 0, bufferSize: AVAudioFrameCount(hop), format: format) {
            [ring, wake] buffer, _ in
            guard let channel = buffer.floatChannelData?[0] else {
                return
            }
            ring.write(channel, count: Int(buffer.frameLength))
            wake.signal()
        }
        do {
            try engine.start()
        } catch {
            input.removeTap(onBus: 0)
            publishError("microphone unavailable \(error.localizedDescription)")
            return
        }
        
        let error = coreError
        let stabilizer = PitchStabilizer(hopsPerSecond: format.sampleRate / Double(hop))
        let thread = Thread { [self, tuner] in
            dspLoop(tuner: tuner, hop: hop, grid: grid, stabilizer: stabilizer, coreError: error)
            finished.signal()
        }
        thread.name = "dsp"
        thread.qualityOfService = .userInteractive
        thread.start()
        dspThread = thread
    }
    
    private func dspLoop(tuner: Tuner?, hop: Int, grid: SpectrumGrid, stabilizer: PitchStabilizer, coreError: String?) {
        let hopBytes = hop * 4
        var pcm = Data(count: hopBytes)
        var noise = Data(count: Self.noiseHops * hopBytes)
        var noiseOffset = 0
        var noiseError: String?
        var hops = 0
        var dropped = 0
        var nextPlotNs: UInt64 = 0
        var stabilizedTarget = targetMidi.load(ordering: .relaxed)
        while !Thread.current.isCancelled {
            wake.wait()
            while ring.availbale >= hop {
                pcm.withUnsafeMutableBytes { ring.read(into: $0, count: hop)}
                hops += 1
                
                var runDone = false
                if learnRequested.load(ordering: .acquiring) {
                    if noiseOffset == 0 {
                        noiseError = nil
                    }
                    noise.replaceSubrange(noiseOffset..<noiseOffset + hopBytes, with: pcm)
                    noiseOffset += hopBytes
                    if noiseOffset == noise.count {
                        noiseError = learnNoise(tuner, noise)
                        if noiseError == nil {
                            calibrated.store(true, ordering: .relaxed)
                        }
                        noiseOffset = 0
                        runDone = true
                    }
                }
                
                var result: TunerResult?
                if let tuner {
                    do {
                        result = try tuner.process(pcm: pcm)
                    } catch {
                        dropped += 1
                    }
                }
                // Cents are relative to the target, so a new target starts a new reading.
                let target = targetMidi.load(ordering: .relaxed)
                if target != stabilizedTarget {
                    stabilizer.reset()
                    stabilizedTarget = target
                }
                let reading = stabilizer.update(pitchHz: result?.pitchHz ?? 0, cents: result?.cents ?? 0)
                published.store(Snapshot(result: result, reading: reading, hops: hops, dropped: dropped, coreError: coreError, noiseError: noiseError), ordering: .releasing)
                
                if let tuner, let result {
                    let now = DispatchTime.now().uptimeNanoseconds
                    if now >= nextPlotNs {
                        nextPlotNs = max(nextPlotNs + Self.plotPeriodNs, now)
                        publishPlot(result, grid: grid)
                    }
                }
                
                if runDone {
                    learnRequested.store(false, ordering: .releasing)
                }
            }
        }
    }
    
    ///nil on success, otherwise why learning failed.
    private func learnNoise(_ tuner: Tuner?, _ noise: Data) -> String? {
        guard let tuner else {
            return "core unavailable"
        }
        do {
            try tuner.learnNoise(pcm: noise)
            return nil
        } catch {
            return error.localizedDescription
        }
    }
    
    private func publishPlot(_ result: TunerResult, grid: SpectrumGrid) {
        result.spectrum.withUnsafeBytes {grid.downsample($0, into: plot.backSlot())}
        plot.publish()
        DispatchQueue.main.async { [self] in
            plotObserver?.plotDidUpdate()
        }
    }
    
    private func ghostFor(_ tuner: Tuner, _ grid: SpectrumGrid) -> [Float]? {
        guard let target = try? tuner.targetSpectrum() else { return nil }
        return [Float](unsafeUninitializedCapacity: SpectrumGrid.points) { out, count in
            target.withUnsafeBytes { grid.downsample($0, into: out) }
            count = SpectrumGrid.points
        }
    }
    
    func stop() {
        guard let thread = dspThread else { return }
        
        engine.inputNode.removeTap(onBus: 0)
        engine.stop()
        thread.cancel()
        wake.signal()
        finished.wait()
        dspThread = nil
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }
    
    @MainActor func setTarget(_ midi: Int32) {
        guard (0...127).contains(midi), midi != targetMidi.load(ordering: .relaxed) else { return }
        do {
            try tuner?.setTarget(midiNote: midi)
        } catch {
            return
        }
        targetMidi.store(midi, ordering: .relaxed)
        if let tuner, let grid {
            ghost = ghostFor(tuner, grid)
            plotObserver?.ghostDidChange()
        }
    }
    
    func requestNoiseLearning() { learnRequested.store(true, ordering: .releasing) }
    
    private func publishError(_ message: String) {
        published.store(Snapshot(result: nil, hops: 0, dropped: 0, coreError: message, noiseError: nil), ordering: .releasing)
    }
}
