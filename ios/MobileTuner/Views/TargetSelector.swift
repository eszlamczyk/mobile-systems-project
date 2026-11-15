import SwiftUI

struct TargetSelector: View {
    let presets: [Preset]
    @Binding var preset: Preset?

    var body: some View {
        Menu {
            Picker("Target", selection: $preset) {
                ForEach(presets) { preset in
                    Text(preset.name).tag(Optional(preset))
                }
                Divider()
                Text("Single note").tag(Preset?.none)
            }
        } label: {
            HStack {
                Text(preset?.name ?? "Single note")
                    .font(.headline)
                Spacer()
                Image(systemName: "chevron.up.chevron.down")
                    .imageScale(.small)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .background(.quaternary.opacity(0.6), in: .rect(cornerRadius: 12))
        }
        .tint(.primary)
        .accessibilityLabel("Target: \(preset?.name ?? "Single note")")
    }
}

#Preview {
    @Previewable @State var preset: Preset? = Preset.all.first
    TargetSelector(presets: Preset.all, preset: $preset).padding()
}
