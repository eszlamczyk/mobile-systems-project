import Atomics
import DspCore

nonisolated final class Snapshot: AtomicReference, @unchecked Sendable {
    let result: TunerResult?
    let hops: Int
    let dropped: Int
    let coreError: String?
    let noiseError: String?
    
    init(result: TunerResult?, hops: Int, dropped: Int, coreError: String?, noiseError: String?) {
        self.result = result
        self.hops = hops
        self.dropped = dropped
        self.coreError = coreError
        self.noiseError = noiseError
    }
    
    static let empty = Snapshot(result: nil, hops: 0, dropped: 0, coreError: nil, noiseError: nil)
}
