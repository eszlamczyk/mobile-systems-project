package pl.edu.agh.student.mobiletuner.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import pl.edu.agh.student.mobiletuner.Note
import pl.edu.agh.student.mobiletuner.Preset
import pl.edu.agh.student.mobiletuner.ui.theme.MobileTunerTheme

@Composable
fun StringButtons(
    notes: List<Int>,
    selected: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        notes.forEachIndexed { index, midi ->
            NoteChip(
                label = Note.name(midi),
                isSelected = index == selected,
                onClick = { onSelect(index) },
                modifier = Modifier
                    .weight(1f)
                    .semantics {
                        contentDescription = "String ${notes.size - index}, ${Note.fullName(midi)}"
                    },
            )
        }
    }
}

@Composable
fun NoteChip(
    label: String,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val background = if (isSelected) {
        MaterialTheme.colorScheme.primary
    } else {
        MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f)
    }
    val foreground = if (isSelected) {
        MaterialTheme.colorScheme.onPrimary
    } else {
        MaterialTheme.colorScheme.onSurface
    }

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(background)
            .clickable(role = Role.Button, onClick = onClick)
            .semantics { selected = isSelected }
            .defaultMinSize(minHeight = 44.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            fontSize = 20.sp,
            fontWeight = FontWeight.SemiBold,
            color = foreground,
            maxLines = 1,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun StringButtonsPreview() {
    MobileTunerTheme {
        StringButtons(
            notes = Preset.halfStepDown.tuning.notes,
            selected = 0,
            onSelect = {},
            modifier = Modifier.padding(16.dp),
        )
    }
}
