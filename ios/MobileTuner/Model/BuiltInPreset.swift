nonisolated enum BuiltInPreset: String, CaseIterable {
    case eStandard, halfStepDown, fullStepDown, dropD, dropC, dropB
    case openG, openD, openE, openC, dadgad, bass4String, ukulele, violin

    var tuning: Preset {
        switch self {
        case .eStandard: return Preset(name: "E standard", notes: [40, 45, 50, 55, 59, 64])
        case .halfStepDown: return Preset(name: "Half step down", notes: [39, 44, 49, 54, 58, 63])
        case .fullStepDown: return Preset(name: "Full step down", notes: [38, 43, 48, 53, 57, 62])
        case .dropD: return Preset(name: "Drop D", notes: [38, 45, 50, 55, 59, 64])
        case .dropC: return Preset(name: "Drop C", notes: [36, 43, 48, 53, 57, 62])
        case .dropB: return Preset(name: "Drop B", notes: [35, 42, 47, 52, 56, 61])
        case .openG: return Preset(name: "Open G", notes: [38, 43, 50, 55, 59, 62])
        case .openD: return Preset(name: "Open D", notes: [38, 45, 50, 54, 57, 62])
        case .openE: return Preset(name: "Open E", notes: [40, 47, 52, 56, 59, 64])
        case .openC: return Preset(name: "Open C", notes: [36, 43, 48, 55, 60, 64])
        case .dadgad: return Preset(name: "DADGAD", notes: [38, 45, 50, 55, 57, 62])
        case .bass4String: return Preset(name: "Bass 4 string", notes: [28, 33, 38, 43])
        case .ukulele: return Preset(name: "Ukulele", notes: [67, 60, 64, 69])
        case .violin: return Preset(name: "Violin", notes: [55, 62, 69, 76])
        }
    }
}
