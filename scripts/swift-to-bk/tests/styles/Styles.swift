import SwiftUI

struct Styles: View {
    @State private var on = true
    @State private var busy = false
    var body: some View {
        HStack {
            Text("Large").font(.largeTitle)
            Text("Small").font(.caption)
            Button("Plain") { busy.toggle() }.buttonStyle(.plain)
            Button("Borderless") { busy.toggle() }.buttonStyle(.borderless).disabled(busy)
            Toggle("Switch", isOn: $on).toggleStyle(.switch)
        }
        .padding(.horizontal, 20)
        .frame(maxWidth: .infinity)
    }
}
