package pl.edu.agh.student.mobiletuner.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import pl.edu.agh.student.mobiletuner.Note
import pl.edu.agh.student.mobiletuner.ui.theme.MobileTunerTheme

@Composable
fun NotePicker(
    midi: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val currentOctave = Note.octave(midi).coerceIn(0, 8)
    val pitchClass = ((midi % 12) + 12) % 12

    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        for (row in 0..1) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                for (col in 0 until 6) {
                    val pc = row * 6 + col
                    val note = (currentOctave + 1) * 12 + pc
                    NoteChip(
                        label = Note.names[pc],
                        isSelected = note == midi,
                        onClick = { onSelect(note) },
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                text = "Octave $currentOctave",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )

            Row(
                modifier = Modifier
                    .clip(RoundedCornerShape(8.dp))
                    .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f)),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                StepperButton(
                    symbol = "\u2212",
                    description = "Decrease octave",
                    enabled = currentOctave > 0,
                    onClick = { onSelect(currentOctave * 12 + pitchClass) },
                )
                VerticalDivider(
                    modifier = Modifier.height(20.dp),
                    color = MaterialTheme.colorScheme.outlineVariant,
                )
                StepperButton(
                    symbol = "+",
                    description = "Increase octave",
                    enabled = currentOctave < 8,
                    onClick = { onSelect((currentOctave + 2) * 12 + pitchClass) },
                )
            }
        }
    }
}

@Composable
private fun StepperButton(
    symbol: String,
    description: String,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Box(
        modifier = Modifier
            .width(48.dp)
            .height(34.dp)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .alpha(if (enabled) 1f else 0.38f)
            .semantics { contentDescription = description },
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = symbol,
            fontSize = 20.sp,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun NotePickerPreview() {
    MobileTunerTheme {
        NotePicker(
            midi = 69,
            onSelect = {},
            modifier = Modifier.padding(16.dp),
        )
    }
}
