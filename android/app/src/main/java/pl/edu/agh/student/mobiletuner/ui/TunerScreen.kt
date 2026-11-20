package pl.edu.agh.student.mobiletuner.ui

import android.Manifest
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.LifecycleStartEffect
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.delay
import pl.edu.agh.student.mobiletuner.TunerViewModel
import pl.edu.agh.student.mobiletuner.audio.Snapshot
import pl.edu.agh.student.mobiletuner.ui.theme.SpectrumPlot

@Composable
fun TunerScreen(
    modifier: Modifier = Modifier,
    vm: TunerViewModel = viewModel(),
) {
    val permissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) { granted ->
        vm.onPermissionResult(granted)
    }

    LaunchedEffect(Unit) {
        if (!vm.micGranted) {
            permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
        }
    }

    LifecycleResumeEffect(Unit) {
        vm.refreshPermission()
        onPauseOrDispose {}
    }

    if (vm.micGranted) {
        LifecycleStartEffect(vm.engine) {
            vm.start()
            onStopOrDispose {
                vm.stop()
            }
        }
    }

    // The UI never waits on DSP: it samples the latest snapshot at most 30 times per second.
    var snap by remember { mutableStateOf(vm.engine.latest) }
    var calibrating by remember { mutableStateOf(vm.engine.isLearningNoise) }
    LaunchedEffect(vm.engine) {
        while (true) {
            snap = vm.engine.latest
            calibrating = vm.engine.isLearningNoise
            delay(33L)
        }
    }

    var lastToast by remember { mutableStateOf("") }
    vm.toast?.let { lastToast = it }

    Box(modifier = modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            TargetSelector(
                presets = vm.presets,
                preset = vm.preset,
                onSelect = vm::selectPreset,
            )

            val currentPreset = vm.preset
            if (currentPreset != null) {
                StringButtons(
                    notes = currentPreset.notes,
                    selected = vm.stringIndex,
                    onSelect = vm::selectString,
                )
            } else {
                NotePicker(
                    midi = vm.singleNote,
                    onSelect = vm::selectSingleNote,
                )
            }

            TuningPanel(
                targetMidi = vm.targetMidi,
                cents = snap.reading?.cents,
                pitchHz = snap.reading?.pitchHz,
                isLive = snap.reading?.isLive ?: false,
            )

            StatusLine(micDenied = vm.micDenied, snap = snap)

            // Outside snapshot polling: the engine pushes frames to the plot itself.
            SpectrumPlot(
                engine = vm.engine,
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
            )

            CalibrateButton(
                enabled = vm.micGranted && !calibrating,
                calibrating = calibrating,
                noiseError = snap.noiseError,
                onClick = vm::calibrate,
            )
        }

        AnimatedVisibility(
            visible = vm.toast != null,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = 24.dp),
        ) {
            Surface(
                shape = CircleShape,
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                tonalElevation = 6.dp,
                shadowElevation = 4.dp,
            ) {
                Text(
                    text = lastToast,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp),
                )
            }
        }
    }
}

@Composable
private fun StatusLine(micDenied: Boolean, snap: Snapshot) {
    when {
        micDenied -> Text(
            text = "Microphone permission denied",
            color = MaterialTheme.colorScheme.error,
            style = MaterialTheme.typography.bodyMedium,
        )
        snap.coreError != null -> Text(
            text = "Core: ${snap.coreError}",
            color = MaterialTheme.colorScheme.error,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
private fun CalibrateButton(
    enabled: Boolean,
    calibrating: Boolean,
    noiseError: String?,
    onClick: () -> Unit,
) {
    val contentColor = MaterialTheme.colorScheme.onSurface

    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f))
                .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
                .alpha(if (enabled || calibrating) 1f else 0.45f)
                .padding(vertical = 14.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (calibrating) {
                CircularProgressIndicator(
                    modifier = Modifier.size(18.dp),
                    strokeWidth = 2.dp,
                    color = contentColor,
                )
                Text(
                    text = "Calibrating… stay quiet",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = contentColor,
                )
            } else {
                WaveformMinusIcon(color = contentColor)
                Text(
                    text = "Calibrate noise",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = contentColor,
                )
            }
        }

        if (noiseError != null && !calibrating) {
            Text(
                text = "Calibration failed: $noiseError",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Composable
private fun WaveformMinusIcon(color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier = modifier.size(20.dp)) {
        val w = size.width
        val h = size.height
        val barWidth = 1.8.dp.toPx()
        val bars = floatArrayOf(0.28f, 0.56f, 0.82f, 0.44f)
        val xs = floatArrayOf(0.14f, 0.34f, 0.54f, 0.74f)
        for (i in bars.indices) {
            val half = h * bars[i] * 0.5f
            val x = w * xs[i]
            drawLine(
                color = color,
                start = Offset(x, h * 0.5f - half),
                end = Offset(x, h * 0.5f + half),
                strokeWidth = barWidth,
                cap = StrokeCap.Round,
            )
        }
        drawLine(
            color = color,
            start = Offset(w * 0.66f, h * 0.82f),
            end = Offset(w * 0.94f, h * 0.82f),
            strokeWidth = barWidth,
            cap = StrokeCap.Round,
        )
    }
}
