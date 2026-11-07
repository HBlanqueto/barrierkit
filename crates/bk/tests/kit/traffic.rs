use crate::image::{decode, pixel};
use bk::ProbeExt;
use bk::traffic::TrafficLights;
use ctappearance::traffic::{TrafficInput, visual};
use ctappearance::{AppearanceConfig, Resolved, resolve};
use ctdesign::{ButtonKind, Rgba};
use freya_core::element::ElementExt;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use std::rc::Rc;
use torin::prelude::Size2D;

fn resolved() -> Resolved {
    resolve(&AppearanceConfig::default(), None)
}

struct Case {
    input: TrafficInput,
}

impl App for Case {
    fn render(&self) -> impl IntoElement {
        TrafficLights::new().input(self.input)
    }
}

fn render(input: TrafficInput, scale: f64) -> (Vec<u8>, usize) {
    let app = AppComponent::new(Case { input });
    let (mut test, _) = TestingRunner::new(
        app,
        Size2D::new(80.0, 40.0),
        |runner| runner.provide_root_context(|| State::create(resolved())),
        scale,
    );
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let png = test.render();
    let (width, _, rgba) = decode(png.as_bytes());
    (rgba, width)
}

fn sample(input: TrafficInput, scale: f64) -> [u8; 4] {
    let (image, width) = render(input, scale);
    let x = (6.0 * scale).round() as usize;
    let y = (1.0 * scale).round() as usize;
    pixel(&image, width, x, y)
}

struct FixedCase {
    input: TrafficInput,
    fixed: bool,
}

impl App for FixedCase {
    fn render(&self) -> impl IntoElement {
        TrafficLights::new()
            .input(self.input)
            .fixed_width(self.fixed)
    }
}

fn render_fixed(input: TrafficInput, fixed: bool, scale: f64) -> (Vec<u8>, usize) {
    let app = AppComponent::new(FixedCase { input, fixed });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, scale);
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let png = test.render();
    let (width, _, rgba) = decode(png.as_bytes());
    (rgba, width)
}

fn compare(actual: [u8; 4], expected: Rgba) {
    let expected = expected.to_rgba8();
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(actual.abs_diff(expected) <= 1, "{actual:?} != {expected:?}");
    }
}

#[test]
fn traffic_lights_match_the_shared_visual_colours() {
    let states = [
        TrafficInput {
            focused: true,
            group_hovered: false,
            pressed: None,
            zoom_supported: true,
        },
        TrafficInput {
            focused: true,
            group_hovered: true,
            pressed: None,
            zoom_supported: true,
        },
        TrafficInput {
            focused: true,
            group_hovered: false,
            pressed: Some(ButtonKind::Close),
            zoom_supported: true,
        },
        TrafficInput {
            focused: false,
            group_hovered: true,
            pressed: None,
            zoom_supported: true,
        },
    ];
    let resolved = resolved();
    let palette = ctappearance::palette(resolved.scheme);
    for scale in [1.0, 2.0] {
        for input in states {
            let expected = visual(
                resolved.lights,
                resolved.traffic_style,
                palette.traffic_lights_inactive,
                palette.traffic_rim,
                input,
            )[0]
            .fill;
            compare(sample(input, scale), expected);
        }
    }
}

#[test]
fn hover_paints_the_close_glyph() {
    let input = TrafficInput {
        focused: true,
        group_hovered: true,
        pressed: None,
        zoom_supported: true,
    };
    let (image, width) = render(input, 1.0);
    let center = pixel(&image, width, 6, 6);
    let fill = pixel(&image, width, 6, 1);
    assert_ne!(center, fill, "the close glyph must cover the centre");
}

#[test]
fn the_shared_module_has_the_same_group_footprint() {
    let (_, _, width, height) = ctappearance::traffic::group_rect();
    assert_eq!(width, 3.0 * 14.0 + 2.0 * 9.0);
    assert_eq!(height, 14.0);
    assert_eq!(ctappearance::traffic::button_rect(1).2, 14.0);
}

#[test]
fn fixed_width_clicks_call_the_expected_actions() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let actions = Rc::new(RefCell::new(Vec::new()));
    let closes = Rc::new(RefCell::new(0u32));

    struct ActionCase {
        actions: Rc<RefCell<Vec<ButtonKind>>>,
        closes: Rc<RefCell<u32>>,
    }

    impl App for ActionCase {
        fn render(&self) -> impl IntoElement {
            let actions = self.actions.clone();
            let closes = self.closes.clone();
            TrafficLights::new()
                .focused(true)
                .fixed_width(true)
                .zoom_supported(true)
                .on_action(move |kind| {
                    actions.borrow_mut().push(kind);
                    if kind == ButtonKind::Close {
                        *closes.borrow_mut() += 1;
                    }
                })
        }
    }

    let app = AppComponent::new(ActionCase {
        actions: actions.clone(),
        closes: closes.clone(),
    });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, 1.0);
    test.sync_and_update();

    let centers = [(6.0, 6.0), (26.0, 6.0), (46.0, 6.0)];
    for (x, y) in centers {
        test.click_cursor((x, y));
        test.sync_and_update();
    }

    let actions = actions.borrow();
    assert_eq!(
        *actions,
        vec![
            ButtonKind::Close,
            ButtonKind::Minimize,
            ButtonKind::Maximize
        ]
    );
    assert_eq!(
        *closes.borrow(),
        1,
        "only the close light must call on_close"
    );
}

#[test]
fn fixed_width_green_uses_plus_glyph() {
    let input = TrafficInput {
        focused: true,
        group_hovered: true,
        pressed: None,
        zoom_supported: false,
    };
    let (normal, width) = render_fixed(input, false, 1.0);
    let (fixed, _) = render_fixed(input, true, 1.0);
    let center_x = 46;
    let center_y = 6;
    let normal_center = pixel(&normal, width, center_x, center_y);
    let fixed_center = pixel(&fixed, width, center_x, center_y);
    assert_ne!(
        normal_center, fixed_center,
        "fixed-width green light must render the plus glyph"
    );
}

fn render_png(test: &mut TestingRunner) -> (Vec<u8>, usize) {
    let png = test.render();
    let (width, _, rgba) = decode(png.as_bytes());
    (rgba, width)
}

struct InteractiveCase;

impl App for InteractiveCase {
    fn render(&self) -> impl IntoElement {
        TrafficLights::new().focused(true)
    }
}

#[test]
fn leaving_the_group_hides_the_glyphs() {
    let app = AppComponent::new(InteractiveCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, 1.0);
    test.sync_and_update();
    test.move_cursor((6.0, 6.0));
    test.sync_and_update();
    let (image, width) = render_png(&mut test);
    assert_ne!(
        pixel(&image, width, 6, 6),
        pixel(&image, width, 6, 1),
        "the close glyph must be visible while the pointer is over the light"
    );

    test.move_cursor((-1.0, -1.0));
    test.sync_and_update();
    let (image, width) = render_png(&mut test);
    assert_eq!(
        pixel(&image, width, 6, 6),
        pixel(&image, width, 6, 1),
        "the glyph must hide when the pointer leaves the group"
    );
}

struct ProbedCase;

impl App for ProbedCase {
    fn render(&self) -> impl IntoElement {
        TrafficLights::new().focused(true).probe("traffic-group")
    }
}

#[test]
fn moving_within_the_group_renders_it_at_most_once() {
    let app = AppComponent::new(ProbedCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, 1.0);
    test.sync_and_update();
    let group = |test: &TestingRunner| {
        test.find(|node, el| {
            (el.accessibility().builder.author_id() == Some("traffic-group"))
                .then(|| node.element())
        })
        .expect("the traffic-light group exists")
    };
    let address = |element: &Rc<dyn ElementExt>| Rc::as_ptr(element) as *const ();
    let mut last = address(&group(&test));
    let mut renders = 0;
    for x in [2.0, 6.0, 10.0, 14.0, 18.0, 22.0, 26.0, 30.0, 34.0, 38.0] {
        test.move_cursor((x, 6.0));
        test.sync_and_update();
        let current = address(&group(&test));
        if current != last {
            renders += 1;
            last = current;
        }
    }
    assert!(
        renders <= 1,
        "moving inside the group must render it at most once, got {renders}"
    );
}

#[test]
fn moving_the_pointer_away_after_a_press_clears_hover_and_press() {
    let app = AppComponent::new(InteractiveCase);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, 1.0);
    test.sync_and_update();
    test.move_cursor((6.0, 6.0));
    test.press_cursor((6.0, 6.0));
    test.sync_and_update();
    test.send_event(freya_core::events::platform::PlatformEvent::Mouse {
        name: freya_core::events::platform::MouseEventName::MouseMove,
        cursor: (-1.0, -1.0).into(),
        button: None,
    });
    test.sync_and_update();
    let (image, width) = render_png(&mut test);
    let fill = pixel(&image, width, 6, 1);
    compare(fill, resolved().lights[0]);
    assert_eq!(
        pixel(&image, width, 6, 6),
        fill,
        "a lost pointer must not leave the glyph or the pressed fill"
    );
}

struct CloseOnlyCase {
    input: TrafficInput,
}

impl App for CloseOnlyCase {
    fn render(&self) -> impl IntoElement {
        TrafficLights::new().input(self.input).close_only(true)
    }
}

#[test]
fn close_only_keeps_minimise_and_zoom_inactive() {
    let input = TrafficInput {
        focused: true,
        group_hovered: true,
        pressed: None,
        zoom_supported: true,
    };
    let app = AppComponent::new(CloseOnlyCase { input });
    let (mut test, _) = TestingRunner::new(app, Size2D::new(80.0, 40.0), |_| {}, 1.0);
    test.sync_and_update();
    let (width, _, image) = decode(test.render().as_bytes());
    let at = |x: f64, y: f64| -> [u8; 3] {
        let [r, g, b, _] = pixel(&image, width, x as usize, y as usize);
        [r, g, b]
    };
    let resolved = ctappearance::resolve(
        &ctappearance::load_appearance(),
        ctappearance::load_dominant(),
    );
    let inactive = ctappearance::palette(resolved.scheme)
        .traffic_lights_inactive
        .to_rgba8();
    let (origin, _, diameter) = ctappearance::traffic::button_rect(0);
    for index in 1..3 {
        let (x, _, _) = ctappearance::traffic::button_rect(index);
        let pixel = at(x - origin + diameter * 0.2, diameter / 2.0);
        for (actual, expected) in pixel.iter().zip(inactive) {
            assert!(actual.abs_diff(expected) <= 1, "light {index}: {pixel:?}");
        }
    }
    let close = at(diameter * 0.2, diameter / 2.0);
    assert_ne!(close, [inactive[0], inactive[1], inactive[2]]);
}
