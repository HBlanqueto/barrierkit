import SwiftUI

struct Counter: View {
    @State private var count = 0
    var body: some View {
        VStack {
            Text("\(count)").font(.title)
            Button("Add one") { count += 1 }.buttonStyle(.borderedProminent)
        }
    }
}

@main
struct CounterApp: App {
    var body: some Scene {
        WindowGroup("Counter") {
            Counter()
        }
        .defaultSize(width: 420, height: 300)
    }
}
