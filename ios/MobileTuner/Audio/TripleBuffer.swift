import Atomics

/// Lock-free writer/reader handoff of fixed-size frames. Tuned for performance (no alloc after init)
nonisolated final class TripleBuffer: @unchecked Sendable {
    private static let index = 3
    private static let fresh = 4
    
    let size: Int
    private let buf: UnsafeMutableBufferPointer<Float>
    
    // Index of the shared middle slot
    private let middle = ManagedAtomic<Int>(1)
    private var back = 0
    private var front = 2
    
    init(size: Int) {
        self.size = size
        buf = .allocate(capacity: 3 * size)
        buf.initialize(repeating: 0)
    }
    
    deinit {
        buf.deallocate()
    }
    
    func backSlot() -> UnsafeMutableBufferPointer<Float> {
        slot(back)
    }
    
    func publish() {
        back = middle.exchange(back | Self.fresh, ordering: .acquiringAndReleasing) & Self.index
    }
    
    func latest() -> UnsafeBufferPointer<Float> {
        if middle.load(ordering: .relaxed) & Self.fresh != 0 {
            front = middle.exchange(front, ordering: .acquiringAndReleasing) & Self.index
        }
        return UnsafeBufferPointer(slot(front))
    }
    
    private func slot (_ i: Int) -> UnsafeMutableBufferPointer<Float> {
        UnsafeMutableBufferPointer(rebasing: buf[i * size ..< (i+1) * size])
    }
}
