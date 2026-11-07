use freya_core::integration::AppComponent;
use freya_core::prelude::*;

use crate::platform::{RunError, WindowOptions, run_window};
use crate::style::Palette;
use crate::theme::color;
use crate::titlebar::Titlebar;

pub struct Application {
    name: String,
}

impl Application {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
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
        }
    }
}

pub struct Scene {
    options: WindowOptions,
    title: String,
    content: AppComponent,
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
        } = self;
        run_window(move || window_root(&title, content.clone()), options)
    }
}

fn window_root(title: &str, content: AppComponent) -> impl IntoElement + use<> {
    rect()
        .expanded()
        .background(color(Palette::current().window_background))
        .child(Titlebar::new().title(title))
        .child(rect().expanded().center().child(content))
}
