# Translating Swift apps to BarrierKit

**Goal:** decide how a Swift or AppKit app becomes a BarrierKit app by a tool,
and which kit gaps stop that tool today.
**Scope:** the input (Swift source, or a running app), the translator, the kit
surface it needs and the order to close the gaps. It does not cover running on
macOS ([macos-arm-port.md](macos-arm-port.md)), iOS or UIKit, or the look of
controls ([coverage matrix](swiftui-appkit-coverage.md)).
**Snapshot:** `main` at `dc9656b2`, 2026-11-22. Xcode 27.2, Swift 6.4.2.
**Reader:** the owner, and the engineer who builds the tool.
**Input:** [Swift syntax in Rust](swift-syntax-in-rust.md),
[SwiftUI API gap](swiftui-api-gap.md), the [coverage matrix](swiftui-appkit-coverage.md),
the [glossary](../barrierkit/20-glossary.md), `~/src/probes` and one
experiment ([Evidence](#evidence)).
**Output:** a design with three inputs, a minimum tool, its test, and the
ranked gaps. Since 2026-11-22 also the first tool, `scripts/swift-to-bk/`
([usage](../../scripts/swift-to-bk/README.md)), and what it showed.

## Steps

1. Read the [verdict](#verdict) and the [decisions](#decisions-for-the-owner).
2. To translate an app, follow the [usage](../../scripts/swift-to-bk/README.md);
   to extend the tool, read the [status](#status-of-the-stages), the
   [results](#results) and the [limits](#limits-of-the-tool).
3. To pick kit work, take the first open row of the [gaps](#gaps-that-block-the-translator).

## Verdict

A tool can translate the SwiftUI an app writes into the kit's idioms for the
constructs the kit already has, and it can say exactly which ones it lacks.
It cannot translate AppKit code: the kit has no responder chain, delegates,
Auto Layout, key-value observing or hosting, and ADR 0085 made all of them
non-goals until an app asks. A translator is such an app.

Reverse engineering a compiled app is a different job: the tool reads what the
app shows (accessibility tree, view and layer tree, pixels), not its code.
That yields a visual clone to iterate on with the [parity test](macos-arm-port.md#parity-test).

## Evidence

| # | Experiment | Result |
|---|---|---|
| E5 | `swiftc -typecheck Demo.swift -Xfrontend -dump-ast` on a 10-line SwiftUI `Counter` | a typed, resolved AST of 158 KB: each call names its declaration, such as `SwiftUICore.(file).VStack.init(alignment:spacing:content:)` and `Text extension.font`; the `@State` macro is expanded |
| E6 | The same with `swiftc -frontend … -dump-ast-format json` | fails: the macro plugin for `@State` is not loaded in `-frontend` mode |
| E6b | `swiftc -typecheck F.swift -Xfrontend -dump-ast -Xfrontend -dump-ast-format -Xfrontend json` (2026-11-22) | works through the driver, but types and declarations are mangled names (`$s7SwiftUI6VStackV…`) and string literals are base64. The tool reads the text form |
| E7 | The existing probe `~/src/probes/axdump.swift` | reads the accessibility tree (role, position, size, font of the text). Not run for this study: it needs the accessibility permission |

The typed AST is the better input than syntax alone: a name such as `padding`
resolves to the SwiftUI declaration it calls, so the tool needs no guess.

## Three inputs

| Input | What it gives | Tool | State |
|---|---|---|---|
| Source | exact SwiftUI use: views, modifiers, state, environment | `swiftc -typecheck -Xfrontend -dump-ast`, or SwiftSyntax | E5 works on text; a parser is not written |
| SDK interface | every public SwiftUI and AppKit name | `.swiftinterface` files of the SDK | the gap study used an uncommitted script; commit it |
| Running app | role, geometry, text, fonts, view and layer tree | accessibility API, `NSView` and `CALayer` dumps of the probes | probes exist; no translator |

For an app without source, only the third applies. Limit it to apps the owner
may inspect: the owner's own, open source, or Apple's system apps for visual
reference. `AGENTS.md` already keeps Apple artwork out of the repository, and
the tool must too.

## Stages

1. **Table.** Turn the [glossary](../barrierkit/20-glossary.md) and the
   constructs of [16](../barrierkit/16-swiftui-to-the-kit.md) into one data
   file: Swift declaration to kit call. The glossary has 414 public types; the
   file is machine-readable, the chapters stay the explanation.
2. **View tree.** Read a `View` struct from the typed AST and emit one kit
   component: stacks, text, buttons, `Form`, `List`. Emit `use_state` for
   `@State`, `.children(iter.map(..))` with `.key` for `ForEach`, a Rust `if`
   for `if`.
3. **Report.** For each construct with no kit row, emit a `TODO` that names it,
   with the row of the [API gap](swiftui-api-gap.md). A translation is done
   when the report is empty, not when the code compiles.
4. **Check.** Compile the output as a recipe (`[[example]]` in
   `crates/bk/Cargo.toml`) and run the parity test on it.
5. **Running app.** Emit a tree of kit containers from the accessibility and
   view dumps, with fixed sizes, then let the parity test drive the fixes.

Stage 2 on the `Counter` of E5 must give the shape of
[`counter.rs`](../../agents/recipes/counter.rs):

```swift
struct Counter: View {
    @State private var count = 0
    var body: some View {
        VStack { Text("Count: \(count)").font(.title); Button("Add") { count += 1 }.buttonStyle(.borderedProminent) }
    }
}
```

```rust
let mut count = use_state(|| 0);
VStack::new()
    .child(Text::new(count.read().to_string()).font(TITLE))
    .child(Button::new("Add one").style(ButtonStyle::bordered_prominent).on_press(move |_| *count.write() += 1))
```

The two differ in their strings only. The translator must also handle what
the recipe hides: the interpolated `"Count: \(count)"` becomes an owned
string, and a control that takes `&'static str` needs `i18n::intern`
([swift-syntax-in-rust.md](swift-syntax-in-rust.md), 1.6).

## Status of the stages

Built on 2026-11-22 with the defaults of the decisions below: SwiftUI source
only, the typed AST of `swiftc`, the tool beside `scripts/`.

| Stage | State | Where |
|---|---|---|
| 1. Table | done: one JSON file of 1,897 rows | `scripts/swift-to-bk/table.json`, built by `make-table.py` from the glossary, [chapter 16](../barrierkit/16-swiftui-to-the-kit.md), the closed lists of the [API gap](swiftui-api-gap.md) and 25 hand-written rules (`rules.json`) |
| 2. View tree | done for the constructs of the [usage](../../scripts/swift-to-bk/README.md#what-it-translates) | `translate.py`, `swiftast.py` |
| 3. Report | done: a `TODO(swift: construct) -> row` line at the construct and a summary at the end | same |
| 4. Check | done: 11 tests, five recipes that build on macOS | `tests/run.py`, `check.sh`, `recipe.py`, `agents/recipes/swift_*.rs` |
| 5. Running app | not started | |

The table keeps what is known: each row has the Swift name, the kit item,
a status (`mapped`, `partial`, `missing`, `n/a`, `out-of-scope`) and, for the
gap rows, the section of the API gap. The code keeps how a construct is
lowered. A Swift name with a `mapped` row and no lowering reports
`translator: not lowered yet`; a name with a `missing` row reports its gap
row.

## Results

| Measure | Result |
|---|---|
| `Counter` | the output equals [`counter.rs`](../../agents/recipes/counter.rs) except the `.options(..)` line: the recipe adds `min_size`, which has no SwiftUI source |
| Recipes | `swift_counter`, `swift_form`, `swift_lists`, `swift_components`, `swift_styles` translate with an empty report and pass `cargo build -p bk --example` on macOS arm64. They were not run in a window |
| Incomplete input | `tests/gaps` reports 7 constructs (`ZStack`, `foregroundStyle`, `tag` three times, `Picker`, `keyboardShortcut`) and the output still compiles, because each gap becomes a `rect()` |
| Tests | 11: the parser, the declaration names, the table (current, statuses, gap rows) and the goldens |
| Time | `swiftc -typecheck` takes about 1.2 s for a 10-line view; the text AST is 150 KB for it and 470 KB for a 20-line view |
| A sample outside the tests (a to-do list with `NavigationStack`, `toolbar`, `@AppStorage`, `Label`) | 10 `TODO`s, each with its row; no crash |

## Limits of the tool

What the tool does not do, in the order it hurt on the samples:

1. **No model code.** Only `View` structs, `@State`, `@Binding`, constants and
   data structs used by `ForEach` are read. `@Observable` classes, methods,
   computed properties, `@State var items: [T]` with `append`, `@AppStorage`
   and `@Environment` become `TODO`s.
2. **One file.** Types from another file are unknown; `swiftc` would accept
   several files, but the tool passes one.
3. **Modifiers set, they do not wrap.** The tool emits the kit's order-free
   form and wraps a control in `rect()` when it needs `.padding` or `.width`.
   `.frame` is a border-box size in the kit; the tool does not correct the
   padding ([swift-syntax-in-rust.md](swift-syntax-in-rust.md), 1.3).
4. **Nothing inherits.** `.font` and `.buttonStyle` on a container report a
   `TODO`; the kit needs `.control_style` for the second.
5. **Literals stay literals.** Text goes into the program as written; the tool
   does not write `tr!` ids or `.ftl` files.
6. **Defaults are fixed.** `.padding()` becomes `SPACE_4` (16) and `Toggle`
   becomes `Checkbox`, as on macOS; the tool cannot read a default from the
   AST.
7. **Names.** Swift names become `snake_case`; a collision with a kit or Rust
   name is not checked.

What the typed AST does not carry:

- Comments, blank lines and formatting. The tool cannot keep the author's
  comments.
- The value of a defaulted argument: `default_argument_expr` names only the
  parameter. `VStack()` and `VStack(spacing: nil)` look alike.
- A readable type for a modifier chain in the JSON form (mangled); the text
  form has it but as a string.
- The macro expansion is part of it: `@State` adds `_x`, `__x` and `$x`
  declarations that the tool must recognise and skip.
- The builder desugaring: each view of a `@ViewBuilder` appears as
  `$__builderN`, with the written expression in `original_init` and the typed
  one in `processed_init`. An `if` without `else` has an empty `else` arm.
- Conditional compilation (`#if`, `#available`) is already resolved for the
  machine that runs `swiftc`.

## Next gaps, in order

From the samples, closest to the translator first. The first four are tool
work, the rest kit work from the [gaps table](#gaps-that-block-the-translator).

1. Model state: `@Observable`, arrays and structs in `@State`, mutation of
   them in actions.
2. `Picker` and `.tag` to `PopUpButton`, `Form` and `Section` to `Form`,
   `Button(role:)`, `.keyboardShortcut` to `Button::keyboard_shortcut`.
3. Colours and shapes: `.foregroundStyle`, `.foregroundColor`, `.background`,
   `.cornerRadius` to a `ShapeStyle`; the kit has no named colours.
4. Effects and environment: `.onAppear`, `.onChange`, `.task`, `@Environment`.
5. `NavigationStack` (top 20, row 6) and the declarative toolbar (row 7): the
   most frequent `TODO`s of real apps.
6. `isPresented` sheets, alerts and popovers (row 8), then `Binding` (row 9).

## What the kit has and lacks

From the coverage matrix (221 rows) and the API gap (1,672 items), both
snapshots of 2026-11-20 and 2026-11-21:

| Source | Done | Partial | Missing |
|---|---|---|---|
| Coverage matrix, rows | 65 measured, 36 implemented | 81 | 39 |
| API gap, items | 144 implemented | 504 | 733 (284 `n/a`, 7 out of scope) |

ADRs 0084 to 0089 closed rows of the top 20 after those snapshots: owned
labels, views in `List` and `Table`, `Spacer`, `Grid` and `frame`, scenes and
the standard menus, ambient `disabled`, environment values and focus by name.

The AppKit side has no kit counterpart for these
([swift-syntax-in-rust.md](swift-syntax-in-rust.md), 2): `NSResponder` and
validation, delegates and data sources, KVO and bindings, `NSLayoutConstraint`,
`NotificationCenter`, `NSHostingView` and `NSViewRepresentable`, `CALayer`,
`NSDocument`, `NSCursor` as a named API.

## Gaps that block the translator

Ranked by how many apps a gap stops. The numbers are the rows of the
[top 20](swiftui-api-gap.md#top-20-gaps) that are still open.

| # | Gap | Why a translator needs it | Size | Blocker |
|---|---|---|---|---|
| 1 | `NavigationStack` with a typed path (6) | every multi-screen app | M | none |
| 2 | Declarative toolbar: `ToolbarItem(placement:)`, spacers, labels (7) | AppKit and SwiftUI toolbars are data | M | measurement |
| 3 | `isPresented` and `item` for `sheet`, `alert`, `popover` (8) | the common SwiftUI form | M | Wayland: a popup grab needs an input serial; not a limit on macOS |
| 4 | `Binding` (9) | a `$x` has no target | M | ADR 0085 |
| 5 | Gestures (13), typed drag and drop (14) | custom interaction | L, M | Freya: one handler per event |
| 6 | Text model: `Text + Text`, Markdown, `textSelection` (15) | rich labels | S to M | none |
| 7 | `withAnimation`, `Transition`, `matchedGeometryEffect` (16) | animation is state-driven | M each | design |
| 8 | Responder chain and target-action | every AppKit app | M | ADR 0085 |
| 9 | Delegates: `windowShouldClose`, `applicationShouldTerminate`, `application(_:open:)` | quit, close veto, open URL | S each | ADR 0085 |
| 10 | `NSViewRepresentable` equivalent | AppKit views inside SwiftUI | L | none |
| 11 | Accessibility modifiers on every control (18) | VoiceOver parity | M | Freya |

Rows 4, 8 and 9 contradict ADR 0085. The ADR says «until an app asks»; the
translator is the first caller that would, so the owner decides whether it
stays a non-goal.

## Decisions for the owner

The first tool used these defaults, which stand until the owner decides
otherwise: decision 1, SwiftUI source only; decision 3, the typed AST of
`swiftc`; decision 4, beside `scripts/`, outside the workspace.

1. The scope of the first translator: SwiftUI source only (stages 1 to 4), or
   also the running-app input (stage 5).
2. Whether ADR 0085 changes: `Binding`, a minimal responder chain and window
   and app delegates.
3. The parser: the typed AST of `swiftc` (needs Xcode on the machine that
   translates) or SwiftSyntax (needs no type check, and names are guessed).
4. Where the tool lives: a crate in this workspace, or beside `scripts/`. It
   must not enter the compositor's dependencies (`scripts/check-client-only-deps.sh`).
5. UIKit: out of this session. It needs its own session and its own table;
   SwiftUI source translates the same way, UIKit does not.

## Not verified

- That the typed AST has every modifier chain of a larger app; the samples are
  at most 25 lines.
- That the five recipes look right in a window: they were built, not run.
- Clippy on the five recipes under the Linux toolchain (see the commit).
- That the accessibility dump of an app gives enough to lay out a window.
- Any size above S for the gaps; they come from the earlier studies.
