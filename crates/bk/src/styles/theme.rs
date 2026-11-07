use freya_core::prelude::*;

use super::{SliderOverride, SliderStyle};

#[derive(Clone, PartialEq)]
pub struct Theme {
    pub(super) slider: SliderOverride,
}

impl Theme {
    pub fn tengen() -> Self {
        Self {
            slider: SliderOverride::default(),
        }
    }

    pub fn slider(self, restyle: impl Fn(SliderStyle) -> SliderStyle + 'static) -> Self {
        Self {
            slider: self.slider.then(restyle),
        }
    }

    pub(crate) fn current() -> Self {
        try_consume_context::<State<Self>>().map_or_else(Self::tengen, |theme| theme.read().clone())
    }

    pub(super) fn within(self, scope: Self) -> Self {
        Self {
            slider: self.slider.then(move |style| scope.slider.apply(style)),
        }
    }
}

pub(crate) fn use_provide_theme(theme: Theme) {
    let mut shared = use_state(|| theme.clone());
    shared.set_if_modified(theme);
    use_hook(|| provide_context(shared));
}

#[derive(PartialEq)]
pub struct StyleScope {
    theme: Theme,
    child: Element,
}

impl StyleScope {
    pub(crate) fn new(theme: Theme, child: impl IntoElement) -> Self {
        Self {
            theme,
            child: child.into_element(),
        }
    }
}

impl Component for StyleScope {
    fn render(&self) -> impl IntoElement {
        let parent = use_hook(try_consume_context::<State<Theme>>);
        let inherited = parent.map_or_else(Theme::tengen, |theme| theme.read().clone());
        use_provide_theme(inherited.within(self.theme.clone()));
        self.child.clone()
    }
}
