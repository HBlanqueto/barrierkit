use bk::modifiers::ViewModifiers;
use ctappearance::{AppearanceConfig, resolve};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

struct Case(std::rc::Rc<std::cell::RefCell<Option<State<bool>>>>);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let presented = use_state(|| false);
        *self.0.borrow_mut() = Some(presented);
        rect()
            .expanded()
            .background(Color::WHITE)
            .sheet(presented, || {
                rect()
                    .width(Size::px(80.0))
                    .height(Size::px(60.0))
                    .background(Color::from_rgb(200, 0, 0))
                    .into_element()
            })
    }
}

fn red_pixels() -> impl FnMut(&mut TestingRunner) -> usize {
    |test| {
        test.sync_and_update();
        test.sync_and_update();
        let (_, _, pixels) = crate::image::decode(test.render().as_bytes());
        pixels
            .chunks(4)
            .filter(|p| p[0] == 200 && p[1] == 0 && p[2] == 0)
            .count()
    }
}

#[test]
fn a_sheet_shows_while_its_state_is_true_and_goes_when_it_is_false() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let handle = Default::default();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(std::rc::Rc::clone(&handle))),
        Size2D::new(300.0, 200.0),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        1.0,
    );
    let mut count = red_pixels();
    assert_eq!(count(&mut test), 0);
    let mut presented = handle.borrow().unwrap();
    presented.set(true);
    assert!(count(&mut test) > 1000);
    presented.set(false);
    assert_eq!(count(&mut test), 0);
}
