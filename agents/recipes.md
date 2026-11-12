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
literal above becomes `tr!("id")`:

```rust
Text::new(tr!("hello")).font(TITLE)
```

## 9. A stopwatch

SwiftUI: a `Text` fed by `TimelineView(.periodic(from:by:))` with Start/Stop and Lap/Reset buttons.

The program is [`recipes/stopwatch.rs`](recipes/stopwatch.rs).

`Timers::after` runs a callback once after a delay ([8](../docs/barrierkit/08-windows.md)).
`use_side_effect` runs again when `now` or `started` change, so each tick
schedules the next one while the stopwatch runs.

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
