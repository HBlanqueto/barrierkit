use std::rc::Rc;

use freya_core::prelude::*;

pub struct FocusState<T: 'static> {
    targets: State<Vec<(T, AccessibilityId)>>,
}

impl<T> Clone for FocusState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for FocusState<T> {}

impl<T> PartialEq for FocusState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.targets == other.targets
    }
}

pub fn use_focus_state<T: 'static>() -> FocusState<T> {
    FocusState {
        targets: use_state(Vec::new),
    }
}

impl<T: Clone + PartialEq + 'static> FocusState<T> {
    pub fn get(&self) -> Option<T> {
        self.targets
            .read()
            .iter()
            .find(|(_, id)| id.is_focused())
            .map(|(value, _)| value.clone())
    }

    pub fn set(&self, value: Option<T>) {
        let targets = self.targets.peek();
        match value {
            Some(value) => {
                if let Some((_, id)) = targets.iter().find(|(target, _)| *target == value) {
                    id.request_focus();
                }
            }
            None => targets.iter().for_each(|(_, id)| id.request_unfocus()),
        }
    }
}

type Unregister = Rc<dyn Fn()>;

#[derive(Clone)]
struct FocusTag(Rc<dyn Fn(AccessibilityId) -> Unregister>);

fn tag<T: Clone + 'static>(targets: State<Vec<(T, AccessibilityId)>>, value: T) -> FocusTag {
    FocusTag(Rc::new(move |id| {
        let mut targets = targets;
        targets.write().push((value.clone(), id));
        Rc::new(move || {
            let mut targets = targets;
            if let Some(mut targets) = targets.try_write() {
                targets.retain(|(_, target)| *target != id);
            }
        })
    }))
}

struct Registration(Option<Unregister>);

impl Drop for Registration {
    fn drop(&mut self) {
        if let Some(unregister) = self.0.take() {
            unregister();
        }
    }
}

pub(crate) fn use_focus_target(id: AccessibilityId) {
    use_hook(move || {
        Rc::new(Registration(
            try_consume_context::<FocusTag>().map(|tag| (tag.0)(id)),
        ))
    });
}

#[derive(PartialEq)]
pub(crate) struct Focused<T: 'static> {
    state: FocusState<T>,
    value: T,
    child: Element,
}

impl<T: 'static> Focused<T> {
    pub(crate) fn new(state: FocusState<T>, value: T, child: impl IntoElement) -> Self {
        Self {
            state,
            value,
            child: child.into_element(),
        }
    }
}

// The tag reads `value` once, when the scope is created: a control below registers under it as it mounts.
impl<T: Clone + PartialEq + 'static> Component for Focused<T> {
    fn render(&self) -> impl IntoElement {
        let (targets, value) = (self.state.targets, self.value.clone());
        use_provide_context(move || tag(targets, value));
        self.child.clone()
    }
}

#[derive(PartialEq)]
pub(crate) struct DefaultFocus<T: 'static> {
    state: FocusState<T>,
    value: T,
    child: Element,
}

impl<T: 'static> DefaultFocus<T> {
    pub(crate) fn new(state: FocusState<T>, value: T, child: impl IntoElement) -> Self {
        Self {
            state,
            value,
            child: child.into_element(),
        }
    }
}

// The effect runs after the children have mounted and registered their ids.
impl<T: Clone + PartialEq + 'static> Component for DefaultFocus<T> {
    fn render(&self) -> impl IntoElement {
        let (state, value) = (self.state, self.value.clone());
        use_after_side_effect(move || state.set(Some(value.clone())));
        self.child.clone()
    }
}
