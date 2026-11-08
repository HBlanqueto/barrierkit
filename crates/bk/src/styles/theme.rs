use std::any::Any;
use std::rc::Rc;

use ctappearance::Resolved;
use freya_core::current_context::CurrentContext;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;

pub trait ControlStyle: Clone + 'static {
    fn tengen() -> Self;

    fn accessible(self, _: &Resolved) -> Self {
        self
    }
}

pub(crate) fn show_borders<S>(style: S, appearance: &Resolved, bordered: impl FnOnce(S) -> S) -> S {
    if appearance.show_borders {
        bordered(style)
    } else {
        style
    }
}

pub(crate) fn reduce_motion<S>(style: S, appearance: &Resolved, still: impl FnOnce(S) -> S) -> S {
    if appearance.reduced_motion {
        still(style)
    } else {
        style
    }
}

#[derive(Clone, Debug)]
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
    resolve_from(S::tengen(), instance)
}

pub(crate) fn resolve_from<S: ControlStyle>(base: S, instance: &Theme) -> S {
    let themed = match try_consume_context::<State<Theme>>() {
        Some(theme) => theme.read().apply(base),
        None => base,
    };
    instance.apply(themed).accessible(&crate::theme::resolved())
}

pub(crate) fn current_theme() -> Theme {
    CurrentContext::try_with(|_| try_consume_context::<State<Theme>>())
        .flatten()
        .map_or_else(Theme::tengen, |theme| theme.peek().clone())
}

pub(crate) struct PopupRoot {
    pub(crate) theme: Theme,
    pub(crate) content: AppComponent,
}

impl App for PopupRoot {
    fn render(&self) -> impl IntoElement {
        use_provide_theme(self.theme.clone());
        self.content.clone()
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

    struct Probe(Rc<std::cell::Cell<f32>>);

    impl App for Probe {
        fn render(&self) -> impl IntoElement {
            self.0.set(resolve::<SliderStyle>(&Theme::tengen()).width);
            rect()
        }
    }

    struct Opener(Rc<std::cell::Cell<f32>>);

    impl App for Opener {
        fn render(&self) -> impl IntoElement {
            let width = self.0.clone();
            use_hook(move || {
                crate::platform::Popups::get().open(
                    crate::platform::PopupSpec::default(),
                    AppComponent::new(Probe(width)),
                )
            });
            rect()
        }
    }

    #[test]
    fn a_popup_resolves_styles_with_the_theme_of_the_view_that_opened_it() {
        let width = Rc::new(std::cell::Cell::new(0.0));
        let scope = Theme::tengen().style(|s: SliderStyle| SliderStyle { width: 100.0, ..s });
        let opener = AppComponent::new(Opener(width.clone()));
        let window = AppComponent::from(move || StyleScope::new(scope.clone(), opener.clone()));
        let (popups, queue) =
            crate::platform::Popups::for_test(crate::platform::ObjectId::null(), None);
        let size = torin::prelude::Size2D::new(100.0, 100.0);
        let (mut runner, _) = freya_testing::TestingRunner::new(
            window,
            size,
            move |runner| {
                runner.provide_root_context(move || popups.clone());
            },
            1.0,
        );
        runner.sync_and_update();
        let content = queue
            .borrow_mut()
            .drain(..)
            .find_map(|command| match command {
                crate::platform::PopupCommand::Open { content, .. } => Some(content),
                _ => None,
            })
            .expect("the opener opens a popup");
        let (mut popup, _) = freya_testing::TestingRunner::new(content, size, |_| {}, 1.0);
        popup.sync_and_update();
        assert_eq!(width.get(), 100.0);
    }
}
