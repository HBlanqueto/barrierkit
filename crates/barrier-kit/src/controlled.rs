use freya_core::prelude::*;

pub(crate) fn use_controlled<T: Clone + PartialEq + 'static>(prop: &T) -> State<T> {
    let mut value = use_state(|| prop.clone());
    let mut seen = use_state(|| prop.clone());
    if *seen.peek() != *prop {
        seen.set(prop.clone());
        value.set(prop.clone());
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use freya_core::integration::{App, AppComponent};
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    struct Control {
        prop: u8,
        handle: State<Option<State<u8>>>,
        shown: State<u8>,
    }
    impl Component for Control {
        fn render(&self) -> impl IntoElement {
            let value = use_controlled(&self.prop);
            let (mut handle, mut shown) = (self.handle, self.shown);
            handle.set(Some(value));
            shown.set(*value.read());
            rect()
        }
    }
    impl PartialEq for Control {
        fn eq(&self, other: &Self) -> bool {
            self.prop == other.prop
        }
    }
    struct Host {
        prop: State<u8>,
        handle: State<Option<State<u8>>>,
        shown: State<u8>,
    }
    impl App for Host {
        fn render(&self) -> impl IntoElement {
            Control {
                prop: *self.prop.read(),
                handle: self.handle,
                shown: self.shown,
            }
        }
    }

    #[test]
    fn a_control_keeps_its_own_change_until_the_app_changes_the_prop() {
        let mut prop = State::create_global(1);
        let handle = State::create_global(None);
        let shown = State::create_global(0);
        let app = AppComponent::new(Host {
            prop,
            handle,
            shown,
        });
        let (mut test, _) = TestingRunner::new(app, Size2D::new(10.0, 10.0), |_| {}, 1.0);
        test.sync_and_update();
        assert_eq!(*shown.peek(), 1);
        let mut value = (*handle.peek()).expect("the control rendered");
        value.set(9);
        test.sync_and_update();
        assert_eq!(*shown.peek(), 9);
        prop.set(2);
        test.sync_and_update();
        assert_eq!(*shown.peek(), 2);
    }
}
