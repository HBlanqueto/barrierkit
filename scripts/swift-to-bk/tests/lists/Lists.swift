import SwiftUI

struct Fruit {
    let id: Int
    let name: String
}

struct Lists: View {
    let fruits = [Fruit(id: 1, name: "Apple"), Fruit(id: 2, name: "Banana")]
    var body: some View {
        VStack {
            ForEach(fruits, id: \.id) { fruit in
                HStack(spacing: 4) {
                    Text(fruit.name)
                    Spacer()
                    Text("\(fruit.id)")
                }
            }
            Divider()
            ForEach(0..<3) { index in
                Text("Row \(index)")
            }
            List {
                ForEach(fruits, id: \.id) { fruit in
                    Text(fruit.name)
                }
            }
        }
    }
}
