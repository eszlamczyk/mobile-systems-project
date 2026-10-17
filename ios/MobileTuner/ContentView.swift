import DspCore
import SwiftUI

struct ContentView: View {
    var body: some View {
        // Every Tuner method is still `todo!()` in dsp-core, so nothing calls into the core yet.
        // The import above is what proves the package links.
        Text("Hello iOS!")
            .padding()
    }
}

#Preview {
    ContentView()
}
