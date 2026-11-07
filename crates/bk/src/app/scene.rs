use freya_core::integration::AppComponent;
use freya_core::prelude::*;

use crate::platform::{RunError, WindowOptions, open_utility_window, run_window};
use crate::style::Palette;
use crate::styles::{Theme, use_provide_theme};
use crate::theme::color;
use crate::titlebar::Titlebar;

pub struct Application {
    name: String,
    theme: Theme,
}

impl Application {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            theme: Theme::tengen(),
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn window<E: IntoElement>(
        self,
        title: impl Into<String>,
        content: impl Fn() -> E + 'static,
    ) -> Scene {
        let title = title.into();
        Scene {
            options: WindowOptions::new(title.clone(), self.name),
            title,
            content: AppComponent::from(content),
            theme: self.theme,
        }
    }
}

pub struct Scene {
    options: WindowOptions,
    title: String,
    content: AppComponent,
    theme: Theme,
}

impl Scene {
    pub fn options(mut self, configure: impl FnOnce(WindowOptions) -> WindowOptions) -> Self {
        self.options = configure(self.options);
        self
    }

    pub fn run(self) -> Result<(), RunError> {
        let Self {
            options,
            title,
            content,
            theme,
        } = self;
        run_window(
            move || window_root(&title, content.clone(), theme.clone()),
            options,
        )
    }
}

pub struct UtilityWindow {
    options: WindowOptions,
    content: AppComponent,
}

impl UtilityWindow {
    pub fn new<E: IntoElement>(
        title: impl Into<String>,
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

fn window_root(title: &str, content: AppComponent, theme: Theme) -> impl IntoElement + use<> {
    use_provide_theme(theme);
    rect()
        .expanded()
        .background(color(Palette::current().window_background))
        .child(Titlebar::new().title(title))
        .child(rect().expanded().center().child(content))
}
