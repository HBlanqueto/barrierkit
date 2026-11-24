use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use ctappearance::Resolved;
use ctdesign::{ControlSize, Font};
use freya_core::current_context::CurrentContext;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;

use super::{
    ButtonStyle, CheckboxStyle, ComboBoxStyle, DatePickerStyle, PopUpButtonStyle,
    ProgressViewStyle, RadioGroupStyle, SearchFieldStyle, SegmentedControlStyle, ShapeStyle,
    SliderStyle, SpinnerStyle, StateValue, StepperStyle, SwitchStyle, TabViewStyle, TextFieldStyle,
    TextStyle,
};

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

    // One theme per size: a new one each render is unequal (`Rc::ptr_eq`) and renders every themed control below again.
    pub(crate) fn control_size(size: ControlSize) -> Self {
        thread_local!(static SIZED: RefCell<Vec<(ControlSize, Theme)>> = const { RefCell::new(Vec::new()) });
        SIZED.with_borrow_mut(|sized| {
            if let Some((_, theme)) = sized.iter().find(|(built, _)| *built == size) {
                return theme.clone();
            }
            let theme = Self::sized(size);
            sized.push((size, theme.clone()));
            theme
        })
    }

    fn sized(size: ControlSize) -> Self {
        Self::tengen()
            .style(move |style: ButtonStyle| style.sized(size))
            .style(move |style: PopUpButtonStyle| style.sized(size))
            .style(move |style: SegmentedControlStyle| style.sized(size))
            .style(move |style: TabViewStyle| style.sized(size))
            .style(move |style: CheckboxStyle| style.sized(size))
            .style(move |style: RadioGroupStyle| style.sized(size))
            .style(move |style: SwitchStyle| style.sized(size))
            .style(move |style: SliderStyle| style.sized(size))
            .style(move |style: StepperStyle| style.sized(size))
            .style(move |style: TextFieldStyle| style.sized(size))
            .style(move |style: SearchFieldStyle| style.sized(size))
            .style(move |style: ComboBoxStyle| style.sized(size))
            .style(move |style: DatePickerStyle| style.sized(size))
            .style(move |style: ProgressViewStyle| style.sized(size))
            .style(move |style: SpinnerStyle| style.sized(size))
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

// A `Label` in a scope keeps the compact layout of a button label (measured against AppKit); outside one it is SwiftUI's.
#[derive(Clone, Copy)]
pub(crate) struct InTextStyleScope;

#[derive(PartialEq)]
pub(crate) struct TextStyleScope {
    font: Font,
    foreground: ShapeStyle,
    content: Callback<(), Element>,
}

impl TextStyleScope {
    pub(crate) fn new(font: Font, foreground: ShapeStyle, content: Callback<(), Element>) -> Self {
        Self {
            font,
            foreground,
            content,
        }
    }
}

// The theme is built once and reads the style when applied, so `use_provide_theme` sees an equal
// theme on every render and does not notify the readers it just served.
impl Component for TextStyleScope {
    fn render(&self) -> impl IntoElement {
        use_hook(|| provide_context(InTextStyleScope));
        let mut text = use_state(|| (self.font, self.foreground.clone()));
        text.set_if_modified((self.font, self.foreground.clone()));
        let scope = use_hook(move || {
            Theme::tengen().style(move |_: TextStyle| {
                let (font, foreground) = text.peek().clone();
                TextStyle {
                    font,
                    foreground: StateValue::new(foreground),
                }
            })
        });
        let parent = use_hook(try_consume_context::<State<Theme>>);
        let inherited = parent.map_or_else(Theme::tengen, |theme| theme.read().clone());
        use_provide_theme(inherited.within(scope));
        self.content.call(())
    }
}
