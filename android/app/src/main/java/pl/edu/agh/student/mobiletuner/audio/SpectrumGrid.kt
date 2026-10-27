package pl.edu.agh.student.mobiletuner.audio

import java.nio.FloatBuffer
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.log2
import kotlin.math.pow

// Fixed log-freq grid of [POINTS] points over the band. Refer to docs/SPECTRUM_GRID.md
class SpectrumGrid(sampleRate: Int, fftSize: Int, private val minHz: Float, private val maxHz: Float) {
    private val bins = fftSize / 2
    private val first = IntArray(POINTS)
    private val last = IntArray(POINTS)
    private val pos = FloatArray(POINTS)
    private val lo: Int
    private val hi: Int

    init {
        val hzPerBin = sampleRate.toFloat() / fftSize
        lo = ceil(minHz / hzPerBin).toInt()
        hi = minOf(floor(maxHz / hzPerBin).toInt(), bins - 1)
        for (i in 0 until POINTS) {
            first[i] = maxOf(ceil(hzAt(i - 0.5f) / hzPerBin).toInt(), lo)
            last[i] = minOf(floor(hzAt(i + 0.5f) / hzPerBin).toInt(), hi)
            pos[i] = (hzAt(i.toFloat()) / hzPerBin).coerceIn(lo.toFloat(), hi.toFloat())
        }
    }

    private fun hzAt(i: Float): Float = minHz * (maxHz/minHz).pow(i / (POINTS - 1))

    // horizontal position - normalized
    fun xOf(hz: Float): Float = log2(hz / minHz) / log2(maxHz / minHz)

    fun downsample(src: FloatBuffer, out: FloatArray) {
        for (i in 0 until POINTS) {
            val db = if (last[i] >= first[i]) {
                var m = src.get(first[i])
                for (k in first[i] + 1..last[i]) m = maxOf(m, src.get(k))
                m
            } else {
                catmullRom(src, pos[i])
            }
            out[i] = normalise(db)
        }
    }

    // TODO: REMOVE - Placeholder unstill dsp-core implements target_spectrum
    fun placeholderGhost(midi: Int, out: FloatArray) {
        val f0 = 440f * 2f.pow((midi - 69) / 12f)
        for (i in 0 until POINTS) {
            val f = hzAt(i.toFloat())
            var db = DB_MIN
            var n = 1
            while (n * f0 <= maxHz) {
                val semis = 12f * log2(f/(n*f0))
                db = maxOf(db, PLACEHOLDER_DB - 20f * log10(n.toFloat()) - 40f * semis * semis)
                n++
            }
            out[i] = normalise(db) }
    }

    // https://en.wikipedia.org/wiki/Catmull%E2%80%93Rom_spline
    private fun catmullRom(src: FloatBuffer, x: Float): Float {
        val k = x.toInt()
        val t = x - k
        val p0 = src.get((k - 1).coerceIn(lo, hi))
        val p1 = src.get(k.coerceIn(lo, hi))
        val p2 = src.get((k + 1).coerceIn(lo, hi))
        val p3 = src.get((k + 2).coerceIn(lo, hi))

        return p1 + 0.5f * t * (p2 - p0 + t * (2 * p0 - 5 * p1 + 4 * p2 - p3 + t * (3 * (p1 - p2) + p3 - p0)))
    }

    private fun normalise(db: Float) = ((db - DB_MIN) / (DB_MAX - DB_MIN)).coerceIn(0f, 1f)

    companion object {
        const val POINTS = 400
        const val DB_MIN = -100f
        const val DB_MAX = 0
        private const val PLACEHOLDER_DB = -20f
    }
}