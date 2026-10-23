package pl.edu.agh.student.mobiletuner

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import pl.edu.agh.student.mobiletuner.audio.TunerEngine

class TunerViewModel(app: Application) : AndroidViewModel(app) {
    val engine = TunerEngine(app)

    override fun onCleared() {
        engine.close()
    }
}
