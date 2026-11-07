# Recipes

Read from [`prompt.md`](prompt.md). Each recipe is a whole `src/main.rs` that
compiles against the current `main`. Start from the closest one and change only
what the task needs. Names are explained in the
[glossary](../barrierkit/15-glossary.md).

## 1. Text in a window

SwiftUI: `WindowGroup { Text("Hello, world").font(.title).padding(40) }`

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("hello")
        .window("Hello", || Text::new("Hello, world").font(TITLE).padding(40.))
        .run()
}
```

## 2. State and a button

SwiftUI: `@State var count = 0` with `VStack { Text("\(count)"); Button("Add one") { count += 1 } }`

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

## 3. Rows and columns

SwiftUI: `VStack(alignment: .leading, spacing: 4) { HStack { … } }`

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("profile")
        .window("Profile", || {
            VStack::new()
                .cross_align(Alignment::Start)
                .spacing(SPACE_1)
                .child(Text::new("Ana Díaz").font(HEADLINE))
                .child(
                    HStack::new()
                        .child(Text::new("Madrid").font(CAPTION))
                        .child(Text::new("Designer").font(CAPTION)),
                )
        })
        .run()
}
```

## 4. A toggle bound to state

SwiftUI: `Toggle("Wi-Fi", isOn: $wifi)`

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    Application::new("network")
        .window("Network", || {
            let mut wifi = use_state(|| true);
            VStack::new()
                .child(Switch::new("Wi-Fi").on(*wifi.read()).on_toggle(move |on| wifi.set(on)))
                .child(Text::new(if *wifi.read() { "Connected" } else { "Off" }))
        })
        .run()
}
```

## 5. A settings form

SwiftUI: `Form { Section("General") { Toggle("Sounds", isOn: $sounds) } }`

```rust
use barrier_kit::prelude::*;
use barrier_kit::views::form::{Form, Row, Section};

fn main() -> Result<(), RunError> {
    Application::new("settings-sample")
        .window("Settings", || {
            let mut sounds = use_state(|| true);
            Form::new().section(
                Section::titled("General").row(
                    Row::new("Sounds").control(
                        Switch::new("Sounds")
                            .on(*sounds.read())
                            .on_toggle(move |on| sounds.set(on)),
                    ),
                ),
            )
        })
        .run()
}
```

For a real preferences window use `PreferencesWindow` (chapter
[10](../barrierkit/10-patterns.md)).

## 6. A reusable view

SwiftUI: `struct Greeting: View { let name: String; var body: some View { … } }`

```rust
use barrier_kit::prelude::*;

#[derive(PartialEq)]
struct Greeting {
    name: &'static str,
}

impl Component for Greeting {
    fn render(&self) -> impl IntoElement {
        Text::new(format!("Hello, {}", self.name)).font(HEADLINE)
    }
}

fn main() -> Result<(), RunError> {
    Application::new("greeting")
        .window("Greeting", || {
            VStack::new()
                .child(Greeting { name: "Ana" })
                .child(Greeting { name: "Luis" })
        })
        .run()
}
```

A `Component` must be `PartialEq`: the kit redraws it only when its fields
change.

## 7. Drawing the whole window yourself

Use it when the titlebar needs a sidebar or toolbar items. `run_window` takes
the same closure but adds nothing: you draw the background and the titlebar.

```rust
use barrier_kit::prelude::*;

fn main() -> Result<(), RunError> {
    run_window(
        || {
            rect()
                .expanded()
                .background(color(Palette::current().window_background))
                .child(Titlebar::new().title("Notes").with_sidebar(true))
                .child(Text::new("No notes yet").padding(SPACE_4))
        },
        WindowOptions::new("Notes", "notes"),
    )
}
```

## 8. Translated text

Follow the steps in chapter [18](../barrierkit/18-languages.md); then every
literal above becomes `tr!("id")`:

```rust
Text::new(tr!("hello")).font(TITLE)
```
