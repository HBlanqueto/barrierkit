import SwiftUI

struct Gaps: View {
    @State private var choice = 0
    var body: some View {
        VStack {
            LazyVStack {
                Text("Back")
                Text("Front").foregroundStyle(.red)
            }
            Text("Tagged").tag(1)
            Picker("Choice", selection: $choice) {
                Text("One").tag(0)
                Text("Two").tag(1)
            }
            Button("Go") { choice += 1 }.keyboardShortcut("g")
        }
    }
}
