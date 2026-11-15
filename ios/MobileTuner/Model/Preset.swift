nonisolated struct Preset: Hashable, Identifiable {
    let name: String
    let notes: [Int32]

    var id: String { name }
}

extension Preset {
    static let all: [Preset] = BuiltInPreset.allCases.map(\.tuning)
}
