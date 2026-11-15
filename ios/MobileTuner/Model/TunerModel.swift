import AVFoundation
import Observation

@Observable
final class TunerModel {
    /// Exposed only for `SpectrumPlot`, which the engine drives directly (frames bypass SwiftUI).
    let engine = TunerEngine()

    private(set) var micGranted: Bool
    private(set) var micDenied: Bool
    /// Short message shown at the bottom of the screen for 3 s.
    private(set) var toast: String?
    private var toastTask: Task<Void, Never>?

    let presets: [Preset]
    /// nil means single-note mode.
    var preset: Preset? {
        didSet { stringIndex = 0; applyTarget() }
    }
    var stringIndex = 0 {
        didSet { applyTarget() }
    }
    var singleNote: Int32 = 69 {
        didSet { applyTarget() }
    }
    var targetMidi: Int32 {
        if let preset, preset.notes.indices.contains(stringIndex) {
            return preset.notes[stringIndex]
        }
        return singleNote
    }

    /// Latest DSP result. Not observable (written by the DSP thread), so views poll it, at most 30 times per second.
    var latest: Snapshot { engine.latest }
    /// Not observable either, same as `latest`.
    var isCalibrating: Bool { engine.isLearningNoise }

    init(presets: [Preset] = Preset.all) {
        self.presets = presets
        preset = presets.first
        let permission = AVAudioApplication.shared.recordPermission
        micGranted = permission == .granted
        micDenied = permission == .denied
        engine.onCalibrationLost = { [weak self] in
            self?.showToast("Audio device changed, calibrate again")
        }
        applyTarget()
    }

    func showToast(_ message: String) {
        toast = message
        toastTask?.cancel()
        toastTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(3))
            guard !Task.isCancelled else { return }
            self?.toast = nil
        }
    }

    func requestPermission() async {
        if micGranted {
            return
        }
        micGranted = await AVAudioApplication.requestRecordPermission()
        micDenied = !micGranted
    }

    /// No-op without microphone permission.
    func start() {
        guard micGranted else { return }
        engine.start()
    }

    func stop() {
        engine.stop()
    }

    /// Records ~1 s of room noise so spectral subtraction can remove it from the plot.
    func calibrate() {
        engine.requestNoiseLearning()
    }

    private func applyTarget() {
        engine.setTarget(targetMidi)
    }

    func handleInterruption(_ note: Notification) {
        guard
            let rawType = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
            let type = AVAudioSession.InterruptionType(rawValue: rawType)
        else {
            return
        }
        switch type {
        case .began:
            stop()
        case .ended:
            let rawOptions = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0
            let options = AVAudioSession.InterruptionOptions(rawValue: rawOptions)
            if options.contains(.shouldResume) {
                start()
            }
        @unknown default:
            break
        }
    }
}
