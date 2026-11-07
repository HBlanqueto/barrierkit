# 2. Your first app

**Goal:** open a window, show text, add state, and understand the ideas the
whole kit rests on.
**Reader:** anyone who has never used BarrierKit or Freya.
**Input:** the repository and `nix develop`. **Output:** a window that adds.
**Names:** every name used here is in the [glossary](15-glosario.md).

## Hello, world

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("hello")
        .window("Hello", || Text::new("Hello, world").font(TITLE).padding(40.))
        .run()
}
```

The same program in SwiftUI, line by line:

| SwiftUI | BarrierKit |
|---|---|
| `import SwiftUI` | `use barrier_kit::prelude::*;` |
| `@main struct HelloApp: App` | `fn main()` + `Application::new("hello")` |
| `WindowGroup { … }` | `.window("Hello", \|\| …)` |
| `Text("Hello, world")` | `Text::new("Hello, world")` |
| `.font(.title)` | `.font(TITLE)` |
| `.padding(40)` | `.padding(40.)` |
| (implicit) | `.run()` |

## Running it

1. Enter the dev shell: `nix develop`.
2. Create a binary crate with one dependency:

   ```toml
   [dependencies]
   barrier-kit = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<commit on main>" }
   ```

3. Put the code in `src/main.rs` and launch it with `cargo run`.

The window opens with the titlebar, the traffic lights and the window
background, and the text sits in the center.

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

Press «Add one»: the number goes up and the window repaints.

## The ideas

1. **The interface is a function of the state.** The closure passed to
   `.window` describes how the window looks now. Nothing is moved or deleted
   by hand: when the state changes, the kit calls the closure again and
   applies the difference.
2. **The app holds the state.** `use_state(|| 0)` creates a value that
   survives between calls. `read()` reads it and `write()` changes it. The
   kit's controls are *controlled*: they receive the value (`value`, `on`,
   `selected`) and report the change (`on_change`, `on_toggle`, `on_press`).
3. **Everything is built by chaining methods.** `Type::new(..).option(..)`:
   `Text`, `VStack`, `Button` and Freya's `rect()` all work the same way.
   Layout and event methods (`padding`, `width`, `expanded`, `on_press`) work
   on `Text`, `VStack` and `HStack` too.
4. **No color or size is written by hand.** `Text` takes its color from the
   system palette, and sizes and typefaces come from the tokens (`TITLE`,
   `SPACE_3`, `WINDOW_MIN_SIZE`). The app follows light or dark mode and the
   accent on its own (chapter [4](04-apariencia-y-tokens.md)).
5. **One import.** `barrier_kit::prelude::*` brings the kit's entry points,
   the controls, Freya's elements and hooks, and the type and spacing tokens.

## What `Application` does for you

`Application::new(name).window(title, content)` returns a `Scene`. Its
`run()`:

1. Builds `WindowOptions::new(title, name)`. The `name` is the short name
   behind the `app_id` and the saved window size (chapter
   [6](06-ventanas.md)).
2. Draws the window background (`Palette::current().window_background`).
3. Adds a `Titlebar` with the title.
4. Centers `content` in the rest of the window.
5. Calls `run_window`.

`.options(|window| …)` gives access to every `WindowOptions` setting: size,
limits, menu, full-screen bar.

## When you need more control

`Scene` covers a window with a titlebar and centered content. When the window
needs its own titlebar (a sidebar, toolbar items) or several windows, call
`run_window` directly. It takes the same closure, or a type that implements
`App`:

```rust
fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .background(color(Palette::current().window_background))
                .child(Titlebar::new().title("Notes").with_sidebar(true))
                .child(Text::new("…"))
        },
        WindowOptions::new("Notes", "notes"),
    )
}
```

## App and Component

A larger app moves its views into types. The root implements `App`. Any
reusable piece implements `Component` and must be `PartialEq`, so the kit
knows when to draw it again:

```rust
use barrier_kit::prelude::*;

#[derive(PartialEq)]
struct Greeting {
    name: &'static str,
}

impl Component for Greeting {
    fn render(&self) -> impl IntoElement {
        Text::new(self.name).font(HEADLINE)
    }
}

fn main() -> Result<(), RunError> {
    Application::new("greeting")
        .window("Greeting", || Greeting { name: "Ana" })
        .run()
}
```

Next: [3. Layers](03-capas.md).
