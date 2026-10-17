package pl.edu.agh.student.mobiletuner.audio

import uniffi.dsp_core.TunerResult

data class Snapshot(val result: TunerResult?, val hops: Long, val dropped: Long, val coreError: String?) {
    companion object { val EMPTY = Snapshot(null, 0, 0, null)}
}
