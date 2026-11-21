use std::{collections::HashMap, fs, path::PathBuf};

use bk::adornments::Divider;
use bk::buttons::HelpLink;
use bk::choices::RadioGroup;
use bk::color_picker::ColorPicker;
use bk::combo_box::ComboBox;
use bk::compact::Stepper;
use bk::controls::{
    Button, Checkbox, CircularSlider, ControlSize, ScrollView, SearchField, SecureField,
    SegmentedControl, Slider, Switch, TextField,
};
use bk::date::{CivilDate, CivilDateTime};
use bk::date_picker::DatePicker;
use bk::indicator::{CapacityIndicator, RatingIndicator, RelevancyIndicator};
use bk::menu::{CheckState, MenuItem, MenuView};
use bk::password::PasswordField;
use bk::popup_button::PopUpButton;
use bk::progress::{ProgressRing, ProgressView};
use bk::sidebar::{Sidebar, SidebarItem, SidebarSection};
use bk::styles::{ButtonStyle, GroupBoxStyle};
use bk::symbol::Symbol;
use bk::theme;
use bk::titlebar::Titlebar;
use bk::token_field::TokenField;
use bk::views::group::{DisclosureGroup, GroupBox};
use bk::views::split::{SidebarToggle, SplitView};
use bk::views::table::{SortOrder, Table};
use ctappearance::{AppearanceConfig, ColorScheme, resolve};
use ctdesign::{Rgba, SANS_FAMILIES};
use freya_components::theming::hooks::use_init_theme;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use torin::prelude::{Direction, Gaps, Position, Size, Size2D};

pub(super) type Builder = fn() -> Element;

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

pub(super) fn system_blue() -> Rgba {
    let resolved = bk::theme::resolved();
    match (resolved.scheme == ColorScheme::Light, resolved.contrast) {
        (true, false) => Rgba::rgb8(0, 136, 255),
        (false, false) => Rgba::rgb8(0, 145, 255),
        (true, true) => Rgba::rgb8(30, 110, 244),
        (false, true) => Rgba::rgb8(92, 184, 255),
    }
}

pub(super) fn state_string() -> State<String> {
    use_state(String::new)
}

pub(super) fn controls() -> Vec<(&'static str, Builder)> {
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
        ("slider-circular", || {
            rect()
                .margin(Gaps::new(0.0, 0.0, 0.0, 38.0))
                .child(CircularSlider::new("Dial").value(0.25))
                .into_element()
        }),
        ("pulldown", || {
            PopUpButton::new("Pull")
                .item("Pull")
                .item("A")
                .pulls_down(true)
                .into_element()
        }),
        ("disclosure", || DisclosureGroup::new("").into_element()),
        ("check-mixed", || {
            Checkbox::new("Mixed").mixed(true).into_element()
        }),
        ("check-on-disabled", || {
            Checkbox::new("Off")
                .checked(true)
                .disabled(true)
                .into_element()
        }),
        ("default-disabled", || {
            Button::new("Default")
                .style(ButtonStyle::bordered_prominent)
                .disabled(true)
                .into_element()
        }),
        ("check-off-disabled", || {
            Checkbox::new("Off").disabled(true).into_element()
        }),
        ("radio-off-disabled", || {
            RadioGroup::new("Radio")
                .option("Off")
                .selected(1)
                .disabled(true)
                .into_element()
        }),
        ("radio-on-disabled", || {
            RadioGroup::new("Radio")
                .option("Off")
                .selected(0)
                .disabled(true)
                .into_element()
        }),
        ("switch-off-disabled", || {
            Switch::new("Switch").disabled(true).into_element()
        }),
        ("slider-disabled", || {
            Slider::new("Slider")
                .value(0.5)
                .disabled(true)
                .style(|mut s| {
                    s.width = 160.0;
                    s
                })
                .into_element()
        }),
        ("popup-disabled", || {
            PopUpButton::new("Option")
                .item("Option")
                .item("Other")
                .disabled(true)
                .into_element()
        }),
        ("colorwell-blue", || {
            ColorPicker::new("Color", system_blue()).into_element()
        }),
        ("segmented-disabled", || {
            rect()
                .width(Size::px(158.0))
                .child(
                    SegmentedControl::new("Seg")
                        .segment("One")
                        .segment("Two")
                        .segment("Three")
                        .selected(0)
                        .disabled(true),
                )
                .into_element()
        }),
        ("stepper-disabled", || {
            Stepper::new("Stepper", 0, 10)
                .value(5)
                .disabled(true)
                .style(|mut s| {
                    s.value_position = None;
                    s
                })
                .into_element()
        }),
        ("combo-disabled", || {
            ComboBox::new(use_state(|| "Combo".to_string()), "Combo")
                .item("One")
                .width(Size::px(140.0))
                .disabled(true)
                .into_element()
        }),
        ("circular", || {
            rect()
                .margin(Gaps::new(0.0, 0.0, 0.0, 3.0))
                .child(Button::system_image("", Symbol::Plus).style(ButtonStyle::circular))
                .into_element()
        }),
        ("slider-ticks", || {
            Slider::new("Ticks")
                .value(0.5)
                .number_of_tick_marks(5)
                .style(|mut s| {
                    s.width = 160.0;
                    s
                })
                .into_element()
        }),
        ("colorwell", || {
            ColorPicker::new("Color", Rgba::rgb8(255, 255, 255)).into_element()
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
        ("textfield-filled", || {
            TextField::new(use_state(|| "Query".to_string()), "Placeholder")
                .width(Size::px(140.0))
                .into_element()
        }),
        ("search-filled", || {
            SearchField::new(use_state(|| "Query".to_string()), "Search")
                .width(Size::px(160.0))
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
            Stepper::new("Stepper", 0, 10)
                .value(5)
                .style(|mut s| {
                    s.value_position = None;
                    s
                })
                .into_element()
        }),
        ("ring-regular", || {
            ProgressRing::new("Ring", 0.5).into_element()
        }),
        ("datepicker", || {
            rect()
                .margin(Gaps::new(3.0, 0.0, 0.0, 0.0))
                .child(DatePicker::new(
                    "Date",
                    CivilDateTime::new(CivilDate::new(2027, 1, 15), 8, 0),
                ))
                .into_element()
        }),
        ("calendar", || {
            DatePicker::new(
                "Date",
                CivilDateTime::new(CivilDate::new(2027, 1, 15), 8, 0),
            )
            .graphical()
            .into_element()
        }),
        ("level-warn", || {
            rect()
                .width(Size::px(120.0))
                .child(CapacityIndicator::new("Level", 0.8).thresholds(0.7, 0.9))
                .into_element()
        }),
        ("level-crit", || {
            rect()
                .width(Size::px(120.0))
                .child(CapacityIndicator::new("Level", 1.0).thresholds(0.7, 0.9))
                .into_element()
        }),
        ("discrete-5", || {
            rect()
                .width(Size::px(120.0))
                .child(
                    CapacityIndicator::new("Level", 0.5)
                        .discrete(10)
                        .thresholds(0.7, 0.9),
                )
                .into_element()
        }),
        ("discrete-10", || {
            rect()
                .width(Size::px(120.0))
                .child(
                    CapacityIndicator::new("Level", 1.0)
                        .discrete(10)
                        .thresholds(0.7, 0.9),
                )
                .into_element()
        }),
        ("rating-3", || {
            RatingIndicator::new("Rating", 3).into_element()
        }),
        ("relevancy-3", || {
            rect()
                .width(Size::px(100.0))
                .child(RelevancyIndicator::new("Relevancy", 0.6))
                .into_element()
        }),
        ("secure", || {
            SecureField::new(
                use_state(|| {
                    let mut field = PasswordField::default();
                    "secret".chars().for_each(|c| field.insert(c));
                    field
                }),
                "Secure",
            )
            .width(Size::px(140.0))
            .into_element()
        }),
        ("token", || {
            TokenField::new(
                use_state(|| vec!["One".to_string(), "Two".to_string()]),
                "Tokens",
            )
            .width(Size::px(160.0))
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
                .child(
                    GroupBox::new(Some("Box"))
                        .style(|style| GroupBoxStyle {
                            label_inset: 10.75,
                            label_height: 12.0,
                            ..style
                        })
                        .child(rect().height(Size::px(28.0))),
                )
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
                        .sort(Some(SortOrder {
                            column: 0,
                            ascending: true,
                        }))
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
                .height(Size::px(20.0))
                .center()
                .child(ProgressView::determinate("Progress", 0.5))
                .into_element()
        }),
    ]
}

pub(super) fn layout(path: &str) -> HashMap<String, (f32, f32, f32, f32)> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
        .lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let name = f.next()?.to_string();
            let x = f.next()?.parse().ok()?;
            let y = f.next()?.parse().ok()?;
            let w = f.next()?.parse().ok()?;
            let h = f.next()?.parse().ok()?;
            Some((name, (x, y, w, h)))
        })
        .collect()
}

pub(super) fn render_scene(
    placed: Vec<(f32, f32, Builder)>,
    dark: bool,
    black: bool,
    content: (f32, f32),
    press: Option<(f32, f32)>,
) -> Vec<u8> {
    let scale = scene_scale();
    let resolved = resolve(
        &AppearanceConfig {
            color_scheme: if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            },
            reduce_transparency: std::env::var_os("APPKIT_REDUCE").is_some(),
            contrast: std::env::var_os("APPKIT_CONTRAST").is_some(),
            accent: std::env::var("APPKIT_ACCENT").ok(),
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
        Size2D::new(content.0 * scale as f32, content.1 * scale as f32),
        move |runner| {
            runner.provide_root_context(move || State::create(resolved));
        },
        scale,
    );
    test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
    test.sync_and_update();
    test.sync_and_update();
    if let Some((x, y)) = press {
        test.move_cursor((f64::from(x) * scale, f64::from(y) * scale));
        test.press_cursor((f64::from(x) * scale, f64::from(y) * scale));
        test.sync_and_update();
        test.sync_and_update();
    }
    test.render().as_bytes().to_vec()
}

fn scene_scale() -> f64 {
    std::env::var("APPKIT_SCALE")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(4.0)
}

pub(super) fn scene_content() -> (f32, f32) {
    std::env::var("APPKIT_CONTENT")
        .ok()
        .and_then(|text| {
            let (w, h) = text.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or((1600.0, 768.0))
}

pub(super) fn scene_tag() -> String {
    std::env::var("APPKIT_TAG")
        .ok()
        .filter(|tag| !tag.is_empty())
        .map_or(String::new(), |tag| format!("-{tag}"))
}

#[test]
#[ignore]
fn appkit_scene() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        for black in [false, true] {
            let placed = controls()
                .into_iter()
                .filter_map(|(name, build)| positions.get(name).map(|&(x, y, ..)| (x, y, build)))
                .collect();
            let name = format!(
                "kit{}-{}-{}.png",
                scene_tag(),
                if dark { "dark" } else { "light" },
                if black { "black" } else { "white" }
            );
            fs::write(
                out.join(name),
                render_scene(placed, dark, black, scene_content(), None),
            )
            .unwrap();
        }
    }
}

fn pressed() -> Vec<(&'static str, Builder)> {
    let all = controls();
    [
        "push-pressed",
        "default-pressed",
        "check-off-pressed",
        "check-on-pressed",
        "radio-off-pressed",
        "radio-on-pressed",
        "popup-pressed",
        "calendar-focused",
    ]
    .into_iter()
    .filter_map(|name| {
        let base = match name {
            "calendar-focused" => "calendar".to_string(),
            _ => name.replace("-pressed", "-regular"),
        };
        all.iter()
            .find(|(candidate, _)| *candidate == base)
            .map(|&(_, build)| (name, build))
    })
    .collect()
}

#[test]
#[ignore]
fn appkit_pressed() {
    let layout_path = std::env::var("APPKIT_LAYOUT").expect("APPKIT_LAYOUT");
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    let positions = layout(&layout_path);
    fs::create_dir_all(&out).unwrap();
    for (name, build) in pressed() {
        let Some(&(x, y, w, h)) = positions.get(name) else {
            continue;
        };
        for dark in [false, true] {
            for black in [false, true] {
                let png = render_scene(
                    vec![(x, y, build)],
                    dark,
                    black,
                    scene_content(),
                    Some((x + w / 2.0, y + h / 2.0)),
                );
                fs::write(
                    out.join(format!(
                        "kit{}-pressed-{name}-{}-{}.png",
                        scene_tag(),
                        if dark { "dark" } else { "light" },
                        if black { "black" } else { "white" }
                    )),
                    png,
                )
                .unwrap();
            }
        }
    }
}

const TOOLBAR_CELL: f32 = 44.0;

fn toolbar_glyphs() -> Vec<(&'static str, Builder)> {
    fn glyph(symbol: Symbol) -> Element {
        use bk::compact::IconButton;
        use bk::controls::ToolGroup;
        use bk::styles::{ShapeStyle, StateValue, ToolGroupStyle};
        ToolGroup::new()
            .style(|style: ToolGroupStyle| ToolGroupStyle {
                background: StateValue::new(ShapeStyle::Clear),
                separator: None,
                border: None,
                ..style
            })
            .child(IconButton::new(symbol, "glyph"))
            .into_element()
    }
    vec![
        ("chevron.left", || glyph(Symbol::ChevronLeft)),
        ("chevron.right", || glyph(Symbol::ChevronRight)),
        ("plus", || glyph(Symbol::Plus)),
        ("sidebar.left", || glyph(Symbol::SidebarLeft)),
        ("ellipsis", || glyph(Symbol::Ellipsis)),
        ("magnifyingglass", || glyph(Symbol::MagnifyingGlass)),
    ]
}

#[test]
#[ignore]
fn appkit_toolbar() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    fs::create_dir_all(&out).unwrap();
    let glyphs = toolbar_glyphs();
    let width = glyphs.len() as f32 * TOOLBAR_CELL;
    for dark in [false, true] {
        let placed = glyphs
            .iter()
            .enumerate()
            .map(|(i, &(_, build))| (i as f32 * TOOLBAR_CELL + 4.0, 4.0, build))
            .collect();
        fs::write(
            out.join(format!(
                "kit-toolbar-{}.png",
                if dark { "dark" } else { "light" }
            )),
            render_scene(placed, dark, dark, (width, 44.0), None),
        )
        .unwrap();
    }
}

struct BarScene {
    dark: bool,
    sidebar: bool,
}

impl App for BarScene {
    fn render(&self) -> impl IntoElement {
        use bk::views::navigation::{History, NavigationButtons};
        use_init_theme(|| theme::for_scheme(self.dark));
        let history = use_state(|| {
            let mut history = History::new("a");
            history.push("b");
            history.push("c");
            history.back();
            history
        });
        rect()
            .expanded()
            .background(if self.dark {
                (0, 0, 0)
            } else {
                (255, 255, 255)
            })
            .child(
                bk::titlebar::Titlebar::new()
                    .with_sidebar(self.sidebar)
                    .left(NavigationButtons::new(history)),
            )
    }
}

#[test]
#[ignore]
fn appkit_toolbar_bars() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    fs::create_dir_all(&out).unwrap();
    let scale = scene_scale();
    for (bar, sidebar, height, platter) in [
        ("compact", false, 40.0, 24.0),
        ("unified", true, 52.0, 36.0),
    ] {
        for dark in [false, true] {
            let resolved = resolve(
                &AppearanceConfig {
                    color_scheme: if dark {
                        ColorScheme::Dark
                    } else {
                        ColorScheme::Light
                    },
                    ..Default::default()
                },
                None,
            );
            let (mut test, _) = TestingRunner::new(
                AppComponent::new(BarScene { dark, sidebar }),
                Size2D::new(400.0 * scale as f32, height * scale as f32),
                move |runner| {
                    runner.provide_root_context(move || State::create(resolved));
                },
                scale,
            );
            test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
            test.sync_and_update();
            test.sync_and_update();
            let pair = test
                .find_many(|node, _| {
                    let area = node.layout().area;
                    (area.height() == platter * scale as f32
                        && area.width() > platter * scale as f32 * 2.0)
                        .then_some(area)
                })
                .into_iter()
                .min_by(|a, b| a.width().total_cmp(&b.width()))
                .expect("the navigation pair");
            let name = format!("kit-bar-{bar}-{}", if dark { "dark" } else { "light" });
            fs::write(out.join(format!("{name}.png")), test.render().as_bytes()).unwrap();
            fs::write(
                out.join(format!("{name}.json")),
                format!(
                    "{{\"x\": {}, \"y\": {}, \"w\": {}, \"h\": {}}}",
                    pair.min_x() / scale as f32,
                    pair.min_y() / scale as f32,
                    pair.width() / scale as f32,
                    pair.height() / scale as f32
                ),
            )
            .unwrap();
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

struct GlyphLab(bk::symbol::Symbol);

const LAB_SIZES: usize = 21;
const LAB_STROKES: usize = 17;
const LAB_CELL: f32 = 24.0;

fn lab_step() -> f32 {
    std::env::var("APPKIT_LAB_STEP")
        .ok()
        .and_then(|step| step.parse().ok())
        .unwrap_or(0.5)
}

impl App for GlyphLab {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(false));
        let mut root = rect().expanded().background((255, 255, 255));
        for row in 0..LAB_STROKES {
            for column in 0..LAB_SIZES {
                let size = 8.0 + column as f32 * lab_step();
                let stroke = (1.0 + row as f32 * 0.1) * 16.0 / size;
                root = root.child(
                    rect()
                        .position(
                            Position::new_absolute()
                                .left(column as f32 * LAB_CELL)
                                .top(row as f32 * LAB_CELL),
                        )
                        .width(Size::px(LAB_CELL))
                        .height(Size::px(LAB_CELL))
                        .center()
                        .child(
                            bk::symbol::Image::system(self.0)
                                .color(ctdesign::Rgba::rgb8(0, 0, 0))
                                .style(move |s| bk::styles::ImageStyle { size, stroke, ..s }),
                        ),
                );
            }
        }
        root
    }
}

#[test]
#[ignore]
fn appkit_glyph_lab() {
    use bk::symbol::Symbol;
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    for (name, symbol) in [
        ("chevron-up-down", Symbol::ChevronUpDown),
        ("chevron-down", Symbol::ChevronDown),
        ("chevron-up", Symbol::ChevronUp),
        ("chevron-right", Symbol::ChevronRight),
        ("question-mark", Symbol::QuestionMark),
        ("magnifying-glass", Symbol::MagnifyingGlass),
        ("x-mark", Symbol::XMark),
        ("plus", Symbol::Plus),
        ("minus", Symbol::Minus),
    ] {
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(GlyphLab(symbol)),
            Size2D::new(
                LAB_SIZES as f32 * LAB_CELL * 4.0,
                LAB_STROKES as f32 * LAB_CELL * 4.0,
            ),
            |_| {},
            4.0,
        );
        test.sync_and_update();
        test.sync_and_update();
        fs::write(
            out.join(format!("lab-{name}.png")),
            test.render().as_bytes(),
        )
        .unwrap();
    }
}

const SIDEBAR_WINDOW: (f32, f32) = (900.0, 560.0);

struct SidebarWindow {
    dark: bool,
}

impl App for SidebarWindow {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        let item = |title: &'static str, symbol: Symbol| {
            SidebarItem::new(title)
                .symbol(symbol, Rgba::rgb8(0, 122, 255))
                .selected(title == "Recents")
        };
        let sidebar = Sidebar::new()
            .traffic_lights(true)
            .child(
                SidebarSection::new("Favorites")
                    .child(item("AirDrop", Symbol::House))
                    .child(item("Recents", Symbol::Clock))
                    .child(item("Applications", Symbol::SquareGrid))
                    .child(item("Documents", Symbol::Document)),
            )
            .child(
                SidebarSection::new("Locations")
                    .child(item("iCloud Drive", Symbol::Folder))
                    .child(item("Network", Symbol::Network)),
            );
        let detail = rect().expanded().direction(Direction::Vertical).child(
            Titlebar::new()
                .with_sidebar(true)
                .title("sidebar")
                .left(SidebarToggle)
                .right(bk::compact::IconButton::new(Symbol::Plus, "Plus")),
        );
        SplitView::new(sidebar, detail)
    }
}

#[test]
#[ignore]
fn appkit_sidebar() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        let resolved = resolve(
            &AppearanceConfig {
                color_scheme: if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                reduced_motion: true,
                traffic_lights: ctappearance::TrafficLightStyle::Standard,
                scroll_bars: ctappearance::ScrollBars::Always,
                ..Default::default()
            },
            None,
        );
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(SidebarWindow { dark }),
            Size2D::new(SIDEBAR_WINDOW.0, SIDEBAR_WINDOW.1),
            move |runner| {
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
        test.sync_and_update();
        test.sync_and_update();
        fs::write(
            out.join(format!(
                "kit-sidebar-{}.png",
                if dark { "dark" } else { "light" }
            )),
            test.render().as_bytes(),
        )
        .unwrap();
    }
}

const PICKER_WINDOW: (f32, f32) = (350.0, 60.0);

struct PickerScene {
    dark: bool,
}

impl App for PickerScene {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| theme::for_scheme(self.dark));
        rect()
            .expanded()
            .background(if self.dark {
                (37, 37, 37)
            } else {
                (247, 247, 247)
            })
            .child(
                rect()
                    .position(Position::new_absolute().left(23.0).top(14.0))
                    .child(
                        bk::choices::ColorWell::accents("Accent color")
                            .multicolor("Multicolor")
                            .selected(Some(0)),
                    ),
            )
    }
}

#[test]
#[ignore]
fn appkit_picker() {
    let out = PathBuf::from(std::env::var("APPKIT_OUT").expect("APPKIT_OUT"));
    fs::create_dir_all(&out).unwrap();
    for dark in [false, true] {
        let resolved = resolve(
            &AppearanceConfig {
                color_scheme: if dark {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                ..Default::default()
            },
            None,
        );
        let (mut test, _) = TestingRunner::new(
            AppComponent::new(PickerScene { dark }),
            Size2D::new(PICKER_WINDOW.0, PICKER_WINDOW.1),
            move |runner| {
                runner.provide_root_context(move || State::create(resolved));
            },
            1.0,
        );
        test.set_default_fonts(&SANS_FAMILIES.map(std::borrow::Cow::Borrowed));
        test.sync_and_update();
        test.sync_and_update();
        fs::write(
            out.join(format!(
                "kit-picker-{}.png",
                if dark { "dark" } else { "light" }
            )),
            test.render().as_bytes(),
        )
        .unwrap();
    }
}
