use freya_animation::prelude::use_animation_transition;
use std::time::Duration;

use ct_appearance::SidebarStyle;
use ct_design::{DURATION_BASE, DURATION_FAST_MS, SIDEBAR, SPLIT_VIEW};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::animation::Spring;
use crate::app::{Action, Shortcut, ShortcutKey};
use crate::compact::IconButton;
use crate::i18n::text;
use crate::sidebar::{SidebarControl, sidebar_control};
use crate::style::Palette;
use crate::symbol::Symbol;
use crate::theme::{SurfaceKind, color, surface_background};

pub const TOGGLE_SIDEBAR: Action = Action::new("toggle-sidebar", || text!("toggle-sidebar"))
    .shortcut(Shortcut::new(
        Modifiers::CONTROL.union(Modifiers::ALT),
        ShortcutKey::Char('s'),
    ));

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SidebarLayout {
    pub width: f32,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NarrowSidebar {
    #[default]
    Overlay,
    Push,
}

#[derive(PartialEq)]
pub struct SidebarToggle;

impl Component for SidebarToggle {
    fn render(&self) -> impl IntoElement {
        let control = sidebar_control();
        rect().probe("sidebar-toggle").child(
            IconButton::new(Symbol::SidebarLeft, TOGGLE_SIDEBAR.title())
                .disabled(control.is_none())
                .on_press(move |_| {
                    if let Some(control) = control {
                        control.toggle();
                    }
                }),
        )
    }
}

#[derive(PartialEq)]
pub struct SplitView {
    sidebar: Element,
    detail: Element,
    inspector: Option<(Element, bool)>,
    extension: Option<Element>,
    sidebar_visible: bool,
    sidebar_width: Option<f32>,
    sidebar_range: (f32, f32),
    resizable: bool,
    narrow_sidebar: NarrowSidebar,
    on_layout: Option<Callback<SidebarLayout, ()>>,
    key: DiffKey,
}

impl SplitView {
    pub fn new(sidebar: impl IntoElement, detail: impl IntoElement) -> Self {
        Self {
            sidebar: sidebar.into_element(),
            detail: detail.into_element(),
            inspector: None,
            extension: None,
            sidebar_visible: true,
            sidebar_width: None,
            sidebar_range: (SPLIT_VIEW.sidebar_min, SPLIT_VIEW.sidebar_max),
            resizable: true,
            narrow_sidebar: NarrowSidebar::Overlay,
            on_layout: None,
            key: DiffKey::None,
        }
    }

    pub fn inspector(mut self, inspector: impl IntoElement, presented: bool) -> Self {
        self.inspector = Some((inspector.into_element(), presented));
        self
    }

    pub fn background_extension(mut self, view: impl IntoElement) -> Self {
        self.extension = Some(view.into_element());
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

    pub fn sidebar_range(mut self, min: f32, max: f32) -> Self {
        self.sidebar_range = (min, max);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn narrow_sidebar(mut self, narrow_sidebar: NarrowSidebar) -> Self {
        self.narrow_sidebar = narrow_sidebar;
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

impl Component for SplitView {
    fn render(&self) -> impl IntoElement {
        let resolved = crate::theme::resolved();
        let palette = Palette::current();
        let island = resolved.sidebar.style == SidebarStyle::Island;
        let inset = if island { SIDEBAR.inset as f32 } else { 0.0 };
        let mut window_width = use_state(|| f32::MAX);
        let mut presented = use_state(|| self.inspector.as_ref().is_some_and(|(_, shown)| *shown));
        let wanted = self.inspector.as_ref().is_some_and(|(_, shown)| *shown);
        if *presented.peek() != wanted {
            presented.set(wanted);
        }
        let inspector_shown = crate::animation::use_switch_progress(presented, DURATION_BASE);
        let (min_width, max_width) = self.sidebar_range;
        let initial_width = self
            .sidebar_width
            .unwrap_or(SIDEBAR.width as f32)
            .clamp(min_width, max_width);
        let mut sidebar_width = use_state(|| initial_width);
        let mut dragging = use_state(|| false);
        let mut grip_hovered = use_state(|| false);
        let grip_id = use_a11y();
        let grip_focus = use_focus(grip_id);
        use_drop(move || {
            if *grip_hovered.peek() || *dragging.peek() {
                Cursor::set(CursorIcon::default());
            }
        });
        let mut visible = use_state(|| self.sidebar_visible);
        let mut auto_hidden = use_state(|| false);
        let mut occupied = use_state(|| 0.0f32);
        let mut settled = use_state(|| false);
        let mut overlaid = use_state(|| false);
        let control = SidebarControl {
            visible,
            auto_hidden,
            occupied,
            settled,
            overlaid,
        };
        use_hook(|| provide_context(control));

        let narrow = *window_width.read() < SPLIT_VIEW.collapse_below;
        let overlay = narrow && self.narrow_sidebar == NarrowSidebar::Overlay;
        let mut was_narrow = use_state(|| false);
        if *was_narrow.peek() != narrow {
            was_narrow.set(narrow);
            if narrow && *visible.peek() {
                visible.set(false);
                auto_hidden.set(true);
            } else if !narrow {
                visible.set(true);
                auto_hidden.set(false);
            }
        }

        let shown = crate::animation::use_switch_progress(visible, DURATION_BASE);
        crate::chrome::report_sidebar_shown(*visible.read());

        let reduced_motion = use_reactive(&resolved.reduced_motion);
        let mut follow_from_to = use_state(|| (initial_width, initial_width));
        let follow = use_animation_transition(follow_from_to, move |_, (from, to)| {
            let time = if *reduced_motion.peek() {
                0
            } else {
                DURATION_FAST_MS
            };
            Spring::new(from, to, Duration::from_millis(time), 0.0)
        });
        let mut resize_to = move |target: f32| {
            let target = target.clamp(min_width, max_width);
            let from = if *dragging.peek() {
                target
            } else {
                follow.get().current()
            };
            follow_from_to.set((from, target));
            sidebar_width.set(target);
        };
        let width = if *dragging.read() {
            *sidebar_width.read()
        } else {
            follow.get().current().clamp(min_width, max_width)
        };
        let layout = SidebarLayout {
            width: sidebar_width.read().clamp(min_width, max_width),
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
        occupied.set_if_modified(if overlay { 0.0 } else { container * shown });
        overlaid.set_if_modified(overlay && *visible.read());
        settled.set_if_modified(*visible.read() && shown >= 1.0);
        let sidebar = || {
            rect()
                .width(Size::px(container))
                .height(Size::fill())
                .padding(Gaps::new_all(inset))
                .child(self.sidebar.clone())
        };

        let mut root = rect()
            .expanded()
            .direction(Direction::Horizontal)
            .maybe(crate::backdrop::tinted().is_none(), |el| {
                el.background(color(surface_background(
                    &crate::theme::resolved(),
                    SurfaceKind::Toplevel,
                )))
            })
            .on_sized(move |event: Event<SizedEventData>| {
                window_width.set_if_modified(event.area.size.width);
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                if *dragging.read() {
                    resize_to(
                        event.global_location().x as f32 - 2.0 * inset + SPLIT_VIEW.grip / 2.0,
                    );
                }
            })
            .on_global_pointer_press(move |_| {
                if *dragging.peek() && !*grip_hovered.peek() {
                    Cursor::set(CursorIcon::default());
                }
                dragging.set_if_modified(false);
            })
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                if TOGGLE_SIDEBAR
                    .shortcut
                    .is_some_and(|shortcut| shortcut.matches(event.modifiers, &event.key))
                {
                    control.toggle();
                } else if event.key == Key::Named(NamedKey::Escape) {
                    control.dismiss();
                }
            });

        if let Some(extension) = self.extension.as_ref().filter(|_| !overlay && shown > 0.0) {
            let band = container * shown;
            let detail_width = (*window_width.read() - band).max(1.0);
            root = root.child(
                rect()
                    .position(Position::new_absolute().left(0.0).top(0.0))
                    .width(Size::px(band))
                    .height(Size::percent(100.0))
                    .overflow(Overflow::Clip)
                    .child(
                        rect()
                            .position(Position::new_absolute().left(band - 1.0).top(0.0))
                            .width(Size::px(1.0))
                            .height(Size::fill())
                            .overflow(Overflow::Clip)
                            .scale((band, 1.0))
                            .transform_origin(TransformOrigin::right())
                            .child(
                                rect()
                                    .width(Size::px(detail_width))
                                    .height(Size::fill())
                                    .child(extension.clone()),
                            ),
                    ),
            );
        }
        if !overlay && shown > 0.0 {
            root = root.child(
                rect()
                    .width(Size::px(container * shown))
                    .height(Size::fill())
                    .maybe(shown < 1.0, |el| el.overflow(Overflow::Clip))
                    .offset_x(container * (shown - 1.0))
                    .child(
                        sidebar().maybe_child((self.resizable && shown >= 1.0).then(|| {
                            let highlighted = *grip_hovered.read()
                                || *dragging.read()
                                || matches!(*grip_focus.read(), Focus::Keyboard);
                            let width = *sidebar_width.read();
                            rect()
                                .a11y_id(grip_id)
                                .a11y_focusable(true)
                                .a11y_role(AccessibilityRole::Splitter)
                                .position(
                                    Position::new_absolute()
                                        .left(container - inset - SPLIT_VIEW.grip)
                                        .top(0.0),
                                )
                                .width(Size::px(SPLIT_VIEW.grip))
                                .height(Size::percent(100.0))
                                .padding(Gaps::new(
                                    SPLIT_VIEW.grip_line_inset,
                                    0.0,
                                    SPLIT_VIEW.grip_line_inset,
                                    (SPLIT_VIEW.grip - SPLIT_VIEW.grip_line) / 2.0,
                                ))
                                .on_pointer_enter(move |_| {
                                    grip_hovered.set(true);
                                    Cursor::set(CursorIcon::ColResize);
                                })
                                .on_pointer_leave(move |_| {
                                    grip_hovered.set(false);
                                    if !*dragging.peek() {
                                        Cursor::set(CursorIcon::default());
                                    }
                                })
                                .on_mouse_down(move |_| dragging.set(true))
                                .on_key_down(move |event: Event<KeyboardEventData>| {
                                    match event.key {
                                        Key::Named(NamedKey::ArrowLeft) => {
                                            resize_to(width - SPLIT_VIEW.key_step)
                                        }
                                        Key::Named(NamedKey::ArrowRight) => {
                                            resize_to(width + SPLIT_VIEW.key_step)
                                        }
                                        _ => {}
                                    }
                                })
                                .maybe_child(highlighted.then(|| {
                                    rect()
                                        .width(Size::px(SPLIT_VIEW.grip_line))
                                        .height(Size::fill())
                                        .corner_radius(SPLIT_VIEW.grip_line / 2.0)
                                        .background(color(resolved.accent))
                                }))
                        })),
                    ),
            );
        }
        root = root.child(
            rect()
                .width(Size::flex(1.0))
                .height(Size::fill())
                .child(self.detail.clone()),
        );
        if let Some((inspector, _)) = self.inspector.as_ref().filter(|_| inspector_shown > 0.0) {
            root = root
                .child(
                    rect()
                        .width(Size::px(ct_design::HAIRLINE))
                        .height(Size::fill())
                        .background(color(palette.separator)),
                )
                .child(
                    rect()
                        .width(Size::px(SPLIT_VIEW.inspector * inspector_shown))
                        .height(Size::fill())
                        .overflow(Overflow::Clip)
                        .child(
                            rect()
                                .width(Size::px(SPLIT_VIEW.inspector))
                                .height(Size::fill())
                                .child(inspector.clone()),
                        ),
                );
        }
        if overlay && shown > 0.0 {
            root = root
                .child(
                    rect()
                        .position(Position::new_absolute().left(0.0).top(0.0))
                        .width(Size::fill())
                        .height(Size::percent(100.0))
                        .layer(Layer::Overlay)
                        .opacity(shown)
                        .background(color(palette.scrim))
                        .on_press(move |_| control.dismiss()),
                )
                .child(
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
