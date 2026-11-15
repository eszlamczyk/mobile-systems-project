package pl.edu.agh.student.mobiletuner.audio

import java.util.concurrent.atomic.AtomicInteger

// Lock-free writer/reader handoff of fixed-size frames. Tuned for performance (no alloc after init)
class TripleBuffer(size: Int) {
    private val buffers = Array(3) { FloatArray(size) }

    //Index of the shared middle slot
    private val middle = AtomicInteger(1)
    private var back = 0
    private var front = 2

    fun back(): FloatArray = buffers[back]

    fun publish() {
        back = middle.getAndSet(back or FRESH) and INDEX
    }

    fun latest(): FloatArray {
        if (middle.get() and FRESH != 0) front = middle.getAndSet(front) and INDEX
        return buffers[front]
    }

    private companion object {
        const val INDEX = 3
        const val FRESH = 4
    }

}