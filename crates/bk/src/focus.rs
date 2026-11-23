use std::rc::Rc;

use freya_core::prelude::*;

pub struct FocusState<T: 'static> {
    targets: State<Vec<(T, AccessibilityId)>>,
    forgotten: State<Vec<AccessibilityId>>,
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
        forgotten: use_state(Vec::new),
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
        self.sweep();
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

impl<T: Clone + PartialEq + 'static> FocusState<T> {
    fn register(&self, value: T, id: AccessibilityId) {
        self.sweep();
        let mut targets = self.targets;
        targets.write().push((value, id));
    }

    fn forget(&self, id: AccessibilityId) {
        let (mut targets, mut forgotten) = (self.targets, self.forgotten);
        if let Some(mut targets) = targets.try_write() {
            targets.retain(|(_, target)| *target != id);
        } else if let Some(mut forgotten) = forgotten.try_write() {
            forgotten.push(id);
        }
    }

    // A target forgotten while the list was borrowed waits here until the next write at event time.
    fn sweep(&self) {
        if self.forgotten.peek().is_empty() {
            return;
        }
        let (mut targets, mut forgotten) = (self.targets, self.forgotten);
        let gone = std::mem::take(&mut *forgotten.write());
        targets.write().retain(|(_, id)| !gone.contains(id));
    }
}

type Unregister = Rc<dyn Fn()>;

#[derive(Clone)]
struct FocusTag(Rc<dyn Fn(AccessibilityId) -> Unregister>);

fn tag<T: Clone + PartialEq + 'static>(state: FocusState<T>, value: T) -> FocusTag {
    FocusTag(Rc::new(move |id| {
        state.register(value.clone(), id);
        Rc::new(move || state.forget(id))
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
        let (state, value) = (self.state, self.value.clone());
        use_provide_context(move || tag(state, value));
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

#[cfg(test)]
mod tests {
    use super::*;
    use freya_core::integration::{App, AppComponent};
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    struct Idle;
    impl App for Idle {
        fn render(&self) -> impl IntoElement {
            rect()
        }
    }

    #[test]
    fn a_target_forgotten_while_the_list_is_borrowed_is_gone_after_the_next_registration() {
        let (test, _) = TestingRunner::new(
            AppComponent::new(Idle),
            Size2D::new(10.0, 10.0),
            |_| {},
            1.0,
        );
        let ids = test.run_in(|| [(); 3].map(|()| AccessibilityId::new_unique()));
        let state = FocusState::<u8> {
            targets: State::create_global(Vec::new()),
            forgotten: State::create_global(Vec::new()),
        };
        state.register(0, ids[0]);
        state.register(1, ids[1]);
        {
            let _busy = state.targets.peek();
            state.forget(ids[0]);
        }
        assert_eq!(state.targets.peek().len(), 2);
        state.register(2, ids[2]);
        let values: Vec<u8> = state
            .targets
            .peek()
            .iter()
            .map(|(value, _)| *value)
            .collect();
        assert_eq!(values, [1, 2]);
    }
}
