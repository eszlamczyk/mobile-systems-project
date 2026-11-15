nonisolated struct Preset: Hashable, Identifiable {
    let name: String
    let notes: [Int32]

    var id: String { name }
}

extension Preset {
    // TODO: temporary. Replace with the `Preset` enum (JBRS) mapped into `Preset` values.
    static let all: [Preset] = [
        Preset(name: "E standard", notes: [40, 45, 50, 55, 59, 64]),
        Preset(name: "Drop D", notes: [38, 45, 50, 55, 59, 64]),
    ]
}
