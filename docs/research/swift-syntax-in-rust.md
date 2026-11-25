# Swift syntax in Rust

**Goal:** show what each Swift, SwiftUI and AppKit construct becomes in the
BarrierKit API, what is lost and what cannot be said at all.
**Scope:** the constructs that shape a UI API: result builders, property
wrappers, opaque types, modifiers, style protocols, static members, labels and
closures, key paths, generics, concurrency; and the AppKit idioms: target-action,
delegates, responders, KVO, Auto Layout, drawing, notifications, hosting and
lifecycle. The look and metrics of controls are in the
[coverage matrix](swiftui-appkit-coverage.md), not here.
**Snapshot:** `main` at `b52708df`, 2026-11-21. Xcode 27.2, Swift 6.4.2,
macOS 27.2.1, `freya-*` and `torin` 0.5.3.
**Reader:** an engineer or AI agent who writes a kit app from SwiftUI
knowledge, or who changes the kit's API.
**Input:** a Swift construct. **Output:** the kit idiom, its limits and the
evidence.

## Steps

1. Find the construct in the [contents](#contents).
2. Read its row: the kit idiom, what is lost and the evidence.
3. When the semantics matter, open the probe named in the Evidence column
   ([section 4](#4-probes)).
4. Before you translate a whole SwiftUI app, read the
   [surprises](#5-surprises-for-a-swiftui-developer).

## Contents

1. [Swift constructs in the kit](#1-swift-constructs-in-the-kit)
2. [AppKit idioms in the kit](#2-appkit-idioms-in-the-kit)
3. [Pairs](#3-pairs)
4. [Probes](#4-probes)
5. [Surprises for a SwiftUI developer](#5-surprises-for-a-swiftui-developer)
6. [Rows to add to the coverage matrix](#6-rows-to-add-to-the-coverage-matrix)
7. [Not verified and decisions](#7-not-verified-and-decisions)

## Summary

A SwiftUI developer does **not** write a kit app without surprises. Of 20
SwiftUI snippets (35 sub-cases) translated with only `agents/` and the
glossary, none maps one to one: 21 differ in shape, 13 have no kit equivalent,
10 differ in name, 6 meet a documentation gap and 4 compile but behave
differently. The glossary lists every public type (414 of 414 found), but no
syntax: it has no row for `ForEach`, `@Binding`, `@Environment`, `.task` or
`withAnimation`.

Findings that decide most translations:

1. **The kit sets, SwiftUI wraps.** A kit modifier changes one node; a SwiftUI
   modifier wraps the view in a new one. Order does not nest in the kit: the
   padding is always inside the background (S1, section 1.3).
2. **Only 10 kit types take modifiers.** Every control is a `Component` with
   its own methods; `.padding`, `.background` and `.help` need a wrapper
   `rect()`.
3. **There are no builders.** Children go in by method; branches of different
   types need `.into_element()`; `for` and `if` are plain Rust.
4. **There is no `Binding`.** Eleven items (the text fields, `ComboBox`,
   `TokenField`, `ImageWell`, `RuleEditor`…) take `State<T>`; the rest take a
   value and a callback and keep a private copy.
5. **Hooks depend on call order**, and `.help` is a hook. A wrong order panics.
   The agent docs never name `use_hook`, `use_memo`, `use_future`, contexts or
   `Callback`.
6. **Nothing inherits.** `.disabled`, `.foreground_style` and `.font` stay on
   one element. Style scopes compose instead of replacing.
7. **AppKit's plumbing is absent:** no responder chain (actions go to every
   handler of the window), no delegates, no close veto, no Auto Layout, no
   notification bus, no hosting of foreign views.
8. **14 statements in `agents/` and the chapters are stale, wrong,
   incomplete or trap-prone** ([5.4](#54-stale-or-wrong-statements)).
   Example: `agents/prompt.md` says `.help` works "on any view".

Method in numbers: 32 Swift programs (18 SwiftUI, 14 AppKit) compiled and run;
six reruns by me matched; 15 sampled claims about the source matched.

## Sources and method

| Source | Used for |
|---|---|
| Kit source `crates/bk/src` (`bk/`), `crates/ctdesign/src` (`ctd/`) | what the API accepts |
| `freya-core-0.5.3/src` (`fc/`), `freya-components-0.5.3/src` (`fcomp/`), `freya-animation-0.5.3/src` (`fa/`), `torin-0.5.3/src` (`torin/`), in `~/.cargo/registry/src/` | state, events, layout |
| Xcode 27.2 SDK: AppKit headers, `SwiftUICore.swiftinterface` | what Apple declares |
| Swift programs built with `swiftc` | what Apple does |
| [Swift Evolution](https://github.com/swiftlang/swift-evolution/tree/main/proposals) and [The Swift Programming Language](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/) | the language rules |

Line numbers are those of the snapshot. A path with no prefix is under
`crates/bk/src`. Rows marked *read* come from the source only; rows with an
S or K number come from a program that ran.

Six read-only agents split the work: view building, state and lifecycle,
AppKit idioms, a documentation audit, SwiftUI probes and AppKit probes. I
sampled their output: I reran S1, S2, S4, S5 and K4, and checked 15 claims in
the kit, Freya and the SDK (the hook rule, `Callback` equality, context
caching, `use_defaults`, the ten forwarding types, that `Button` is a
component, the `Gaps` tuple order, the stepped slider, the `Sheet` type, the
124 symbols, the one-shot `DisclosureGroup`, the handler clash of `.help` and
`.context_menu`, the empty `Spacer` and `ZStack` grep, the `Titlebar` slots
and the `@MainActor` on `View`). All matched.

Swift evidence compiles in Swift 5 language mode, the `swiftc` 6.4 default;
S7 also ran in Swift 6 mode. The probes are outside the repo; the Swift
excerpts in [section 4](#4-probes) are enough to reproduce the results.

## 1. Swift constructs in the kit

Columns: the Swift construct, the kit idiom, what is lost or impossible, and
the evidence. The names follow the [glossary](../barrierkit/20-glossary.md).

### 1.1 Children and result builders

SE-0289 ([result builders](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0289-result-builders.md))
lets a closure list views. The kit has no closure builder.

| Swift | Kit idiom | Lost or impossible | Evidence |
|---|---|---|---|
| `@ViewBuilder` closure with several views | `.child(a).child(b)`, `.children(iter)`, `.maybe_child(opt)` on stacks, `rect()`, `ScrollView` | No tuples, no builder closure. Only `Application::window`, `UtilityWindow::new` and `Popover::new` take a closure that returns a view | `fc/elements/extensions.rs:82-131`, `app/scene.rs:28`, `popover.rs:19`; S2 |
| `if`, `if/else`, `switch` in a builder | a Rust `if` outside the chain; `.maybe_child(cond.then(..))`; arms of different types end in `.into_element()` | `_ConditionalContent` and `Optional` (S2) have no kit type. There is no `()` or `Option` view | `fc/element.rs:285-293` |
| `ForEach(items, id: \.id)` | `.children(iter.map(..))` with `.key(id)` on each child | Swift rejects a raw `for` in a builder (S2). The kit takes any iterator, but unkeyed children match by position and a duplicate key panics in debug builds | `fc/elements/extensions.rs:133-150`, `fc/path_element.rs:152-170` |
| `Group`, `AnyView`, `EmptyView` | `Vec<Element>`, `Element`, no child | no `EmptyView` | `fc/element.rs:259` |
| `Spacer`, `ZStack` | `rect().width(Size::flex(1.))` inside a parent with `Content::flex()`; an absolute `Position` with `.layer(..)` | no named types; `flex` fails silently without the parent flag | `torin/measure.rs:621,703` |
| `ToolbarContentBuilder` (`.toolbar`) | `Titlebar::new().left(view).right(view)`, items in a `ToolGroup`; only under `run_window` | one view per side, a second call replaces the first; no placement | `titlebar.rs:84-90` |
| `CommandsBuilder` (`.commands`) | `MainMenu::new(vec![Menu::new(title, items)])`, `Action`, `use_menu_action` | a data model with string ids; no `CommandGroup(replacing:)` | `main_menu.rs:18-30`, `app/action.rs:130` |
| `SceneBuilder`, `WindowGroup`, `Settings`, `MenuBarExtra` | `Application::new(n).window(t, f)`, `UtilityWindow`, `SettingsWindow`, `StatusItem` | one window per `Application`; kit `Scene` is one window, not a list | `app/scene.rs:10-97` |
| `TableColumnBuilder`, `Picker { ForEach }`, `TabView { .tabItem }`, `Form { Section }` | data builders: `Table::rows(Vec<Vec<String>>)`, `PopUpButton::item(..)`, `TabView::tab(Symbol, label, view)`, `Form::section(Section::titled(..).row(..))` | text cells, labels as `&'static str`, selection by index | `views/table.rs:40-95`, `views/tabs.rs:56`, `views/form.rs:40-205` |
| `Menu { Button; Divider }`, `.contextMenu` | `MenuButton::new(label, Vec<MenuItem>)` with `on_activate`; `.context_menu(items, callback)` | one string-id callback; an item carries no closure | `buttons.rs:36`, `modifiers.rs:12` |

A string passed to `.child("x")` becomes a bare Freya label: black and
unthemed. Use `Text` (`fc/elements/label.rs:73-77`).

### 1.2 Opaque types and views as values

SE-0244 ([opaque result types](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0244-opaque-result-types.md))
gives `some View`. Rust has `impl Trait` for the same job.

| Swift | Kit idiom | Lost or impossible | Evidence |
|---|---|---|---|
| `some View` | `-> impl IntoElement` | one concrete type per function; branches need `.into_element()` | `fc/element.rs:285-293` |
| `struct V: View { var body }` | `#[derive(PartialEq)]` struct with `impl Component { fn render(&self) }` | every field must be `PartialEq`; a `Callback` field is never equal, so the component re-renders on each parent render | `fc/element.rs:379-385`, `fc/event_handler.rs:41-46` |
| `AnyView` | `Element` (`.into_element()`) | none | `fc/element.rs:449` |
| `Content: View` generic | store an `Element`, take `impl IntoElement` in the constructor | a bare generic field needs `PartialEq` | `views/feedback.rs:110-115` |
| `ViewModifier` struct | an extension trait with a blanket impl (`ViewModifiers`), or `.with(..)` | no protocol; `WithExt` covers `rect`, `label`, `paragraph`, `image` only | `modifiers.rs:11-59`, `fc/elements/extensions.rs:153-163` |
| `EquatableView`, `.equatable()` | built in: a child re-renders when its props differ | S9: Swift skips a child only when its fields compare; the kit always compares | `fc/runner.rs:833-842`; S9 |
| `@main struct App` | `fn main` and `Application` | `App` in the prelude is Freya's trait (always equal) | `fc/element.rs:298-320` |

### 1.3 Modifier chaining and order

Swift **wraps**. Each modifier returns a new view that contains the old one,
so the order changes size and paint (S1:
`ModifiedContent<ModifiedContent<Text, _PaddingLayout>, _BackgroundStyleModifier<Color>>`).
The kit **sets**. `padding`, `margin`, `background` and `border` are fields of
one Torin node; the colour is painted over the area inside the margin, so the
padding is always inside it (*read*: `torin/tree_adapter.rs:60-62`,
`fc/elements/rect.rs:493`). A fixed size includes the padding
(`torin/measure.rs:210-231`).

Four kinds of "view" decide which modifiers exist:

| Kind | Examples | Freya modifiers (`.padding`, `.width`, `.background`, `.on_press`) |
|---|---|---|
| Freya element | `rect()`, `label()`, `paragraph()` | yes |
| Kit element wrapper | `Text`, `VStack`, `HStack`, `Badge`, `Divider`, `Avatar`, `Scrim`, `SymbolTile`, `NavigationButtons`, `ContentUnavailableView` | yes (`Text` has no children or background) |
| Kit component | `Button`, `Switch`, `Slider`, `TextField`, `PopUpButton`, `TabView`, `Form`, `List`, `Table`, `Sheet`, `Alert`, `Sidebar` | **no**: own methods, `.key`, `.maybe` only |
| `Image` | `Image::system(..)` | `.size`, `.frame(w, h)`, `.color`, `.style`, `.rendering_mode` |

The ten wrappers are the `forward_element!` and `forward_container!` call
sites (`elements.rs:129,147,163`, `adornments.rs:49,82,142`, `media.rs:40`,
`sidebar.rs:193`, `views/navigation.rs:72`, `views/feedback.rs:67`). The
rule for the rest is in [chapter 7](../barrierkit/07-menus-and-popovers.md):
call the modifier on a `rect()` that holds the control.

Order-sensitive spots in the kit:

| Spot | Rule |
|---|---|
| `Text::style`, `Badge::style`, `Divider::style`, `Avatar::style`, `SymbolTile::style` | They rebuild the view: call them first. Only `Text` is documented (`elements.rs:106-126`, [chapter 2](../barrierkit/02-first-app.md#text-and-stacks)) |
| `.padding`, `.margin`, `.background`, `.corner_radius`, `.width` | Each call **replaces** the last. `.border`, `.shadow`, `.child` **append** (`fc/elements/extensions.rs:639,653,969-1011`) |
| Event handlers | One per event name; the last wins (`fc/elements/extensions.rs:176-180`) |
| `.help(..)` and `.context_menu(..)` | Both write `on_pointer_down` (`tooltip.rs:76`, `fc/elements/extensions.rs:402-414`): the later one replaces the earlier |
| `.control_style(..)`, `.control_size(..)` | They return a `StyleScope`, which has no Freya traits: only another style modifier, `.maybe`, `.map` or `.into_element()` can follow (`modifiers.rs:62`) |

SwiftUI modifier against kit method:

| SwiftUI | Kit or Freya | Same order semantics |
|---|---|---|
| `.frame(width:height:)` | `.width(Size::px(w))`, `.height(..)`; `Image::frame(w, h)` | no: border-box. `.frame(w).padding(p)` is `.width(..)` plus `.margin(p)` |
| `.frame(maxWidth: .infinity)` | `.width(Size::fill())` | partly: later siblings can overflow (`torin/values/size.rs:151`) |
| `.padding(.horizontal, n)` | `.padding((0., n))`: the tuple is (vertical, horizontal); four values are (top, right, bottom, left) | no: replaces, inside the background (`torin/values/gaps.rs:36-52`) |
| `.background(_)` | `.background(Fill)`, `.background_style(ShapeStyle)` | no: one layer; the outer padding is `.margin` |
| `.overlay` | none: an absolute child with `.layer(..)` | no |
| `.border` | `.border(Option<Border>)` | partly |
| `.cornerRadius`, `.clipShape` | `.corner_radius(r)` plus `.overflow(Overflow::Clip)` | partly: the radius alone does not clip (`fc/data.rs:291-297`) |
| `.opacity`, `.scaleEffect`, `.rotationEffect` | `.opacity`, `.scale`, `.rotation(deg)` | yes (layout-neutral) |
| `.offset(x:y:)` | `Position::new_absolute().left(x).top(y)`; **not** `offset_x`, which shifts the children | no |
| `.shadow` | `.shadow`, `.shadows(Vec<ShadowStyle>)` | partly |
| `.foregroundStyle`, `.font` | `.foreground_style(ShapeStyle)`, `Text::font(Font)` | no: one element, never inherited (S3) |
| `.lineLimit` | `Text::line_limit(n)` | yes |
| `.disabled` | `.disabled(bool)` on each control | no: no propagation |
| `.hidden`, `.tag`, `.mask`, `.layoutPriority` | none | none |
| `.help`, `.contextMenu` | `.help("..")` (a hook), `.context_menu(items, callback)` | partly: the clash above |
| `.onTapGesture` | `.on_press(..)` (also Space and Enter) | partly: one handler |
| `.fixedSize` | the default `Size::Inner` hugs content | partly |
| `.aspectRatio` | none for views: `AspectRatio` is for images only (`fc/elements/image.rs:108`) | no |
| `.blur` | `.backdrop_blur(f32)` blurs what is behind, not the view | no |

### 1.4 Style protocols

| Swift | Kit idiom | Lost or impossible | Evidence |
|---|---|---|---|
| `.buttonStyle(.bordered)` | `Button::style(\|s\| ..)`, or a preset: `.style(ButtonStyle::bordered_prominent)`. The default `ButtonStyle::tengen()` is `.bordered` | no `ButtonStyle::bordered` | `styles/button.rs`; S5; [`counter.rs`](../../crates/bk/examples/counter.rs) line 11 |
| custom `ButtonStyle.makeBody(configuration:)` | a plain struct of fields; `StateValue<ShapeStyle>` holds a value per hover, press, focus and disabled | no body: the label cannot be restructured | `styles/button.rs:40-70`; S5 |
| `.buttonStyle(_)` on a subtree | `.control_style(\|s: ButtonStyle\| ..)`, or `Theme::style` for the whole app | the closure's type picks the style; the scope has no other modifiers | [`styled_control.rs`](../../crates/bk/examples/styled_control.rs) lines 28-31 |
| the nearest ancestor wins | scopes **compose**: the inner closure sees the outer result | SwiftUI replaces: a child's own `.buttonStyle` wins (S5) | `styles/theme.rs:184-205` |
| `ToggleStyle`, `LabelStyle`, `PickerStyle` | one `*Style` struct per control (`SwitchStyle`, `PopUpButtonStyle`) | no protocols; a style that spans controls does not exist | [glossary](../barrierkit/20-glossary.md), ADR 0064 in [`decisions.md`](../decisions.md) |
| `.controlSize(.small)` | `.control_size(ControlSize::Small)` on a view, `.size(..)` on a control; 13 families | `.size` wins over the scope | [`control_sizes.rs`](../../crates/bk/examples/control_sizes.rs) lines 14-22 |
| precedence | tengen, theme, scopes (outermost first), instance, accessibility | `resolve` is crate-private: an app cannot make its own themed control (UNVERIFIED: no other path) | `styles.rs:170-171`, [`agents/styles.md`](../../agents/styles.md#levels) |

### 1.5 Static and implicit members

SE-0287 ([implicit member chains](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0287-implicit-member-chains.md))
and SE-0299 ([static member lookup in generic contexts](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0299-extend-generic-static-member-lookup.md))
make `.title` and `.buttonStyle(.bordered)` work. Rust has no `.member`
shorthand: the kit uses constants and full paths.

| Swift | Kit | Note |
|---|---|---|
| `.title`, `.largeTitle`, `.body` | `TITLE`, `LARGE_TITLE`, `BODY`: 12 `Font` constants | `Font` is a plain struct: no `.bold()`; `.system(size:)` is `Font { size: 15, ..BODY }` (`ctd/lib.rs:51`) |
| `.secondary`, `.primary` | `ShapeStyle::Role(Role::SecondaryLabel)`; 31 `Role`s; 7 `ShapeStyle` variants | S6: Swift resolves `.secondary` by the parameter type |
| `.accentColor`, `.tint` | `ShapeStyle::Accent` | named colours (`.red`) do not exist: `Rgba` has `TRANSPARENT` only |
| `.regularMaterial` | `ShapeStyle::Material(Thickness::Regular)` | |
| `Image(systemName:)` | `Image::system(Symbol::X)`: 124 variants, no string lookup | S6: an unknown name is silent in Swift; in the kit it does not compile |
| `.leading`, `.infinity` | `Alignment::Start`, `Size::fill()` | two orientation enums: `Direction` (layout) and `Axis` (styles) |

S5 shows the error messages. Swift: `reference to member 'nonexistent' cannot
be resolved without a contextual type`. A concrete Rust enum gives a precise
error for free.

### 1.6 Labels and trailing closures

SE-0279 ([multiple trailing closures](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0279-multiple-trailing-closures.md))
gives `Button("Save") { }` and `.alert(..) { } message: { }`. The kit puts the
label first, then chains the options and the callback.

Two rules cover most controls. The first argument of `new` is the accessible
label. It is a `&'static str` in 92 public functions and an `impl Into<String>`
in 28 (`grep -rn "pub fn [a-z_]*(.*&'static str"`). A formatted label needs
`i18n::intern(String)` ([chapter 17](../barrierkit/17-languages.md)), which
`agents/` never mentions. Callbacks take one argument, even when it is unused:
`move |_| ..`.

| SwiftUI | Kit | Value shape |
|---|---|---|
| `Text("x")` | `Text::new(impl Into<Cow<'static, str>>)`, `.font`, `.line_limit`, `.style` | |
| `Image(systemName:)` | `Image::system(Symbol)`, `Image::data`, `Image::rgba` | |
| `Label` | none: `HStack` of `Image` and `Text`. `Label` in the prelude is Freya's text element | |
| `Button("t") { }` | `Button::new(impl Into<String>).on_press(cb)`, `Button::system_image(..)` | the action is a method |
| `Toggle(isOn:)` | `Switch::new(l).on(b).on_toggle(cb)`, `Checkbox::new(l).checked(b)` | `bool` plus callback |
| `Picker` | `PopUpButton::new(l).item(&str).selected(usize).on_change(cb)`, `SegmentedControl`, `RadioGroup` | `usize`; no `.tag` |
| `Slider(value:in:step:)` | free: `Slider::new(l).range(a..=b).value(f32).on_change(cb)`; stepped: `Slider::stepped(l, steps)` | `stepped` takes a **count**; the value is an index; `.range` after it changes the count (`controls/slider.rs:44-50,147-149`) |
| `Stepper` | `Stepper::new(l, min, max).value(i64)` | `i64` |
| `TextField(text:)` | `TextField::new(State<String>, placeholder)`, `.on_submit` | `State<String>`; no `on_change` |
| `SecureField` | `SecureField::new(State<PasswordField>, l)` | not a `String` |
| `TextEditor` | `TextEditor::new(State<String>, ..)` | |
| `DatePicker`, `ColorPicker` | `DatePicker::new(l, CivilDateTime)`, `ColorPicker::new(l, Rgba)` | own types |
| `ProgressView`, `Gauge` | `ProgressView::determinate(l, f32)`, `Gauge::new(l, f32, String)` | `f32` from 0 to 1 |
| `Link` | `Link::new(&str).on_press(cb)` | no URL |
| `Menu` | `MenuButton::new(l, Vec<MenuItem>).on_activate(cb)` | id string. Kit `Menu` is `CommandMenu` |
| `List`, `Table` | `List::new(l, Vec<String>).selected(Vec<usize>)`, `Table::new(l, headers).rows(Vec<Vec<String>>).on_sort(cb)` | text cells; the app sorts |
| `Form`, `Section` | `Form::new().section(Section::titled(..).row(Row::toggle(..)))` | [`settings_form.rs`](../../crates/bk/examples/settings_form.rs) |
| `GroupBox`, `DisclosureGroup` | `GroupBox::new(Option<&str>).child(v)`, `DisclosureGroup::new(l).expanded(bool).child(v)` | `expanded` seeds the state once (`views/group.rs:119-120`) |
| `TabView`, `NavigationSplitView` | `TabView::new(l).tab(Symbol, &str, view).selected(usize)`, `SplitView::new(sidebar, detail)` | index; a sidebar item carries `.selected(bool)` |
| `ScrollView`, `Divider` | `ScrollView::new().direction(Direction)`, `Divider::new()` | |

### 1.7 Key paths, generics and identity

| Swift | Kit idiom | Lost or impossible | Evidence |
|---|---|---|---|
| `ForEach(items, id: \.id)`, `Identifiable` | `.key(x.id)`: the value is hashed with the element's `TypeId` | no `Identifiable`; the key must be `Hash` | `fc/elements/extensions.rs:140-150`; S9 |
| `.id(_:)` resets state | `.key(..)` changed: the scope is dropped, its tasks cancel and `use_drop` runs | none | `fc/runner.rs:1267-1285`; S9 |
| `List(selection:)`, `Picker(selection:)` of any `Hashable` | `usize`, `Vec<usize>`, or a tree path `Vec<Vec<usize>>` | no `.tag`; map the index to your value | `views/list.rs:43-55`, `views/tree.rs:150-199` |
| `Picker<SelectionValue>`, `ForEach<Data, ID>` | none: no public view or control is generic | only helpers are (`StateValue<T>`, `UndoManager<T>`, `Publisher<T>`, `History<T>`) | `grep -rnE "pub (struct\|enum\|trait\|fn\|type) [A-Za-z_0-9]+<" crates/bk/src` |
| `TableColumn(value: \.name)`, `KeyPathComparator`, `.sorted(using:)` | pre-flatten rows to strings; `Table::on_sort(SortOrder { column, ascending })` | no key paths; the app sorts and maps the index after sorting | `views/table.rs:65`; S6 |
| `$model.items[0].name` (SE-0252 key path member lookup) | none; `Writable::map` projects a field but keeps the parent's subscribers | no `$` projection | `fc/lifecycle/writable.rs:162-172`; S8 |
| `.map(\.name)` (SE-0249 key paths as functions) | a closure | Rust has no key-path syntax | S6 |

### 1.8 Property wrappers

SE-0258 ([property wrappers](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0258-property-wrappers.md))
hide storage behind `@State` and friends. Rust has no such attribute: the kit
uses hooks that return a handle.

| Swift | Kit or Freya | Lost or impossible | Trap |
|---|---|---|---|
| `@State` | `let mut x = use_state(\|\| v)`: a `Copy` `State<T>`; `read()`, `write()`, `set(..)`, `peek()` ([`counter.rs`](../../crates/bk/examples/counter.rs) lines 6-13) | `x += 1` becomes `*x.write() += 1`; whole-value tracking | forgetting `mut`; a read inside a handler subscribes nothing (`fc/lifecycle/state.rs:306,366`; `fc/runner.rs:656-690`) |
| `@Binding`, `$x` | pass the `State<T>` itself; or value plus callback ([`toggle.rs`](../../crates/bk/examples/toggle.rs) lines 6-13) | no `Binding` type. Freya has `Writable` and `WritableSetter`, but no kit control takes them | text-like controls take `State<T>`; the rest keep a private copy (`controlled.rs:14-19`) |
| `.constant(_)` | `Readable::from_value` | a constant `.value(x)` does not snap back as `.constant` does | S8 |
| `@Bindable` | a `State`, or `Writable::map` | no `$model.field` | a field write wakes every reader |
| `@Observable` | a `Copy` struct of `State`s, the kit's own idiom (`interaction.rs:7-13`, `app/undo.rs`) | no derive; per-property tracking is manual. S4: `@Observable` re-runs only the readers of the changed property | `grep -rn "Observable"` over crates, docs and agents finds nothing |
| `ObservableObject`, `@Published`, `@StateObject`, `@ObservedObject` | the same struct of `State`s | S4: SwiftUI re-runs on any property | |
| `@Environment`, `@EnvironmentObject` | `use_provide_context`, `use_consume::<T>()` | one value per type per scope; panics if missing | both are `use_hook`-cached: read once; make it reactive by putting a `State` inside (`fc/lifecycle/context.rs:243,279`) |
| `@FocusState` | `use_a11y()`, `.a11y_id(id)`, `id.request_focus()`, `use_focus(id)` | no enum of fields; `TextField` and `SecureField` have only `.auto_focus(bool)` | only `SearchField` takes an id (`controls/text_field.rs:399`) |
| `@FocusedValue`, `@Namespace`, `@Query` | none | | |
| `@AppStorage`, `@SceneStorage` | `use_defaults::<S>(app)`, `update_defaults`, `load_restoration`, `save_restoration` | one TOML struct per app, not per key; `use_defaults` is read-only and updates after the file watcher fires | `app/defaults.rs:11-82` |
| `@GestureState` | a `State<bool>` cleared in `on_global_pointer_up` | no recogniser | `interaction.rs:62` |
| `@ScaledMetric` | `Font::scaled(resolved().text_size)` | fonts only | `elements.rs:111` |

The rule of hooks. `use_hook` is index based: an extra hook after the first
render, or one of another type, panics (`fc/lifecycle/base.rs:75-95`). A hook
belongs to the enclosing component, so a plain function that calls hooks
shares its caller's list. `.help` hides a hook (`modifiers.rs:20-22`), so
`ForEach { row.help(..) }` and `if x { b.help(..) }` break the rule. The
checklist names it ([`checklist.md`](../../agents/checklist.md), question 6)
but lists only `use_state` and `use_defaults`.

SwiftUI has no such rule: a wrapper is keyed by its property, not its order.

### 1.9 Environment

Contexts resolve by `TypeId`, nearest ancestor first (`fc/lifecycle/context.rs:210-232`).
Popups, sheets and tooltips run in separate runners and take a snapshot of the
theme when they open (`platform/popup.rs:151-152`, `tooltip.rs:48`).

| Swift key | Read in the kit | Override for a subtree | Nearest wins |
|---|---|---|---|
| `\.colorScheme`, contrast, accent | `theme::resolved()`, `Palette::current()` | `use_appearance(\|c\| ..)`; a `Backdrop` provides its own palette | yes |
| `\.controlSize` | `.control_size(..)` | the same, composing with the parent | yes, cumulative |
| button and slider styles | `.control_style(..)`, `Application::theme` | the same | yes, cumulative |
| `\.isEnabled` | `.disabled(bool)` per control | none | no inheritance |
| `\.font`, `\.foregroundStyle` | per element | none | no inheritance (S3 shows Swift inherits) |
| `\.dynamicTypeSize` | `resolved().text_size` | none | global |
| `\.locale` | `tr!`, `i18n::use_app_language` | none: a process-wide `OnceLock`, first call wins | global |
| `\.layoutDirection` | a field of each control style | `.control_style(..)` | per family |
| `\.accessibilityReduceMotion` | `use_reduced_motion()` | through `use_appearance` | yes |
| `\.dismiss` | none: a `Sheet` closes when you stop drawing it; `WindowControls::get().close()` | | |
| `\.openWindow`, `\.openURL` | `UtilityWindow::new(..).open()`, `WindowControls::open_window`; `portal::open_uri(..).await` | | |
| custom `EnvironmentKey` | a `Clone` type in `use_provide_context` | `provide_context` in the subtree | yes |

S3 gives the Swift rule: `.environment(_:_:)` reaches only what it wraps.
A closure-taking modifier applied earlier in the chain (`.background { }`,
`.overlay { }`) does not see it; sheet, popover and toolbar content see the
environment of their modifier's position.

### 1.10 Lifecycle and effects

| Swift | Kit or Freya | When it runs | Evidence |
|---|---|---|---|
| `init` | the `Component` struct, built by the parent | on every parent render | |
| `@State` initial value | `use_state(\|\| v)`, `use_hook(\|\| ..)` | once, in the first render | `fc/lifecycle/base.rs:75-105` |
| `deinit`, `.onDisappear` | `use_drop(..)` | the scope is removed | `fc/lifecycle/base.rs:126` |
| `.onAppear` | `use_hook(..)` in render, or `use_after_side_effect` | after the render that created it | S7 shows `.onAppear` before `.task` |
| `.task { }` | `use_hook(\|\| spawn(async { .. }))` | polled in the next pass; cancelled on unmount | `fc/lifecycle/task.rs:87-120` |
| `.task(id:)` | `use_future(..)` reading a `State` **outside** the async block | restarts when that state changes | `fc/lifecycle/future_task.rs:146` |
| `.onChange(of:)` | `use_side_effect_with_deps(&v, ..)`; the old value from `use_previous_and_current` | fires at creation too | `fc/lifecycle/effect.rs:268` |
| `.onReceive` | `use_published(initial, start)` | the UI thread; the last value wins | `app/published.rs:19-39` |
| `.onSubmit` | `on_submit` on each field | not a subtree modifier | `controls/text_field.rs:194` |
| `.refreshable` | none | | |
| computed property | `use_memo(..)` | first value in render, then recomputed in a task | `fc/lifecycle/memo.rs:29` |

A scope re-renders when a `State` it read changes, or when its parent
re-rendered and the props differ (`fc/runner.rs:833-842`). `Callback` and
`EventHandler` are never equal (`fc/event_handler.rs:41-46,144-148`): a
control that holds a handler re-renders on every parent render. The window
closure passed to `Application::window` re-runs only through its own reads
(`AppComponent` is always equal, `fc/element.rs:316`).

### 1.11 Concurrency

| Swift | Kit or Freya | Lost or impossible | Evidence |
|---|---|---|---|
| `@MainActor` on `View` | everything runs on the UI thread; `State` is not `Send` (*read*, derived) | no isolation to declare; S7: `View` is `@preconcurrency @MainActor`, and a nonisolated access gives warnings, not errors, even in Swift 6 mode | S7 |
| `Task { }` | `spawn(async { .. })`, `spawn_in_window`, `spawn_global` | futures are `'static`, not `Send` (`LocalBoxFuture`); `spawn` ends with the component | `fc/lifecycle/task.rs:54,72,87` |
| `Task.detached` | `thread(\|\| ..).await` (the work is `Send + 'static`) | | `fc/io/thread.rs:4-8` |
| `.task` cancellation | cooperative in Swift (S7); `spawn` is dropped at unmount | | |
| `actor`, `Sendable`, `AsyncStream` | none; the only cross-thread type is `Arc<Mutex<T>>` inside `Publisher` | | `app/published.rs:7-17` |
| a result back to the UI | `use_published(initial, start)`, `use_wake`, `Timers::wakeup` | last value wins; a timer has no cancel handle | `app/published.rs`, `platform/timer.rs:52-79` |
| `Timer`, `TimelineView` | `Timers::get().every(..)`, `use_timeline(active, every)` ([`stopwatch.rs`](../../crates/bk/examples/stopwatch.rs) line 16) | periodic only | `app/timeline.rs:9` |

The thread rule is in [chapter 13](../barrierkit/13-app.md) and
[chapter 8](../barrierkit/08-windows.md); `agents/` does not repeat it.

### 1.12 Animation, focus and gestures

| Swift | Kit or Freya | Status |
|---|---|---|
| `withAnimation`, `Transaction` | none; `Animation::None` and `motion_ms` honour reduced motion (`animation.rs:24`) | missing |
| `.animation(_, value:)` | `use_animation_transition(state, \|from, to\| ..)` from `freya-animation` | per value; the app adds `freya-animation` to its `Cargo.toml` |
| `.transition`, `matchedGeometryEffect` | none: removal is immediate (`fc/runner.rs:1240-1285`) | missing |
| `Animatable` | implement `AnimatedValue` (`fa/hook.rs:137-146`); `Spring` | manual |
| `.keyboardShortcut` | `Action` and `Shortcut` through an enabled `MainMenu` item only: "A shortcut that is in no menu does nothing" ([chapter 13](../barrierkit/13-app.md#actions-and-shortcuts)) | partial |
| `.onKeyPress`, `.onHover` | `on_key_down`, `on_pointer_enter` plus `on_pointer_leave` (no `Bool`) | partial |
| `.onTapGesture` | `on_press`; a double click with `EventsCombos` | partial |
| `DragGesture`, `MagnifyGesture`, `LongPressGesture` | none; raw `on_pointer_*` plus `State` | missing |
| `.draggable`, `.dropDestination` | out: `use_drag_source().attach(..)`; in: `on_file_drop`, files only | partial |

### 1.13 Layout containers

| Swift | Kit or Torin | Lost or impossible |
|---|---|---|
| `VStack`, `HStack` | `VStack::new()`, `HStack::new()`: a `rect()` with centre alignment and `SPACE_3` spacing; `rect()` has spacing 0 | per-child alignment, baselines, alignment guides |
| `Spacer`, `ZStack`, `overlay` | see 1.1 | no named types |
| `LazyVStack`, `Grid`, `ViewThatFits`, `Layout` | none; `Table` virtualises its rows; `Content::Wrap` wraps | no custom layouts |
| `GeometryReader` | `.on_sized(..)` into a `State`: the value arrives one pass late | no proposal-based size |
| `.frame(maxWidth: .infinity)` | `Size::fill()`; `percent`, `min_*`, `max_*` | two `fill` siblings overflow |
| `Size` literals | `Size::px(100.)`: there is no `From<f32>` | `.width(100.)` does not compile |

Torin has eight `Size` variants, two `Direction`s, six `Alignment`s (they act
on the whole child group), four `Content` modes and three `Position`s; there
is no right-to-left mode (`torin/values/size.rs:117`).

## 2. AppKit idioms in the kit

Columns: the AppKit idiom with what the probe measured, the kit equivalent and
the gap. K numbers are in [section 4](#4-probes).

| Idiom | AppKit, measured | Kit | Gap |
|---|---|---|---|
| Target-action | `NSControl.target` and `action`; a nil target walks the key window's responder chain, then `NSApp`, then its delegate; `sendAction` returns `false` if nobody answers (K1) | `.on_press(closure)`; `Action` plus `use_menu_action`, which calls **every** handler of the window (`app/action.rs:116-145`) | no sender, no targeting. A shortcut in no menu does nothing, and a menu match swallows the key from the focused control (`platform/input/seat.rs:515-560`) |
| Responder chain | `View C, B, A, NSViewController, NSWindow, NSWindowController`; `sendAction` adds the window delegate, `NSApp` and the app delegate (K2) | none: `grep -ri responder crates docs/barrierkit agents` finds nothing. Focus is Freya's (`a11y_focusable`, `request_focus`, `use_focus`); events bubble to the parent until `stop_propagation` (`fc/runner.rs:536-575`) | no `validateMenuItem` (menu state is pushed with `use_menu_enabled`); menus belong to a window; the Command key is Ctrl |
| Key view loop | geometric, top to bottom and left to right, with the content view as a stop (K2) | Tab walks the tree in pre-order over enabled focusable nodes (`fcomp/integration.rs:13-22`) | no `nextKeyView`; the order is the tree order |
| Delegates, data sources | `NSTableViewDataSource` asks only the visible rows plus about 300 height samples; cells recycle until `reloadData`; an outline asks children on expand (K9) | data by value: `Table::rows(Vec<Vec<String>>)`, `OutlineView` takes a whole `Vec<TreeNode>`; events by `on_*`; selection by index | no lazy provider, no `reloadData`, no cell views; `Table` virtualises rendering only (`views/table.rs:402`) |
| Text delegates | `controlTextDidChange`, `doCommandBy` | `TextField::on_submit`; `TextView::on_change`, `on_key` | no per-keystroke callback on `TextField` |
| Window and app delegates | `windowShouldClose`, `applicationShouldTerminate` and `application(_:open:)` fire in a fixed order (K8) | none: `WindowOptions` has no callback field; `request_close` closes at once; the process ends with the last window | no close veto, no quit API, no open-URL event (`platform/window.rs:54-69,761-769`) |
| KVO, KVC, Cocoa Bindings | fires on equal values and on the setting thread; a binding pushes model to UI at once, UI to model on commit (K3) | `State::read` subscribes; `use_memo`, `use_side_effect`, `use_previous_and_current`; `TextField(State<String>)` is a one-property binding | no KVC, no key paths, no controllers. `grep -rn "keypath\|KVO\|addObserver"` over crates, docs and agents finds nothing |
| Auto Layout | priorities decide which constraint is kept; lower hugging stretches; the conflict log appears at activation (K4) | Torin: single-pass boxes with `Size`, `Content`, `Position`. `NSStackView` fill-equally is `Size::flex(1.0)` plus `Content::flex()`; equal spacing is `Alignment::SpaceBetween` | no constraints, anchors, guides, priorities, hugging, baselines, `NSGridView`. `AspectRatio` is for images only |
| `draw(_:)` | invalidation is lazy and unioned; `dirtyRect` can exceed the bounds; `wantsUpdateLayer` replaces `draw` (K5) | `Canvas::new(revision, painter)` repaints when `revision` changes (`canvas.rs:36,59-63`); a custom element implements `ElementExt` | a `Canvas` has no events or hit test; it is y-down; no dirty rects; the painter is raw Skia |
| `NotificationCenter` | observers run on the poster's thread; `queue: .main` from a background thread blocks the poster; a released token keeps delivering (K6) | none. A context that holds a `State`; `use_published` from threads (the last value only); `use_menu_action` for ids | no bus. `notifications::post` is a user-notification service |
| `NSViewRepresentable`, `NSHostingView` | SwiftUI evaluates lazily; `update` runs only when the value changes; `dismantle` follows the SwiftUI tree, not the host (K7) | none (`grep -rn "Representable\|NSHosting\|subsurface" crates docs agents` finds nothing). The unit of embedding is a `Component`, a `Canvas` or an `ElementExt` | no foreign content |
| View and window lifecycle | `loadView` and `viewDidLoad` at assignment; `viewWillAppear` and `viewDidAppear` inside `orderFront`; `windowShouldClose` before `windowWillClose` (K8) | `use_hook` and `use_drop`; `on_visible` and `on_hidden`; `on_sized`; `WindowControls` status for focus and full screen | no `NSWindowController`, no `NSDocument`; observe only |
| Pasteboard, drag, undo, panels | `undo:` is answered by the `NSWindow`; `copy:` and `paste:` by the first-responder text view; undo groups by run-loop turn (K10) | `Clipboard::set/get(String)` (text only); `use_drag_source` (out) and `on_file_drop` (files in); `use_undo(state)` snapshots one `State`; `portal::open_files`, `save_file` | no typed pasteboard, no undo groups or names, two `use_undo` managers in a window both fire |

Two more rows from the headers: `NSCursor` is `.cursor(CursorIcon)` from
Freya, not in the glossary; `NSTrackingArea` is `on_pointer_enter` and
`on_pointer_leave` per element.

AppKit names that the [glossary](../barrierkit/20-glossary.md) and the
[coverage matrix](swiftui-appkit-coverage.md) both lack (a `grep -w` finds 0
in each): `NSResponder`, `NSUserInterfaceValidations`, `NSActionCell`, the
delegate and data-source protocols of table, outline, collection, browser,
text field, text view, window, split view, search field, token field and combo
box, `NSObjectController`, `NSArrayController`, `NSTreeController`,
`NSUserDefaultsController`, `NSKeyValueObserving`, `NSLayoutConstraint`,
`NSLayoutAnchor`, `NSLayoutGuide`, `CALayer`, `NotificationCenter`,
`DistributedNotificationCenter`, `NSViewRepresentable`,
`NSViewControllerRepresentable`, `NSHostingView`, `NSHostingController`,
`NSHostingMenu`, `NSViewController` and `NSCursor`.

## 3. Pairs

Each pair has the Swift line that does not carry over, and the recipe that
does the same job. The recipes build in `./dev.sh check`.

```swift
// ✗ a Binding: the kit has none
@State var count = 0
Button("Add one") { count += 1 }
```

✓ [`counter.rs`](../../crates/bk/examples/counter.rs) lines 6-13: `use_state`,
`.on_press`, `*count.write() += 1`.

```swift
// ✗ Toggle(isOn: $wifi)
Toggle("Wi-Fi", isOn: $wifi)
```

✓ [`toggle.rs`](../../crates/bk/examples/toggle.rs) lines 6-13: `Switch` takes
`.on(value)` and `.on_toggle(callback)`.

```swift
// ✗ a view with a body
struct Greeting: View { let name: String; var body: some View { Text(name) } }
```

✓ [`reusable_view.rs`](../../crates/bk/examples/reusable_view.rs) lines 3-12: a
`PartialEq` struct and `impl Component`.

```swift
// ✗ stack alignment and spacing as initializer arguments
VStack(alignment: .leading, spacing: 4) { Text("Ana"); HStack { Text("Madrid") } }
```

✓ [`rows_and_columns.rs`](../../crates/bk/examples/rows_and_columns.rs) lines 6-14:
`.cross_align(Alignment::Start)` and `.spacing(SPACE_1)`.

```swift
// ✗ Form { Section { Toggle } }
Form { Section("General") { Toggle("Sounds", isOn: $sounds) } }
```

✓ [`settings_form.rs`](../../crates/bk/examples/settings_form.rs) lines 7-12:
`Form::new().section(Section::titled(..).row(Row::toggle(..)))`.

```swift
// ✗ .controlSize on a container
HStack { Button("Save"); Toggle("Wi-Fi", isOn: $on) }.controlSize(.small)
```

✓ [`control_sizes.rs`](../../crates/bk/examples/control_sizes.rs) lines 14-22:
`.control_size(size)` on the `HStack`.

```swift
// ✗ a ButtonStyle on one control and on a subtree
Button("Reset").buttonStyle(.borderedProminent)
VStack { … }.buttonStyle(Pill())
```

✓ [`styled_control.rs`](../../crates/bk/examples/styled_control.rs) lines 6-31: a
theme override, an instance `.style(..)` and a `.control_style(..)` scope.
There is no `makeBody`.

```swift
// ✗ ForEach over state, redrawn by a TimelineView
TimelineView(.periodic(from: .now, by: 0.01)) { _ in
    ForEach(laps) { lap in Text(lap.text) }
}
```

✓ [`stopwatch.rs`](../../crates/bk/examples/stopwatch.rs) lines 16 and 55-59:
`use_timeline(running, TICK)` and `.children(iter.map(..))`. The recipe gives
no `.key(..)`: add one for a list that reorders.

```swift
// ✗ .toolbar { ToolbarItemGroup { … } }
.toolbar { ToolbarItemGroup { Button("Add", systemImage: "plus") { }; Button("More", systemImage: "ellipsis") { } } }
```

✓ [`titlebar_over_gradient.rs`](../../crates/bk/examples/titlebar_over_gradient.rs)
lines 9-33, or [`custom_window.rs`](../../crates/bk/examples/custom_window.rs):
`Titlebar::new().right(ToolGroup::new().child(..))` inside `run_window`.

```swift
// ✗ @main App with a WindowGroup
@main struct HelloApp: App { var body: some Scene { WindowGroup { Text("Hello") } } }
```

✓ [`hello_world.rs`](../../crates/bk/examples/hello_world.rs) lines 4-8:
`Application::new(..).window(title, closure).run()`.

## 4. Probes

Each probe is a minimal Swift program built with `swiftc`. S numbers are
SwiftUI, K numbers are AppKit. The probes ran with the app in the accessory
policy, windows at alpha 0 and no focus change.

### S1 Modifier order

```swift
Text("a").padding().background(Color.red)   // ModifiedContent<ModifiedContent<Text, _PaddingLayout>, _BackgroundStyleModifier<Color>>
Text("a").background(Color.red).padding()   // the two layers swap
```

| Chain | `fittingSize` or pixel |
|---|---|
| `.frame(width: 100).padding(10)` | 120 × 36 |
| `.padding(10).frame(width: 100)` | 100 × 36 |
| `.padding(10).background(.red)`, pixel (2,2) | red: the padding is painted |
| `.background(.red).padding(10)`, pixel (2,2) | clear |
| `.padding(10).border(.blue)` against `.border(.blue).padding(10)` | the border moves from x = 0 to x = 10 |
| `.clipShape(circle).shadow` against `.shadow.clipShape(circle)` | the first keeps the shadow; the second clips it |

A flat bag of fields cannot express both orders: the kit expresses only the
first of each pair (section 1.3).

### S2 ViewBuilder lowering

| Source | Type or call |
|---|---|
| `if c { Text } else { Image }` | `_ConditionalContent<Text, Image>` |
| `if c { Text }` | `Optional<Text>` |
| `switch` with three cases | `_ConditionalContent<_ConditionalContent<A, B>, C>` |
| `if #available` | `Optional<AnyView>` |
| `ForEach` | `ForEach<Range<Int>, Int, Text>` |
| twelve children | `TupleContent<Pack{Text x12}>` on the macOS 27 target; `TupleView<(Text, …)>` with `-target arm64-apple-macos15.0` |
| `for i in 0..<3 { Text("\(i)") }` | `error: closure containing control flow statement cannot be used with result builder 'ViewBuilder'` |

The custom builder (`s2c`) traces SE-0289: each `{ }` block calls
`buildBlock`; `if` without `else` calls `buildOptional`; `if/else` calls
`buildEither(first:/second:)`; a three-case `switch` builds a balanced tree
(`first(second(x))`); `for-in` builds each iteration as a block and then
`buildArray`; `if #available` calls `buildLimitedAvailability` and
`buildOptional`; `buildFinalResult` ends the function. Only expressions get
`buildExpression`: a `let` does not. Plain Rust blocks cannot be intercepted
like this, so the kit uses explicit constructors.

### S3 Environment

A custom `EnvironmentKey` set at the root reads 1; a nested override reads 2,
then 3; siblings still read 1. Order of `.environment`:

| Chain | Value seen |
|---|---|
| `.background { BG }.environment(7)` | 7 |
| `.environment(6).background { BG }` | 1 (the outer value) |
| `.overlay` | the same as `.background` |
| sheet content, env set outside the modifier | reaches the content |
| sheet content, env set only on the presenting child | 1, not 5 |

`font`, `controlSize` and `foregroundStyle` propagate like environment values.
`padding` and `background` do not: `VStack { 2 boxes }.padding(10)` is 40 × 60
(one ring), against 40 × 80 with per-child padding.

### S4 Observation

```swift
@Observable final class M { var a = 0; var b = 0 }
withObservationTracking { _ = m.a } onChange: { fired += 1 }
m.b += 1   // fired = 0: b was not read
m.a += 1   // fired = 1, with willSet semantics; a second change does not fire
```

The tracker is one-shot and fires on the mutating thread. In a hosted view:
an `@Observable` reader of `a` does not run `body` when `b` changes; a
`@State` struct re-runs on any field change (not on `s.b = s.b`);
`@State` holding a class mutated in place does not re-run; an
`ObservableObject` re-runs on any `@Published`. A kit `State<T>` behaves like
the struct: whole value (`fc/lifecycle/state.rs`).

### S5 Style protocols

The parent's `.buttonStyle` reaches the descendant button; a child's own
`.buttonStyle` wins; an inner container wins over an outer one. A synthetic
mouse-down calls `makeBody` with `isPressed = true`, and the action fires on
mouse-up. A custom `ToggleStyle` re-runs `makeBody` after the toggle flips.
`.bordered` resolves through `S: PrimitiveButtonStyle` to `BorderedButtonStyle`
(SE-0299). Diagnostics:

```text
buttonStyle(.nonexistent)  reference to member 'nonexistent' cannot be resolved without a contextual type
font(.titel)               type 'Font?' has no member 'titel'
foregroundStyle(.reed)     type 'ShapeStyle' has no member 'reed'
padding(.horizonal, 4)     type 'Edge.Set' has no member 'horizonal'
```

### S6 Key paths and implicit members

`\.id` is `KeyPath<Item, Int>`; `\.name` is `WritableKeyPath`; `\.self` is
`WritableKeyPath<Item, Item>`. `ForEach(items, id: \.id)` and
`ForEach(items)` produce the same type. `map(\.name)` works because a key path
acts as a function (SE-0249). `Font.title == Font.system(.title)`. `.secondary`
resolves by the contextual type: `HierarchicalShapeStyle` for a `ShapeStyle`
parameter, `Color` for a `Color` parameter. `Image(systemName:)` with an
unknown symbol is silent (a 0 × 0 image).

### S7 Concurrency

`View` is `@preconcurrency @MainActor` (`SwiftUICore.swiftinterface`, the
`View` protocol). A nonisolated function that touches `body`, `@State` or
`init` gets **warnings** in Swift 6 mode (`main actor-isolated property 'body'
can not be referenced from a nonisolated context`); a user-declared
`@MainActor` class gets an error. `Task { }` in a view method inherits the
main actor; `Task.detached` and `Task { @concurrent in }` run off the main
thread. `.task` starts after `.onAppear`. On removal `.onDisappear` runs
first, then the task is cancelled (`Task.isCancelled`; `Task.sleep` throws
`CancellationError`). A change of `.task(id:)` starts the new task and
cancels the old one cooperatively.

### S8 Binding

`MemoryLayout<Binding<Int>>.size` is 24. `Mirror` shows a `transaction`, a
`location` reference (`LocationBox<FunctionalLocation<Int>>`, or
`ConstantLocation` for `.constant`) and a cached `_value`: a reference to a
location, not a plain pair of closures. A copy aliases the same location.
`.constant(1)` ignores a write of 99. A child that increments an `@Binding`
twice moves the parent's `@State` to 2 and the parent's `body` runs once, because
the parent passed `$n` without reading `n`. `Binding($opt)` is failable.
`$arr[0].name` on `@State` and `$model.items[1].name` on `@Bindable` write
through.

### S9 Identity

A change of `.id(gen)` resets the child's `@State` (2 back to 0) and fires
`onDisappear` then `onAppear`. A plain parent re-evaluation re-runs the
child's `init` but not its `body`, and keeps its `@State`. After reordering
`[a, b, c]` to `[c, b, a]`, `ForEach(id: \.self)` keeps the mark on `a`;
`ForEach(indices, id: \.self)` leaves it at position 0, now showing `c`. A
child that captures a fresh closure re-runs on every parent pass; an
`Equatable` child or a non-capturing closure is skipped. `.equatable()`
changed nothing measurable.

### K1 Target-action

```text
b2 target=nil, performClick           -> Responder(root).probeAction
sendAction(probeAction, to: nil)      = true     sendAction(unknown, to: nil) = false
sendAction(appDelAction, to: nil)     = true     (reaches NSApp.delegate)
NSApp.target(forAction: probeAction)  = root
```

An item of a menu with `autoenablesItems = true` is enabled when something
answers its action and that responder's `validateMenuItem` agrees; only the
first responder found is asked. An explicit target is never checked: sending
to an object without the selector raises `NSInvalidArgumentException`. Without
a key window only `NSApp` and its delegate answer. (K1, K2 and K10 emulate a
key window by overriding `NSApplication.keyWindow`, because the probes must not
steal focus.)

### K2 Responder chain and key loop

```text
nextResponder: View C -> View B -> View A -> NSViewController -> NSWindow -> NSWindowController
delivery of a nil-target action: C > B > A > viewController > window > windowController > windowDelegate > NSApp > appDelegate
key loop after recalculateKeyViewLoop: A -> B -> C -> D -> contentView -> A   (geometric)
```

`makeFirstResponder` called `resignFirstResponder` on the old view, then
`becomeFirstResponder` on the new; it did not consult `acceptsFirstResponder`
(non-key window; not verified with a key window).

### K3 KVO and bindings

`observe(\.x, options: [.initial])` fires inside the call. A set fires the
property's `didSet` and then the observer, even for an equal value, on the
thread that set it. `bind(.value, …)` pushes the model to the control at once;
the control pushes back only when editing ends (a field) or on an action (a
slider). A non-`dynamic` `@objc` property is silent on a Swift set.

### K4 Auto Layout

```text
required 100 vs 750:200 vs 250:300           -> width 100 (lower priorities dropped, no log)
container 300, hug L1=251 L2=250             -> L2 stretches (lower hugging stretches)
NSStackView 50/80/30, spacing 8, width 300:
  fill [0 174][182 80][270 30]   fillEqually 95/94/95   equalSpacing x = 0, 120, 270
```

A conflict logs at **activation**, to the unified log (stderr only with
`OS_ACTIVITY_DT_MODE=YES`): `Unable to simultaneously satisfy constraints`;
the last added is broken yet stays active. `translatesAutoresizingMaskIntoConstraints`
is true by default and conflicts with an explicit width. Torin has no analogue
of any of this.

### K5 Drawing

`needsDisplay` draws nothing until the run-loop turn; rectangles are unioned
(`(0,0,5,5)` and `(60,50,5,5)` give one draw of `(0,0,65,55)`); `dirtyRect`
can exceed the bounds; `displayIfNeeded()` draws once and then not; hidden
and orphan views are not drawn; `isFlipped` moves the origin; with
`wantsUpdateLayer = true` only `updateLayer` runs. A kit `Canvas` is the
opposite: it repaints when its `revision` changes.

### K6 Notifications

`post` returns after the observers ran, in registration order, on the posting
thread. `queue: .main` from a background thread **blocks the poster** until
main pumps. `removeObserver` stops delivery; a released token does not.
`NSWorkspace.shared.notificationCenter` is a separate centre.

### K7 Hosting

`NSHostingView(rootView:)` evaluates nothing until the first size query or
attach; then `body`, `makeCoordinator`, `makeNSView`, `updateNSView`,
`sizeThatFits`. A state change gives `body`, `updateNSView`; an equal value
gives `body` only. A representable removed from the tree gets `updateNSView`,
`dismantleNSView`, then `Coordinator.deinit`. `NSHostingController` never
calls its `loadView` and `viewDidLoad` overrides.

### K8 Lifecycle

```text
VC.loadView, VC.viewDidLoad                 (at the contentViewController assignment)
applicationWillFinishLaunching, applicationDidFinishLaunching
orderFront: windowDidMove, VC.viewWillAppear, VC.viewDidAppear
performClose: windowShouldClose, windowWillClose, VC.viewWillDisappear, VC.viewDidDisappear
next turn: applicationShouldTerminateAfterLastWindowClosed, applicationShouldTerminate, applicationWillTerminate
```

`windowDidBecomeKey` and `applicationDidBecomeActive` did not fire: an
accessory app that does not activate cannot observe them.

### K9 Table and outline data

A view-based `NSTableView` with 10 000 rows in 300 pt asked for 15 views at
load (`heightOfRow` 301 times, not 10 000), 18 views after a scroll to row
5500 (15 reused, 3 created) and made all 15 cells fresh after `reloadData`.
`insertRows(at:)` asks one row. The outline asks `numberOfChildren` and
`child(_:ofItem:)` on expand only and restores nested expansion by item
identity.

### K10 Undo and pasteboard

`undo:` is answered by the `NSWindow`; `copy:` and `paste:` by the
first-responder text view. `window.undoManager` is the same object for the
window and its views. With `groupsByEvent`, two registrations in one run-loop
turn undo together. A manual `beginUndoGrouping` plus `undo()` raises
`NSInternalInconsistencyException`.

## 5. Surprises for a SwiftUI developer

### 5.1 The question

Would an agent that knows SwiftUI write the kit's app without surprises, using
`agents/` and the glossary? No. The audit translated 20 snippets: 0 map one to
one; the closest are `Text`, `VStack` and `HStack`, `Button::on_press` and
`TextField::on_submit`. The three agent files teach the shape of an app
(`Application`, `use_state`, the controlled pattern, the glossary lookup), and
`agents/prompt.md` step 1 asks for the SwiftUI code first. No document says
what to do with `@Binding`, `@Environment`, `@Observable`, `@FocusState`,
`ForEach` or `if/else`.

### 5.2 Ranked surprises

| # | Surprise | Evidence |
|---|---|---|
| 1 | Modifier chains stop compiling on controls: `Button::new("x").padding(..).help("..")`. Only 10 types take them; `agents/prompt.md` says "any view" | 1.3; D4 below |
| 2 | Modifiers set, not wrap: the padding is inside the background, repeated `.padding` replaces, a fixed width includes the padding | S1; 1.3 |
| 3 | Different types in `if/else` or `match` branches do not compile; erase with `.into_element()` | 1.1 |
| 4 | No `Picker`, `Toggle`, `ForEach`, `.tag`: `PopUpButton`, `Switch`, `Checkbox`, `usize` selection, `Vec<String>` rows | 1.6, 1.7 |
| 5 | No `Binding`: `TextField` takes `State<String>`, everything else value plus callback with a private copy; a constant `.value(x)` does not snap back | 1.8 |
| 6 | Labels are `&'static str`: `format!` and `tr!("id", n = x)` do not fit; `i18n::intern` is only in chapter 17 | 1.6 |
| 7 | Hooks are order-based, `.help` is a hook, a wrong order panics at run time; `use_future`, `use_side_effect*`, contexts and `Callback` are undocumented in `agents/` | 1.8, 1.10 |
| 8 | Nothing inherits: `.disabled`, `.foreground_style`, `.font`; style scopes compose instead of replacing | 1.4, 1.9; S3, S5 |
| 9 | `Sheet`, `Alert` and `Popover` are not modifiers: drawn while a flag is set; `Alert` has two buttons plus a `.destructive()` flag; `Popover::show(area)` needs an `on_sized` area | 1.1; audit rows 3a-3d |
| 10 | No `.toolbar`, `.searchable`, `.navigationTitle` or scene list: a toolbar forces `run_window` and a hand-built title bar; one window per `Application` | 1.1 |
| 11 | `Text::style` and five other `::style` methods rebuild the view and drop earlier modifiers; only `Text` is documented | 1.3 |
| 12 | Menus are data with one string-id callback; kit `Menu` is SwiftUI's `CommandMenu` and kit `MenuButton` is SwiftUI's `Menu` | 1.1 |
| 13 | `Slider::stepped(label, steps)` takes a count and its value is an index; `.range` after it changes the count | 1.6 |
| 14 | `Callback` is never equal: a component with a handler never memoises; a `PartialEq` struct cannot hold a bare closure | 1.2, 1.10 |
| 15 | Actions reach every handler of the window; there is no close veto, quit API or open-URL event | section 2; K1, K8 |

Runners-up: `.foreground_style` does not cascade from a stack (`agents/styles.md`);
the shortcut modifier is Ctrl; `Application` centres the content; `Label` in
the prelude is Freya's text element; `.offset` shifts children, it does not
move the view.

### 5.3 Names

| Measure | Result |
|---|---|
| Glossary table rows parsed | 950 (634 with SwiftUI `—`, 315 naming a SwiftUI token) |
| Kit name differs from the SwiftUI name | 221 of 315; 107 carry `≈`, 114 claim an exact equivalent |
| Pattern of the 221 | 33 `*Style` structs, 40 `.style` methods, 13 `.size` methods, 6 style presets, 129 other |
| AppKit-only rows with an `NS` prefix in the kit | 0 of 78 (rule 2 holds); `ButtonKind` (`NSWindow.ButtonType`) and `MinimizeTarget` (`NSDockTile`) differ after the prefix |
| Public items missing from the glossary | 0 of 414 |
| Seeded sample (every 38th row) | 25 of 25 exist |

Deliberate and documented (about 80 rows): `Application` for `App` (rule 6);
`Component`, `IntoElement` and `use_state` in the glossary's Freya table; the
`*Style` structs and `.style(..)` instead of `.buttonStyle` and friends
(ADR 0064: no naming rule covers it); `.size` for `.controlSize`; views for
modifiers (ADR 0061 to 0063); `.foreground_style` and `.background_style`
(Freya owns the plain names).

Undocumented (about 41 names; SwiftUI has the concept, rule 1 says use its
name, no reason is given): `PopUpButton`, `SegmentedControl`, `RadioGroup`
(`Picker`); `Switch`, `Checkbox` (`Toggle`); `MenuButton` (`Menu`);
`main_menu::Menu` (`CommandMenu`); `MainMenu` (`Commands`); `ToolGroup`;
`Titlebar` (`.toolbar`); `SplitView` (`NavigationSplitView`); `Row`
(`LabeledContent`); `Shortcut`; `StatusItem` (`MenuBarExtra`);
`SettingsWindow` (`Settings`); `Gradient` (SwiftUI `Gradient` is the stop
list); `Thickness`, `MaterialStyle` and `media::Material` (all three map to
SwiftUI `Material`); and others.

Collisions the glossary never warns about: kit `Menu` against `MenuButton`;
kit `Scene` is one window; `Label`, `Rect`, `Paragraph`, `App`, `Component`
and `Key` in the prelude are Freya's; no `Toggle`, `Picker`, `Spacer`,
`ZStack`, `Group`, `ForEach`, `Binding`. Callbacks not named `on_<verb>`
(rule 5): `Alert::confirm`, `cancel`, `add_button`; the positional callbacks
of `Row::navigation`, `destructive`, `toggle`; `SheetButtons::new`. Twenty
`with_*` builders do not follow "named after what they set".

### 5.4 Stale or wrong statements

| # | Statement | Where | Evidence | Verdict |
|---|---|---|---|---|
| D1 | "The 121 `Symbol`s" | `agents/prompt.md:79`, glossary row `symbol::Symbol` | `symbol.rs:19-144` has 124 variants; the glossary's variant table has 124 rows | stale |
| D2 | the font list omits `TITLE_3`, `SUBHEADLINE`, `FOOTNOTE`, `CAPTION_2` | `agents/prompt.md:56-57`, [chapter 2](../barrierkit/02-first-app.md) | `prelude.rs:1-5` exports them | incomplete |
| D3 | "An app depends only on `bk`" | `agents/prompt.md:37-39` | [chapter 14](../barrierkit/14-animations.md) adds `freya-animation`; chapter 13 adds `freya-clipboard`; chapter 3 and `titlebar_over_gradient.rs` import `ctdesign` | wrong |
| D4 | `.help` and `.context_menu` work "on any view" | `agents/prompt.md:66-70` | [chapter 7](../barrierkit/07-menus-and-popovers.md): wrap controls in a `rect()` | contradiction |
| D5 | `Sheet` is a rect and takes the modifiers | [chapter 11](../barrierkit/11-views.md), chapter 7 | `views/feedback.rs:69-73`: a `PartialEq` struct | wrong |
| D6 | "Controls are controlled: pass the value" | `agents/prompt.md:61-62`, checklist 4 | 11 types take `State<T>`; the exception is only in [chapter 5](../barrierkit/05-controls.md#the-controlled-pattern) | incomplete |
| D7 | "No literal sizes" has no throwaway exemption | `agents/prompt.md:54-55` | `hello_world.rs` line 6 `.padding(40.)`, `counter.rs` line 14 `.size((420, 300))` | self-contradiction |
| D8 | the `Text::style` order rule | chapter 2 only | the same rebuild in `Badge`, `Divider`, `Avatar`, `SymbolTile` | undocumented for four types |
| D9 | "Call hooks on every render" | `agents/prompt.md:68-70`, checklist 6 | right, but `ForEach { row.help(..) }` breaks it; the checklist names two hooks of about ten | trap-prone |
| D10 | step 1: "write down the SwiftUI code" | `agents/prompt.md:19` | no document maps `@Binding`, `@Environment`, `@Observable`, `@FocusState`, `ForEach`, `if/else` | gap |
| D11 | glossary step 1: search for `Toggle` | glossary, Steps | the only row with `Toggle` is `Row::toggle` | misleading |
| D12 | "Colors only from `Palette::current()`" | `agents/prompt.md:54` | `styles.md` and `styled_control.rs` use `ShapeStyle::Role` and `Accent` | incomplete |
| D13 | the prelude table | chapter 2 | `prelude.rs:17` also exports `IconStyle`, `TemplateRenderingMode` | incomplete |
| D14 | recipes are files, never a block in `recipes.md` | `AGENTS.md` | `recipes.md:74-76` has a block; 12 sections, 11 `[[example]]` | minor |

Checked and true: `.cargo/config.toml` forces `MALAYSIA_CONFIG_DIR`;
`./dev.sh check` runs `scripts/check-recipes.sh`; `use_app_language` takes the
`Application` name; `Application::window` centres the content.

### 5.5 What `agents/` never teaches

A `grep -c` of each name over `agents/*.md` and `docs/barrierkit/*.md` finds
0 for: `@Binding`, `@Environment`, `@Observable`, `@Bindable`, `@FocusState`,
`ForEach`, `ZStack`, `Spacer`, `EmptyView`, `AnyView`, `GeometryReader`,
`withAnimation`, `use_memo`, `use_drop`, `use_future`, `use_side_effect`,
`use_consume`, `Callback`, `onAppear`, `.task`, `onChange`, `NSResponder`,
`NSLayoutConstraint`, `NotificationCenter`. The only SwiftUI names the
agent files give are `@State` (`recipes.md:28`), `TimelineView` and
`controlSize`.

## 6. Rows to add to the coverage matrix

I did not edit [`swiftui-appkit-coverage.md`](swiftui-appkit-coverage.md).
These rows are proposals; the priorities are mine. The matrix is keyed by look
and behaviour and has none of these syntax-level rows (`grep -n
"ZStack\|Spacer\|ForEach\|GeometryReader\|withAnimation"` finds nothing).

| SwiftUI | AppKit | Kit | Status | Pri |
|---|---|---|---|---|
| `Binding`, `$state` | `bind(_:to:)` | `State<T>` on 11 controls; value plus callback elsewhere | partial | P1 |
| `@Observable`, `ObservableObject` | KVO | a struct of `State`s, by hand | partial | P2 |
| `@FocusState`, `.focused` | `makeFirstResponder` | `request_focus` exists; `TextField` has no id | partial | P1 |
| `.disabled`, `\.isEnabled` through a tree | `NSControl.isEnabled` | per control | partial | P2 |
| `.foregroundStyle`, `.font` inheritance | `NSAppearance` | per element | missing | P2 |
| `withAnimation`, `.transition`, `matchedGeometryEffect` | `NSAnimationContext` | none | missing | P3 |
| `.sheet(isPresented:)`, `\.dismiss` | `beginSheet` | by existence | partial | P3 |
| `.confirmationDialog` | `NSAlert` | `Alert` with three buttons | missing | P2 |
| `.task`, `.onAppear`, `.onChange`, `.onDisappear` | `viewDidAppear` | Freya hooks, undocumented | partial | P1 |
| `ForEach`, `Group`, `EmptyView`, `AnyView` | none | `.children`, `Element` | partial | P3 |
| `Spacer`, `ZStack`, `.overlay`, `Grid`, `ViewThatFits`, `GeometryReader`, `LazyVStack` | `NSGridView` | by hand, or none | missing | P2 |
| `.padding`, `.background`, `.help` on every control | none | 10 types only | partial | P1 |
| `ButtonStyle.makeBody` | `NSButtonCell` | field structs | partial | P3 |
| `Picker` with `.tag` | none | index | partial | P2 |
| dynamic text labels | none | `&'static str` plus `intern` | partial | P2 |
| `.keyboardShortcut` outside a menu | `performKeyEquivalent` | menu only | partial | P2 |
| `@AppStorage` per key | `UserDefaults` | one struct | partial | P3 |
| `.onKeyPress`, gestures | `keyDown`, `NSGestureRecognizer` | raw events | partial | P3 |
| responder chain, `validateMenuItem` | `NSResponder` | broadcast per window | missing | P1 |
| `windowShouldClose`, `applicationShouldTerminate`, `application(_:open:)` | window and app delegates | none | missing | P1 |
| lazy data sources | `NSTableViewDataSource` | eager strings | partial | P2 |
| Auto Layout constraints | `NSLayoutConstraint` | none (by design) | missing | P3 |
| KVO, KVC | `NSKeyValueObserving` | none | missing | P3 |
| `NotificationCenter` | `NSNotificationCenter` | none | missing | P2 |
| `NSViewRepresentable`, `NSHostingView` | same | none | missing | P3 |
| `Canvas` events | `NSView` mouse events | a `Canvas` has none | partial | P2 |

## 7. Not verified and decisions

### 7.1 Not verified

| Item | Why |
|---|---|
| The padding-inside-background rule | read from Torin and Freya; not rendered in the kit |
| `Text` and `Image` resolve their style at construction, so a same-render `.control_style(\|s: TextStyle\| ..)` may miss them | read from `elements.rs:119-126`; not run |
| `State` is not `Send` | derived from the types; no compile error was produced |
| `use_published`, `use_wake` and `use_timeline` may panic after unmount | callbacks are never removed (`platform/timer.rs:74-79`); not run |
| `use_appearance` reaches popups and sheets | the theme is a snapshot at open time; not run |
| `Canvas` needs another Skia dependency | no `pub use freya_engine` found; not compiled |
| `.key(&str)` literal | probably fails; not compiled |
| `prevent_default` effect; Ctrl+Shift+Z redo in a one-line field | `freya-edit` matches a lowercase `y` only |
| `makeFirstResponder` and `acceptsFirstResponder` | measured with a non-key window only |
| K1, K2, K10 | a key window was emulated by overriding `NSApplication.keyWindow` |
| `windowDidBecomeKey`, `applicationDidBecomeActive` | cannot fire without activating the probe |
| S2 on older targets | only `macos15.0` was compared with the default |
| Swift 6 mode | every probe but S7 ran in Swift 5 mode |
| [coverage matrix](swiftui-appkit-coverage.md) row on drops ("drop only on image well") against [chapter 13](../barrierkit/13-app.md) ("any element") | the two disagree; I did not run either |

Side effects of the probes: K3 wrote its own defaults domain
(`~/Library/Preferences/k3.plist`); the agent deleted it and restarted
`cfprefsd`, which restarts on its own. Two scratch files were written to
`/tmp` and removed.

### 7.2 Decisions for the user

1. **Modifiers on controls. Decided 2026-11-21: every public control takes the
   Freya container modifiers.** `.padding`, `.background` and the other
   `ContainerExt`, `BackgroundExt` and `DecorationExt` methods set properties
   on the control's own node, as they do on the 10 forwarding types; order
   does not matter. The options were:
   - **Wrap every control** (chosen). Of the 98 types that implement
     `Component` in `crates/bk/src`, 77 are `pub struct`s (a script over
     `impl Component for` and `pub struct`, minus the private `Divider` of
     `views/split_panes.rs:137`). `forward_container!` forwards a
     `Rect` held in `.0` (`elements.rs:66-94`), but a control derives
     `PartialEq` (`controls/button.rs:5`) and `Rect` does not (it derives
     `Default, Clone`: `freya-core` `elements/rect.rs:562`). `LayoutData`,
     `StyleState` and `EffectData` do derive `PartialEq` (`freya-core`
     `data.rs:60,85,100`), and they are what the macro delegates to, so a
     control can hold them instead. How the control applies them in `render`,
     and what that does to `Size::fill()`, is not verified.
     **Done 2026-11-21.** 73 of the 77 hold an `Overrides` (`LayoutData`,
     `StyleState`, `EffectData`, `EventHandlers`: `elements.rs`) and merge it
     into their own root node; the five left out are `StyleScope`, `Sheet`,
     `Alert`, `SidebarToggle` and `Titlebar`. There is no extra node, except
     one `rect()` around the control that carries event handlers (an `Event`
     cannot be cloned, so two handlers for one event cannot chain). `.padding`,
     `.margin`, `.height` and `Size::fill()` keep their layout and the order of
     the calls does not matter. Finding 2
     and surprise 1 above describe the state before. `.key(..)` is still a no
     op on most controls: only `ScrollView`, `TextView` and `RuleEditor` read
     it in `render_key`.
   - **State the wrap rule** in `agents/` (`rect().padding(..).child(..)`):
     no code change; the surprise stays.
   - **One generic wrapper** on `IntoElement`, as `StyleModifiers` already is
     (`modifiers.rs:61-71`). It wraps like SwiftUI, but `VStack.padding(..)`
     then fails with E0034, because the 10 forwarding types already get
     `padding` from Freya's `ContainerExt` (a scratch program of the same
     shape). Sizing under a fit-size wrapper and `.key` pass-through are not
     verified.
2. **A `Binding` type. Decided 2026-11-21: none.** Text controls keep
   `State<String>` and every other control keeps value plus callback; nothing
   is added until an app asks. Freya's `Writable` compares equal for any two
   values (`freya-core` `lifecycle/writable.rs:60`), and neither malaysia nor
   curseterm (`1375c72`) uses `Writable` or `use_radio`. The reasons and the
   greps are in [ADR 0085](../decisions.md).
3. **Actions and lifecycle. Decided 2026-11-21: non-goals until an app asks.**
   No routing of an `Action` to the focused view, no close veto, no quit call
   and no open-URL event. A grep of malaysia and curseterm finds no use of
   them, and the three repositories have no issue asking for one
   ([ADR 0085](../decisions.md)).

The 14 fixes of [5.4](#54-stale-or-wrong-statements) and the chapter
[SwiftUI to the kit](../barrierkit/16-swiftui-to-the-kit.md) were written on
2026-11-21. Five statements (D1 to D4 and D13) had already been fixed by other
sessions; D5 to D12 and D14 were fixed then. The coverage matrix got the two
rows of decision 3 in section 14 and a note on `Binding`.
