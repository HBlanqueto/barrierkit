use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use std::borrow::Cow;

use crate::app::commands::Commands;
use crate::app::windows::{MAIN_WINDOW, Scene, Scenes, WindowGroup};
use crate::platform::{RunError, WindowOptions, app_name, open_utility_window, run_window};
use crate::styles::Theme;

pub struct Application {
    id: String,
    theme: Theme,
    scenes: Vec<Scene>,
    commands: Option<Commands>,
}

impl Application {
    /// `id` is reverse DNS, like a bundle identifier: `com.example.notes`.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            theme: Theme::tengen(),
            scenes: Vec::new(),
            commands: Some(Commands::new()),
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        for scene in &mut self.scenes {
            scene.options.theme = theme.clone();
        }
        self.theme = theme;
        self
    }

    pub fn window<E: IntoElement>(
        self,
        title: impl Into<Cow<'static, str>>,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        self.scene(WindowGroup::new(MAIN_WINDOW, title, content))
    }

    pub fn scene(mut self, scene: impl Into<Scene>) -> Self {
        let mut scene = scene.into();
        scene.options.app_id = self.id.clone();
        scene.options.app_name = app_name(&self.id).to_owned();
        scene.options.theme = self.theme.clone();
        self.scenes.push(scene);
        self
    }

    /// Configures the window added last, so it goes after its `window` or `scene`.
    pub fn options(mut self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        if let Some(scene) = self.scenes.last_mut() {
            scene.options = configure(scene.options.clone());
        }
        self
    }

    pub fn commands(mut self, commands: Commands) -> Self {
        self.commands = Some(commands);
        self
    }

    pub fn run(self) -> Result<(), RunError> {
        let (options, root) = self.launch()?;
        run_window(root, options)
    }

    pub(crate) fn launch(self) -> Result<(WindowOptions, AppComponent), RunError> {
        let about = Scene::about(app_name(&self.id));
        let app = self.scene(about);
        let (scenes, launch) = Scenes::new(app_name(&app.id).to_owned(), app.scenes, app.commands);
        Ok(scenes.window(launch.ok_or(RunError::NoWindow)?, None))
    }
}

pub struct UtilityWindow {
    options: WindowOptions,
    content: AppComponent,
}

impl UtilityWindow {
    pub fn new<E: IntoElement>(
        title: impl Into<Cow<'static, str>>,
        app_id: impl Into<String>,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        Self {
            options: WindowOptions::new(title, app_id),
            content: AppComponent::from(content),
        }
    }

    pub fn options(mut self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        self.options = configure(self.options);
        self
    }

    pub fn open(self) {
        open_utility_window(self.content, self.options);
    }
}
