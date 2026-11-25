use bk::prelude::*;
use ctdesign::Rgba;

const TAGS: [&str; 6] = ["Rust", "Freya", "Skia", "Metal", "AppKit", "Torin"];
const FLOW_WIDTH: f32 = 220.0;

#[derive(PartialEq)]
struct Flow {
    spacing: f32,
}

impl Layout for Flow {
    fn place_subviews(&self, proposal: Size2D, subviews: &[Size2D]) -> Placement {
        let (mut x, mut y, mut line, mut widest) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
        let mut origins = Vec::new();
        for subview in subviews {
            if x > 0.0 && x + subview.width > proposal.width {
                (x, y, line) = (0.0, y + line + self.spacing, 0.0);
            }
            origins.push((x, y));
            widest = widest.max(x + subview.width);
            x += subview.width + self.spacing;
            line = line.max(subview.height);
        }
        Placement {
            size: Size2D::new(widest, y + line),
            origins,
        }
    }
}

fn tag(label: &'static str) -> impl IntoElement {
    Text::new(label)
        .frame()
        .padding(SPACE_2)
        .background_style(ShapeStyle::Role(Role::SystemFill))
        .clip_shape(Shape::Capsule)
}

fn toolbar(width: f32) -> impl IntoElement {
    rect().width(Size::px(width)).child(
        ViewThatFits::new(Axis::Horizontal)
            .candidate(
                260.0,
                HStack::new()
                    .child(Button::new("Share"))
                    .child(Button::new("Duplicate"))
                    .child(Button::new("Delete")),
            )
            .candidate(0.0, Button::new("Actions")),
    )
}

fn main() -> Result<(), RunError> {
    Application::new("com.example.layout-layers")
        .window("Layout layers", || {
            let badge = rect()
                .width(Size::px(10.0))
                .height(Size::px(10.0))
                .background_style(ShapeStyle::Color(Rgba::rgba8(255, 59, 48, 1.0)))
                .clip_shape(Shape::Circle);
            let stack = ZStack::new()
                .alignment(ContentAlignment::BOTTOM_LEADING)
                .child(
                    rect()
                        .width(Size::px(160.0))
                        .height(Size::px(60.0))
                        .background_style(ShapeStyle::Accent)
                        .clip_shape(Shape::RoundedRectangle {
                            radius: 10.0,
                            style: CornerStyle::Continuous,
                        }),
                )
                .child(Text::new("Caption over a card").padding(SPACE_2));
            let flow = TAGS.iter().fold(
                LayoutContainer::with_layout(Flow { spacing: SPACE_2 }).width(Size::px(FLOW_WIDTH)),
                |flow, label| flow.child(tag(label)),
            );
            VStack::new()
                .padding(SPACE_4)
                .spacing(SPACE_4)
                .child(stack)
                .child(
                    Image::system(Symbol::Bell)
                        .overlay(ContentAlignment::TOP_TRAILING, badge)
                        .background_view(
                            ContentAlignment::CENTER,
                            rect()
                                .width(Size::px(40.0))
                                .height(Size::px(40.0))
                                .background_style(ShapeStyle::Role(Role::SystemFill))
                                .clip_shape(Shape::Circle),
                        ),
                )
                .child(flow)
                .child(toolbar(320.0))
                .child(toolbar(120.0))
        })
        .run()
}
