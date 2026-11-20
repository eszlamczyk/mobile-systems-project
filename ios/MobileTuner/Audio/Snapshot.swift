import Atomics
import DspCore

nonisolated final class Snapshot: AtomicReference, @unchecked Sendable {
    let result: TunerResult?
    /// What the tuning panel shows: `result` stabilized and held over gaps, nil when nothing to show.
    let reading: PitchStabilizer.Reading?
    let hops: Int
    let dropped: Int
    let coreError: String?
    let noiseError: String?
    
    init(result: TunerResult?, reading: PitchStabilizer.Reading? = nil, hops: Int, dropped: Int, coreError: String?, noiseError: String?) {
        self.result = result
        self.reading = reading
        self.hops = hops
        self.dropped = dropped
        self.coreError = coreError
        self.noiseError = noiseError
    }
    
    static let empty = Snapshot(result: nil, hops: 0, dropped: 0, coreError: nil, noiseError: nil)
}
