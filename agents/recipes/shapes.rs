use bk::prelude::*;
use ctdesign::Rgba;

const SIDE: f32 = 72.0;
const LINE_WIDTH: f32 = 4.0;
const CORNER_RADIUS: f32 = 18.0;

fn accent() -> ShapeStyle {
    ShapeStyle::Accent
}

fn sunset() -> ShapeStyle {
    ShapeStyle::Gradient(Gradient::new(
        45.0,
        vec![
            (ShapeStyle::Color(Rgba::rgba8(255, 149, 0, 1.0)), 0.0),
            (ShapeStyle::Color(Rgba::rgba8(255, 45, 85, 1.0)), 1.0),
        ],
    ))
}

fn framed(view: impl IntoElement) -> Rect {
    rect()
        .width(Size::px(SIDE))
        .height(Size::px(SIDE))
        .child(view)
}

fn row(views: Vec<Element>) -> HStack {
    HStack::new().spacing(SPACE_4).children(views)
}

fn main() -> Result<(), RunError> {
    Application::new("shapes")
        .window("Shapes", || {
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_4)
                .child(row(vec![
                    framed(Rectangle::new().fill(accent())).into_element(),
                    framed(RoundedRectangle::new(CORNER_RADIUS).fill(accent())).into_element(),
                    framed(Capsule::new().fill(accent()))
                        .height(Size::px(SIDE / 2.0))
                        .into_element(),
                    framed(Circle::new().fill(accent())).into_element(),
                    framed(Ellipse::new().fill(accent()))
                        .height(Size::px(SIDE / 2.0))
                        .into_element(),
                    framed(
                        Path::new()
                            .move_to((SIDE / 2.0, 4.0))
                            .add_line((SIDE - 4.0, SIDE - 4.0))
                            .add_curve((4.0, SIDE - 4.0), (SIDE / 2.0, SIDE / 2.0), (4.0, SIDE))
                            .close_subpath()
                            .fill(accent()),
                    )
                    .into_element(),
                ]))
                .child(row(vec![
                    framed(Rectangle::new().stroke_border(accent(), LINE_WIDTH)).into_element(),
                    framed(
                        RoundedRectangle::new(CORNER_RADIUS).stroke_border(accent(), LINE_WIDTH),
                    )
                    .into_element(),
                    framed(Capsule::new().stroke_border(accent(), LINE_WIDTH))
                        .height(Size::px(SIDE / 2.0))
                        .into_element(),
                    framed(Circle::new().stroke_border(accent(), LINE_WIDTH)).into_element(),
                    framed(Ellipse::new().stroke_border(accent(), LINE_WIDTH))
                        .height(Size::px(SIDE / 2.0))
                        .into_element(),
                    framed(
                        Path::new()
                            .move_to((8.0, SIDE - 8.0))
                            .add_curve((SIDE - 8.0, SIDE - 8.0), (8.0, 8.0), (SIDE - 8.0, 8.0))
                            .stroke(accent(), LINE_WIDTH),
                    )
                    .into_element(),
                ]))
                .child(row(vec![
                    framed(Rectangle::new().fill(sunset())).into_element(),
                    framed(rect().expanded().background_style(sunset()).clip_shape(
                        Shape::RoundedRectangle {
                            radius: CORNER_RADIUS,
                            style: CornerStyle::Continuous,
                        },
                    ))
                    .into_element(),
                    framed(
                        rect()
                            .expanded()
                            .background_style(sunset())
                            .clip_shape(Shape::Capsule),
                    )
                    .height(Size::px(SIDE / 2.0))
                    .into_element(),
                    framed(
                        rect()
                            .expanded()
                            .background_style(sunset())
                            .clip_shape(Shape::Circle),
                    )
                    .into_element(),
                ]))
        })
        .run()
}
