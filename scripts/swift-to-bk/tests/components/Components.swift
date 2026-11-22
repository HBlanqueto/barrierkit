import SwiftUI

struct Greeting: View {
    let name: String
    var body: some View {
        Text("Hello, \(name)").font(.headline)
    }
}

struct NameField: View {
    @Binding var name: String
    var body: some View {
        TextField("Name", text: $name)
    }
}

struct Components: View {
    @State private var name = "Ana"
    var body: some View {
        VStack {
            Greeting(name: "Luis")
            NameField(name: $name)
            Greeting(name: name)
        }
    }
}
