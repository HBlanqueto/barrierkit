use bk::style::continuous_corner;
use ctappearance::corner;
use ctdesign::{CORNER_SMOOTHING, CORNER_WINDOW};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Size, Size2D};

const SAMPLES: &str = include_str!("../../../ctappearance/tests/data/corner-samples.csv");
const SIZE: f32 = 160.0;

struct Corner(CornerRadius);

impl App for Corner {
    fn render(&self) -> impl IntoElement {
        rect().expanded().background((255, 255, 255)).child(
            rect()
                .width(Size::px(SIZE))
                .height(Size::px(SIZE))
                .corner_radius(self.0)
                .background((0, 0, 0)),
        )
    }
}

fn drawn_coverage(radius: CornerRadius, tile: usize) -> Vec<f32> {
    let side = SIZE + 8.0;
    let (mut test, _) = TestingRunner::new(
        AppComponent::new(Corner(radius)),
        Size2D::new(side, side),
        |_| {},
        1.0,
    );
    test.sync_and_update();
    let (width, _, pixels) = crate::image::decode(test.render().as_bytes());
    (0..tile * tile)
        .map(|index| {
            let (x, y) = (index % tile, index / tile);
            1.0 - f32::from(pixels[(y * width + x) * 4]) / 255.0
        })
        .collect()
}

fn rms_and_max(drawn: &[f32], expected: &[f32]) -> (f32, f32) {
    let sum: f32 = drawn
        .iter()
        .zip(expected)
        .map(|(a, b)| (a - b) * (a - b))
        .sum();
    let max = drawn
        .iter()
        .zip(expected)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
    ((sum / drawn.len() as f32).sqrt(), max)
}

fn sample(name: &str) -> (usize, Vec<f32>) {
    let line = SAMPLES
        .lines()
        .find(|line| line.split(',').next() == Some(name))
        .expect("sample");
    let fields: Vec<&str> = line.split(',').skip(1).collect();
    let alpha = fields[3..]
        .iter()
        .map(|value| value.parse::<f32>().unwrap() / 255.0)
        .collect();
    (fields[0].parse().unwrap(), alpha)
}

#[test]
fn a_continuous_corner_of_the_kit_is_the_reference_corner_within_the_parity_tolerance() {
    let radii = [
        2.5,
        3.5,
        4.0,
        4.5,
        5.0,
        5.5,
        6.0,
        6.5,
        7.0,
        8.0,
        9.0,
        12.0,
        12.5,
        CORNER_WINDOW.fitted,
        26.0,
    ];
    for radius in radii {
        let tile = (radius * (1.0 + CORNER_SMOOTHING)).ceil() as usize + 3;
        let drawn = drawn_coverage(continuous_corner(radius), tile);
        let expected = corner::tile(radius, CORNER_SMOOTHING, tile, 0.0, 0.0);
        let (rms, max) = rms_and_max(&drawn, &expected);
        assert!(
            rms <= 0.02 && max <= 0.2,
            "radius {radius}: rms {rms} max {max}"
        );
    }
}

// Skia's anti-aliasing is not the reference's 16-row box filter: against it the kit's corner has rms <= 0.019 and
// maximum <= 0.142 from radius 2.5 to 26, so the maximum against a capture is the parity check's 0.2 plus that.
#[test]
fn a_continuous_corner_of_the_kit_is_the_corner_macos_draws_on_screen() {
    for (name, radius) in [
        ("layer-continuous-8", 8.0),
        ("layer-continuous-16", 16.0),
        ("layer-continuous-26", 26.0),
        ("window-16", CORNER_WINDOW.fitted),
        ("sheet-16", CORNER_WINDOW.fitted),
    ] {
        let (tile, measured) = sample(name);
        let drawn = drawn_coverage(continuous_corner(radius), tile);
        let (rms, max) = rms_and_max(&drawn, &measured);
        assert!(rms <= 0.02 && max <= 0.25, "{name}: rms {rms} max {max}");
        let old = CornerRadius::new_all(radius).with_smoothing(0.52866);
        let (old_rms, _) = rms_and_max(&drawn_coverage(old, tile), &measured);
        assert!(
            old_rms > 0.02,
            "{name}: the kit's old smoothing, rms {old_rms}"
        );
    }
}
