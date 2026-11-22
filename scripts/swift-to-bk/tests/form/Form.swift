import SwiftUI

struct Form: View {
    @State private var name = ""
    @State private var subscribed = false
    @State private var clicks = 0
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Name", text: $name)
            Toggle("Subscribe", isOn: $subscribed)
            if subscribed {
                Text("Thanks, \(name)")
            } else {
                Text("Not subscribed")
            }
            if clicks > 2 && !subscribed {
                Text("Still here?")
            }
            Button("Click") { clicks += 1 }
            Text(subscribed ? "yes" : "no").padding(12).frame(width: 200, height: 40)
        }
        .padding()
    }
}
