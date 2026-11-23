use freya_core::{integration::AppComponent, prelude::State};
use std::borrow::Cow;
use std::{cell::RefCell, rc::Rc};

use super::{ObjectId, RunError, calloop::ping::Ping, window_style::Bar};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FullscreenBar {
    #[default]
    Solid,
    Glass,
    ProgressiveGlass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecorationResponse {
    Client,
    Server,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowStyle {
    #[default]
    TitleBar,
    HiddenTitleBar,
    Plain,
}

impl WindowStyle {
    pub(crate) fn inner_bar_height(self, system_title_bar: bool) -> u32 {
        match self {
            Self::TitleBar if system_title_bar => 0,
            Self::TitleBar => ctdesign::TITLEBAR_CLIENT_HEIGHT,
            Self::HiddenTitleBar => ctdesign::SETTINGS.toolbar_height,
            Self::Plain => 0,
        }
    }
}

// The limits count the content below the bar drawn inside the window, as SwiftUI's `frame(minHeight:)` sits under the toolbar (ADR 0117).
pub(crate) fn below_bar(limit: Option<(u32, u32)>, bar: u32) -> Option<(u32, u32)> {
    limit.map(|(width, height)| (width, if height == 0 { 0 } else { height + bar }))
}

#[derive(Clone, Debug)]
pub struct WindowOptions {
    pub title: Cow<'static, str>,
    pub app_name: String,
    pub app_id: Option<String>,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub menu: Option<crate::main_menu::MainMenu>,
    pub fullscreen_bar: FullscreenBar,
    pub fullscreen_push: bool,
    pub hud: bool,
    pub status_item: Option<crate::status_item::StatusItem>,
    pub panel: PanelBehavior,
    pub(crate) modal: bool,
    pub theme: crate::styles::Theme,
    pub scene: Option<&'static str>,
    pub window_style: WindowStyle,
    pub opaque: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelBehavior {
    pub hides_on_deactivate: bool,
    pub floating: bool,
    pub key_only_if_needed: bool,
}

impl PanelBehavior {
    pub const UTILITY: Self = Self {
        hides_on_deactivate: true,
        floating: true,
        key_only_if_needed: true,
    };

    pub(super) fn flags(self) -> super::window_style::Panel {
        use super::window_style::Panel;
        [
            (self.hides_on_deactivate, Panel::HidesOnDeactivate),
            (self.floating, Panel::Floating),
            (self.key_only_if_needed, Panel::KeyOnlyIfNeeded),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .fold(Panel::empty(), |flags, (_, flag)| flags | flag)
    }
}

impl WindowOptions {
    pub fn new(title: impl Into<Cow<'static, str>>, app_name: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_name: app_name.into(),
            app_id: None,
            size: ctdesign::WINDOW_DEFAULT_SIZE,
            min_size: None,
            max_size: None,
            menu: None,
            fullscreen_bar: FullscreenBar::Solid,
            fullscreen_push: false,
            hud: false,
            status_item: None,
            panel: PanelBehavior::UTILITY,
            modal: false,
            theme: crate::styles::Theme::tengen(),
            scene: None,
            window_style: WindowStyle::TitleBar,
            opaque: true,
        }
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    pub fn theme(mut self, theme: crate::styles::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn window_style(mut self, window_style: WindowStyle) -> Self {
        self.window_style = window_style;
        self
    }

    pub fn opaque(mut self, opaque: bool) -> Self {
        self.opaque = opaque;
        self
    }

    pub fn size(mut self, size: (u32, u32)) -> Self {
        self.size = size;
        self
    }

    pub fn min_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.min_size = size;
        self
    }

    pub fn max_size(mut self, size: Option<(u32, u32)>) -> Self {
        self.max_size = size;
        self
    }

    pub fn menu(mut self, menu: crate::main_menu::MainMenu) -> Self {
        self.menu = Some(menu);
        self
    }

    pub fn fullscreen_bar(mut self, bar: FullscreenBar) -> Self {
        self.fullscreen_bar = bar;
        self
    }

    pub fn fullscreen_push(mut self, push: bool) -> Self {
        self.fullscreen_push = push;
        self
    }

    pub fn hud(mut self, hud: bool) -> Self {
        self.hud = hud;
        self
    }

    pub fn panel(mut self, panel: PanelBehavior) -> Self {
        self.panel = panel;
        self
    }

    pub fn status_item(mut self, item: crate::status_item::StatusItem) -> Self {
        self.status_item = Some(item);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStatus {
    pub activated: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub decoration: DecorationResponse,
    pub fixed: bool,
}

impl Default for WindowStatus {
    fn default() -> Self {
        Self {
            activated: true,
            maximized: false,
            fullscreen: false,
            decoration: DecorationResponse::Client,
            fixed: false,
        }
    }
}

#[derive(Clone)]
pub struct WindowControls {
    pub(super) queue: Rc<RefCell<Vec<WindowCommand>>>,
    pub(super) ping: Ping,
    pub(super) pointer_serial: Rc<std::cell::Cell<u32>>,
    pub status: State<WindowStatus>,
    pub title: Cow<'static, str>,
    pub fixed: bool,
    pub fixed_width: bool,
    pub(super) surface_id: ObjectId,
    pub focus_bar: State<bool>,
    pub focus_bar_progress: State<f32>,
    pub focus_overlay: bool,
    pub focus_bar_height: State<f32>,
    pub desktop: State<Option<u32>>,
    pub fullscreen_bar: State<FullscreenBar>,
    pub fullscreen_push: State<bool>,
    pub(crate) overlay_titlebar_height: State<f32>,
    // Where the sidebar spans the window, left and width: on macOS the full-screen bar leaves a soft edge over it.
    pub(crate) sidebar_span: State<(f32, f32)>,
    pub hud: bool,
    pub panel: bool,
    pub ime_preedit: State<String>,
    pub(crate) primary: super::clipboard::Clipboard,
    pub(crate) clipboard: super::clipboard::Clipboard,
}

pub(crate) fn window_activated() -> bool {
    freya_core::prelude::try_consume_root_context::<WindowControls>()
        .is_none_or(|controls| controls.status.read().activated)
}

impl WindowControls {
    pub fn traffic_lights_shown(&self) -> bool {
        let status = *self.status.read();
        match status.decoration {
            DecorationResponse::Server => false,
            // AppKit keeps a hybrid window's toolbar row in full screen and reveals its lights there.
            DecorationResponse::Hybrid => true,
            DecorationResponse::Client => {
                !status.fullscreen || *self.focus_bar_progress.read() > 0.0
            }
        }
    }

    pub(crate) fn draws_traffic_lights(&self) -> bool {
        self.status.read().decoration == DecorationResponse::Client
    }

    pub fn titlebar_overlaid(&self) -> bool {
        self.focus_overlay && self.status.read().fullscreen
    }

    pub fn fullscreen_content_top(&self) -> f32 {
        let below_bar = match *self.fullscreen_bar.read() {
            FullscreenBar::ProgressiveGlass => ctdesign::FULLSCREEN_GLASS.content_top,
            FullscreenBar::Glass => ctdesign::FULLSCREEN_BAND.content_top,
            FullscreenBar::Solid => ctdesign::FULLSCREEN_BAR.content_top,
        };
        if self.titlebar_overlaid() && !*self.fullscreen_push.read() {
            *self.focus_bar_height.read() + below_bar
        } else {
            0.0
        }
    }

    pub(crate) fn push_offset(&self) -> f32 {
        if *self.fullscreen_push.read() && self.titlebar_overlaid() {
            (*self.focus_bar_height.read() + *self.overlay_titlebar_height.read())
                * *self.focus_bar_progress.read()
        } else {
            0.0
        }
    }

    pub fn titlebar_overlay_top(&self) -> Option<f32> {
        let progress = *self.focus_bar_progress.read();
        let hidden_by = ctdesign::TITLEBAR_CLIENT_HEIGHT as f32 * (1.0 - progress);
        (self.titlebar_overlaid() && progress > 0.0)
            .then(|| *self.focus_bar_height.read() * progress - hidden_by)
    }

    pub fn get() -> Self {
        freya_core::prelude::consume_root_context()
    }

    pub(crate) fn id(&self) -> ObjectId {
        self.surface_id.clone()
    }

    fn send(&self, command: WindowCommand) {
        self.queue.borrow_mut().push(command);
        self.ping.ping();
    }

    pub fn present(&self, activation_token: String) {
        self.send(WindowCommand::Present(Some(activation_token)));
    }

    pub(crate) fn present_without_token(&self) {
        self.send(WindowCommand::Present(None));
    }

    pub fn make_key_and_order_front(&self) {
        self.send(WindowCommand::MakeKey);
    }

    pub fn close(&self) {
        self.send(WindowCommand::Close);
    }

    pub fn set_bar(&self, bar: Bar) {
        self.send(WindowCommand::Bar(bar));
    }

    pub fn set_title(&self, title: impl Into<Cow<'static, str>>) {
        self.send(WindowCommand::SetTitle(title.into()));
    }

    pub fn copy_rich(&self, text: String, html: String) {
        self.clipboard.set_rich(text, html);
    }

    pub fn primary_selection(&self) -> Box<dyn freya_clipboard::copypasta::ClipboardProvider> {
        self.primary.provider()
    }

    pub fn set_ime_cursor_area(&self, x: i32, y: i32, width: i32, height: i32) {
        self.send(WindowCommand::ImeCursor((x, y, width, height)));
    }

    pub(crate) fn open_sheet(&self, app: AppComponent, mut options: WindowOptions) {
        options.modal = true;
        self.send(WindowCommand::Sheet(Box::new((app, options))));
    }

    pub(crate) fn fit(&self, size: (u32, u32)) {
        self.send(WindowCommand::Fit(size));
    }

    pub fn open_window(&self, app: AppComponent, options: WindowOptions) {
        self.send(WindowCommand::Open(Box::new((app, options))));
    }

    pub fn set_background_alpha(&self, alpha: f32) {
        self.send(WindowCommand::BackgroundAlpha(alpha.clamp(0.0, 1.0)));
    }

    pub fn minimize(&self) {
        self.send(WindowCommand::Minimize);
    }

    pub fn toggle_maximize(&self) {
        self.send(WindowCommand::ToggleMaximize);
    }

    pub fn toggle_fullscreen(&self) {
        self.send(WindowCommand::ToggleFullscreen);
    }

    pub fn move_window(&self) {
        self.send(WindowCommand::Move(self.pointer_serial.get()));
    }

    pub fn resize(&self, edge: WindowEdge) {
        self.send(WindowCommand::Resize(self.pointer_serial.get(), edge));
    }
}

pub(crate) enum WindowCommand {
    Close,
    Minimize,
    ToggleMaximize,
    ToggleFullscreen,
    Move(u32),
    Resize(u32, WindowEdge),
    Present(Option<String>),
    MakeKey,
    Bar(Bar),
    SetTitle(Cow<'static, str>),
    BackgroundAlpha(f32),
    Open(Box<(AppComponent, WindowOptions)>),
    Sheet(Box<(AppComponent, WindowOptions)>),
    Fit((u32, u32)),
    ImeCursor((i32, i32, i32, i32)),
}

pub(crate) struct OpenUtility {
    pub(crate) app: AppComponent,
    pub(crate) options: WindowOptions,
}

pub(crate) fn open_utility_window(app: AppComponent, options: WindowOptions) {
    super::KitSender::get().send(OpenUtility { app, options });
}

pub fn run_window(app: impl Into<AppComponent>, options: WindowOptions) -> Result<(), RunError> {
    super::sys::run_window(app.into(), options)
}

#[cfg(test)]
mod tests {
    use super::below_bar;

    #[test]
    fn the_limits_add_the_bar_and_keep_an_unbounded_maximum() {
        assert_eq!(below_bar(Some((600, 360)), 52), Some((600, 412)));
        assert_eq!(below_bar(Some((600, 0)), 52), Some((600, 0)));
        assert_eq!(below_bar(None, 52), None);
    }
}
