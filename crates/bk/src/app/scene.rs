use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use std::borrow::Cow;

use crate::app::commands::Commands;
use crate::app::windows::{MAIN_WINDOW, Scene, Scenes, WindowGroup};
use crate::platform::{RunError, WindowOptions, open_utility_window, run_window};
use crate::styles::Theme;

pub struct Application {
    name: String,
    theme: Theme,
    scenes: Vec<Scene>,
    commands: Option<Commands>,
}

impl Application {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
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
        scene.options.app_name = self.name.clone();
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
        let (scenes, launch) = Scenes::new(self.name, self.scenes, self.commands);
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
        name: impl Into<String>,
        content: impl Fn() -> E + 'static,
    ) -> Self {
        Self {
            options: WindowOptions::new(title, name),
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
