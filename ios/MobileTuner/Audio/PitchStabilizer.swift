/// Turns per-hop pitch readings into a steady value for the tuning panel.
///
/// Raw YIN readings jump during a pluck (the attack is inharmonic) and vanish as soon
/// as the note decays below the gate. Three rules make the display usable:
/// - a reading is shown only after `minAgreeing` consecutive hops agree within `jumpCents`,
///   so single-hop outliers never reach the screen;
/// - the shown value is the median of the last `window` agreeing hops;
/// - after detection stops, the last value is held for `holdHops` hops (docs/FFT.md, gate).
///
/// DSP thread only: created on the main thread, then handed to the DSP thread and never shared.
/// Allocates once in `init`.
nonisolated final class PitchStabilizer: @unchecked Sendable {
    struct Reading: Equatable {
        let cents: Float
        let pitchHz: Float
        /// False while holding the last reading after detection stopped.
        let isLive: Bool
    }

    static let window = 5
    static let minAgreeing = 3
    static let jumpCents: Float = 50

    private let holdHops: Int
    private let cents = UnsafeMutablePointer<Float>.allocate(capacity: window)
    private let hertz = UnsafeMutablePointer<Float>.allocate(capacity: window)
    private let scratch = UnsafeMutablePointer<Float>.allocate(capacity: window)
    private var count = 0
    private var next = 0
    private var lastCents: Float = 0
    private var shown: Reading?
    private var silentHops = 0

    /// `holdSeconds` of hold at `hopsPerSecond` hops per second.
    init(hopsPerSecond: Double, holdSeconds: Double = 1) {
        holdHops = max(1, Int(hopsPerSecond * holdSeconds))
    }

    deinit {
        cents.deallocate()
        hertz.deallocate()
        scratch.deallocate()
    }

    /// Forgets everything, for example when the target note changes.
    func reset() {
        count = 0
        next = 0
        shown = nil
        silentHops = 0
    }

    /// Feeds one hop. `pitchHz <= 0` means nothing was detected.
    func update(pitchHz: Float, cents value: Float) -> Reading? {
        guard pitchHz > 0 else {
            count = 0
            silentHops += 1
            guard let last = shown, silentHops <= holdHops else {
                shown = nil
                return nil
            }
            shown = Reading(cents: last.cents, pitchHz: last.pitchHz, isLive: false)
            return shown
        }

        if count > 0, abs(value - lastCents) > Self.jumpCents {
            count = 0
        }
        lastCents = value
        cents[next] = value
        hertz[next] = pitchHz
        next = (next + 1) % Self.window
        count = min(count + 1, Self.window)

        if count >= Self.minAgreeing {
            silentHops = 0
            shown = Reading(cents: median(cents), pitchHz: median(hertz), isLive: true)
        } else if let last = shown {
            // A new note is settling: keep showing the previous one, but no longer as live.
            silentHops += 1
            shown = silentHops <= holdHops ? Reading(cents: last.cents, pitchHz: last.pitchHz, isLive: false) : nil
        }
        return shown
    }

    /// Median of the newest `count` entries of a ring of `window` values.
    private func median(_ ring: UnsafeMutablePointer<Float>) -> Float {
        for i in 0..<count {
            scratch[i] = ring[(next - 1 - i + Self.window) % Self.window]
        }
        // Insertion sort: at most five elements.
        for i in 1..<count {
            let value = scratch[i]
            var j = i - 1
            while j >= 0, scratch[j] > value {
                scratch[j + 1] = scratch[j]
                j -= 1
            }
            scratch[j + 1] = value
        }
        return count % 2 == 1 ? scratch[count / 2] : (scratch[count / 2 - 1] + scratch[count / 2]) / 2
    }
}
