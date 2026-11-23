use std::cell::RefCell;
use std::rc::Rc;

use bk::views::navigation_stack::{NavigationPath, NavigationStack, use_navigation_path};
use ctappearance::{AppearanceConfig, resolve};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

const SIZE: (f32, f32) = (200.0, 120.0);

type Handle = Rc<RefCell<Option<State<NavigationPath<u8>>>>>;

struct Case(Handle);

impl App for Case {
    fn render(&self) -> impl IntoElement {
        let path = use_state(NavigationPath::<u8>::new);
        *self.0.borrow_mut() = Some(path);
        NavigationStack::new(
            path,
            rect().expanded().background(Color::WHITE),
            |route: &u8| rect().expanded().background(Color::from_rgb(*route, 0, 0)),
        )
    }
}

fn run() -> (TestingRunner, Handle) {
    let resolved = resolve(
        &AppearanceConfig {
            reduced_motion: true,
            ..Default::default()
        },
        None,
    );
    let handle = Handle::default();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Case(handle.clone())),
        Size2D::new(SIZE.0, SIZE.1),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        1.0,
    );
    test.sync_and_update();
    (test, handle)
}

fn center(test: &mut TestingRunner) -> [u8; 4] {
    test.sync_and_update();
    test.sync_and_update();
    let (width, height, pixels) = crate::image::decode(test.render().as_bytes());
    let at = ((height / 2) * width + width / 2) * 4;
    [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

#[test]
fn a_stack_shows_its_root_then_the_route_pushed_and_the_root_again_after_a_pop() {
    let (mut test, handle) = run();
    let path = handle.borrow().unwrap();
    assert_eq!(center(&mut test), [255, 255, 255, 255]);
    let mut path = path;
    path.write().push(200);
    assert_eq!(center(&mut test), [200, 0, 0, 255]);
    path.write().pop();
    assert_eq!(center(&mut test), [255, 255, 255, 255]);
}

#[test]
fn a_page_pushes_onto_the_path_of_its_stack_without_a_callback_chain() {
    struct Deep(Handle);
    impl App for Deep {
        fn render(&self) -> impl IntoElement {
            let path = use_state(NavigationPath::<u8>::new);
            *self.0.borrow_mut() = Some(path);
            NavigationStack::new(path, Pusher, |_: &u8| rect().expanded())
        }
    }
    #[derive(PartialEq)]
    struct Pusher;
    impl Component for Pusher {
        fn render(&self) -> impl IntoElement {
            let mut path = use_navigation_path::<u8>();
            if path.peek().is_empty() {
                path.write().push(7);
            }
            rect().expanded()
        }
    }
    let handle = Handle::default();
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Deep(handle.clone())),
        Size2D::new(SIZE.0, SIZE.1),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    test.sync_and_update();
    assert_eq!(handle.borrow().unwrap().peek().len(), 1);
}
