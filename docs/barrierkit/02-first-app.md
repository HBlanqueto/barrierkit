# 2. Your first app

**Goal:** open a window with text, then add state that changes it.
**Scope:** from an empty crate to a window that counts. Controls are in
[5](05-controls.md), window options in [8](08-windows.md), full programs for
other tasks in the [recipes](../../agents/recipes.md).
**Reader:** anyone who has never used BarrierKit or Freya. SwiftUI helps but is
not needed.
**Input:** the repository; on Linux, `nix develop`. **Output:** two programs that run.

## Steps

1. On Linux, enter the dev shell: `nix develop`. `./dev.sh check` refuses to
   run outside it. On macOS, `rustup` is enough
   ([Build and check on macOS](18-platform.md#build-and-check-on-macos)).
2. Create a binary crate (`cargo new hello`) and add one dependency:

   ```toml
   [dependencies]
   bk = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   ```

3. Replace `src/main.rs` with the program in [Hello, world](#hello-world).
4. Run `cargo run`. A window opens with a title bar, traffic lights and the
   text in the center.
5. Replace `src/main.rs` with the program in [Adding state](#adding-state) and
   run it again. Press «Add one»: the number goes up.

## Hello, world

```rust
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.hello")
        .window("Hello", || {
            Text::new("Hello, world").font(TITLE).padding(40.)
        })
        .run()
}
```

1. `use bk::prelude::*;` brings every name the program uses.
2. `Application::new("com.example.hello")` creates the app. `"com.example.hello"` is its id; the menus show its last part, `hello`.
3. `.window("Hello", || …)` adds the window. The closure is the view.
4. `Text::new(…).font(TITLE).padding(40.)` is the view: a title-sized text
   with 40 px of space around it.
5. `.run()` opens the window and returns when it closes.

The SwiftUI and AppKit name of each piece is in the [glossary](20-glossary.md).

## Adding state

```rust
use bk::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("com.example.counter")
        .window("Counter", || {
            let mut count = use_state(|| 0);
            VStack::new()
                .child(Text::new(count.read().to_string()).font(TITLE))
                .child(
                    Button::new("Add one")
                        .style(ButtonStyle::bordered_prominent)
                        .on_press(move |_| *count.write() += 1),
                )
        })
        .options(|window| window.size((420, 300)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}
```

1. `use_state(|| 0)` creates a value that survives between calls of the view.
2. `count.read()` reads it; `count.write()` changes it.
3. When the value changes, the kit calls the view again and applies the
   difference.
4. `VStack` places the text above the button.
5. `.options(…)` sets the window size and its minimum size.

## The ideas

1. **The view is a function of the state.** Nothing is moved or deleted by
   hand.
2. **The app holds the state.** Controls are *controlled*: they take the
   `State` they edit (`Switch::new("Wi-Fi", wifi)`) and write the change to it;
   a callback is only for an action (`on_press`).
3. **Views are built by chaining methods:** `Type::new(..).option(..)`.
4. **No color or size is written by hand.** `Text` takes its color from the
   palette, and sizes and fonts are tokens ([4](04-appearance-and-tokens.md)).
5. **One import:** the [prelude](#the-prelude).

## What `Application` does for you

| Item | What it does |
|---|---|
| `Application::new(id)` | Creates the app. `id` is reverse DNS and stable, like a bundle identifier (`com.example.notes`): it is each window's `app_id` and names the app's own folders for its settings and saved state ([13](13-app.md#settings-and-restoration)). Its last part is the name the menus show |
| `Application::theme(theme)` | Gives every window the [`Theme`](../../agents/styles.md#levels); without it, `Theme::tengen()` |
| `Application::window(title, content)` | Declares the window that opens at launch, a `WindowGroup` with the id `MAIN_WINDOW`. `content` is a closure that returns the view; hooks go at its top |
| `Application::scene(scene)` | Declares another [scene](08-windows.md#scenes): `WindowGroup`, `Window` or `Settings` |
| `Application::options(\|window\| …)` | Changes any [`WindowOptions`](08-windows.md) setting of the scene declared last: size, limits, menu, full-screen bar |
| `Application::commands(commands)` | Changes the [standard menus](13-app.md#standard-menus) of every window; without it, `Commands::new()` |
| `Application::run()` | Draws the window background (`Palette::current().window_background`), the title bar (a kit [`Titlebar`](09-titlebars.md) with the title, over AppKit's own bar on macOS), centers `content` and calls [`run_window`](08-windows.md). Returns `Result<(), RunError>`; `RunError::NoWindow` when only `Settings` is declared |

A window that draws its own title bar (a sidebar, toolbar items) sets
`.options(|window| window.window_style(WindowStyle::HiddenTitleBar))` and gets
no kit `Titlebar` ([ADR 0115](../decisions.md#0115--standard-menus-by-default-edit-items-reach-the-focused-text-field-macos-menus-are-appkits),
[recipe 7](../../agents/recipes.md#7-drawing-the-whole-window-yourself)). An app
with several windows declares scenes
([recipe 25](../../agents/recipes.md#25-several-windows)).

## Text and stacks

| Item | Defaults | Own methods |
|---|---|---|
| `Text::new(content)` | Font `BODY`, color `Palette::current().label`, wraps to as many lines as needed | `.style(\|s\| ..)`, `.font(Font)`, `.line_limit(n)`, `.bold(bool)`, `.italic(bool)`, `.underline(bool)`, `.strikethrough(bool)`, `.text_selection(bool)`; `Text::markdown(..)` reads inline Markdown and `a + b` joins runs ([6](06-text-and-editing.md#text-runs)) |
| `Label::new(title, Symbol)` | The symbol in a 16 wide, 16.5 high box, then the title 6 apart (SwiftUI's 48 wide `Label("Doc", ...)`); in a button label the compact `SPACE_1` layout; in a `List` row the symbol takes the accent | — |
| `VStack::new()` | Children in a column, centered, apart by what SwiftUI puts between a text, a symbol, a label, a checkbox and any other control (`STACK_COLUMN_GAPS`: 0 between two texts, 8 under a text, 5 over one, 8 between controls, more beside a larger text by `STACK_TEXT_ABOVE` and `STACK_TEXT_BELOW`, as SwiftUI's; none under a `Titlebar`, which SwiftUI keeps out of the stack); `.spacing(x)` makes them all `x` | — |
| `HStack::new()` | Children in a row, centered, `SPACE_2` apart | — |

`content` and `title` accept a `&'static str`, a `String` or the result of `tr!`.

`Text::style` rebuilds the text: call it before `.font`, `.line_limit` and any
Freya method. `Badge::style`, `Divider::style`, `Avatar::style` and
`SymbolTile::style` rebuild their views the same way.

```rust
// ✗ the style drops the font
Text::new("Title").font(TITLE).style(|s| s)
// ✓
Text::new("Title").style(|s| s).font(TITLE)
```

The four types wrap Freya's `label()` and `rect()` and forward all their
traits, so Freya's methods work on them: layout (`padding`, `width`,
`expanded`), text style (`color`, `font_size`), events (`on_press`) and, on the
stacks, children and alignment (`child`, `children`, `spacing`,
`main_align`, `cross_align`, `background`, `corner_radius`).

The defaults of `Text` come from a `TextStyle`, so a theme can change the font
and color of every text; its fields are in
[`styles/text.rs`](../../crates/bk/src/styles/text.rs).
The stacks have no style of their own: their methods and the
[paint on views](../../agents/styles.md#paint-on-views) style them.

## The prelude

`use bk::prelude::*;` brings:

| From | Names |
|---|---|
| The kit: app and windows | `Application`, `Scene`, `WindowGroup`, `Window`, `Settings`, `UtilityWindow`, `Commands`, `CommandGroup`, `CommandGroupPlacement`, `OpenWindowAction`, `DismissWindowAction`, `use_open_window`, `use_dismiss_window`, `use_app_storage`, `Document`, `OpenedUrl`, `use_document`, `FileDialog`, `ContentType`, `run_window`, `WindowOptions`, `WindowStyle`, `WindowToolbarFullScreenVisibility`, `RunError`, `Titlebar`, `WindowToolbarStyle`, `ToolbarItem`, `ToolbarItemPlacement`, `ToolbarSpacer` |
| The kit: views | `Text`, `Label`, `VStack`, `HStack`, `Divider`, `Image`, `Symbol`, `ImageScale`, `SymbolRenderingMode`, `SymbolWeight`, `IconStyle`, `TemplateRenderingMode`, `Rectangle`, `RoundedRectangle`, `Capsule`, `Circle`, `Ellipse`, `Path`, and the [`controls`](05-controls.md) module: `Button`, `ButtonRole`, `ControlSize`, `Checkbox`, `Switch`, `Slider`, `CircularSlider`, `SegmentedControl`, `TextField`, `SecureField`, `SearchField`, `ToolGroup`, `ScrollView` and its `ScrollPosition`, `ScrollController`, `use_scroll_controller`… |
| The kit: app views | `SplitView`, `Sidebar`, `SidebarItem`, `List`, `Form`, `Section`, `Row`, `TabView`, `TextEditor`, `ProgressView`, `PopUpButton`, `IconButton`, `ToolbarSearch`, `with_animation`, `use_animated`, `use_timeline` |
| The kit: layout | `Spacer`, `Frame`, `ZStack`, `ZStackLayout`, `Grid`, `GridRow`, `LazyVGrid`, `GridItem`, `GridCell`, `Layout`, `LayoutContainer`, `Placement`, `ViewThatFits`, `ContentAlignment`, `HorizontalAlignment`, `VerticalAlignment` |
| The kit: modifiers | [`ViewModifiers`](07-menus-and-popovers.md#view-modifiers) (`.help`, `.context_menu`, `.sheet`, `.alert`, `.popover`, `.on_hover`…), [`StyleModifiers`](../../agents/styles.md#levels) (`.control_style`, `.control_size`, `.frame`, `.glass_effect`, `.content_transition`), [`EnvironmentModifiers`](16-swiftui-to-the-kit.md#environment) (`.disabled`, `.environment`, `.layout_direction`, `.keyboard_shortcut`, `.focused`, `.default_focus`…), `GridModifiers`, `OverlayModifiers`, `ShapeModifiers`, `AccessibilityModifiers` with `AccessibilityTraits` and `AccessibilityChildBehavior` |
| The kit: state and style | `use_environment`, `use_is_enabled`, `FocusState`, `use_focus_state`, `Transition`, `ContentTransition`, `Presence`, `Glass`, `Palette`, `text`, `color` and everything in [`styles`](../../agents/styles.md#value-types) (`Theme`, `ControlStyle`, `SliderStyle`, `ShapeStyle`, `Rgba`…) |
| Freya | its whole prelude: `rect()`, `label()`, `use_state`, `Component`, `App`, `IntoElement`, element methods; its `Label` type gives way to the kit's |
| Torin | `Alignment`, `Direction`, `Size`, `Size2D` |
| Tokens | `BODY`, `CALLOUT`, `CAPTION`, `CAPTION_2`, `FOOTNOTE`, `HEADLINE`, `LARGE_TITLE`, `MONO`, `SUBHEADLINE`, `TITLE`, `TITLE_2`, `TITLE_3`, `Font`, `SPACE_1`…`SPACE_4`, `WINDOW_DEFAULT_SIZE`, `WINDOW_MIN_SIZE` |

Everything else is imported from its module, for example
`bk::views::table::Table` ([3](03-layers.md)).

Next: [3. Layers](03-layers.md).
