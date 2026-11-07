# Recipes

**Goal:** start a program from a whole `src/main.rs` that already compiles.
**Scope:** one program per common task. Each item's reference lives in its
chapter; the names are in the [glossary](../barrierkit/20-glossary.md).
**Reader:** an AI agent sent here by [`prompt.md`](prompt.md), or a person.
**Input:** a task. **Output:** the closest program, ready to change.

## Steps

1. Find the recipe closest to the task in the list below.
2. Copy it into `src/main.rs`.
3. Change only what the task needs.
4. Build it inside `nix develop` and go through the [checklist](checklist.md).

## 1. Text in a window

SwiftUI: `WindowGroup { Text("Hello, world").font(.title).padding(40) }`

The program is in [chapter 2, Hello, world](../barrierkit/02-first-app.md#hello-world).

## 2. State and a button

SwiftUI: `@State var count = 0` with `VStack { Text("\(count)"); Button("Add one") { count += 1 } }`

The program is in [chapter 2, Adding state](../barrierkit/02-first-app.md#adding-state).

## 3. Rows and columns

SwiftUI: `VStack(alignment: .leading, spacing: 4) { HStack { … } }`

```rust
use bk::prelude::*;

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
use bk::prelude::*;

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
use bk::prelude::*;
use bk::views::form::{Form, Row, Section};

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

For a real preferences window use `SettingsWindow` (chapter
[12](../barrierkit/12-patterns.md)).

## 6. A reusable view

SwiftUI: `struct Greeting: View { let name: String; var body: some View { … } }`

```rust
use bk::prelude::*;

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
use bk::prelude::*;

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

Follow the steps in chapter [17](../barrierkit/17-languages.md); then every
literal above becomes `tr!("id")`:

```rust
Text::new(tr!("hello")).font(TITLE)
```

## 9. A stopwatch

SwiftUI: a `Text` fed by `TimelineView(.periodic(from:by:))` with Start/Stop and Lap/Reset buttons.

```rust
use std::time::{Duration, Instant};

use bk::platform::Timers;
use bk::prelude::*;

const TICK: Duration = Duration::from_millis(10);

fn main() -> Result<(), RunError> {
    Application::new("stopwatch")
        .window("Stopwatch", || {
            let mut started = use_state(|| None::<Instant>);
            let mut stored = use_state(|| Duration::ZERO);
            let now = use_state(Instant::now);
            let mut laps = use_state(Vec::<Duration>::new);

            use_side_effect(move || {
                let _ = *now.read();
                if started.read().is_some() {
                    Timers::get().after(TICK, move || {
                        let mut now = now;
                        now.set(Instant::now());
                    });
                }
            });

            let running = started.read().is_some();
            let elapsed = *stored.read()
                + started
                    .read()
                    .map_or(Duration::ZERO, |start| now.read().duration_since(start));

            VStack::new()
                .child(
                    Text::new(format_time(elapsed))
                        .font(LARGE_TITLE)
                        .font_family(MONO.family),
                )
                .child(
                    HStack::new()
                        .child(
                            Button::new(if running { "Lap" } else { "Reset" })
                                .disabled(!running && elapsed.is_zero())
                                .on_press(move |_| {
                                    if running {
                                        laps.write().push(elapsed);
                                    } else {
                                        stored.set(Duration::ZERO);
                                        laps.write().clear();
                                    }
                                }),
                        )
                        .child(
                            Button::new(if running { "Stop" } else { "Start" })
                                .style(ButtonStyle::Primary)
                                .on_press(move |_| {
                                    if running {
                                        stored.set(elapsed);
                                        started.set(None);
                                    } else {
                                        started.set(Some(Instant::now()));
                                    }
                                }),
                        ),
                )
                .children(laps.read().iter().enumerate().rev().map(|(index, lap)| {
                    Text::new(format!("Lap {}   {}", index + 1, format_time(*lap)))
                        .font_family(MONO.family)
                        .into_element()
                }))
        })
        .options(|window| window.size((320, 420)).min_size(Some(WINDOW_MIN_SIZE)))
        .run()
}

fn format_time(elapsed: Duration) -> String {
    let centiseconds = elapsed.as_millis() / 10;
    format!(
        "{:02}:{:02}.{:02}",
        centiseconds / 6000,
        centiseconds / 100 % 60,
        centiseconds % 100
    )
}
```

`Timers::after` runs a callback once after a delay ([8](../barrierkit/08-windows.md)).
`use_side_effect` runs again when `now` or `started` change, so each tick
schedules the next one while the stopwatch runs.

