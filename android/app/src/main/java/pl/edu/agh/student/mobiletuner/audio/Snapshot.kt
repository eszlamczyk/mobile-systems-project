package pl.edu.agh.student.mobiletuner.audio

import uniffi.dsp_core.TunerResult

data class Snapshot(
    val result: TunerResult?,
    val reading: PitchStabilizer.Reading? = null,
    val hops: Long,
    val dropped: Long,
    val coreError: String?,
    val noiseError: String? = null,
) {
    companion object {
        val EMPTY = Snapshot(null, null, 0, 0, null, null)
    }
}
