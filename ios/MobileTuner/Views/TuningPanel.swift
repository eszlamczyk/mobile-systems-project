import SwiftUI

struct TuningPanel: View {
    let targetMidi: Int32
    let cents: Float?
    let pitchHz: Float?
    /// False while the last reading is held after the note faded; the panel dims.
    var isLive = true

    var body: some View {
        VStack(spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 2) {
                Text(Note.name(targetMidi))
                    .font(.system(size: 96, weight: .bold, design: .rounded))
                Text("\(Note.octave(targetMidi))")
                    .font(.title.weight(.semibold))
                    .opacity(0.7)
            }
            Text(cents.map { String(format: "%+.1f cents", $0) } ?? "– cents")
                .font(.title2.monospacedDigit())
            Text(pitchHz.map { String(format: "%.1f Hz", $0) } ?? "– Hz")
                .font(.body.monospacedDigit())
                .opacity(0.8)
        }
        .foregroundStyle(cents == nil ? Color.primary : Color.white)
        .frame(maxWidth: .infinity)
        .padding(.vertical, 24)
        .background(background, in: .rect(cornerRadius: 20))
        .opacity(cents != nil && !isLive ? 0.6 : 1)
        .animation(.easeOut(duration: 0.15), value: cents)
        .animation(.easeOut(duration: 0.3), value: isLive)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(accessibilityText)
    }

    private var background: AnyShapeStyle {
        if let cents {
            AnyShapeStyle(Self.color(cents: cents))
        } else {
            AnyShapeStyle(.quaternary.opacity(0.6))
        }
    }

    static func color(cents: Float) -> Color {
        let t = min(max((abs(Double(cents)) - 2) / 48, 0), 1)
        return Color(hue: 0.33 * (1 - t), saturation: 0.75, brightness: 0.85)
    }

    private var accessibilityText: String {
        let target = "Target \(Note.fullName(targetMidi))"
        guard let cents else { return "\(target), no pitch detected" }
        let direction = cents > 0 ? "sharp" : "flat"
        let offset = "\(target), \(String(format: "%.0f", abs(cents))) cents \(direction)"
        guard let pitchHz else { return offset }
        return "\(offset), \(String(format: "%.1f", pitchHz)) hertz"
    }
}

#Preview {
    VStack(spacing: 16) {
        TuningPanel(targetMidi: 38, cents: 1.2, pitchHz: 73.5)
        TuningPanel(targetMidi: 45, cents: -22, pitchHz: 108.6, isLive: false)
        TuningPanel(targetMidi: 64, cents: nil, pitchHz: nil)
    }
    .padding()
}
