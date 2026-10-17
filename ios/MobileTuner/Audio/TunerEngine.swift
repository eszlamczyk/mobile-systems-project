import AVFoundation
import Atomics
import DspCore

nonisolated final class TunerEngine: @unchecked Sendable {
    static let hop = 1024
    static let noiseHops = 47
    
    private let engine = AVAudioEngine()
    private let ring = SPSCRing(capacity: 1 << 14 ) // ~340 ms at 48kHz
    
    private let wake = DispatchSemaphore(value: 0)
    private let finished = DispatchSemaphore(value: 0)
    
    private let published = ManagedAtomic<Snapshot>(.empty)
    private let learnRequested = ManagedAtomic<Bool>(false)
    private var dspThread: Thread?
    private var tuner: Tuner?
    private var targetMidi: Int32 = 69
    
    var latest: Snapshot {
        published.load(ordering: .acquiring)
    }
    
    var isLearningNoise: Bool {
        learnRequested.load(ordering: .acquiring)
    }
    
    func start() {
        guard dspThread == nil else { return }
        
        let session = AVAudioSession.sharedInstance()
        
        do {
            try session.setCategory(.record, mode: .measurement)
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
        
        var coreError: String?
        if tuner == nil {
            do {
                let t = try Tuner(config: TunerConfig(
                    sampleRate: UInt32(format.sampleRate),
                    fftSize: 4096,
                    hopSize: UInt32(Self.hop),
                    minHz: 50,
                    maxHz: 5000
                ))
                try t.setTarget(midiNote: targetMidi)
                tuner = t
            } catch {
                coreError = error.localizedDescription
            }
        }
        
        input.installTap(onBus: 0, bufferSize: AVAudioFrameCount(Self.hop), format: format) {
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
        let thread = Thread { [self, tuner] in
            dspLoop(tuner: tuner, coreError: error)
            finished.signal()
        }
        thread.name = "dsp"
        thread.qualityOfService = .userInteractive
        thread.start()
        dspThread = thread
    }
    
    private func dspLoop(tuner: Tuner?, coreError: String?) {
        let hopBytes = Self.hop * 4
        var pcm = Data(count: hopBytes)
        var noise = Data(count: Self.noiseHops * hopBytes)
        var noiseOffset = 0
        var noiseError: String?
        var hops = 0
        var dropped = 0
        while !Thread.current.isCancelled {
            wake.wait()
            while ring.availbale >= Self.hop {
                pcm.withUnsafeMutableBytes { ring.read(into: $0, count: Self.hop)}
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
                published.store(Snapshot(result: result, hops: hops, dropped: dropped, coreError: coreError, noiseError: noiseError), ordering: .releasing)
                
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
    
    func setTarget(_ midi: Int32) {
        targetMidi = midi
        try? tuner?.setTarget(midiNote: midi)
    }
    
    func requestNoiseLearning() { learnRequested.store(true, ordering: .releasing) }
    
    private func publishError(_ message: String) {
        published.store(Snapshot(result: nil, hops: 0, dropped: 0, coreError: message, noiseError: nil), ordering: .releasing)
    }
}
