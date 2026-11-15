import SwiftUI

/// One button per string of the active preset. Tapping sets the target.
struct StringButtons: View {
    let notes: [Int32]
    @Binding var selected: Int

    var body: some View {
        HStack(spacing: 8) {
            ForEach(notes.indices, id: \.self) { index in
                NoteChip(label: Note.name(notes[index]), isSelected: index == selected) {
                    selected = index
                }
                .accessibilityLabel("String \(notes.count - index), \(Note.fullName(notes[index]))")
            }
        }
    }
}

/// Shared square button for a note name. Used by `StringButtons` and `NotePicker`.
struct NoteChip: View {
    let label: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(label)
                .font(.title3.weight(.semibold))
                .frame(maxWidth: .infinity, minHeight: 44)
                .foregroundStyle(isSelected ? Color.white : Color.primary)
                .background(
                    isSelected ? AnyShapeStyle(Color.accentColor) : AnyShapeStyle(.quaternary.opacity(0.6)),
                    in: .rect(cornerRadius: 10)
                )
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

#Preview {
    @Previewable @State var selected = 0
    StringButtons(notes: Preset.all[1].notes, selected: $selected)
        .padding()
}
