use freya_core::{integration::AppComponent, prelude::State};
use std::borrow::Cow;
use std::{cell::RefCell, rc::Rc};

use crate::platform::{
    ObjectId, RunError, calloop::ping::Ping, clipboard::Clipboard, window_style::Bar,
};

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

/// SwiftUI's `WindowToolbarFullScreenVisibility`: whether the bar stays pinned in full screen or comes down with the menu bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowToolbarFullScreenVisibility {
    #[default]
    Automatic,
    OnHover,
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

pub(crate) fn app_name(app_id: &str) -> &str {
    app_id.rsplit_once('.').map_or(app_id, |(_, name)| name)
}

// The limits count the content below the bar drawn inside the window, as SwiftUI's `frame(minHeight:)` sits under the toolbar (docs/barrierkit/08-windows.md#window-options).
pub(crate) fn below_bar(limit: Option<(u32, u32)>, bar: u32) -> Option<(u32, u32)> {
    limit.map(|(width, height)| (width, if height == 0 { 0 } else { height + bar }))
}

#[derive(Clone, Debug)]
pub struct WindowOptions {
    pub title: Cow<'static, str>,
    pub app_name: String,
    pub app_id: String,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub menu: Option<crate::main_menu::MainMenu>,
    pub hud: bool,
    pub status_item: Option<crate::status_item::StatusItem>,
    pub panel: PanelBehavior,
    pub(crate) modal: bool,
    pub theme: crate::styles::Theme,
    pub scene: Option<&'static str>,
    pub window_style: WindowStyle,
    pub window_toolbar_full_screen_visibility: WindowToolbarFullScreenVisibility,
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
}

impl WindowOptions {
    pub(super) fn fixed(&self) -> bool {
        self.min_size.is_some() && self.min_size == self.max_size
    }

    /// `app_id` is reverse DNS (`com.example.notes`); its last part is the name the menus show.
    pub fn new(title: impl Into<Cow<'static, str>>, app_id: impl Into<String>) -> Self {
        let app_id = app_id.into();
        Self {
            title: title.into(),
            app_name: app_name(&app_id).to_owned(),
            app_id,
            size: ctdesign::WINDOW_DEFAULT_SIZE,
            min_size: None,
            max_size: None,
            menu: None,
            hud: false,
            status_item: None,
            panel: PanelBehavior::UTILITY,
            modal: false,
            theme: crate::styles::Theme::tengen(),
            scene: None,
            window_style: WindowStyle::TitleBar,
            window_toolbar_full_screen_visibility: WindowToolbarFullScreenVisibility::Automatic,
        }
    }

    pub fn theme(mut self, theme: crate::styles::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn window_style(mut self, window_style: WindowStyle) -> Self {
        self.window_style = window_style;
        self
    }

    pub fn window_toolbar_full_screen_visibility(
        mut self,
        visibility: WindowToolbarFullScreenVisibility,
    ) -> Self {
        self.window_toolbar_full_screen_visibility = visibility;
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
    pub(crate) titlebar_height: State<f32>,
    // Where the sidebar spans the window, left and width: on macOS the full-screen bar leaves a soft edge over it.
    pub(crate) sidebar_span: State<(f32, f32)>,
    pub hud: bool,
    pub panel: bool,
    pub toolbar_on_hover: bool,
    pub ime_preedit: State<String>,
    pub(crate) primary: Clipboard,
    pub(crate) clipboard: Clipboard,
}

pub(crate) fn window_activated() -> bool {
    freya_core::prelude::try_consume_root_context::<WindowControls>()
        .is_none_or(|controls| controls.status.read().activated)
}

impl WindowControls {
    pub(super) fn new(
        options: &WindowOptions,
        decoration: DecorationResponse,
        panel: bool,
        surface_id: ObjectId,
        ping: Ping,
        primary: Clipboard,
        clipboard: Clipboard,
    ) -> Self {
        let fixed = options.fixed();
        Self {
            queue: Rc::default(),
            ping,
            pointer_serial: Rc::default(),
            status: State::create_global(WindowStatus {
                decoration,
                fixed,
                ..WindowStatus::default()
            }),
            title: options.title.clone(),
            fixed,
            fixed_width: matches!(
                (options.min_size, options.max_size),
                (Some((min, _)), Some((max, _))) if min > 0 && min == max
            ),
            surface_id,
            focus_bar: State::create_global(false),
            focus_bar_progress: State::create_global(0.0),
            focus_overlay: false,
            focus_bar_height: State::create_global(ctdesign::MENUBAR_HEIGHT as f32),
            desktop: State::create_global(None),
            titlebar_height: State::create_global(0.0),
            sidebar_span: State::create_global((0.0, 0.0)),
            hud: options.hud,
            panel,
            toolbar_on_hover: options.window_toolbar_full_screen_visibility
                == WindowToolbarFullScreenVisibility::OnHover,
            ime_preedit: State::create_global(String::new()),
            primary,
            clipboard,
        }
    }

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
        if self.titlebar_overlaid() {
            *self.focus_bar_height.read() + ctdesign::FULLSCREEN_BAR.content_top
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
    crate::app::record_app_id(&options.app_id);
    super::sys::run_window(app.into(), options)
}
