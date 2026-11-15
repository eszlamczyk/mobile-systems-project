import AVFoundation
import SwiftUI

struct TunerScreen: View {
    @Bindable var model: TunerModel
    @Environment(\.scenePhase) private var phase

    var body: some View {
        content
        .task {
            await model.requestPermission()
            model.start()
        }
        .onChange(of: phase) { _, newPhase in
            if newPhase == .active {
                model.start()
            } else if newPhase == .background {
                model.stop()
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: AVAudioSession.interruptionNotification).receive(on: DispatchQueue.main)) { note in
            model.handleInterruption(note)
        }
    }

    private var content: some View {
        VStack(spacing: 16) {
            TargetSelector(presets: model.presets, preset: $model.preset)

            if let preset = model.preset {
                StringButtons(notes: preset.notes, selected: $model.stringIndex)
            } else {
                NotePicker(midi: $model.singleNote)
            }

            // The UI never waits on DSP: it samples the latest snapshot at most 30 times per second.
            TimelineView(.animation(minimumInterval: 1.0 / 30)) { _ in
                let snap = model.latest
                let detected = snap.result.flatMap { $0.pitchHz > 0 ? $0 : nil }
                VStack(spacing: 16) {
                    TuningPanel(
                        targetMidi: model.targetMidi,
                        cents: detected?.cents,
                        pitchHz: detected?.pitchHz
                    )
                    statusLine(snap)
                }
            }

            // Outside the timeline: the engine pushes frames to the plot itself.
            SpectrumPlot(engine: model.engine)
                .frame(maxHeight: .infinity)

            calibrateButton
        }
        .padding()
        .overlay(alignment: .bottom) {
            if let toast = model.toast {
                Text(toast)
                    .font(.callout)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(.thinMaterial, in: Capsule())
                    .padding(.bottom, 24)
                    .transition(.opacity)
            }
        }
        .animation(.default, value: model.toast)
    }

    private var calibrateButton: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 10)) { _ in
            let calibrating = model.isCalibrating
            let error = model.latest.noiseError
            VStack(spacing: 6) {
                Button {
                    model.calibrate()
                } label: {
                    HStack(spacing: 8) {
                        if calibrating {
                            ProgressView()
                            Text("Calibrating… stay quiet")
                        } else {
                            Image(systemName: "waveform.badge.minus")
                            Text("Calibrate noise")
                        }
                    }
                    .font(.headline)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 14)
                    .background(.quaternary.opacity(0.6), in: .rect(cornerRadius: 12))
                }
                .buttonStyle(.plain)
                .disabled(!model.micGranted || calibrating)

                if let error, !calibrating {
                    Text("Calibration failed: \(error)")
                        .font(.footnote)
                        .foregroundStyle(.red)
                }
            }
        }
    }

    @ViewBuilder
    private func statusLine(_ snap: Snapshot) -> some View {
        if model.micDenied {
            Text("Microphone permission denied").foregroundStyle(.red)
        } else if let error = snap.coreError {
            Text("Core: \(error)").foregroundStyle(.red)
        }
    }
}

#Preview {
    TunerScreen(model: TunerModel())
}
