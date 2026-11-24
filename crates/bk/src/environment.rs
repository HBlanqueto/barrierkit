use freya_core::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct IsEnabled(bool);

impl Default for IsEnabled {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(PartialEq)]
pub(crate) struct EnvironmentScope<T> {
    value: T,
    child: Element,
}

impl<T> EnvironmentScope<T> {
    pub(crate) fn new(value: T, child: impl IntoElement) -> Self {
        Self {
            value,
            child: child.into_element(),
        }
    }
}

impl<T: Clone + PartialEq + 'static> Component for EnvironmentScope<T> {
    fn render(&self) -> impl IntoElement {
        use_provide_environment(self.value.clone());
        self.child.clone()
    }
}

// One `State` per scope takes the new value, so the readers that subscribed to it render again.
pub(crate) fn use_provide_environment<T: Clone + PartialEq + 'static>(value: T) {
    let mut shared = use_state(|| value.clone());
    shared.set_if_modified(value);
    use_hook(move || provide_context(shared));
}

#[derive(PartialEq)]
pub(crate) struct DisabledScope {
    disabled: bool,
    child: Element,
}

impl DisabledScope {
    pub(crate) fn new(disabled: bool, child: impl IntoElement) -> Self {
        Self {
            disabled,
            child: child.into_element(),
        }
    }
}

// A disabled view stays disabled below an enabled one: the scope reads its parent before it provides its own value.
impl Component for DisabledScope {
    fn render(&self) -> impl IntoElement {
        let enabled = use_is_enabled() && !self.disabled;
        EnvironmentScope::new(IsEnabled(enabled), self.child.clone())
    }
}

pub fn use_environment<T: Clone + PartialEq + Default + 'static>() -> T {
    try_consume_context::<State<T>>().map_or_else(T::default, |shared| shared.read().clone())
}

pub fn use_is_enabled() -> bool {
    use_environment::<IsEnabled>().0
}

pub(crate) fn use_disabled(disabled: bool) -> bool {
    disabled || !use_is_enabled()
}
