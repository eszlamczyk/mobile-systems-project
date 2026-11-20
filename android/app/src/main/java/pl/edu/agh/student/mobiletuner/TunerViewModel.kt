package pl.edu.agh.student.mobiletuner

import android.Manifest
import android.annotation.SuppressLint
import android.app.Application
import android.content.pm.PackageManager
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.content.ContextCompat
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import pl.edu.agh.student.mobiletuner.audio.TunerEngine

class TunerViewModel(app: Application) : AndroidViewModel(app) {
    val engine = TunerEngine(app)

    val presets: List<TuningPreset> = Preset.entries.map { it.tuning }

    /** Null means single-note mode. */
    var preset by mutableStateOf<TuningPreset?>(presets.firstOrNull())
        private set

    var stringIndex by mutableIntStateOf(0)
        private set

    var singleNote by mutableIntStateOf(69)
        private set

    val targetMidi: Int
        get() {
            val current = preset
            return if (current != null && stringIndex in current.notes.indices) {
                current.notes[stringIndex]
            } else {
                singleNote
            }
        }

    var micGranted by mutableStateOf(
        ContextCompat.checkSelfPermission(app, Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
    )
        private set

    var micDenied by mutableStateOf(false)
        private set

    var toast by mutableStateOf<String?>(null)
        private set

    private var toastJob: Job? = null

    init {
        applyTarget()
    }

    fun selectPreset(newPreset: TuningPreset?) {
        preset = newPreset
        stringIndex = 0
        applyTarget()
    }

    fun selectString(index: Int) {
        stringIndex = index
        applyTarget()
    }

    fun selectSingleNote(midi: Int) {
        singleNote = midi.coerceIn(0, 127)
        applyTarget()
    }

    fun onPermissionResult(granted: Boolean) {
        micGranted = granted
        micDenied = !granted
    }

    fun refreshPermission() {
        val granted = ContextCompat.checkSelfPermission(
            getApplication(),
            Manifest.permission.RECORD_AUDIO,
        ) == PackageManager.PERMISSION_GRANTED
        if (granted) {
            micGranted = true
            micDenied = false
        }
    }

    fun showToast(message: String) {
        toast = message
        toastJob?.cancel()
        toastJob = viewModelScope.launch {
            delay(3_000L)
            toast = null
        }
    }

    @SuppressLint("MissingPermission")
    fun start() {
        if (!micGranted) return
        engine.start()
    }

    fun stop() {
        engine.stop()
    }

    /** Records ~1 s of room noise so spectral subtraction can remove it from the plot. */
    fun calibrate() {
        engine.requestNoiseLearning()
    }

    private fun applyTarget() {
        engine.setTarget(targetMidi)
    }

    override fun onCleared() {
        engine.close()
    }
}
