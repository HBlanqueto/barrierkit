use super::*;
use crate::i18n::text;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use ctdesign::{MENU_DARK, MENU_LIGHT, MENU_MARK_TEXT_GAP};
use freya_core::elements::label::LabelElement;
use freya_core::integration::AppComponent;
use freya_testing::TestingRunner;
use torin::prelude::Size2D;

use crate::platform::timer::{TimerQueue, Wake};
use crate::platform::{ObjectId, PopupCommand, PopupHandle, PopupId, Popups};

fn sample() -> Vec<MenuItem> {
    vec![
        MenuItem::action("new", "New Window").with_shortcut("Ctrl+N"),
        MenuItem::Separator,
        MenuItem::action("open", "Open")
            .with_enabled(false)
            .with_check(CheckState::Checkbox(true)),
        MenuItem::submenu("Recent", vec![MenuItem::action("r1", "a.txt")]),
    ]
}

#[test]
fn preferred_size_is_positive() {
    let (w, h) = MenuView::preferred_size(&sample(), MENU_LIGHT);
    assert!(w > 0.0);
    assert!(h > 0.0);
}

#[test]
fn popover_metrics_keep_the_live_menu_colours() {
    use ctappearance::{AppearanceConfig, ColorScheme, TintParts, WindowColorMode, resolve};

    let accent = Rgba {
        r: 0.9,
        g: 0.2,
        b: 0.7,
        a: 1.0,
    };
    let dark = resolve(
        &AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            reduce_transparency: true,
            ..Default::default()
        },
        None,
    );
    let tinted = resolve(
        &AppearanceConfig {
            color_scheme: ColorScheme::Dark,
            accent_color: Some([accent.r, accent.g, accent.b]),
            reduce_transparency: true,
            window_color: WindowColorMode::Accent,
            tint_parts: TintParts {
                bars: true,
                ..TintParts::default()
            },
            ..Default::default()
        },
        None,
    );

    for resolved in [dark, tinted] {
        let expected = menu_palette(&resolved, SurfaceKind::Toplevel);
        let view = MenuView::new(sample(), |_| {}, || {})
            .with_surface(SurfaceKind::Toplevel)
            .with_popover_metrics();
        let actual = view.palette_source.with_resolved(&resolved);
        assert_eq!(actual.bg, expected.bg);
        assert_eq!(actual.separator_color, expected.separator_color);
    }
}

#[test]
fn root_box_follows_the_configured_popup_size() {
    use freya_testing::TestingRunner;
    use torin::prelude::Size2D;

    let items: Vec<MenuItem> = std::iter::once(MenuItem::SearchField {
        placeholder: "Buscar".to_string(),
    })
    .chain((0..3).map(|i| MenuItem::Action {
        id: format!("item-{i}"),
        label: format!("Item {i}"),
        shortcut: None,
        enabled: true,
        check: CheckState::None,
        icon: None,
        badge: None,
    }))
    .collect();
    assert_eq!(MenuView::preferred_size(&items, MENU_LIGHT), (220.0, 116.0));

    let view = MenuView::new(items, |_| {}, || {}).with_fixed_palette(MENU_LIGHT);
    let app = AppComponent::new(view);
    let (mut test, _) = TestingRunner::new(app, Size2D::new(340.0, 300.0), |_| {}, 1.0);
    test.sync_and_update();

    let area = test
        .find(|node, element| {
            let style = element.style();
            (style.background == Fill::Color(theme::color(MENU_LIGHT.bg)))
                .then(|| node.layout().area)
        })
        .expect("menu root box");
    assert_eq!((area.size.width, area.size.height), (340.0, 300.0));
}

#[test]
fn preferred_size_accounts_for_separators() {
    let with_sep = sample();
    let without_sep: Vec<MenuItem> = with_sep
        .iter()
        .filter(|item| !matches!(item, MenuItem::Separator))
        .cloned()
        .collect();
    let (_, h_with) = MenuView::preferred_size(&with_sep, MENU_LIGHT);
    let (_, h_without) = MenuView::preferred_size(&without_sep, MENU_LIGHT);
    assert!(h_with > h_without);
}

#[test]
fn navigation_skips_separators_and_disabled_items() {
    let items = sample();
    assert_eq!(first_selectable(&items), Some(0));
    assert_eq!(next_selectable(&items, Some(0), 1), Some(3));
    assert_eq!(next_selectable(&items, Some(3), 1), Some(0));
    assert_eq!(last_selectable(&items), Some(3));
}

fn action(label: &str) -> MenuItem {
    MenuItem::Action {
        id: format!("id:{label}"),
        label: label.to_string(),
        shortcut: None,
        enabled: true,
        check: CheckState::None,
        icon: None,
        badge: None,
    }
}

#[test]
fn separators_at_the_start_and_end_are_dropped() {
    let mut items = vec![
        MenuItem::Separator,
        action("A"),
        MenuItem::Separator,
        MenuItem::Separator,
        action("B"),
        MenuItem::Separator,
    ];
    normalize_separators(&mut items);
    assert_eq!(
        items.iter().filter_map(MenuItem::label).collect::<Vec<_>>(),
        vec!["A", "B"]
    );
}

#[test]
fn separator_runs_collapse_into_one() {
    let mut items = vec![
        action("A"),
        MenuItem::Separator,
        MenuItem::Separator,
        MenuItem::Separator,
        action("B"),
    ];
    normalize_separators(&mut items);
    assert_eq!(items.len(), 3);
    assert!(matches!(items[1], MenuItem::Separator));
}

#[test]
fn separators_are_normalized_inside_submenus() {
    let mut items = vec![MenuItem::Submenu {
        id: None,
        label: "Sub".to_string(),
        enabled: true,
        items: vec![
            MenuItem::Separator,
            action("A"),
            MenuItem::Separator,
            MenuItem::Separator,
        ],
        icon: None,
    }];
    normalize_separators(&mut items);
    let MenuItem::Submenu { items, .. } = &items[0] else {
        panic!("the submenu survives");
    };
    assert_eq!(
        items.iter().filter_map(MenuItem::label).collect::<Vec<_>>(),
        vec!["A"]
    );
}

#[test]
fn a_menu_of_only_separators_becomes_empty() {
    let mut items = vec![
        MenuItem::Separator,
        MenuItem::Separator,
        MenuItem::Separator,
    ];
    normalize_separators(&mut items);
    assert!(items.is_empty());
}

#[test]
fn an_empty_menu_shows_the_placeholder() {
    let rows = with_empty_hint(&[]);
    assert!(matches!(
        &*rows,
        [MenuItem::Action {
            label,
            enabled: false,
            ..
        }] if label == text!("no-items")
    ));
    assert_eq!(
        MenuView::preferred_size(&[], MENU_LIGHT),
        MenuView::preferred_size(&rows, MENU_LIGHT)
    );
}

#[test]
fn the_placeholder_is_not_selectable_or_activatable() {
    let rows = with_empty_hint(&[]);
    assert!(!rows[0].selectable());
    assert_eq!(first_selectable(&rows), None);
    assert_eq!(next_selectable(&rows, None, 1), None);
    assert_eq!(last_selectable(&rows), None);
    assert_eq!(activate_at(&rows, 0), None);
}

#[test]
fn the_placeholder_is_not_in_the_search_results() {
    let searched = visible_items(&[], &[], &[], 0, 0);
    assert!(
        !searched
            .iter()
            .any(|item| item.label() == Some(text!("no-items")))
    );
}

#[test]
fn live_items_replace_the_placeholder() {
    assert_eq!(with_empty_hint(&[])[0].label(), Some(text!("no-items")));
    assert_eq!(
        with_empty_hint(&[action("Guardar")])[0].label(),
        Some("Guardar")
    );
}

#[test]
fn empty_remote_submenu_shows_a_disabled_loading_row() {
    let loading = submenu_contents(true, &[]);
    assert_eq!(loading.len(), 1);
    assert!(matches!(
        &loading[0],
        MenuItem::Action {
            label,
            enabled: false,
            ..
        } if label == text!("loading")
    ));
    assert!(submenu_contents(false, &[]).is_empty());
}

#[test]
fn activate_at_ignores_disabled_and_submenus() {
    let items = sample();
    assert_eq!(activate_at(&items, 0), Some("new".to_string()));
    assert_eq!(activate_at(&items, 1), None);
    assert_eq!(activate_at(&items, 2), None);
    assert_eq!(activate_at(&items, 3), None);
}

#[test]
fn navigation_skips_headers_and_search_fields() {
    let items = vec![
        MenuItem::SearchField {
            placeholder: "Buscar".to_string(),
        },
        MenuItem::Header {
            label: "Mitades".to_string(),
            count: None,
        },
        MenuItem::action("a", "A"),
        MenuItem::Header {
            label: "Cuartos".to_string(),
            count: None,
        },
        MenuItem::SearchField {
            placeholder: "Filtrar".to_string(),
        },
        MenuItem::submenu("B", Vec::new()),
    ];
    assert_eq!(first_selectable(&items), Some(2));
    assert_eq!(next_selectable(&items, Some(2), 1), Some(5));
    assert_eq!(next_selectable(&items, Some(5), 1), Some(2));
    assert_eq!(last_selectable(&items), Some(5));
}

#[test]
fn icons_widen_the_menu_by_the_leading_column_and_gap() {
    let label = "Configuración avanzada del sistema";
    let built = |icon: Option<MenuIcon>| {
        vec![
            MenuItem::Action {
                id: "a".to_string(),
                label: label.to_string(),
                shortcut: Some("Super+,".to_string()),
                enabled: true,
                check: CheckState::None,
                icon: icon.clone(),
                badge: None,
            },
            MenuItem::Submenu {
                id: None,
                label: "Disposición".to_string(),
                enabled: true,
                items: Vec::new(),
                icon,
            },
        ]
    };
    let plain = MenuView::preferred_size(&built(None), MENU_LIGHT);
    let with_icons = MenuView::preferred_size(&built(Some(MenuIcon::svg("x"))), MENU_LIGHT);
    assert_eq!(
        with_icons.0 - plain.0,
        MENU_LIGHT.leading_width + MENU_MARK_TEXT_GAP,
        "an icon column is only reserved when the menu actually uses one"
    );
}

#[test]
fn left_handed_submenus_keep_their_descendants_on_the_left() {
    assert_eq!(
        submenu_positioner(true),
        (PopupAnchor::TopLeft, PopupGravity::BottomLeft)
    );
    assert_eq!(
        submenu_positioner(false),
        (PopupAnchor::TopRight, PopupGravity::BottomRight)
    );
}

#[test]
fn submenu_anchor_rect_matches_the_row_top() {
    let items = vec![
        MenuItem::action("one", "One"),
        MenuItem::Separator,
        MenuItem::action("two", "Two"),
        MenuItem::Separator,
        MenuItem::submenu("More", vec![MenuItem::action("deep", "Deep")]),
    ];
    let m = MENU_LIGHT;
    let (width, _) = MenuView::preferred_size(&items, m);
    let (x, y, w, h) = submenu_anchor_rect(4, &items, m, width, 0.0);
    let row_top = item_top(4, &items, m) + m.padding;
    assert_eq!(y as f32, row_top);
    assert_eq!(x, 0);
    assert_eq!(w as f32, width);
    assert_eq!(h as f32, m.item_height);
}

#[test]
fn shortcuts_use_macos_glyphs_in_order() {
    assert_eq!(format_shortcut("Ctrl+Shift+G"), "⌃⇧G");
    assert_eq!(format_shortcut("Alt+Inicio"), "⌥Inicio");
    assert_eq!(format_shortcut("Super+L"), "❖L");
    assert_eq!(format_shortcut("Ctrl+Shift+Z"), "⌃⇧Z");
    assert_eq!(format_shortcut("Shift+Ctrl+G"), "⌃⇧G");
    assert_eq!(format_shortcut("Cmd+D"), "❖D");
    assert_eq!(format_shortcut("Alt+Super+Escape"), "⌥❖⎋");
    assert!(!format_shortcut("Command+D").contains('\u{2318}'));
    assert_eq!(format_shortcut("F5"), "F5");
    assert_eq!(format_shortcut("Ctrl+Super+Left"), "⌃❖←");
    assert_eq!(format_shortcut("Ctrl+Shift+Super+Right"), "⌃⇧❖→");
    assert_eq!(format_shortcut("Up"), "↑");
    assert_eq!(format_shortcut("Down"), "↓");
    assert_eq!(format_shortcut("Return"), "↩");
    assert_eq!(format_shortcut("Enter"), "↩");
    assert_eq!(format_shortcut("Ctrl+Tab"), "⌃⇥");
    assert_eq!(format_shortcut("Control+comma"), "⌃,");
    assert_eq!(format_shortcut("Shift+Delete"), "⇧⌦");
    assert_eq!(format_shortcut("BackSpace"), "⌫");
    assert_eq!(format_shortcut("Page_Up"), "⇞");
    assert_eq!(format_shortcut("Ctrl+space"), "⌃␣");
    assert_eq!(format_shortcut("Ctrl+Num+plus"), "⌃+");
    assert_eq!(format_shortcut("Ctrl+bracketleft"), "⌃[");
}

#[test]
fn mark_sizes_fit_the_leading_column() {
    for (name, m) in [("light", MENU_LIGHT), ("dark", MENU_DARK)] {
        assert!(
            (6.0..=8.0).contains(&m.mark_dot),
            "{name}: radio dot {} is outside 6-8 px",
            m.mark_dot
        );
        assert!(
            m.mark_dot <= m.leading_width,
            "{name}: radio dot is wider than its column"
        );
        assert!(
            m.mark_check > 0.0 && m.mark_check <= m.leading_width,
            "{name}: check glyph does not fit its column"
        );
    }
}

#[test]
fn focused_shortcut_and_chevron_keep_4_5_contrast() {
    for (name, m) in [("light", MENU_LIGHT), ("dark", MENU_DARK)] {
        for color in [
            row_fg(true, true, m.shortcut_color, m),
            row_fg(true, true, m.chevron_color, m),
            m.hover_fg,
        ] {
            let ratio = color.contrast_ratio(m.hover_bg);
            assert!(
                ratio >= 4.5,
                "{name}: focused row color has {ratio:.2}:1 contrast on accent"
            );
        }
    }
}

#[test]
fn preferred_size_handles_accents_and_long_shortcuts() {
    let items = vec![
        MenuItem::action("images", "Imágenes").with_shortcut("Ctrl+Shift+G"),
        MenuItem::action("videos", "Vídeos").with_shortcut("Ctrl+Shift+Z"),
    ];
    let m = MENU_LIGHT;
    let (w, h) = MenuView::preferred_size(&items, m);

    let label_w = measure_text("Imágenes", m.font_size);
    let shortcut_w = measure_text(&format_shortcut("Ctrl+Shift+G"), m.font_size);
    let expected_min =
        m.item_padding_x * 2.0 + label_w + m.shortcut_gap + shortcut_w + m.padding * 2.0;
    assert!(
        w >= expected_min,
        "menu width {w} should fit the widest accented label + shortcut ({expected_min})"
    );

    let expected_height = m.item_height * 2.0 + m.padding * 2.0;
    assert!(
        h >= expected_height,
        "menu height {h} should fit two rows ({expected_height})"
    );
}

fn label_x(runner: &TestingRunner, text: &str) -> f32 {
    runner
        .find(|node, element| {
            (element as &dyn Any)
                .downcast_ref::<LabelElement>()
                .filter(|label| label.text == text)
                .map(|_| node.layout().area.origin.x)
        })
        .unwrap_or_else(|| panic!("label '{text}' not found"))
}

fn action_with(label: &str, check: CheckState, icon: Option<MenuIcon>) -> MenuItem {
    MenuItem::Action {
        id: format!("id-{label}"),
        label: label.to_string(),
        shortcut: None,
        enabled: true,
        check,
        badge: None,
        icon,
    }
}

#[test]
fn plain_menu_text_starts_at_item_padding() {
    let items = vec![action("Solo texto")];
    let (runner, _) = make_runner(items);
    let m = MENU_LIGHT;
    let x = label_x(&runner, "Solo texto");
    let expected = m.padding + m.item_padding_x;
    assert!(
        (x - expected).abs() <= 1.0,
        "plain text starts at {x}, expected {expected}"
    );
}

#[test]
fn a_mark_column_starts_the_text_after_the_mark_inset_column_and_gap() {
    let checked = vec![action_with("Solo texto", CheckState::Checkbox(false), None)];
    let (checked_runner, _) = make_runner(checked);
    let diff = label_x(&checked_runner, "Solo texto");
    let m = MENU_LIGHT;
    let expected = m.padding + m.mark_inset + m.leading_width + MENU_MARK_TEXT_GAP;
    assert!(
        (diff - expected).abs() <= 1.0,
        "checkbox adds {diff}, expected {expected}"
    );
}

#[test]
fn icon_shifts_text_by_leading_column_and_gap() {
    let plain = vec![action("Solo texto")];
    let with_icon = vec![action_with(
        "Solo texto",
        CheckState::None,
        Some(MenuIcon::svg("x")),
    )];
    let (plain_runner, _) = make_runner(plain);
    let (icon_runner, _) = make_runner(with_icon);
    let diff = label_x(&icon_runner, "Solo texto") - label_x(&plain_runner, "Solo texto");
    let expected = MENU_LIGHT.leading_width + MENU_MARK_TEXT_GAP;
    assert!(
        (diff - expected).abs() <= 1.0,
        "icon adds {diff}, expected {expected}"
    );
}

#[test]
fn submenu_keeps_its_own_inset() {
    let child = vec![action_with("Hijo", CheckState::Checkbox(false), None)];
    let parent = vec![MenuItem::Submenu {
        id: None,
        label: "Padre".to_string(),
        enabled: true,
        icon: None,
        items: child.clone(),
    }];
    let m = MENU_LIGHT;
    assert_eq!(row_text_inset(&parent, m), m.item_padding_x);
    assert_eq!(
        row_text_inset(&child, m),
        m.mark_inset + m.leading_width + MENU_MARK_TEXT_GAP
    );
}

#[test]
fn an_icon_shifts_only_the_text_of_its_own_row() {
    let items = vec![
        action("Sin icono"),
        action_with("Con icono", CheckState::None, Some(MenuIcon::svg("x"))),
    ];
    let (runner, _) = make_runner(items);
    let m = MENU_LIGHT;
    assert!((label_x(&runner, "Sin icono") - (m.padding + m.item_padding_x)).abs() <= 1.0);
    let shift = label_x(&runner, "Con icono") - label_x(&runner, "Sin icono");
    assert!(
        (shift - (m.leading_width + MENU_MARK_TEXT_GAP)).abs() <= 1.0,
        "the icon row moves its text by {shift}"
    );
}

#[test]
fn menu_without_columns_is_narrower_by_both_leading_columns_and_gaps() {
    let label = "Opción bastante larga del menú sin columnas";
    let plain = vec![action(label)];
    let with_columns = vec![action_with(
        label,
        CheckState::Checkbox(false),
        Some(MenuIcon::svg("x")),
    )];
    let m = MENU_LIGHT;
    let (plain_w, _) = MenuView::preferred_size(&plain, m);
    let (columns_w, _) = MenuView::preferred_size(&with_columns, m);
    assert_eq!(
        columns_w - plain_w,
        m.mark_inset - m.item_padding_x + 2.0 * (m.leading_width + MENU_MARK_TEXT_GAP),
        "plain menu should be narrower by the mark and icon columns plus their gaps"
    );
}

#[test]
fn menu_with_mark_and_icon_insets_text_past_both_columns() {
    let items = vec![action_with(
        "Mixto",
        CheckState::Checkbox(true),
        Some(MenuIcon::svg("x")),
    )];
    let (runner, _) = make_runner(items);
    let m = MENU_LIGHT;
    let x = label_x(&runner, "Mixto");
    let expected = m.padding + m.mark_inset + 2.0 * (m.leading_width + MENU_MARK_TEXT_GAP);
    assert!(
        (x - expected).abs() <= 1.0,
        "two-column text starts at {x}, expected {expected}"
    );
}

#[test]
fn a_checked_row_with_an_icon_keeps_the_mark_and_adds_the_icon_column() {
    let plain = vec![action_with("X", CheckState::Checkbox(true), None)];
    let with_icon = vec![action_with(
        "X",
        CheckState::Checkbox(true),
        Some(MenuIcon::svg("x")),
    )];
    let (plain_runner, _) = make_runner(plain);
    let (icon_runner, _) = make_runner(with_icon);
    let m = MENU_LIGHT;
    let shift = label_x(&icon_runner, "X") - label_x(&plain_runner, "X");
    assert!(
        (shift - (m.leading_width + MENU_MARK_TEXT_GAP)).abs() <= 1.0,
        "the icon column shifts the text by {shift}"
    );
}

fn searchable(label: &str, enabled: bool) -> SearchableEntry {
    SearchableEntry {
        path: "Ventana".to_string(),
        item: MenuItem::Action {
            id: format!("id-{label}"),
            label: label.to_string(),
            shortcut: None,
            enabled,
            check: CheckState::None,
            icon: None,
            badge: None,
        },
        location: None,
        source: SearchSource::Menu,
    }
}

#[test]
fn search_orders_enabled_before_disabled_and_word_start_before_substring() {
    let entries = vec![
        searchable("zzz foo", false),
        searchable("foo bar", true),
        searchable("bar foo", true),
        searchable("afoo", true),
    ];
    let results = filter_search_results("foo", &entries);
    let labels: Vec<&str> = results.iter().map(|e| e.item.label().unwrap()).collect();
    assert_eq!(labels, vec!["foo bar", "bar foo", "afoo", "zzz foo"]);
}

#[test]
fn visible_items_limits_menu_results_to_nine_rows() {
    let items = vec![
        MenuItem::SearchField {
            placeholder: "Buscar".to_string(),
        },
        MenuItem::action("report", "Informar"),
    ];
    let searchable: Vec<SearchableEntry> = (0..24)
        .map(|i| searchable(&format!("result {i}"), true))
        .collect();
    let visible = visible_items(&items, &searchable, &[], 0, 0);
    let menu_header = visible.iter().find(
        |item| matches!(item, MenuItem::Header { label, .. } if label == text!("menu-items")),
    );
    assert!(menu_header.is_some());
    let count = match menu_header {
        Some(MenuItem::Header { count, .. }) => *count,
        _ => None,
    };
    assert_eq!(count, Some(24));
    let menu_result_count = visible
        .iter()
        .filter(|item| {
            matches!(
                item,
                MenuItem::SearchResult {
                    source: SearchSource::Menu,
                    ..
                }
            )
        })
        .count();
    assert_eq!(menu_result_count, MenuView::MAX_MENU_ROWS);
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for pixel in pixels.chunks_mut(4) {
        pixel.copy_from_slice(&[255, 0, 0, 255]);
    }
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }
    out
}

#[test]
fn a_16px_png_icon_is_decoded_scaled_and_cached() {
    let bytes: std::sync::Arc<[u8]> = png(16, 16).into();
    let handle = icon_image(&bytes, 16.0).expect("the PNG decodes");
    assert_eq!(handle.image.width(), 16);
    assert_eq!(handle.image.height(), 16);
    assert_eq!(&handle.bytes[..4], &[255, 0, 0, 255]);

    let again = icon_image(&bytes, 16.0).expect("the cached PNG decodes");
    assert!(handle == again, "the cache returns the same Skia image");

    let too_big = vec![0u8; MenuIcon::MAX_IMAGE_BYTES + 1];
    assert!(MenuIcon::image(too_big).is_none());
    assert!(decode_icon(b"not a png", 16.0).is_none());
}

#[test]
fn an_icon_with_a_side_over_the_limit_is_not_decoded() {
    let side = MenuIcon::MAX_IMAGE_SIDE as u32;
    assert!(decode_icon(&png(side, 1), 16.0).is_some());
    assert!(decode_icon(&png(side + 1, 1), 16.0).is_none());
}

#[test]
fn visible_items_scrolls_to_keep_twelfth_result_visible() {
    let items = vec![MenuItem::SearchField {
        placeholder: "Buscar".to_string(),
    }];
    let searchable: Vec<SearchableEntry> = (0..24)
        .map(|i| searchable(&format!("result {i}"), true))
        .collect();
    let scroll = 11;
    let visible = visible_items(&items, &searchable, &[], scroll, 0);
    let first_label = visible
        .iter()
        .find_map(|item| match item {
            MenuItem::SearchResult { item, .. } => item.label(),
            _ => None,
        })
        .unwrap();
    assert_eq!(first_label, "result 11");
    let menu_result_count = visible
        .iter()
        .filter(|item| {
            matches!(
                item,
                MenuItem::SearchResult {
                    source: SearchSource::Menu,
                    ..
                }
            )
        })
        .count();
    assert_eq!(menu_result_count, MenuView::MAX_MENU_ROWS);
}

fn submenu_test(label: &str) -> MenuItem {
    MenuItem::Submenu {
        id: None,
        label: label.to_string(),
        enabled: true,
        icon: None,
        items: vec![MenuItem::Action {
            id: format!("{label}-child"),
            label: format!("{label} child"),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
            badge: None,
        }],
    }
}

fn drain_commands(queue: &Rc<RefCell<Vec<PopupCommand>>>) -> Vec<PopupCommand> {
    queue.borrow_mut().drain(..).collect()
}

type Opens = Vec<(PopupId, Option<Rc<dyn Fn()>>)>;

fn opens_and_dismissals(commands: &[PopupCommand]) -> (Opens, Vec<PopupId>) {
    let mut opens = Opens::new();
    let mut closes = Vec::new();
    for cmd in commands {
        match cmd {
            PopupCommand::Open { id, spec, .. } => {
                opens.push((*id, spec.on_dismissed.clone()));
            }
            PopupCommand::Close { id } => closes.push(*id),
            PopupCommand::Reposition { .. } => {}
        }
    }
    (opens, closes)
}

type CommandQueue = Rc<RefCell<Vec<PopupCommand>>>;

fn make_runner(items: Vec<MenuItem>) -> (TestingRunner, CommandQueue) {
    make_runner_with(items, None)
}

fn make_runner_with(items: Vec<MenuItem>, timers: Option<Timers>) -> (TestingRunner, CommandQueue) {
    let palette = MENU_LIGHT;
    let size = MenuView::preferred_size(&items, palette);
    let app = AppComponent::new(MenuView::new(items, |_| {}, || {}).with_fixed_palette(palette));
    let owner = ObjectId::null();
    let root_id = PopupId::test(0);
    let (popups, queue) = Popups::for_test(owner, Some(root_id));
    let (mut runner, _) = TestingRunner::new(
        app,
        Size2D::new(size.0, size.1),
        |r| {
            let popups = popups.clone();
            let handle = PopupHandle::for_test(popups.clone(), root_id);
            r.provide_root_context(move || popups.clone());
            r.provide_root_context(move || handle.clone());
            if let Some(timers) = timers {
                r.provide_root_context(move || timers.clone());
            }
        },
        1.0,
    );
    runner.sync_and_update();
    (runner, queue)
}

fn aim_at_open_submenu() -> (TestingRunner, CommandQueue, PopupId, Rc<dyn Fn()>) {
    let tall = MenuItem::submenu("A", vec![action("1"), action("2"), action("3")]);
    let items = vec![tall, submenu_test("B")];
    let m = MENU_LIGHT;
    let (width, _) = MenuView::preferred_size(&items, m);
    let (ping, _) = sctk::reexports::calloop::ping::make_ping().expect("test ping");
    let timers = Timers {
        queue: TimerQueue::default(),
        ping: ping.clone(),
        wake: Wake {
            ping,
            callbacks: Rc::default(),
        },
    };
    let (mut runner, queue) = make_runner_with(items, Some(timers.clone()));
    let row_top = |row: f32| m.padding + m.item_height * row;

    runner.move_cursor([(width / 2.0) as f64, (row_top(0.0) + 1.0) as f64]);
    runner.sync_and_update();
    let (opens, _) = opens_and_dismissals(&drain_commands(&queue));
    let child = opens[0].0;

    let towards_child = [
        [width * 0.75, row_top(1.0) + 1.0],
        [width - m.padding - 2.0, row_top(1.5)],
    ];
    for [x, y] in towards_child {
        runner.move_cursor([x as f64, y as f64]);
        runner.sync_and_update();
    }
    assert!(
        drain_commands(&queue).is_empty(),
        "the submenu stays open on the way to it"
    );
    let (_, fire) = timers.queue.borrow_mut().pop().expect("the intent timer");
    (runner, queue, child, fire)
}

#[test]
fn aiming_at_the_open_submenu_switches_rows_only_when_the_intent_timer_fires() {
    let (_runner, queue, child, fire) = aim_at_open_submenu();
    fire();
    let (opens, closes) = opens_and_dismissals(&drain_commands(&queue));
    assert_eq!(closes, [child]);
    assert_eq!(opens.len(), 1);
}

#[test]
fn the_intent_timer_does_nothing_once_the_menu_is_gone() {
    let (runner, queue, _, fire) = aim_at_open_submenu();
    drop(runner);
    fire();
    assert!(drain_commands(&queue).is_empty());
}

#[test]
fn stale_submenu_dismissal_does_not_orphan_sibling_child() {
    let items = vec![submenu_test("A"), submenu_test("B")];
    let (mut runner, queue) = make_runner(items);
    let palette = MENU_LIGHT;
    let size = MenuView::preferred_size(&[submenu_test("A"), submenu_test("B")], palette);
    let x = size.0 / 2.0;
    let row0_y = palette.padding + palette.item_height / 2.0;
    let row1_y = palette.padding + palette.item_height + palette.item_height / 2.0;

    runner.move_cursor([x as f64, row0_y as f64]);
    runner.sync_and_update();
    let mut commands = drain_commands(&queue);
    let (mut opens, closes) = opens_and_dismissals(&commands);
    assert!(closes.is_empty(), "first open should not close anything");
    assert_eq!(opens.len(), 1, "first hover should open one child");
    let (id0, dismiss0) = opens.remove(0);

    runner.move_cursor([x as f64, row1_y as f64]);
    runner.sync_and_update();
    commands = drain_commands(&queue);
    let (opens, closes) = opens_and_dismissals(&commands);
    assert!(
        closes.contains(&id0),
        "moving to sibling must close old child"
    );
    assert_eq!(opens.len(), 1, "second hover should open one child");
    let (id1, _) = opens.into_iter().next().unwrap();
    assert_ne!(id0, id1, "new child must have a different id");

    if let Some(dismiss) = dismiss0 {
        dismiss();
    }

    runner.move_cursor([x as f64, row0_y as f64]);
    runner.sync_and_update();
    let commands = drain_commands(&queue);
    let (_, closes) = opens_and_dismissals(&commands);
    assert!(
        closes.contains(&id1),
        "stale dismissal must not orphan the sibling submenu"
    );
}

#[test]
fn semibold_text_measures_wider_than_regular() {
    let regular = measure_text("Disco de arranque", 11.0);
    let semibold = measure_text_weight("Disco de arranque", 11.0, FontWeight::SEMI_BOLD);
    assert!(semibold > regular, "{semibold} must exceed {regular}");
}
