//! Golden-image tests for the menu control.

use std::{
    fs,
    path::{Path, PathBuf},
};

use barrier_kit::menu::{CheckState, MenuItem, MenuView};
use barrier_kit::theme::{SurfaceKind, menu_palette};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use freya_testing::TestingRunner;
use freya_testing::prelude::{Code, Key, KeyboardEventName, Modifiers, NamedKey};
use torin::prelude::Size2D;
use vow_appearance::{AppearanceConfig, ColorScheme, Resolved, resolve};
use vow_design::{MENU_DARK, MENU_LIGHT, MenuPalette};

const TOLERANCE: u8 = 3;

fn accent_items() -> Vec<MenuItem> {
    vec![
        MenuItem::Action {
            id: "images".to_string(),
            label: "Imágenes".to_string(),
            shortcut: Some("Ctrl+Shift+G".to_string()),
            enabled: true,
            check: CheckState::None,
            icon: None,
        },
        MenuItem::Action {
            id: "videos".to_string(),
            label: "Vídeos".to_string(),
            shortcut: Some("Ctrl+Shift+Z".to_string()),
            enabled: false,
            check: CheckState::None,
            icon: None,
        },
    ]
}

fn menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::Action {
            id: "open".to_string(),
            label: "Open".to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
        },
        MenuItem::Separator,
        MenuItem::Action {
            id: "close".to_string(),
            label: "Close".to_string(),
            shortcut: Some("Ctrl+W".to_string()),
            enabled: true,
            check: CheckState::None,
            icon: None,
        },
        MenuItem::Separator,
        MenuItem::Action {
            id: "hidden".to_string(),
            label: "Hidden Files".to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Checkbox(true),
            icon: None,
        },
        MenuItem::Action {
            id: "preview".to_string(),
            label: "Preview".to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Checkbox(false),
            icon: None,
        },
        MenuItem::Action {
            id: "list".to_string(),
            label: "List".to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Radio(true),
            icon: None,
        },
        MenuItem::Action {
            id: "icons".to_string(),
            label: "Icons".to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::Radio(false),
            icon: None,
        },
        MenuItem::Separator,
        MenuItem::Action {
            id: "delete".to_string(),
            label: "Delete".to_string(),
            shortcut: None,
            enabled: false,
            check: CheckState::None,
            icon: None,
        },
        MenuItem::Submenu {
            id: None,
            label: "More".to_string(),
            enabled: true,
            icon: None,
            items: vec![MenuItem::Submenu {
                id: None,
                label: "Level 2".to_string(),
                enabled: true,
                icon: None,
                items: vec![MenuItem::Action {
                    id: "deep".to_string(),
                    label: "Deep item".to_string(),
                    shortcut: None,
                    enabled: true,
                    check: CheckState::None,
                    icon: None,
                }],
            }],
        },
    ]
}

fn render(dark: bool) -> Vec<u8> {
    let palette = if dark { MENU_DARK } else { MENU_LIGHT };
    let items = menu_items();
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    test.render().as_ref().to_vec()
}

fn render_accents(dark: bool) -> Vec<u8> {
    let palette = if dark { MENU_DARK } else { MENU_LIGHT };
    let items = accent_items();
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    test.render().as_ref().to_vec()
}

fn golden_path(dark: bool) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!(
            "menu-view-{}.png",
            if dark { "dark" } else { "light" }
        ))
}

fn golden_path_accents(dark: bool) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!(
            "menu-view-accents-{}.png",
            if dark { "dark" } else { "light" }
        ))
}

fn decode_bytes(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buffer[..info.buffer_size()]
            .chunks(3)
            .flat_map(|px| [px[0], px[1], px[2], 255])
            .collect(),
        other => panic!("unexpected PNG colour type {other:?}"),
    };
    (info.width, info.height, rgba)
}

fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    decode_bytes(&bytes)
}

fn assert_right_padding_is_background(png: &[u8], palette: MenuPalette, label: &str) {
    let (width, height, pixels) = decode_bytes(png);
    let padding = palette.padding.ceil() as u32;
    let start = width.saturating_sub(padding);
    let background = palette.bg.to_rgba8();
    let mut offenders = Vec::new();
    for y in 0..height {
        for x in start..width {
            let offset = ((y * width + x) * 4) as usize;
            let pixel = &pixels[offset..offset + 4];
            if pixel
                .iter()
                .zip(&background)
                .any(|(actual, expected)| actual.abs_diff(*expected) > TOLERANCE)
            {
                offenders.push((x, y));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{label}: {} text pixels in the right padding (first at {:?})",
        offenders.len(),
        offenders.first()
    );
}

fn pixel_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    pixels[offset..offset + 4].try_into().unwrap()
}

fn compare(golden: &Path, actual_png: &[u8], dark: bool, name: &str) {
    let actual_path = std::env::temp_dir().join(format!(
        "malaysia-menu-{name}-{}.png",
        if dark { "dark" } else { "light" }
    ));
    fs::write(&actual_path, actual_png).unwrap();
    let (gw, gh, golden_px) = decode(golden);
    let (aw, ah, actual_px) = decode(&actual_path);
    assert_eq!((gw, gh), (aw, ah), "menu image size changed");
    let bad = golden_px
        .iter()
        .zip(&actual_px)
        .filter(|(a, b)| a.abs_diff(**b) > TOLERANCE)
        .count();
    assert_eq!(
        bad,
        0,
        "menu ({}): {bad} channels differ by more than {TOLERANCE}; actual at {actual_path:?}",
        if dark { "dark" } else { "light" }
    );
}

#[test]
fn menu_matches_its_golden_images() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    for dark in [false, true] {
        let path = golden_path(dark);
        let actual = render(dark);
        if update || !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &actual).unwrap();
        } else {
            compare(&path, &actual, dark, "view");
        }
    }
}

#[test]
fn menu_renders_accents_and_long_shortcuts_without_wrapping() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    for dark in [false, true] {
        let path = golden_path_accents(dark);
        let actual = render_accents(dark);
        if update || !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &actual).unwrap();
        } else {
            compare(&path, &actual, dark, "accents");
        }
    }
}

#[test]
fn menu_text_stays_out_of_the_right_padding() {
    for dark in [false, true] {
        let palette = if dark { MENU_DARK } else { MENU_LIGHT };
        let theme = if dark { "dark" } else { "light" };
        assert_right_padding_is_background(&render(dark), palette, theme);
        assert_right_padding_is_background(&render_accents(dark), palette, theme);
    }
}

#[test]
fn open_title_highlight_uses_neutral_hover_not_accent() {
    for dark in [false, true] {
        let palette = if dark { MENU_DARK } else { MENU_LIGHT };
        let theme = if dark { "dark" } else { "light" };
        assert_ne!(
            palette.bar_hover_bg, palette.hover_bg,
            "{theme}: title hover must not reuse the item accent"
        );
    }
}

fn long_label_items() -> Vec<MenuItem> {
    vec![MenuItem::Action {
        id: "long".to_string(),
        label: "a".repeat(120),
        shortcut: Some("Ctrl+Shift+VeryLongShortcut".to_string()),
        enabled: true,
        check: CheckState::None,
        icon: None,
    }]
}

fn render_long(dark: bool) -> Vec<u8> {
    let palette = if dark { MENU_DARK } else { MENU_LIGHT };
    let items = long_label_items();
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    test.render().as_ref().to_vec()
}

#[test]
fn a_120_character_label_is_truncated_in_the_middle_and_keeps_the_shortcut() {
    for dark in [false, true] {
        let palette = if dark { MENU_DARK } else { MENU_LIGHT };
        let items = long_label_items();
        let size = MenuView::preferred_size(&items, palette);
        assert!(
            size.0 <= palette.max_width,
            "{}: menu width {} exceeds max_width {}",
            if dark { "dark" } else { "light" },
            size.0,
            palette.max_width
        );
        assert_right_padding_is_background(&render_long(dark), palette, "long-label");
    }
}

fn action(label: &str) -> MenuItem {
    MenuItem::Action {
        id: format!("id:{label}"),
        label: label.to_string(),
        shortcut: None,
        enabled: true,
        check: CheckState::None,
        icon: None,
    }
}

fn appearance(scheme: ColorScheme, accent: [f32; 3]) -> Resolved {
    resolve(
        &AppearanceConfig {
            color_scheme: scheme,
            accent_color: Some(accent),
            reduce_transparency: true,
            ..Default::default()
        },
        None,
    )
}

#[test]
fn an_open_menu_recolors_when_the_accent_changes() {
    let purple = appearance(ColorScheme::Dark, [0.5, 0.0, 1.0]);
    let blue = appearance(ColorScheme::Dark, [0.0, 0.3, 1.0]);
    let items = vec![action("Open")];
    let palette = menu_palette(&purple, SurfaceKind::LayerShell);
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}));
    let (mut test, mut state) = TestingRunner::new(
        app,
        Size2D::new(size.0, size.1),
        |runner| runner.provide_root_context(|| State::create(purple)),
        1.0,
    );
    test.sync_and_update();

    let x = (palette.padding + palette.item_padding_x / 2.0) as u32;
    let y = (palette.padding + palette.item_height / 2.0) as u32;
    let (width, _, pixels) = decode_bytes(&test.render());
    assert_eq!(
        pixel_at(&pixels, width, x, y),
        palette.hover_bg.to_rgba8(),
        "the highlighted row must start with the purple accent"
    );

    state.set(blue);
    test.sync_and_update();
    let (width, _, pixels) = decode_bytes(&test.render());
    let expected = menu_palette(&blue, SurfaceKind::LayerShell)
        .hover_bg
        .to_rgba8();
    assert_ne!(expected, palette.hover_bg.to_rgba8());
    assert_eq!(
        pixel_at(&pixels, width, x, y),
        expected,
        "an open menu must follow the new accent"
    );
}

#[test]
fn an_open_menu_recolors_when_the_scheme_changes() {
    let light = appearance(ColorScheme::Light, [0.5, 0.0, 1.0]);
    let dark = appearance(ColorScheme::Dark, [0.5, 0.0, 1.0]);
    let items = vec![action("Open")];
    let palette = menu_palette(&light, SurfaceKind::LayerShell);
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}));
    let (mut test, mut state) = TestingRunner::new(
        app,
        Size2D::new(size.0, size.1),
        |runner| runner.provide_root_context(|| State::create(light)),
        1.0,
    );
    test.sync_and_update();

    let x = (size.0 / 2.0) as u32;
    let y = (palette.padding / 2.0) as u32;
    let (width, _, pixels) = decode_bytes(&test.render());
    assert_eq!(
        pixel_at(&pixels, width, x, y),
        palette.bg.to_rgba8(),
        "the panel must start light"
    );

    state.set(dark);
    test.sync_and_update();
    let (width, _, pixels) = decode_bytes(&test.render());
    let expected = menu_palette(&dark, SurfaceKind::LayerShell).bg.to_rgba8();
    assert_ne!(expected, palette.bg.to_rgba8());
    assert_eq!(
        pixel_at(&pixels, width, x, y),
        expected,
        "an open menu must follow the new scheme"
    );
}

fn find_label(test: &TestingRunner, text: &str) -> bool {
    !test
        .find_many(|node, el| {
            Label::try_downcast(el)
                .is_some_and(|label| label.text == text)
                .then_some(node)
        })
        .is_empty()
}

fn key(test: &mut TestingRunner, key: Key, modifiers: Modifiers, name: KeyboardEventName) {
    test.send_event(freya_testing::prelude::PlatformEvent::Keyboard {
        name,
        key,
        code: Code::Unidentified,
        modifiers,
    });
    test.sync_and_update();
}

#[test]
fn a_menu_taller_than_its_popup_scrolls_with_arrows_and_the_wheel() {
    let items: Vec<MenuItem> = (0..40).map(|i| action(&format!("Elemento {i}"))).collect();
    let (width, height) = MenuView::preferred_size(&items, MENU_LIGHT);
    assert!(height > 300.0);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(MENU_LIGHT));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(width, 300.0), |_| {}, 1.0);
    test.sync_and_update();
    assert!(find_label(&test, "▼"), "more items below");
    assert!(!find_label(&test, "▲"), "nothing above at the top");

    test.scroll((width as f64 / 2.0, 150.0), (0.0, -120.0));
    assert!(find_label(&test, "▲"), "the wheel scrolled down");

    key(
        &mut test,
        Key::Named(NamedKey::End),
        Modifiers::default(),
        KeyboardEventName::KeyDown,
    );
    assert!(!find_label(&test, "▼"), "the last row is visible");
}

#[test]
fn alt_underlines_the_mnemonic_letter() {
    let mark = barrier_kit::menu::MNEMONIC_MARK;
    let items = vec![action(&format!("{mark}Archivo"))];
    let size = MenuView::preferred_size(&items, MENU_LIGHT);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(MENU_LIGHT));
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    assert!(find_label(&test, "Archivo"), "plain label without Alt");

    let alt = Key::Named(NamedKey::Alt);
    key(
        &mut test,
        alt.clone(),
        Modifiers::ALT,
        KeyboardEventName::KeyDown,
    );
    assert!(
        !find_label(&test, "Archivo"),
        "Alt switches to the underlined paragraph"
    );
    key(
        &mut test,
        alt,
        Modifiers::default(),
        KeyboardEventName::KeyUp,
    );
    assert!(
        find_label(&test, "Archivo"),
        "releasing Alt hides the underline"
    );
}

#[test]
fn only_plain_space_activates_an_item() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let activated = Rc::new(RefCell::new(Vec::<String>::new()));
    let sink = activated.clone();
    let items = vec![action("Abrir")];
    let size = MenuView::preferred_size(&items, MENU_LIGHT);
    let app = AppComponent::new(
        MenuView::new(items, move |id| sink.borrow_mut().push(id), || {})
            .with_fixed_palette(MENU_LIGHT),
    );
    let (mut test, _) = TestingRunner::new(app, Size2D::new(size.0, size.1), |_| {}, 1.0);
    test.sync_and_update();
    let space = Key::Character(" ".into());
    for modifiers in [Modifiers::CONTROL, Modifiers::ALT] {
        key(
            &mut test,
            space.clone(),
            modifiers,
            KeyboardEventName::KeyDown,
        );
    }
    assert!(activated.borrow().is_empty(), "Ctrl/Alt+Space do nothing");
    key(
        &mut test,
        space,
        Modifiers::default(),
        KeyboardEventName::KeyDown,
    );
    assert_eq!(*activated.borrow(), vec!["id:Abrir".to_string()]);
}
