use std::any::{Any, TypeId};
use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use freya_core::integration::AppComponent;
use freya_core::prelude::*;

use super::commands::{
    BRING_ALL_TO_FRONT, CLOSE_WINDOW, Commands, Layout, MINIMIZE, NEW_WINDOW, QUIT, SETTINGS,
    TOGGLE_FULL_SCREEN, WINDOW_ACTIVATE, WINDOW_OPEN, ZOOM,
};
use super::use_menu_action;
use crate::document::first_edited;
use crate::i18n::text;
use crate::main_menu::MenuHandle;
use crate::platform::{WindowControls, WindowOptions, WindowStyle};
use crate::titlebar::Titlebar;

pub const MAIN_WINDOW: &str = "main";
pub const SETTINGS_WINDOW: &str = "settings";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Group,
    Window,
    Settings,
}

#[derive(Clone, Copy)]
struct ValueType {
    id: TypeId,
    same: fn(&dyn Any, &dyn Any) -> bool,
}

type Build = Rc<dyn Fn(Option<&dyn Any>) -> AppComponent>;

pub struct Scene {
    id: &'static str,
    kind: Kind,
    value: Option<ValueType>,
    pub(super) options: WindowOptions,
    build: Build,
}

fn plain<E: IntoElement>(content: impl Fn() -> E + 'static) -> Build {
    let content = AppComponent::from(content);
    Rc::new(move |_| content.clone())
}

impl Scene {
    fn new(kind: Kind, id: &'static str, title: Cow<'static, str>, build: Build) -> Self {
        Self {
            id,
            kind,
            value: None,
            options: WindowOptions::new(title, ""),
            build,
        }
    }

    fn with_options(mut self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        self.options = configure(self.options);
        self
    }
}

pub struct WindowGroup(Scene);

impl WindowGroup {
    pub fn new<E: IntoElement>(
        id: &'static str,
        title: impl Into<Cow<'static, str>>,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        Self(Scene::new(Kind::Group, id, title.into(), plain(content)))
    }

    pub fn for_value<T: Clone + PartialEq + 'static, E: IntoElement>(
        id: &'static str,
        title: impl Into<Cow<'static, str>>,
        content: impl Fn(T) -> E + 'static,
    ) -> Self {
        let content = Rc::new(content);
        let build: Build = Rc::new(move |value| {
            let value = value
                .and_then(|value| value.downcast_ref::<T>())
                .expect("a window of a value group opens with its value")
                .clone();
            let content = content.clone();
            AppComponent::from(move || content(value.clone()))
        });
        let mut scene = Scene::new(Kind::Group, id, title.into(), build);
        scene.value = Some(ValueType {
            id: TypeId::of::<T>(),
            same: |a, b| a.downcast_ref::<T>() == b.downcast_ref::<T>(),
        });
        Self(scene)
    }

    pub fn options(self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        Self(self.0.with_options(configure))
    }
}

pub struct Window(Scene);

impl Window {
    pub fn new<E: IntoElement>(
        id: &'static str,
        title: impl Into<Cow<'static, str>>,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        Self(Scene::new(Kind::Window, id, title.into(), plain(content)))
    }

    pub fn options(self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        Self(self.0.with_options(configure))
    }
}

pub struct Settings(Scene);

impl Settings {
    pub fn new<E: IntoElement>(content: impl Fn() -> E + 'static) -> Self {
        Self(Scene::new(
            Kind::Settings,
            SETTINGS_WINDOW,
            text!("settings").into(),
            plain(content),
        ))
    }

    pub fn options(self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        Self(self.0.with_options(configure))
    }
}

impl From<WindowGroup> for Scene {
    fn from(scene: WindowGroup) -> Self {
        scene.0
    }
}

impl From<Window> for Scene {
    fn from(scene: Window) -> Self {
        scene.0
    }
}

impl From<Settings> for Scene {
    fn from(scene: Settings) -> Self {
        scene.0
    }
}

struct Open {
    number: u32,
    scene: usize,
    value: Option<Rc<dyn Any>>,
    controls: WindowControls,
}

struct Inner {
    app: String,
    scenes: Vec<Scene>,
    commands: Option<Commands>,
    open: State<Vec<Open>>,
    next: Cell<u32>,
}

#[derive(Clone)]
pub(crate) struct Scenes(Rc<Inner>);

impl Scenes {
    pub(crate) fn new(
        app: String,
        scenes: Vec<Scene>,
        commands: Option<Commands>,
    ) -> (Self, Option<usize>) {
        let launch = scenes.iter().position(|scene| scene.kind != Kind::Settings);
        let inner = Inner {
            app,
            scenes,
            commands,
            open: State::create_global(Vec::new()),
            next: Cell::new(0),
        };
        (Self(Rc::new(inner)), launch)
    }

    pub(crate) fn window(
        &self,
        index: usize,
        value: Option<Rc<dyn Any>>,
    ) -> (WindowOptions, AppComponent) {
        let scene = &self.0.scenes[index];
        let mut options = scene.options.clone();
        options.scene = Some(scene.id).filter(|id| *id != MAIN_WINDOW);
        if options.menu.is_none()
            && let Some(commands) = &self.0.commands
        {
            options.menu = Some(commands.menu_bar(&self.layout(&self.0.open.peek(), None, true)));
        }
        let scenes = self.clone();
        let root = AppComponent::from(move || scene_root(&scenes, index, value.clone()));
        (options, root)
    }

    fn layout(&self, open: &[Open], active: Option<u32>, zoom: bool) -> Layout {
        let scenes = &self.0.scenes;
        let of_kind = |kind| scenes.iter().filter(move |scene| scene.kind == kind);
        Layout {
            app: self.0.app.clone(),
            settings: of_kind(Kind::Settings).next().is_some(),
            new_window: self.new_window_scene().is_some(),
            window_scenes: of_kind(Kind::Window)
                .map(|scene| (scene.id, scene.options.title.clone()))
                .collect(),
            windows: open
                .iter()
                .filter(|window| scenes[window.scene].kind == Kind::Group)
                .map(|window| (window.number, scenes[window.scene].options.title.clone()))
                .collect(),
            active,
            zoom,
        }
    }

    fn new_window_scene(&self) -> Option<usize> {
        self.0
            .scenes
            .iter()
            .position(|scene| scene.kind == Kind::Group && scene.value.is_none())
    }

    fn register(&self, scene: usize, value: Option<Rc<dyn Any>>, controls: WindowControls) -> u32 {
        let number = self.0.next.get() + 1;
        self.0.next.set(number);
        let mut open = self.0.open;
        open.write().push(Open {
            number,
            scene,
            value,
            controls,
        });
        number
    }

    fn unregister(&self, number: u32) {
        let mut open = self.0.open;
        open.write().retain(|window| window.number != number);
    }

    fn publish_window_menu(&self, own: u32, zoom: bool) {
        let handle = use_hook(try_consume_root_context::<MenuHandle>);
        if let (Some(commands), Some(handle)) = (&self.0.commands, handle) {
            let layout = self.layout(&self.0.open.read(), Some(own), zoom);
            handle.set_menu_items(text!("menu-window"), commands.window_items(&layout));
        }
    }

    fn open_scene(&self, id: &str, value: Option<Rc<dyn Any>>, from: &WindowControls) {
        let Some(index) = self.0.scenes.iter().position(|scene| scene.id == id) else {
            tracing::warn!(id, "bk: no scene has this id");
            return;
        };
        self.open_index(index, value, from);
    }

    fn open_index(&self, index: usize, value: Option<Rc<dyn Any>>, from: &WindowControls) {
        let scene = &self.0.scenes[index];
        if scene.value.is_some() != value.is_some() {
            tracing::warn!(
                id = scene.id,
                "bk: a window group opens with a value or without, as declared"
            );
            return;
        }
        let open = self.0.open.peek();
        let shown = open.iter().find(|window| {
            window.scene == index
                && match (&scene.value, &window.value, &value) {
                    (Some(kind), Some(shown), Some(wanted)) => (kind.same)(&**shown, &**wanted),
                    _ => scene.kind != Kind::Group,
                }
        });
        match shown {
            Some(window) => window.controls.make_key_and_order_front(),
            None => {
                let (options, root) = self.window(index, value);
                from.open_window(root, options);
            }
        }
    }

    fn open_value<T: Clone + PartialEq + 'static>(&self, value: T, from: &WindowControls) {
        let wanted = TypeId::of::<T>();
        match self
            .0
            .scenes
            .iter()
            .position(|scene| scene.value.is_some_and(|kind| kind.id == wanted))
        {
            Some(index) => self.open_index(index, Some(Rc::new(value)), from),
            None => tracing::warn!("bk: no window group takes this value type"),
        }
    }

    fn dismiss(&self, id: &str) {
        for window in self.0.open.peek().iter() {
            if self.0.scenes[window.scene].id == id {
                window.controls.close();
            }
        }
    }

    fn perform(&self, action: &str, own: &WindowControls) {
        let front = |number: u32| {
            if let Some(window) = self.0.open.peek().iter().find(|w| w.number == number) {
                window.controls.make_key_and_order_front();
            }
        };
        if action == NEW_WINDOW.id {
            if let Some(index) = self.new_window_scene() {
                self.open_index(index, None, own);
            }
        } else if action == SETTINGS.id {
            self.open_scene(SETTINGS_WINDOW, None, own);
        } else if action == CLOSE_WINDOW.id {
            own.close();
        } else if action == QUIT.id {
            // A document with unsaved changes asks first; the user quits again after answering, as on macOS.
            let open = self.0.open.peek();
            match first_edited().and_then(|id| open.iter().find(|w| w.controls.id() == id)) {
                Some(edited) => edited.controls.close(),
                None => open.iter().for_each(|window| window.controls.close()),
            }
        } else if action == MINIMIZE.id {
            own.minimize();
        } else if action == ZOOM.id {
            own.toggle_maximize();
        } else if action == TOGGLE_FULL_SCREEN.id {
            own.toggle_fullscreen();
        } else if action == BRING_ALL_TO_FRONT.id {
            for window in self.0.open.peek().iter() {
                window.controls.make_key_and_order_front();
            }
        } else if let Some(number) = action.strip_prefix(WINDOW_ACTIVATE) {
            front(number.parse().unwrap_or_default());
        } else if let Some(id) = action.strip_prefix(WINDOW_OPEN) {
            self.open_scene(id, None, own);
        }
    }
}

fn scene_root(scenes: &Scenes, index: usize, value: Option<Rc<dyn Any>>) -> Element {
    let controls = WindowControls::get();
    let number = use_hook({
        let (scenes, controls, value) = (scenes.clone(), controls.clone(), value.clone());
        move || scenes.register(index, value, controls)
    });
    use_drop({
        let scenes = scenes.clone();
        move || scenes.unregister(number)
    });
    use_provide_context({
        let scenes = scenes.clone();
        move || scenes
    });
    use_menu_action({
        let (scenes, controls) = (scenes.clone(), controls.clone());
        move |action| scenes.perform(action, &controls)
    });
    scenes.publish_window_menu(number, !controls.fixed);

    let scene = &scenes.0.scenes[index];
    let content = (scene.build)(value.as_deref());
    match scene.kind {
        Kind::Settings => content.into_element(),
        Kind::Group | Kind::Window => window_root(&scene.options, content),
    }
}

// A window with a hidden title bar or a plain frame draws its own bar, as SwiftUI's `.hiddenTitleBar`.
fn window_root(options: &WindowOptions, content: AppComponent) -> Element {
    rect()
        .expanded()
        .background(crate::theme::color(
            crate::style::Palette::current().window_background,
        ))
        .maybe_child(
            (options.window_style == WindowStyle::TitleBar)
                .then(|| Titlebar::new().title(options.title.clone())),
        )
        .child(rect().expanded().center().child(content))
        .into_element()
}

#[derive(Clone)]
pub struct OpenWindowAction {
    scenes: Scenes,
    controls: WindowControls,
}

impl OpenWindowAction {
    pub fn open(&self, id: &str) {
        self.scenes.open_scene(id, None, &self.controls);
    }

    pub fn open_value<T: Clone + PartialEq + 'static>(&self, value: T) {
        self.scenes.open_value(value, &self.controls);
    }
}

pub fn use_open_window() -> OpenWindowAction {
    OpenWindowAction {
        scenes: use_consume::<Scenes>(),
        controls: WindowControls::get(),
    }
}

#[derive(Clone)]
pub struct DismissWindowAction {
    scenes: Scenes,
    controls: WindowControls,
}

impl DismissWindowAction {
    pub fn dismiss(&self) {
        self.controls.close();
    }

    pub fn dismiss_id(&self, id: &str) {
        self.scenes.dismiss(id);
    }
}

pub fn use_dismiss_window() -> DismissWindowAction {
    DismissWindowAction {
        scenes: use_consume::<Scenes>(),
        controls: WindowControls::get(),
    }
}
