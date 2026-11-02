//! Split view: resizable sidebar that hides and shows with an animation, the
//! detail and an optional inspector.

use freya_animation::prelude::{AnimNum, Ease, Function, use_animation_transition};
use freya_core::prelude::*;
use keyboard_types::Modifiers;
use serde::{Deserialize, Serialize};
use torin::prelude::{Content, Direction, Gaps, Position, Size};
use vow_appearance::SidebarStyle;
use vow_design::{DURATION_BASE, SIDEBAR, SPLIT_VIEW};

use crate::app::{Action, Shortcut, ShortcutKey};
use crate::compact::IconButton;
use crate::style::Palette;
use crate::symbol::Symbol;
use crate::theme::color;

pub const TOGGLE_SIDEBAR: Action =
    Action::new("toggle-sidebar", "Mostrar u ocultar la barra lateral").shortcut(Shortcut::new(
        Modifiers::CONTROL.union(Modifiers::ALT),
        ShortcutKey::Char('s'),
    ));

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SidebarLayout {
    pub width: f32,
    pub visible: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub struct SidebarControl {
    visible: State<bool>,
    auto_hidden: State<bool>,
}

impl SidebarControl {
    pub fn visible(&self) -> bool {
        *self.visible.read()
    }

    pub fn toggle(self) {
        let (mut visible, mut auto_hidden) = (self.visible, self.auto_hidden);
        auto_hidden.set(false);
        let shown = *visible.peek();
        visible.set(!shown);
    }
}

pub fn sidebar_control() -> Option<SidebarControl> {
    try_consume_context()
}

#[derive(PartialEq)]
pub struct SidebarToggle;

impl Component for SidebarToggle {
    fn render(&self) -> impl IntoElement {
        let control = sidebar_control();
        IconButton::new(Symbol::SidebarLeft, TOGGLE_SIDEBAR.title)
            .disabled(control.is_none())
            .on_press(move |_| {
                if let Some(control) = control {
                    control.toggle();
                }
            })
    }
}

#[derive(PartialEq)]
pub struct SplitView {
    sidebar: Element,
    detail: Element,
    inspector: Option<Element>,
    sidebar_visible: bool,
    sidebar_width: Option<f32>,
    on_layout: Option<Callback<SidebarLayout, ()>>,
    key: DiffKey,
}

impl SplitView {
    pub fn new(sidebar: impl IntoElement, detail: impl IntoElement) -> Self {
        Self {
            sidebar: sidebar.into_element(),
            detail: detail.into_element(),
            inspector: None,
            sidebar_visible: true,
            sidebar_width: None,
            on_layout: None,
            key: DiffKey::None,
        }
    }

    pub fn inspector(mut self, inspector: Option<Element>) -> Self {
        self.inspector = inspector;
        self
    }

    pub fn sidebar_visible(mut self, visible: bool) -> Self {
        self.sidebar_visible = visible;
        self
    }

    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = Some(width);
        self
    }

    pub fn on_layout(mut self, callback: impl Into<Callback<SidebarLayout, ()>>) -> Self {
        self.on_layout = Some(callback.into());
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SplitView {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

fn clamp_sidebar(width: f32) -> f32 {
    width.clamp(SPLIT_VIEW.sidebar_min, SPLIT_VIEW.sidebar_max)
}

impl Component for SplitView {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let palette = Palette::current();
        let island = resolved.sidebar.style == SidebarStyle::Island;
        let inset = if island { SIDEBAR.inset as f32 } else { 0.0 };
        let mut window_width = use_state(|| f32::MAX);
        let mut sidebar_width =
            use_state(|| clamp_sidebar(self.sidebar_width.unwrap_or(SIDEBAR.width as f32)));
        let mut dragging = use_state(|| false);
        let mut visible = use_state(|| self.sidebar_visible);
        let mut auto_hidden = use_state(|| false);
        let control = SidebarControl {
            visible,
            auto_hidden,
        };
        use_hook(|| provide_context(control));

        let narrow = *window_width.read() < SPLIT_VIEW.collapse_below;
        let mut was_narrow = use_state(|| false);
        if *was_narrow.peek() != narrow {
            was_narrow.set(narrow);
            if narrow && *visible.peek() {
                visible.set(false);
                auto_hidden.set(true);
            } else if !narrow && *auto_hidden.peek() {
                visible.set(true);
                auto_hidden.set(false);
            }
        }

        let reduced_motion = resolved.reduced_motion;
        let reveal = use_animation_transition(visible, move |from: bool, to: bool| {
            AnimNum::new(f32::from(u8::from(from)), f32::from(u8::from(to)))
                .time(if reduced_motion { 0 } else { DURATION_BASE })
                .ease(Ease::InOut)
                .function(Function::Cubic)
        });
        let shown = reveal.get().value();

        let width = *sidebar_width.read();
        let layout = SidebarLayout {
            width,
            visible: *visible.read() || *auto_hidden.read(),
        };
        let mut reported = use_state(|| layout);
        if !*dragging.read() && *reported.peek() != layout {
            reported.set(layout);
            if let Some(callback) = &self.on_layout {
                callback.call(layout);
            }
        }

        let container = width + 2.0 * inset;
        let sidebar = || {
            rect()
                .width(Size::px(container))
                .height(Size::fill())
                .padding(Gaps::new_all(inset))
                .child(self.sidebar.clone())
        };

        let mut root = rect()
            .width(Size::fill())
            .height(Size::fill())
            .direction(Direction::Horizontal)
            .on_sized(move |event: Event<SizedEventData>| {
                window_width.set_if_modified(event.area.size.width);
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                if *dragging.read() {
                    sidebar_width.set(clamp_sidebar(event.global_location().x as f32 - inset));
                }
            })
            .on_global_pointer_press(move |_| dragging.set_if_modified(false))
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if TOGGLE_SIDEBAR
                    .shortcut
                    .is_some_and(|shortcut| shortcut.matches(event.modifiers, &event.key))
                {
                    control.toggle();
                }
            });

        if !narrow && shown > 0.0 {
            root = root.child(
                rect()
                    .width(Size::px(container * shown))
                    .height(Size::fill())
                    .overflow(Overflow::Clip)
                    .child(sidebar().maybe_child((shown >= 1.0).then(|| {
                        rect()
                            .a11y_role(AccessibilityRole::Splitter)
                            .position(
                                Position::new_absolute()
                                    .left(container - SPLIT_VIEW.grip / 2.0)
                                    .top(0.0),
                            )
                            .width(Size::px(SPLIT_VIEW.grip))
                            .height(Size::fill())
                            .on_mouse_down(move |_| dragging.set(true))
                    }))),
            );
        }
        root = root.child(
            rect()
                .width(Size::flex(1.0))
                .height(Size::fill())
                .child(self.detail.clone()),
        );
        if let Some(inspector) = &self.inspector {
            root = root
                .child(
                    rect()
                        .width(Size::px(1.0))
                        .height(Size::fill())
                        .background(color(palette.separator)),
                )
                .child(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector))
                        .height(Size::fill())
                        .child(inspector.clone()),
                );
        }
        if narrow && shown > 0.0 {
            root = root.child(
                sidebar()
                    .position(
                        Position::new_absolute()
                            .left(-container * (1.0 - shown))
                            .top(0.0),
                    )
                    .layer(Layer::Overlay),
            );
        }
        root.content(Content::flex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_width_stays_between_its_minimum_and_maximum() {
        assert_eq!(clamp_sidebar(0.0), SPLIT_VIEW.sidebar_min);
        assert_eq!(clamp_sidebar(10_000.0), SPLIT_VIEW.sidebar_max);
        assert_eq!(clamp_sidebar(SIDEBAR.width as f32), SIDEBAR.width as f32);
    }
}
