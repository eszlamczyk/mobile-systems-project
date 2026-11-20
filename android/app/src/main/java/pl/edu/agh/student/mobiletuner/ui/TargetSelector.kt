package pl.edu.agh.student.mobiletuner.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import pl.edu.agh.student.mobiletuner.Preset
import pl.edu.agh.student.mobiletuner.TuningPreset
import pl.edu.agh.student.mobiletuner.ui.theme.MobileTunerTheme

@Composable
fun TargetSelector(
    presets: List<TuningPreset>,
    preset: TuningPreset?,
    onSelect: (TuningPreset?) -> Unit,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }
    val label = preset?.name ?: "Single note"
    val contentColor = MaterialTheme.colorScheme.onSurface

    Box(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f))
                .clickable(role = Role.DropdownList) { expanded = true }
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .semantics(mergeDescendants = true) {
                    contentDescription = "Target: $label"
                },
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = contentColor,
            )
            Spacer(modifier = Modifier.weight(1f))
            ChevronUpDownIcon(color = contentColor)
        }

        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
        ) {
            for (item in presets) {
                DropdownMenuItem(
                    text = { Text(item.name) },
                    trailingIcon = if (item == preset) {
                        { CheckIcon(color = MaterialTheme.colorScheme.primary) }
                    } else {
                        null
                    },
                    onClick = {
                        expanded = false
                        onSelect(item)
                    },
                )
            }
            HorizontalDivider()
            DropdownMenuItem(
                text = { Text("Single note") },
                trailingIcon = if (preset == null) {
                    { CheckIcon(color = MaterialTheme.colorScheme.primary) }
                } else {
                    null
                },
                onClick = {
                    expanded = false
                    onSelect(null)
                },
            )
        }
    }
}

@Composable
private fun ChevronUpDownIcon(color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier = modifier.size(16.dp)) {
        val stroke = Stroke(
            width = 1.75.dp.toPx(),
            cap = StrokeCap.Round,
            join = StrokeJoin.Round,
        )
        val w = size.width
        val h = size.height
        val up = Path().apply {
            moveTo(w * 0.25f, h * 0.38f)
            lineTo(w * 0.5f, h * 0.14f)
            lineTo(w * 0.75f, h * 0.38f)
        }
        val down = Path().apply {
            moveTo(w * 0.25f, h * 0.62f)
            lineTo(w * 0.5f, h * 0.86f)
            lineTo(w * 0.75f, h * 0.62f)
        }
        drawPath(up, color = color, style = stroke)
        drawPath(down, color = color, style = stroke)
    }
}

@Composable
private fun CheckIcon(color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier = modifier.size(16.dp)) {
        val stroke = Stroke(
            width = 2.dp.toPx(),
            cap = StrokeCap.Round,
            join = StrokeJoin.Round,
        )
        val w = size.width
        val h = size.height
        val path = Path().apply {
            moveTo(w * 0.18f, h * 0.52f)
            lineTo(w * 0.42f, h * 0.76f)
            lineTo(w * 0.84f, h * 0.26f)
        }
        drawPath(path, color = color, style = stroke)
    }
}

@Preview(showBackground = true)
@Composable
private fun TargetSelectorPreview() {
    MobileTunerTheme {
        TargetSelector(
            presets = Preset.entries.map { it.tuning },
            preset = Preset.eStandard.tuning,
            onSelect = {},
            modifier = Modifier.padding(16.dp),
        )
    }
}
