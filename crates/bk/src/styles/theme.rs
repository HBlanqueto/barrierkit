use std::any::Any;
use std::rc::Rc;

use ctappearance::Resolved;
use freya_core::prelude::*;

pub trait ControlStyle: Clone + 'static {
    fn tengen() -> Self;

    fn accessible(self, appearance: &Resolved) -> Self;
}

#[derive(Clone)]
pub struct Theme {
    overrides: Vec<Rc<dyn Any>>,
}

impl Theme {
    pub fn tengen() -> Self {
        Self {
            overrides: Vec::new(),
        }
    }

    pub fn style<S: ControlStyle>(mut self, restyle: impl Fn(S) -> S + 'static) -> Self {
        let restyle: Box<dyn Fn(S) -> S> = Box::new(restyle);
        self.overrides.push(Rc::new(restyle));
        self
    }

    pub(crate) fn apply<S: ControlStyle>(&self, style: S) -> S {
        self.overrides
            .iter()
            .filter_map(|restyle| restyle.downcast_ref::<Box<dyn Fn(S) -> S>>())
            .fold(style, |style, restyle| restyle(style))
    }

    pub(super) fn within(self, scope: Self) -> Self {
        Self {
            overrides: [self.overrides, scope.overrides].concat(),
        }
    }
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.overrides.len() == other.overrides.len()
            && self
                .overrides
                .iter()
                .zip(&other.overrides)
                .all(|(own, other)| Rc::ptr_eq(own, other))
    }
}

pub(crate) fn resolve<S: ControlStyle>(instance: &Theme) -> S {
    let themed = try_consume_context::<State<Theme>>()
        .map_or_else(S::tengen, |theme| theme.read().apply(S::tengen()));
    instance.apply(themed).accessible(&crate::theme::resolved())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::styles::SliderStyle;

    #[derive(Clone)]
    struct Other(f32);

    impl ControlStyle for Other {
        fn tengen() -> Self {
            Self(1.0)
        }

        fn accessible(self, _: &Resolved) -> Self {
            self
        }
    }

    fn doubled_track(style: SliderStyle) -> SliderStyle {
        SliderStyle {
            track_height: style.track_height * 2.0,
            ..style
        }
    }

    #[test]
    fn theme_then_containers_outermost_first_then_instance_each_see_the_previous_level() {
        let theme = Theme::tengen().style(|s: SliderStyle| SliderStyle { width: 100.0, ..s });
        let outer = Theme::tengen().style(doubled_track);
        let inner = Theme::tengen().style(|s: SliderStyle| SliderStyle {
            track_height: s.track_height + 1.0,
            ..s
        });
        let instance = Theme::tengen().style(|s: SliderStyle| SliderStyle {
            track_height: s.track_height * 10.0,
            ..s
        });
        let themed = theme
            .within(outer)
            .within(inner)
            .apply(SliderStyle::tengen());
        let style = instance.apply(themed);
        let tengen = SliderStyle::tengen();
        assert_eq!(style.width, 100.0);
        assert_eq!(style.track_height, (tengen.track_height * 2.0 + 1.0) * 10.0);
        assert_eq!(style.knob_size, tengen.knob_size);
    }

    #[test]
    fn an_override_changes_only_the_style_its_closure_takes() {
        let theme = Theme::tengen()
            .style(|s: SliderStyle| SliderStyle { width: 100.0, ..s })
            .style(|Other(value)| Other(value * 3.0));
        assert_eq!(theme.apply(Other::tengen()).0, 3.0);
        assert_eq!(theme.apply(SliderStyle::tengen()).width, 100.0);
    }
}
