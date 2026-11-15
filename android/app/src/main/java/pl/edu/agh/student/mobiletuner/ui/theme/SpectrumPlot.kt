package pl.edu.agh.student.mobiletuner.ui.theme

import androidx.compose.foundation.layout.Spacer
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import pl.edu.agh.student.mobiletuner.audio.PlotListener
import pl.edu.agh.student.mobiletuner.audio.TunerEngine
import kotlin.math.pow

// Component for the live spectrum over the target ghost. New frames invalidate only the draw
// never composition or layout.
@Composable
fun SpectrumPlot(engine: TunerEngine, modifier: Modifier = Modifier) {
    val frame = remember { mutableLongStateOf(0L) }
    DisposableEffect(engine) {
        // Snapshot state is written from any thread. Keep it this way so compose applies it on
        // next frame
        engine.plotListener = PlotListener { frame.longValue++ }
        onDispose { engine.plotListener = null }
    }
    val lineColor = MaterialTheme.colorScheme.primary
    val ghostColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f)
    val gridColor = MaterialTheme.colorScheme.outlineVariant

    Spacer(modifier.drawWithCache {
        val live = Path()
        val ghost = Path()
        var ghostSrc: FloatArray? = null
        val stroke = Stroke(width = 1.5.dp.toPx())
        val fill = lineColor.copy(alpha = 0.2f)
        val gridXs = (2..8).map { size.width * engine.grid.xOf(C0_HZ * 2f.pow(it)) }

        onDrawBehind {
            frame.longValue
            val g = engine.ghost
            if (g !== ghostSrc ) {
                ghostSrc = g
                ghost.rewind()
                if (g != null) trace(ghost, g, size)
            }
            live.rewind()
            trace(live, engine.plot.latest(), size)

            for (x in gridXs) drawLine(gridColor, Offset(x, 0f), Offset(x, size.height))
            drawPath(ghost, ghostColor, style = stroke)
            drawPath(live, fill)
            drawPath(live, lineColor, style = stroke)
        }
    })
}

// One path from bottom-left through every point to bottom-right
// Can be both filled and stroked
private fun trace(path: Path, yVec: FloatArray, size: Size) {
    val dx = size.width / (yVec.size - 1)
    path.moveTo(0f, size.height)
    for (i in yVec.indices) path.lineTo(i * dx, size.height * (1f - yVec[i]))
    path.lineTo(size.width, size.height)
}

private const val C0_HZ = 16.3516f
