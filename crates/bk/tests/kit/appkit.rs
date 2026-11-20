use std::{collections::HashMap, fs, path::PathBuf};

use bk::adornments::Divider;
use bk::buttons::HelpLink;
use bk::choices::RadioGroup;
use bk::combo_box::ComboBox;
use bk::compact::Stepper;
use bk::controls::{
    Button, Checkbox, ControlSize, ScrollView, SearchField, SegmentedControl, Slider, Switch,
    TextField,
};
use bk::date::{CivilDate, CivilDateTime};
use bk::date_picker::DatePicker;
use bk::indicator::CapacityIndicator;
use bk::menu::{CheckState, MenuItem, MenuView};
use bk::popup_button::PopUpButton;
use bk::progress::{ProgressRing, ProgressView};
use bk::sidebar::{SidebarItem, SidebarSection};
use bk::styles::ButtonStyle;
use bk::theme;
use bk::views::group::GroupBox;
use bk::views::table::Table;
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::SANS_FAMILIES;
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Position, Size, Size2D};

type Builder = fn() -> Element;

struct Scene {
    dark: bool,
    black: bool,
    placed: Vec<(f32, f32, Builder)>,
}

impl App for Scene {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let mut root = rect().expanded().background(if self.black {
            (0, 0, 0)
        } else {
            std::env::var("APPKIT_BG")
                .ok()
                .and_then(|hex| u32::from_str_radix(&hex, 16).ok())
                .map_or((255, 255, 255), |rgb| {
                    ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
                })
        });
        for (x, y, build) in &self.placed {
            root = root.child(
                rect()
                    .position(Position::new_absolute().left(*x).top(*y))
                    .child(build()),
            );
        }
        root
    }
}

fn state_string() -> State<String> {
    use_state(String::new)
}

fn controls() -> Vec<(&'static str, Builder)> {
    vec![
        ("push-small", || {
            Button::new("Button")
                .size(ControlSize::Small)
                .into_element()
        }),
        ("push-regular", || Button::new("Button").into_element()),
        ("push-large", || {
            Button::new("Button")
                .size(ControlSize::Large)
                .into_element()
        }),
        ("default-regular", || {
            Button::new("Default")
                .style(ButtonStyle::bordered_prominent)
                .into_element()
        }),
        ("push-disabled", || {
            Button::new("Off").disabled(true).into_element()
        }),
        ("check-off-regular", || {
            Checkbox::new("Check").into_element()
        }),
        ("check-on-regular", || {
            Checkbox::new("Check").checked(true).into_element()
        }),
        ("radio-off-regular", || {
            RadioGroup::new("Radio")
                .option("Radio")
                .selected(1)
                .into_element()
        }),
        ("radio-on-regular", || {
            RadioGroup::new("Radio")
                .option("Radio")
                .selected(0)
                .into_element()
        }),
        ("switch-off-regular", || {
            Switch::new("Switch").into_element()
        }),
        ("switch-on-regular", || {
            Switch::new("Switch").on(true).into_element()
        }),
        ("switch-on-disabled", || {
            Switch::new("Switch").on(true).disabled(true).into_element()
        }),
        ("slider-regular", || {
            Slider::new("Slider")
                .value(0.5)
                .style(|mut s| {
                    s.width = 160.0;
                    s
                })
                .into_element()
        }),
        ("segmented-regular", || {
            rect()
                .width(Size::px(158.0))
                .child(
                    SegmentedControl::new("Seg")
                        .segment("One")
                        .segment("Two")
                        .segment("Three")
                        .selected(0),
                )
                .into_element()
        }),
        ("textfield-regular", || {
            TextField::new(state_string(), "Placeholder")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("search", || {
            SearchField::new(state_string(), "Search")
                .width(Size::px(160.0))
                .into_element()
        }),
        ("popup-regular", || {
            PopUpButton::new("Option")
                .item("Option")
                .item("Other")
                .into_element()
        }),
        ("stepper-regular", || {
            Stepper::new("Stepper", 0, 10).into_element()
        }),
        ("ring-regular", || {
            ProgressRing::new("Ring", 0.5).into_element()
        }),
        ("datepicker", || {
            DatePicker::new(
                "Date",
                CivilDateTime::new(CivilDate::new(2027, 1, 15), 8, 0),
            )
            .into_element()
        }),
        ("level", || {
            rect()
                .width(Size::px(120.0))
                .child(CapacityIndicator::new("Level", 0.6))
                .into_element()
        }),
        ("combo", || {
            ComboBox::new(use_state(|| "Combo".to_string()), "Combo")
                .item("One")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("separator", || {
            rect()
                .width(Size::px(120.0))
                .child(Divider::new())
                .into_element()
        }),
        ("help", || HelpLink::new().into_element()),
        ("box", || {
            rect()
                .width(Size::px(120.0))
                .height(Size::px(50.0))
                .child(GroupBox::new(Some("Box")))
                .into_element()
        }),
        ("table", || {
            let row = |name: &str, kind: &str| vec![name.to_string(), kind.to_string()];
            rect()
                .width(Size::px(320.0))
                .height(Size::px(180.0))
                .child(
                    Table::new("Files", vec!["Name", "Kind"])
                        .rows(vec![
                            row("adr-0047.pdf", "PDF"),
                            row("notas.txt", "Text"),
                            row("portada.png", "Image"),
                            row("datos.csv", "Table"),
                            row("music.mp3", "Audio"),
                            row("video.mov", "Movie"),
                        ])
                        .selected(vec![1]),
                )
                .into_element()
        }),
        ("sidebar", || {
            rect()
                .width(Size::px(220.0))
                .height(Size::px(260.0))
                .child(
                    SidebarSection::new("Favorites")
                        .child(SidebarItem::new("AirDrop"))
                        .child(SidebarItem::new("Recents").selected(true))
                        .child(SidebarItem::new("Applications")),
                )
                .child(
                    SidebarSection::new("Locations")
                        .child(SidebarItem::new("Desktop"))
                        .child(SidebarItem::new("Documents")),
                )
                .into_element()
        }),
        ("scroll-legacy", || {
            ScrollView::new()
                .width(Size::px(120.0))
                .height(Size::px(160.0))
                .children([rect()
                    .width(Size::px(100.0))
                    .height(Size::px(600.0))
                    .into_element()])
                .into_element()
        }),
        ("progress-bar", || {
            rect()
                .width(Size::px(160.0))
                .child(ProgressView::determinate("Progress", 0.5))
                .into_element()
        }),
    ]
}

fn layout(path: &str) -> HashMap<String, (f32, f32)> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
        .lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let name = f.next()?.to_string();
            let x = f.next()?.parse().ok()?;
            let y = f.next()?.parse().ok()?;
            Some((name, (x, y)))
        })
        .collect()
}

#[test]
#[ignore]
fn appkit_scene() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let (content_w, content_h): (f32, f32) = std::env::var("APPKIT_CONTENT")
        .ok()
        .and_then(|text| {
            let (w, h) = text.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or((1600.0, 768.0));
    let scale = 4.0_f64;
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        for black in [false, true] {
            let placed = controls()
                .into_iter()
                .filter_map(|(name, build)| positions.get(name).map(|&(x, y)| (x, y, build)))
                .collect();
            let resolved = resolve(
                &AppearanceConfig {
                    color_scheme: if dark {
                        ColorScheme::Dark
                    } else {
                        ColorScheme::Light
                    },
                    reduce_transparency: std::env::var_os("APPKIT_REDUCE").is_some(),
                    contrast: std::env::var_os("APPKIT_CONTRAST").is_some(),
                    scroll_bars: ctappearance::ScrollBars::Always,
                    ..Default::default()
                },
                None,
            );
            let (mut test, _) = TestingRunner::new(
                AppComponent::new(Scene {
                    dark,
                    black,
                    placed,
                }),
                Size2D::new(content_w * scale as f32, content_h * scale as f32),
                move |runner| {
                    runner.provide_root_context(move || State::create(resolved));
                },
                scale,
            );
            test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
            test.sync_and_update();
            test.sync_and_update();
            let tag = std::env::var("APPKIT_TAG")
                .ok()
                .filter(|tag| !tag.is_empty())
                .map_or(String::new(), |tag| format!("-{tag}"));
            let name = format!(
                "kit{tag}-{}-{}.png",
                if dark { "dark" } else { "light" },
                if black { "black" } else { "white" }
            );
            fs::write(out.join(name), test.render().as_bytes()).unwrap();
        }
    }
}

#[test]
#[ignore]
fn appkit_menu() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let plain = || {
        vec![
            MenuItem::action("new", "New").with_shortcut("\u{2318}N"),
            MenuItem::action("open", "Open\u{2026}").with_shortcut("\u{2318}O"),
            MenuItem::Separator,
            MenuItem::action("close", "Close").with_shortcut("\u{2318}W"),
            MenuItem::action("off", "Disabled").with_enabled(false),
        ]
    };
    let marked = || {
        vec![
            MenuItem::action("new", "New").with_shortcut("\u{2318}N"),
            MenuItem::action("open", "Open").with_shortcut("\u{2318}O"),
            MenuItem::submenu("Recent", vec![MenuItem::action("r1", "Item")]),
            MenuItem::action("checked", "Checked").with_check(CheckState::Checkbox(true)),
            MenuItem::action("mixed", "Mixed").with_check(CheckState::Mixed),
            MenuItem::submenu("Plain", vec![MenuItem::action("r2", "Item")]),
            MenuItem::action("two", "Plain two").with_shortcut("\u{2318}P"),
        ]
    };
    for (tag, items) in [("", plain as fn() -> Vec<MenuItem>), ("marks-", marked)] {
        for dark in [false, true] {
            let palette = if dark {
                ctdesign::MENU_DARK
            } else {
                ctdesign::MENU_LIGHT
            };
            let items = items();
            let size = MenuView::preferred_size(&items, palette);
            let app =
                AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
            let (mut test, _) =
                TestingRunner::new(app, Size2D::new(size.0 * 4.0, size.1 * 4.0), |_| {}, 4.0);
            test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
            test.sync_and_update();
            test.sync_and_update();
            fs::write(
                out.join(format!(
                    "kit-menu-{tag}{}.png",
                    if dark { "dark" } else { "light" }
                )),
                test.render().as_bytes(),
            )
            .unwrap();
            println!("menu {tag}{dark} {size:?}");
        }
    }
}
