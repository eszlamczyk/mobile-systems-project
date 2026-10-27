import AVFoundation
import Observation

@Observable
final class TunerModel {
    let engine = TunerEngine()
    var granted = AVAudioApplication.shared.recordPermission == .granted
    private(set) var toast: String?
    private var toastTask: Task<Void, Never>?

    init() {
        engine.onCalibrationLost = { [weak self] in
            self?.showToast("Audio device changed, calibrate again")
        }
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
        if granted {
            return
        }
        granted = await AVAudioApplication.requestRecordPermission()
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
            engine.stop()
        case .ended:
            let rawOptions = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt ?? 0
            let options = AVAudioSession.InterruptionOptions(rawValue: rawOptions)
            if granted && options.contains(.shouldResume) {
                engine.start()
            }
        @unknown default:
            break
        }
    }
}
