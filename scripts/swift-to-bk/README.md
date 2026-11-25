# swift-to-bk: translate SwiftUI source to BarrierKit

**Goal:** turn a SwiftUI file into a BarrierKit program, and list what the kit
cannot yet express.
**Scope:** one SwiftUI source file in, one Rust program out. It reads views,
`@State`, `@Binding`, `ForEach` and `if`. It does not read AppKit or UIKit
code, nor several files at once. Design and limits:
[Translating Swift apps to BarrierKit](../../docs/research/swift-app-translation.md).
**Reader:** an engineer who ports a SwiftUI app, or an agent that does.
**Input:** a `.swift` file. The machine needs Xcode (`swiftc`) and Python 3.
**Output:** Rust on stdout (formatted with `rustfmt`), and a summary of open
`TODO` lines on stderr.

## Steps

1. Translate the file:

   ```sh
   python3 -I scripts/swift-to-bk/translate.py MyApp.swift
   ```

2. Read the summary at the end of the output. A translation is done when it is
   empty. Each line names the Swift construct and the row that explains it:

   ```text
   // TODO(swift: View.tag) -> swiftui-api-gap.md §7.2 view modifiers, partial (x3)
   ```

   `translator: not lowered yet` means the kit has the item and the tool does
   not write it yet. `translator: Swift expression not lowered` means a
   statement or value the tool does not read. A row of the
   [API gap](../../docs/research/swiftui-api-gap.md) means the kit lacks it.
3. Pass `--strict` to exit with status 1 while the summary is not empty. Pass
   `--name`, `--title` or `--root` to set the application name, the window
   title or the view that fills the window.
4. To keep a finished translation as a recipe, run:

   ```sh
   python3 -I scripts/swift-to-bk/recipe.py my_app MyApp.swift
   ```

   It writes `crates/bk/examples/swift_my_app.rs`, which Cargo finds as an
   example, and runs `cargo build -p bk --example swift_my_app`.
   It writes nothing while the summary is not empty.
5. Check the tool after a change: `./dev.sh check` builds every `swift_*`
   recipe with the other examples.
6. After you edit the glossary, the constructs table of chapter 16 or the
   API gap, rebuild the table:

   ```sh
   python3 -I scripts/swift-to-bk/make-table.py
   ```

## What it translates

| Swift | BarrierKit |
|---|---|
| `VStack`, `HStack`, `ScrollView`, `Spacer`, `Divider` | the same types; `alignment:` and `spacing:` become `.cross_align` and `.spacing` |
| `Text("a \(n)")` | `Text::new(format!("a {}", n.read()))` |
| `Button("t") { n += 1 }`, `Button(action:label:)` | `Button::new("t").on_press(move \|_\| *n.write() += 1)`, `.label(\|\| ..)` |
| `TextField("p", text: $s)` | `TextField::new(s, "p")` |
| `Toggle("t", isOn: $b)` | `Checkbox` (the macOS default); `.toggleStyle(.switch)` gives `Switch` |
| `Image(systemName:)`, `Label(_:systemImage:)` | `Image::system(Symbol::X)`, `Label::new(t, Symbol::X)` |
| `List { ForEach(items) { .. } }` | `List::new(..).row(\|index\| ..)` over a constant array |
| `ForEach(items, id: \.id)` | `.children(ITEMS.into_iter().map(\|x\| .. .key(x.id)))` |
| `if`, `if`/`else` | `.maybe_child(c.then(\|\| ..))`, `.child(if c { a.into_element() } else { b.into_element() })` |
| `@State var n = 0` | `let mut n = use_state(\|\| 0)` |
| `@Binding var s: String` | a `State<String>` field of a `Component` |
| `.padding`, `.frame`, `.font`, `.buttonStyle`, `.disabled` | `.padding(..)`, `.width(Size::px(..))`, `.font(TITLE)`, `.style(ButtonStyle::..)`, `.disabled(..)` |
| `WindowGroup`, `.defaultSize` | `Application::new(..).window(..)`, `.options(..)` |
| another `View` struct | `#[derive(PartialEq)] struct` with `impl Component` |

Every other construct becomes a `TODO` and a placeholder (`rect()`), so the
output still compiles.
