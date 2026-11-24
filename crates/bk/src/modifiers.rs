use ctdesign::{ControlSize, Rgba};
use freya_core::prelude::*;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::app::Shortcut;
use crate::document::OpenedUrl;
use crate::environment::{DisabledScope, EnvironmentScope};
use crate::file_dialog::{ContentType, DefaultDirectory, FileDialog};
use crate::focus::{DefaultFocus, FocusState, Focused};
use crate::glass_container::{Glass, GlassContainer};
use crate::layout::{Frame, LayoutDirectionScope};
use crate::menu::MenuItem;
use crate::shapes::UNBOUNDED;
use crate::styles::{
    ControlState, ControlStyle, GlassContainerStyle, LayoutDirection, Painted, Paints, ShadowStyle,
    Shape, ShapeStyle, StateValue, StyleScope, Theme,
};
use crate::transition::{ContentTransition, ContentTransitioned};
use crate::views::feedback::Alert;

pub trait ViewModifiers: EventHandlersExt + Sized {
    fn context_menu(
        self,
        items: Vec<MenuItem>,
        on_activate: impl Into<Callback<String, ()>>,
    ) -> Self {
        crate::context_menu::context_menu(self, items, on_activate)
    }

    fn sheet(self, is_presented: State<bool>, content: impl Fn() -> Element) -> Self
    where
        Self: ChildrenExt,
    {
        let presented = *is_presented.read();
        self.maybe_child(presented.then(|| crate::views::feedback::Sheet::new(content())))
    }

    fn alert(self, is_presented: State<bool>, content: impl Fn() -> Alert) -> Self
    where
        Self: ChildrenExt,
    {
        let presented = *is_presented.read();
        self.maybe_child(presented.then(|| content().dismissing(is_presented)))
    }

    fn popover(
        self,
        is_presented: State<bool>,
        size: (f32, f32),
        content: impl Fn() -> Element + 'static,
    ) -> Self
    where
        Self: LayoutExt,
    {
        crate::popover::use_popover(self, is_presented, size, content)
    }

    fn on_hover(self, perform: impl Into<Callback<bool, ()>>) -> Self {
        let perform = perform.into();
        let leave = perform.clone();
        self.on_pointer_enter(move |_| perform.call(true))
            .on_pointer_leave(move |_| leave.call(false))
    }

    fn on_tap_gesture(self, count: u32, perform: impl Into<Callback<(), ()>>) -> Self {
        let perform = perform.into();
        self.gesture(
            crate::gesture::TapGesture::new()
                .count(count)
                .on_ended(move |_| perform.call(())),
        )
    }

    fn gesture(self, gesture: impl crate::gesture::Attachable) -> Self {
        crate::gesture::attach(self, gesture, crate::gesture::Priority::Normal)
    }

    fn simultaneous_gesture(self, gesture: impl crate::gesture::Attachable) -> Self {
        crate::gesture::attach(self, gesture, crate::gesture::Priority::Simultaneous)
    }

    fn high_priority_gesture(self, gesture: impl crate::gesture::Attachable) -> Self {
        crate::gesture::attach(self, gesture, crate::gesture::Priority::High)
    }

    fn help(self, text: impl Into<Cow<'static, str>>) -> Self {
        crate::tooltip::use_tooltip(text).attach(self)
    }

    fn focus_ring(self, corner_radius: f32, color: Rgba) -> Self
    where
        Self: DecorationExt,
    {
        crate::interaction::ring(self, corner_radius, color)
    }

    fn foreground_style(self, style: ShapeStyle) -> Self
    where
        Self: TextStyleExt,
    {
        let paints = Paints::current(ControlState::default());
        self.color(paints.fill(&StateValue::new(style)))
    }

    fn background_style(self, style: ShapeStyle) -> Self
    where
        Self: BackgroundExt + DecorationExt + EffectExt,
    {
        self.painted(
            &Paints::current(ControlState::default()),
            &StateValue::new(style),
        )
    }

    fn shadows(self, shadows: Vec<ShadowStyle>) -> Self
    where
        Self: DecorationExt,
    {
        shadows
            .into_iter()
            .fold(self, |view, shadow| view.shadow(Shadow::from(shadow)))
    }

    /// A sheet of the window that chooses files; `on_completion` runs with the paths only when the user chose.
    fn file_importer(
        self,
        is_presented: State<bool>,
        allowed_content_types: Vec<ContentType>,
        allows_multiple_selection: bool,
        on_completion: impl Into<Callback<Vec<PathBuf>, ()>>,
    ) -> Self {
        let dialog = FileDialog::open()
            .allowed_content_types(allowed_content_types)
            .allows_multiple_selection(allows_multiple_selection);
        crate::file_dialog::use_file_dialog(is_presented, dialog, on_completion.into());
        self
    }

    /// A sheet of the window that asks where to save; the app writes the file at the path it receives.
    fn file_exporter(
        self,
        is_presented: State<bool>,
        default_filename: impl Into<String>,
        allowed_content_types: Vec<ContentType>,
        on_completion: impl Into<Callback<PathBuf, ()>>,
    ) -> Self {
        let dialog =
            FileDialog::save(default_filename).allowed_content_types(allowed_content_types);
        let done = on_completion.into();
        let first = Callback::new(move |paths: Vec<PathBuf>| {
            if let Some(path) = paths.into_iter().next() {
                done.call(path);
            }
        });
        crate::file_dialog::use_file_dialog(is_presented, dialog, first);
        self
    }

    /// Runs for each file or address that Finder, `open` or a link hands the app.
    fn on_open_url(self, perform: impl Into<Callback<OpenedUrl, ()>>) -> Self {
        crate::document::use_open_url(perform.into());
        self
    }
}

impl<T: EventHandlersExt> ViewModifiers for T {}

pub trait StyleModifiers: IntoElement + Sized {
    fn control_style<S: ControlStyle>(self, restyle: impl Fn(S) -> S + 'static) -> StyleScope {
        StyleScope::new(Theme::tengen().style(restyle), self)
    }

    fn control_size(self, size: ControlSize) -> StyleScope {
        StyleScope::new(Theme::control_size(size), self)
    }

    fn frame(self) -> Frame {
        Frame::new(self)
    }

    /// Glass behind the view in `shape`; an uneven shape takes its largest corner as one radius. Reduce Transparency makes a regular glass the toolbar's plain fill, as SwiftUI's is.
    fn glass_effect(self, glass: Glass, shape: Shape) -> impl IntoElement {
        let corners = shape.corner_radius(UNBOUNDED, UNBOUNDED, LayoutDirection::LeftToRight);
        let radius = corners
            .top_left()
            .max(corners.top_right())
            .max(corners.bottom_right())
            .max(corners.bottom_left());
        GlassContainer::new(0.0)
            .style(move |style| GlassContainerStyle {
                clear: glass.clear,
                solid: if glass.clear {
                    style.solid
                } else {
                    StateValue::new(ShapeStyle::Color(crate::theme::resolved().toolgroup.bg))
                },
                ..style
            })
            .shaped_item("glass", radius, matches!(shape, Shape::Circle), self)
    }

    /// Crossfades the view when it changes; it runs inside `with_animation`, as in SwiftUI.
    fn content_transition(self, transition: ContentTransition) -> impl IntoElement {
        ContentTransitioned::new(self.into_element(), transition)
    }
}

impl<T: IntoElement> StyleModifiers for T {}

pub trait EnvironmentModifiers: IntoElement + Sized {
    fn disabled(self, disabled: bool) -> impl IntoElement {
        DisabledScope::new(disabled, self)
    }

    fn environment<T: Clone + PartialEq + 'static>(self, value: T) -> impl IntoElement {
        EnvironmentScope::new(value, self)
    }

    fn layout_direction(self, direction: LayoutDirection) -> impl IntoElement {
        LayoutDirectionScope::new(direction, self)
    }

    fn keyboard_shortcut(self, shortcut: Shortcut) -> impl IntoElement {
        EnvironmentScope::new(Some(shortcut), self)
    }

    fn file_dialog_default_directory(self, directory: impl Into<PathBuf>) -> impl IntoElement {
        EnvironmentScope::new(DefaultDirectory(Some(directory.into())), self)
    }

    fn focused<T: Clone + PartialEq + 'static>(
        self,
        state: FocusState<T>,
        value: T,
    ) -> impl IntoElement {
        Focused::new(state, value, self)
    }

    fn default_focus<T: Clone + PartialEq + 'static>(
        self,
        state: FocusState<T>,
        value: T,
    ) -> impl IntoElement {
        DefaultFocus::new(state, value, self)
    }
}

impl<T: IntoElement> EnvironmentModifiers for T {}
