import Atomics

nonisolated final class SPSCRing : @unchecked Sendable {
    private let buf: UnsafeMutablePointer<Float>
    private let mask: Int
    private let head = ManagedAtomic<Int>(0) // writen by producer
    private let tail = ManagedAtomic<Int>(0) // written by consumer
    
    init(capacity: Int) {
        precondition(capacity.nonzeroBitCount == 1, "capacity must be a power of two")
        buf = .allocate(capacity: capacity)
        buf.initialize(repeating: 0, count: capacity)
        mask = capacity - 1
    }
    
    deinit {
        buf.deallocate()
    }
    
    var availbale: Int {
        head.load(ordering: .acquiring) - tail.load(ordering: .relaxed)
    }
    
    @discardableResult
    func write(_ src: UnsafePointer<Float>, count: Int) -> Bool {
        let h = head.load(ordering: .relaxed)
        let t = tail.load(ordering: .acquiring)
        guard mask + 1 - (h - t) >= count else { return false }
        for i in 0..<count {
            buf[(h+i) & mask] = src[i]
        }
        head.store(h + count, ordering: .releasing)
        return true
    }
    
    func read(into dst: UnsafeMutableRawBufferPointer, count: Int) {
        let t = tail.load(ordering: .relaxed)
        for i in 0..<count {
            dst.storeBytes(of: buf[(t + i) & mask], toByteOffset: i * 4, as: Float.self)
        }
        tail.store(t + count, ordering: .releasing)
    }
}
