use freya_animation::prelude::use_animation_transition;

use ctdesign::{SIDEBAR, SPLIT_VIEW, TOOLGROUP};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Area, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::animation::Clock;
use crate::app::{Action, Shortcut, ShortcutKey, use_menu_action};
use crate::compact::IconButton;
use crate::controls::ToolGroup;
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::i18n::text;
use crate::platform::backdrop::Declared;
use crate::sidebar::{SidebarControl, sidebar_control};
use crate::styles::Painted;
use crate::styles::{
    ControlState, LayoutDirection, Paints, SplitViewStyle, StateValue, Theme, resolve,
};
use crate::symbol::Symbol;
use crate::titlebar::{TITLEBAR_OVERLAY_LAYER, sidebar_toggle_placement};

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
    Overlay,
    #[default]
    Push,
}

#[derive(PartialEq)]
pub struct SidebarToggle;

impl Component for SidebarToggle {
    fn render(&self) -> impl IntoElement {
        let control = sidebar_control();
        let (offset, width) = sidebar_toggle_placement(
            control.map_or(0.0, |control| control.occupied()),
            control.map_or(0.0, |control| control.content()),
        );
        rect()
            .width(Size::px(width))
            .height(Size::px(TOOLGROUP.height as f32))
            .child(
                rect()
                    .position(Position::new_absolute().top(0.0).left(offset))
                    .maybe(offset < 0.0, |el| el.layer(TITLEBAR_OVERLAY_LAYER + 1))
                    .child(
                        ToolGroup::new().probe("sidebar-toggle").child(
                            IconButton::new(Symbol::SidebarLeft, TOGGLE_SIDEBAR.title())
                                .disabled(control.is_none())
                                .on_press(move |_| {
                                    if let Some(control) = control {
                                        control.toggle();
                                    }
                                }),
                        ),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct SplitView {
    sidebar: Element,
    content: Option<Element>,
    detail: Element,
    inspector: Option<(Element, bool)>,
    extension: Option<Element>,
    sidebar_visible: bool,
    sidebar_width: Option<f32>,
    sidebar_range: (f32, f32),
    resizable: bool,
    narrow_sidebar: NarrowSidebar,
    on_layout: Option<Callback<SidebarLayout, ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SplitView {
    pub fn new(sidebar: impl IntoElement, detail: impl IntoElement) -> Self {
        Self {
            sidebar: sidebar.into_element(),
            content: None,
            detail: detail.into_element(),
            inspector: None,
            extension: None,
            sidebar_visible: true,
            sidebar_width: None,
            sidebar_range: (SPLIT_VIEW.sidebar_min, SPLIT_VIEW.sidebar_max),
            resizable: true,
            narrow_sidebar: NarrowSidebar::Push,
            on_layout: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    /// A column between the sidebar and the detail; it resizes within `sidebar_range` and stays when the sidebar hides.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_element());
        self
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

    pub fn style(mut self, restyle: impl Fn(SplitViewStyle) -> SplitViewStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

forward_key!(SplitView);

forward_overrides!(SplitView);

impl Component for SplitView {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: SplitViewStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default());
        let mut window_width = use_state(|| f32::MAX);
        let mut presented = use_state(|| self.inspector.as_ref().is_some_and(|(_, shown)| *shown));
        let wanted = self.inspector.as_ref().is_some_and(|(_, shown)| *shown);
        if *presented.peek() != wanted {
            presented.set(wanted);
        }
        let inspector_shown =
            crate::animation::use_animated(f32::from(u8::from(*presented.read())), style.animation);
        let native_panes = crate::glass_panel::native_pane_glass();
        let mut inspector_area = use_state(Area::default);
        let mut detail_area = use_state(Area::default);
        crate::glass_panel::use_publish_inspector(
            (native_panes && self.inspector.is_some() && inspector_shown > 0.0)
                .then(|| *inspector_area.read()),
        );
        let (min_width, max_width) = self.sidebar_range;
        let initial_width = self
            .sidebar_width
            .unwrap_or(SIDEBAR.width as f32)
            .clamp(min_width, max_width);
        let mut sidebar_width = use_state(|| initial_width);
        let mut dragging = use_state(|| false);
        let mut content_width = use_state(|| initial_width);
        let mut content_dragging = use_state(|| false);
        let mut visible = use_state(|| self.sidebar_visible);
        let mut auto_hidden = use_state(|| false);
        let mut occupied = use_state(|| 0.0f32);
        let mut content_shown = use_state(|| 0.0f32);
        let mut settled = use_state(|| false);
        let mut overlaid = use_state(|| false);
        let control = SidebarControl {
            visible,
            auto_hidden,
            occupied,
            content: content_shown,
            settled,
            overlaid,
        };
        use_hook(|| provide_context(control));
        use_menu_action(move |id| {
            if id == TOGGLE_SIDEBAR.id {
                control.toggle();
            }
        });

        let overlay = self.narrow_sidebar == NarrowSidebar::Overlay
            && *window_width.read() < SPLIT_VIEW.collapse_below;
        let mut was_narrow = use_state(|| false);
        if *was_narrow.peek() != overlay {
            was_narrow.set(overlay);
            if overlay && *visible.peek() {
                visible.set(false);
                auto_hidden.set(true);
            } else if !overlay {
                visible.set(true);
                auto_hidden.set(false);
            }
        }

        let shown =
            crate::animation::use_animated(f32::from(u8::from(*visible.read())), style.animation);

        let resize = style.resize_animation;
        let resize_ms = use_reactive(&resize.duration_ms());
        let mut follow_from_to = use_state(|| (initial_width, initial_width));
        let follow = use_animation_transition(follow_from_to, move |_: (f32, f32), _| {
            Clock::new(0.0, 1.0).time(*resize_ms.peek())
        });
        let followed = move || {
            let (from, to) = *follow_from_to.peek();
            if resize.duration_ms() == 0 {
                to
            } else {
                from + (to - from) * resize.progress(follow.get().value())
            }
        };
        let mut resize_to = move |target: f32| {
            let target = target.clamp(min_width, max_width);
            let from = if *dragging.peek() { target } else { followed() };
            follow_from_to.set((from, target));
            sidebar_width.set(target);
        };
        let container = if *dragging.read() {
            *sidebar_width.read()
        } else {
            followed().clamp(min_width, max_width)
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

        let content_column = content_width.read().clamp(min_width, max_width);
        content_shown.set_if_modified(if self.content.is_some() {
            content_column
        } else {
            0.0
        });
        occupied.set_if_modified(if overlay { 0.0 } else { container * shown });
        overlaid.set_if_modified(overlay && *visible.read());
        settled.set_if_modified(*visible.read() && shown >= 1.0);
        let sidebar = || {
            rect()
                .width(Size::px(container))
                .height(Size::fill())
                .child(self.sidebar.clone())
        };

        let lead = if overlay { 0.0 } else { container * shown };
        let trail = if native_panes && self.inspector.is_some() {
            style.inspector_width * inspector_shown
        } else {
            0.0
        };
        let line = || {
            rect()
                .width(Size::px(style.inspector_line.1))
                .height(Size::fill())
                .background(paints.fill(&StateValue::new(style.inspector_line.0.clone())))
        };
        let root = rect()
            .expanded()
            .direction(Direction::Horizontal)
            .cursor((*dragging.read() || *content_dragging.read()).then_some(CursorIcon::ColResize))
            .painted(&paints, &style.background)
            .on_sized(move |event: Event<SizedEventData>| {
                window_width.set_if_modified(event.area.size.width);
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let edge = event.element_location().x as f32 + style.grip_width / 2.0;
                if *dragging.read() {
                    resize_to(edge);
                }
                if *content_dragging.read() {
                    content_width.set((edge - *occupied.peek()).clamp(min_width, max_width));
                }
            })
            .on_global_pointer_up(move |_| {
                dragging.set_if_modified(false);
                content_dragging.set_if_modified(false);
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
            })
            .maybe_child(
                self.extension
                    .as_ref()
                    .filter(|_| lead > 0.0 || trail > 0.0)
                    .map(|extension| {
                        let detail = *detail_area.read();
                        let detail_width = detail.width().max(1.0);
                        let laid_out = |left: f32| {
                            rect()
                                .position(Position::new_absolute().left(left).top(0.0))
                                .width(Size::px(detail_width))
                                .height(Size::fill())
                                .child(extension.clone())
                        };
                        let stretched =
                            |left: f32, width: f32, column: f32, origin: TransformOrigin| {
                                rect()
                                    .position(Position::new_absolute().left(left).top(0.0))
                                    .width(Size::px(1.0))
                                    .height(Size::fill())
                                    .overflow(Overflow::Clip)
                                    .scale((width, 1.0))
                                    .transform_origin(origin)
                                    .child(laid_out(-column))
                            };
                        rect()
                            .position(Position::new_absolute().left(0.0).top(0.0))
                            .width(if native_panes {
                                Size::fill()
                            } else {
                                Size::px(lead)
                            })
                            .height(Size::percent(100.0))
                            .overflow(Overflow::Clip)
                            .maybe_child((lead > 0.0).then(|| {
                                stretched(lead - 1.0, lead, 0.0, TransformOrigin::right())
                            }))
                            .maybe_child((trail > 0.0).then(|| {
                                stretched(
                                    detail.max_x(),
                                    trail,
                                    detail_width - 1.0,
                                    TransformOrigin::left(),
                                )
                            }))
                    }),
            )
            .maybe_child((!overlay && shown > 0.0).then(|| {
                rect()
                    .width(Size::px(container * shown))
                    .height(Size::fill())
                    .maybe(shown < 1.0, |el| el.overflow(Overflow::Clip))
                    .offset_x(container * (shown - 1.0))
                    .child(
                        sidebar().maybe_child((self.resizable && shown >= 1.0).then(|| {
                            ColumnGrip {
                                edge: container,
                                width: *sidebar_width.read(),
                                dragging,
                                resize: Callback::new(resize_to),
                                style: self.style.clone(),
                            }
                        })),
                    )
            }))
            .maybe_child(self.content.as_ref().map(|content| {
                rect()
                    .width(Size::px(content_column))
                    .height(Size::fill())
                    .child(content.clone())
                    .maybe_child(self.resizable.then(|| ColumnGrip {
                        edge: content_column,
                        width: content_column,
                        dragging: content_dragging,
                        resize: Callback::new(move |width: f32| {
                            content_width.set(width.clamp(min_width, max_width))
                        }),
                        style: self.style.clone(),
                    }))
            }))
            .maybe_child(self.content.is_some().then(line))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .on_sized(move |event: Event<SizedEventData>| {
                        detail_area.set_if_modified(event.area)
                    })
                    .child(self.detail.clone()),
            )
            .maybe_child(
                (!native_panes && self.inspector.is_some() && inspector_shown > 0.0).then(line),
            )
            .maybe_child(
                self.inspector
                    .as_ref()
                    .filter(|_| inspector_shown > 0.0)
                    .map(|(inspector, _)| {
                        rect()
                            .width(Size::px(style.inspector_width * inspector_shown))
                            .height(Size::fill())
                            .overflow(Overflow::Clip)
                            .on_sized(move |event: Event<SizedEventData>| {
                                inspector_area.set_if_modified(event.area)
                            })
                            .maybe_child(native_panes.then(Declared::pane))
                            .child(
                                rect()
                                    .width(Size::px(style.inspector_width))
                                    .height(Size::fill())
                                    .child(inspector.clone()),
                            )
                            .maybe_child(native_panes.then(|| {
                                line()
                                    .height(Size::percent(100.0))
                                    .position(Position::new_absolute().left(0.0).top(0.0))
                            }))
                    }),
            )
            .maybe_child((overlay && shown > 0.0).then(|| {
                rect()
                    .position(Position::new_absolute().left(0.0).top(0.0))
                    .width(Size::fill())
                    .height(Size::percent(100.0))
                    .layer(Layer::Overlay)
                    .opacity(shown)
                    .background(paints.fill(&style.scrim))
                    .on_press(move |_| control.dismiss())
            }))
            .maybe_child((overlay && shown > 0.0).then(|| {
                sidebar()
                    .position(
                        Position::new_absolute()
                            .left(-container * (1.0 - shown))
                            .top(0.0),
                    )
                    .layer(Layer::Overlay)
            }));
        self.overrides.apply(root.content(Content::flex()))
    }
}

#[derive(PartialEq)]
struct ColumnGrip {
    edge: f32,
    width: f32,
    dragging: State<bool>,
    resize: Callback<f32, ()>,
    style: Theme,
}

impl Component for ColumnGrip {
    fn render(&self) -> impl IntoElement {
        let style: SplitViewStyle = resolve(&self.style);
        let mut dragging = self.dragging;
        let mut hovered = use_state(|| false);
        let id = use_a11y();
        let focus = use_focus(id);
        let state = ControlState {
            hovered: *hovered.read(),
            pressed: *dragging.read(),
            focused: matches!(*focus.read(), Focus::Keyboard),
            disabled: false,
        };
        let paints = Paints::current(state);
        let line = paints.color(style.grip.resolve(state));
        let (resize, width) = (self.resize.clone(), self.width);
        rect()
            .a11y_id(id)
            .a11y_focusable(true)
            .a11y_role(AccessibilityRole::Splitter)
            .layer(crate::scroll::THUMB_LAYER + 1)
            .position(
                Position::new_absolute()
                    .left(self.edge - style.grip_width)
                    .top(0.0),
            )
            .width(Size::px(style.grip_width))
            .height(Size::percent(100.0))
            .padding(Gaps::new(
                style.grip_line_inset,
                0.0,
                style.grip_line_inset,
                (style.grip_width - style.grip_line_width) / 2.0,
            ))
            .cursor(CursorIcon::ColResize)
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .on_mouse_down(move |_| dragging.set(true))
            .on_key_down(move |event: Event<KeyboardEventData>| match event.key {
                Key::Named(NamedKey::ArrowLeft) => resize.call(width - SPLIT_VIEW.key_step),
                Key::Named(NamedKey::ArrowRight) => resize.call(width + SPLIT_VIEW.key_step),
                _ => {}
            })
            .maybe_child((line.a > 0.0).then(|| {
                rect()
                    .width(Size::px(style.grip_line_width))
                    .height(Size::fill())
                    .corner_radius(style.grip_shape.corner_radius(
                        style.grip_line_width,
                        style.grip_line_width,
                        LayoutDirection::LeftToRight,
                    ))
                    .background(paints.fill(&style.grip))
            }))
    }
}
