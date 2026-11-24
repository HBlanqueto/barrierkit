use std::time::{Duration, Instant};

use bk::app::use_timeline;
use bk::prelude::*;

const TICK: Duration = Duration::from_millis(10);

fn main() -> Result<(), RunError> {
    Application::new("com.example.stopwatch")
        .window("Stopwatch", || {
            let mut started = use_state(|| None::<Instant>);
            let mut stored = use_state(|| Duration::ZERO);
            let mut laps = use_state(Vec::<Duration>::new);

            let running = started.read().is_some();
            use_timeline(running, TICK);
            let elapsed = *stored.read()
                + started
                    .read()
                    .map_or(Duration::ZERO, |start| Instant::now().duration_since(start));

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
                                .style(ButtonStyle::bordered_prominent)
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
