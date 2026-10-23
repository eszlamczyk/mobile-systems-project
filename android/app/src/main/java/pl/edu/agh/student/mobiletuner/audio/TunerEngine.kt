package pl.edu.agh.student.mobiletuner.audio

import android.Manifest
import android.content.Context
import android.media.AudioFormat
import android.media.AudioFormat.CHANNEL_IN_MONO
import android.media.AudioFormat.ENCODING_PCM_FLOAT
import android.media.AudioManager
import android.media.AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED
import android.media.AudioRecord
import android.media.MediaRecorder.AudioSource.UNPROCESSED
import android.media.MediaRecorder.AudioSource.VOICE_RECOGNITION
import android.os.Process
import androidx.annotation.RequiresPermission
import uniffi.dsp_core.Tuner
import uniffi.dsp_core.TunerConfig
import java.nio.ByteBuffer
import java.nio.ByteOrder

class TunerEngine(context: Context) : AutoCloseable {
    @Volatile var latest = Snapshot.EMPTY
        private set

    @Volatile private var running = false
    @Volatile private var tuner: Tuner? = null
    @Volatile private var targetMidi = 69

    @Volatile private var learnRequested = false
    val isLearningNoise: Boolean
        get() = learnRequested

    @Volatile var noiseError: String? = null
        private set

    private var thread: Thread? = null

    // Analysis parameters default in dsp-core (TunerConfig); only the rate is ours.
    private val config = TunerConfig(sampleRate = SAMPLE_RATE_HZ.toUInt())
    private val hopBytes = config.hopSize.toInt() * 4
    private val pcm = ByteBuffer.allocateDirect(hopBytes).order(ByteOrder.nativeOrder())
    private val noise = ByteBuffer.allocateDirect(NOISE_HOPS * hopBytes).order(ByteOrder.nativeOrder())

    private val audioManager = context.getSystemService(AudioManager::class.java)

    @RequiresPermission(Manifest.permission.RECORD_AUDIO)
    fun start() {
        if (running) return
        val record = AudioRecord.Builder()
            .setAudioSource(if (unprocessedSupported()) UNPROCESSED else VOICE_RECOGNITION)
            .setAudioFormat(AudioFormat.Builder()
                    .setEncoding(ENCODING_PCM_FLOAT)
                    .setSampleRate(SAMPLE_RATE_HZ)
                    .setChannelMask(CHANNEL_IN_MONO)
                    .build())
            .setBufferSizeInBytes(
                maxOf(
                    AudioRecord.getMinBufferSize(
                        SAMPLE_RATE_HZ, CHANNEL_IN_MONO, ENCODING_PCM_FLOAT
                    ),
                    hopBytes * 4))
            .build()
        // Fails when another app holds the mic; startRecording() would then throw on the DSP thread.
        if (record.state != AudioRecord.STATE_INITIALIZED) {
            record.release()
            latest = Snapshot(null, 0, 0, "microphone unavailable")
            return
        }

        var coreError: String? = null
        if (tuner == null) tuner = try{
            Tuner(config).also { it.setTarget(targetMidi) }
        } catch (e: Exception) {
            coreError = e.message
            null
        }
        // A learning run cut short by stop() starts over rather than mixing two sessions.
        noise.clear()
        running = true
        thread = Thread({ loop(record, coreError)}, "dsp").apply { start() }
    }

    private fun loop(record: AudioRecord, coreError: String?) {
        Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
        var hops = 0L
        var dropped = 0L
        record.startRecording()
        while (running) {
            // read() never moves position; it must stay 0 (the UniFFI converter ignores it)
            if (record.read(pcm, hopBytes, AudioRecord.READ_BLOCKING) != hopBytes) continue
            hops++
            val t = tuner
            if (learnRequested && collectNoiseHop(t)) {
                learnRequested = false
            }
            val result = if (t == null) null else try {
                t.process(pcm)
            } catch (e: Exception) {
                dropped++
                null
            }

            latest = Snapshot(result, hops, dropped, coreError)
        }
        record.stop()
        record.release()
    }

    private fun collectNoiseHop(t: Tuner?): Boolean {
        noise.put(pcm)
        pcm.rewind()
        if (noise.hasRemaining()) return false
        noise.rewind()
        noiseError = if (t == null) {
            "core unavailable"
        } else try {
            t.learnNoise(noise)
            null
        } catch (e: Exception) {
            // The previous profile stays; the user can calibrate again.
            e.message ?: e.javaClass.simpleName
        }
        noise.clear()
        return true
    }

    fun requestNoiseLearning() {
        noiseError = null
        learnRequested = true
    }

    private fun unprocessedSupported(): Boolean =
        audioManager?.getProperty(PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED) == "true"


    fun setTarget(midi: Int) {
        targetMidi=midi
        runCatching { tuner?.setTarget(midi) }
    }

    fun stop() {
        running = false
        thread?.join()
        thread = null
    }

    override fun close() {
        stop()
        tuner?.close()
        tuner = null
    }

    companion object {
        // Requested explicitly, so AudioRecord either runs at it or fails to initialise.
        const val SAMPLE_RATE_HZ = 48000
        // ~1 s of silence at 48 kHz with the default hop
        const val NOISE_HOPS = 47
    }
}