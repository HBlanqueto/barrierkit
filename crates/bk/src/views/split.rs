use ctdesign::{Font, LIST_GROUP, LIST_SECTION, SPLIT_VIEW, SUBHEADLINE, TOOLGROUP};
use freya_core::prelude::*;
use serde::{Deserialize, Serialize};
use torin::prelude::{Area, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::animation::with_animation;
use crate::app::{Action, Shortcut, ShortcutKey, use_menu_action};
use crate::compact::IconButton;
use crate::controls::ToolGroup;
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::i18n::text;
use crate::platform::WindowControls;
use crate::platform::backdrop::Declared;
use crate::sidebar::{SidebarControl, sidebar_control};
use crate::styles::Painted;
use crate::styles::{
    Animation, ControlState, FormStyle, LayoutDirection, Paints, Role, ShapeStyle, SplitViewStyle,
    StateValue, StyleScope, Theme, TitlebarStyle, appearance_index, resolve,
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
        let (offset, width) =
            sidebar_toggle_placement(control.map_or(0.0, |control| control.occupied()));
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

// AppKit gives the first responder at launch to the first list outside the sidebar: in SwiftUI's split view, the content column's.
#[derive(Clone, Copy)]
pub(crate) struct InitialFocus;

#[derive(PartialEq)]
struct ContentColumn(Element);

impl Component for ContentColumn {
    fn render(&self) -> impl IntoElement {
        use_provide_context(|| InitialFocus);
        self.0.clone()
    }
}

// How far the bar runs past the detail: left over the content column and right over the inspector, as SwiftUI's toolbar.
#[derive(Clone, Copy)]
pub(crate) struct BarOverhang {
    pub(crate) leading: State<f32>,
    pub(crate) trailing: State<f32>,
    // Whether the bar draws its veil and separator: the inspector's glass then starts below the bar, which runs over it as over the detail.
    pub(crate) band: State<bool>,
}

#[derive(PartialEq)]
pub struct SplitView {
    sidebar: Element,
    content: Option<Element>,
    detail: Element,
    inspector: Option<(Element, bool)>,
    extension: Option<Element>,
    sidebar_visible: bool,
    sidebar_present: bool,
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
            sidebar_present: true,
            sidebar_width: None,
            sidebar_range: (SPLIT_VIEW.sidebar_min, f32::INFINITY),
            resizable: true,
            narrow_sidebar: NarrowSidebar::Push,
            on_layout: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    /// A column between the sidebar and the detail; it resizes within `sidebar_range` and stays when the sidebar hides.
    pub(crate) fn without_sidebar(detail: impl IntoElement) -> Self {
        Self {
            sidebar_visible: false,
            sidebar_present: false,
            ..Self::new(rect(), detail)
        }
    }

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
        let mut origin_x = use_state(|| 0.0f32);
        let inspector_theme = use_hook(inspector_form_theme);
        let mut presented = use_state(|| self.inspector.as_ref().is_some_and(|(_, shown)| *shown));
        let wanted = self.inspector.as_ref().is_some_and(|(_, shown)| *shown);
        if *presented.peek() != wanted {
            presented.set(wanted);
        }
        let inspector_shown =
            crate::animation::use_animated(f32::from(u8::from(*presented.read())), style.animation);
        let native_panes = crate::glass_panel::native_pane_glass();
        let mut inspector_span = use_state(|| 0.0f32);
        inspector_span.set_if_modified(
            self.inspector
                .as_ref()
                .map_or(0.0, |_| style.inspector_width * inspector_shown),
        );
        let mut content_span = use_state(|| 0.0f32);
        let bar_band = use_state(|| false);
        use_hook(|| {
            provide_context(BarOverhang {
                leading: content_span,
                trailing: inspector_span,
                band: bar_band,
            })
        });
        let bar_height = try_consume_root_context::<WindowControls>()
            .map_or(0.0, |controls| *controls.titlebar_height.read());
        let mut inspector_area = use_state(Area::default);
        let mut detail_area = use_state(Area::default);
        crate::glass_panel::use_publish_inspector(
            (native_panes && self.inspector.is_some() && inspector_shown > 0.0).then(|| {
                let mut area = *inspector_area.read();
                let below = if *bar_band.read() { bar_height } else { 0.0 };
                area.origin.y += below;
                area.size.height -= below;
                area
            }),
        );
        let (min_width, max_width) = self.sidebar_range;
        let initial_width = self
            .sidebar_width
            .unwrap_or(SPLIT_VIEW.sidebar_width)
            .clamp(min_width, max_width);
        let mut sidebar_width = use_state(|| initial_width);
        let mut dragging = use_state(|| false);
        let mut content_width = use_state(|| SPLIT_VIEW.content_width.clamp(min_width, max_width));
        let mut content_dragging = use_state(|| false);
        let mut visible = use_state(|| self.sidebar_visible);
        let mut toggle_animation = use_state(|| style.animation);
        toggle_animation.set(style.animation);
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
            animation: toggle_animation,
            present: self.sidebar_present,
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
            with_animation(style.animation, || {
                if overlay && *visible.peek() {
                    visible.set(false);
                    auto_hidden.set(true);
                } else if !overlay {
                    visible.set(true);
                    auto_hidden.set(false);
                }
            });
        }

        // A change of the sidebar animates only inside `with_animation`, as in SwiftUI.
        let shown =
            crate::animation::use_animated(f32::from(u8::from(*visible.read())), Animation::None);

        let mut resize_to =
            move |target: f32| sidebar_width.set(target.clamp(min_width, max_width));
        let container = sidebar_width.read().clamp(min_width, max_width);
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
        content_span.set_if_modified(if self.content.is_some() {
            content_column + style.inspector_line.1
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
                origin_x.set_if_modified(event.area.min_x());
            })
            .on_global_pointer_move(move |event: Event<PointerEventData>| {
                let edge =
                    event.global_location().x as f32 - *origin_x.peek() + style.grip_width / 2.0;
                if *dragging.read() {
                    // AppKit holds the minimum, then collapses in one frame near half of it.
                    visible.set_if_modified(edge >= min_width / 2.0);
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
                                double_click_width: Some(min_width),
                                style: self.style.clone(),
                            }
                        })),
                    )
            }))
            .maybe_child(self.content.as_ref().map(|content| {
                rect()
                    .width(Size::px(content_column))
                    .height(Size::fill())
                    .padding(Gaps::new(bar_height, 0.0, 0.0, 0.0))
                    .child(ContentColumn(content.clone()))
                    .maybe_child(self.resizable.then(|| ColumnGrip {
                        edge: content_column,
                        width: content_column,
                        dragging: content_dragging,
                        resize: Callback::new(move |width: f32| {
                            content_width.set(width.clamp(min_width, max_width))
                        }),
                        double_click_width: None,
                        style: self.style.clone(),
                    }))
            }))
            .maybe_child(self.content.is_some().then(line))
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .key("detail")
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
                                    .padding(Gaps::new(bar_height, 0.0, 0.0, 0.0))
                                    .child(StyleScope::new(
                                        inspector_theme.clone(),
                                        inspector.clone(),
                                    )),
                            )
                            .maybe_child(native_panes.then(|| {
                                // SwiftUI's divider starts under the bar's veil, as the bar runs over the inspector.
                                let top = if *bar_band.read() { bar_height } else { 0.0 };
                                line()
                                    .height(Size::percent(100.0))
                                    .position(Position::new_absolute().left(0.0).top(top))
                            }))
                            .maybe_child(
                                (native_panes && *bar_band.read())
                                    .then(|| inspector_bar_line(bar_height))
                                    .flatten(),
                            )
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
    double_click_width: Option<f32>,
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
        let (resize, width, double_click_width) =
            (self.resize.clone(), self.width, self.double_click_width);
        let reset = self.resize.clone();
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
            .on_mouse_down(move |event: Event<MouseEventData>| {
                let double = EventsCombos::<()>::pressed(event.global_location).is_double();
                match double_click_width.filter(|_| double) {
                    Some(width) => reset.call(width),
                    None => dragging.set(true),
                }
            })
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

fn inspector_bar_line(height: f32) -> Option<Rect> {
    let paints = Paints::current(ControlState::default());
    let (ink, thickness) = crate::theme::window_material_here().then(TitlebarStyle::line)?;
    Some(
        rect()
            .position(Position::new_absolute().top(height).left(0.0))
            .width(Size::fill())
            .height(Size::px(thickness))
            .background(paints.fill(&StateValue::new(ink))),
    )
}

// A form in SwiftUI's inspector keeps its inset to the column, and over the column's material its cards and titles are the sidebar's.
fn inspector_form_theme() -> Theme {
    Theme::tengen().style(|style: FormStyle| FormStyle {
        padding: [SPLIT_VIEW.inspector_form_padding; 4],
        header_padding: [
            style.header_padding[0],
            style.header_padding[1],
            SPLIT_VIEW.inspector_header_gap,
            style.header_padding[3],
        ],
        header: StateValue::new(ShapeStyle::Role(Role::SecondaryLabel)),
        header_font: Font {
            weight: LIST_SECTION.header_weight,
            ..SUBHEADLINE
        },
        background: StateValue::new(ShapeStyle::Color(
            LIST_GROUP.inspector_card[appearance_index()],
        )),
        title: StateValue::new(ShapeStyle::Color(
            crate::sidebar::item_palette(true).item_ink,
        )),
        ..style
    })
}
