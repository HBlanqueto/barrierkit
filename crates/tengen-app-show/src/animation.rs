use std::sync::LazyLock;

use barrier_kit::backdrop::{Accent, Tint};
use barrier_kit::controls::{Button, ButtonStyle, SegmentedControl, Switch};
use barrier_kit::patterns::preferences::{Pane, PaneGroup};
use barrier_kit::progress::{ProgressBar, Spinner};
use barrier_kit::symbol::Symbol;
use barrier_kit::views::form::{Form, Row, Section};
use freya_animation::prelude::Function;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Size};
use vow_design::{DURATION_BASE, DURATION_SLOW_MS, SPACE_2, SPLIT_VIEW, SWITCH};

use crate::chrome::segmented;
use crate::events::{
    Arrivals, CopyToast, HoverCard, Inbox, PlusToggle, Popover, PressCard, ShakeField,
};
use crate::kit::{Download, SCENES, sidebar_section};
use crate::localize::tr;
use crate::motion::{EasedTrack, Property, PropertyTile, SpringTrack};
use crate::slides::{Stage, pager};
use crate::{preview, tint};

static CURVES: LazyLock<[(&str, Function); 7]> = LazyLock::new(|| {
    [
        (tr!("linear"), Function::Linear),
        (tr!("sine"), Function::Sine),
        (tr!("cubic"), Function::Cubic),
        (tr!("exponential"), Function::Expo),
        (tr!("back"), Function::Back),
        (tr!("elastic"), Function::Elastic),
        (tr!("bounce"), Function::Bounce),
    ]
});

static SPRINGS: LazyLock<[(&str, f32); 3]> = LazyLock::new(|| {
    [
        (tr!("stiff"), 0.0),
        (tr!("smooth"), 0.3),
        (tr!("lively"), 0.7),
    ]
});

static PROPERTIES: LazyLock<[(&str, Property); 7]> = LazyLock::new(|| {
    [
        (tr!("opacity"), Property::Opacity),
        (tr!("scale"), Property::Scale),
        (tr!("rotation"), Property::Rotation),
        (tr!("offset"), Property::Offset),
        (tr!("color"), Property::Tint),
        (tr!("radius"), Property::Radius),
        (tr!("width"), Property::Width),
    ]
});

fn curves(mut ahead: State<bool>) -> Form {
    let at_end = *ahead.read();
    let play = Row::new(tr!("play")).control(
        Button::new(if at_end { tr!("go-back") } else { tr!("play") })
            .style(ButtonStyle::Primary)
            .on_press(move |_| ahead.toggle()),
    );
    let durations = [
        (tr!("fast"), SWITCH.transition_ms),
        (tr!("base"), DURATION_BASE),
        (tr!("slow"), DURATION_SLOW_MS),
    ];
    Form::new()
        .section(CURVES.iter().fold(
            Section::titled(tr!("freya-curves")).row(play),
            |section, &(name, function)| {
                section.row(Row::new(name).control(EasedTrack {
                    ahead: at_end,
                    function,
                    ms: DURATION_SLOW_MS,
                }))
            },
        ))
        .section(SPRINGS.iter().fold(
            Section::titled(tr!("grade-spring")),
            |section, &(name, bounce)| {
                section.row(Row::new(name).control(SpringTrack {
                    ahead: at_end,
                    bounce,
                }))
            },
        ))
        .section(durations.into_iter().fold(
            Section::titled(tr!("tengen-durations")),
            |section, (name, ms)| {
                section.row(Row::new(name).control(EasedTrack {
                    ahead: at_end,
                    function: Function::Cubic,
                    ms,
                }))
            },
        ))
}

fn properties(mut on: State<bool>) -> Form {
    let animated = *on.read();
    let toggle = Row::new(tr!("animate")).control(
        Switch::new(tr!("animate"))
            .on(animated)
            .on_toggle(move |value: bool| on.set(value)),
    );
    Form::new().section(PROPERTIES.iter().fold(
        Section::titled(tr!("properties")).row(toggle),
        |section, &(name, property)| {
            section.row(Row::new(name).control(PropertyTile {
                on: animated,
                property,
            }))
        },
    ))
}

fn kit_controls(mut backdrop: State<usize>) -> Form {
    let selected = *backdrop.read();
    Form::new()
        .section(
            Section::titled(tr!("kit-transitions"))
                .row(Row::new(tr!("switch")).control(Switch::new(tr!("animated"))))
                .row(Row::new(tr!("sidebar-section")).control(sidebar_section()))
                .row(
                    Row::new(tr!("indeterminate")).control(
                        rect()
                            .width(Size::px(SPLIT_VIEW.inspector))
                            .child(ProgressBar::indeterminate(tr!("searching"))),
                    ),
                )
                .row(Row::new(tr!("activity")).control(Spinner::new(tr!("loading"))))
                .row(Row::new(tr!("download-row")).control(Download)),
        )
        .section(
            Section::titled(tr!("tinted-backdrop"))
                .row(Row::new(tr!("scene")).control(segmented(
                    tr!("scene"),
                    &SCENES.map(|(name, _)| name),
                    selected,
                    move |index| backdrop.set(index),
                )))
                .row(
                    Row::new(tr!("fade"))
                        .detail(tr!("fade-detail"))
                        .control(preview(
                            SCENES[selected].1,
                            Accent::FromBackdrop,
                            Tint::Accent,
                        )),
                ),
        )
}

fn appearing() -> Form {
    Form::new().section(
        Section::titled(tr!("enter-and-exit"))
            .row(Row::new(tr!("list")).control(Arrivals))
            .row(Row::new(tr!("card")).control(Popover)),
    )
}

fn navigation(page: State<usize>, mut transition: State<usize>, mut period: State<usize>) -> Form {
    let style = *transition.read();
    let shown_period = *period.read();
    Form::new()
        .section(
            Section::titled(tr!("slides"))
                .row(Row::new(tr!("transition")).control(segmented(
                    tr!("transition"),
                    &[
                        tr!("slide"),
                        tr!("fade"),
                        tr!("zoom"),
                        tr!("cover"),
                        tr!("spring"),
                    ],
                    style,
                    move |index| transition.set(index),
                )))
                .row(
                    Row::new(tr!("carousel")).control(
                        rect()
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .child(Stage {
                                page: *page.read(),
                                transition: style,
                            })
                            .child(pager(page)),
                    ),
                ),
        )
        .section(
            Section::titled(tr!("tabs")).row(
                Row::new(tr!("with-segments"))
                    .detail(tr!("with-segments-detail"))
                    .control(
                        rect()
                            .cross_align(Alignment::Center)
                            .spacing(SPACE_2)
                            .child(
                                SegmentedControl::new(tr!("period"))
                                    .segment(tr!("day"))
                                    .segment(tr!("week"))
                                    .segment(tr!("month"))
                                    .selected(shown_period)
                                    .on_change(move |index: usize| period.set(index)),
                            )
                            .child(Stage {
                                page: shown_period,
                                transition: 0,
                            }),
                    ),
            ),
        )
}

fn feedback() -> Form {
    Form::new()
        .section(
            Section::titled(tr!("pointer"))
                .row(Row::new(tr!("hover")).control(HoverCard))
                .row(
                    Row::new(tr!("press"))
                        .detail(tr!("press-detail"))
                        .control(PressCard),
                )
                .row(Row::new(tr!("toggle")).control(PlusToggle)),
        )
        .section(
            Section::titled(tr!("response"))
                .row(
                    Row::new(tr!("error"))
                        .detail(tr!("error-detail"))
                        .control(ShakeField),
                )
                .row(Row::new(tr!("notification")).control(Inbox))
                .row(Row::new(tr!("notice")).control(CopyToast)),
        )
}

fn accessibility(mut reduced: State<bool>) -> Form {
    Form::new().section(
        Section::titled(tr!("accessibility")).row(
            Row::new(tr!("reduce-motion"))
                .detail(tr!("reduce-motion-detail"))
                .control(
                    Switch::new(tr!("reduce-motion"))
                        .on(*reduced.read())
                        .on_toggle(move |on: bool| reduced.set(on)),
                ),
        ),
    )
}

pub fn group(reduced: State<bool>) -> PaneGroup {
    let ahead = use_state(|| false);
    let animated = use_state(|| false);
    let backdrop = use_state(|| 0);
    let page = use_state(|| 0);
    let transition = use_state(|| 0);
    let period = use_state(|| 0);
    PaneGroup::new(Some(tr!("animation")))
        .pane(Pane::new(
            "curvas",
            tr!("curves"),
            Symbol::SlidersVertical,
            tint(0),
            curves(ahead),
        ))
        .pane(Pane::new(
            "propiedades",
            tr!("properties"),
            Symbol::SquareGrid,
            tint(1),
            properties(animated),
        ))
        .pane(Pane::new(
            "controles",
            tr!("kit-in-motion"),
            Symbol::ChevronRight,
            tint(6),
            kit_controls(backdrop),
        ))
        .pane(Pane::new(
            "aparicion",
            tr!("appearing"),
            Symbol::Plus,
            tint(4),
            appearing(),
        ))
        .pane(Pane::new(
            "diapositivas",
            tr!("slides"),
            Symbol::Play,
            tint(7),
            navigation(page, transition, period),
        ))
        .pane(Pane::new(
            "eventos",
            tr!("pointer-and-response"),
            Symbol::CircleHalf,
            tint(5),
            feedback(),
        ))
        .pane(Pane::new(
            "movimiento",
            tr!("motion"),
            Symbol::Pause,
            tint(2),
            accessibility(reduced),
        ))
}
