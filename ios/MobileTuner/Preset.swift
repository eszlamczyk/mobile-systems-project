struct TuningPreset {
    let name: String
    let notes: [Int32]
}

enum Preset: String, CaseIterable {
    case eStandard, halfStepDown, fullStepDown, dropD, dropC, dropB
    case openG, openD, openE, openC, dadgad, bass4String, ukulele, violin

    var tuning: TuningPreset {
        switch self {
        case .eStandard: return TuningPreset(name: "E standard", notes: [40, 45, 50, 55, 59, 64])
        case .halfStepDown: return TuningPreset(name: "Half step down", notes: [39, 44, 49, 54, 58, 63])
        case .fullStepDown: return TuningPreset(name: "Full step down", notes: [38, 43, 48, 53, 57, 62])
        case .dropD: return TuningPreset(name: "Drop D", notes: [38, 45, 50, 55, 59, 64])
        case .dropC: return TuningPreset(name: "Drop C", notes: [36, 43, 48, 53, 57, 62])
        case .dropB: return TuningPreset(name: "Drop B", notes: [35, 42, 47, 52, 56, 61])
        case .openG: return TuningPreset(name: "Open G", notes: [38, 43, 50, 55, 59, 62])
        case .openD: return TuningPreset(name: "Open D", notes: [38, 45, 50, 54, 57, 62])
        case .openE: return TuningPreset(name: "Open E", notes: [40, 47, 52, 56, 59, 64])
        case .openC: return TuningPreset(name: "Open C", notes: [36, 43, 48, 55, 60, 64])
        case .dadgad: return TuningPreset(name: "DADGAD", notes: [38, 45, 50, 55, 57, 62])
        case .bass4String: return TuningPreset(name: "Bass 4 string", notes: [28, 33, 38, 43])
        case .ukulele: return TuningPreset(name: "Ukulele", notes: [67, 60, 64, 69])
        case .violin: return TuningPreset(name: "Violin", notes: [55, 62, 69, 76])
        }
    }
}
