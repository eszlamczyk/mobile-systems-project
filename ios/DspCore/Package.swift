// swift-tools-version:5.9
// Local package wrapping dsp-core. Both the XCFramework and Sources/DspCore/DspCore.swift
// are produced by scripts/build-ios.sh; run it before opening the app in Xcode.
import PackageDescription

let package = Package(
    name: "DspCore",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "DspCore", targets: ["DspCore"]),
    ],
    targets: [
        // Static libdsp_core.a per platform plus the DspCoreFFI C module.
        .binaryTarget(name: "DspCoreFFI", path: "DspCoreFFI.xcframework"),
        // UniFFI-generated Swift: `Tuner`, `TunerConfig`, `TunerResult`.
        .target(name: "DspCore", dependencies: ["DspCoreFFI"]),
    ]
)
