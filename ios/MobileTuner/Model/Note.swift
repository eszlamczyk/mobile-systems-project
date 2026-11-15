import Foundation

nonisolated enum Note {
    static let names = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"]

    static func name(_ midi: Int32) -> String {
        names[Int(((midi % 12) + 12) % 12)]
    }

    static func octave(_ midi: Int32) -> Int {
        Int(midi) / 12 - 1
    }

    static func fullName(_ midi: Int32) -> String {
        "\(name(midi))\(octave(midi))"
    }

    static func frequency(_ midi: Int32) -> Double {
        440 * pow(2, Double(midi - 69) / 12)
    }
}
