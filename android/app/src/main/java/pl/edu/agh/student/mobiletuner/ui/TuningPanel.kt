package pl.edu.agh.student.mobiletuner.ui

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import pl.edu.agh.student.mobiletuner.Note
import pl.edu.agh.student.mobiletuner.ui.theme.MobileTunerTheme
import java.util.Locale
import kotlin.math.abs

@Composable
fun TuningPanel(
    targetMidi: Int,
    cents: Float?,
    pitchHz: Float?,
    modifier: Modifier = Modifier,
    isLive: Boolean = true,
) {
    val idleBackground = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f)
    val targetBackground = if (cents != null) tuningColor(cents) else idleBackground
    val background by animateColorAsState(
        targetValue = targetBackground,
        animationSpec = tween(durationMillis = 150, easing = FastOutSlowInEasing),
        label = "tuningBackground",
    )
    val panelAlpha by animateFloatAsState(
        targetValue = if (cents != null && !isLive) 0.6f else 1f,
        animationSpec = tween(durationMillis = 300, easing = FastOutSlowInEasing),
        label = "tuningLiveAlpha",
    )
    val foreground = if (cents == null) MaterialTheme.colorScheme.onSurface else Color.White
    val accessibility = accessibilityText(targetMidi, cents, pitchHz)

    Column(
        modifier = modifier
            .fillMaxWidth()
            .alpha(panelAlpha)
            .clip(RoundedCornerShape(20.dp))
            .background(background)
            .padding(vertical = 24.dp)
            .semantics(mergeDescendants = true) {
                contentDescription = accessibility
            },
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = Note.name(targetMidi),
                fontSize = 96.sp,
                lineHeight = 96.sp,
                fontWeight = FontWeight.Bold,
                color = foreground,
                modifier = Modifier.alignByBaseline(),
            )
            Text(
                text = Note.octave(targetMidi).toString(),
                fontSize = 28.sp,
                fontWeight = FontWeight.SemiBold,
                color = foreground.copy(alpha = 0.7f),
                modifier = Modifier.alignByBaseline(),
            )
        }

        Text(
            text = cents?.let { String.format(Locale.US, "%+.1f cents", it) } ?: "– cents",
            style = MaterialTheme.typography.titleLarge.copy(fontFeatureSettings = "tnum"),
            color = foreground,
        )

        Text(
            text = pitchHz?.let { String.format(Locale.US, "%.1f Hz", it) } ?: "– Hz",
            style = MaterialTheme.typography.bodyLarge.copy(fontFeatureSettings = "tnum"),
            color = foreground.copy(alpha = 0.8f),
        )
    }
}

fun tuningColor(cents: Float): Color {
    val t = ((abs(cents) - 2f) / 48f).coerceIn(0f, 1f)
    return Color.hsv(hue = 118.8f * (1f - t), saturation = 0.75f, value = 0.85f)
}

private fun accessibilityText(targetMidi: Int, cents: Float?, pitchHz: Float?): String {
    val target = "Target ${Note.fullName(targetMidi)}"
    if (cents == null) return "$target, no pitch detected"
    val direction = if (cents > 0f) "sharp" else "flat"
    val offset = "$target, ${String.format(Locale.US, "%.0f", abs(cents))} cents $direction"
    if (pitchHz == null) return offset
    return "$offset, ${String.format(Locale.US, "%.1f", pitchHz)} hertz"
}

@Preview(showBackground = true)
@Composable
private fun TuningPanelPreview() {
    MobileTunerTheme {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            TuningPanel(targetMidi = 38, cents = 1.2f, pitchHz = 73.5f)
            TuningPanel(targetMidi = 45, cents = -22f, pitchHz = 108.6f, isLive = false)
            TuningPanel(targetMidi = 64, cents = null, pitchHz = null)
        }
    }
}
