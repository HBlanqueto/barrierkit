# 2. Your first app

**Goal:** open a window with text, then add state that changes it.
**Scope:** from an empty crate to a window that counts. Controls are in
[5](05-controls.md), window options in [8](08-windows.md), full programs for
other tasks in the [recipes](../ai/recipes.md).
**Reader:** anyone who has never used BarrierKit or Freya. SwiftUI helps but is
not needed.
**Input:** the repository and `nix develop`. **Output:** two programs that run.

## Steps

1. Enter the dev shell: `nix develop`. Outside it, `build.rs` stops the build on
   purpose.
2. Create a binary crate (`cargo new hello`) and add one dependency:

   ```toml
   [dependencies]
   barrier-kit = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   ```

3. Replace `src/main.rs` with the program in [Hello, world](#hello-world).
4. Run `cargo run`. A window opens with a title bar, traffic lights and the
   text in the center.
5. Replace `src/main.rs` with the program in [Adding state](#adding-state) and
   run it again. Press «Add one»: the number goes up.

## Hello, world

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("hello")
        .window("Hello", || Text::new("Hello, world").font(TITLE).padding(40.))
        .run()
}
```

1. `use barrier_kit::prelude::*;` brings every name the program uses.
2. `Application::new("hello")` creates the app. `"hello"` is its short name.
3. `.window("Hello", || …)` adds the window. The closure is the view.
4. `Text::new(…).font(TITLE).padding(40.)` is the view: a title-sized text
   with 40 px of space around it.
5. `.run()` opens the window and returns when it closes.

The SwiftUI and AppKit name of each piece is in the [glossary](20-glossary.md).

## Adding state

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("counter")
        .window("Counter", || {
            let mut count = use_state(|| 0);
            VStack::new()
                .child(Text::new(count.read().to_string()).font(TITLE))
                .child(
                    Button::new("Add one")
                        .style(ButtonStyle::Primary)
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
2. **The app holds the state.** Controls are *controlled*: they receive the
   value (`value`, `on`, `selected`) and report the change (`on_change`,
   `on_toggle`, `on_press`).
3. **Views are built by chaining methods:** `Type::new(..).option(..)`.
4. **No color or size is written by hand.** `Text` takes its color from the
   palette, and sizes and fonts are tokens ([4](04-appearance-and-tokens.md)).
5. **One import:** the [prelude](#the-prelude).

## What `Application` does for you

| Item | What it does |
|---|---|
| `Application::new(name)` | Creates the app. `name` is short, lowercase and stable: it becomes the `app_id`, the settings domain and the key of the saved window size |
| `Application::window(title, content)` | Returns a `Scene` for one window. `content` is a closure that returns the view; hooks go at its top |
| `Scene::options(\|window\| …)` | Changes any [`WindowOptions`](08-windows.md) setting: size, limits, menu, full-screen bar |
| `Scene::run()` | Draws the window background (`Palette::current().window_background`), a [`Titlebar`](09-titlebars.md) with the title, centers `content` and calls [`run_window`](08-windows.md). Returns `Result<(), RunError>` |

Use `run_window` directly when the window draws its own title bar (a sidebar,
toolbar items) or the app opens several windows: see
[recipe 7](../ai/recipes.md#7-drawing-the-whole-window-yourself).

## Text and stacks

| Item | Defaults | Own methods |
|---|---|---|
| `Text::new(content)` | Font `BODY`, color `Palette::current().label`, wraps to as many lines as needed | `.font(Font)` |
| `VStack::new()` | Children in a column, centered, `SPACE_3` apart | — |
| `HStack::new()` | Children in a row, centered, `SPACE_3` apart | — |

`content` accepts a `&'static str`, a `String` or the result of `tr!`.

The three types wrap Freya's `label()` and `rect()` and forward all their
traits, so Freya's methods work on them: layout (`padding`, `width`,
`expanded`), text style (`color`, `font_size`), events (`on_press`) and, on the
stacks, children and alignment (`child`, `children`, `spacing`,
`main_align`, `cross_align`, `background`, `corner_radius`).

## The prelude

`use barrier_kit::prelude::*;` brings:

| From | Names |
|---|---|
| The kit | `Application`, `Scene`, `Text`, `VStack`, `HStack`, `run_window`, `WindowOptions`, `RunError`, `Palette`, `text`, `color`, `Titlebar` and every control in [`controls`](05-controls.md) |
| Freya | its whole prelude: `rect()`, `label()`, `use_state`, `Component`, `App`, `IntoElement`, element methods |
| Torin | `Alignment`, `Direction`, `Size` |
| Tokens | `BODY`, `CALLOUT`, `CAPTION`, `HEADLINE`, `LARGE_TITLE`, `MONO`, `TITLE`, `TITLE_2`, `Font`, `SPACE_1`…`SPACE_4`, `WINDOW_DEFAULT_SIZE`, `WINDOW_MIN_SIZE` |

Everything else is imported from its module, for example
`barrier_kit::views::form::Form` ([3](03-layers.md)).

Next: [3. Layers](03-layers.md).
