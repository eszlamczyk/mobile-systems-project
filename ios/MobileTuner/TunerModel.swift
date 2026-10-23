import AVFoundation
import Observation

@Observable
final class TunerModel {
    let engine = TunerEngine()
    var granted = AVAudioApplication.shared.recordPermission == .granted

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
