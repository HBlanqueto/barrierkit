use crate::image::{decode, pixel};
use bk::prelude::{IconStyle, Image, TemplateRenderingMode};
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

#[derive(PartialEq)]
struct Styled(IconStyle);

impl Component for Styled {
    fn render(&self) -> impl IntoElement {
        Image::data(RED_SQUARE.as_bytes().to_vec())
            .icon_style(self.0)
            .size(16.0)
    }
}

#[derive(PartialEq)]
struct Amber(IconStyle, u8);

impl Component for Amber {
    fn render(&self) -> impl IntoElement {
        use_appearance(|config| config.accent_color = Some([0.91, 0.64, 0.24]));
        let ground = self.1;
        rect()
            .background((ground, ground, ground))
            .child(Styled(self.0))
    }
}

#[derive(PartialEq)]
struct Graphite;

impl Component for Graphite {
    fn render(&self) -> impl IntoElement {
        use_appearance(|config| config.icon_color = Some([0.557, 0.557, 0.576]));
        rect()
            .background((255, 255, 255))
            .child(Styled(IconStyle::Tinted))
    }
}

struct GraphiteHost;

impl App for GraphiteHost {
    fn render(&self) -> impl IntoElement {
        Graphite
    }
}

struct StyledHost(IconStyle, u8);

impl App for StyledHost {
    fn render(&self) -> impl IntoElement {
        Amber(self.0, self.1)
    }
}

fn center(scheme: ColorScheme) -> [u8; 4] {
    center_of(Host(scheme))
}

fn center_of(host: impl App + 'static) -> [u8; 4] {
    let app = AppComponent::new(host);
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

#[test]
fn a_graphite_tinted_icon_loses_its_hue() {
    let [red, green, blue, _] = center_of(GraphiteHost);
    assert!(
        red.abs_diff(green) < 6 && green.abs_diff(blue) < 8,
        "{red} {green} {blue}"
    );
}

#[test]
fn a_tinted_icon_keeps_the_hue_of_its_accent() {
    let [red, _, blue, _] = center_of(StyledHost(IconStyle::Tinted, 255));
    assert!(red > blue, "{red} {blue}");
}

#[test]
fn a_clear_icon_lets_the_window_show_through() {
    let [dark, ..] = center_of(StyledHost(IconStyle::Clear, 0));
    let [light, ..] = center_of(StyledHost(IconStyle::Clear, 255));
    assert!(light > dark + 40, "{light} {dark}");
}

const SUN_AND_NIGHT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="11" fill="#ffd34d"/><rect y="11" width="16" height="5" fill="#6a1b9a"/></svg>"##;

#[derive(PartialEq)]
struct Plated(ColorScheme);

impl Component for Plated {
    fn render(&self) -> impl IntoElement {
        let scheme = self.0;
        use_appearance(move |config| {
            config.color_scheme = scheme;
            config.icon_style = IconStyle::Dark;
        });
        rect().child(PlatedIcon)
    }
}

#[derive(PartialEq)]
struct PlatedIcon;

impl Component for PlatedIcon {
    fn render(&self) -> impl IntoElement {
        Image::data(SUN_AND_NIGHT.as_bytes().to_vec())
            .app_icon()
            .size(64.0)
    }
}

struct PlatedHost(ColorScheme);

impl App for PlatedHost {
    fn render(&self) -> impl IntoElement {
        Plated(self.0)
    }
}

fn plated_pixels(scheme: ColorScheme) -> (Vec<u8>, usize) {
    let app = AppComponent::new(PlatedHost(scheme));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(64.0, 64.0), |_| {}, 1.0);
    test.sync_and_update();
    let _ = test.render();
    test.sync_and_update();
    let png = test.render();
    let (width, _, rgba) = decode(png.as_bytes());
    (rgba, width)
}

#[test]
fn a_dark_style_app_icon_gets_the_hue_of_its_darkest_color_in_light_and_graphite_in_dark() {
    let (light, width) = plated_pixels(ColorScheme::Light);
    let [red, green, blue, _] = pixel(&light, width, 9, 32);
    assert!(red > green && blue > green, "plate {red} {green} {blue}");
    let [r, g, b, _] = pixel(&light, width, 32, 28);
    assert!(r > 200 && g > 160 && b < 130, "logo {r} {g} {b}");
    let (dark, width) = plated_pixels(ColorScheme::Dark);
    let [r, g, b, _] = pixel(&dark, width, 9, 32);
    assert!(r.abs_diff(b) < 12 && r < 90, "graphite {r} {g} {b}");
}
