use bk::modifiers::ViewModifiers;
use bk::views::feedback::Alert;
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

struct AlertCase(std::rc::Rc<std::cell::RefCell<Option<State<bool>>>>);

impl App for AlertCase {
    fn render(&self) -> impl IntoElement {
        let presented = use_state(|| false);
        *self.0.borrow_mut() = Some(presented);
        rect().expanded().alert(presented, || {
            Alert::new("Delete file?", "It cannot be undone.").confirm("Delete", |_| {})
        })
    }
}

fn centre_of(test: &TestingRunner, label: &str) -> Option<(f64, f64)> {
    test.find(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then(|| {
            let area = node.layout().area;
            (
                f64::from(area.min_x() + area.width() / 2.0),
                f64::from(area.min_y() + area.height() / 2.0),
            )
        })
    })
}

#[test]
fn an_alert_shows_while_its_state_is_true_and_a_button_press_dismisses_it() {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let handle = Default::default();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(AlertCase(std::rc::Rc::clone(&handle))),
        Size2D::new(600.0, 400.0),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        1.0,
    );
    test.sync_and_update();
    assert!(centre_of(&test, "Delete file?").is_none());
    let mut presented = handle.borrow().unwrap();
    presented.set(true);
    test.sync_and_update();
    test.sync_and_update();
    assert!(centre_of(&test, "Delete file?").is_some());
    test.click_cursor(centre_of(&test, "Delete").expect("the confirm button"));
    test.sync_and_update();
    test.sync_and_update();
    assert!(!*presented.read());
    assert!(centre_of(&test, "Delete file?").is_none());
}
