use crate::image::{decode, pixel};
use bk::prelude::{Image, TemplateRenderingMode};
use bk::theme::use_appearance;
use ctappearance::ColorScheme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

const RED_SQUARE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="16" fill="#ff0000"/></svg>"##;

#[derive(PartialEq)]
struct Template;

impl Component for Template {
    fn render(&self) -> impl IntoElement {
        Image::data(RED_SQUARE.as_bytes().to_vec())
            .rendering_mode(TemplateRenderingMode::Template)
            .size(16.0)
    }
}

#[derive(PartialEq)]
struct Forced(ColorScheme);

impl Component for Forced {
    fn render(&self) -> impl IntoElement {
        let scheme = self.0;
        use_appearance(move |config| config.color_scheme = scheme);
        rect().child(Template)
    }
}

struct Host(ColorScheme);

impl App for Host {
    fn render(&self) -> impl IntoElement {
        Forced(self.0)
    }
}

fn center(scheme: ColorScheme) -> [u8; 4] {
    let app = AppComponent::new(Host(scheme));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(16.0, 16.0), |_| {}, 1.0);
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let png = test.render();
    let (width, _, rgba) = decode(png.as_bytes());
    pixel(&rgba, width, 8, 8)
}

#[test]
fn a_template_image_paints_its_shape_white_in_dark_and_black_in_light() {
    let [_, green, blue, _] = center(ColorScheme::Dark);
    assert!(green > 200 && blue > 200, "dark: {green} {blue}");
    let [red, ..] = center(ColorScheme::Light);
    assert!(red < 60, "light: {red}");
}
