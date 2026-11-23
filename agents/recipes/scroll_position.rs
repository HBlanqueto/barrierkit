use bk::animation::with_animation;
use bk::prelude::*;
use ctdesign::SPRING_SHEET_MS;

const ROWS: usize = 100;

fn main() -> Result<(), RunError> {
    Application::new("scroll-position")
        .window("Scroll position", || {
            let scroll = use_scroll_controller();
            let mut position = use_state(ScrollPosition::default);
            let mut offset = use_state(|| 0.0f32);
            let mut phase = use_state(ScrollPhase::default);
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_2)
                .child(
                    HStack::new()
                        .spacing(SPACE_2)
                        .child(
                            Button::new("Top")
                                .on_press(move |_| position.set(ScrollPosition::edge(Edge::Top))),
                        )
                        .child(Button::new("Row 40").on_press(move |_| {
                            position.set(ScrollPosition::id("row-40", Some(Alignment::Center)))
                        }))
                        .child(Button::new("Bottom, animated").on_press(move |_| {
                            with_animation(
                                Animation::Spring {
                                    duration_ms: SPRING_SHEET_MS,
                                    bounce: 0.0,
                                },
                                || position.set(ScrollPosition::edge(Edge::Bottom)),
                            )
                        }))
                        .child(Button::new("Row 10, by the handle").on_press(move |_| {
                            scroll.scroll_to(
                                ScrollPosition::id("row-10", Some(Alignment::Start)),
                                Animation::Ease { duration_ms: 300 },
                            )
                        })),
                )
                .child(Text::new(format!(
                    "Offset {:.0}, {:?}, by the user: {}",
                    *offset.read(),
                    *phase.read(),
                    position.read().is_positioned_by_user()
                )))
                .child(
                    ScrollView::new()
                        .controller(scroll)
                        .scroll_position(position)
                        .on_scroll_geometry_change(move |geometry: ScrollGeometry| {
                            offset.set_if_modified(geometry.content_offset.1)
                        })
                        .on_scroll_phase_change(move |(_, now): (ScrollPhase, ScrollPhase)| {
                            phase.set_if_modified(now)
                        })
                        .children((0..ROWS).map(|row| {
                            Text::new(format!("Row {row}"))
                                .padding(SPACE_2)
                                .id(format!("row-{row}"))
                                .into_element()
                        })),
                )
        })
        .run()
}
