use bk::prelude::*;
use ctdesign::Rgba;
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

const RED: Rgba = Rgba::rgba8(255, 0, 0, 1.0);
const WHITE: [u8; 4] = [255, 255, 255, 255];
const RED_PIXEL: [u8; 4] = [255, 0, 0, 255];

struct Scene<F: Fn() -> Element + 'static>(F);

impl<F: Fn() -> Element + 'static> App for Scene<F> {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .background((255, 255, 255))
            .child((self.0)())
    }
}

fn pixels_of(
    view: impl Fn() -> Element + 'static,
    (width, height): (f32, f32),
) -> impl Fn(usize, usize) -> [u8; 4] {
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Scene(view)),
        Size2D::new(width, height),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    let (stride, _, image) = crate::image::decode(test.render().as_bytes());
    move |x, y| crate::image::pixel(&image, stride, x, y)
}

fn framed(view: impl IntoElement, (width, height): (f32, f32)) -> Element {
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .child(view)
        .into_element()
}

fn clipped(shape: Shape, (width, height): (f32, f32)) -> Element {
    rect()
        .width(Size::px(width))
        .height(Size::px(height))
        .background((255, 0, 0))
        .clip_shape(shape)
        .into_element()
}

#[test]
fn a_filled_circle_leaves_the_corners_of_its_frame_alone() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || framed(Circle::new().fill(ShapeStyle::Color(RED)), size),
        size,
    );
    assert_eq!(pixel(50, 50), RED_PIXEL);
    assert_eq!(pixel(2, 2), WHITE);
}

#[test]
fn a_filled_ellipse_follows_a_frame_that_is_not_square() {
    let size = (100.0, 50.0);
    let pixel = pixels_of(
        move || framed(Ellipse::new().fill(ShapeStyle::Color(RED)), size),
        size,
    );
    assert_eq!(pixel(50, 25), RED_PIXEL);
    assert_eq!(pixel(3, 3), WHITE);
    assert_eq!(pixel(10, 25), RED_PIXEL);
}

#[test]
fn a_filled_path_covers_its_polygon_only() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || {
            framed(
                Path::new()
                    .move_to((10.0, 10.0))
                    .add_line((90.0, 10.0))
                    .add_line((50.0, 90.0))
                    .close_subpath()
                    .fill(ShapeStyle::Color(RED)),
                size,
            )
        },
        size,
    );
    assert_eq!(pixel(50, 30), RED_PIXEL);
    assert_eq!(pixel(15, 80), WHITE);
}

#[test]
fn a_border_stays_inside_the_frame_and_leaves_the_middle_empty() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || {
            framed(
                Circle::new().stroke_border(ShapeStyle::Color(RED), 10.0),
                size,
            )
        },
        size,
    );
    assert_eq!(pixel(2, 50), RED_PIXEL);
    assert_eq!(pixel(50, 2), RED_PIXEL);
    assert_eq!(pixel(50, 50), WHITE);
}

#[test]
fn a_circle_clip_cuts_the_corners_of_a_square_view() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(move || clipped(Shape::Circle, size), size);
    assert_eq!(pixel(50, 50), RED_PIXEL);
    assert_eq!(pixel(2, 2), WHITE);
}

#[test]
fn a_capsule_clip_rounds_the_ends_of_a_wide_view_into_half_circles() {
    let size = (100.0, 40.0);
    let pixel = pixels_of(move || clipped(Shape::Capsule, size), size);
    assert_eq!(pixel(50, 20), RED_PIXEL);
    assert_eq!(pixel(3, 20), RED_PIXEL);
    assert_eq!(pixel(2, 2), WHITE);
}

#[test]
fn a_rounded_rectangle_clip_cuts_only_the_corners() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || {
            clipped(
                Shape::RoundedRectangle {
                    radius: 30.0,
                    style: CornerStyle::Continuous,
                },
                size,
            )
        },
        size,
    );
    assert_eq!(pixel(1, 1), WHITE);
    assert_eq!(pixel(50, 1), RED_PIXEL);
    assert_eq!(pixel(1, 50), RED_PIXEL);
}

#[test]
fn a_gradient_fills_a_shape_and_stops_at_its_edge() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || {
            let gradient = Gradient::new(
                90.0,
                vec![
                    (ShapeStyle::Color(RED), 0.0),
                    (ShapeStyle::Color(Rgba::rgba8(0, 0, 255, 1.0)), 1.0),
                ],
            );
            framed(Circle::new().fill(ShapeStyle::Gradient(gradient)), size)
        },
        size,
    );
    assert!(pixel(30, 50)[0].abs_diff(pixel(70, 50)[0]) > 60);
    assert_eq!(pixel(2, 2), WHITE);
}

#[test]
fn the_border_of_a_continuous_rounded_rectangle_is_one_closed_line() {
    let size = (100.0, 100.0);
    let pixel = pixels_of(
        move || {
            framed(
                RoundedRectangle::new(30.0).stroke_border(ShapeStyle::Color(RED), 10.0),
                size,
            )
        },
        size,
    );
    assert_eq!(pixel(50, 2), RED_PIXEL);
    assert_eq!(pixel(2, 50), RED_PIXEL);
    assert_eq!(pixel(50, 50), WHITE);
}
