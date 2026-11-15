import SwiftUI

/// Twelve note names plus an octave stepper, shown in single-note mode.
struct NotePicker: View {
    @Binding var midi: Int32

    var body: some View {
        VStack(spacing: 8) {
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 6), spacing: 8) {
                ForEach(0..<12, id: \.self) { pitchClass in
                    let note = Int32((Note.octave(midi) + 1) * 12 + pitchClass)
                    NoteChip(label: Note.names[pitchClass], isSelected: note == midi) {
                        midi = note
                    }
                }
            }
            Stepper("Octave \(Note.octave(midi))", value: octave, in: 0...8)
        }
    }

    private var octave: Binding<Int> {
        Binding(
            get: { Note.octave(midi) },
            set: { midi = Int32(($0 + 1) * 12) + midi % 12 }
        )
    }
}

#Preview {
    @Previewable @State var midi: Int32 = 69
    NotePicker(midi: $midi).padding()
}
