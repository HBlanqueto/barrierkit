use crate::controls::shadowed;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    ControlState, FloatingPaneStyle, LayoutDirection, Paints, ShapeStyle, StateValue, Theme,
    resolve,
};
use ctdesign::{FLOATING_PANE, Rgba};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Position, Size};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaneFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, PartialEq)]
struct Sides {
    left: bool,
    top: bool,
    right: bool,
    bottom: bool,
}

const MOVE: Sides = Sides {
    left: true,
    top: true,
    right: true,
    bottom: true,
};

impl Sides {
    fn apply(self, frame: PaneFrame, dx: f32, dy: f32) -> PaneFrame {
        if self == MOVE {
            return PaneFrame {
                x: frame.x + dx,
                y: frame.y + dy,
                ..frame
            };
        }
        let mut moved = frame;
        if self.left {
            moved.x += dx;
            moved.width -= dx;
        }
        if self.right {
            moved.width += dx;
        }
        if self.top {
            moved.y += dy;
            moved.height -= dy;
        }
        if self.bottom {
            moved.height += dy;
        }
        moved
    }

    fn cursor(self) -> CursorIcon {
        match (self.left || self.right, self.top || self.bottom) {
            (true, false) => CursorIcon::ColResize,
            (false, true) => CursorIcon::RowResize,
            _ if self.left == self.top => CursorIcon::NwseResize,
            _ => CursorIcon::NeswResize,
        }
    }
}

#[derive(PartialEq)]
pub struct FloatingPane {
    pub frame: PaneFrame,
    pub title: String,
    pub active: bool,
    pub accent: Rgba,
    pub title_bar: Rgba,
    pub title_ink: Rgba,
    pub content: Element,
    pub on_raise: Callback<(), ()>,
    pub on_frame: Callback<PaneFrame, ()>,
    pub style: Theme,
}

impl FloatingPane {
    pub fn style(
        mut self,
        restyle: impl Fn(FloatingPaneStyle) -> FloatingPaneStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl Component for FloatingPane {
    fn render(&self) -> impl IntoElement {
        let drag = use_state(|| None::<(Sides, (f64, f64), PaneFrame)>);
        let (accent, title_bar, title_ink) = (self.accent, self.title_bar, self.title_ink);
        let instance = self.style.clone();
        let style: FloatingPaneStyle =
            resolve(&Theme::tengen().style(move |s: FloatingPaneStyle| {
                instance.apply(FloatingPaneStyle {
                    outline: s.outline.clone().focused(ShapeStyle::Color(accent)),
                    title_bar: StateValue::new(ShapeStyle::Color(title_bar)),
                    title: StateValue::new(ShapeStyle::Color(title_ink)),
                    ..s
                })
            }));
        let state = ControlState {
            focused: self.active,
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let frame = self.frame;
        let band = FLOATING_PANE.resize_band;
        let corner_radius =
            style
                .shape
                .corner_radius(frame.width, frame.height, LayoutDirection::LeftToRight);
        let raise = self.on_raise.clone();
        let grab = move |sides: Sides| {
            let mut drag = drag;
            let raise = raise.clone();
            move |event: Event<MouseEventData>| {
                event.stop_propagation();
                raise.call(());
                drag.set(Some((
                    sides,
                    event.data().global_location.to_tuple(),
                    frame,
                )));
            }
        };
        let handle = |sides: Sides, position: Position, width: Size, height: Size| {
            rect()
                .position(position)
                .width(width)
                .height(height)
                .layer(1)
                .cursor(sides.cursor())
                .on_mouse_down(grab(sides))
                .into_element()
        };
        let side = |left, top, right, bottom| Sides {
            left,
            top,
            right,
            bottom,
        };
        let edge = Size::px(band);
        let handles = vec![
            handle(
                side(true, false, false, false),
                Position::new_absolute().left(0.0).top(band),
                edge.clone(),
                Size::fill(),
            ),
            handle(
                side(false, false, true, false),
                Position::new_absolute().right(0.0).top(band),
                edge.clone(),
                Size::fill(),
            ),
            handle(
                side(false, true, false, false),
                Position::new_absolute().top(0.0).left(band),
                Size::fill(),
                edge.clone(),
            ),
            handle(
                side(false, false, false, true),
                Position::new_absolute().bottom(0.0).left(band),
                Size::fill(),
                edge.clone(),
            ),
            handle(
                side(true, true, false, false),
                Position::new_absolute().left(0.0).top(0.0),
                edge.clone(),
                edge.clone(),
            ),
            handle(
                side(false, true, true, false),
                Position::new_absolute().right(0.0).top(0.0),
                edge.clone(),
                edge.clone(),
            ),
            handle(
                side(true, false, false, true),
                Position::new_absolute().left(0.0).bottom(0.0),
                edge.clone(),
                edge.clone(),
            ),
            handle(
                side(false, false, true, true),
                Position::new_absolute().right(0.0).bottom(0.0),
                edge.clone(),
                edge,
            ),
        ];
        let on_raise = self.on_raise.clone();
        let on_frame = self.on_frame.clone();
        let title_bar = rect()
            .width(Size::fill())
            .height(Size::px(style.title_height))
            .corner_radius(
                CornerRadius::new(
                    corner_radius.top_left(),
                    corner_radius.top_right(),
                    0.0,
                    0.0,
                )
                .with_smoothing(corner_radius.smoothing()),
            )
            .painted(&paints, &style.title_bar)
            .main_align(Alignment::Center)
            .cross_align(Alignment::Center)
            .cursor(CursorIcon::Grab)
            .on_mouse_down(grab(MOVE))
            .child(text(
                self.title.clone(),
                style.title_font,
                paints.color(style.title.resolve(state)),
            ));
        let pane = rect()
            .position(Position::new_absolute().left(band).top(band))
            .width(Size::px(frame.width))
            .height(Size::px(frame.height))
            .direction(Direction::Vertical)
            .content(Content::flex())
            .corner_radius(corner_radius)
            .painted(&paints, &style.background)
            .border(
                Border::new()
                    .width(style.outline_width)
                    .alignment(BorderAlignment::Outer)
                    .fill(crate::theme::color(
                        paints.color(style.outline.resolve(state)),
                    )),
            );
        rect()
            .position(
                Position::new_absolute()
                    .left(frame.x - band)
                    .top(frame.y - band),
            )
            .width(Size::px(frame.width + 2.0 * band))
            .height(Size::px(frame.height + 2.0 * band))
            .layer(Layer::Overlay)
            .cursor((*drag.read()).map(|(sides, ..)| {
                if sides == MOVE {
                    CursorIcon::Grab
                } else {
                    sides.cursor()
                }
            }))
            .on_mouse_down(move |_| on_raise.call(()))
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let Some((sides, (x, y), frame)) = *drag.peek() else {
                    return;
                };
                let (to_x, to_y) = event.data().global_location().to_tuple();
                on_frame.call(sides.apply(frame, (to_x - x) as f32, (to_y - y) as f32));
            })
            .on_global_pointer_up(move |_| {
                let mut drag = drag;
                if drag.peek().is_some() {
                    drag.set(None);
                }
            })
            .child(
                shadowed(pane, &style.shadows).child(title_bar).child(
                    rect()
                        .width(Size::fill())
                        .height(Size::flex(1.0))
                        .child(self.content.clone()),
                ),
            )
            .children(handles)
    }
}
