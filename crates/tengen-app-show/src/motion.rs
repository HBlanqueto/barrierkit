use std::time::Duration;

use barrier_kit::animation::{Spring, motion_ms, use_reduced_motion};
use barrier_kit::style::Palette;
use barrier_kit::theme::color;
use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use torin::prelude::{Position, Size};
use vow_design::{COLLECTION, DURATION_SLOW_MS, SPLIT_VIEW};

const DOT: f32 = 16.0;

pub fn spring(reduced_motion: State<bool>, from: f32, to: f32, ms: u64, bounce: f32) -> Spring {
    Spring::new(
        from,
        to,
        Duration::from_millis(motion_ms(reduced_motion, ms)),
        bounce,
    )
}

pub fn unit(on: bool) -> f32 {
    f32::from(u8::from(on))
}

fn lane(progress: f32) -> Rect {
    let palette = Palette::current();
    rect()
        .width(Size::px(SPLIT_VIEW.inspector))
        .height(Size::px(DOT))
        .corner_radius(DOT / 2.0)
        .background(color(palette.sunken_background))
        .child(
            rect()
                .width(Size::px(DOT))
                .height(Size::px(DOT))
                .corner_radius(DOT / 2.0)
                .background(color(palette.accent))
                .position(
                    Position::new_absolute()
                        .left(progress * (SPLIT_VIEW.inspector - DOT))
                        .top(0.0),
                ),
        )
}

#[derive(PartialEq)]
pub struct EasedTrack {
    pub ahead: bool,
    pub function: Function,
    pub ms: u64,
}

impl Component for EasedTrack {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let ahead = use_reactive(&self.ahead);
        let (function, ms) = (self.function, self.ms);
        let animation = use_animation_transition(ahead, move |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(motion_ms(reduced_motion, ms))
                .ease(Ease::InOut)
                .function(function)
        });
        lane(animation.get().value())
    }
}

#[derive(PartialEq)]
pub struct SpringTrack {
    pub ahead: bool,
    pub bounce: f32,
}

impl Component for SpringTrack {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let ahead = use_reactive(&self.ahead);
        let bounce = self.bounce;
        let animation = use_animation_transition(ahead, move |from, to| {
            spring(
                reduced_motion,
                unit(from),
                unit(to),
                DURATION_SLOW_MS,
                bounce,
            )
        });
        lane(animation.get().current())
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Property {
    Opacity,
    Scale,
    Rotation,
    Offset,
    Tint,
    Radius,
    Width,
}

#[derive(PartialEq)]
pub struct PropertyTile {
    pub on: bool,
    pub property: Property,
}

impl Component for PropertyTile {
    fn render(&self) -> impl IntoElement {
        let reduced_motion = use_reduced_motion();
        let on = use_reactive(&self.on);
        let animation = use_animation_transition(on, move |from, to| {
            AnimNum::new(unit(from), unit(to))
                .time(motion_ms(reduced_motion, DURATION_SLOW_MS))
                .ease(Ease::InOut)
                .function(Function::Cubic)
        });
        let progress = animation.get().value();
        let palette = Palette::current();
        let size = COLLECTION.icon;
        let travel = SPLIT_VIEW.inspector - size;
        let tile = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(COLLECTION.radius)
            .background(color(palette.accent));
        let tile = match self.property {
            Property::Opacity => tile.opacity(1.0 - 0.8 * progress),
            Property::Scale => tile.scale(1.0 - 0.4 * progress),
            Property::Rotation => tile.rotate(90.0 * progress),
            Property::Offset => {
                tile.position(Position::new_absolute().left(progress * travel).top(0.0))
            }
            Property::Tint => {
                tile.background(color(palette.accent.mix(palette.destructive, progress)))
            }
            Property::Radius => {
                tile.corner_radius(COLLECTION.radius + (size / 2.0 - COLLECTION.radius) * progress)
            }
            Property::Width => tile.width(Size::px(size + travel * progress)),
        };
        rect()
            .width(Size::px(SPLIT_VIEW.inspector))
            .height(Size::px(size))
            .child(tile)
    }
}
