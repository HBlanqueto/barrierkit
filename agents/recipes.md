# Recipes

**Goal:** start a program from a whole `src/main.rs` that already compiles.
**Scope:** one program per common task. Each item's reference lives in its
chapter; the names are in the [glossary](../docs/barrierkit/20-glossary.md).
**Reader:** an AI agent sent here by [`prompt.md`](prompt.md), or a person.
**Input:** a task. **Output:** the closest program, ready to change.

## Steps

1. Find the recipe closest to the task in the list below.
2. Copy its file from [`recipes/`](recipes/) into `src/main.rs`. Each file is
   an example of `bk`, so `./dev.sh check` builds and lints it; inside the
   kit, `cargo run -p bk --example <name>` opens it. There `.cargo/config.toml`
   forces `MALAYSIA_CONFIG_DIR` to `crates/bk/tests/config`, so it opens in
   Spanish, without your appearance, and saves its settings in that folder.
3. Change only what the task needs.
4. Build it inside `nix develop` and go through the [checklist](checklist.md).

## 1. Text in a window

SwiftUI: `WindowGroup { Text("Hello, world").font(.title).padding(40) }`

The program is [`recipes/hello_world.rs`](recipes/hello_world.rs); [chapter 2](../docs/barrierkit/02-first-app.md#hello-world) explains it line by line.

## 2. State and a button

SwiftUI: `@State var count = 0` with `VStack { Text("\(count)"); Button("Add one") { count += 1 } }`

The program is [`recipes/counter.rs`](recipes/counter.rs); [chapter 2](../docs/barrierkit/02-first-app.md#adding-state) explains it line by line.

## 3. Rows and columns

SwiftUI: `VStack(alignment: .leading, spacing: 4) { HStack { … } }`

The program is [`recipes/rows_and_columns.rs`](recipes/rows_and_columns.rs).

## 4. A toggle bound to state

SwiftUI: `Toggle("Wi-Fi", isOn: $wifi)`

The program is [`recipes/toggle.rs`](recipes/toggle.rs).

## 5. A settings form

SwiftUI: `Form { Section("General") { Toggle("Sounds", isOn: $sounds) } }`

The program is [`recipes/settings_form.rs`](recipes/settings_form.rs).

For a real preferences window use `SettingsWindow` (chapter
[12](../docs/barrierkit/12-patterns.md)).

## 6. A reusable view

SwiftUI: `struct Greeting: View { let name: String; var body: some View { … } }`

The program is [`recipes/reusable_view.rs`](recipes/reusable_view.rs).

A `Component` must be `PartialEq`: the kit redraws it only when its fields
change.

## 7. Drawing the whole window yourself

Use it when the titlebar needs a sidebar or toolbar items. `run_window` takes
the same closure but adds nothing: you draw the background and the titlebar.

The program is [`recipes/custom_window.rs`](recipes/custom_window.rs).

## 8. Translated text

Follow the steps in chapter [17](../docs/barrierkit/17-languages.md); then every
literal above becomes `tr!("id")`, as in `Text::new(tr!("hello"))`.

## 9. A stopwatch

SwiftUI: a `Text` fed by `TimelineView(.periodic(from:by:))` with Start/Stop and Lap/Reset buttons.

The program is [`recipes/stopwatch.rs`](recipes/stopwatch.rs).

`use_timeline(active, every)` renders the window again every `every` while
`active` is true ([8](../docs/barrierkit/08-windows.md)), so `Instant::now()`
in the render is the time of each tick.

## 10. Buttons in the title bar over a gradient

SwiftUI: `.toolbar { ToolbarItemGroup { … } }` over a `LinearGradient` background.

The program is [`recipes/titlebar_over_gradient.rs`](recipes/titlebar_over_gradient.rs).

1. The gradient is a [`Backdrop`](../docs/barrierkit/15-backdrops-and-media.md#backdrop),
   not a hand-written `LinearGradient`: it gives the content a legible palette
   and makes the `Titlebar` transparent. Its colors come from a
   `BACKDROP_SCENES` preset or from an image (`BackdropSource::Image`).
2. The content is a `Component`, so it reads the backdrop's palette.
3. `run_window` is used because the window draws its own title bar with
   buttons ([8](../docs/barrierkit/08-windows.md)). Every kit window is
   client-side decorated.

## 11. Style one control

SwiftUI: a custom `ButtonStyle` given with `.buttonStyle(_:)` to one view, a stack or the whole app.

The program is [`recipes/styled_control.rs`](recipes/styled_control.rs).

1. The theme makes every slider square and every button glass
   (`ButtonStyle::glass` is a preset: a function of the style, like SwiftUI's
   `.glass`); the stack turns its two sliders
   vertical; `.style` paints one fill red, grey while pressed, and gives the
   `LevelSlider` the `fat` preset, the Control Center bar. Each override names only what
   changes and ends with `..s`. `Theme::style` and `.control_style` take any
   control's style: the parameter's type (`|s: SliderStyle|`) picks it.
2. `Role` colors follow light, dark and contrast; the kit still applies
   reduce motion, reduce transparency and the focus ring.
3. The levels, fields and value types are in [`styles.md`](styles.md).

## 12. Controls at every size

SwiftUI: `.controlSize(.mini)`, `.small`, `.regular`, `.large` or `.extraLarge` on a button, a toggle or a slider.

The program is [`recipes/control_sizes.rs`](recipes/control_sizes.rs).

`.control_size(ControlSize)` on a view sizes every control inside it, like SwiftUI's
`.controlSize(_:)`; `.size(ControlSize)` on one control wins over it. Both take one of five
sizes on `Button`, `PopUpButton`, `SegmentedControl`, `Checkbox`, `RadioGroup`, `Switch`,
`Slider`, `Stepper`, `TextField`, `SearchField`, `ComboBox` and `DatePicker`; the sizes are in
[chapter 5](../docs/barrierkit/05-controls.md#control-size).

## 13. Modifiers on controls

SwiftUI: `Button("Save") { }.padding(8).help("Save the file")` inside an `HStack`.

The program is [`recipes/control_modifiers.rs`](recipes/control_modifiers.rs).

1. Every control takes `.padding`, `.margin`, `.width`, `.background`,
   `.corner_radius`, `.opacity`, `.help` and `.context_menu`, in any order
   ([chapter 5](../docs/barrierkit/05-controls.md#modifiers-on-controls)).
2. A modifier sets a property on that one node; it does not wrap it. Outer
   space is `.margin`: a second `.padding` replaces the first.
3. `.help` uses a hook: call it on every render, never inside a condition or
   a loop.

## 14. A button with a view label

SwiftUI: `Button(action: add) { Label("Add", systemImage: "plus") }`.

The program is [`recipes/button_label.rs`](recipes/button_label.rs).

1. `.label(|| view)` takes a closure that returns the view; it replaces the
   drawn text. The string given to `Button::new` stays the accessible name
   ([chapter 5](../docs/barrierkit/05-controls.md#buttons)).
2. Inside the closure, `Text`, `Label` and `label()` take the button's font and
   the ink of its current state, so the symbol and the title turn white on the
   accent fill of `bordered_prominent`.
3. Use `Label` for a symbol with a title; a bare `Image::system` keeps its own
   color.

## 15. State passed to a child view

SwiftUI: `struct NameField: View { @Binding var name: String … }` used as `NameField(name: $name)`.

The program is [`recipes/binding.rs`](recipes/binding.rs).

1. A text field takes the `State<String>` it edits. The child holds that
   `State` as a field, and the parent reads the same value.
2. Any other control takes the value and a callback, as in
   [recipe 4](#4-a-toggle-bound-to-state). The kit has no `Binding`
   ([ADR 0085](../docs/decisions.md)).

## 16. A value from the environment

SwiftUI: `.environment(\.account, account)` above and `@Environment(\.account) var account` below.

The program is [`recipes/environment.rs`](recipes/environment.rs).

1. `use_provide_context` creates the value once; `use_consume::<T>()` finds it
   by type in the nearest ancestor and panics when none provides it.
2. The value is read once. A value that changes holds a `State`
   ([recipe 17](#17-observable-state)).

## 17. Observable state

SwiftUI: `@Observable class Counter { var count = 0; var step = 1 }` shared through the environment.

The program is [`recipes/observable.rs`](recipes/observable.rs).

1. The model is a `Copy` struct of `State`s. Build it in the render, then give
   it to `use_provide_context`.
2. A view re-renders for the `State`s it reads: `CountLabel` reads `count`, and
   a change to `step` does not touch it.

## 18. Moving the focus

SwiftUI: `@FocusState var searching: Bool`, `.focused($searching)` and `searching = true`.

The program is [`recipes/focus_state.rs`](recipes/focus_state.rs).

1. `use_a11y()` returns the id; `SearchField::a11y_id(id)` gives it to the
   field; `id.request_focus()` moves the focus.
2. Only `SearchField` takes an id. `TextField` and `SecureField` have
   `.auto_focus(bool)`.

## 19. A list from data

SwiftUI: `ForEach(fruits, id: \.id) { Text($0.name) }`

The program is [`recipes/for_each.rs`](recipes/for_each.rs).

1. `.children(iter)` takes any iterator; `.key(id)` gives each child its
   identity, so its state follows the item when the list changes.
2. A duplicate key among siblings panics in debug builds.

## 20. Views that depend on a condition

SwiftUI: `if signedIn { Text("Welcome back") } else { Button("Sign in") { … } }`

The program is [`recipes/if_else.rs`](recipes/if_else.rs).

1. Two arms of different types need `.into_element()`.
2. `.maybe_child(opt)` adds a child only when the option is `Some`.

## 21. A command with a shortcut

SwiftUI: `.commands { CommandMenu("Counter") { Button("Add one") { … }.keyboardShortcut("k") } }`

The program is [`recipes/actions.rs`](recipes/actions.rs).

1. The `Action` is a constant. Its item goes in the `MainMenu`, and the
   shortcut works only through an enabled menu item
   ([chapter 13](../docs/barrierkit/13-app.md#actions-and-shortcuts)).
2. Every handler of the window receives every id, so the handler compares the
   id with `ADD.id`.
