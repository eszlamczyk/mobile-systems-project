package pl.edu.agh.student.mobiletuner.audio

import kotlin.math.abs

/**
 * Turns per-hop pitch readings into a steady value for the tuning panel.
 *
 * Raw YIN readings jump during a pluck (the attack is inharmonic) and vanish as soon
 * as the note decays below the gate. Three rules make the display usable:
 * - a reading is shown only after [MIN_AGREEING] consecutive hops agree within [JUMP_CENTS],
 *   so single-hop outliers never reach the screen;
 * - the shown value is the median of the last [WINDOW] agreeing hops;
 * - after detection stops, the last value is held for [holdHops] hops (docs/FFT.md, gate).
 *
 * DSP thread only. Preallocates all scratch arrays on construction.
 */
class PitchStabilizer(hopsPerSecond: Double, holdSeconds: Double = 1.0) {
    data class Reading(
        val cents: Float,
        val pitchHz: Float,
        /** False while holding the last reading after detection stopped. */
        val isLive: Boolean,
    )

    private val holdHops = maxOf(1, (hopsPerSecond * holdSeconds).toInt())
    private val cents = FloatArray(WINDOW)
    private val hertz = FloatArray(WINDOW)
    private val scratch = FloatArray(WINDOW)
    private var count = 0
    private var next = 0
    private var lastCents = 0f
    private var shown: Reading? = null
    private var silentHops = 0

    fun reset() {
        count = 0
        next = 0
        shown = null
        silentHops = 0
    }

    fun update(pitchHz: Float, centsValue: Float): Reading? {
        if (pitchHz <= 0f) {
            count = 0
            silentHops++
            val last = shown
            if (last == null || silentHops > holdHops) {
                shown = null
                return null
            }
            shown = last.copy(isLive = false)
            return shown
        }

        if (count > 0 && abs(centsValue - lastCents) > JUMP_CENTS) {
            count = 0
        }
        lastCents = centsValue
        cents[next] = centsValue
        hertz[next] = pitchHz
        next = (next + 1) % WINDOW
        count = minOf(count + 1, WINDOW)

        if (count >= MIN_AGREEING) {
            silentHops = 0
            shown = Reading(cents = median(cents), pitchHz = median(hertz), isLive = true)
        } else {
            val last = shown
            if (last != null) {
                // A new note is settling: keep showing the previous one, but no longer as live.
                silentHops++
                shown = if (silentHops <= holdHops) last.copy(isLive = false) else null
            }
        }
        return shown
    }

    private fun median(ring: FloatArray): Float {
        for (i in 0 until count) {
            scratch[i] = ring[(next - 1 - i + WINDOW) % WINDOW]
        }
        // Insertion sort: at most five elements.
        for (i in 1 until count) {
            val value = scratch[i]
            var j = i - 1
            while (j >= 0 && scratch[j] > value) {
                scratch[j + 1] = scratch[j]
                j--
            }
            scratch[j + 1] = value
        }
        return if (count % 2 == 1) {
            scratch[count / 2]
        } else {
            (scratch[count / 2 - 1] + scratch[count / 2]) / 2f
        }
    }

    companion object {
        const val WINDOW = 5
        const val MIN_AGREEING = 3
        const val JUMP_CENTS = 50f
    }
}
