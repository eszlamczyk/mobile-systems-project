import SwiftUI

@main
struct MobileTunerApp: App {
    @State private var model = TunerModel()

    var body: some Scene {
        WindowGroup {
            TunerScreen(model: model)
        }
    }
}
