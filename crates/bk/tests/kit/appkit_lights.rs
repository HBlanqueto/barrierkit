use std::{fs, path::PathBuf};

use bk::theme;
use bk::traffic::TrafficLights;
use ctappearance::traffic::TrafficInput;
use ctappearance::{AppearanceConfig, ColorScheme, TrafficLightStyle, resolve};
use ctdesign::{ButtonKind, Rgba, TITLEBAR};
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Position, Size2D};

const LEFT: f32 = 12.0;
const TOP: f32 = 8.0;
const ROW: f32 = 30.0;

struct Row {
    name: &'static str,
    input: TrafficInput,
    fixed_width: bool,
}

fn input(
    focused: bool,
    group_hovered: bool,
    pressed: Option<ButtonKind>,
    zoom_supported: bool,
) -> TrafficInput {
    TrafficInput {
        focused,
        group_hovered,
        pressed,
        zoom_supported,
        fullscreen: false,
    }
}

fn rows() -> Vec<Row> {
    let row = |name, input, fixed_width| Row {
        name,
        input,
        fixed_width,
    };
    vec![
        row("rest", input(true, false, None, true), false),
        row("hover", input(true, true, None, true), false),
        row(
            "press-close",
            input(true, true, Some(ButtonKind::Close), true),
            false,
        ),
        row(
            "press-min",
            input(true, true, Some(ButtonKind::Minimize), true),
            false,
        ),
        row(
            "press-zoom",
            input(true, true, Some(ButtonKind::Maximize), true),
            false,
        ),
        row("inactive", input(false, false, None, true), false),
        row("inactive-hover", input(false, true, None, true), false),
        row("disabled-zoom", input(true, true, None, false), false),
        row("plus", input(true, true, None, true), true),
        row(
            "fullscreen",
            TrafficInput {
                fullscreen: true,
                ..input(true, true, None, true)
            },
            false,
        ),
    ]
}

struct Scene {
    dark: bool,
    background: (u8, u8, u8),
    rows: Vec<Row>,
}

impl App for Scene {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let mut root = rect().expanded().background(self.background);
        for (index, row) in self.rows.iter().enumerate() {
            root = root.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(LEFT)
                            .top(TOP + index as f32 * ROW),
                    )
                    .child(
                        TrafficLights::new()
                            .input(row.input)
                            .fixed_width(row.fixed_width),
                    ),
            );
        }
        root
    }
}

fn background(name: &str) -> (u8, u8, u8) {
    std::env::var(format!("APPKIT_BG_{}", name.replace('-', "_")))
        .ok()
        .and_then(|hex| u32::from_str_radix(&hex, 16).ok())
        .map_or((255, 255, 255), |rgb| {
            ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
        })
}

#[test]
#[ignore]
fn appkit_lights() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    fs::create_dir_all(&out).unwrap();
    let scale: f64 = std::env::var("APPKIT_SCALE")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1.0);
    let tint: Option<(TrafficLightStyle, Option<[f32; 3]>)> =
        std::env::var("APPKIT_LIGHTS_TINT").ok().map(|text| {
            let (style, color) = text.split_once(':').unwrap_or((&text, ""));
            let rgb = u32::from_str_radix(color, 16).ok().map(|rgb| {
                [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8].map(|c| f32::from(c) / 255.0)
            });
            let style = match style {
                "accent" => TrafficLightStyle::Accent,
                "wallpaper" => TrafficLightStyle::Wallpaper,
                _ => TrafficLightStyle::Standard,
            };
            (style, rgb)
        });
    let suffix = std::env::var("APPKIT_LIGHTS_SUFFIX").unwrap_or_default();
    for (tag, dark, contrast) in [
        ("light", false, false),
        ("dark", true, false),
        ("hc-light", false, true),
        ("hc-dark", true, true),
    ] {
        let (style, color) = tint.unwrap_or((TrafficLightStyle::Standard, None));
        let config = AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            contrast,
            traffic_lights: style,
            accent_color: color,
            ..Default::default()
        };
        let dominant = color.map(|[r, g, b]| Rgba { r, g, b, a: 1.0 });
        let resolved = resolve(&config, dominant);
        let rows = rows();
        let height = TOP + ROW * rows.len() as f32 + 8.0;
        let width = LEFT * 2.0
            + 3.0 * f32::from(TITLEBAR.buttons.diameter as u16)
            + 2.0 * f32::from(TITLEBAR.buttons.spacing as u16);
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(Scene {
                dark,
                background: background(tag),
                rows,
            }),
            Size2D::new(width * scale as f32, height * scale as f32),
            move |runner| {
                runner.provide_root_context(move || State::create(resolved));
            },
            scale,
        );
        test.sync_and_update();
        test.sync_and_update();
        fs::write(
            out.join(format!("kit-lights-{tag}{suffix}.png")),
            test.render().as_bytes(),
        )
        .unwrap();
    }
    let layout: String = rows()
        .iter()
        .enumerate()
        .map(|(index, row)| format!("{}\t{LEFT}\t{}\n", row.name, TOP + index as f32 * ROW))
        .collect();
    fs::write(out.join("layout-lights.tsv"), layout).unwrap();
}
