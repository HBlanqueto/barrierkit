# 20. Glossary

**Goal:** find the BarrierKit name of a concept you know from SwiftUI,
AppKit or UIKit, and name new items the same way.
**Scope:** every public item of `bk`, `ctdesign` and
`ctappearance`, with its SwiftUI, AppKit and UIKit equivalent and the chapter that
documents it. Some methods have a row; the chapters document the rest. The
names of `bk_shell` and `ctshell` are not in this glossary: those crates left the repository.
**Reader:** anyone who writes an app with the kit, adds an item to it, or is an
AI agent ([`.claude/skills/barrierkit-app/SKILL.md`](../../.claude/skills/barrierkit-app/SKILL.md)).
**Input:** a name. **Output:** its counterpart and the chapter to read.

## Steps

1. Search this page for the SwiftUI, AppKit or UIKit name (for example
   `NSPopUpButton` or `UISwitch`). `Toggle` and `Picker` are one SwiftUI view and several
   controls here: search `ToggleStyle` or `PickerStyle`.
2. Read the BarrierKit name in the first column and follow the chapter link.
   The UIKit column is read from the iOSSupport headers of the macOS 27.2 SDK;
   `—` means UIKit has no counterpart and `≈` an approximate one.
   For SwiftUI syntax that is not a name (`$x`, `@Environment`, `ForEach`,
   `if/else`), read [SwiftUI to the kit](16-swiftui-to-the-kit.md).
3. To add or rename a public item, follow [How names are chosen](#how-names-are-chosen)
   and add its row here in the same commit (`AGENTS.md`, «Names and glossary»).

## How names are chosen

1. **SwiftUI first.** When SwiftUI has the concept, use its name: `Text`,
   `VStack`, `Button`, `TabView`.
2. **Then AppKit.** When only AppKit has it, use the AppKit name without the
   `NS` prefix: `ColorWell`, `PathControl`, `TokenField`.
3. **Then plain English.** Otherwise, one or two common words that say what
   the item is, not how it is built: `Palette`, `Sidebar`.
4. **Rust case.** Types `UpperCamelCase`; functions and methods `snake_case`
   (`foregroundStyle` → `foreground_style`); constants `SCREAMING_SNAKE_CASE`
   (`.title` → `TITLE`).
5. **Fixed shapes.** A constructor is `Type::new(required)`. Options are
   chained methods named after what they set (`.font`, `.size`). Events are
   `on_<verb>` (`on_press`, `on_change`). Booleans read as adjectives
   (`disabled`, `focused`). Hooks are `use_<thing>`.
6. **Taken names.** When Freya already uses a name (`App`, `label`, `rect`),
   take the next clear one (`Application`) and record it here.

## Renamed items

| Before | Now | Date |
|---|---|---|
| `vow_design::TEXT` | `ctdesign::BODY` | 2026-11-07 |
| `vow_design::DISPLAY` | `ctdesign::LARGE_TITLE` | 2026-11-07 |
| `vow_design::TITLE_1` | `ctdesign::TITLE` | 2026-11-07 |
| crate `barrier-kit` (`barrier_kit`) | crate `bk`, BarrierKit | 2026-11-07 |
| crate `vow-design` (`vow_design`), then `ct-design` (`ct_design`) | crate `ctdesign`, CTDesign | 2026-11-07 |
| `ctappearance::write_dominant` | `ctappearance::write_wallpaper_state` | 2026-11-08 |
| `ctdesign::MENU_SEARCH_CLEAR_MARGIN` | `ctdesign::MENU_SEARCH_PADDING_X` | 2026-11-11 |
| crate `vow-appearance` (`vow_appearance`), then `ct-appearance` (`ct_appearance`) | crate `ctappearance`, CTAppearance | 2026-11-07 |
| `styles::Theme::slider` | `styles::Theme::style` (generic: `\|s: SliderStyle\|`) | 2026-11-07 |
| `modifiers::StyleModifiers::slider_style` | `modifiers::StyleModifiers::control_style` (generic) | 2026-11-07 |
| enum `controls::ButtonStyle` | struct `styles::ButtonStyle` | 2026-11-07 |
| `TabView` with the toolbar tabs as its only look (`TabViewStyle::tengen()`) | `TabView` draws the classic strip (`TabType::Top`); the toolbar tabs are `TabViewStyle::toolbar()` | 2026-11-20 |
| `ctdesign::SCROLL_BAR.always_opacity`, `SCROLL_BAR.opacity` (one value) | `SCROLL_BAR.opacity` and `SCROLL_BAR.track_opacity`, four values each (light, dark, then both with Increase Contrast); `ScrollViewStyle.always_thumb_opacity` is gone | 2026-11-20 |
| `ScrollViewStyle.animation` | `ScrollViewStyle.fade_in` and `fade_out`; `SCROLL_BAR.fade_in_ms`, `fade_out_ms` | 2026-11-25 |
| `controls::Button::style(ButtonStyle)` | `controls::Button::style(\|s\| ..)`, the closure of every control | 2026-11-07 |
| `ctdesign::DURATION_MENU_CLOSE_MS`, `DURATION_POPOVER_IN_MS`, `DURATION_POPOVER_OUT_MS`, `SPRING_DAMPING`, `SPRING_ALERT_MS` | Removed: deleted, nothing read them | 2026-11-25 |
| `ctdesign::CORNER_POPOVER`, `WINDOW_CLIENT_RESIZE`, `DIALOG`, `ICON_PLATE_GLASS`, `ICON_PLATE_GLASS_DARK`, `ICON_PLATE_GLASS_DARK_ON_DARK`, `KNOB_HELD_SHADOW`, `KNOB_HELD_SHADOW_DARK`, `THEME_FADE_WAIT_MS`, `TITLEBAR_TITLE_GAP`, `DOCK_MENU_PADDING`, `DOCK_MENU_ITEM_PADDING_X`, `DOCK_MENU_SEPARATOR_MARGIN`, `DOCK_MENU_MIN_WIDTH`, `DOCK_MENU_ARROW_INSET`, `MENU_SUBMENU_REVEAL_GAP`, `MENU_SEARCH_ICON`, `MENU_SEARCH_ICON_GAP`, `MENU_SEARCH_CLEAR`, `MENU_SEARCH_CLEAR_GLYPH`, `MENU_SEARCH_PADDING_X`, `MENU_SEARCH_CLEAR_MIX`, `MENU_HELP_ICON` | Removed: only the shell reads them; they live in `ctshell::design` | 2026-11-25 |
| `menu::MenuView::with_searchable`, `with_help_topics`, `with_search_icons`, `with_reveal_roots`, `with_query_state`, `with_navigation`, `with_fixed_palette`, `with_open_submenu`, `with_submenu_source`; `menu::SearchableEntry`, `SearchSource`, `SearchLocation`, `SearchQuery`, `SearchableSource`, `RevealRoots`, `SubmenuSource`, `MenuNavigation`, `normalize_query`, `search_icon_svg`, `submenu_contents`; `MenuItem::SearchField`, `SearchResult`, `Header.count`; `MenuIcon::image`; `popup_button::pop_up_menu`, `arrow_band`; `symbol::Image::glass_plate`, `raster_size` | Removed: the desktop shell's menu bar search and Dock menu had no app; they live in the shell | 2026-11-25 |
| `SplitViewStyle::resize_animation` | Removed: a resize of the sidebar never animates, as AppKit's `setPosition` | 2026-11-25 |
| `chrome::WindowChrome`, `chrome::Reveal`, `chrome::RestingEdge`, `styles::WindowChromeStyle`, `ctdesign::TITLEBAR_REVEAL`, `TitlebarRevealMetrics` | Removed: the desktop shell's reveal chrome had no app; it lives in the shell | 2026-11-25 |
| `ctappearance::backdrop::Backdrop::accent` | Removed: use `highlight` | 2026-11-25 |
| `sidebar::SidebarControl::content` | Removed: the bar runs over the content column by itself | 2026-11-24 |
| `styles::TabType::Left`, `TabViewStyle.tab_padding_y`, `ctdesign::TAB_VIEW.strip_padding_y` and `.left_strip_top` | Removed: no app used the left strip | 2026-11-23 |
| `Titlebar::right(view)` | `.toolbar(ToolbarItem::new(ToolbarItemPlacement::PrimaryAction, view))`, SwiftUI's only way to add trailing items | 2026-11-23 |
| `chrome::TitlebarLayout` (`Top`, `Rail`), `chrome::RailStyle`, `WindowChrome::rail_top`, `rail_bottom`, `WindowChromeStyle::{layout, rail_style, rail_width, rail_padding_top, rail_padding_bottom, rail_spacing, rail_background, drawer_background, title, rail_line, title_angle, title_font}`, `TrafficLights::vertical`, `TrafficLightsStyle::axis`, `SidebarStyle::animation` | Gone: the vertical title bar had no SwiftUI design and no app | 2026-11-23 |
| `ctdesign::TITLEBAR_RAIL`, `TitlebarRailMetrics` | `TITLEBAR_REVEAL`, `TitlebarRevealMetrics`, without the rail fields | 2026-11-23 |
| `platform::FullscreenBar` (`Solid`, `Glass`, `ProgressiveGlass`), `WindowOptions::fullscreen_bar`, `fullscreen_push`, `WindowControls::fullscreen_bar`, `fullscreen_push`, `FullscreenBarStyle::glass`, `ctdesign::FULLSCREEN_BAND` | Gone: focus mode draws one opaque band, and the window's design decides the bar | 2026-11-23 |
| `Application::new(name)`, `WindowOptions::new(title, app_name)`, `UtilityWindow::new(title, content)`, `WindowOptions::app_id(id)`, `titlebar::app_id`, `ctdesign::NATIVE_APP_ID_PREFIX` | `Application::new(id)`, `WindowOptions::new(title, app_id)`, `UtilityWindow::new(title, app_id, content)` with a reverse-DNS id the app chooses; no prefix of Malaysia | 2026-11-24 |
| `Text::bold()`, `italic()`, `underline()`, `strikethrough()` | `Text::bold(bool)`, `italic(bool)`, `underline(bool)`, `strikethrough(bool)`, SwiftUI's `isActive` | 2026-11-24 |
| `ButtonStyle::Secondary` | `ButtonStyle::tengen()`, the default (SwiftUI `.bordered`) | 2026-11-07 |
| `ButtonStyle::Primary` | `ButtonStyle::bordered_prominent` | 2026-11-07 |
| `ButtonStyle::Glass` | `ButtonStyle::glass` | 2026-11-07 |
| `ButtonStyle::GlassProminent` | `ButtonStyle::glass_prominent` | 2026-11-07 |
| `ButtonStyle::Borderless` | `ButtonStyle::borderless` | 2026-11-07 |
| enum `level::LevelStyle` | struct `styles::LevelSliderStyle` | 2026-11-07 |
| `level::LevelSlider::style(LevelStyle)` | `level::LevelSlider::style(\|s\| ..)` | 2026-11-07 |
| `IconStyle::Monochrome` | `IconStyle::Tinted` with a graphite `icon_color` | 2026-11-14 |
| `LevelStyle::Regular` | `LevelSliderStyle::tengen()`, the default | 2026-11-07 |
| `WallpaperSelection::pinned`, `pinned_for`, `set_pinned` (an image index) | `appearance`, `appearance_for`, `set_appearance` (`WallpaperAppearance`) | 2026-11-11 |
| `LevelStyle::Fat(Symbol)` | `LevelSliderStyle::fat(symbol)` | 2026-11-07 |
| `rich_text::TextStyle` | `rich_text::RunStyle`; `styles::TextStyle` is the style of `Text` | 2026-11-07 |
| `chrome::ChromeStyle` | `styles::WindowChromeStyle`, with the look of the rail | 2026-11-07 |
| `chrome::WindowChrome::new(style, child)` | `chrome::WindowChrome::new(child).style(\|s\| ..)` | 2026-11-07 |
| `shell::SpaceInfo`, `Toplevels::space` | `Toplevels::focus_bar` (`FocusBar`); the `space` event keeps only `focus_bar` (Limitless v7) | 2026-11-08 |
| `platform::Bar::Compact`, `WindowCorner::compact`, Limitless `bar` entry `compact` | `Bar::UnifiedCompact`, `WindowCorner::unified_compact`, `unified_compact` | 2026-11-08 |
| `ctdesign::MENU_GLASS`, `MENU_GLASS_CAP_DARK` and `MENU_GLASS_CHROMA_DARK` (an earlier set), then `MENU_GLASS_VEIL`, `MENU_GLASS_LIFT` and `MENU_GLASS_CAP` (one set for every scheme) | `MENU_GLASS_VEIL_LIGHT`, `MENU_GLASS_VEIL_DARK`, `MENU_GLASS_LIFT_LIGHT`, `MENU_GLASS_LIFT_DARK`, `MENU_GLASS_CAP_LIGHT`, `MENU_GLASS_CAP_DARK`, with `MENU_GLASS_CHROMA` in every scheme; `MENU_GLASS_CAP_DARK` now names the dark cap of the new pair | 2026-11-20 |
| `ctdesign::LOCK_FIELD_MIN_WIDTH`, `LOCK_FIELD_MAX_WIDTH` (the field grew while typing) | `ctdesign::LOCK_FIELD_WIDTH`, one fixed width, a project value (the macOS lock screen was never measured) | 2026-11-11 |
| `ctdesign::LOCK_TOP_MARGIN` (72 px) | `ctdesign::LOCK_TOP_PERCENT`, a share of the screen height | 2026-11-11 |
| `ctdesign::LOCK_FIELD_FILL_OPAQUE` (the field with «Reduce transparency») | `ctdesign::LOCK_AVATAR_FILL`; the field is always `LOCK_FIELD_FILL` | 2026-11-11 |
| The 22 views, modifiers and names renamed by [ADR 0063](../decisions.md) (`divider()`, `empty_state`, `use_tooltip`, `PreferencesWindow`, `AppMenu`, `ProgressBar`…) | `Divider`, `ContentUnavailableView`, `.help`, `SettingsWindow`, `MainMenu`, `ProgressView`… | 2026-11-07 |
| `shell::pixmap_image(pixmap, size)` | `symbol::Image::rgba(width, height, rgba).size(size)` | 2026-11-11 |
| `Slider::stepped(label, steps)` as the only constructor, with `.value(usize)` and `.on_change(\|step: usize\| ..)` | `Slider::new(label)` is the free slider; `Slider::stepped` keeps its steps with `.value(f32)` and `.on_change(\|step: f32\| ..)` | 2026-11-16 |
| `ctappearance::SidebarStyle` (`Island`, `Flat`) and `sidebar_style` in `appearance.domain`: the sidebar was a floating island by default | Gone: the sidebar is flat and full height, as in macOS 27; an old `sidebar_style` key is ignored ([Glass](10-sidebar.md#glass)) | 2026-11-20 |
| `ctdesign::SIDEBAR_ISLAND_LIGHT`, `SIDEBAR_ISLAND_DARK`, `SIDEBAR_ISLAND_DARK_TINT`, `SIDEBAR_ISLAND_RIM_*`, `SIDEBAR_ISLAND_INNER_*`; `SIDEBAR.inset`, `SIDEBAR.radius`, `SIDEBAR.inner_width`; `BACKDROP.island` | Gone with the island; the sidebar background is `SIDEBAR_LIGHT`, `SIDEBAR_DARK`, `SIDEBAR_DARK_TINT` | 2026-11-20 |
| `sidebar::SidebarBackdrop` and `Sidebar::backdrop(..)` (the island over a tinted backdrop) | Gone: the sidebar is always transparent over a tinted backdrop | 2026-11-20 |
| `titlebar::sidebar_bar_metrics()` (height and top padding) and `TitlebarStyle::sidebar_padding_top`, `SETTINGS.toolbar_padding_top` | `titlebar::sidebar_bar_height()` | 2026-11-20 |
| `SplitViewStyle::inset` and `SidebarStyle::{shape, border, inner_border, line, padding_bottom}` (island only) | Gone | 2026-11-20 |
| `SidebarSectionStyle::margin_top` (`SIDEBAR.section_margin_top`, 10, above each section) | `SidebarSectionStyle::margin_bottom` (`SIDEBAR.section_margin_bottom`, 13, between sections) | 2026-11-20 |
| `BUTTON.large_height`, `large_padding_x`, `small_height`, `small_padding_x`, `small_font_size`, `small_radius` | `BUTTON.small`, `BUTTON.large`, `BUTTON.mini`, `BUTTON.xl` (`ButtonSize`), or `BUTTON.size(ControlSize)` | 2026-11-20 |
| `SETTINGS.radio_size`, `radio_dot`, `radio_gap`, `check.gap` | `CHECKBOX.size`, `CHECKBOX.radio_dot`, `CHECKBOX.gap` | 2026-11-20 |
| `SETTINGS.popup_height`, `popup_chevron`, `popup_glyph`, `popup_glyph_stroke`, `popup_pulldown_glyph`, `popup_pulldown_drop`, `popup_label_lift`, `popup_text_gap`, `popup_trailing_padding` | `BUTTON.height` and `BUTTON.popup.regular` (`PopupSize`: `glyph`, `glyph_stroke`, `indicator`, `trailing`, `gap`, `pulldown_glyph`, `pulldown_drop`, `label_lift`) | 2026-11-20 |
| `ctappearance::TrafficLightShape`, the key `traffic_light_shape`, `Resolved::traffic_shape`, `TrafficLights::round`, `WindowChromeStyle::round_lights`, `TrafficLightsStyle::shape`, `TITLEBAR.buttons.corner_radius` | Gone: the lights are discs; a file with the key still loads and the next write drops it | 2026-11-20 |
| `ctappearance::lights_for`, `Resolved::lights`, `Resolved::traffic_style` | `ctappearance::traffic::looks`, `Resolved::traffic_lights` (`TrafficLooks`) | 2026-11-20 |
| `ctappearance::traffic::visual(lights, style, inactive, rim, input)`, `ButtonVisual { fill, rim, glyph }`, `ButtonVisual::idle`, `glyph_svg(.., pixels)` | `ctappearance::traffic::visual(looks, input)`, `ButtonVisual { fill, ring, glyph, scale, dodge }`, `glyph_svg(index, color, fixed_width)` | 2026-11-20 |
| `ctdesign::TrafficRim`, `Palette::traffic_lights_inactive`, `Palette::traffic_rim`, `TRAFFIC_RIM_WIDTH`, `TRAFFIC_PRESSED_DARKEN`, `TRAFFIC_GLYPH_STANDARD`, `TRAFFIC_GLYPH_ON_DARK`, `TRAFFIC_GLYPH_ON_LIGHT`, `Button::color`, `TrafficGlyphs::box_size`, the tokens `traffic-close`, `traffic-min`, `traffic-zoom`, `traffic-inactive`, `traffic-rim`, `traffic-rim-inactive` | `ctdesign::TrafficLooks`, `TRAFFIC_LIGHTS_*`, `TRAFFIC_LIGHTS_CONTRAST_*`, `TRAFFIC_RING_WIDTH`, `TRAFFIC_PRESS`; the themes' `traffic-lights` and `traffic-lights-contrast` | 2026-11-20 |
| `styles::TrafficLightsStyle { fills, rims, glyphs, rim_width, glyph_size, shape, effect }` | `styles::TrafficLightsStyle { looks, ring_width, halo, backdrop }` | 2026-11-20 |
| `app::Scene` (one window; `.options`, `.run`), returned by `Application::window` | `Application::window` returns `Application`, which has `.options` and `.run`; `app::Scene` is now what `Application::scene` takes | 2026-11-21 |
| `titlebar::window_drag_handler` | `titlebar::use_window_drag` (a hook) | 2026-11-23 |
| `password::State` | `password::PasswordState` | 2026-11-23 |
| `SecureFieldStyle` `height`, `padding_x`, `shape`, `background`, `text`, `border`, `opacity`, `effect`, `font`; `SwitchStyle::frame_size`; `layout_direction` of `CheckboxStyle`, `LevelSliderStyle`, `RadioGroupStyle`, `SliderStyle`, `StepperStyle` and `SwitchStyle` | Gone: `SecureField` draws from `TextFieldStyle`, a switch takes its track's size, and the controls read the ambient `.layout_direction(..)` | 2026-11-23 |
| Value setters and change callbacks of controls: `Checkbox::checked`, `Switch::on` and their `on_toggle`; `.value` and `.on_change` of `Slider`, `CircularSlider`, `LevelSlider` and `Stepper`; `.selected` and `.on_change` of `SegmentedControl`, `PopUpButton`, `RadioGroup`, `ChoiceCards` and `ColorWell`; `ToggleGroup::on_toggle`; `on_change` of `DatePicker`, `ColorPicker`, `FontPicker`, `RatingIndicator` and `TextView`; `Row::toggle(title, on, on_toggle)` | The control takes the `State<T>` it edits in `new`, like SwiftUI's binding: `Checkbox::new(label, state)`, `Slider::stepped(label, steps, state)`, `Stepper::new(label, state, min, max)`, `ColorWell::accents(label, state)`, `ToggleGroup::item(.., State<bool>)`, `Row::toggle(title, state)`; `RatingIndicator::editable(bool)` | 2026-11-24 |
| `selected` with `on_select` or `on_change` of `TabView`, `List`, `Table`, `OutlineView` and `CollectionView`; `Browser::on_select`; `Table::sort` and `on_sort` | `.selection(State<..>)`; `Table::sort_order(State<Option<SortOrder>>)` | 2026-11-24 |
| `SidebarItem::selected` and `on_select` | `SidebarItem::tag(selection, value)` | 2026-11-24 |
| `styles::ColorEffect`, `styles::system_effect`, the `effect` field of every style, the `effect` argument of `Paints::current` | Gone: every style drew its paints unchanged | 2026-11-23 |
| `ctdesign::DOCK_GAP`, `DOCK_FONT_SIZE`, `DOCK_BAR_HEIGHT` | Gone | 2026-11-11 |
| `ctdesign::FOCUS_RING_COLOR_LIGHT`, `FOCUS_RING_COLOR_DARK`, `FOCUS_RING_COLOR_DARK_TINT` | The focus ring color of each accent in `ctdesign::ACCENTS` | 2026-11-20 |
| `SidebarItem::symbol(Symbol, tint)` | `SidebarItem::symbol_tile` | 2026-11-22 |
| `styles::MaterialStyle::blur` | Gone with the Skia blur: the compositor or AppKit draws it (`MaterialStyle::material`) | 2026-11-22 |
| `styles::GlassContainerStyle::{rim, blur, animation}`, `glass_container::{Morph, plan, MAX_SHAPES}`, `Frame::lerp`, the blur of `ShapeStyle::Material` | Gone with the Skia blur: the system draws the glass, and the kit's fallback is a flat fill | 2026-11-24 |
| `bk::shell::*` (`run_layer`, `LayerSpec`, `run_lock`, `SessionUnlock`, `Toplevels`, `Activation`, `use_status_items` and every other item of the module), `bk = { …, features = ["shell"] }` | `bk_shell::*`, the same names, in crate `bk-shell`; a shell depends on `bk-shell` (bk-shell) | 2026-11-24 |
| `bk::theme::for_scheme`, `bk::theme::use_shell_theme` | `bk_shell::for_scheme`, `bk_shell::use_shell_theme` | 2026-11-24 |
| `ctappearance::dock`, `menubar`, `notifications`, `shell_glass`, `liquid_glass` | `ctshell::dock`, `menubar`, `notifications`, `shell_glass`, `liquid_glass` (ctshell) | 2026-11-24 |
| `ctappearance::WallpaperSelection`, `WallpaperConfig`, `WallpaperCollection`, `WallpaperSource`, `WallpaperScale`, `WallpaperShuffle`, `WallpaperAppearance`, `WallpaperKind`, `ALL_DESKTOPS`, `desktop_key`, `wallpaper_selection_path`, `read_wallpaper_selection`, `write_wallpaper_selection`, `wallpaper_config_path`, `read_wallpaper_config`, `write_wallpaper_config`, `wallpapers_in`, `system_wallpapers_dir`, `custom_colors_dir`, `wallpaper_colors`, `thumbnail_path`, `thumbnail_is_fresh`, `thumbnail_image_size`, `frame_path`, `modified_seconds` | `ctshell::wallpaper::`, the same names | 2026-11-24 |
| `ctappearance::glass_lightness`, `glass_rim_lift`, `glass_blur_passes`, `GlassEdge`, `glass_edge`, `GlassShadow`, `glass_shadow` | `ctshell::resolve::`, the same names; `glass_chroma` stays | 2026-11-24 |
| `ctappearance::material::blur_region`, `ctappearance::menu::panel_over`, `ctappearance::traffic::group_rect`, `glyph_svg` | `ctshell::material::blur_region`, `ctshell::menu::panel_over`, `ctshell::traffic::group_rect`, `glyph_svg` | 2026-11-24 |
| `ctdesign::` shell tokens: `DOCK_*` sizes and timings, `MENUBAR_*` but `MENUBAR_HEIGHT`, `NOTIFICATION_*`, `LOCK_*`, `LAUNCHER_*`, `ABOUT_*` but `ABOUT_NAME`, `LIQUID_GLASS_*`, `SHELL_GLASS_*`, `CURSOR_SHAKE_*`, `LIVE_ACTIVITY_*`, `SPACE_GAP*`, `SPACE_SLIDE_*`, `SCREENSHOT_*`, `FORCE_QUIT_*`, `CONFIRM_*`, `CORNER_{NOTIFICATION,NOTIFICATION_CENTER,CONTROL_CENTER_MODULE,SPOTLIGHT}`, `WALLPAPER_{FPS,THUMBNAIL,FRAME}`, `WINDOW_EXPAND_*`, `TILE_*`, `FOCUS_BAR_*`, `BACKGROUND_BLUR_*`, `GLASS_MASK`, `DESKTOP_CLEAR`, `CIRCLE_EXPONENT`, the five `*_LAYER_NAMESPACE`, `dock_divider`, `RIM_BINS`, `GlassLook`, `GlassRim`, `GlassScale`, `GlassSet`, `GlassLens`, `GlassTint`, `ShellGlass` | `ctshell::design::`, the same names | 2026-11-24 |
| `ctdesign::DockSurface`, `DockTone`, `DOCK_RIM_*`, `DOCK_DIVIDER_*`, `DOCK_LENS_*`, `DOCK_TONE_*`, `DOCK_GLASS_*`, `DOCK_OUTLINE_*`, `MENU_GLASS_*`, `NOTIFICATION_GLASS_VEIL_LIGHT`, `Resolved::dock`, `menu_glass`, `notification_glass` | `ctshell::design::` the same names; `ctshell::resolve::{dock_surface, menu_glass, notification_glass}` build them | 2026-11-25 |
| `ctdesign::MaterialLook`, `MaterialSet`, `MATERIAL_<NAME>_LIGHT`, `MATERIAL_<NAME>_DARK`, `WALLPAPER_TINTING_GRID`, `MENUBAR_TINT_SPREAD`; `ctappearance::material::{looks, over}` | `ctshell::design::` the same names; `ctshell::material::{looks, over}` | 2026-11-25 |
| `ctappearance::{write_appearance, write_wallpaper_state, load_wallpaper_state, WallpaperState, WallpaperOutput}`, `ctappearance::window::write_double_click_titlebar`, `ctappearance::tinting::WallpaperGrid`, `ctappearance::ini::{value, set}`, `ctappearance::corner::{edge, coverage, tile}` | `ctshell::{write, wallpaper_state, ini, corner}::`, the same names; `toml_edit` leaves the kit | 2026-11-25 |
| `ctdesign::SettingsCheck`, `SettingsWallpaper` and the `window_min_height`, `tile_*`, `option_column`, `preview_*`, `check`, `empty_*`, `wallpaper`, `icon_card_*` fields of `SettingsMetrics` | `ctshell::design::SettingsDesktop` (`SETTINGS_DESKTOP`) | 2026-11-25 |
| `ctappearance::{MenuBarSize, ColorFilter}` and the `AppearanceConfig` keys `invert_colors`, `display_contrast`, `menu_bar_size`, `shake_to_locate`, `pointer_size`, `color_filters`, `color_filter`, `color_filter_intensity`, `liquid_glass_tint`, `gtk_bridge`, `qt_bridge`; `ctdesign::{POINTER_SIZES, DISPLAY_CONTRAST, COLOR_FILTER_INTENSITY}` | `ctshell::appearance::{DesktopAppearance, MenuBarSize, ColorFilter}`; the tokens in `ctshell::design`; the file keeps its keys | 2026-11-24 |
| `ctappearance::resolve(appearance, dominant)` | `ctappearance::resolve(appearance, dominant, native_lights)`: each backend passes its `NATIVE_LIGHTS`, Malaysia `false` | 2026-11-24 |
| `main_menu::layout::{DbusMenuLayout, LayoutItem}`, `status_item::{Pixmaps, ITEM_PATH, WATCHER, WATCHER_PATH}` | `bk::extension::{DbusMenuLayout, LayoutItem, Pixmaps, ITEM_PATH, WATCHER, WATCHER_PATH}` (feature `shell`, Linux): the D-Bus code lives in `platform/linux/` and `zbus` is a Linux-only dependency | 2026-11-25 |
| `infinity_protocol::THOUSANDTHS` | `ctdesign::THOUSANDTHS` | 2026-11-24 |
| `infinity-protocol`, `infinity_*` interfaces, `Infinity*` types, `infinity-*.xml` | `limitless`, `limitless_*`, `Limitless*`, `limitless-*.xml` | 2026-11-25 |
| `ctappearance::write_toml`, `keep_unparsed`, `icon_style::luma`, `xdg::cache_home` | `ctshell::write::write_toml`, `keep_unparsed`, `ctshell::material::luma`; `cache_home` is private to `ctshell::wallpaper` | 2026-11-25 |
| `ctappearance::read_appearance`, `read_appearance_from`, `Toolkit`, `TextSizes::get_mut`, `TextSizes::{gtk, qt}`, `AppearanceConfig::text_size_for`, `xdg::user_dir`, `traffic::hit`, `TrafficPointer::on_motion`, `menu::panel_luma`, `AppearanceEvent::WallpaperSelection` | Gone: nothing read them; the text size is `text_sizes.barrierkit` or `text_size` | 2026-11-25 |

## How to read the tables

- **BarrierKit:** the path inside `bk` (for example
  `views::form::Form`), or the full path for the other crates.
- **SwiftUI / AppKit:** the Apple equivalent; `≈` marks a partial match, `—`
  means there is none.
- **Chapter:** where the item is documented.

## Entry points

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `prelude` | module | `import SwiftUI` | `import AppKit` | — | Re-exports the entry points, controls, Freya's prelude and the tokens | [2](02-first-app.md#the-prelude) |
| `elements::Text` | struct | `Text` | `NSTextField(labelWithString:)` | `UILabel` | Text with `BODY` and the label color by default; wraps | [2](02-first-app.md#text-and-stacks) |
| `elements::Text::font_size` | method | `.font(.system(size:))` | `NSFont.systemFont(ofSize:)` | `UIFont.systemFont(ofSize:)` | A size outside the text styles, with SwiftUI's line height for it | [4](04-appearance-and-tokens.md#fonts) |
| `elements::Text::line_limit` | method | `.lineLimit(_:)` | `NSTextField.maximumNumberOfLines` | `UILabel.numberOfLines` | Shows at most `n` lines | [2](02-first-app.md#text-and-stacks) |
| `elements::Text::markdown` | fn | `Text("**x**")`, `AttributedString(markdown:)` | `NSAttributedString(markdown:)` | `NSAttributedString(markdown:)` | Reads inline Markdown: `**bold**`, `*italic*`, `~~strike~~`, `` `code` ``, `[title](https://x.org)`; `Text::new` keeps its string as it is | [6](06-text-and-editing.md#text-runs) |
| `elements::Text::bold`, `italic`, `underline`, `strikethrough` | method | `.bold(_:)`, `.italic(_:)`, `.underline(_:)`, `.strikethrough(_:)` | `NSFontManager.convert(_:toHaveTrait:)`, `NSUnderlineStyle` | — | Styles one `Text` when the `bool` is true, like SwiftUI's `isActive`; a sum of texts keeps each part's own | [6](06-text-and-editing.md#text-runs) |
| `elements::Text::monospaced_digit` | method | `.monospacedDigit()` | `NSFont.monospacedDigitSystemFont(ofSize:weight:)` | `UIFont.monospacedDigitSystemFont(ofSize:weight:)` | Gives every digit the same width, so a number that changes does not move the text after it | [6](06-text-and-editing.md#text-runs) |
| `elements::Text::text_selection` | method | `.textSelection(.enabled)` | `NSTextField.isSelectable` | `UILabel` (none) | Lets the reader select the text and copy it | [6](06-text-and-editing.md#text-runs) |
| `elements::Text` `+` | operator | `Text + Text` | `NSAttributedString.append(_:)` | — | A text of the runs of both, each with its own style | [6](06-text-and-editing.md#text-runs) |
| `elements::Label` | struct | `Label(_:systemImage:)` | `NSTextField` beside an `NSImageView` | `UILabel`, `UIImageView` | A `Symbol` in a 16 × 16.5 box before a title, 6 apart, in the font and color of `Text` (the accent in a `List` row; compact in a button label) | [2](02-first-app.md#text-and-stacks) |
| `elements::VStack` | struct | `VStack` | `NSStackView` (vertical) | `UIStackView` | Column of children, centered, apart by the distances of SwiftUI's `VStack` between neighbouring kinds of view; `.spacing(x)` makes them all `x` | [2](02-first-app.md#text-and-stacks) |
| `elements::HStack` | struct | `HStack` | `NSStackView` (horizontal) | `UIStackView` | Row of children, centered, `SPACE_2` apart | [2](02-first-app.md#text-and-stacks) |

## Style and palette

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `style::Palette` | struct | — | ≈ `NSColor` (semantic colors) | ≈ `UIColor` | Semantic colors of the current context: labels, backgrounds, accent, separators, outlines | [4](04-appearance-and-tokens.md#the-palette) |
| `style::text` | fn | ≈ `Text` | ≈ `NSTextField(labelWithString:)` | ≈ `UILabel` | One-line label with a type token and a color | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `style::continuous_corner` | fn | `RoundedCornerStyle.continuous` | — | `CALayerCornerCurve.continuous` | Corner radius with Apple's continuous curvature | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `style::panel_edges` | fn | — | — | — | Card framed with a panel window's inner and outer lines (glass in dark) | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `ctdesign::ease_out`, `ctdesign::ease_in_out` | fn | — | ≈ `CAMediaTimingFunction` (`easeOut`, `easeInEaseOut`) | ≈ `CAMediaTimingFunction` | Cubic ease-out and ease-in-out of a progress from 0 to 1; the kit, the shell and the compositor share them | [14](14-animations.md#spring) |
| `ctdesign::ease_in_ease_out` | fn | `UnitCurve.easeInOut` | `CAMediaTimingFunction(name: .easeInEaseOut)` | same | Core Animation's ease-in-ease-out Bézier (0.42, 0, 0.58, 1) of a progress from 0 to 1; AppKit's name, because `ease_in_out` is the cubic one; the sidebar shows and hides along it | [14](14-animations.md#spring) |
| `ctdesign::cubic_bezier`, `ctdesign::slider_press` | fn | `UnitCurve.bezier(startControlPoint:endControlPoint:)` | `CAMediaTimingFunction(controlPoints:)` | same | The cubic Bézier through (0, 0), (x1, y1), (x2, y2), (1, 1) of a progress from 0 to 1; `slider_press` is the measured (0, 0.13, 0.53, 1.11) of the knob press and release, which overshoots a little | [14](14-animations.md#spring) |
| `ctdesign::smoothstep` | fn | — | — | — | Hermite step from 0 to 1 between two edges | [14](14-animations.md#spring) |
| `theme::color` | fn | — | — | — | Converts a Tengen `Rgba` to Freya's `Color` | [4](04-appearance-and-tokens.md#text-shapes-and-colors) |
| `theme::resolved` | fn | ≈ `EnvironmentValues` | ≈ `NSApplication.effectiveAppearance` | ≈ `UITraitCollection.userInterfaceStyle` | Resolved appearance at this point of the tree | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::use_appearance` | fn | ≈ `.environment(_:_:)` | ≈ `NSAppearanceCustomization.appearance` | ≈ `UIView.overrideUserInterfaceStyle` | Hook: changes the appearance for a component and its children | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::SurfaceKind` | enum | — | — | — | `Toplevel` or `LayerShell`: which kind of surface a color is for | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::surface_background` | fn | — | — | — | Background color of a window or a shell surface | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::menu_palette` | fn | — | — | — | Menu colors over a given surface, lifted when tinted | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::popover_palette` | fn | — | — | — | Menu palette with pop-up menu metrics | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::apply_popover_metrics` | fn | — | — | — | Replaces a menu palette's metrics with `MENU_POPOVER` | [4](04-appearance-and-tokens.md#theme-functions) |
| `theme::over_glass` | fn | — | — | — | A menu palette over the compositor's glass: clear panel, solid search field; unchanged with Increase Contrast | [4](04-appearance-and-tokens.md#theme-functions) |

## Styles and themes

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `styles` | module | — | — | — | Value types of the styling system, `Theme`, `StyleScope` and the control styles | [Styles](../../.claude/skills/barrierkit-app/styles.md) |
| `styles::ShapeStyle` | enum | `ShapeStyle` | ≈ `NSColor` | ≈ `UIColor` | Paint of a part: a role, the accent, a fixed color, a gradient, the backdrop, a material, the window background (`WindowBackground`, SwiftUI's `.windowBackground`: clear where the system draws the window's material), a system color or nothing | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::SystemColor` | enum | `Color.red`…`Color.gray` | `NSColor.systemRed`… | `UIColor.systemRed`… | The 13 system colors, light and dark as AppKit gives them on macOS 27.2; paint with `ShapeStyle::System` | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Role` | enum | ≈ semantic `Color` (`.primary`, `.secondary`) | ≈ `NSColor.labelColor`… | ≈ `UIColor.label` | One color of the `Palette` (all but `accent`), resolved late | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Gradient` | struct | `LinearGradient`, `AngularGradient`, `EllipticalGradient` | `NSGradient` | ≈ `CAGradientLayer` | Stops of `ShapeStyle` as a linear gradient (`new`, an angle), an angular one (`angular`) or an elliptical one (`elliptical`) | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Gradient::angular` | method | `AngularGradient(gradient:center:angle:)` | — | ≈ `CAGradientLayer` | Sweeps clockwise from the right, starting at an angle in degrees; the center is the view's | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `mesh_gradient::MeshGradient` | struct | `MeshGradient` | — | — | A grid of colors, each at a unit point, blended across the view; `new(width, height, points, colors)` | [15](15-backdrops-and-media.md#mesh-gradient) |
| `mesh_gradient::MeshGradient::smooths_colors` | method | `MeshGradient(.. smoothsColors:)` | — | — | `false` blends in straight lines; the default eases the blend so a color holds around its point | [15](15-backdrops-and-media.md#mesh-gradient) |
| `styles::Gradient::elliptical` | method | `EllipticalGradient(gradient:)` | — | ≈ `CAGradientLayer` | An ellipse whose radii are half the view's width and half its height; center and radii are not offered | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Gradient::color_space` | method | `AnyGradient.colorSpace(_:)` | — | — | Picks how the stops mix | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::GradientColorSpace` | enum | `Gradient.ColorSpace` | — | — | `Perceptual` (default; OKLab, premultiplied: what SwiftUI draws by default on macOS 27.2.1) or `Device` (the mix of the bezel and tab gradients, measured against AppKit) | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Thickness` | enum | `Material` (`.ultraThin`…`.ultraThick`) | `NSVisualEffectView.Material` | `UIBlurEffect.Style` | How translucent and blurred a material is; values in `MATERIAL` | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::StateValue` | struct | — | — | — | A value with optional hovered, pressed, focused and disabled values | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::ControlState` | struct | ≈ `ButtonStyle.Configuration` (`isPressed`) | ≈ `NSControl.isHighlighted`, `isEnabled` | ≈ `UIControl.State` | Hovered, pressed, focused and disabled flags of a control | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Shape` | enum | `Rectangle`, `RoundedRectangle`, `UnevenRoundedRectangle`, `Capsule`, `Circle` | ≈ `NSBezierPath` | ≈ `UIBezierPath` | Outline of a part | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::CornerStyle` | enum | `RoundedCornerStyle` | — | `CALayerCornerCurve` | `Continuous` or `Circular` corners | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::ShadowStyle` | struct | `ShadowStyle` (`.drop`, `.inner`) | `NSShadow` | `NSShadow` | One shadow: color, x, y, blur, spread, inner | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Animation` | enum | `Animation` | ≈ `NSAnimationContext` | ≈ `UIViewPropertyAnimator` | How a change moves: `None`, `Ease`, `Spring` (period and bounce), `Custom`; [Spring](14-animations.md#spring) | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Axis` | enum | `Axis` | `NSUserInterfaceLayoutOrientation` | `NSLayoutConstraint.Axis` | `Horizontal` or `Vertical` | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::Edge` | enum | `Edge` | `NSRectEdge` | `UIRectEdge` | `Top`, `Bottom`, `Leading`, `Trailing` | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::LayoutDirection` | enum | `LayoutDirection` | `NSUserInterfaceLayoutDirection` | `UISemanticContentAttribute` | `LeftToRight` (default) or `RightToLeft`; ambient through `.layout_direction(..)` | [Styles](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| `styles::ControlStyle` | trait | ≈ the style protocols (`ButtonStyle`, `ToggleStyle`) | ≈ `NSCell` | ≈ `UIContentConfiguration` | Implemented by every control style: `tengen()` and the accessibility rules of its control | [Styles](../../.claude/skills/barrierkit-app/styles.md#which-controls-take-a-style) |
| `styles::CircularSliderStyle` | struct | — | ≈ `NSSliderCell` | — | Every visual property of `CircularSlider` | [Styles](../../crates/bk/src/styles/circular_slider.rs) |
| `styles::SliderStyle` | struct | ≈ `ButtonStyle` | ≈ `NSSliderCell` | ≈ `UISlider` | Every visual property of `Slider`; `tengen()` builds the default from the tokens | [Styles](../../.claude/skills/barrierkit-app/styles.md#the-fields-of-a-style) |
| `styles::ButtonStyle` | struct | `ButtonStyle`, `PrimitiveButtonStyle` | ≈ `NSButtonCell`, `NSButton.BezelStyle` | ≈ `UIButton.Configuration` | Every visual property of `Button`; `tengen()` is `.bordered`; `glass` is the native glass it sits on where the platform draws glass | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::bordered_prominent` | method | `PrimitiveButtonStyle.borderedProminent` | ≈ `NSButton.keyEquivalent` `"\r"` | ≈ `UIButton.Configuration.filled()` | Preset: accent fill, `on_accent` label | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::circular` | method | `.buttonBorderShape(.circle)` | ≈ `NSButton.BezelStyle.circular` | ≈ `UIButton.Configuration.cornerStyle` | Preset: a 17 high circle for an icon-only button | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::destructive` | method | `Button(role: .destructive)` | `NSButton.hasDestructiveAction` | `UIMenuElement.Attributes.destructive` | Preset: red label on a faint red fill | [Styles](../../crates/bk/src/styles/button.rs) |
| `views::feedback::Alert::destructive` | method | `Button(role: .destructive)` in `.alert` | `NSAlert.hasDestructiveAction` | `UIAlertAction.Style.destructive` | The primary button is destructive | [11](11-views.md#alert) |
| `views::feedback::Alert::icon` | method | — | `NSAlert.icon` | — | A 64 square above the title: the application icon | [11](11-views.md#alert) |
| `views::feedback::Alert::critical` | method | — | `NSAlert.Style.critical` | — | A caution triangle under the application icon, which shrinks to a corner badge | [11](11-views.md#alert) |
| `views::feedback::Alert::shows_help` | method | — | `NSAlert.showsHelp` | — | A 28 round help button at the top right | [11](11-views.md#alert) |
| `views::feedback::Alert::accessory_view` | method | — | `NSAlert.accessoryView` | — | A view between the message and the buttons | [11](11-views.md#alert) |
| `views::feedback::Alert::add_button` | method | `Button` in `.confirmationDialog` | `NSAlert.addButton(withTitle:)` | `UIAlertController.addAction(_:)` | An extra button; with one the buttons stack | [11](11-views.md#alert) |
| `styles::ButtonStyle::glass` | method | `PrimitiveButtonStyle.glass` | ≈ `NSButton.BezelStyle.glass` | ≈ `UIButton.Configuration.glass()` | Preset: native glass on macOS 26 and later (`ButtonStyle.glass`), else a raised capsule with a hairline rim | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::glass_prominent` | method | `PrimitiveButtonStyle.glassProminent` | — | `UIButton.Configuration.prominentGlass()` | Preset: native glass tinted with the accent on macOS 26 and later, else an accent capsule with a hairline rim | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::plain` | method | `PrimitiveButtonStyle.plain` | — | `UIButton.Configuration.plain()` | Preset: only the label, in the label color | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::link` | method | `PrimitiveButtonStyle.link` | — | — | Preset: only the label, in the accent color | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::small_square` | method | — | `NSButton.BezelStyle.smallSquare` | `UIButton.Configuration.gray()` | Preset: 21 high rectangle, light gradient, 1 wide border | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::textured_square` | method | — | `NSButton.BezelStyle.texturedSquare` | `UIButton.Configuration.gray()` | Preset: 20 high rounded rectangle, nearly white in light | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::textured_rounded` | method | — | `NSButton.BezelStyle.texturedRounded`, `.toolbar` | `UIButton.Configuration.gray()` | Preset: capsule with a faint ring and no fill | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::inline` | method | — | `NSButton.BezelStyle.inline` | `UIButton.Configuration.tinted()` | Preset: 18 high capsule with a 5% fill and a secondary label | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::push_disclosure` | method | — | `NSButton.BezelStyle.pushDisclosure`, `.roundedDisclosure` | — | Preset: 24 square with the push fill, for a chevron | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::ButtonStyle::borderless` | method | `PrimitiveButtonStyle.borderless` | `NSButton.isBordered` `false` | `UIButton.Configuration.plain()` | Preset: no background until hovered | [Styles](../../crates/bk/src/styles/button.rs) |
| `styles::SwitchStyle` | struct | `SwitchToggleStyle` | ≈ `NSSwitch` | ≈ `UISwitch` | Every visual property of `Switch`, with the label position | [Styles](../../crates/bk/src/styles/switch.rs) |
| `styles::CheckboxStyle` | struct | `CheckboxToggleStyle` | ≈ `NSButtonCell` | — | Every visual property of `Checkbox`, with the label position | [Styles](../../crates/bk/src/styles/checkbox.rs) |
| `styles::SegmentedControlStyle` | struct | `SegmentedPickerStyle` | ≈ `NSSegmentedCell` | ≈ `UISegmentedControl` | Every visual property of `SegmentedControl`, with its axis | [Styles](../../crates/bk/src/styles/segmented_control.rs) |
| `styles::ToolGroupStyle` | struct | — | — | — | Every visual property of `ToolGroup`, with its axis | [Styles](../../crates/bk/src/styles/tool_group.rs) |
| `styles::TextFieldStyle` | struct | `TextFieldStyle` | ≈ `NSTextFieldCell` | ≈ `UITextField` | Every visual property of `TextField` | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `styles::SearchFieldStyle` | struct | — | ≈ `NSSearchFieldCell` | ≈ `UISearchTextField` | Every visual property of `SearchField` | [Styles](../../crates/bk/src/styles/search_field.rs) |
| `styles::SecureFieldStyle` | struct | ≈ `TextFieldStyle` | ≈ `NSSecureTextFieldCell` | ≈ `UITextField.isSecureTextEntry` | The dots of `SecureField`; the field draws from the `TextFieldStyle` in context | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `styles::LevelSliderStyle` | struct | — | ≈ `NSSliderCell` | ≈ `UISlider` | Every visual property of `LevelSlider` | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `styles::LevelSliderStyle::fat` | method | — | — | — | Preset: the 22 tall Control Center bar without a knob, with a symbol | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `styles::RadioGroupStyle` | struct | `RadioGroupPickerStyle` | ≈ `NSButtonCell` (radio) | — | Every visual property of `RadioGroup`, with the axis and the label position | [Styles](../../crates/bk/src/styles/radio_group.rs) |
| `styles::ColorWellStyle` | struct | ≈ `PalettePickerStyle` | ≈ `NSColorWell.Style` | ≈ `UIColorWell` | Every visual property of `ColorWell`, with the axis and wrapping of the swatches | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `styles::ChoiceCardsStyle` | struct | — | — | — | Every visual property of `ChoiceCards`, with the axis and wrapping of the cards | [Styles](../../crates/bk/src/styles/choice_cards.rs) |
| `styles::IconButtonStyle` | struct | ≈ `BorderlessButtonStyle` | ≈ `NSButtonCell` | ≈ `UIButton.Configuration` | Every visual property of `IconButton` and `ToolbarOverflow` | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `styles::StepperStyle` | struct | — | ≈ `NSStepperCell` | ≈ `UIStepper` | Every visual property of `Stepper` and of the arrows of `DatePicker`, with the value position | [Styles](../../crates/bk/src/styles/stepper.rs) |
| `styles::ToggleGroupStyle` | struct | — | ≈ `NSSegmentedCell` | ≈ `UISegmentedControl` | Every visual property of `ToggleGroup`, with its axis | [Styles](../../crates/bk/src/styles/toggle_group.rs) |
| `styles::ToolbarSearchStyle` | struct | — | ≈ `NSSearchToolbarItem` | ≈ `UISearchController` | Width of `ToolbarSearch`; its field and button take their own styles | [Styles](../../crates/bk/src/styles/toolbar_search.rs) |
| `styles::MenuButtonStyle` | struct | `ButtonMenuStyle` | ≈ `NSPopUpButtonCell` (pull-down) | ≈ `UIButton.showsMenuAsPrimaryAction` | Every visual property of `MenuButton` and `ShareLink` | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `styles::ComboButtonStyle` | struct | ≈ `MenuStyle` | `NSComboButton.Style` | ≈ `UIButton.showsMenuAsPrimaryAction` | Every visual property of `ComboButton` | [Styles](../../crates/bk/src/styles/combo_button.rs) |
| `styles::HelpLinkStyle` | struct | — | ≈ `NSButton.BezelStyle.helpButton` | — | Every visual property of `HelpLink` | [Styles](../../crates/bk/src/styles/help_link.rs) |
| `styles::ControlGroupStyle` | struct | `ControlGroupStyle` | — | — | Every visual property of `ControlGroup`: background and separators | [Styles](../../crates/bk/src/styles/control_group.rs) |
| `styles::PopUpButtonStyle` | struct | `MenuPickerStyle` | ≈ `NSPopUpButtonCell` | ≈ `UIButton.changesSelectionAsPrimaryAction` | Every visual property of `PopUpButton`: background, label, indicator and chevron | [Styles](../../crates/bk/src/styles/popup_button.rs) |
| `styles::ComboBoxStyle` | struct | — | ≈ `NSComboBoxCell` | — | The arrow button of `ComboBox`; its field takes `TextFieldStyle` | [Styles](../../crates/bk/src/styles/combo_box.rs) |
| `styles::TokenFieldStyle` | struct | — | ≈ `NSTokenFieldCell` | — | Every visual property of `TokenField` and of the tokens of `SearchField` | [Styles](../../crates/bk/src/styles/token_field.rs) |
| `styles::DatePickerStyle` | struct | `DatePickerStyle` | ≈ `NSDatePickerCell` | ≈ `UIDatePicker` | Every visual property of `DatePicker` and its calendar; its arrows take `StepperStyle` | [Styles](../../crates/bk/src/styles/date_picker.rs) |
| `styles::PathControlStyle` | struct | — | ≈ `NSPathControlItem` | — | Every visual property of `PathControl` | [Styles](../../crates/bk/src/styles/path_control.rs) |
| `styles::ImageWellStyle` | struct | — | ≈ `NSImageCell` | — | Every visual property of `ImageWell`, with its drop border | [Styles](../../crates/bk/src/styles/image_well.rs) |
| `styles::ColorPickerStyle` | struct | — | ≈ `NSColorWell.Style` | — | Every visual property of `ColorPicker` and its panel but the color | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `styles::ProgressViewStyle` | struct | `LinearProgressViewStyle` | ≈ `NSProgressIndicator.Style` | ≈ `UIProgressView.Style` | Every visual property of `ProgressView`, with its sweep | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::ProgressRingStyle` | struct | `CircularProgressViewStyle` | ≈ `NSProgressIndicator.Style` | ≈ `UIActivityIndicatorView` | Every visual property of `ProgressRing` | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::SpinnerStyle` | struct | ≈ `CircularProgressViewStyle` | ≈ `NSProgressIndicator.Style` | ≈ `UIActivityIndicatorView` | Every visual property of `Spinner`, with its turn | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `styles::CapacityIndicatorStyle` | struct | ≈ `GaugeStyle` | ≈ `NSLevelIndicatorCell` | — | Every visual property of `CapacityIndicator` | [Styles](../../crates/bk/src/styles/capacity_indicator.rs) |
| `styles::RelevancyIndicatorStyle` | struct | — | ≈ `NSLevelIndicatorCell` | — | Every visual property of `RelevancyIndicator` | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `styles::RatingIndicatorStyle` | struct | — | ≈ `NSLevelIndicatorCell` | — | Every visual property of `RatingIndicator` | [Styles](../../crates/bk/src/styles/rating_indicator.rs) |
| `styles::GaugeStyle` | struct | `GaugeStyle` | — | — | Every visual property of `Gauge` | [Styles](../../crates/bk/src/styles/gauge.rs) |
| `styles::MediaButtonStyle` | struct | ≈ `ButtonStyle` | — | ≈ `UIButton.Configuration` | Every visual property of `MediaButton` | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `styles::RoundButtonStyle` | struct | ≈ `ButtonStyle` | — | ≈ `UIButton.Configuration` | Every visual property of `RoundButton` | [Styles](../../crates/bk/src/styles/round_button.rs) |
| `styles::LinkStyle` | struct | `LinkButtonStyle` | — | — | Every visual property of `Link` | [Styles](../../crates/bk/src/styles/link.rs) |
| `styles::TextStyle` | struct | ≈ `.font(_:)` and `.foregroundStyle(_:)` in the environment | ≈ `NSTextField.font`, `textColor` | ≈ `UIFont`, ≈ `UIColor` | Default font and paint of every `Text` | [Styles](../../crates/bk/src/styles/text.rs) |
| `styles::BadgeStyle` | struct | ≈ `.badgeProminence(_:)` | — | — | Every visual property of `Badge` | [Styles](../../crates/bk/src/styles/badge.rs) |
| `styles::DividerStyle` | struct | — | ≈ `NSBox` (separator) | — | Thickness and paint of `Divider` | [Styles](../../crates/bk/src/styles/divider.rs) |
| `styles::AvatarStyle` | struct | — | — | — | Every visual property of `Avatar` | [Styles](../../crates/bk/src/styles/avatar.rs) |
| `styles::ImageStyle` | struct | ≈ `.imageScale(_:)`, `.foregroundStyle(_:)` | — | ≈ `UIImage.SymbolConfiguration`, ≈ `UIColor` | Default size, symbol stroke and paint of every `Image` | [Styles](../../crates/bk/src/styles/image.rs) |
| `styles::MaterialStyle` | struct | `Material` | ≈ `NSVisualEffectView.Material` | ≈ `UIBlurEffect.Style` | Tint and rim of `Material`, and the `material` (`ctappearance::material::Material`, Tengen `HudWindow`) a compositor that composes the blur draws behind it | [Styles](../../crates/bk/src/styles/material.rs) |
| `styles::TopGlassStyle` | struct | ≈ `.scrollEdgeEffectStyle(_:for:)` | — | ≈ `UIScrollEdgeEffect.Style` | Tint and glass metrics of `TopGlass` | [Styles](../../crates/bk/src/styles/top_glass.rs) |
| `styles::GlassContainerStyle` | struct | ≈ `.glassEffect(_:in:)` | ≈ `NSGlassEffectView` | ≈ `UIGlassEffect` | Tint, solid color and `clear` of `GlassContainer` | [Styles](../../crates/bk/src/styles/glass_container.rs) |
| `styles::BackdropStyle` | struct | — | — | — | Angles, opacity and fade of the gradient layers of `Backdrop` | [Styles](../../crates/bk/src/styles/backdrop.rs) |
| `styles::ScrollViewStyle` | struct | ≈ `.scrollIndicators(_:axes:)` | ≈ `NSScroller.KnobStyle` | ≈ `UIScrollView.showsVerticalScrollIndicator` | Every visual property of the bar, edge fade and top line of `ScrollView` | [Styles](../../crates/bk/src/styles/scroll_view.rs) |
| `styles::TextEditorStyle` | struct | `TextEditorStyle` | ≈ `NSTextView` (`backgroundColor`, `textColor`) | ≈ `UITextView` | Every visual property of `TextEditor` | [Styles](../../crates/bk/src/styles/text_editor.rs) |
| `styles::RuleEditorStyle` | struct | — | ≈ `NSRuleEditor` | — | Gaps, indent and header text of `RuleEditor` | [Styles](../../crates/bk/src/styles/rule_editor.rs) |
| `styles::TitlebarStyle` | struct | ≈ `WindowToolbarStyle` | ≈ `NSWindow.ToolbarStyle`, `NSTitlebarSeparatorStyle` | ≈ `UINavigationBarAppearance` | Every visual property of `Titlebar`: heights, paddings, background, title and hairline | [Styles](../../crates/bk/src/styles/titlebar.rs) |
| `styles::FullscreenBarStyle` | struct | — | — | — | Tint and animation of the band under the overlaid title bar in Malaysia's focus mode; theme and container only | [Styles](../../crates/bk/src/styles/fullscreen_bar.rs) |
| `styles::TrafficLightsStyle` | struct | — | ≈ `NSWindow.standardWindowButton(_:)` | — | Size, gap, ring width, halo, pressed backdrop and the `ButtonVisual` of each traffic light per state | [Styles](../../crates/bk/src/styles/traffic_lights.rs) |
| `styles::SidebarStyle` | struct | ≈ `SidebarListStyle` | ≈ `NSTableView.Style.sourceList` | ≈ `UICollectionLayoutListConfiguration.Appearance.sidebar` | Paddings, the lights' position and gap, and the background of `Sidebar` | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `styles::SidebarItemStyle` | struct | ≈ `.listItemTint(_:)` | ≈ `NSTableCellView` | ≈ `UITableViewCell` | Every visual property of `SidebarItem`, selected and not | [Styles](../../crates/bk/src/styles/sidebar_item.rs) |
| `styles::SidebarSectionStyle` | struct | — | — | — | Every visual property of `SidebarSection` and its fold | [Styles](../../crates/bk/src/styles/sidebar_section.rs) |
| `styles::SidebarAccountStyle` | struct | — | — | — | Every visual property of `SidebarAccount` but its avatar | [Styles](../../crates/bk/src/styles/sidebar_account.rs) |
| `styles::SymbolTileStyle` | struct | — | — | — | Size, shape, paints and border of `SymbolTile` | [Styles](../../crates/bk/src/styles/symbol_tile.rs) |
| `styles::SplitViewStyle` | struct | ≈ `NavigationSplitViewStyle` | ≈ `NSSplitView.DividerStyle` | ≈ `UISplitViewController.Style` | Inset, grip, background, scrim, inspector line and animations of `SplitView` | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `styles::SplitPanesStyle` | struct | — | ≈ `NSSplitView.DividerStyle` | — | Width and paint of the dividers of `SplitPanes` | [Styles](../../crates/bk/src/styles/split_panes.rs) |
| `styles::FormStyle` | struct | `FormStyle` (`.grouped`) | — | `UITableView.Style.insetGrouped` | Every visual property of `Form`, its sections and rows | [Styles](../../crates/bk/src/styles/form.rs) |
| `styles::FormStyle::columns` | method | `FormStyle.columns` | — | ≈ `UITableView.Style.insetGrouped` | No cards; the labels right-aligned in a column, the controls after it | [11](11-views.md#form) |
| `styles::GroupBoxStyle` | struct | `GroupBoxStyle` | ≈ `NSBox` | — | Every visual property of `GroupBox` | [Styles](../../crates/bk/src/styles/group_box.rs) |
| `styles::DisclosureGroupStyle` | struct | `DisclosureGroupStyle` | ≈ `NSButton.BezelStyle.disclosure` | — | Every visual property of `DisclosureGroup` | [Styles](../../crates/bk/src/styles/disclosure_group.rs) |
| `styles::TableStyle` | struct | `TableStyle` | ≈ `NSTableView.Style`, `NSTableHeaderCell` | ≈ `UITableView.Style` | Every visual property of `Table`, its header cells and rows | [Styles](../../crates/bk/src/styles/table.rs) |
| `styles::CollectionViewStyle` | struct | — | ≈ `NSCollectionViewItem` | ≈ `UICollectionViewCell` | Every visual property of `CollectionView` and its tiles | [Styles](../../crates/bk/src/styles/collection_view.rs) |
| `styles::OutlineViewStyle` | struct | ≈ `ListStyle` | ≈ `NSOutlineView`, `NSBrowser` | ≈ `UICollectionView` | Every visual property of `OutlineView` and `Browser` | [Styles](../../crates/bk/src/styles/outline_view.rs) |
| `styles::TabViewStyle` | struct | `TabViewStyle` | ≈ `NSTabView` | ≈ `UITabBarController` | Every visual property of `TabView` and its tabs | [Styles](../../crates/bk/src/styles/tab_view.rs) |
| `styles::TabViewStyle::automatic` | method | `TabView` in a window, `.tabViewStyle(.automatic)` | — | — | The tabs as a glass segmented control centered on the window's bar | [11](11-views.md) |
| `styles::TextViewStyle` | struct | ≈ `TextEditorStyle` | ≈ `NSTextView` (`textColor`, `insertionPointColor`) | ≈ `UITextView` | Paint of the text and caret of `TextView` | [Styles](../../crates/bk/src/styles/text_view.rs) |
| `styles::SheetStyle` | struct | ≈ `presentationBackground(_:)` | ≈ `NSWindow` sheet | ≈ `UIWindow` | Every visual property of `Sheet` and of the sheet under an `Alert` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `styles::AlertStyle` | struct | — | ≈ `NSAlert` | ≈ `UIAlertController` | Every visual property of `Alert` but its sheet, buttons and checkbox | [Styles](../../crates/bk/src/styles/alert.rs) |
| `styles::ContentUnavailableViewStyle` | struct | — | — | — | Every visual property of `ContentUnavailableView` | [Styles](../../crates/bk/src/styles/content_unavailable_view.rs) |
| `styles::SettingsWindowStyle` | struct | — | — | — | Gaps of `SettingsWindow` and its search popover | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `styles::AboutViewStyle` | struct | — | — | — | Every visual property of `AboutView` | [Styles](../../crates/bk/src/styles/about_view.rs) |
| `styles::MenuStyle` | type | ≈ `MenuStyle` | ≈ `NSMenu` drawing | ≈ `UIMenu` | `ctdesign::MenuPalette`: the colors and metrics of menus, pop-up menus, popovers and tooltips | [Styles](../../.claude/skills/barrierkit-app/styles.md#menustyle) |
| `styles::Theme` | struct | ≈ styles set in the environment | ≈ `NSAppearance` | ≈ `UITraitCollection` | Style overrides for the whole app; `Theme::tengen()` changes nothing | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `styles::Theme::style` | method | ≈ `.buttonStyle(_:)` on the scene | — | ≈ `UIButton.Configuration` | Adds an override of the control whose style the closure takes | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `styles::StyleScope` | struct | ≈ a view after `.buttonStyle(_:)` | — | ≈ `UIButton.Configuration` | Component returned by `.control_style`: its overrides reach the controls inside | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `modifiers::StyleModifiers` | trait | `View` modifiers | — | — | Style modifiers for every view | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `modifiers::StyleModifiers::control_size` | method | `.controlSize(_:)` | `NSControl.controlSize` | — | Sizes every control inside the view; a control's own `.size(..)` wins | [5](05-controls.md#control-size) |
| `modifiers::StyleModifiers::control_style` | method | ≈ `.buttonStyle(_:)`, `.toggleStyle(_:)` | — | ≈ `UIButton.Configuration`, ≈ `UISwitch` | Changes the style of every control of that style inside the view | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `modifiers::StyleModifiers::frame` | method | `.frame(width:height:alignment:)` | — | `NSLayoutConstraint` | Wraps the view in a `Frame`; the size methods and `.alignment(..)` follow | [11](11-views.md#frame) |
| `modifiers::EnvironmentModifiers` | trait | `View` modifiers | — | — | Modifiers that put a value in the environment of a view: `disabled`, `environment`, `focused`, `default_focus` | [16](16-swiftui-to-the-kit.md#environment) |
| `modifiers::EnvironmentModifiers::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Disables every control inside the view; its own `.disabled` and every view above add up, and `false` enables nothing | [16](16-swiftui-to-the-kit.md#disabled) |
| `modifiers::EnvironmentModifiers::environment` | method | `.environment(_:_:)` | — | `UITraitCollection` | Gives the view's subtree a value of type `T`; a view that reads it with `use_environment::<T>()` renders again when it changes | [16](16-swiftui-to-the-kit.md#environment) |
| `modifiers::EnvironmentModifiers::focused` | method | `.focused(_:equals:)` | — | `UIResponder.becomeFirstResponder()` | Registers the controls inside the view under a value of a `FocusState` | [16](16-swiftui-to-the-kit.md#focus) |
| `modifiers::EnvironmentModifiers::default_focus` | method | `.defaultFocus(_:_:)` | ≈ `NSWindow.initialFirstResponder` | ≈ `UIResponder.becomeFirstResponder()` | Focuses the control registered under the value once the view has appeared | [16](16-swiftui-to-the-kit.md#focus) |
| `modifiers::EnvironmentModifiers::keyboard_shortcut` | method | `.keyboardShortcut(_:)` | `NSButton.keyEquivalent` | `UIKeyCommand` | Gives every `Button` inside the view a `Shortcut` that it has not set itself; the view's `.disabled` applies | [16](16-swiftui-to-the-kit.md#actions) |
| `environment::use_environment` | fn | `@Environment` | — | `UITraitCollection` | Hook: the value of type `T` of the nearest `.environment(..)`, or `T::default()`; the view renders again when it changes | [16](16-swiftui-to-the-kit.md#environment) |
| `environment::use_is_enabled` | fn | `\.isEnabled` | `NSControl.isEnabled` | `UIControl.isEnabled` | `false` inside a view that `.disabled(true)` disabled | [16](16-swiftui-to-the-kit.md#disabled) |
| `focus::FocusState` | struct | `FocusState` | ≈ `NSWindow.firstResponder` | `UIResponder.isFirstResponder` | Names the focus targets of a window: `get()` is the focused value, `set(Some(v))` moves the focus there, `set(None)` takes it off | [16](16-swiftui-to-the-kit.md#focus) |
| `focus::use_focus_state` | fn | `@FocusState` | — | `UIResponder.isFirstResponder` | Hook: creates a `FocusState<T>` | [16](16-swiftui-to-the-kit.md#focus) |
| `gesture::TapGesture` | struct | `TapGesture` | `NSClickGestureRecognizer` | `UITapGestureRecognizer` | One or more taps; `.count`, `.on_ended` | [5](05-controls.md#gestures) |
| `gesture::LongPressGesture` | struct | `LongPressGesture` | ≈ `NSPressGestureRecognizer` | `UILongPressGestureRecognizer` | The pointer stays down in place; `.minimum_duration`, `.maximum_distance` | [5](05-controls.md#gestures) |
| `gesture::DragGesture` | struct | `DragGesture` | ≈ `NSPanGestureRecognizer` | `UIPanGestureRecognizer` | A drag with translation and velocity; `.minimum_distance` | [5](05-controls.md#gestures) |
| `gesture::DragGesture::touches_only` | method | — | — | `UIScrollView.panGestureRecognizer` | A drag that only a finger begins, not a mouse | [5](05-controls.md#gestures) |
| `gesture::MagnifyGesture` | struct | `MagnifyGesture` | `NSMagnificationGestureRecognizer` | `UIPinchGestureRecognizer` | Two fingers apart or together | [5](05-controls.md#gestures) |
| `gesture::RotateGesture` | struct | `RotateGesture` | `NSRotationGestureRecognizer` | `UIRotationGestureRecognizer` | Two fingers turning | [5](05-controls.md#gestures) |
| `gesture::GestureState` | struct | `GestureState` | — | — | A state that goes back when its gesture ends; `use_gesture_state` | [5](05-controls.md#gestures) |
| `gesture::Attachable::exclusively` | method | `.exclusively(before:)` | ≈ `shouldRequireFailureOf` | ≈ `require(toFail:)` | The first gesture is tried first; the second waits for it | [5](05-controls.md#gestures) |
| `gesture::Attachable::simultaneously` | method | `.simultaneously(with:)` | ≈ `shouldRecognizeSimultaneously` | ≈ `gestureRecognizer(_:shouldRecognizeSimultaneouslyWith:)` | Both gestures run | [5](05-controls.md#gestures) |
| `modifiers::ViewModifiers::gesture` | method | `.gesture(_:)` | ≈ `NSView.addGestureRecognizer(_:)` | `UIView.addGestureRecognizer(_:)` | Attaches a gesture to a view | [5](05-controls.md#gestures) |
| `modifiers::ViewModifiers::simultaneous_gesture` | method | `.simultaneousGesture(_:)` | — | — | A gesture that never fails the others | [5](05-controls.md#gestures) |
| `modifiers::ViewModifiers::high_priority_gesture` | method | `.highPriorityGesture(_:)` | — | — | A gesture that beats those of the views below | [5](05-controls.md#gestures) |
| `gesture::Point` | type | `CGPoint` | `NSPoint` | `CGPoint` | A point `(x, y)` in points | [5](05-controls.md#gestures) |
| `gesture::Phase` | enum | ≈ `TouchPhase` | ≈ `NSEvent.EventType` | `UITouch.Phase` | Down, move, up or cancel of one pointer | [5](05-controls.md#gestures) |
| `gesture::Input` | struct | — | ≈ `NSEvent` | ≈ `UITouch` | One pointer or finger at an instant, in milliseconds | [5](05-controls.md#gestures) |
| `gesture::State` | enum | — | `NSGestureRecognizer.State` | `UIGestureRecognizer.State` | Possible, began, changed, ended, failed or cancelled | [5](05-controls.md#gestures) |
| `gesture::Step` | enum | — | — | — | What a recognizer reports for one input | [5](05-controls.md#gestures) |
| `gesture::Recognizer` | trait | — | `NSGestureRecognizer` | `UIGestureRecognizer` | A gesture's state machine over inputs | [5](05-controls.md#gestures) |
| `gesture::TapRecognizer` | struct | — | `NSClickGestureRecognizer` | `UITapGestureRecognizer` | The state machine of `TapGesture` | [5](05-controls.md#gestures) |
| `gesture::LongPressRecognizer` | struct | — | ≈ `NSPressGestureRecognizer` | `UILongPressGestureRecognizer` | The state machine of `LongPressGesture` | [5](05-controls.md#gestures) |
| `gesture::DragRecognizer` | struct | — | ≈ `NSPanGestureRecognizer` | `UIPanGestureRecognizer` | The state machine of `DragGesture` | [5](05-controls.md#gestures) |
| `gesture::MagnifyRecognizer` | struct | — | `NSMagnificationGestureRecognizer` | `UIPinchGestureRecognizer` | The state machine of `MagnifyGesture` | [5](05-controls.md#gestures) |
| `gesture::RotateRecognizer` | struct | — | `NSRotationGestureRecognizer` | `UIRotationGestureRecognizer` | The state machine of `RotateGesture` | [5](05-controls.md#gestures) |
| `gesture::TapValue` | struct | ≈ `SpatialTapGesture.Value` | — | — | Where a tap ended | [5](05-controls.md#gestures) |
| `gesture::LongPressValue` | struct | — | — | — | Where a long press was held | [5](05-controls.md#gestures) |
| `gesture::DragValue` | struct | `DragGesture.Value` | — | — | Start, location, translation and velocity of a drag | [5](05-controls.md#gestures) |
| `gesture::MagnifyValue` | struct | `MagnifyGesture.Value` | `NSEvent.magnification` | `UIPinchGestureRecognizer.scale` | The ratio of the fingers' distance to the first | [5](05-controls.md#gestures) |
| `gesture::RotateValue` | struct | `RotateGesture.Value` | `NSEvent.rotation` | `UIRotationGestureRecognizer.rotation` | The angle of the fingers' line from the first, in radians | [5](05-controls.md#gestures) |
| `gesture::Priority` | enum | ≈ `GestureMask` | — | — | Normal, high or simultaneous in the arena | [5](05-controls.md#gestures) |
| `gesture::Callbacks` | struct | — | — | — | The changed, ended and reset callbacks of a gesture | [5](05-controls.md#gestures) |
| `gesture::Gesture` | trait | `Gesture` | — | — | A gesture the modifiers take | [5](05-controls.md#gestures) |
| `gesture::Attachable` | trait | — | — | — | What `.gesture` takes: one gesture or several put together | [5](05-controls.md#gestures) |
| `gesture::Combined` | struct | `SimultaneousGesture`, `ExclusiveGesture` | — | — | Gestures put together | [5](05-controls.md#gestures) |
| `gesture::Handlers` | trait | — | — | — | The event handlers of a gesture on a view; hidden | [5](05-controls.md#gestures) |
| `gesture::use_gesture_state` | fn | `@GestureState` | — | — | Creates a `GestureState` | [5](05-controls.md#gestures) |
| `gesture::Trackpad` | struct | — | `NSEvent` (magnify, rotate) | — | A trackpad pinch or rotation, as the runner reads it | [5](05-controls.md#gestures) |
| `gesture::TrackpadKind` | enum | — | `NSEvent.magnification`, `NSEvent.rotation` | — | The change in magnification or in degrees | [5](05-controls.md#gestures) |
| `gesture::TrackpadPhase` | enum | — | `NSEvent.phase` | — | Began, changed or ended | [5](05-controls.md#gestures) |
| `gesture::trackpad` | fn | — | `NSView.magnify(with:)`, `NSView.rotate(with:)` | — | Gives a trackpad gesture to the gestures under the pointer; the macOS runner calls it | [5](05-controls.md#gestures) |
| `gesture::SequenceGesture` | struct | `SequenceGesture` | — | — | The first gesture, then the second from where it ended | [5](05-controls.md#gestures) |
| `gesture::SequenceValue` | enum | `SequenceGesture.Value` | — | — | First, or second with the first's value and the second's if it began | [5](05-controls.md#gestures) |
| `gesture::SequenceRecognizer` | struct | — | — | — | The state machine of `SequenceGesture` | [5](05-controls.md#gestures) |
| `gesture::Gesture::sequenced_before` | method | `.sequenced(before:)` | — | — | `next` can begin once this gesture has ended | [5](05-controls.md#gestures) |
| `modifiers::ViewModifiers::foreground_style` | method | `.foregroundStyle(_:)` | ≈ `NSTextField.textColor` | ≈ `UILabel.textColor` | Text color of `Text`, `label` or `paragraph` from a `ShapeStyle` | [Styles](../../.claude/skills/barrierkit-app/styles.md#paint-on-views) |
| `modifiers::ViewModifiers::background_style` | method | `.background(_:ignoresSafeAreaEdges:)`; Freya already uses `background` | ≈ `NSView.layer.backgroundColor` | ≈ `UIView.layer` | Background of a view from a `ShapeStyle`, with blur for a material | [Styles](../../.claude/skills/barrierkit-app/styles.md#paint-on-views) |
| `modifiers::ViewModifiers::shadows` | method | ≈ `.shadow(color:radius:x:y:)`; Freya already uses `shadow` | ≈ `NSView.shadow` | ≈ `CALayer.shadowOpacity` | Adds a list of `ShadowStyle` to a view | [Styles](../../.claude/skills/barrierkit-app/styles.md#paint-on-views) |
| `controls::Slider::style` | method | ≈ `.buttonStyle(_:)` on one view | ≈ `NSSlider.trackFillColor` | ≈ `UISlider.minimumTrackTintColor` | Changes the style of one slider | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |
| `controls::Button::style` | method | `.buttonStyle(_:)` on one view | ≈ `NSButton.bezelStyle` | ≈ `UIButton.Configuration` | Changes the style of one button; takes a preset or a closure | [Styles](../../crates/bk/src/styles/button.rs) |
| `controls::Switch::style` | method | `.toggleStyle(.switch)` on one view | — | `UISwitch` | Changes the style of one switch | [Styles](../../crates/bk/src/styles/switch.rs) |
| `controls::Checkbox::mixed` | method | — | ≈ `NSButton.allowsMixedState` | — | Shows a dash instead of the check | [5](05-controls.md#switch-and-checkbox) |
| `controls::Checkbox::style` | method | `.toggleStyle(.checkbox)` on one view | — | — | Changes the style of one checkbox | [Styles](../../crates/bk/src/styles/checkbox.rs) |
| `controls::SegmentedControl::style` | method | `.pickerStyle(.segmented)` on one view | ≈ `NSSegmentedControl.segmentStyle` | `UISegmentedControl` | Changes the style of one segmented control | [Styles](../../crates/bk/src/styles/segmented_control.rs) |
| `controls::ToolGroup::style` | method | — | — | — | Changes the style of one tool group | [Styles](../../crates/bk/src/styles/tool_group.rs) |
| `controls::TextField::style` | method | `.textFieldStyle(_:)` on one view | ≈ `NSTextField.bezelStyle` | ≈ `UITextField.borderStyle` | Changes the style of one text field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `controls::SearchField::style` | method | — | — | — | Changes the style of one search field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `controls::SecureField::style` | method | `.textFieldStyle(_:)` on one view | — | — | Changes the style of one password field | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `level::LevelSlider::style` | method | — | — | — | Changes the style of one level slider | [Styles](../../crates/bk/src/styles/level_slider.rs) |
| `choices::RadioGroup::style` | method | `.pickerStyle(.radioGroup)` on one view | — | — | Changes the style of one radio group | [Styles](../../crates/bk/src/styles/radio_group.rs) |
| `choices::ColorWell::style` | method | `.pickerStyle(.palette)` on one view | ≈ `NSColorWell.colorWellStyle` | ≈ `UIColorWell` | Changes the style of one color well | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `choices::ChoiceCards::style` | method | — | — | — | Changes the style of one set of choice cards | [Styles](../../crates/bk/src/styles/color_well.rs) |
| `compact::IconButton::style` | method | `.buttonStyle(_:)` on one view | ≈ `NSButton.bezelStyle` | ≈ `UIButton.Configuration` | Changes the style of one icon button | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `compact::Stepper::style` | method | — | — | — | Changes the style of one stepper | [Styles](../../crates/bk/src/styles/stepper.rs) |
| `compact::ToggleGroup::style` | method | — | — | — | Changes the style of one toggle group | [Styles](../../crates/bk/src/styles/toggle_group.rs) |
| `compact::ToolbarSearch::style` | method | — | — | — | Changes the width of one toolbar search | [Styles](../../crates/bk/src/styles/toolbar_search.rs) |
| `compact::ToolbarOverflow::style` | method | `.buttonStyle(_:)` on one view | — | `UIButton.Configuration` | Changes the `IconButtonStyle` of one overflow button | [Styles](../../crates/bk/src/styles/icon_button.rs) |
| `buttons::MenuButton::style` | method | `.menuStyle(_:)` on one view | — | ≈ `UIButton.showsMenuAsPrimaryAction` | Changes the style of one menu button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ComboButton::style` | method | `.menuStyle(_:)` on one view | ≈ `NSComboButton.style` | `UIButton.showsMenuAsPrimaryAction` | Changes the style of one combo button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::HelpLink::style` | method | — | — | — | Changes the style of one help button | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ShareLink::style` | method | `.menuStyle(_:)` on one view | — | — | Changes the `MenuButtonStyle` of one share link | [Styles](../../crates/bk/src/styles/menu_button.rs) |
| `buttons::ControlGroup::style` | method | `.controlGroupStyle(_:)` on one view | — | — | Changes the style of one control group | [Styles](../../crates/bk/src/styles/control_group.rs) |
| `popup_button::PopUpButton::style` | method | `.pickerStyle(.menu)` on one view | — | ≈ `UIButton.changesSelectionAsPrimaryAction` | Changes the style of one pop-up button | [Styles](../../crates/bk/src/styles/popup_button.rs) |
| `popup_button::PopUpButton::pulls_down` | method | — | `NSPopUpButton.pullsDown` | `UIButton.showsMenuAsPrimaryAction` | The title is the first item, the menu opens below with the rest and a single chevron points down | [5](05-controls.md#choices) |
| `popup_button::PopUpButton::disabled` | method | `.disabled(_:)` | `isEnabled` | `UIControl.isEnabled` | Dims the pop-up button and stops it opening | [5](05-controls.md#choices) |
| `combo_box::ComboBox::style` | method | — | — | — | Changes the style of the arrow button of one combo box | [Styles](../../crates/bk/src/styles/combo_box.rs) |
| `token_field::TokenField::style` | method | — | — | — | Changes the style of one token field | [Styles](../../crates/bk/src/styles/token_field.rs) |
| `date_picker::DatePicker::graphical` | method | `.datePickerStyle(.graphical)` | ≈ `NSDatePicker.datePickerStyle` `.clockAndCalendar` | ≈ `UIDatePicker.preferredDatePickerStyle` | Shows the month calendar itself instead of the field | [5](05-controls.md#date-picker) |
| `date_picker::DatePicker::field` | method | `.datePickerStyle(.field)` | `NSDatePicker.datePickerStyle` `.textField` | `UIDatePicker.preferredDatePickerStyle` | Shows the field without the stepper | [5](05-controls.md#date-picker) |
| `date_picker::DatePicker::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Stepper and field height of one date picker | [5](05-controls.md#date-picker) |
| `date_picker::DatePicker::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Tertiary label ink, a disabled stepper and no input | [5](05-controls.md#date-picker) |
| `date_picker::DatePicker::style` | method | `.datePickerStyle(_:)` on one view | ≈ `NSDatePicker.datePickerStyle` | ≈ `UIDatePicker.preferredDatePickerStyle` | Changes the style of one date picker | [Styles](../../crates/bk/src/styles/date_picker.rs) |
| `path_control::PathControl::style` | method | — | — | — | Changes the style of one path control | [Styles](../../crates/bk/src/styles/path_control.rs) |
| `image_well::ImageWell::style` | method | — | — | — | Changes the style of one image well | [Styles](../../crates/bk/src/styles/image_well.rs) |
| `color_picker::ColorPicker::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Draws the well at half opacity and takes no click | [5](05-controls.md#color-picker) |
| `styles::ColorPickerStyle::minimal` | method | — | `NSColorWell.Style.minimal` | — | Preset: the capsule swatch without the bezel | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `styles::ColorPickerStyle::expanded` | method | — | `NSColorWell.Style.expanded` | — | Preset: a rounded swatch and a color wheel button | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `color_picker::ColorPicker::style` | method | — | ≈ `NSColorWell.colorWellStyle` | ≈ `UIColorWell` | Changes the style of one color picker | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `font_panel::FontPicker::style` | method | `.buttonStyle(_:)` on one view | — | `UIButton.Configuration` | Changes the `ButtonStyle` of one font picker | [Styles](../../crates/bk/src/styles/color_picker.rs) |
| `progress::ProgressView::style` | method | `.progressViewStyle(_:)` on one view | ≈ `NSProgressIndicator.Style` | ≈ `UIProgressView.Style` | Changes the style of one progress bar | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `progress::ProgressRing::style` | method | `.progressViewStyle(.circular)` on one view | — | `UIActivityIndicatorView` | Changes the style of one progress ring | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `progress::Spinner::style` | method | `.progressViewStyle(.circular)` on one view | — | `UIActivityIndicatorView` | Changes the style of one spinner | [Styles](../../crates/bk/src/styles/progress_view.rs) |
| `indicator::CapacityIndicator::style` | method | `.gaugeStyle(_:)` on one view | ≈ `NSLevelIndicator.levelIndicatorStyle` | — | Changes the style of one capacity indicator | [Styles](../../crates/bk/src/styles/capacity_indicator.rs) |
| `indicator::RelevancyIndicator::style` | method | — | ≈ `NSLevelIndicator.levelIndicatorStyle` | — | Changes the style of one relevancy indicator | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `indicator::RatingIndicator::style` | method | — | ≈ `NSLevelIndicator.levelIndicatorStyle` | — | Changes the style of one rating indicator | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `indicator::Gauge::style` | method | `.gaugeStyle(_:)` on one view | — | — | Changes the style of one gauge | [Styles](../../crates/bk/src/styles/relevancy_indicator.rs) |
| `media::MediaButton::style` | method | `.buttonStyle(_:)` on one view | — | `UIButton.Configuration` | Changes the style of one media button | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `media::RoundButton::style` | method | `.buttonStyle(_:)` on one view | — | `UIButton.Configuration` | Changes the style of one round button | [Styles](../../crates/bk/src/styles/media_button.rs) |
| `adornments::Link::style` | method | `.buttonStyle(.link)` on one view | — | `UIButton.Configuration` | Changes the style of one link | [Styles](../../crates/bk/src/styles/link.rs) |
| `elements::Text::style` | method | ≈ `.font(_:)` on one view | — | ≈ `UIFont` | Builds one text again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Badge::style` | method | — | — | — | Builds one badge again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Divider::style` | method | — | — | — | Builds one divider again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `adornments::Avatar::style` | method | — | — | — | Builds one avatar again with its style changed | [Styles](../../crates/bk/src/styles/text.rs) |
| `symbol::Image::style` | method | ≈ `.imageScale(_:)` on one view | — | ≈ `UIImage.SymbolConfiguration` | Changes the style of one image | [Styles](../../crates/bk/src/styles/text.rs) |
| `media::Material::style` | method | — | — | — | Changes the style of one material | [Styles](../../crates/bk/src/styles/material.rs) |
| `media::TopGlass::style` | method | ≈ `.scrollEdgeEffectStyle(_:for:)` on one view | — | ≈ `UIScrollEdgeEffect.Style` | Changes the style of one top glass | [Styles](../../crates/bk/src/styles/material.rs) |
| `glass_container::GlassContainer::style` | method | — | — | — | Changes the style of one glass container | [Styles](../../crates/bk/src/styles/material.rs) |
| `backdrop::Backdrop::style` | method | — | — | — | Changes the style of one backdrop | [Styles](../../crates/bk/src/styles/backdrop.rs) |
| `controls::ScrollView::refreshable` | method | ≈ `.refreshable(action:)` | ≈ `NSRefreshController` (macOS 27) | ≈ `UIRefreshControl` | Calls the action when the content is pulled down past 80 and holds it 60 down while `refreshing` is true | [5](05-controls.md#scrolling) |
| `controls::ScrollView::style` | method | ≈ `.scrollIndicators(_:axes:)` on one view | — | ≈ `UIScrollView.showsVerticalScrollIndicator` | Changes the style of one scroll view | [Styles](../../crates/bk/src/styles/scroll_view.rs) |
| `text_editor::TextEditor::style` | method | `.textEditorStyle(_:)` on one view | — | — | Changes the style of one text editor | [Styles](../../crates/bk/src/styles/text_editor.rs) |
| `rule_editor::RuleEditor::style` | method | — | — | — | Changes the style of one rule editor | [Styles](../../crates/bk/src/styles/rule_editor.rs) |
| `titlebar::Titlebar::style` | method | — | — | — | Changes the style of one title bar | [Styles](../../crates/bk/src/styles/titlebar.rs) |
| `traffic::TrafficLights::style` | method | — | — | — | Changes the style of one set of traffic lights | [Styles](../../crates/bk/src/styles/traffic_lights.rs) |
| `sidebar::Sidebar::style` | method | — | — | — | Changes the style of one sidebar | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarItem::style` | method | — | — | — | Changes the style of one sidebar row | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarSection::style` | method | — | — | — | Changes the style of one sidebar section | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SidebarAccount::style` | method | — | — | — | Changes the style of one account row | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `sidebar::SymbolTile::style` | method | — | — | — | Builds the tile again with a style override | [Styles](../../crates/bk/src/styles/sidebar.rs) |
| `views::split::SplitView::style` | method | — | — | — | Changes the style of one split view | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `views::split_panes::SplitPanes::style` | method | — | — | — | Changes the style of one set of panes | [Styles](../../crates/bk/src/styles/split_view.rs) |
| `views::form::Form::style` | method | `formStyle(_:)` | — | `UITableView.Style.insetGrouped` | Changes the style of one form | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::form::Form::pinned` | method | `safeAreaInset(edge:)` | — | `UIView.safeAreaInsets` | Keeps one section under the title bar of a settings pane while the rest scrolls | [11](11-views.md#form) |
| `views::group::GroupBox::style` | method | `groupBoxStyle(_:)` | — | — | Changes the style of one group box | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::group::DisclosureGroup::style` | method | `disclosureGroupStyle(_:)` | — | — | Changes the style of one disclosure group | [Styles](../../crates/bk/src/styles/form.rs) |
| `views::table::Table::style` | method | `tableStyle(_:)` | ≈ `NSTableView.style` | ≈ `UITableView.Style` | Changes the style of one table | [Styles](../../crates/bk/src/styles/table.rs) |
| `styles::TableStyle::inset_list`, `plain_list`, `bordered_list` | fn | `.listStyle(.inset)`, `.plain`, `.bordered` | `NSTableView.Style.inset`, `.plain` | `UITableView.Style`, `UICollectionLayoutListConfiguration.Appearance` | The styles of `List`: headerless rows with separators | [11](11-views.md#list) |
| `styles::TableStyle::striped` | method | `.inset(alternatesRowBackgrounds: true)` | `NSTableView.usesAlternatingRowBackgroundColors` | — | Alternating rows without separators | [11](11-views.md#list) |
| `styles::TableStyle::full_width`, `plain`, `source_list` | fn | — | `NSTableView.Style.fullWidth`, `.plain`, `.sourceList` | `UITableView.Style` | The classic styles of `Table`: edge to edge rows, 17 high rows, a headerless source list | [11](11-views.md#table) |
| `styles::TableStyle::row_size` | method | `controlSize(_:)` | `NSTableView.rowSizeStyle` | — | Font and icon of small, medium and large rows; the rows of a source list | [11](11-views.md#table) |
| `styles::OutlineViewStyle::classic`, `full_width` | fn | — | `NSOutlineView`, `NSTableView.Style.fullWidth` | ≈ `UICollectionView`, `UITableView.Style` | Gray icons without hairlines; the same with square rows from edge to edge | [11](11-views.md#outlineview) |
| `styles::TabType` | enum | — | `NSTabView.TabType` | — | Where the tabs are: `Top`, `Bottom`, `Toolbar`, `NoTabsBezelBorder`, `NoTabsLineBorder`, `NoTabsNoBorder` | [11](11-views.md#tabview) |
| `styles::TabViewStyle::toolbar` | fn | — | `NSTabViewController.tabStyle = .toolbar` | — | The icon-over-title tabs of a settings window | [11](11-views.md#tabview) |
| `views::collection::CollectionView::style` | method | — | — | — | Changes the style of one collection view | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tree::OutlineView::style` | method | ≈ `listStyle(_:)` | — | ≈ `UICollectionLayoutListConfiguration.Appearance` | Changes the style of one outline view | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tree::Browser::style` | method | — | — | — | Changes the style of one browser | [Styles](../../crates/bk/src/styles/table.rs) |
| `views::tabs::TabView::style` | method | `tabViewStyle(_:)` | — | — | Changes the style of one tab view | [Styles](../../crates/bk/src/styles/tab_view.rs) |
| `views::text_view::TextView::style` | method | ≈ `textEditorStyle(_:)` | — | — | Changes the style of one text view | [Styles](../../crates/bk/src/styles/text_view.rs) |
| `views::feedback::Sheet::style` | method | ≈ `presentationBackground(_:)` | — | ≈ `UISheetPresentationController` | Builds the sheet again with a style; call it right after `new` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `views::feedback::Alert::style` | method | — | — | — | Changes the style of one alert | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `views::feedback::ContentUnavailableView::style` | method | — | — | — | Builds the view again with a style; call it right after `new` | [Styles](../../crates/bk/src/styles/sheet.rs) |
| `views::feedback::ContentUnavailableView::search` | function | `ContentUnavailableView.search(text:)` | — | `UIContentUnavailableConfiguration.search()` | The empty view of a search with no hits, titled with the query | [11](11-views.md#contentunavailableview) |
| `views::feedback::ContentUnavailableView::description` | method | `ContentUnavailableView(_:systemImage:description:)` | — | `UIContentUnavailableConfiguration` | A second line under the title | [11](11-views.md#contentunavailableview) |
| `views::feedback::ContentUnavailableView::actions` | method | `ContentUnavailableView(label:description:actions:)` | — | `UIContentUnavailableConfiguration` | A row under the description, such as a button | [11](11-views.md#contentunavailableview) |
| `patterns::settings::SettingsWindow::navigation_destination` | method | `navigationDestination(item:)` | — | `UINavigationController.pushViewController(_:animated:)` | Opens a pane by id on request | [12](12-patterns.md#settingswindow) |
| `patterns::settings::SettingsWindow::style` | method | — | — | — | Changes the style of one preferences window | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `patterns::about::AboutView::style` | method | — | — | — | Changes the style of one About view | [Styles](../../crates/bk/src/styles/settings_window.rs) |
| `menu::MenuView::style` | method | — | — | — | Changes the style of one menu and its submenus | [7](07-menus-and-popovers.md#style) |
| `menu::MenuView::with_surface` | method | — | — | — | Takes its live palette from a `SurfaceKind` | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuView::with_popover_metrics` | method | — | — | — | Uses the paddings of a menu inside a popover | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuView::with_popup_spec` | method | — | — | — | The spec a submenu was opened with; ← closes a menu that has one | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuView::with_focused_index` | method | — | `NSMenu.highlightedItem` | — | The row highlighted when the menu opens | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuView::with_items_source` | method | — | `NSMenuDelegate.menuNeedsUpdate(_:)` | — | Reads the items from a closure on every render | [7](07-menus-and-popovers.md#helpers) |
| `menu::MenuView::preferred_size` | fn | — | `NSMenu.size` | — | The width and height the items take | [7](07-menus-and-popovers.md#menu-view) |
| `app::Application::theme` | method | ≈ `.environment(_:_:)` on the scene | ≈ `NSApp.appearance` | ≈ `UIWindow.overrideUserInterfaceStyle` | Gives the app its `Theme` | [Styles](../../.claude/skills/barrierkit-app/styles.md#levels) |

## Symbols and fonts

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `symbol::Symbol` | enum | ≈ `Image(systemName:)` names | ≈ `NSImage(systemSymbolName:accessibilityDescription:)` names | ≈ `UIImage` | Name of a built-in line icon; 158 variants, listed with their Apple names in [Symbol variants](#symbol-variants) | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::SymbolWeight` | enum | `Font.Weight` | `NSFont.Weight` | `UIFont.Weight` | The nine weights of a symbol: `UltraLight` to `Black`; `Regular` by default. Named after UIKit's `UIImage.SymbolWeight`: Freya's `FontWeight` is a number the prelude must not shadow; named `FontWeight` on 2026-11-21 only | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::ImageScale` | enum | `Image.Scale` | `NSImage.SymbolScale` | `UIImage.SymbolScale` | `Small`, `Medium` (default) or `Large`: the size of a symbol next to its text | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::SymbolRenderingMode` | enum | `SymbolRenderingMode` | `NSImage.SymbolConfiguration` | `UIImage.SymbolConfiguration` | `Monochrome` (default) or `Hierarchical` (a dimmed second layer) | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::font_weight` | method | `.fontWeight(_:)` | `NSImage.SymbolConfiguration(weight:)` | `UIImage.SymbolConfiguration` | The weight of one symbol: its stroke | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::image_scale` | method | `.imageScale(_:)` | `NSImage.SymbolConfiguration(scale:)` | `UIImage.SymbolConfiguration` | The scale of one symbol: its size and the stroke that goes with it | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::symbol_rendering_mode` | method | `.symbolRenderingMode(_:)` | `NSImage.SymbolConfiguration(hierarchicalColor:)` | `UIImage.SymbolConfiguration` | The rendering mode of one symbol | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image` | struct | `Image(systemName:)` | `NSImage(systemSymbolName:accessibilityDescription:)` | `UIImage` | A symbol as a view; `BODY` in SF points, the label color and the symbol's own SF frame by default | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::data` | fn | `Image(nsImage:)` | `NSImage(data:)` | `UIImage` | An image view from SVG, PNG or JPEG bytes | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::Image::content_transition` | method | `.contentTransition(_:)` | — | — | A system symbol that changes inside `with_animation` crossfades (`Opacity`, default) or swaps at once (`Identity`) | [14](14-animations.md#transitions) |
| `symbol::Image::rgba` | fn | `Image(nsImage:)` | ≈ `NSBitmapImageRep` | ≈ `UIImage` | An image view from straight-alpha RGBA pixels | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::TemplateRenderingMode` | enum | `Image.TemplateRenderingMode` | ≈ `NSImage.isTemplate` | `UIImage.RenderingMode` | `Original` keeps the colors; `Template` paints the shape in the foreground color | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::IconStyle`, `ctappearance::icon_style::IconStyle` | enum | — | — | — | How an app icon is restyled: `Default`, `Dark`, `Clear` or `Tinted` (the icon color); `bk` re-exports the `ctappearance` type | [4](04-appearance-and-tokens.md#icon-styles) |
| `ctappearance::icon_style::IconTone`, `IconStyle::dark` | enum, fn | — | — | — | Light, dark or automatic icons, and whether a style draws dark for a tone and scheme | [4](04-appearance-and-tokens.md#icon-styles) |
| `symbol::Image::icon_style` | fn | ≈ `.grayscale(_:)` and `.colorMultiply(_:)` | — | — | Restyles the image's own colors; has no effect on a template image or a symbol | [4](04-appearance-and-tokens.md#icon-styles) |
| `symbol::Image::app_icon` | fn | — | ≈ `NSWorkspace.icon(forFile:)` | — | Draws an app's own icon on a plate in the user's icon style and color scheme | [4](04-appearance-and-tokens.md#app-icons) |
| `symbol::Image::frame` | fn | `.frame(width:height:)` | — | `NSLayoutConstraint` | Sizes an image that keeps its own colors as a rectangle, not a square | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::Image::system` | fn | `Image(systemName:)` | `NSImage(systemSymbolName:accessibilityDescription:)` | `UIImage(systemName:)` | An image view of a kit `Symbol` | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::rendering_mode` | method | `.renderingMode(_:)` | `NSImage.isTemplate` | `UIImage.withRenderingMode(_:)` | `Template` draws the image in its foreground color; `Original` keeps its own colors | [4](04-appearance-and-tokens.md#images-and-template-rendering) |
| `symbol::Image::font` | method | `.font(_:)` on a symbol | `NSImage.SymbolConfiguration(textStyle:)` | `UIImage.SymbolConfiguration(textStyle:)` | The symbol at a text style's size, in SF points, and weight | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::size` | method | `.font(.system(size:))` on a symbol | `NSImage.SymbolConfiguration(pointSize:weight:)` | `UIImage.SymbolConfiguration(pointSize:)` | The side of the image in points | [4](04-appearance-and-tokens.md#symbols) |
| `symbol::Image::color` | method | `.foregroundStyle(_:)` | `NSImage.SymbolConfiguration(hierarchicalColor:)` | `UIImage.withTintColor(_:)` | The color of a symbol or template image | [4](04-appearance-and-tokens.md#symbols) |
| `ctappearance::icon_plate::Logo`, `Lobe` | struct | — | — | — | A bright or dark icon and the color lobe its plate takes the hue from; `Logo::from_colors` | [4](04-appearance-and-tokens.md#app-icons) |
| `ctappearance::icon_plate::Plate`, `plate` | struct, fn | — | — | — | Plate gradient, rim and glass flag for an icon, an `IconStyle`, whether it is dark and a tint | [4](04-appearance-and-tokens.md#app-icons) |
| `ctappearance::icon_plate::logo_matrix` | fn | — | — | — | Color matrix for the icon on its plate; `None` keeps its colors | [4](04-appearance-and-tokens.md#app-icons) |
| `ctappearance::icon_style::IconStyle::matrix_on_glass` | fn | — | — | — | `IconStyle::matrix` for an icon whose plate the compositor draws as glass: a clear icon also loses its dark body | [4](04-appearance-and-tokens.md#app-icons) |
| `ctappearance::icon_style::IconStyle::luminance_to_alpha` | method | — | — | — | True when the style's matrix makes the alpha follow the luminance, so the result is cut to the image's shape | [4](04-appearance-and-tokens.md#icon-styles) |
| `register_font` | fn | — | — | — | Adds a typeface from font file bytes; `false` if not a font | [4](04-appearance-and-tokens.md#fonts) |

### Symbol variants

Each `Symbol` with its SwiftUI name (`Image(systemName:)`) and the AppKit name of the same symbol (`NSImage(systemSymbolName:)`); they are the same string. A variant drawn before the 2026-11-20 frame (see [Symbols](04-appearance-and-tokens.md#symbols)) is marked ≈ when its drawing only resembles the SF symbol.

| Symbol | Apple name | Drawn |
|---|---|---|
| `MagnifyingGlass` | `magnifyingglass` | before 2026-11-20; redrawn 2026-11-22 |
| `XMark` | `xmark` | before 2026-11-20 |
| `SpeakerWave` | `speaker.wave.2` | before 2026-11-20; redrawn 2026-11-22 |
| `Plus` | `plus` | before 2026-11-20 |
| `Minus` | `minus` | before 2026-11-20 |
| `Ellipsis` | `ellipsis` | before 2026-11-20; redrawn 2026-11-22 |
| `ChevronUpDown` | `chevron.up.chevron.down` | before 2026-11-20 |
| `ChevronRight` | `chevron.right` | before 2026-11-20 |
| `ChevronLeft` | `chevron.left` | before 2026-11-20 |
| `ChevronUp` | `chevron.up` | before 2026-11-20 |
| `ChevronDown` | `chevron.down` | before 2026-11-20 |
| `TriangleLeft` | `arrowtriangle.left.fill` ≈ | before 2026-11-20 |
| `TriangleRight` | `arrowtriangle.right.fill` ≈ | before 2026-11-20 |
| `TriangleUp` | `arrowtriangle.up.fill` ≈ | 2026-11-23 |
| `TriangleDown` | `arrowtriangle.down.fill` ≈ | 2026-11-23 |
| `DiamondFill` | `diamond.fill` | 2026-11-23 |
| `Dot` | `circle.fill` ≈ | before 2026-11-20 |
| `CircleHalf` | `circle.lefthalf.filled` | before 2026-11-20 |
| `Play` | `play.fill` | before 2026-11-20; redrawn 2026-11-22 |
| `Pause` | `pause.fill` | before 2026-11-20; redrawn 2026-11-22 |
| `Shuffle` | `shuffle` | before 2026-11-20; redrawn 2026-11-22 |
| `ChevronLeftForwardSlashChevronRight` | `chevron.left.forwardslash.chevron.right` | before 2026-11-20; redrawn 2026-11-22 |
| `Backward` | `backward.fill` | before 2026-11-20; redrawn 2026-11-22 |
| `Forward` | `forward.fill` | before 2026-11-20; redrawn 2026-11-22 |
| `ListBullet` | `list.bullet` | before 2026-11-20 |
| `SquareGrid` | `square.grid.2x2` ≈ | before 2026-11-20 |
| `SlidersVertical` | `slider.vertical.3` | before 2026-11-20; redrawn 2026-11-22 |
| `SidebarLeft` | `sidebar.left` | before 2026-11-20; redrawn 2026-11-22 |
| `House` | `house` | before 2026-11-20; redrawn 2026-11-22 |
| `Tv` | `tv` | before 2026-11-20; redrawn 2026-11-22 |
| `Film` | `film` | before 2026-11-20; redrawn 2026-11-22 |
| `Star` | `star` | before 2026-11-20 |
| `Clock` | `clock` | before 2026-11-20 |
| `Bookmark` | `bookmark` | before 2026-11-20; redrawn 2026-11-22 |
| `ArrowDownToLine` | `arrow.down.to.line` | before 2026-11-20; named `ArrowDown` until 2026-11-21; redrawn 2026-11-22 |
| `Checkmark` | `checkmark` | before 2026-11-20 |
| `Captions` | `captions.bubble` | before 2026-11-20; redrawn 2026-11-22 |
| `Expand` | `arrow.up.left.and.arrow.down.right` ≈ | before 2026-11-20 |
| `GoBackward` | `gobackward` | before 2026-11-20; redrawn 2026-11-22 |
| `GoForward` | `goforward` | before 2026-11-20; redrawn 2026-11-22 |
| `StarFill` | `star.fill` | before 2026-11-20 |
| `Folder` | `folder` | before 2026-11-20; redrawn 2026-11-22 |
| `Document` | `doc` | before 2026-11-20; redrawn 2026-11-22 |
| `ChevronDoubleRight` | `chevron.forward.2` | before 2026-11-20; redrawn 2026-11-22 |
| `Accessibility` | `accessibility` | before 2026-11-20 |
| `Display` | `display` | before 2026-11-20; redrawn 2026-11-22 |
| `MenubarDockRectangle` | `menubar.dock.rectangle` | before 2026-11-20; redrawn 2026-11-22 |
| `Photo` | `photo` | before 2026-11-20; redrawn 2026-11-22 |
| `Flower` | `camera.macro` ≈ | before 2026-11-20 |
| `Moon` | `moon` | before 2026-11-20 |
| `Bell` | `bell` | before 2026-11-20 |
| `Bed` | `bed.double` | before 2026-11-20; redrawn 2026-11-22 |
| `Person` | `person` | before 2026-11-20 |
| `Briefcase` | `briefcase` | before 2026-11-20; redrawn 2026-11-22 |
| `Printer` | `printer` | before 2026-11-20; redrawn 2026-11-22 |
| `Phone` | `phone` | before 2026-11-20 |
| `SquareOnSquare` | `square.on.square` | before 2026-11-20; redrawn 2026-11-22 |
| `Wifi` | `wifi` | before 2026-11-20 |
| `Bluetooth` | no public name | before 2026-11-20 |
| `Network` | `network` | before 2026-11-20 |
| `Vpn` | no public name | before 2026-11-20 |
| `Lock` | `lock` | before 2026-11-20; redrawn 2026-11-22 |
| `Bolt` | `bolt.fill` | before 2026-11-20 |
| `Battery` | `battery.100` | before 2026-11-20; redrawn 2026-11-22 |
| `Keyboard` | `keyboard` | before 2026-11-20; redrawn 2026-11-22 |
| `QuestionMark` | `questionmark` | before 2026-11-20 |
| `EllipsisCircle` | `ellipsis.circle` | before 2026-11-20 |
| `Square` | `square` | before 2026-11-20; redrawn 2026-11-22 |
| `Circle` | `circle` | before 2026-11-20; redrawn 2026-11-22 |
| `CapsuleOnRectangle` | `capsule.on.rectangle` | before 2026-11-20; redrawn 2026-11-22 |
| `CapsuleOnRectangleFill` | `capsule.on.rectangle.fill` | before 2026-11-20; redrawn 2026-11-22 |
| `ICloud` | `icloud` | 2026-11-20 |
| `ArrowDownCircle` | `arrow.down.circle` | 2026-11-20 |
| `MusicNote` | `music.note` | 2026-11-20 |
| `InternalDrive` | `internaldrive` | 2026-11-20 |
| `ExternalDrive` | `externaldrive` | 2026-11-20 |
| `Trash` | `trash` | 2026-11-20 |
| `Tag` | `tag` | 2026-11-20 |
| `SquareAndArrowUp` | `square.and.arrow.up` | 2026-11-20 |
| `SquareAndArrowDown` | `square.and.arrow.down` | 2026-11-20 |
| `ArrowUpArrowDown` | `arrow.up.arrow.down` | 2026-11-20 |
| `Line3HorizontalDecrease` | `line.3.horizontal.decrease` | 2026-11-20 |
| `InfoCircle` | `info.circle` | 2026-11-20 |
| `PlusCircle` | `plus.circle` | 2026-11-20 |
| `MinusCircle` | `minus.circle` | 2026-11-20 |
| `XMarkCircle` | `xmark.circle` | 2026-11-20 |
| `CheckmarkCircle` | `checkmark.circle` | 2026-11-20 |
| `ExclamationMarkCircle` | `exclamationmark.circle` | 2026-11-20 |
| `QuestionMarkCircle` | `questionmark.circle` | 2026-11-20 |
| `ExclamationMarkTriangle` | `exclamationmark.triangle` | 2026-11-20 |
| `NoSign` | `nosign` | 2026-11-20 |
| `ArrowLeft` | `arrow.left` | 2026-11-20 |
| `ArrowRight` | `arrow.right` | 2026-11-20 |
| `ArrowUp` | `arrow.up` | 2026-11-20 |
| `ArrowUpForward` | `arrow.up.forward` | 2026-11-20 |
| `Heart` | `heart` | 2026-11-20 |
| `Key` | `key` | 2026-11-20 |
| `Mic` | `mic` | 2026-11-20 |
| `Camera` | `camera` | 2026-11-20 |
| `Envelope` | `envelope` | 2026-11-20 |
| `Calendar` | `calendar` | 2026-11-20 |
| `Flag` | `flag` | 2026-11-20 |
| `PaperPlane` | `paperplane` | 2026-11-20 |
| `Scissors` | `scissors` | 2026-11-20 |
| `XMarkCircleFill` | `xmark.circle.fill` | 2026-11-20 |
| `PlusCircleFill` | `plus.circle.fill` | 2026-11-20 |
| `MinusCircleFill` | `minus.circle.fill` | 2026-11-20 |
| `CheckmarkCircleFill` | `checkmark.circle.fill` | 2026-11-20 |
| `ExclamationMarkCircleFill` | `exclamationmark.circle.fill` | 2026-11-20 |
| `InfoCircleFill` | `info.circle.fill` | 2026-11-20 |
| `QuestionMarkCircleFill` | `questionmark.circle.fill` | 2026-11-20 |
| `ExclamationMarkTriangleFill` | `exclamationmark.triangle.fill` | 2026-11-20 |
| `GearShape` | `gearshape` | 2026-11-20 |
| `ArrowClockwise` | `arrow.clockwise` | 2026-11-20 |
| `ArrowCounterclockwise` | `arrow.counterclockwise` | 2026-11-20 |
| `TextAlignLeft` | `text.alignleft` | 2026-11-20 |
| `TextAlignCenter` | `text.aligncenter` | 2026-11-20 |
| `TextAlignRight` | `text.alignright` | 2026-11-20 |
| `TextJustify` | `text.justify` | 2026-11-20 |
| `MacWindow` | `macwindow` | 2026-11-20 |
| `SidebarRight` | `sidebar.right` | 2026-11-20 |
| `DocOnDoc` | `doc.on.doc` | 2026-11-20 |
| `ArrowUturnBackward` | `arrow.uturn.backward` | 2026-11-20 |
| `ArrowUturnForward` | `arrow.uturn.forward` | 2026-11-20 |
| `DesktopComputer` | `desktopcomputer` | 2026-11-20 |
| `PersonCropCircle` | `person.crop.circle` | 2026-11-20 |
| `Eject` | `eject` | 2026-11-20 |
| `ArrowDown` | `arrow.down` | 2026-11-21 |
| `Link` | `link` | 2026-11-21 |
| `Pencil` | `pencil` | 2026-11-21 |
| `SquareAndPencil` | `square.and.pencil` | 2026-11-21 |
| `LockOpen` | `lock.open` | 2026-11-21 |
| `LockFill` | `lock.fill` | 2026-11-21 |
| `StopFill` | `stop.fill` | 2026-11-21 |
| `SpeakerSlash` | `speaker.slash` | 2026-11-21 |
| `SpeakerWave1` | `speaker.wave.1` | 2026-11-21 |
| `Repeat` | `repeat` | 2026-11-21 |
| `CheckmarkSquare` | `checkmark.square` | 2026-11-21 |
| `Tray` | `tray` | 2026-11-21 |
| `DocText` | `doc.text` | 2026-11-21 |
| `XMarkRectangle` | `xmark.rectangle` | 2026-11-21 |
| `PlusRectangle` | `plus.rectangle` | 2026-11-21 |
| `RectangleOnRectangle` | `rectangle.on.rectangle` | 2026-11-21 |
| `RectangleDashed` | `rectangle.dashed` | 2026-11-21 |
| `MacWindowOnRectangle` | `macwindow.on.rectangle` | 2026-11-21 |
| `ArrowRightCircle` | `arrow.right.circle` | 2026-11-21 |
| `ArrowLeftCircle` | `arrow.left.circle` | 2026-11-21 |
| `ArrowUpCircle` | `arrow.up.circle` | 2026-11-21 |
| `ChevronRightCircle` | `chevron.right.circle` | 2026-11-21 |
| `ChevronLeftCircle` | `chevron.left.circle` | 2026-11-21 |
| `PlayCircle` | `play.circle` | 2026-11-21 |
| `PauseCircle` | `pause.circle` | 2026-11-21 |
| `HeartFill` | `heart.fill` | 2026-11-21 |
| `CameraFill` | `camera.fill` | 2026-11-21 |
| `Bold` | `bold` | 2026-11-24 |
| `Italic` | `italic` | 2026-11-24 |
| `TextFormatSize` | `textformat.size` | 2026-11-24 |
| `GearShape2` | `gearshape.2` | 2026-11-24 |

## Helpers

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `init_tracing` | fn | — | — | — | Installs logging to stderr by `RUST_LOG`, and Tracy with `tracy` | [4](04-appearance-and-tokens.md#helpers) |
| `ProbeExt` | trait | `.accessibilityIdentifier(_:)` | `accessibilityIdentifier()` | `UIAccessibilityIdentification.accessibilityIdentifier` | `probe(id)`: sets a `rect`'s accessibility id for tests | [4](04-appearance-and-tokens.md#helpers) |

## Appearance settings

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::AppearanceConfig` | struct | — | — | — | The user's `appearance.domain`: mode, accent, tints, contrast, motion | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ColorScheme` | enum | ≈ `ColorScheme` | ≈ `NSAppearance` | ≈ `UITraitCollection` | `Light`, `Dark`, `DarkTint`; SwiftUI has only light and dark | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ColorScheme::is_dark` | method | — | — | — | True for `Dark` and `DarkTint` | [4](04-appearance-and-tokens.md#the-systems-light-or-dark) |
| `ctappearance::set_system_scheme` | fn | — | ≈ `NSApp.effectiveAppearance` | — | Records the light or dark choice of the desktop (macOS, the freedesktop portal); `None` when it states none | [4](04-appearance-and-tokens.md#the-systems-light-or-dark) |
| `ctappearance::local_date` | fn | — | — | — | Today's local date as year, month and day | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::WindowColorMode` | enum | — | — | — | Window tint source: `Neutral` or `Accent` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TintStrength` | enum | — | — | — | `Subtle`, `Normal`, `Strong`, `Intense`; `factor()` from `TINT_STRENGTH` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TintParts` | struct | — | — | — | Which parts take the tint: sidebars, window, groups, controls, menus, and the bars (menu bar and Dock) | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::SidebarGlass` | struct | ≈ `Glass` | ≈ `NSGlassEffectView` | ≈ `UIGlassEffect` | The `base`, `solid` and `chroma` of a sidebar's glass panel; `Resolved.sidebar.glass` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TrafficLightStyle` | enum | — | — | — | `Standard`, `Accent`, `Wallpaper` traffic-light colors | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::TextSizes` | struct | — | ≈ the per-app list of Accessibility › Display › Text size | — | The optional BarrierKit text size | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::AppearanceConfig::scheme_at`, `current_scheme` | method | — | ≈ `NSApp.effectiveAppearance` | ≈ `UITraitCollection.userInterfaceStyle` | The color scheme at an hour, or now in local time, with `automatic_scheme` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::AppearanceConfig::transparency_reduced` | method | — | ≈ `NSWorkspace.accessibilityDisplayShouldReduceTransparency` | ≈ `UIAccessibility.isReduceTransparencyEnabled` | True with `reduce_transparency` or `contrast` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ScrollBars` | enum | ≈ `ScrollIndicatorVisibility` | ≈ `NSScroller.Style` | ≈ `UIScrollView.showsVerticalScrollIndicator` | Show scroll bars `Automatic` (by mouse or touchpad), `WhenScrolling` or `Always` | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctappearance::ScrollBarClick` | enum | — | ≈ `AppleScrollerPagingBehavior` | — | A click on the track goes to the `NextPage` or the `ClickedSpot` | [4](04-appearance-and-tokens.md#the-appearance-file) |

## Appearance files and watch

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::config_dir` | fn | — | — | — | `$MALAYSIA_CONFIG_DIR` or `$XDG_CONFIG_HOME/malaysia` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::data_dir` | fn | — | — | — | `$MALAYSIA_DATA_DIR` or `$XDG_DATA_HOME/malaysia` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::appearance_path` | fn | — | — | — | Path of `appearance.domain` | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::wallpaper_state_path` | fn | — | — | — | Path of `wallpaper.state`, the wallpaper's dominant color and grid | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::load_appearance` | fn | — | — | — | Reads the appearance file, or the default on any error | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::replace_file` | fn | — | — | — | Replaces a file atomically with new text | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::create_private_dir` | fn | — | — | — | Creates a folder and its parents with permission 0700 | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::load_dominant` | fn | — | — | — | Reads the wallpaper's dominant color | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::WallpaperState::grid_for` | method | — | — | — | The color grid of a desktop, or the shared one | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::file_uri`, `file_path` | fn | — | ≈ `g_filename_to_uri` | — | A path as a `file://` URI escaped like GLib, and back | [4](04-appearance-and-tokens.md#reading-and-writing) |
| `ctappearance::watch::AppearanceWatch` | struct | — | — | — | inotify watch on the config and data folders; `watch_app_folder(path)`, `drain()` | [4](04-appearance-and-tokens.md#watching) |
| `ctappearance::watch::AppearanceEvent` | enum | — | — | — | `Appearance`, `Wallpaper`, `Defaults(Option<OsString>)`; `restyles()` | [4](04-appearance-and-tokens.md#watching) |

## Appearance resolution

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::resolve` | fn | — | — | — | Turns the config and wallpaper color into concrete colors | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::Resolved` | struct | ≈ `EnvironmentValues` | — | ≈ `UITraitCollection` | Resolved appearance: accent, focus ring, link, tints, surfaces, palettes | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::Resolved::behind_window` | method | — | ≈ desktop tinting of `NSVisualEffectView.Material.sidebar` | ≈ `UIBlurEffect.Style` | Tints a window's `tint_parts` with the wallpaper behind it | [4](04-appearance-and-tokens.md#wallpaper-behind-a-window) |
| `ctappearance::Surfaces` | struct | — | — | — | Tinted window, sunken, surface, raised, control, hover and line colors | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::SidebarSurface` | struct | — | — | — | Sidebar style, background, rim, inner line and palette | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::ToolgroupSurface` | struct | — | — | — | Toolbar group background and palette | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::palette` | fn | — | — | — | Base `ctdesign::Palette` of a color scheme | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::palette_in` | fn | — | — | — | `palette`, with the Increase Contrast colors when `contrast` is true | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::Resolved::menu_separator` | method | — | — | — | The separator ink of a menu over its panel color | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::glass_chroma` | fn | — | — | — | The chroma of `ctdesign::GLASS` a glass panel takes in a scheme, and its cap | [4](04-appearance-and-tokens.md#view-metrics) |

## Window behavior

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::window::WindowBehavior` | struct | — | — | — | `window.domain`: titlebar double click and green button actions | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::read_window_behavior` | fn | — | — | — | Reads `window.domain`, or the default | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::DoubleClickAction` | enum | — | — | — | Titlebar double click: `Fill`, `Minimize`, `Zoom`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::GreenButtonAction` | enum | — | — | — | Green light: `Focus`, `Zoom`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::TitlebarAction` | enum | — | — | — | What to do: `Focus`, `Maximize`, `Minimize`, `None` | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::green_action` | fn | — | — | — | Green-button action for a window, fixed width or not | [4](04-appearance-and-tokens.md#window-behavior) |
| `ctappearance::window::double_click_action` | fn | — | — | — | Double-click action for a window, zoomable or not | [4](04-appearance-and-tokens.md#window-behavior) |

## Traffic-light geometry

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::traffic::inset` | fn | — | — | — | Distance of the lights from the left edge of a bar: 9 in the 32 px bar, 12 in the 40 px bar | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::corner::concentric` | fn | `RoundedRectangularShapeCorners.concentric(minimum:)` | `NSViewCornerRadius.containerConcentric(_:)` | `UICornerConfiguration.corners(radius:)` | The radius of a nested corner from its container's radius and insets | [4](04-appearance-and-tokens.md#corners) |
| `ctappearance::traffic::button_rect` | fn | — | ≈ `NSWindow.standardWindowButton(_:)` frame | — | Rectangle of one light | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::visual` | fn | — | — | — | Fill and ring stops, glyph, scale and dodge of each light for an input | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::looks` | fn | — | — | — | The three lights of a style: the measured look, or recolored with the accent's or the wallpaper's tone | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::Glyph` | enum | — | — | — | The glyph of a light: close, minimize, zoom enter, zoom exit (full screen) and plus | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::rounded` | fn | — | — | — | The corners of a zoom triangle, rounded | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::Corner` | struct | — | — | — | Enter, vertex and exit of one rounded corner | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::TrafficInput` | struct | — | — | — | Focus, group hover, pressed light and zoom support | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::ButtonVisual` | struct | — | — | — | Fill and ring stops, optional glyph, scale and dodge of one light | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::TrafficPointer` | struct | — | — | — | Pointer state machine for hover, arm and click of the lights | [4](04-appearance-and-tokens.md#traffic-light-geometry) |
| `ctappearance::traffic::TrafficPointer::on_group_enter`, `on_button_enter`, `on_leave_button`, `on_leave`, `on_release`, `pressed_visual` | method | — | — | — | The pointer events of the lights; each returns whether the look changed, `on_release` the light that was clicked, `pressed_visual` the light drawn pressed | [4](04-appearance-and-tokens.md#traffic-light-geometry) |

## Backdrop colors

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::backdrop::Backdrop` | struct | — | — | — | Three gradient layers and a base color derived from an image | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::ImagePalette` | struct | — | — | — | Three representative colors and the most vivid one of an image | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::Oklab` | struct | — | — | — | A color in the OKLab space | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::Oklab::from_rgba`, `to_rgba`, `to_linear` | method | — | — | — | sRGB to OKLab and back; `to_linear` is not clamped, so a channel outside 0 to 1 is out of gamut | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::take_chroma`, `take_chroma_capped` | fn | — | — | — | A gray that keeps its lightness and takes part of a source's hue; the capped one saturates smoothly towards a cap ([Glass](10-sidebar.md#glass)) | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::Backdrop::highlight`, `highlights` | method | — | — | — | The highlight of a color over the backdrop, and of each of its layers | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::WallpaperTinting` | struct | — | — | — | How much of the wallpaper behind it a window takes: `chroma` and `lift` | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::backdrop::WallpaperTinting::take` | method | — | — | — | Tints a gray with the wallpaper behind it, as macOS 26 tints a window background | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::backdrop::WallpaperTinting::sidebar` | method | — | — | — | The sidebar's share of a tinting | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::backdrop::lift` | fn | — | — | — | Moves a color's OKLab lightness part of the way to `WALLPAPER_TINTING_TARGET` | [4](04-appearance-and-tokens.md#resolution) |
| `ctappearance::backdrop::palette_from_pixels` | fn | — | — | — | Image palette from RGBA8 pixels, by k-means in OKLab | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::dominant_from_pixels` | fn | — | — | — | Dominant chromatic color of RGBA8 pixels | [4](04-appearance-and-tokens.md#backdrop-colors) |
| `ctappearance::backdrop::ink_on` | fn | — | — | — | Light or dark ink with more contrast on a background | [4](04-appearance-and-tokens.md#backdrop-colors) |

## Language and directories

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctappearance::language::read_language` | fn | — | — | — | The `language` key of a `.domain` file | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::language::system_language` | fn | — | — | — | The language chosen in `language.domain` | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::language::requested_languages` | fn | — | — | — | App, system or the given preferred languages as tags, in order | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::language::locale_languages` | fn | — | — | — | The POSIX-locale languages as tags, gettext order | [4](04-appearance-and-tokens.md#language) |
| `ctappearance::xdg::config_home` | fn | — | — | — | `$XDG_CONFIG_HOME` or `~/.config` | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::xdg::data_home` | fn | — | — | — | `$XDG_DATA_HOME` or `~/.local/share` | [4](04-appearance-and-tokens.md#directories) |
| `ctappearance::xdg::data_dirs` | fn | — | — | — | `$XDG_DATA_DIRS` or `/usr/local/share` and `/usr/share` | [4](04-appearance-and-tokens.md#directories) |

## Design token types

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctdesign::Rgba` | struct | ≈ `Color` | ≈ `NSColor` | ≈ `UIColor` | sRGB color, channels 0–1, with mixing, luma and contrast helpers | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Rgba::blend`, `darken`, `lighten`, `to_rgba8`, `to_linear`, `from_linear`, `relative_luminance`, `contrast_ratio`, `to_hsl`, `from_hsl`, `Rgba::TRANSPARENT`, `LUMA` | method, constant | — | ≈ `NSColor.blended(withFraction:of:)` | — | Mixing, lighter and darker steps, 8-bit and linear sRGB, HSL, the luma weights and the WCAG contrast of a color | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Font` | struct | ≈ `Font.TextStyle` | ≈ `NSFont.TextStyle` | ≈ `UIFont.TextStyle` | Type token: family, size, line height, weight | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Font::scaled` | method | — | — | — | The font with size and line height times a factor | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctdesign::Palette` | struct | — | — | — | Base colors of one theme: inks, lines, accent, titlebar | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::FocusPair` | struct | — | — | — | One color for a focused window, one for unfocused | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::PanelLine` | struct | — | — | — | Inner line of a focused panel: sides, edges and glow | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficLooks` | struct | — | — | — | The close, minimize, zoom and inactive `TrafficLook` of a scheme, their halo and the backdrop of a pressed light | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficLook` | struct | — | — | — | Base tone, fill stops, ring stops and glyph color of one traffic light | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficHalo` | struct | — | — | — | Color, blur and vertical offset of the shadow around a traffic light | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficPress` | struct | — | — | — | Scale and dodge of a pressed traffic light | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ShellPalette` | struct | — | — | — | Menu bar and Dock colors | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::MenuPalette` | struct | — | — | — | Menu colors and metrics | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::GlassControl`, `GLASS_CONTROL` | struct, const | — | — | — | A control on its own glass on macOS 27: full-strength label ink, a toolbar item's glass height, the toolbar segmented control's thumbs (inset, gap, selected and hover veils) and the toolbar search field's padding and gap | [15](15-backdrops-and-media.md#on-macos) |
| `ctdesign::SidebarPalette` | struct | — | — | — | Sidebar item, selection, section and line inks | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ToolgroupPalette` | struct | — | — | — | Toolbar group separator and rim | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::FrameLines`, `FRAME_LINES_{LIGHT,DARK,DARK_TINT}`, `FRAME_LINES_CONTRAST_*`, `ctdesign::CalendarPalette`, `RelevancyPalette`, `GaugeContrast`, `PressedPalette`, `DisabledPalette`, `ContrastPalette` | struct | — | — | — | Colors of a theme for the graphical date picker, the relevancy indicator, the gauges with Increase Contrast, pressed controls, disabled controls and Increase Contrast | [4](04-appearance-and-tokens.md#the-palette) |
| `ctdesign::Palette::contrasted` | method | ≈ `colorSchemeContrast` | ≈ `accessibilityDisplayShouldIncreaseContrast` | — | The palette with the colors of a `ContrastPalette` in place | [4](04-appearance-and-tokens.md#the-palette) |
| `ctdesign::TitlebarToolbarStart`, `TITLEBAR_TOOLBAR_START`, `TITLEBAR_FULLSCREEN_START` | struct, constant | — | — | — | Where a toolbar's first item starts after the lights in the unified, compact and expanded bars, and from the edge in full screen | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::TitlebarRevealMetrics` | struct | — | — | — | Pill, reveal zone, corner and resting edges of a hidden title bar | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::ToolbarItemHighlight`, `TOOLBAR_ITEM_HIGHLIGHT` | struct, constant | — | — | — | The circle under the pointer and under a press of a toolbar item on glass; an item alone on its glass lights the whole capsule | [9](09-titlebars.md#toolbar-items) |
| `ctdesign::GlassSelection`, `SIDEBAR_GLASS_SELECTION_LIGHT`, `SIDEBAR_GLASS_SELECTION_DARK` | struct, constant | — | — | — | The veil a sidebar on glass lays over a selection without focus, and in an inactive window | [10](10-sidebar.md#glass) |
| `ctdesign::GlassMetrics`, `GLASS` | struct, constant | — | — | — | The sidebar glass over a backdrop: chroma, lightness, blur passes, edge and shadow per scheme | [4](04-appearance-and-tokens.md#glass-over-a-backdrop) |
| `ctdesign::IconPlateMetrics`, `ICON_PLATE`, `IconPlateLook`, `ICON_PLATE_*`, `IconLogoEnds`, `ICON_LOGO_*`, `IconStyleMetrics`, `ICON_STYLE` | struct, constant | — | — | — | The plate behind a restyled app icon (body, corner, shadows, the look of each style) and the tint and cut of the dark, tinted and clear styles | [4](04-appearance-and-tokens.md#icon-styles) |
| `ctdesign::CircularSliderMetrics`, `CIRCULAR_SLIDER`, `ColorPickerMetrics`, `COLOR_PICKER`, `HelpLinkMetrics`, `HELP_LINK` | struct, constant | — | — | — | Metrics of the circular slider, the color well and its wheel, and the help button's glyph | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TabViewMetrics`, `TabViewSize`, `TAB_VIEW`, `ListSectionMetrics`, `LIST_SECTION` | struct, constant | — | — | — | The tab view's strip and pill per size and state; a list section's header height and gap | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::GestureMetrics`, `GESTURE` | struct, constant | — | — | ≈ `UIGestureRecognizer` | The distance and time limits of the tap, long press, drag, magnify and rotate gestures | [5](05-controls.md#gestures) |
| `ctdesign::ScrollPhysicsMetrics`, `SCROLL_PHYSICS`, `SCROLL_FLING_POWER`, `SCROLL_SETTLED_OVER`, `SCROLL_SETTLED_SPEED` | struct, constant | — | ≈ `NSScrollView.verticalScrollElasticity` | ≈ `UIScrollView.decelerationRate` | The fling, rubber band, return, settle search, wheel line step and pull-to-refresh constants of a scroll view | [5](05-controls.md#scrolling) |
| `ctdesign::ShadowLayer` | struct | ≈ `.shadow(color:radius:x:y:)` | ≈ `NSShadow` | ≈ `NSShadow` | One shadow: blur, offset, opacity, color | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ShadowStyle` | struct | — | — | — | Ambient and contact shadows, scaled when unfocused | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::MaterialThickness` | struct | ≈ `Material` | ≈ `NSVisualEffectView.Material` | ≈ `UIBlurEffect.Style` | Opacity and blur of one material thickness | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::WindowCorner` | struct | — | — | — | Window corner radii and continuous-curve shape | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::WindowStyle` | struct | — | — | — | Corner radius and superellipse exponent of a dialog or pop-up | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::CornerShape` | struct | `RoundedRectangle` (`cornerRadius`, `style`) | `CALayer.cornerRadius`, `cornerCurve` | `UIBezierPath`, `CALayer.cornerRadius` | The nominal radius, the radius that draws the measured silhouette and the smoothing of one element | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::CornerControls` | struct | — | `NSControl.ControlSize` | — | The corner radius of each control at the five control sizes | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::WindowResize` | struct | — | — | — | Width of the resize band at edges and corners | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Titlebar` | struct | — | — | — | Server-side titlebar thickness, lights and font | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TitlebarButtons` | struct | — | — | — | Order, diameter, spacing and left insets (`inset`, `inset_compact`) of the traffic lights | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Button` | struct | — | — | — | One traffic light: kind | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::ButtonKind` | enum | — | `NSWindow.ButtonType` | — | `Close`, `Minimize`, `Maximize` | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::TrafficGlyphs` | struct | — | — | — | Geometry of the close, minimize, zoom and plus glyphs | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Segment` | struct | — | — | — | A line from point `a` to point `b` | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::Triangle` | struct | — | — | — | Three points of a filled glyph triangle | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::DoubleClick` | struct | — | ≈ `NSEvent.doubleClickInterval` | — | Double-click interval and pointer slop | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsStrength` | struct | — | — | — | Tint-strength scale of the settings app | [4](04-appearance-and-tokens.md#token-types) |
| `ctdesign::SettingsMetrics` | struct | — | — | — | Sizes of the settings window, sidebar and search | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ButtonMetrics` | struct | — | — | — | Push button sizes, radius, font and pressed scale; one `ButtonSize` per size and the pop-up rows; `size(ControlSize)` picks a row | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BezelRow`, `BezelMetrics` | struct | — | — | — | Height, padding, radius and label size of one button bezel, and the five of them (`BUTTON.bezel`) | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BezelPalette`, `BEZEL_{LIGHT,DARK,DARK_TINT}`, `BEZEL_CONTRAST_{LIGHT,DARK,DARK_TINT}` | struct, constant | — | — | — | Fills, borders and pressed fills of the small square, textured and inline bezels per look | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ControlSize` | enum | `ControlSize` | `NSControl.ControlSize` | — | `Mini`, `Small`, `Regular`, `Large`, `ExtraLarge`; the key of every `size(ControlSize)` lookup; `row([mini, small, large, xl])` picks a row, `None` at `Regular` | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ButtonSize` | struct | — | — | — | Height, padding, font size and radius of a push button, pop-up or segmented control at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::PopupSize`, `PopupSizes` | struct | — | — | — | Arrow glyph, indicator box, trailing padding, gap and pull-down glyph of a pop-up at one size, and the five of them | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::CheckboxSize` | struct | — | — | — | Box, radius, mark, label gap, label font and radio dot of a checkbox or radio at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SwitchSize` | struct | — | — | — | Track, knob and inset of a switch at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TextFieldSize` | struct | — | — | — | Height, font size and line height of a text field at `Mini` or `Small` | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SearchFieldMetrics` | struct | — | — | — | The `Mini`, `Small`, `Large` and `ExtraLarge` rows of a search field | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SearchFieldSize` | struct | — | — | — | Height, font, magnifier and clear button of a search field at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ComboBoxSize` | struct | — | — | — | Height, font, arrow button and chevron of a combo box at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SliderSize` | struct | — | — | — | Height, track and knob width of a slider at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SegmentedSize` | struct | — | — | — | Height, padding, radius, divider and font of a segmented control at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::StepperSize` | struct | — | — | — | Arrows box, radius and chevron of a stepper at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::IconButtonMetrics` | struct | — | — | — | Icon button size, radius and icon size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SwitchMetrics` | struct | — | — | — | Switch track, knob and transition | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::CheckboxMetrics` | struct | — | — | — | Checkbox size, radius and mark | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SliderMetrics` | struct | — | — | — | Slider height, track and knob | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SliderFatMetrics` | struct | — | — | — | Wide level slider height, minimum fill and icon position | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SegmentedMetrics` | struct | — | — | — | Segmented control padding, segments and font | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::StepperMetrics` | struct | — | — | — | Stepper size, radius and chevrons | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TextEditorMetrics` | struct | — | — | — | Text editor padding and line height | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TextFieldMetrics` | struct | — | — | — | Text field height, padding and radius | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TokenFieldMetrics` | struct | — | — | — | Token height, padding, gap and opacity | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SpellingMetrics` | struct | — | — | — | Size of the misspelling dot | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ComboBoxMetrics` | struct | — | — | — | Combo box button and chevron sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::PathControlMetrics` | struct | — | — | — | Path control height, chevrons and icons | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::DatePickerSize` | struct | — | — | — | Field height and stepper of a date picker at one size | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::DatePickerMetrics` | struct | — | — | — | Calendar size, header, weekday row and radii | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ImageWellMetrics` | struct | — | — | — | Image well size, radius, inset and drop border | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::LevelIndicatorMetrics` | struct | — | — | — | Level indicator bars, stars and relevancy sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::GaugeMetrics` | struct | — | — | — | Gauge size, stroke, arc and marker | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::ProgressMetrics` | struct | — | — | — | Progress bar, ring and spinner sizes and timing | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BadgeMetrics` | struct | — | — | — | Badge height and padding | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::FocusRingMetrics` | struct | — | — | — | Focus ring width | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SymbolMetrics` | struct | — | — | — | Symbol grid and stroke width | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TooltipMetrics` | struct | — | — | — | Tooltip delay, padding, radius, pointer gap, wrapping and colors per appearance | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::PopoverMetrics` | struct | — | — | — | Popover radius, arrow and padding | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::MenuPopoverMetrics` | struct | — | — | — | Pop-up menu item, padding, check and swatch grid sizes | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::DragMetrics` | struct | — | — | — | Pointer travel before a drag starts | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::FullscreenBarMetrics` | struct | — | — | — | Content offset and band line of a full-screen bar | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::FullscreenGlassMetrics` | struct | — | — | — | Height, blur, layers, tint and mask of full-screen glass | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::SidebarMetrics` | struct | — | — | — | Sidebar width, traffic lights, items and sections | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ToolgroupMetrics` | struct | — | — | — | Toolbar group height, padding and separators | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SplitViewMetrics` | struct | — | — | — | Sidebar limits, collapse width, inspector, grip and key step | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SplitPanesMetrics` | struct | — | — | — | Divider width and minimum pane size | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::TableMetrics` | struct | — | — | — | Table rows, header, zebra and column sizes | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::CollectionMetrics` | struct | — | — | — | Collection tile, icon, gap, radius and selection | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ListGroupMetrics` | struct | — | — | — | Form group radius, rows, separators and headers | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::GroupBoxMetrics` | struct | — | — | — | Group box radius, padding, label and fill | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::DisclosureMetrics` | struct | — | — | — | Disclosure indent, rows, chevron and icon | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ToolbarTabsMetrics` | struct | — | — | — | Toolbar tab container and tab sizes | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::ScrollBarMetrics` | struct | — | — | — | Scroll bar thumb, track, opacity and hide delay | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::FontPanelMetrics` | struct | — | — | — | Font panel size list and size column | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MediaMetrics` | struct | — | — | — | Media buttons, glass material, inks and scrims | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MaterialMetrics` | struct | — | — | — | One `MaterialThickness` per `styles::Thickness` | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::BackdropMetrics` | struct | — | — | — | Backdrop gradient, glass, ink and accent parameters | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::BackdropScenes` | struct | — | — | — | Preset backdrop gradient: dusk | [4](04-appearance-and-tokens.md#view-metrics) |

## Design token constants

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `ctdesign::SPACE_1`, `SPACE_2`, `SPACE_3`, `SPACE_4` | constant | — | — | — | Spacing steps: 4, 8, 12, 16 | [4](04-appearance-and-tokens.md#spacing) |
| `ctdesign::HAIRLINE` | constant | — | — | — | Separator width, 1 px | [4](04-appearance-and-tokens.md#spacing) |
| `ctdesign::LARGE_TITLE`, `TITLE`, `TITLE_2`, `TITLE_3`, `HEADLINE`, `SUBHEADLINE`, `BODY`, `CALLOUT`, `FOOTNOTE`, `CAPTION`, `CAPTION_2` | constant | `Font.TextStyle` | `NSFont.TextStyle` | `UIFont.TextStyle` | Type styles named after SwiftUI's | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::TOOLBAR_TITLE`, `ABOUT_NAME` | constant | — | — | — | The window title of a unified toolbar and the app name of `AboutView` | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::MONO` | constant | `Font.monospaced()` | `NSFont.monospacedSystemFont(ofSize:weight:)` | `UIFont.monospacedSystemFont(ofSize:weight:)` | Monospaced body text, Geist Mono | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::SANS_FAMILIES`, `TEXT_LINE_HEIGHT` | constant | — | — | — | Sans-serif fallback list; line-height factor for measuring text | [4](04-appearance-and-tokens.md#type) |
| `ctdesign::DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS` | constant | — | — | — | Durations: 120, 250, 420 ms | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::ease_default`, `ease_out_appkit` | fn | `UnitCurve.easeInOut` | `CAMediaTimingFunction(name: .default)`, `.easeOut` | same | Core Animation's default Bézier (0.25, 0.1, 0.25, 1) and `easeOut` (0, 0, 0.58, 1) of a progress from 0 to 1; AppKit's name for the second, because `ease_out` is the cubic one. `DisclosureGroup` and `SidebarSection` use the first; an `OutlineView` expansion the second | [14](14-animations.md#what-animates-in-the-kit) |
| `ctdesign::DURATION_DISCLOSURE_MS`, `DURATION_OUTLINE_EXPAND_MS` | constant | — | — | — | Measured durations of a disclosure and an outline expansion: 248, 200 ms | [14](14-animations.md#what-animates-in-the-kit) |
| `ctdesign::DURATION_LIST_ROW_MS` | constant | `List` rows in `withAnimation` | ≈ `NSTableView.insertRows(at:withAnimation:)` | — | Measured SwiftUI list row change on macOS: 248 ms | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::SPRING_SHEET_MS`, `SPRING_PUSH_MS` | constant | ≈ `Spring.duration` | — | ≈ `UISheetPresentationController`, `UINavigationController` | Perceptual duration of the measured Mac Catalyst springs (bounce 0): 344, 251 ms | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::SCROLL_POCKET_BLUR`, `SCROLL_POCKET_SCALE_LIGHT`, `SCROLL_POCKET_OFFSET_LIGHT`, `SCROLL_POCKET_SCALE_DARK`, `SCROLL_POCKET_OFFSET_DARK` | constant | ≈ `scrollEdgeEffectStyle(_:for:)` | ≈ `NSScrollPocket` (private), `preferredScrollEdgeEffectStyle` | ≈ `UIScrollEdgeEffect` | Blur sigma in points and the tone line of the scroll pocket, light and dark: 3.4, 0.5x + 0.5349 and 0.4x + 0.0976 | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::ScrollEdgeMetrics`, `SCROLL_EDGE` | struct, constant | ≈ `scrollEdgeEffectStyle(_:for:)` | ≈ `NSScrollPocket` (private) | — | The soft edge's fade and the full-screen sidebar pocket of the native macOS materials | [4](04-appearance-and-tokens.md#motion) |
| `ctdesign::LIGHT`, `DARK`, `DARK_TINT` | constant | — | — | — | Base palette of each theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::CONTRAST_LIGHT`, `CONTRAST_DARK`, `CONTRAST_DARK_TINT` | constant | ≈ `colorSchemeContrast` | ≈ `accessibilityDisplayShouldIncreaseContrast` | — | The `ContrastPalette` of each theme that `Palette::contrasted` puts in place under Increase Contrast | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::DISABLED_{LIGHT,DARK,DARK_TINT}`, `DISABLED_CONTRAST_*`, `PRESSED_{LIGHT,DARK,DARK_TINT}`, `PRESSED_CONTRAST_*` | constant | — | — | — | The `DisabledPalette` and `PressedPalette` of each theme, and of each with Increase Contrast | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::THOUSANDTHS` | constant | — | — | — | 1000: a fraction travels in the window protocol as thousandths in a `u32` | [18](18-platform.md#wayland-protocols) |
| `ctdesign::ACCENT_CHROMA_GAIN` | constant | — | — | — | Factor on the share of the accent's chroma a window takes when the accent tints it: 2.5 | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::BG_WINDOW_LIGHT`, `BG_WINDOW_DARK`, `BG_WINDOW_DARK_TINT`, `BG_SURFACE_LIGHT`, `BG_SURFACE_DARK`, `BG_SURFACE_DARK_TINT`, `BG_SUNKEN_LIGHT`, `BG_SUNKEN_DARK`, `BG_SUNKEN_DARK_TINT`, `BG_RAISED_LIGHT`, `BG_RAISED_DARK`, `BG_RAISED_DARK_TINT`, `BG_CONTROL_LIGHT`, `BG_CONTROL_DARK`, `BG_CONTROL_DARK_TINT`, `BG_HOVER_LIGHT`, `BG_HOVER_DARK`, `BG_HOVER_DARK_TINT` | constant | — | ≈ `NSColor.windowBackgroundColor`… | — | Background layers per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::LAYER_FACTOR_BG_SUNKEN`, `LAYER_FACTOR_SURFACE`, `LAYER_FACTOR_RAISED`, `LAYER_FACTOR_CONTROL`, `LAYER_FACTOR_LINE` | constant | — | — | — | Share of the window tint each layer takes | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::WINDOW_TINT_ACCENT`, `TINT_STRENGTH` | constant | — | — | — | Accent tint amounts in light and dark; strength factors | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::TEXT_SIZES` | constant | — | — | — | The steps of the text size slider | [4](04-appearance-and-tokens.md#the-appearance-file) |
| `ctdesign::GLASS_INK_{LIGHT,DARK,DARK_TINT}`, `GLASS_INK_MUTED_{LIGHT,DARK,DARK_TINT}`, `GLASS_INK_FAINT_{LIGHT,DARK,DARK_TINT}` | constant | — | — | — | The ink of text on glass, per theme: dark in the light theme and white in the dark ones | [4](04-appearance-and-tokens.md#theme-functions) |
| `ctappearance::menu::MenuInk`, `ink_over`, `dark_ink_over` | struct, fn | — | — | — | The three inks of a menu on glass and the one a panel of a given luma takes: dark above `MENU_DARK_INK_ABOVE`, white below | [4](04-appearance-and-tokens.md#theme-functions) |
| `ctappearance::material::Material` | enum | — | `NSVisualEffectView.Material` | `UIBlurEffect.Style` | The 14 materials of the macOS 27.2 SDK | [4](04-appearance-and-tokens.md#window-materials) |
| `ctdesign::WALLPAPER_TINTING`, `WALLPAPER_TINTING_LIFT`, `WALLPAPER_TINTING_TARGET`, `WALLPAPER_TINTING_SIDEBAR`, `WALLPAPER_TINTING_SIDEBAR_LIFT` | constant | — | — | — | Share of the wallpaper's chroma a window takes; share of the way its lightness moves to the target; that target; the sidebar's multiples | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::ACCENT_MIN_CONTRAST_ON_WINDOW`, `ON_ACCENT_MIN_CONTRAST` | constant | — | — | — | Contrast minimums: accent on window 3, ink on accent 4.5 | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::DOMINANT_LIGHTNESS`, `DOMINANT_MIN_CHROMA` | constant | — | — | — | Lightness range and minimum chroma of a wallpaper's dominant color | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::ACCENTS` | constant | — | `AppleAccentColor` | — | The eight named accents with every color each decides, measured per appearance (light, dark, Increase Contrast light and dark). | [4](04-appearance-and-tokens.md#accents) |
| `ctdesign::Accent`, `AccentColors` | struct | — | ≈ `NSColor.controlAccentColor`, `selectedContentBackgroundColor`, `selectedTextBackgroundColor`, `keyboardFocusIndicatorColor` | — | A named accent, and its colors in one appearance (`Accent::colors`) | [4](04-appearance-and-tokens.md#accents) |
| `ctdesign::ACCENT_RULE`, `AccentRule`, `OklchShift` | constant, struct | — | — | — | The rule that derives the other colors from the `control` of an accent macOS has no name for | [4](04-appearance-and-tokens.md#accents) |
| `ctdesign::ACCENT_SWATCHES` | constant | — | — | — | The name and light `control` of each accent in `ACCENTS`, in the order of System Settings | [4](04-appearance-and-tokens.md#accents) |
| `ctdesign::MULTICOLOR` | constant | — | — | — | The 25 stops, 15° apart from the top clockwise, of the Multicolor swatch of System Settings | [4](04-appearance-and-tokens.md#accents) |
| `ctappearance::accent::named`, `named_by_color`, `blue`, `derived` | fn | — | — | — | The named accent of a name, of a light `control` an old config wrote, the default one, and the colors the rule gives an accent without a name | [4](04-appearance-and-tokens.md#accents) |
| `ctdesign::LEVEL_INDICATOR_COLORS` | constant | — | — | — | Normal, warning and critical level colors | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SHELL_LIGHT`, `SHELL_LIGHT_OPAQUE`, `SHELL_DARK`, `SHELL_DARK_OPAQUE`, `SHELL_DARK_TINT`, `SHELL_DARK_TINT_OPAQUE` | constant | — | — | — | Menu bar and Dock colors per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::THEME_SHADOW_OPACITY` | constant | — | — | — | Shadow opacity of the Freya shell theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::MENU_LIGHT`, `MENU_LIGHT_OPAQUE`, `MENU_DARK`, `MENU_DARK_OPAQUE`, `MENU_DARK_TINT`, `MENU_DARK_TINT_OPAQUE` | constant | — | — | — | Menu palettes per theme, translucent and opaque | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_LIGHT`, `SIDEBAR_LIGHT_OPAQUE`, `SIDEBAR_DARK`, `SIDEBAR_DARK_OPAQUE`, `SIDEBAR_DARK_TINT`, `SIDEBAR_DARK_TINT_OPAQUE` | constant | — | — | — | Flat sidebar background per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_GLASS_BASE_{LIGHT,DARK,DARK_TINT}`, `SIDEBAR_GLASS_SOLID_{LIGHT,DARK,DARK_TINT}` | constant | — | — | — | Per theme, the color a sidebar on glass starts from and its solid color without the glass | [10](10-sidebar.md#glass) |
| `ctdesign::TOOLBAR_BG_LIGHT`, `TOOLBAR_BG_DARK`, `TOOLBAR_BG_DARK_TINT`, `TOOLBAR_LINE_LIGHT`, `TOOLBAR_LINE_DARK`, `TOOLBAR_LINE_DARK_TINT` | constant | — | — | — | Background of the unified title bar beside a sidebar and the line under it | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::SIDEBAR_PALETTE_LIGHT`, `SIDEBAR_PALETTE_DARK`, `SIDEBAR_PALETTE_DARK_TINT` | constant | — | — | — | Sidebar inks per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::TOOLGROUP_BG_LIGHT`, `TOOLGROUP_BG_LIGHT_OPAQUE`, `TOOLGROUP_BG_DARK`, `TOOLGROUP_BG_DARK_OPAQUE`, `TOOLGROUP_BG_DARK_TINT`, `TOOLGROUP_BG_DARK_TINT_OPAQUE`, `TOOLGROUP_PALETTE_LIGHT`, `TOOLGROUP_PALETTE_DARK`, `TOOLGROUP_PALETTE_DARK_TINT` | constant | — | — | — | Toolbar group background and lines per theme | [4](04-appearance-and-tokens.md#colors) |
| `ctdesign::WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` | constant | ≈ `.defaultSize(_:)` | ≈ `NSWindow.minSize` | — | Minimum and default window size: 480 × 360, 640 × 420 | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::WINDOW_CORNER`, `WINDOW_RESIZE` | constant | — | — | — | Window corner shape; resize band widths | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::CORNER_WINDOW`, `CORNER_SHEET`, `CORNER_ALERT`, `CORNER_MENU`, `CORNER_TOOLTIP` | constant | — | `NSView.effectiveCornerRadii` | — | The measured radius and curve of a window, sheet, alert, menu and tooltip | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::CORNER_CONTROLS`, `CORNER_SMOOTHING`, `CORNER_CONCENTRIC_REGION` | constant | — | `NSView.effectiveCornerRadii` | — | Control radii by size; the smoothing of the continuous curve; the width of the concentric region | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::DIALOG_PADDING`, `DIALOG_ALERT_WIDTH`, `DIALOG_SOLID_LIGHT`, `DIALOG_SOLID_DARK`, `DIALOG_SOLID_SCRIM_LIGHT`, `DIALOG_ALERT_STACK_SPACING`, `DIALOG_ALERT_HELP_SIZE`, `DIALOG_ALERT_HELP_INSET`, `DIALOG_ALERT_BADGE`, `DIALOG_ALERT_CAUTION`, `DIALOG_ALERT_CAUTION_SIZE`, `DIALOG_ALERT_CAUTION_AT`, `DIALOG_ALERT_PADDING`, `DIALOG_ALERT_SPACING`, `DIALOG_ALERT_ICON`, `DIALOG_ALERT_ICON_GAP`, `DIALOG_ALERT_LINE_HEIGHT`, `DIALOG_BUTTON_WIDTH`, `DIALOG_SHEET_WIDTH`, `DIALOG_SHEET_HEIGHT`, `POPUP`, `PANEL_GLOW_DEPTH` | constant | — | — | — | Pop-up corners, dialog padding, the alert's width, padding, spacing, icon and line height, the dialog button width, the size of a sheet window, panel glow depth | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::SHADOW`, `SHADOW_DARK` | constant | — | ≈ `NSShadow` | ≈ `NSShadow` | Window shadows in light and dark | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::TITLEBAR`, `TITLEBAR_CLIENT_HEIGHT`, `TITLEBAR_PADDING_X`, `TITLEBAR_EXPANDED_TITLE_HEIGHT`, `TITLEBAR_EXPANDED_TITLE_GAP`, `TITLEBAR_EXPANDED_TOOLBAR_BOTTOM`, `TITLEBAR_TOOLBAR_START`, `TITLEBAR_FULLSCREEN_START`, `TITLEBAR_SYSTEM_VEIL`, `TITLEBAR_DOUBLE_CLICK` | constant | — | — | — | Titlebar sizes, lights and double click | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::FULLSCREEN_BAR`, `FULLSCREEN_GLASS`, `HUD_BACKGROUND` | constant | — | — | — | Where the content starts under the focus-mode bar, the `TopGlass` band, HUD window background | [4](04-appearance-and-tokens.md#windows-and-titlebars) |
| `ctdesign::TRAFFIC_LIGHTS_{LIGHT,DARK,DARK_TINT}`, `TRAFFIC_LIGHTS_CONTRAST_*`, `TRAFFIC_GLYPHS`, `TRAFFIC_RING_WIDTH`, `TRAFFIC_MARGIN`, `TRAFFIC_PRESS`, `TRAFFIC_GLYPH_FADE_MS`, `TRAFFIC_CLOSE_MINIMIZE_DARKEN`, `TRAFFIC_ZOOM_LIGHTEN` | constant | — | — | — | Traffic-light looks, glyphs, ring, margin, press and the shifts that derive tinted tones | [4](04-appearance-and-tokens.md#traffic-lights) |
| `ctdesign::BUTTON`, `ICON_BUTTON`, `SWITCH`, `CHECKBOX`, `SLIDER`, `SLIDER_FAT`, `SLIDER_KEY_STEP`, `KNOB_SHADOW`, `SEGMENTED`, `STEPPER`, `TOOLBAR_SEARCH_WIDTH` | constant | — | — | — | Metrics of buttons, toggles, sliders and toolbar controls | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::TEXT_FIELD`, `SEARCH_FIELD`, `TOKEN_FIELD`, `TEXT_EDITOR`, `SPELLING`, `COMBO_BOX`, `PATH_CONTROL`, `DATE_PICKER`, `IMAGE_WELL` | constant | — | — | — | Metrics of fields, the text editor and pickers | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SEARCH_CLEAR_*`, `TOKEN_FILL_*`, `TOKEN_FILL_CONTRAST_*`, `STEPPER_DIVIDER_*`, `STEPPER_DIVIDER_CONTRAST_*`, `DISCLOSURE_TRIANGLE_*`, `DISCLOSURE_TRIANGLE_CONTRAST_*`, `CALENDAR_*`, `CALENDAR_CONTRAST_*`, `DATE_FIELD_*`, `DATE_FIELD_CONTRAST_*`, `DATE_FIELD_BORDER_*`, `DATE_FIELD_BORDER_CONTRAST_*` | constant | — | — | — | Per theme (`_LIGHT`, `_DARK`, `_DARK_TINT`) and with Increase Contrast: the search field's clear button, a token's fill, the stepper's divider, the disclosure triangle, the `CalendarPalette` of the date picker's calendar, and its field's fill and border | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BUTTON_DESTRUCTIVE_FILL` | constant | — | — | — | The alpha of `Role::DestructiveFill`, the destructive color a destructive button fills with: 0.22 | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::BAR_TRACK_*`, `RING_TRACK_*`, `LEVEL_TRACK_*`, `LEVEL_BORDER_*`, `RELEVANCY_*`, `RELEVANCY_TRACK_*`, `RELEVANCY_CONTRAST_*`, `GAUGE_CONTRAST_*`, `LEVEL_INDICATOR_COLORS_DARK`, `LEVEL_INDICATOR_COLORS_CONTRAST`, `LEVEL_INDICATOR_COLORS_CONTRAST_DARK`, `LEVEL_RING_RADII`, `LEVEL_BOTTOM` | constant | — | — | — | Per theme: the tracks of the progress bar, the progress ring and the capacity indicator, its border, the `RelevancyPalette`, the `GaugeContrast` of Increase Contrast and the level colors in dark and with Increase Contrast; the corner radii of the capacity indicator's rings and how much thinner its bottom edge is | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::LEVEL_INDICATOR`, `GAUGE`, `PROGRESS`, `BADGE`, `FOCUS_RING`, `SYMBOL`, `SYMBOL_SCALE_SIZE`, `SYMBOL_WEIGHT_STROKE_SMALL`, `SYMBOL_WEIGHT_STROKE_MEDIUM`, `SYMBOL_WEIGHT_STROKE_LARGE`, `SYMBOL_HIERARCHY`, `ICON_STYLE`, `ICON_PLATE`, `ICON_PLATE_LOBE_DARK`, `ICON_PLATE_LOBE_LIGHT`, `ICON_PLATE_GRAPHITE`, `ICON_PLATE_SILVER`, `ICON_PLATE_CLEAR`, `ICON_PLATE_CLEAR_DARK`, `ICON_PLATE_TINTED`, `ICON_PLATE_TINTED_DARK`, `ICON_LOGO_CLEAR`, `ICON_LOGO_CLEAR_DARK`, `ICON_LOGO_TINTED`, `ICON_LOGO_TINTED_DARK`, `TOOLTIP`, `POPOVER`, `DRAG` | constant | — | — | — | Metrics of indicators, focus ring, symbols, icon styles, tooltips, popovers, drag; `SYMBOL` is the grid and the regular stroke (1, 1 for a tile), `SYMBOL_SCALE_SIZE` the box factor of the three scales, `SYMBOL_WEIGHT_STROKE_*` the stroke of the nine weights per scale as a multiple of the regular one, and `SYMBOL_HIERARCHY` the alpha of the primary, secondary and tertiary layers | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::MENU_POPOVER`, `MENU_MARK_TEXT_GAP`, `MENU_ICON_TEXT_GAP`, `MENU_SUBMENU_INTENT_MS`, `MENU_SCROLL_TICK_MS`, `MENU_SEPARATOR_INK_MIX` | constant | — | — | — | Menu and pop-up menu metrics and timing | [4](04-appearance-and-tokens.md#control-metrics) |
| `ctdesign::SIDEBAR`, `TOOLGROUP`, `SPLIT_VIEW`, `SPLIT_PANES`, `TABLE`, `COLLECTION`, `LIST_GROUP`, `LIST_SECTION`, `GROUP_BOX`, `DISCLOSURE`, `TOOLBAR_TABS`, `TAB_VIEW`, `SCROLL_BAR` | constant | — | — | — | Metrics of sidebars, split views, lists, tables and groups | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::SETTINGS`, `FONT_PANEL`, `MEDIA`, `BACKDROP`, `BACKDROP_SCENES` | constant | — | — | — | Metrics of settings, font panel, media, backdrops | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::LABEL_ICON_WIDTH`, `LABEL_ICON_HEIGHT`, `LABEL_ICON_FRAME`, `LABEL_GAP`, `STACK_COLUMN_GAPS` | constant | ≈ `Label`, `VStack` spacing | — | — | The 16 × 16.5 box of a `Label`'s symbol, its size as a multiple of the font, the 6 gap before the title; the `VStack` gap between each pair of neighbouring kinds of view (text, image, label, checkbox, control) | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::CONTENT_UNAVAILABLE_ICON`, `CONTENT_UNAVAILABLE_ICON_LINE`, `CONTENT_UNAVAILABLE_ICON_GAP`, `CONTENT_UNAVAILABLE_TITLE`, `CONTENT_UNAVAILABLE_DESCRIPTION`, `CONTENT_UNAVAILABLE_DESCRIPTION_GAP`, `CONTENT_UNAVAILABLE_ACTIONS_GAP` | constant | ≈ `ContentUnavailableView` | — | ≈ `UIContentUnavailableConfiguration` | The symbol size, its line and gap, the title and description fonts and the gaps of `ContentUnavailableView` | [4](04-appearance-and-tokens.md#view-metrics) |
| `ctdesign::MATERIAL` | constant | `Material` | `NSVisualEffectView.Material` | `UIBlurEffect.Style` | Opacity and blur of the five material thicknesses | [4](04-appearance-and-tokens.md#materials) |
| `ctdesign::MENUBAR_HEIGHT` | constant | — | — | — | The slim menu bar height, which the focus-mode bar takes | [4](04-appearance-and-tokens.md#shell) |
| `ctdesign::DOCK_LABEL_RADIUS`, `DOCK_LABEL_ARROW_*`, `DOCK_MENU_ARROW_TIP_RADIUS`, `DOCK_MENU_ARROW_FILLET` | constant | — | — | — | The label and menu shapes of the Dock the kit draws | [4](04-appearance-and-tokens.md#shell) |

## Buttons

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `controls::Button` | struct | `Button` | `NSButton` | `UIButton` | Push button with a text label | [5](05-controls.md#buttons) |
| `controls::Button::system_image` | fn | `Button(_:systemImage:)` | `NSButton(title:image:target:action:)` | `UIButton` | Push button with a symbol before its label | [5](05-controls.md#buttons) |
| `controls::Button::label` | method | `Button(action:label:)` | — | `UIButton` | A closure that returns the view drawn in place of the text; the string of `new` stays the accessible name | [5](05-controls.md#buttons) |
| `controls::ControlSize` | enum | `ControlSize` | `NSControl.ControlSize` | — | `Mini`, `Small`, `Regular`, `Large`, `ExtraLarge`: the size of a control (`ctdesign::ControlSize`) | [5](05-controls.md#control-size) |
| `controls::TextField::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Height and font of one text field (`Mini` 19, `Small` 22; larger sizes stay 24) | [5](05-controls.md#control-size) |
| `controls::TextField::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Draws the text in the tertiary label and takes no input | [5](05-controls.md#text-fields) |
| `controls::SearchField::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Height, font, magnifier and clear button of one search field | [5](05-controls.md#control-size) |
| `controls::SearchField::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Draws the text and magnifier in the tertiary label and takes no input | [5](05-controls.md#text-fields) |
| `controls::SecureField::disabled` | method | `.disabled(_:)` | `NSControl.isEnabled` | `UIControl.isEnabled` | Draws the dots in the tertiary label and takes no input | [5](05-controls.md#text-fields) |
| `combo_box::ComboBox::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Height, font, arrow button and chevron of one combo box | [5](05-controls.md#control-size) |
| `styles::TextFieldStyle::plain` | method | `TextFieldStyle.plain` | `NSTextField.isBezeled` `false` | `UITextField.BorderStyle` | Preset: no bezel or fill, 16 high | [Styles](../../crates/bk/src/styles/text_field.rs) |
| `controls::Button::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Height, padding, font and corners of one button | [5](05-controls.md#control-size) |
| `controls::Button::keyboard_shortcut` | method | `.keyboardShortcut(_:modifiers:)` | `NSButton.keyEquivalent` | `UIKeyCommand` | Calls `on_press` on a `Shortcut` while the button is enabled; a menu item with the same key wins | [5](05-controls.md#buttons) |
| `controls::ButtonRole` | enum | `ButtonRole` | ≈ `NSButton.hasDestructiveAction` | ≈ `UIAction.Attributes.destructive` | `Destructive` only; SwiftUI's `cancel`, `confirm` and `close` are not built | [5](05-controls.md#buttons) |
| `controls::Button::role` | method | `Button(_:role:action:)` | `NSButton.hasDestructiveAction` | `UIAction.Attributes` | `Destructive` draws `ButtonStyle::destructive` | [5](05-controls.md#buttons) |
| `popup_button::PopUpButton::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one pop-up or pull-down button | [5](05-controls.md#control-size) |
| `controls::SegmentedControl::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one segmented control | [5](05-controls.md#control-size) |
| `controls::Checkbox::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one checkbox | [5](05-controls.md#control-size) |
| `choices::RadioGroup::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one radio group | [5](05-controls.md#control-size) |
| `controls::Switch::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one switch, which takes the size of its track | [5](05-controls.md#control-size) |
| `controls::Slider::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one slider | [5](05-controls.md#control-size) |
| `compact::Stepper::size` | method | ≈ `.controlSize(_:)` | `NSControl.controlSize` | — | Size of one stepper | [5](05-controls.md#control-size) |
| `compact::IconButton` | struct | `≈ Button` | `≈ NSButton` | `UIButton` | Button showing only a symbol; 36 capsule inside a tool group | [5](05-controls.md#compact-buttons) |
| `compact::Stepper` | struct | `Stepper` | `NSStepper` | `UIStepper` | Integer value with up and down arrows | [5](05-controls.md#compact-buttons) |
| `compact::ToggleGroup` | struct | — | `≈ NSSegmentedControl` | `UISegmentedControl` | Row of independent formatting toggles (select-any tracking) | [5](05-controls.md#compact-buttons) |
| `compact::Face` | enum | — | — | — | `Bold`, `Italic`, `Underline`: how a toggle's text is drawn | [5](05-controls.md#compact-buttons) |
| `buttons::MenuButton` | struct | `Menu` | `≈ NSPopUpButton` | ≈ `UIButton.showsMenuAsPrimaryAction` | Button with a chevron that opens an action menu (pull-down) | [5](05-controls.md#menu-buttons) |
| `buttons::MenuButton::system_image` | function | `Menu(_:systemImage:)` | — | `UIMenu` | A menu button that shows only a symbol | [5](05-controls.md#menu-buttons) |
| `buttons::MenuButton::label_style` | method | `.labelStyle(_:)` | — | — | Shows the symbol alone or before the label | [5](05-controls.md#menu-buttons) |
| `buttons::LabelStyle` | enum | `LabelStyle` (`.iconOnly`, `.titleAndIcon`) | — | — | How a menu button with a symbol shows it | [5](05-controls.md#menu-buttons) |
| `buttons::ComboButton` | struct | `≈ Menu` | `NSComboButton` | ≈ `UIButton` | Main action plus a chevron that opens a menu | [5](05-controls.md#menu-buttons) |
| `buttons::HelpLink` | struct | `HelpLink` | `≈ NSButton.BezelStyle.helpButton` | — | Round "?" help button | [5](05-controls.md#menu-buttons) |
| `buttons::ControlGroup` | struct | `ControlGroup` | — | `UIBarButtonItemGroup` | Controls joined on one background with separators | [5](05-controls.md#menu-buttons) |
| `buttons::ShareLink` | struct | `ShareLink` | `≈ NSSharingServicePicker` | `UIActivityViewController` | Menu to copy, email or open shared content; no share portal | [5](05-controls.md#menu-buttons) |
| `buttons::mail_uri` | fn | — | — | — | Builds a percent-encoded `mailto:?body=` URI | [5](05-controls.md#menu-buttons) |
| `controls::ToolGroup` | struct | `ToolbarItemGroup` | `NSToolbarItemGroup` | `UIBarButtonItemGroup` | Toolbar items in 36 slots; capsule with separators | [5](05-controls.md#toolbar-controls) |
| `compact::ToolbarSearch` | struct | `≈ .searchable(text:placement:prompt:)` | `NSSearchToolbarItem` | ≈ `UISearchController` | Toolbar search field, optionally collapsed to an icon | [5](05-controls.md#toolbar-controls) |
| `compact::ToolbarOverflow` | struct | — | — | — | Chevron button that lists extra toolbar items in a menu | [5](05-controls.md#toolbar-controls) |

## Toggles and choices

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `controls::Switch` | struct | `ToggleStyle.switch` | `NSSwitch` | `UISwitch` | On/off switch | [5](05-controls.md#switch-and-checkbox) |
| `controls::Checkbox` | struct | `ToggleStyle.checkbox` | `≈ NSButton` | — | Checkbox with its label (NSButton checkbox type) | [5](05-controls.md#switch-and-checkbox) |
| `choices::RadioGroup` | struct | `PickerStyle.radioGroup` | `≈ NSButton` | — | Column of radio options; arrows move the choice | [5](05-controls.md#choices) |
| `controls::SegmentedControl` | struct | `PickerStyle.segmented` | `NSSegmentedControl` | `UISegmentedControl` | Segments filling the width; one is chosen | [5](05-controls.md#choices) |
| `popup_button::PopUpButton` | struct | `PickerStyle.menu` | `NSPopUpButton` | ≈ `UIButton.changesSelectionAsPrimaryAction` | Shows the chosen item; opens the list as a menu | [5](05-controls.md#choices) |
| `popup_button::PopUpButton::section_header` | method | `Section(_:)` in a `Picker` | `NSMenuItem.sectionHeader(title:)` | `UIMenu.Options.displayInline` | Titles the items that follow, after a separator | [5](05-controls.md#choices) |
| `popup_button::PopUpButton::separator` | method | `Divider()` in a `Picker` | `NSMenuItem.separator()` | `UIMenu.Options.displayInline` | A line before the items that follow | [5](05-controls.md#choices) |
| `choices::ColorWell` | struct | `≈ PickerStyle.palette` | — | `UIColorWell` | Row of preset color swatches; edits the chosen index, the `State<Option<usize>>` of `new` | [5](05-controls.md#choices) |
| `choices::ColorWell::accents` | fn | — | ≈ the accent picker of System Settings | — | A `ColorWell` of `ACCENT_SWATCHES` that draws each swatch in the `control` of the current appearance | [5](05-controls.md#choices) |
| `choices::ColorWell::multicolor` | method | — | ≈ the «Multicolor» accent of System Settings | — | A rainbow swatch first, at index 0 | [5](05-controls.md#choices) |
| `choices::ChoiceCards` | struct | — | — | — | Large preview cards with captions; one is chosen | [5](05-controls.md#choices) |
| `choices::swatch_name` | fn | — | — | — | The localized name of a `ctdesign::ACCENT_SWATCHES` id, as `ColorWell` shows it | [5](05-controls.md#choices) |

## Sliders and indicators

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `controls::CircularSlider` | struct | — | `NSSlider` with `sliderType = .circular` | — | A 24 gray disc with a dash that turns once around, dragged or moved with the arrows; edits the `State<f32>` of `new`, `.disabled` | [5](05-controls.md#sliders) |
| `controls::Slider` | struct | `Slider` | `NSSlider` | `UISlider` | Free slider: a continuous value the knob follows to any point of the track, with optional marks and captions | [5](05-controls.md#sliders) |
| `controls::Slider::stepped` | method | `Slider(value:in:step:)` | `NSSlider.allowsTickMarkValuesOnly` | — | The slider that stops only at a step, with a tick and a caption per step | [5](05-controls.md#sliders) |
| `controls::Slider::range` | method | `Slider(value:in:)` | `NSSlider.minValue`, `maxValue` | `UISlider.minimumValue`, `UISlider.maximumValue` | The values the free knob covers; 0 to 1 by default | [5](05-controls.md#sliders) |
| `controls::Slider::number_of_tick_marks` | method | — | `NSSlider.numberOfTickMarks` | — | Ticks spread evenly; they never snap the knob | [5](05-controls.md#sliders) |
| `controls::Slider::continuous` | method | ≈ `Slider(onEditingChanged:)` | `NSSlider.isContinuous` | `UISlider.isContinuous` | With `false`, the bound `State` changes once, when the knob is released | [5](05-controls.md#sliders) |
| `level::LevelSlider` | struct | `Slider` | `NSSlider` | `UISlider` | Continuous 0–1 slider, regular or fat with a symbol | [5](05-controls.md#sliders) |
| `progress::ProgressView` | struct | `ProgressView` | `NSProgressIndicator` | `UIProgressView` | Determinate or indeterminate progress bar | [5](05-controls.md#progress) |
| `progress::ProgressRing` | struct | `ProgressViewStyle.circular` | `≈ NSProgressIndicator` | `UIActivityIndicatorView` | Determinate circular progress ring | [5](05-controls.md#progress) |
| `progress::Spinner` | struct | `≈ ProgressView` | `≈ NSProgressIndicator` | `UIActivityIndicatorView` | Indeterminate spinning indicator, spinning style | [5](05-controls.md#progress) |
| `indicator::CapacityIndicator` | struct | `≈ GaugeStyle.linearCapacity` | `≈ NSLevelIndicator` | — | Capacity bar, green-yellow-red by thresholds, continuous or discrete | [5](05-controls.md#level-indicators) |
| `indicator::RelevancyIndicator` | struct | — | `≈ NSLevelIndicator` | — | Relevancy style: thin lines proportional to the value | [5](05-controls.md#level-indicators) |
| `indicator::Gauge` | struct | `GaugeStyle.accessoryCircular` | — | — | Open ring with a marker; capacity mode is a filled ring | [5](05-controls.md#level-indicators) |
| `indicator::RatingIndicator` | struct | — | `≈ NSLevelIndicator` | — | Five-star rating of the `State<u32>` of `new`; editable with `.editable(true)` (`NSLevelIndicator.isEditable`) | [5](05-controls.md#level-indicators) |
| `indicator::level_color` | fn | — | — | — | Index 0–2 of the capacity color for a value | [5](05-controls.md#level-indicators) |
| `indicator::gauge_marker` | fn | — | — | — | Center of the gauge marker for a `GaugeStyle` and a value | [5](05-controls.md#level-indicators) |

## Text fields and pickers

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `controls::TextField` | struct | `TextField` | `NSTextField` | `UITextField` | One-line field bound to a `State<String>`, with spell checking | [5](05-controls.md#text-fields) |
| `controls::TextField::autocorrection_disabled` | method | `.autocorrectionDisabled(_:)` | `NSTextView.isContinuousSpellCheckingEnabled` | `UITextInputTraits.spellCheckingType` | Turns spell checking off for names and codes | [5](05-controls.md#text-fields) |
| `controls::TextField::on_submit` | method | `.onSubmit(_:)` | `NSTextField.action` | `UITextFieldDelegate.textFieldShouldReturn(_:)` | Runs when the user presses Enter, with the text | [5](05-controls.md#text-fields) |
| `controls::SearchField` | struct | `≈ .searchable(text:tokens:placement:prompt:token:)` | `NSSearchField` | `UISearchTextField` | Search field with clear button and optional tokens | [5](05-controls.md#text-fields) |
| `controls::SecureField` | struct | `SecureField` | `NSSecureTextField` | `UITextField.isSecureTextEntry` | Masked password field; Enter submits the secret | [5](05-controls.md#text-fields) |
| `controls::SecureField::revealed` | method | — | — | — | Shows the typed text instead of dots | [5](05-controls.md#text-fields) |
| `combo_box::ComboBox` | struct | — | `NSComboBox` | — | Text field with an arrow that opens suggested values | [5](05-controls.md#text-fields) |
| `password::PasswordField` | struct | — | — | — | Zeroizing password buffer with idle, verifying and error states | [5](05-controls.md#passwords) |
| `password::PasswordState` | enum | — | — | — | `Idle`, `Verifying`, `Error(String)` of a password field | [5](05-controls.md#passwords) |
| `color_picker::ColorPicker` | struct | `ColorPicker` | `≈ NSColorWell` | `UIColorWell` | Color well opening an HSB popover, not a color panel | [5](05-controls.md#color-picker) |
| `color_picker::ColorPanel::show` | fn | — | `NSColorPanel` | `UIColorPickerViewController` | Opens the HSB color popover beside an area, without a well | [5](05-controls.md#color-picker) |
| `color_picker::hsb_to_rgb` | fn | — | `≈ NSColor.init(hue:saturation:brightness:alpha:)` | — | Opaque color from hue, saturation and brightness | [5](05-controls.md#color-picker) |
| `color_picker::rgb_to_hsb` | fn | — | `≈ NSColor.getHue(_:saturation:brightness:alpha:)` | — | Hue, saturation and brightness of a color | [5](05-controls.md#color-picker) |
| `color_picker::hex` | fn | — | — | — | `#RRGGBB` string of a color | [5](05-controls.md#color-picker) |
| `date_picker::DatePicker` | struct | `DatePicker` | `NSDatePicker` | `UIDatePicker` | Segmented date/time field with stepper and pop-up calendar | [5](05-controls.md#date-picker) |
| `date_picker::DateElements` | enum | `DatePickerComponents` | `NSDatePicker.ElementFlags` | — | `Date`, `Time` or `DateAndTime` parts shown | [5](05-controls.md#date-picker) |
| `date::CivilDate` | struct | — | — | — | Calendar date with day and month arithmetic | [5](05-controls.md#date-picker) |
| `date::CivilDateTime` | struct | — | — | — | Calendar date plus hour and minute | [5](05-controls.md#date-picker) |
| `date::DatePart` | enum | — | — | — | `Day`, `Month`, `Year`, `Hour`, `Minute` | [5](05-controls.md#date-picker) |
| `date::days_in_month` | fn | — | — | — | Days in a month, with leap years | [5](05-controls.md#date-picker) |
| `date::typing_is_done` | fn | — | — | — | Whether typed digits complete a date part | [5](05-controls.md#date-picker) |
| `path_control::PathControl` | struct | — | `NSPathControl` | — | Path parts with icons separated by chevrons | [5](05-controls.md#path-control) |
| `image_well::ImageWell` | struct | — | `≈ NSImageView` | `UIImageView` | Editable image well: drop, choose or clear an image | [5](05-controls.md#image-well) |
| `image_well::is_image` | fn | — | — | — | Whether a path has an image extension | [5](05-controls.md#image-well) |

## Adornments, scrolling and drawing

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `adornments::Badge` | struct | `≈ .badge(_:)` | — | `UITabBarItem.badgeValue` | Red capsule counter | [5](05-controls.md#adornments) |
| `adornments::Divider` | struct | `Divider` | `≈ NSBox` | — | Hairline separator filling the width | [5](05-controls.md#adornments) |
| `adornments::Avatar` | struct | — | — | — | Circle with initials | [5](05-controls.md#adornments) |
| `adornments::Avatar::named` | function | — | — | — | An avatar with the initials of a name | [5](05-controls.md#adornments) |
| `adornments::Avatar::picture` | method | ≈ `.clipShape(Circle())` over an `Image` | ≈ `NSImageView` with a circular mask | ≈ `UIImageView` | Shows an image file in the circle instead of the initials | [5](05-controls.md#adornments) |
| `adornments::initials` | fn | ≈ `PersonNameComponentsFormatter` (`.abbreviated`) | — | ≈ `PersonNameComponentsFormatter` | First letter of each of the first two words of a name, in capitals | [5](05-controls.md#adornments) |
| `adornments::Link` | struct | `Link` | — | `UIButton` | Pressable text in the link color | [5](05-controls.md#adornments) |
| `controls::ScrollView` | struct | `ScrollView` | `NSScrollView` | `UIScrollView` | Scrolling viewport with overlay bar and edge effect | [5](05-controls.md#scrolling) |
| `controls::ScrollController` | struct | `≈ ScrollViewProxy` | `≈ NSClipView` | `UIScrollView.scrollRectToVisible(_:animated:)` | Scrolls a `ScrollView` from code | [5](05-controls.md#scrolling) |
| `controls::use_scroll_controller` | fn | `≈ ScrollViewReader` | — | `UIScrollView.scrollRectToVisible(_:animated:)` | Creates a `ScrollController` | [5](05-controls.md#scrolling) |
| `controls::ScrollController::scroll_to` | method | `ScrollViewProxy.scrollTo(_:anchor:)` | `≈ NSClipView.scroll(to:)` | `UIScrollView.setContentOffset(_:animated:)` | Scrolls to a `ScrollPosition`, jumping or with an `Animation` | [5](05-controls.md#scroll-position) |
| `controls::ScrollController::content_offset` | method | `ScrollGeometry.contentOffset` | `≈ NSClipView.bounds.origin` | `UIScrollView.contentOffset` | The offset of the content, positive when scrolled | [5](05-controls.md#scroll-position) |
| `controls::ScrollController::geometry` | method | `ScrollGeometry` | — | — | The offset and the sizes of content and viewport now | [5](05-controls.md#scroll-position) |
| `controls::ScrollController::phase` | method | `ScrollPhase` | — | `UIScrollView.isTracking`, `isDragging`, `isDecelerating` | The phase of the scroll now | [5](05-controls.md#scroll-position) |
| `controls::ScrollController::scroll_to_x` | method | `ScrollViewProxy.scrollTo(_:anchor:)` | `≈ NSClipView.scroll(to:)` | `UIScrollView.setContentOffset(_:animated:)` | Jumps the content to a horizontal offset; false if the view cannot scroll | [5](05-controls.md#scrolling) |
| `controls::ScrollController::reveal_y` | method | `ScrollViewProxy.scrollTo(_:anchor:)` | `NSView.scrollToVisible(_:)` | `UIScrollView.scrollRectToVisible(_:animated:)` | Scrolls the least that shows the span from `top` to `bottom` of the content | [5](05-controls.md#scrolling) |
| `controls::ScrollController::scroll_by_wheel` | method | — | `NSScrollView.scrollWheel(with:)` | — | Scrolls by a wheel event caught outside the view | [5](05-controls.md#scrolling) |
| `controls::ScrollPosition` | struct | `ScrollPosition` | — | — | Where a `ScrollView` is or should go: a point, an edge or a view by id | [5](05-controls.md#scroll-position) |
| `controls::ScrollPosition::at` | fn | `ScrollPosition(x:y:)` | — | — | A point of the content | [5](05-controls.md#scroll-position) |
| `controls::ScrollPosition::edge` | fn | `ScrollPosition(edge:)` | — | — | An edge of the content | [5](05-controls.md#scroll-position) |
| `controls::ScrollPosition::id` | fn | `ScrollPosition(id:anchor:)` | — | — | The view with an `id`, aligned by `Start`, `Center` or `End`, or the least scroll that shows it | [5](05-controls.md#scroll-position) |
| `controls::ScrollPosition::point` | method | `ScrollPosition.point` | — | — | The point of a position set by a point or by the user | [5](05-controls.md#scroll-position) |
| `controls::ScrollPosition::is_positioned_by_user` | method | `ScrollPosition.isPositionedByUser` | — | — | Whether the user scrolled to this position | [5](05-controls.md#scroll-position) |
| `controls::ScrollPhase` | enum | `ScrollPhase` | — | — | `Idle`, `Tracking`, `Interacting`, `Decelerating`, `Animating` | [5](05-controls.md#scroll-position) |
| `controls::ScrollGeometry` | struct | `ScrollGeometry` | — | — | `content_offset`, `content_size` and `container_size` | [5](05-controls.md#scroll-position) |
| `controls::ScrollView::scroll_position` | method | `.scrollPosition(_:anchor:)` | — | — | Binds a `State<ScrollPosition>`: the view scrolls to what the app sets and writes the user's scroll back | [5](05-controls.md#scroll-position) |
| `controls::ScrollView::on_scroll_geometry_change` | method | `.onScrollGeometryChange(for:of:action:)` | — | `UIScrollViewDelegate.scrollViewDidScroll(_:)` | Calls the action with the `ScrollGeometry` when it changes | [5](05-controls.md#scroll-position) |
| `controls::ScrollView::on_scroll_phase_change` | method | `.onScrollPhaseChange(_:)` | — | `UIScrollViewDelegate.scrollViewWillBeginDragging(_:)` | Calls the action with the old and the new `ScrollPhase` | [5](05-controls.md#scroll-position) |
| `controls::ScrollView::direction` | method | `ScrollView(_:)` axes | `NSScrollView.hasHorizontalScroller` | `UIScrollView.alwaysBounceHorizontal` | Scrolls along `Direction::Vertical` (the default) or `Horizontal` | [5](05-controls.md#scrolling) |
| `controls::ScrollView::scroll_bar` | method | `.scrollIndicators(_:)` | `NSScrollView.hasVerticalScroller` | `UIScrollView.showsVerticalScrollIndicator` | Shows or hides the scroll bar | [5](05-controls.md#scrolling) |
| `controls::ScrollView::edge_effect` | method | `.scrollEdgeEffectStyle(_:for:)` | `≈ NSScrollPocket` | `≈ UIScrollEdgeEffect` | Fades the content under an edge it is scrolled past | [5](05-controls.md#scrolling) |
| `controls::ScrollView::top_line` | method | — | `NSTitlebarSeparatorStyle.line` | — | Draws a separator along the top edge | [5](05-controls.md#scrolling) |
| `controls::ScrollView::controller` | method | `ScrollViewReader` | — | — | Scrolls with a `ScrollController` made by `use_scroll_controller` | [5](05-controls.md#scrolling) |
| `controls::IdentityModifiers::id` | method | `.id(_:)` | — | — | Names a view so that `ScrollPosition::id` finds it inside a `ScrollView` | [5](05-controls.md#scroll-position) |
| `drag::DragSource` | struct | `≈ .draggable(_:)` | `NSDraggingSource` | ≈ `UIDragInteractionDelegate` | Starts a drag to another app past a threshold | [5](05-controls.md#drag-source) |
| `drag::use_drag_source` | fn | `≈ .draggable(_:)` | — | `UIDragInteraction` | Creates a `DragSource` | [5](05-controls.md#drag-source) |
| `drag::DragPayload` | enum | — | `≈ NSDraggingItem`, `NSPasteboardItem` | `UIDragItem` | `Files`, `Text`, `Html { text, html }` or `Image(png bytes)`: what a drag or the pasteboard holds | [5](05-controls.md#drag-source) |
| `pasteboard::Transferable` | trait | `Transferable` | `NSPasteboardWriting`, `NSPasteboardReading` | `NSItemProviderWriting` | A value with a `DragPayload` and a way back from one; `String`, `Vec<PathBuf>` and `DragPayload` have it | [13](13-app.md#clipboard) |
| `pasteboard::Pasteboard` | struct | `≈ .copyable(_:)`, `.pasteDestination(for:)` | `NSPasteboard` | `UIPasteboard` | The system clipboard (`general`) or a private board (`named`) | [13](13-app.md#clipboard) |
| `pasteboard::Pasteboard::copy` | method | `≈ .copyable(_:)` | `NSPasteboard.writeObjects(_:)` | `UIPasteboard.setItems(_:)` | Replaces the board with a `Transferable`; false when the platform cannot hold its type | [13](13-app.md#clipboard) |
| `pasteboard::Pasteboard::paste` | method | `≈ .pasteDestination(for:)` | `NSPasteboard.readObjects(forClasses:)` | `UIPasteboard.items` | The richest item on the board as a `Transferable` | [13](13-app.md#clipboard) |
| `drag::URI_LIST` | constant | — | `≈ NSPasteboard.PasteboardType.fileURL` | — | MIME type `text/uri-list` | [5](05-controls.md#drag-source) |
| `canvas::Canvas` | struct | `Canvas` | `≈ NSView` | `UIView` | Element that paints with Skia, repainting on a new revision | [5](05-controls.md#canvas) |
| `canvas::CanvasContext` | struct | `GraphicsContext` | `NSGraphicsContext` | ≈ `UIGraphicsImageRenderer` | Skia canvas, logical size, scale and fonts for a painter | [5](05-controls.md#canvas) |
| `shapes::Rectangle` | struct | `Rectangle` | `≈ NSBezierPath(rect:)` | `≈ UIBezierPath(rect:)` | A rectangle that fills its frame | [5](05-controls.md#shapes) |
| `shapes::RoundedRectangle` | struct | `RoundedRectangle` | `≈ NSBezierPath(roundedRect:xRadius:yRadius:)` | `≈ UIBezierPath(roundedRect:cornerRadius:)` | A rectangle with a continuous corner of the radius given to `new` | [5](05-controls.md#shapes) |
| `shapes::Capsule` | struct | `Capsule` | — | `≈ UIBezierPath(roundedRect:cornerRadius:)` | A rectangle with half circles at the ends | [5](05-controls.md#shapes) |
| `shapes::Circle` | struct | `Circle` | `≈ NSBezierPath(ovalIn:)` | `≈ UIBezierPath(ovalIn:)` | A circle as wide as the frame is tall, centered | [5](05-controls.md#shapes) |
| `shapes::Ellipse` | struct | `Ellipse` | `≈ NSBezierPath(ovalIn:)` | `≈ UIBezierPath(ovalIn:)` | An ellipse in the frame | [5](05-controls.md#shapes) |
| `shapes::Path` | struct | `Path` | `≈ NSBezierPath` | `≈ UIBezierPath` | A path of lines and curves, in the coordinates of the frame | [5](05-controls.md#shapes) |
| `shapes::Path::move_to` | method | `Path.move(to:)` | `NSBezierPath.move(to:)` | `UIBezierPath.move(to:)` | Starts a subpath | [5](05-controls.md#shapes) |
| `shapes::Path::add_line` | method | `Path.addLine(to:)` | `NSBezierPath.line(to:)` | `UIBezierPath.addLine(to:)` | A line to a point | [5](05-controls.md#shapes) |
| `shapes::Path::add_curve` | method | `Path.addCurve(to:control1:control2:)` | `NSBezierPath.curve(to:controlPoint1:controlPoint2:)` | `UIBezierPath.addCurve(to:controlPoint1:controlPoint2:)` | A cubic curve to a point | [5](05-controls.md#shapes) |
| `shapes::Path::close_subpath` | method | `Path.closeSubpath()` | `NSBezierPath.close()` | `UIBezierPath.close()` | Closes the subpath with a line | [5](05-controls.md#shapes) |
| `shapes::Rectangle::fill`, `RoundedRectangle::fill`, `Capsule::fill`, `Circle::fill`, `Ellipse::fill`, `Path::fill` | method | `Shape.fill(_:)` | `≈ NSBezierPath.fill()` | `≈ UIBezierPath.fill()` | Fills the shape with a `ShapeStyle`: a color, a role, the accent or a gradient | [5](05-controls.md#shapes) |
| `shapes::Rectangle::stroke_border`, `RoundedRectangle::stroke_border`, `Capsule::stroke_border`, `Circle::stroke_border`, `Ellipse::stroke_border` | method | `InsettableShape.strokeBorder(_:lineWidth:)` | `≈ NSBezierPath.stroke()` | `≈ UIBezierPath.stroke()` | Draws the outline inside the frame, so no part of the line is cut | [5](05-controls.md#shapes) |
| `shapes::Path::stroke` | method | `Shape.stroke(_:lineWidth:)` | `≈ NSBezierPath.stroke()` | `≈ UIBezierPath.stroke()` | Draws the line centered on the path; what leaves the frame is cut | [5](05-controls.md#shapes) |
| `shapes::ShapeModifiers::clip_shape` | method | `.clipShape(_:)` | `≈ CALayer.mask` | `≈ CALayer.mask` | Clips a view and its children to a `Shape`: a rectangle, a rounded rectangle, a capsule or a circle | [5](05-controls.md#shapes) |
| `interaction::Interaction` | struct | — | `≈ NSTrackingArea` | ≈ `UIHoverGestureRecognizer` | Hover, press and focus state of a custom control | [5](05-controls.md#building-a-new-control) |
| `interaction::use_interaction` | fn | `≈ .onHover(perform:)` | — | `UIHoverGestureRecognizer` | Creates an `Interaction` | [5](05-controls.md#building-a-new-control) |
| `modifiers::ViewModifiers::focus_ring` | method | — | `≈ NSView.drawFocusRingMask()` | — | Ring of a given color around an element with children | [7](07-menus-and-popovers.md#focus_ring) |

## Text editing

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `text_editor::TextEditor` | struct | `TextEditor` | `NSTextView` | `UITextView` | Multi-line editor with find bar, rich text and spelling | [6](06-text-and-editing.md#text-editor) |
| `text_editor::find_matches` | fn | — | `≈ NSTextFinder` | `UIFindInteraction` | Case-insensitive, non-overlapping matches in character indices | [6](06-text-and-editing.md#find-bar) |
| `rich_text::RunStyle` | struct | — | — | — | Bold, italic, underline, strikethrough, link, family, size and foreground style (`AttributedString.foregroundColor`) of a run | [6](06-text-and-editing.md#rich-text) |
| `rich_text::StyleRun` | struct | — | — | — | A `RunStyle` over a character range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::FontChoice` | struct | `≈ Font` | `≈ NSFontDescriptor` | `UIFontDescriptor` | Family, size, bold and italic | [6](06-text-and-editing.md#rich-text) |
| `rich_text::Emphasis` | enum | — | `≈ NSFontDescriptor.SymbolicTraits` | — | `Bold`, `Italic` or `Underline` | [6](06-text-and-editing.md#rich-text) |
| `rich_text::char_styles` | fn | — | — | — | One style per character from runs | [6](06-text-and-editing.md#rich-text) |
| `rich_text::runs_from` | fn | — | — | — | Runs from per-character styles, merging neighbors | [6](06-text-and-editing.md#rich-text) |
| `rich_text::restyle_after_edit` | fn | — | — | — | Moves runs after an edit; typed text inherits, links do not | [6](06-text-and-editing.md#rich-text) |
| `rich_text::toggle` | fn | — | — | — | Toggles bold, italic or underline on a range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::font_at` | fn | — | — | — | Font at a character, `BODY` when unset | [6](06-text-and-editing.md#rich-text) |
| `rich_text::apply_font` | fn | — | `≈ NSFontManager.convert(_:)` | — | Applies a font choice to a range | [6](06-text-and-editing.md#rich-text) |
| `rich_text::link_at` | fn | — | — | — | Link at a character, if any | [6](06-text-and-editing.md#rich-text) |
| `rich_text::markdown` | fn | `AttributedString(markdown:)` | `NSAttributedString(markdown:)` | `NSAttributedString(markdown:)` | The plain text and the runs of inline Markdown; unpaired marks stay literal | [6](06-text-and-editing.md#text-runs) |
| `font_panel::FontPicker` | struct | — | `≈ NSFontPanel` | `UIFontPickerViewController` | Button with family and size that opens the font panel | [6](06-text-and-editing.md#font-panel) |
| `font_panel::faces` | fn | — | — | — | Typefaces a family offers; Regular always | [6](06-text-and-editing.md#font-panel) |
| `font_panel::parse_size` | fn | — | — | — | Typed font size if finite and positive | [6](06-text-and-editing.md#font-panel) |
| `font_panel::remember` | fn | — | — | — | Moves a family to the front of the recent list | [6](06-text-and-editing.md#font-panel) |
| `spelling::words` | fn | — | — | — | Character ranges of words; apostrophes join letters | [6](06-text-and-editing.md#spelling) |
| `spelling::misspelled` | fn | — | `≈ NSSpellChecker.checkSpelling(of:startingAt:)` | — | Ranges of words the checker does not know | [6](06-text-and-editing.md#spelling) |
| `spelling::word_at` | fn | — | — | — | Word range containing or touching a character | [6](06-text-and-editing.md#spelling) |
| `spelling::replace_chars` | fn | — | — | — | Text with a character range replaced | [6](06-text-and-editing.md#spelling) |
| `spelling::dictionary_name` | fn | — | `≈ NSSpellChecker.setLanguage(_:)` | — | Hunspell dictionary name for a language tag | [6](06-text-and-editing.md#spelling) |
| `spelling::personal_words` | fn | — | — | — | Words of an Enchant personal list | [6](06-text-and-editing.md#spelling) |
| `token_field::TokenField` | struct | — | `NSTokenField` | — | Tags as capsules; Enter or comma commits | [6](06-text-and-editing.md#token-field) |
| `token_field::TokenKey` | enum | — | — | — | `Backspace`, `Delete`, `Left`, `Right` on an empty input | [6](06-text-and-editing.md#token-field) |
| `token_field::commit_token` | fn | — | — | — | Pushes trimmed draft as a token; false if empty | [6](06-text-and-editing.md#token-field) |
| `token_field::token_selection` | fn | — | — | — | Next selected token and removed index for a key | [6](06-text-and-editing.md#token-field) |

## Rule editor

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `rule_editor::RuleEditor` | struct | — | `≈ NSPredicateEditor` | — | Edits a predicate as field, operator and value rows | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::RuleField` | struct | — | `≈ NSPredicateEditorRowTemplate` | — | A filterable field: label and kind | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::RuleKind` | enum | — | — | — | `Text`, `Number` or `Choice(Vec<Cow<'static, str>>)` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Operator` | enum | — | — | — | `Contains`, `BeginsWith`, `EndsWith`, `Is`, `IsNot`, `Greater`, `Less` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::operators` | fn | — | — | — | Operators offered for a field kind | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Rule` | struct | — | — | — | One condition: field, operator and value | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Criterion` | enum | — | — | — | `Rule(Rule)` or a nested `Group` | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate` | struct | — | — | — | All or any of its criteria; evaluates an item | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Rule::matches` | method | — | `NSPredicate.evaluate(with:)` | — | Whether a field value meets the rule | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate::evaluate` | method | — | `NSPredicate.evaluate(with:)` | — | Whether an item, read field by field, meets the predicate | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate::rule` | method | — | — | — | The rule with this id, at any depth | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate::remove` | method | — | `NSRuleEditor.removeRow(at:)` | — | Removes the criterion with this id | [6](06-text-and-editing.md#rule-editor) |
| `rule_editor::Predicate::insert_after` | method | — | `NSRuleEditor.insertRow(at:with:asSubrowOfRow:animate:)` | — | Inserts a new rule, or a group when `compound`, after this id | [6](06-text-and-editing.md#rule-editor) |

## Menus

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `menu::MenuView` | struct | — | `≈ NSMenu` | `UIMenu` | Draws a list of menu items with keyboard, search and submenus | [7](07-menus-and-popovers.md#menu-view) |
| `menu::MenuItem` | enum | — | `NSMenuItem` | `UIAction` | A menu row: `Action`, `Submenu`, `Header`, `Separator`, `Palette` | [7](07-menus-and-popovers.md#menu-items) |
| `menu::CheckState` | enum | — | `≈ NSControl.StateValue` | — | Mark of an action: `None`, `Checkbox(bool)`, `Radio(bool)`, `Mixed` (a dash, dbusmenu `toggle-state` -1), `Minimized` (the Dock's diamond) | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuIcon` | enum | — | `≈ NSMenuItem.image` | — | Row icon: `Svg` or `Swatch` color circle | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::action` | fn | `Button` | `NSMenuItem(title:action:keyEquivalent:)` | `UIAction(title:handler:)` | An enabled action row with an id and a label | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::submenu` | fn | `Menu` | `NSMenuItem.submenu` | `UIMenu(children:)` | A row that opens these items | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::header` | fn | `Section(header:)` | `NSMenuItem.sectionHeader(title:)` | `UIMenu(title:options: .displayInline)` | A section title row | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::palette` | fn | `ControlGroup` | `NSMenu.palette(images:titles:)` | `UIMenu.Options.displayAsPalette` | A row of icon cells, each an action | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::placeholder` | fn | — | — | — | A disabled row that only shows a label | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::with_id` | method | — | `NSMenuItem.identifier` | `UIMenu.identifier` | The id of a submenu | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::with_shortcut` | method | `.keyboardShortcut(_:modifiers:)` | `NSMenuItem.keyEquivalent` | `UIKeyCommand` | The shortcut of an action, as text such as `"Ctrl+S"` | [7](07-menus-and-popovers.md#shortcuts) |
| `menu::MenuItem::with_enabled` | method | `.disabled(_:)` | `NSMenuItem.isEnabled` | `UIMenuElement.Attributes.disabled` | Enables or disables an action or a submenu | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::with_check` | method | `Toggle` | `NSMenuItem.state` | `UIMenuElement.State` | The `CheckState` mark of an action | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::with_icon` | method | `Label` | `NSMenuItem.image` | `UIAction.image` | The icon of an action or a submenu | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuItem::with_badge` | method | `.badge(_:)` | `NSMenuItem.badge` | — | A badge text at the end of an action | [7](07-menus-and-popovers.md#menu-items) |
| `menu::MenuIcon::svg` | fn | — | — | — | An icon from SVG text | [7](07-menus-and-popovers.md#menu-items) |
| `menu::ItemsSource` | type | — | — | — | Closure that returns a menu's items on every render | [7](07-menus-and-popovers.md#helpers) |
| `menu::normalize_separators` | fn | — | — | — | Removes leading, trailing and repeated separators, also in submenus | [7](07-menus-and-popovers.md#helpers) |
| `menu::MNEMONIC_MARK` | constant | — | `≈ NSMenuItem.mnemonic` | — | U+2063 placed before the letter that Alt underlines | [7](07-menus-and-popovers.md#mnemonics) |
| `menu::strip_mnemonic` | fn | — | — | — | Returns a label without the mnemonic mark | [7](07-menus-and-popovers.md#mnemonics) |
| `menu::format_shortcut` | fn | — | — | — | Turns shortcut text such as `"Ctrl+Shift+G"` into glyphs: `"⌃⇧G"` on Linux, `"⇧⌘G"` on macOS | [7](07-menus-and-popovers.md#shortcuts) |
| `menu::shortcut_matches` | fn | — | — | — | Whether a key event with modifiers is the given shortcut text | [7](07-menus-and-popovers.md#shortcuts) |
| `menu::measure_text` | fn | — | — | — | Width in pixels of text in the system sans font | [7](07-menus-and-popovers.md#helpers) |
| `menu::measure_text_weight` | fn | — | — | — | Width in pixels of text at a given font weight | [7](07-menus-and-popovers.md#helpers) |
| `menu::measure_wrapped` | fn | — | — | — | Widest line and number of lines of text wrapped at a width | [7](07-menus-and-popovers.md#helpers) |

## Menu search

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|

## Context menus, popovers and tooltips

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `modifiers::ViewModifiers` | trait | `View` modifiers | — | — | SwiftUI-style modifiers for every element and control with events | [7](07-menus-and-popovers.md#view-modifiers) |
| `modifiers::ViewModifiers::context_menu` | method | `.contextMenu(menuItems:)` | `NSMenu.popUpContextMenu(_:with:for:)` | `UIContextMenuInteraction` | Opens a menu at the pointer on secondary click | [7](07-menus-and-popovers.md#context_menu) |
| `popover::Popover` | struct | `.popover(isPresented:attachmentAnchor:arrowEdge:content:)` | `NSPopover` | `UIPopoverPresentationController` | Content in a popover with an arrow; `.show(anchor)` opens it below an area | [7](07-menus-and-popovers.md#popover) |
| `popover::Popover::on_dismissed` | method | — | ≈ `NSPopoverDelegate.popoverDidClose(_:)` | ≈ `UIPopoverPresentationControllerDelegate.presentationControllerDidDismiss(_:)` | Calls the closure when the popup closes; `.show` returns its `PopupId` | [7](07-menus-and-popovers.md#popover) |
| `modifiers::ViewModifiers::help` | method | `.help(_:)` | `NSView.toolTip` | `UIToolTipInteraction` | Tooltip with a fixed text after a 1 s hover | [7](07-menus-and-popovers.md#help-tooltips) |

## Accessibility, file dialogs and documents

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `accessibility::AccessibilityModifiers` | trait | Accessibility modifiers | `NSAccessibilityProtocol` | `UIAccessibilityElement` | The accessibility modifiers below, on every view that has Freya's `AccessibilityExt` | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_label` | method | `.accessibilityLabel(_:)` | `NSAccessibilityProtocol.accessibilityLabel()` | `UIAccessibilityElement.accessibilityLabel` | The name a screen reader speaks; on macOS it reaches AXTitle | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_hint` | method | `.accessibilityHint(_:)` | `NSAccessibilityProtocol.accessibilityHelp()` | `UIAccessibilityElement.accessibilityHint` | What the view does; on macOS it reaches AXHelp | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_value` | method | `.accessibilityValue(_:)` | `NSAccessibilityProtocol.accessibilityValue()` | `UIAccessibilityElement.accessibilityValue` | The current value of the view | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_hidden` | method | `.accessibilityHidden(_:)` | `NSAccessibilityProtocol.isAccessibilityElement()` | `UIAccessibilityElement.isAccessibilityElement` | Removes the view and its children from the accessibility tree | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_identifier` | method | `.accessibilityIdentifier(_:)` | `NSAccessibilityProtocol.accessibilityIdentifier()` | `UIAccessibilityIdentification.accessibilityIdentifier` | A stable id that the AX API reads as AXIdentifier | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_element` | method | `.accessibilityElement(children:)` | — | `UIAccessibilityContainerType` | Makes the view one element that combines, ignores or contains its children | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityModifiers::accessibility_add_traits` | method | `.accessibilityAddTraits(_:)` | `NSAccessibilityProtocol.accessibilityRole()` | `UIAccessibilityElement.accessibilityTraits` | Gives the view a role (button, header, link, image, static text) or selects it | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityChildBehavior` | enum | `AccessibilityChildBehavior` | — | — | `Ignore`, `Combine` or `Contain`: what the view does with its children's elements | [5](05-controls.md#accessibility-modifiers) |
| `accessibility::AccessibilityTraits` | struct | `AccessibilityTraits` | `NSAccessibility.Role` | `UIAccessibilityTraits` | A set of `IS_BUTTON`, `IS_HEADER`, `IS_LINK`, `IS_IMAGE`, `IS_STATIC_TEXT` and `IS_SELECTED`, joined with `\|` | [5](05-controls.md#accessibility-modifiers) |
| `file_dialog::ContentType` | enum | ≈ `UTType` | `NSSavePanel.allowedFileTypes` | `UTType` | A file kind a dialog allows: `Folder` or `extension("png")` | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog` | struct | ≈ `.fileImporter(isPresented:allowedContentTypes:onCompletion:)` | `NSOpenPanel`, `NSSavePanel` | `UIDocumentPickerViewController` | An open or save dialog with a title, kinds, a selection rule and a start folder | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::open` | fn | — | `NSOpenPanel.openPanel()` | — | An open dialog | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::save` | fn | — | `NSSavePanel.savePanel()` | — | A save dialog that proposes a name | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::title` | method | — | `NSSavePanel.message` | — | The line of text above the file list | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::allowed_content_types` | method | `allowedContentTypes` | `NSSavePanel.allowedFileTypes` | — | The kinds the dialog allows; `Folder` alone chooses a folder | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::allows_multiple_selection` | method | `allowsMultipleSelection` | `NSOpenPanel.allowsMultipleSelection` | — | Lets the user choose several files | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::directory` | method | ≈ `.fileDialogDefaultDirectory(_:)` | `NSSavePanel.directoryURL` | — | The folder the dialog starts in | [8](08-windows.md#documents-and-file-dialogs) |
| `file_dialog::FileDialog::present` | method | — | `NSSavePanel.beginSheetModal(for:)` | — | Shows the dialog as a sheet of a window and returns the chosen paths; empty on cancel | [8](08-windows.md#documents-and-file-dialogs) |
| `modifiers::ViewModifiers::file_importer` | method | `.fileImporter(isPresented:allowedContentTypes:allowsMultipleSelection:onCompletion:)` | `NSOpenPanel.beginSheetModal(for:)` | `UIDocumentPickerViewController` | A sheet that chooses files while the `State<bool>` is true | [8](08-windows.md#documents-and-file-dialogs) |
| `modifiers::ViewModifiers::file_exporter` | method | ≈ `.fileExporter(isPresented:document:contentType:defaultFilename:onCompletion:)` | `NSSavePanel.beginSheetModal(for:)` | `UIDocumentPickerViewController` | A sheet that chooses where to save; the app writes the file | [8](08-windows.md#documents-and-file-dialogs) |
| `modifiers::EnvironmentModifiers::file_dialog_default_directory` | method | `.fileDialogDefaultDirectory(_:)` | `NSSavePanel.directoryURL` | — | The start folder of the dialogs below the view | [8](08-windows.md#documents-and-file-dialogs) |
| `modifiers::ViewModifiers::on_open_url` | method | `.onOpenURL(perform:)` | `NSApplicationDelegate.application(_:open:)` | `UIApplicationDelegate.application(_:open:options:)` | Runs for each file or address that Finder, `open` or a link hands to the app | [8](08-windows.md#documents-and-file-dialogs) |
| `document::OpenedUrl` | enum | `URL` | `NSURL` | `URL` | What `on_open_url` receives: `File(path)` or `Other(text)` | [8](08-windows.md#documents-and-file-dialogs) |
| `document::Document` | struct | `FileDocument` | `NSDocument` | `UIDocument` | A window's document: `is_edited`, `set_edited`, `url`, `set_url` and `save()` | [8](08-windows.md#documents-and-file-dialogs) |
| `document::use_document` | fn | ≈ `DocumentGroup` | `NSDocument.isDocumentEdited` | `UIDocument.hasUnsavedChanges` | Hook: makes the window a document; an edited one asks "Save changes?" before it closes | [8](08-windows.md#documents-and-file-dialogs) |

## Windows

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `platform::run_window` | fn | `≈ App` | `≈ NSApplicationMain(_:_:)` | `UIApplicationMain` | Opens a window with a root view and runs the event loop | [8](08-windows.md#running-a-window) |
| `platform::WindowOptions::theme` | method | ≈ `.environment(_:_:)` on the window | ≈ `NSAppearanceCustomization.appearance` | ≈ `UIView.overrideUserInterfaceStyle` | Gives one window its `Theme` | [8](08-windows.md#window-options) |
| `platform::WindowOptions` | struct | `≈ Window` | — | `UIWindowScene` | Title, app id and its name, size, limits, menu and full-screen options of a window | [8](08-windows.md#window-options) |
| `platform::WindowControls` | struct | — | `≈ NSWindow` | `UIWindow` | Window state and commands: close, minimize, zoom, full screen, move, resize | [8](08-windows.md#window-controls) |
| `platform::WindowControls::make_key_and_order_front` | method | — | `NSWindow.makeKeyAndOrderFront(_:)` | `UIWindow.makeKeyAndVisible()` | Brings the window to the front with the keyboard, using the last press | [8](08-windows.md#window-controls) |
| `platform::WindowStatus` | struct | — | — | — | Configure state: activated, maximized, fullscreen, decoration, fixed | [8](08-windows.md#window-state) |
| `platform::DecorationResponse` | enum | — | — | — | Who draws the title bar: `Client`, `Server` or `Hybrid` (system lights, kit bar) | [8](08-windows.md#window-state) |
| `platform::WindowStyle` | enum | `WindowStyle` | `≈ NSWindow.StyleMask` | — | Who frames a main window on macOS: `TitleBar`, `HiddenTitleBar`, `Plain` (the kit's Linux frame) or `Borderless` (SwiftUI's `.plain`; AppKit's `.borderless`, as the kit's `Plain` is another frame) | [8](08-windows.md#window-style) |
| `platform::WindowOptions::min_size`, `max_size` | method | `frame(minWidth:minHeight:)` on the window's root | `NSWindow.contentMinSize` | — | Limits of the content below the window's bar; the window adds the bar | [8](08-windows.md#window-options) |
| `platform::WindowOptions::window_style` | method | `windowStyle(_:)` | — | — | Chooses the `WindowStyle` of a window | [8](08-windows.md#window-style) |
| `platform::WindowResizability`, `WindowOptions::window_resizability` | enum, method | `WindowResizability`, `windowResizability(_:)` | `NSWindow.contentMinSize`, `contentMaxSize` | — | `ContentSize` fits the window to its view and keeps it there | [8](08-windows.md#window-options) |
| `platform::WindowToolbarFullScreenVisibility`, `WindowOptions::window_toolbar_full_screen_visibility` | enum, method | `WindowToolbarFullScreenVisibility`, `windowToolbarFullScreenVisibility(_:)` | `NSApplication.PresentationOptions.autoHideToolbar` | — | `Automatic` pins the bar in full screen; `OnHover` brings it down with the menu bar | [8](08-windows.md#window-options) |
| `platform::WindowEdge` | enum | — | — | — | Edge or corner for a compositor resize | [8](08-windows.md#resizing-from-the-edges) |
| `app::UtilityWindow` | struct | `UtilityWindow` | `≈ NSPanel` | — | A floating panel child of the current window; `.open()` shows it | [8](08-windows.md#utility-windows) |
| `platform::PanelBehavior` | struct | — | `≈ NSPanel` | — | Panel flags: hides on deactivate, floating, key only if needed | [8](08-windows.md#utility-windows) |
| `platform::Bar` | enum | — | `≈ NSWindow.ToolbarStyle` | — | Bar the window draws, for its corner: `Titlebar`, `UnifiedCompact`, `Unified`, `Utility` | [9](09-titlebars.md#corner-radius) |
| `platform::MainMenuAddress` | struct | — | — | — | D-Bus service and path of a window's menu bar menu | [8](08-windows.md#menu-address) |
| `platform::ObjectId` | struct | — | — | — | Wayland object id, re-exported from `wayland-backend` | [8](08-windows.md#menu-address) |
| `platform::RunError` | enum | — | — | — | Why a window cannot run: display, protocol, Vulkan, event loop, I/O | [8](08-windows.md#errors) |

## Popups

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `platform::Popups` | struct | — | — | — | Opens and closes `xdg_popup`s from a window or popup | [8](08-windows.md#popups) |
| `platform::PopupSpec` | struct | — | — | — | Popup position, size, constraint adjustment, grab and dismiss callback | [8](08-windows.md#popups) |
| `platform::PopupAnchor` | enum | — | — | — | Point of the anchor rectangle a popup attaches to; eight values | [8](08-windows.md#popups) |
| `platform::PopupGravity` | enum | — | — | — | Direction a popup grows from its anchor point; eight values | [8](08-windows.md#popups) |
| `platform::PopupConstraintAdjustment` | struct | — | — | — | Flip, slide or resize flags for a popup that does not fit | [8](08-windows.md#popups) |
| `platform::PopupId` | struct | — | — | — | Identifies an open popup | [8](08-windows.md#popups) |
| `platform::PopupHandle` | struct | — | — | — | A popup's own handle: close, reposition, open a child | [8](08-windows.md#popups) |
| `platform::PopoverShape` | enum | — | — | — | Corner and arrow of a popover: `Popover`, `Menu`, `Label` | [7](07-menus-and-popovers.md#other-popover-shapes) |

## Timers and input

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `platform::Timers` | struct | — | — | — | Runs callbacks on the UI thread after a delay or on wake | [8](08-windows.md#timers) |
| `platform::Timers::every` | method | ≈ `Timer.scheduledTimer(repeats: true)` | `Timer.scheduledTimer(withTimeInterval:repeats:)` | ≈ `Timer.scheduledTimer(withTimeInterval:repeats:block:)` | Calls a callback at a fixed interval until its `Timer` is invalidated | [8](08-windows.md#timers) |
| `app::use_timeline` | fn | ≈ `TimelineView(.periodic(from:by:))` | — | ≈ `CADisplayLink` | Hook that renders the component every interval while it is active | [8](08-windows.md#timers) |
| `platform::WakeHandle` | struct | — | — | — | `Send` handle another thread uses to run wake callbacks | [8](08-windows.md#timers) |
| `platform::WakeHandle::invalidate` | method | — | ≈ `Timer.invalidate()` | — | Unregisters the `wakeup` callback the handle came from | [8](08-windows.md#timers) |
| `platform::Timer` | struct | ≈ `Timer` | `Timer` | `Timer` | A repeating callback that `every` returns | [8](08-windows.md#timers) |
| `platform::Timer::invalidate` | method | ≈ `Timer.invalidate()` | `Timer.invalidate()` | `Timer.invalidate()` | Stops the repeating callback | [8](08-windows.md#timers) |
| `platform::modifier_flags` | fn | — | `NSEvent.modifierFlags` | — | Modifier keys held now, for a pointer handler | [8](08-windows.md#input-helpers) |
| `platform::inserts_text` | fn | — | — | — | Whether a key press types text: no Control, Alt or Super | [8](08-windows.md#input-helpers) |

## Titlebars

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `titlebar::WindowToolbarStyle` | enum | `WindowToolbarStyle` | `≈ NSWindow.ToolbarStyle` | — | The bar of a `Titlebar`: `Unified` (52 high, title at the start), `UnifiedCompact` (40 high, title at the start) or `Expanded` (a 28 title row over a 40 toolbar row, items 28 high) | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar::subtitle` | method | `.navigationSubtitle(_:)` | `NSWindow.subtitle` | — | A second line under the title, or after it in an expanded bar | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar::shows_title` | method | `.unified(showsTitle:)`, `.unifiedCompact(showsTitle:)` | `NSWindow.titleVisibility` | — | `false` drops the title; the trailing items then follow the lights | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar::toolbar_background_visibility` | method | `.toolbarBackgroundVisibility(_:for:)` | ≈ `NSWindow.titlebarAppearsTransparent` | — | `Visibility::Hidden` leaves the bar with no fill and no hairline, so the content under it shows | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar::toolbar_style` | method | `windowToolbarStyle(_:)` | `NSWindow.toolbarStyle` | — | Chooses the `WindowToolbarStyle`; by default `Unified` with a sidebar or items | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar` | struct | `≈ .toolbar(content:)` | `≈ NSToolbar` | ≈ `UIToolbar` | The title bar the kit draws beside the native lights (`HiddenTitleBar`) or its own (Linux, `Plain`): a left item, toolbar items by placement, title | [9](09-titlebars.md#titlebar) |
| `titlebar::Titlebar::toolbar` | method | `.toolbar(content:)` | `NSToolbar.insertItem(withItemIdentifier:at:)` | — | Adds one `ToolbarItem` or `ToolbarSpacer`, laid by placement | [9](09-titlebars.md#toolbar-items) |
| `views::toolbar::ToolbarItem` | struct | `ToolbarItem` | `≈ NSToolbarItem` | `≈ UIBarButtonItem` | A view with a `ToolbarItemPlacement` | [9](09-titlebars.md#toolbar-items) |
| `views::toolbar::ToolbarItemPlacement` | enum | `ToolbarItemPlacement` | — | — | Where an item goes: `Automatic`, `Principal`, `Navigation`, `PrimaryAction`, `Status` | [9](09-titlebars.md#toolbar-items) |
| `views::toolbar::ToolbarSpacer` | struct | `ToolbarSpacer` (`SpacerSizing` as `fixed()` and `flexible()`) | `NSToolbarItem.Identifier.space`, `.flexibleSpace` | `UIBarButtonItem.SystemItem.fixedSpace`, `.flexibleSpace` | A gap of one padding or the split between the navigation side and the trailing side | [9](09-titlebars.md#toolbar-items) |
| `views::toolbar::ToolbarContent` | enum | `ToolbarContent` | — | — | What `Titlebar::toolbar` accepts: an item or a spacer | [9](09-titlebars.md#toolbar-items) |
| `traffic::TrafficLights` | struct | — | `≈ NSWindow.standardWindowButton(_:)` | — | Close, minimize and zoom buttons | [9](09-titlebars.md#traffic-lights) |
| `traffic::TrafficLights::fullscreen` | method | — | `≈ NSWindow.styleMask.fullScreen` | — | Minimize disabled and the exit glyph on zoom, for a full-screen window | [9](09-titlebars.md#traffic-lights) |
| `traffic::TrafficLights::for_window` | fn | — | `≈ NSWindow.standardWindowButton(_:)` | — | Traffic lights wired to the window and user settings | [9](09-titlebars.md#traffic-lights) |
| `titlebar::use_window_drag` | fn | — | `≈ NSWindow.performDrag(with:)` | — | Press handler that moves the window or runs the double-click action | [9](09-titlebars.md#traffic-lights) |
| `titlebar::sidebar_bar_height` | fn | — | — | — | Height of the title bar row beside a sidebar | [9](09-titlebars.md#helpers) |

## Sidebar and split view

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `sidebar::Sidebar` | struct | `SidebarListStyle` | `NSTableView.Style.sourceList` | `UICollectionLayoutListConfiguration.Appearance.sidebar` | The navigation column: flat and full height | [10](10-sidebar.md#sidebar) |
| `sidebar::SidebarItem` | struct | ≈ `Label` | — | ≈ `UIListContentConfiguration` | A sidebar row with a bare symbol (or a tile) and label | [10](10-sidebar.md#sidebaritem) |
| `sidebar::SidebarItem::tag` | method | `.tag(_:)` in a `List(selection:)` | — | — | Selected while the `State` holds the value; a click stores it | [10](10-sidebar.md#sidebaritem) |
| `sidebar::SidebarItem::symbol_tile` | method | — | — | — | Draws the row's symbol on a colored tile | [10](10-sidebar.md#sidebaritem) |
| `styles::TitlebarStyle::line` | fn | ≈ `NSTitlebarSeparatorStyle.line` | `NSTitlebarSeparatorStyle` | — | The separator under the bar; `Titlebar::hairline(true)` draws it, no line is the default | [9](09-titlebars.md#titlebar) |
| `sidebar::SidebarSection` | struct | `Section` | — | `UITableView` | A titled, collapsible group of sidebar rows | [10](10-sidebar.md#sidebarsection) |
| `sidebar::SidebarAccount` | struct | — | — | — | The account row: avatar with initials, name and a detail line | [10](10-sidebar.md#sidebaraccount) |
| `sidebar::SidebarControl` | struct | ≈ `NavigationSplitViewVisibility` | ≈ `NSSplitViewController.toggleSidebar(_:)` | ≈ `UISplitViewController.DisplayMode` | Handle to toggle, dismiss and read the enclosing `SplitView`'s sidebar | [10](10-sidebar.md#sidebarcontrol) |
| `sidebar::sidebar_control` | fn | — | — | — | The enclosing `SplitView`'s `SidebarControl`, or `None` outside one | [10](10-sidebar.md#sidebarcontrol) |
| `sidebar::SymbolTile` | struct | — | — | — | The colored square tile with a white symbol; `.rim(color)` adds a border | [10](10-sidebar.md#symboltile) |
| `views::split::SplitView` | struct | `NavigationSplitView` | `NSSplitViewController` | `UISplitViewController` | Sidebar, optional content column, detail and optional inspector; resizes, hides and collapses the sidebar | [10](10-sidebar.md#splitview) |
| `modifiers::ViewModifiers::inspector` | method | `.inspector(isPresented:content:)` | `NSSplitViewItem(inspectorWithViewController:)` | — | A view with an inspector column and no sidebar | [10](10-sidebar.md#inspector) |
| `views::split::SplitView::sidebar_visible`, `sidebar_width`, `sidebar_range`, `resizable`, `narrow_sidebar`, `on_layout`, `inspector`, `background_extension` | method | `NavigationSplitView(columnVisibility:)`, `navigationSplitViewColumnWidth(min:ideal:max:)`, `inspector(isPresented:content:)`, `backgroundExtensionEffect()` | `NSSplitViewItem.isCollapsed`, `minimumThickness`, `maximumThickness`, `canCollapse` | `UISplitViewController.preferredPrimaryColumnWidth` | Whether the sidebar starts shown, its width and range, whether it resizes, what happens in a narrow window, the callback with its layout, the inspector column and the view mirrored under the sidebar and inspector | [10](10-sidebar.md) |
| `views::split::SplitView::content` | method | `NavigationSplitView(sidebar:content:detail:)` | `≈ NSSplitViewController.addSplitViewItem(_:)` | `≈ UISplitViewController.setViewController(_:for:)` | The column between the sidebar and the detail | [10](10-sidebar.md#content-column) |
| `views::split::SidebarLayout` | struct | — | — | — | Sidebar width and visibility reported by `on_layout`, serializable | [10](10-sidebar.md#sidebarlayout) |
| `views::split::NarrowSidebar` | enum | — | — | — | Where the sidebar opens in narrow windows: `Overlay` or `Push` | [10](10-sidebar.md#narrow-windows) |
| `views::split::SidebarToggle` | struct | — | `NSToolbarItem.Identifier.toggleSidebar` | — | The button that shows or hides the sidebar: inside the sidebar while it is docked, in the titlebar when it is hidden | [10](10-sidebar.md#hiding-sidebartoggle-and-toggle_sidebar) |
| `views::split::TOGGLE_SIDEBAR` | constant | — | `NSSplitViewController.toggleSidebar(_:)` | — | The show-or-hide-sidebar action, Ctrl+Alt+S | [10](10-sidebar.md#hiding-sidebartoggle-and-toggle_sidebar) |

## Layout

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `layout::Spacer` | struct | `Spacer` | ≈ `NSLayoutGuide` | ≈ `UILayoutGuide` | Takes the space the other children of its stack leave; `.min_length` | [11](11-views.md#spacer) |
| `layout::Frame` | struct | `frame(width:height:alignment:)` | — | `NSLayoutConstraint` | A view with its own size and the alignment of its content; `view.frame()` returns it | [11](11-views.md#frame) |
| `layout::Frame::alignment` | method | `frame(alignment:)` | — | — | Where the content sits inside the frame | [11](11-views.md#frame) |
| `layout::Grid` | struct | `Grid` | `NSGridView` | — | Rows of cells whose columns line up | [11](11-views.md#grid-and-gridrow) |
| `layout::LazyVGrid` | struct | `LazyVGrid` | ≈ `NSCollectionViewFlowLayout` | ≈ `UICollectionViewFlowLayout` | Cells in adaptive columns that share the row's width | [11](11-views.md#lazyvgrid) |
| `layout::GridItem::adaptive`, `spacing` | method | `GridItem(.adaptive(minimum:), spacing:)` | — | — | A column at least `minimum` wide, and its gap | [11](11-views.md#lazyvgrid) |
| `layout::Grid::row`, `child`, `alignment`, `horizontal_spacing`, `vertical_spacing` | method | `Grid(alignment:horizontalSpacing:verticalSpacing:)` | `NSGridView.addRow(with:)`, `rowSpacing`, `columnSpacing` | — | Adds a `GridRow` or a view that spans the row, and sets the cell alignment and the gaps between columns and rows | [11](11-views.md#grid-and-gridrow) |
| `layout::GridRow` | struct | `GridRow` | `NSGridRow` | — | The cells of one row of a `Grid` | [11](11-views.md#grid-and-gridrow) |
| `layout::GridRow::alignment` | method | `GridRow(alignment:)` | `NSGridRow.yPlacement` | — | The alignment of the cells of one row | [11](11-views.md#grid-and-gridrow) |
| `layout::GridCell` | struct | — | `NSGridCell` | — | A view with a column span or a column alignment, returned by the two grid modifiers | [11](11-views.md#grid-and-gridrow) |
| `layout::GridCell::grid_cell_columns`, `grid_column_alignment` | method | `gridCellColumns(_:)`, `gridColumnAlignment(_:)` | `NSGridCell.xPlacement`, `NSGridView.mergeCells(inHorizontalRange:verticalRange:)` | — | The columns a cell spans, and the horizontal alignment of its column | [11](11-views.md#grid-and-gridrow) |
| `layout::GridModifiers` | trait | `View` modifiers | — | — | `grid_cell_columns` (`.gridCellColumns(_:)`, ≈ `NSGridCell.mergeCells`) and `grid_column_alignment` (`.gridColumnAlignment(_:)`, ≈ `NSGridColumn.xPlacement`) | [11](11-views.md#grid-and-gridrow) |
| `layout::ContentAlignment` | struct | `Alignment` | — | `UIStackView.Alignment` | Where content sits in its space on both axes; nine constants from `TOP_LEADING` to `BOTTOM_TRAILING`. SwiftUI's name is taken by torin's `Alignment` | [11](11-views.md#frame) |
| `layout::HorizontalAlignment` | enum | `HorizontalAlignment` | `NSLayoutConstraint.Attribute` (`leading`, `centerX`, `trailing`) | `UIStackView.Alignment` | `Leading`, `Center` or `Trailing`; the left or right edge by the layout direction | [11](11-views.md#frame) |
| `layout::VerticalAlignment` | enum | `VerticalAlignment` | `NSLayoutConstraint.Attribute` (`top`, `centerY`, `bottom`) | `UIStackView.Alignment` | `Top`, `Center` or `Bottom`; no baselines | [11](11-views.md#frame) |
| `layout::ZStack` | type | `ZStack` | — | — | A container that draws its children on top of one another; `.alignment(ContentAlignment)` places them. An alias of `LayoutContainer<ZStackLayout>` | [11](11-views.md#zstack-overlay-and-background_view) |
| `layout::ZStackLayout` | struct | `ZStackLayout` | — | — | The `Layout` of a `ZStack`: sized by its largest child, `alignment` places the rest | [11](11-views.md#zstack-overlay-and-background_view) |
| `layout::OverlayModifiers` | trait | `View` modifiers | — | — | `overlay` (`.overlay(alignment:content:)`) and `background_view` (`.background(alignment:content:)`; Freya's `background` is taken): the view alone sizes the result | [11](11-views.md#zstack-overlay-and-background_view) |
| `layout::Layout` | trait | `Layout` | — | — | A rule that places subviews: `place_subviews(proposal, sizes)` returns a `Placement`. SwiftUI's `sizeThatFits` and `placeSubviews` are one call | [11](11-views.md#custom-layout) |
| `layout::LayoutContainer` | struct | a `Layout` called as a view | — | — | The view of a `Layout`: `LayoutContainer::with_layout(layout).child(..)` | [11](11-views.md#custom-layout) |
| `layout::Placement` | struct | `ProposedViewSize` result of `sizeThatFits` plus `place(at:)` | — | — | The size of a container and the origin of each subview in it | [11](11-views.md#custom-layout) |
| `layout::ViewThatFits` | struct | `ViewThatFits` | — | — | Shows the first candidate whose stated extent fits the width or height offered, the last when none does | [11](11-views.md#viewthatfits) |
| `layout::ViewThatFits::candidate` | method | the content of `ViewThatFits` | — | — | Adds a view with the extent it needs, tried in order | [11](11-views.md#viewthatfits) |
| `modifiers::EnvironmentModifiers::layout_direction` | method | `.environment(\.layoutDirection, _)` | `NSView.userInterfaceLayoutDirection` | `UIView.semanticContentAttribute` | Sets the layout direction of a view: mirrors the stacks, frames, grids, ZStacks and texts already built in it and sets it for everything built below, controls included | [11](11-views.md#right-to-left) |

## Forms and groups

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `views::form::Form` | struct | `Form` | — | `UITableView` | Settings layout: sections of rows in rounded cards | [11](11-views.md#form) |
| `views::form::Form::section`, `highlighted` | method | the content of `Form` | — | — | Adds a `Section`; highlights the row with a title, as a settings search does | [11](11-views.md#form) |
| `views::form::Section` | struct | `Section` | — | `UITableView` | One form card with optional title and footer | [11](11-views.md#section) |
| `views::form::Section::titled`, `row`, `footer` | method | `Section(_:)`, `Section(footer:)` | — | `UITableView` section header and footer | A section with a title, a row added to it, and the text below the card | [11](11-views.md#section) |
| `views::form::Section::header` | method | `Section(content:header:)` | — | `UITableView` | A view in place of a form card's title | [11](11-views.md#section) |
| `views::form::Section::illustration` | method | — | — | — | A full-width view at the top of a form card | [11](11-views.md#section) |
| `views::form::Row` | struct | `LabeledContent` | — | `UIListContentConfiguration` | A form row: control, value, navigation or destructive | [11](11-views.md#row) |
| `views::form::Row::value`, `navigation`, `destructive`, `control`, `detail`, `keywords` | fn, method | `LabeledContent(_:value:)`, `NavigationLink`, `Button(role: .destructive)` | — | `UIListContentConfiguration` | A row that shows a value, opens a page or runs a destructive action; the control after its title, the second line under it, and the words a settings search matches | [11](11-views.md#form) |
| `views::form::Row::page` | function | ≈ `NavigationLink(value:)` | — | ≈ `UINavigationController.pushViewController(_:animated:)` | A row that opens a page of a `SettingsWindow` | [11](11-views.md#row) |
| `views::form::Row::button` | function | `Button` in a `Form` | — | `UIButton`, `UITableView` | A row with a bordered button at the start | [11](11-views.md#row) |
| `views::form::Row::toggle` | function | `Toggle` in a `Form` | — | `UISwitch`, `UITableView` | A row with a switch on the right | [11](11-views.md#row) |
| `views::form::Row::symbol` | method | ≈ `Label(_:systemImage:)` | — | ≈ `UIListContentConfiguration` | A `SymbolTile` before the row title | [11](11-views.md#row) |
| `views::form::Row::icon` | method | `Label(title:icon:)` | — | `UIListContentConfiguration` | Any view before the row title | [11](11-views.md#row) |
| `views::form::Row::hover_control` | method | ≈ `.onHover` | — | ≈ `UIHoverGestureRecognizer` | A control shown only while the pointer is over the row | [11](11-views.md#row) |
| `views::form::Row::stacked` | method | ≈ `.pickerStyle(.radioGroup)` under its label | — | ≈ `UISegmentedControl` | The control goes under the title at any width | [11](11-views.md#row) |
| `views::group::GroupBox` | struct | `GroupBox` | `NSBox` | — | A rounded box with an optional label that groups controls | [11](11-views.md#groupbox) |
| `views::group::GroupBox::child` | method | the content of `GroupBox` | `NSBox.contentView` | — | Adds a view inside the box | [11](11-views.md#groupbox) |
| `views::group::DisclosureGroup` | struct | `DisclosureGroup` | — | `UICellAccessory.OutlineDisclosure` | A chevron header that shows or hides its indented children | [11](11-views.md#disclosuregroup) |
| `views::group::DisclosureGroup::expanded`, `child` | method | `DisclosureGroup(isExpanded:)` | — | — | Whether the group starts open, and a view in its indented content | [11](11-views.md#disclosuregroup) |

## Data views

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `views::list::List` | struct | `List` | `NSTableView` without header | `UITableView` | One column of rows, text or views, with separators and selection; `inset`, `plain` and `bordered` styles | [11](11-views.md#list) |
| `views::list::List::selection`, `style` | method | `List(selection:)`, `listStyle(_:)` | `NSTableView.selectedRowIndexes` | `UITableView.indexPathsForSelectedRows` | The `State` of the selected rows (without it rows do not select), and the `TableStyle` of one list | [11](11-views.md#list) |
| `views::list::List::row` | method | `ForEach` in a `List` | `NSTableView.view(atColumn:row:makeIfNecessary:)` | `UITableView.cellForRow(at:)` | Draws every row as a view built from the row index | [11](11-views.md#views-in-rows-and-cells) |
| `views::list::List::section` | method | `Section` in a `List` | group rows of `NSTableView` | `UITableView` | A header over a group of rows | [11](11-views.md#list) |
| `views::table::Table` | struct | `Table` | `NSTableView` | ≈ `UITableView` | Rows of text or views in sortable, resizable, hideable columns with multiple selection | [11](11-views.md#table) |
| `views::table::Table::rows`, `selection`, `sort_order` | method | `Table(_:selection:sortOrder:)` | `NSTableView.selectedRowIndexes`, `sortDescriptors` | `UITableView.indexPathsForSelectedRows` | The text of the rows, the `State` of the selected rows, and the `State` of the sorted column, which a header click changes | [11](11-views.md#table) |
| `views::table::Table::cell` | method | `TableColumn(_:content:)` | `NSTableView.viewFor(tableColumn:row:)` | ≈ `UITableView.cellForRow(at:)` | Draws the cell of a column as a view built from the row index | [11](11-views.md#views-in-rows-and-cells) |
| `views::table::Table::icons` | method | `Label` in a `TableColumn` | `NSTableCellView.imageView` | `UITableViewCell.imageView` | One symbol per row before the title of the first column | [11](11-views.md#table) |
| `views::table::ListRowModifiers` | trait | `View` modifiers | — | — | The `list_row_*` methods of `List` and `Table`; each takes a closure of the row index | [11](11-views.md#list-row-modifiers) |
| `views::table::ListRows` | struct | — | — | — | The row closures a `List` or `Table` holds for `ListRowModifiers`; an app never builds one | [11](11-views.md#list-row-modifiers) |
| `views::table::ListRowModifiers::list_row_insets` | method | `.listRowInsets(_:)` | ≈ `NSTableView.intercellSpacing` | `UIView.layoutMargins` | Horizontal padding of the cells of one row; the height stays `row_height` | [11](11-views.md#list-row-modifiers) |
| `views::table::ListRowModifiers::list_row_separator` | method | `.listRowSeparator(_:edges:)` | ≈ `NSTableView.gridStyleMask` | `UITableView.separatorStyle` | Shows or hides the line under one row | [11](11-views.md#list-row-modifiers) |
| `views::table::ListRowModifiers::list_row_background` | method | `.listRowBackground(_:)` | ≈ `NSTableRowView.backgroundColor` | `UITableViewCell.backgroundView` | Fills one row that is not selected | [11](11-views.md#list-row-modifiers) |
| `views::table::Visibility` | enum | `Visibility` | ≈ `NSView.isHidden` | ≈ `UIView.isHidden` | `Automatic`, `Visible`, `Hidden` | [11](11-views.md#list-row-modifiers) |
| `views::table::Table::scroll_bar`, `views::list::List::scroll_bar`, `views::tree::OutlineView::scroll_bar` | method | `scrollIndicators(_:)` | `NSScrollView.hasVerticalScroller` | `UIScrollView.showsVerticalScrollIndicator` | Shows or hides the scroll bar | [11](11-views.md#table) |
| `views::table::SortOrder` | struct | — | ≈ `NSTableView.sortDescriptors` | — | The sorted column index and direction; one column, not a list | [11](11-views.md#sortorder) |
| `views::table::next_sort` | fn | — | — | — | The order a header click asks for: flip or start ascending | [11](11-views.md#table-helpers) |
| `views::table::shown_columns` | fn | — | — | — | Column indices not hidden | [11](11-views.md#table-helpers) |
| `views::table::toggle_column` | fn | — | — | — | Hides or shows a column, keeping at least one visible | [11](11-views.md#table-helpers) |
| `views::collection::CollectionView` | struct | ≈ `LazyVGrid` | `NSCollectionView` | `UICollectionView` | A wrapping grid of icon tiles with names and multiple selection | [11](11-views.md#collectionview) |
| `views::collection::CollectionView::item`, `selection` | method | `ForEach` in a `LazyVGrid`, `List(selection:)` | `NSCollectionView.selectionIndexPaths` | `UICollectionView.indexPathsForSelectedItems` | Adds a tile with an icon and a name; the `State` of the selected indexes | [11](11-views.md#collectionview) |
| `views::collection::select` | fn | — | — | — | Click selection rule: plain, Ctrl/Super toggle, Shift range | [11](11-views.md#select) |
| `views::tree::TreeNode` | struct | — | ≈ `NSTreeNode` | — | A node with title, optional icon and children | [11](11-views.md#treenode) |
| `views::tree::TreeNode::icon`, `child` | method | the children of `OutlineGroup` | `NSTreeNode.children` | — | The symbol before the title, and a node added under this one | [11](11-views.md#treenode) |
| `views::tree::OutlineView` | struct | `OutlineGroup` | `NSOutlineView` | ≈ `UICollectionView` | A tree as indented rows with disclosure chevrons | [11](11-views.md#outlineview) |
| `views::tree::OutlineView::selection`, `views::tree::Browser::selection` | method | `List(selection:)` | `NSOutlineView.selectedRowIndexes`, `NSBrowser.selectionIndexPaths` | — | The `State` of the selected nodes as paths of child indexes; without it the view keeps its own | [11](11-views.md#outlineview) |
| `views::tree::TreeNode::expanded` | method | `DisclosureGroup(isExpanded:)` | `NSOutlineView.expandItem(_:)` | `UICollectionViewDiffableDataSource.apply` | A node that starts open | [11](11-views.md#treenode) |
| `views::tree::open_paths` | fn | — | — | — | The paths of the nodes marked expanded | [11](11-views.md#tree-helpers) |
| `views::tree::Browser` | struct | — | `NSBrowser` | — | A tree as Finder-style columns | [11](11-views.md#browser) |
| `views::tree::visible_rows` | fn | — | — | — | Path and depth of each row shown for a set of open paths | [11](11-views.md#tree-helpers) |
| `views::tree::node_at` | fn | — | — | — | The node at a path, or `None` | [11](11-views.md#tree-helpers) |

## Containers

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `views::tabs::TabView` | struct | `TabView` | `NSTabView` | ≈ `UITabBarController` | Pages chosen from a strip of tabs on the top or bottom of a box, or icon-over-title tabs | [11](11-views.md#tabview) |
| `views::tabs::TabView::tab`, `selection` | method | `Tab(_:systemImage:)`, `TabView(selection:)` | `NSTabView.addTabViewItem(_:)`, `selectTabViewItem(at:)` | `UITabBarController.selectedIndex` | Adds a tab with a symbol, a title and its page; the `State` of the selected tab, without which the view keeps its own | [11](11-views.md#tabview) |
| `views::tabs::TabView::size` | method | `controlSize(_:)` | `NSTabView.controlSize` | — | Small and mini strips | [11](11-views.md#tabview) |
| `views::split_panes::SplitPanes` | struct | `HSplitView`, `VSplitView` | `NSSplitView` | ≈ `UISplitViewController` | Panes in a row or column with draggable dividers; controlled ratios | [11](11-views.md#splitpanes) |
| `views::split_panes::SplitPanes::pane`, `ratios`, `on_ratios` | method | the content of `HSplitView` | `NSSplitView.addArrangedSubview(_:)`, `setPosition(_:ofDividerAt:)` | — | Adds a pane; the share of each pane and the callback that receives new shares during a drag | [11](11-views.md#splitpanes) |
| `views::text_view::TextView` | struct | ≈ `TextEditor` | `NSTextView` | `UITextView` | Frameless multi-line editable text without scrolling | [11](11-views.md#textview) |
| `views::text_view::TextView::on_focus_change`, `auto_focus`, `cursor`, `width`, `height` | method | `focused(_:)` | `NSTextView.delegate`, `insertionPointColor` | `UITextView.delegate` | The callback for focus, focus on mount, the caret color and the size | [11](11-views.md#textview) |
| `views::text_view::KeyPress` | struct | `KeyPress` | ≈ `NSEvent` | ≈ `UIEvent` | Key, modifiers and caret position passed to `TextView::on_key` | [11](11-views.md#keypress) |

## Feedback

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `views::feedback::ContentUnavailableView` | struct | `ContentUnavailableView` | — | `UIContentUnavailableConfiguration` | A faint symbol over a line of text for an empty view; it fills its space and centers its content | [11](11-views.md#contentunavailableview) |
| `views::feedback::Sheet` | struct | `.sheet(isPresented:onDismiss:content:)` | `NSWindow.beginSheet(_:completionHandler:)` | `UISheetPresentationController` | A modal child window centered on its window (`xdg-dialog-v1`); the parent takes no clicks and cannot close while it shows | [11](11-views.md#sheet) |
| `views::feedback::Alert` | struct | `.alert(_:isPresented:actions:message:)` | `NSAlert` | `UIAlertController` | A sheet with title, message, two buttons and optional suppression checkbox | [11](11-views.md#alert) |
| `views::feedback::Alert::confirm`, `cancel`, `suppression` | method | the buttons of `.alert`, `dialogSuppressionToggle(isSuppressed:)` | `NSAlert.addButton(withTitle:)`, `showsSuppressionButton` | `UIAlertAction` | The label and callback of each button; the checkbox that stores "do not show again" in a `State<bool>` | [11](11-views.md#alert) |
| `views::feedback::SheetButtons` | struct | ≈ `ToolbarItem(placement: .confirmationAction)` | — | ≈ `UIBarButtonItem` | Cancel and default buttons at the bottom of a sheet, with an optional view at the left | [11](11-views.md#sheetbuttons) |
| `views::feedback::SheetButtons::confirm_disabled`, `leading` | method | `.disabled(_:)` on the confirmation item | — | — | Disables the default button; puts a view at the leading end of the button row | [11](11-views.md#sheetbuttons) |
| `views::feedback::SheetButtons::large` | method | ≈ `.controlSize(.large)` | `NSControl.controlSize` | — | Large buttons of `DIALOG_BUTTON_WIDTH` | [11](11-views.md#sheetbuttons) |
| `views::feedback::SheetButtons::equal_width` | method | — | — | — | The two buttons split the row evenly, as in an alert | [11](11-views.md#sheetbuttons) |

## Navigation and search

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `views::navigation::History` | struct | — | — | — | A back-and-forward list of pages | [11](11-views.md#history) |
| `views::navigation::History::push`, `back`, `forward`, `current`, `can_go_back`, `can_go_forward` | method | — | ≈ `WKBackForwardList` | — | Adds a page after the current one and drops the pages ahead of it; moves back and forward; the current page and whether a move is possible | [11](11-views.md#history) |
| `views::navigation::NavigationButtons` | struct | — | — | — | Back and forward buttons in a tool group, with Alt+arrow shortcuts | [11](11-views.md#navigationbuttons) |
| `modifiers::ViewModifiers::sheet` | method | `sheet(isPresented:content:)` | ≈ `NSViewController.presentAsSheet(_:)` | ≈ `UIViewController.present(_:animated:)` | Shows a `Sheet` while a `State<bool>` is true | [11](11-views.md#sheet) |
| `modifiers::ViewModifiers::alert` | method | `alert(_:isPresented:actions:message:)` | ≈ `NSAlert.beginSheetModal(for:)` | ≈ `UIViewController.present(_:animated:)` | Shows an `Alert` while a `State<bool>` is true; any button, Enter or Escape writes `false` | [11](11-views.md#sheet) |
| `modifiers::ViewModifiers::popover` | method | `popover(isPresented:attachmentAnchor:arrowEdge:content:)` | ≈ `NSPopover.show(relativeTo:of:preferredEdge:)` | ≈ `UIViewController.present(_:animated:)` | Opens a `Popover` of the given size at the view while a `State<bool>` is true; a click outside writes `false` | [11](11-views.md#sheet) |
| `modifiers::ViewModifiers::on_hover` | method | `onHover(perform:)` | ≈ `NSTrackingArea` with `mouseEntered(with:)` and `mouseExited(with:)` | `UIHoverGestureRecognizer` | Calls the closure with `true` when the pointer enters the view and `false` when it leaves; it shares the pointer enter and leave events with a gesture | [16](16-swiftui-to-the-kit.md#constructs) |
| `modifiers::ViewModifiers::on_tap_gesture` | method | `onTapGesture(count:perform:)` | — | ≈ `UITapGestureRecognizer.numberOfTapsRequired` | Runs the closure after `count` taps: `.gesture(TapGesture::new().count(count).on_ended(..))` | [16](16-swiftui-to-the-kit.md#constructs) |
| `views::navigation_stack::NavigationPath` | struct | `NavigationPath` | — | ≈ `UINavigationController.viewControllers` | The routes pushed over the root of a stack: `push`, `pop`, `pop_to_root`, `truncate`, `len` | [11](11-views.md#navigationstack) |
| `views::navigation_stack::NavigationStack` | struct | `NavigationStack` | — | ≈ `UINavigationController` | The root and one page per route, with the measured push and pop | [11](11-views.md#navigationstack) |
| `views::navigation_stack::NavigationLink` | struct | `NavigationLink(value:)` | — | ≈ `UINavigationController.pushViewController(_:animated:)` | A button that pushes a route onto the path of the stack above | [11](11-views.md#navigationstack) |
| `views::navigation_stack::use_navigation_path` | fn | ≈ `NavigationPath` | — | ≈ `UIViewController.navigationController` | The path of the stack above a page | [11](11-views.md#navigationstack) |
| `ctdesign::NAVIGATION_PARALLAX`, `NAVIGATION_DIM` | constant | — | — | ≈ `UINavigationController` push | How far the page below leaves (0.3 of the width) and how dark it gets (0.13) | [4](04-appearance-and-tokens.md#motion) |
| `views::navigation::GO_BACK`, `views::navigation::GO_FORWARD` | constant | — | — | — | The Back (Alt+←) and Forward (Alt+→) actions | [11](11-views.md#go_back-and-go_forward) |
| `views::search::SearchIndex` | struct | — | — | — | Entries searched by title and keywords, ignoring case and accents | [11](11-views.md#searchindex) |
| `views::search::SearchIndex::add`, `search` | method | — | — | — | Adds an entry with a key, a title and keywords; the entries that match a query | [11](11-views.md#searchindex) |
| `views::search::Entry` | struct | — | — | — | One search entry: key, title and keywords | [11](11-views.md#entry) |
| `views::search::fold` | fn | — | — | — | Lowercases text and removes accents for matching | [11](11-views.md#search-helpers) |
| `views::search::word_start` | fn | — | — | — | Whether the query starts a word other than the first | [11](11-views.md#search-helpers) |

## Patterns

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `patterns::settings::SettingsWindow` | struct | ≈ `Settings` | — | ≈ `UIApplicationDelegate` | Settings window with pane sidebar, search, history and saved state | [12](12-patterns.md#settingswindow) |
| `patterns::settings::Pane` | struct | — | — | — | One preferences pane: id, title, symbol, tint and form | [12](12-patterns.md#pane) |
| `patterns::settings::Pane::page` | method | ≈ `navigationDestination` | — | ≈ `UINavigationController.pushViewController(_:animated:)` | A page without a sidebar item, opened by `Row::page` | [12](12-patterns.md#pane) |
| `patterns::settings::PaneGroup` | struct | — | — | — | Panes under an optional sidebar section title | [12](12-patterns.md#panegroup) |
| `patterns::settings::index` | fn | — | — | — | Builds the search index over panes and their rows | [12](12-patterns.md#index) |
| `patterns::about::AboutView` | struct | — | ≈ `NSApplication.orderFrontStandardAboutPanel(_:)` | — | Content of an About window: icon, name, copyright, optional version | [12](12-patterns.md#aboutview) |
| `patterns::language::LanguagePicker` | struct | — | — | — | Pop-up that writes the app or system language to a domain file | [12](12-patterns.md#languagepicker) |
| `patterns::language::app_language_row` | fn | — | — | — | A "Language" form row with the app's picker | [12](12-patterns.md#app_language_row-and-system_language_row) |
| `patterns::language::system_language_row` | fn | — | — | — | A "Language" form row with the system-wide picker | [12](12-patterns.md#app_language_row-and-system_language_row) |
| `patterns::language::write_language` | fn | — | — | — | Sets or removes the `language` key in a TOML file | [12](12-patterns.md#write_language) |

## App and scenes

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `app::Application` | struct | `App` | `NSApplication` | `UIApplication` | The app; its reverse-DNS id (≈ `CFBundleIdentifier`) is the `app_id`, the settings folder and the saved window size. `.window`, `.scene`, `.options`, `.commands`, `.run` | [2](02-first-app.md#what-application-does-for-you) |
| `app::Scene` | struct | `Scene` | — | `UIScene` | A declared scene, what `Application::scene` takes; `WindowGroup`, `Window` and `Settings` convert into it | [8](08-windows.md#scenes) |
| `app::WindowGroup` | struct | `WindowGroup` | — | `UIWindowScene` | Windows that share their content, one per `open`; `for_value` gives one per value | [8](08-windows.md#scenes) |
| `app::Window` | struct | `Window` | `NSWindow` | `UIWindow` | One window with an id; opening it again fronts it | [8](08-windows.md#scenes) |
| `app::Settings` | struct | `Settings` | `NSWindowController` | ≈ `UIViewController` | The single settings window, id `SETTINGS_WINDOW` | [8](08-windows.md#scenes) |
| `app::MAIN_WINDOW`, `app::SETTINGS_WINDOW`, `app::ABOUT_WINDOW` | constant | — | — | — | The ids of the window `Application::window` declares, of the settings scene and of the About window | [8](08-windows.md#scenes) |
| `app::OpenWindowAction` | struct | `OpenWindowAction` | — | `UIApplication.requestSceneSessionActivation(_:userActivity:options:errorHandler:)` | `open(id)` and `open_value(value)` | [8](08-windows.md#scenes) |
| `app::use_open_window` | fn | `EnvironmentValues.openWindow` | — | `UIApplication.requestSceneSessionActivation(_:userActivity:options:errorHandler:)` | Hook that returns the `OpenWindowAction` | [8](08-windows.md#scenes) |
| `app::DismissWindowAction` | struct | `DismissWindowAction` | `NSWindow.performClose(_:)` | `UIApplication.requestSceneSessionDestruction(_:options:errorHandler:)` | `dismiss()` and `dismiss_id(id)` | [8](08-windows.md#scenes) |
| `app::use_dismiss_window` | fn | `EnvironmentValues.dismissWindow` | — | `UIApplication.requestSceneSessionDestruction(_:options:errorHandler:)` | Hook that returns the `DismissWindowAction` | [8](08-windows.md#scenes) |
| `app::Commands` | struct | `Commands`, `TextEditingCommands` | `NSApplication.mainMenu` | `UIMenuBuilder`, `UIMenu.Identifier.edit` | The standard menus of the app; `.menu`, `.group`, `.text_editing` | [13](13-app.md#standard-menus) |
| `app::CommandGroup` | struct | `CommandGroup` | — | `UIMenuBuilder` | Items placed before, after or in place of a standard group | [13](13-app.md#standard-menus) |
| `app::CommandGroupPlacement` | enum | `CommandGroupPlacement` | — | `UIMenu.Identifier` | The standard groups: `AppInfo`, `AppSettings`, `AppTermination`, `NewItem`, `SaveItem`, `UndoRedo`, `Pasteboard`, `TextEditing`, `WindowSize`, `SingleWindowList`, `WindowArrangement`, `WindowList`, `Help` | [13](13-app.md#standard-menus) |

## Actions and shortcuts

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `app::Action` | struct | — | ≈ `NSMenuItem` | ≈ `UIAction` | A command: id, translated title and shortcut, shared by menus and keys | [13](13-app.md#actions-and-shortcuts) |
| `app::Shortcut` | struct | `KeyboardShortcut` | ≈ `NSMenuItem.keyEquivalent` | `UIKeyCommand` | Modifiers plus one key; gives the menu text and matches presses | [13](13-app.md#actions-and-shortcuts) |
| `app::Shortcut::DEFAULT_ACTION` | const | `KeyboardShortcut.defaultAction` | `NSButton.keyEquivalent` = Return | `UIKeyCommand` | Enter with no modifier | [13](13-app.md#actions-and-shortcuts) |
| `app::Shortcut::CANCEL_ACTION` | const | `KeyboardShortcut.cancelAction` | `NSButton.keyEquivalent` = Escape | `UIKeyCommand` | Escape with no modifier | [13](13-app.md#actions-and-shortcuts) |
| `app::ShortcutKey` | enum | `KeyEquivalent` | — | `UIKeyCommand.input` | The key of a shortcut: `Char`, `Left`, `Right`, `Named` | [13](13-app.md#actions-and-shortcuts) |
| `app::use_menu_action` | fn | — | ≈ `NSApplication.sendAction(_:to:from:)` | — | Hook that receives every action id from menus, shortcuts and activation | [13](13-app.md#actions-and-shortcuts) |
| `app::UNDO` | constant | `CommandGroupPlacement.undoRedo` | — | `UIMenu.Identifier.undoRedo` | The kit's Undo action, Ctrl+Z | [13](13-app.md#undo) |
| `app::REDO` | constant | `CommandGroupPlacement.undoRedo` | — | `UIMenu.Identifier.undoRedo` | The kit's Redo action, Ctrl+Shift+Z | [13](13-app.md#undo) |
| `app::ABOUT` | constant | `CommandGroupPlacement.appInfo` | `NSApplication.orderFrontStandardAboutPanel(_:)` | — | The About item of the app menu; the kit handles it | [13](13-app.md#standard-menus) |
| `app::NEW_WINDOW`, `app::CLOSE_WINDOW`, `app::SETTINGS`, `app::QUIT` | constant | `CommandGroupPlacement.newItem`, `.appSettings`, `.appTermination` | `NSApplication.terminate(_:)` for `QUIT` | `UIMenu.Identifier.newScene` | The actions of the File and app menus, Ctrl+N, W, comma and Q; the kit handles them | [13](13-app.md#standard-menus) |
| `app::TOGGLE_FULL_SCREEN` | constant | View ▸ Enter Full Screen | `NSWindow.toggleFullScreen(_:)` | — | The standard View menu action; the kit toggles full screen | [13](13-app.md#standard-menus) |
| `app::MINIMIZE`, `app::ZOOM`, `app::BRING_ALL_TO_FRONT` | constant | `CommandGroupPlacement.windowSize`, `.windowArrangement` | `NSWindow.performMiniaturize(_:)`, `performZoom(_:)`, `NSApplication.arrangeInFront(_:)` | `UIMenu.Identifier.window` | The actions of the Window menu; the kit handles them | [13](13-app.md#standard-menus) |
| `app::CUT`, `app::COPY`, `app::PASTE`, `app::DELETE`, `app::SELECT_ALL` | constant | `CommandGroupPlacement.pasteboard` | `NSText.cut(_:)` and the rest | `UIMenu.Identifier.edit` | The Edit menu actions; the app handles them | [13](13-app.md#standard-menus) |
| `app::FIND`, `app::FIND_AND_REPLACE`, `app::FIND_NEXT`, `app::FIND_PREVIOUS`, `app::USE_SELECTION_FOR_FIND`, `app::JUMP_TO_SELECTION` | constant | `TextEditingCommands` | — | `UIMenu.Identifier.edit` | The items of Edit ▸ Find; the app handles them | [13](13-app.md#standard-menus) |
| `app::HELP` | constant | `CommandGroupPlacement.help` | `NSApplication.showHelp(_:)` | `UIMenu.Identifier.help` | The Help menu item, F1; the app handles it | [13](13-app.md#standard-menus) |

## Menu bar menu

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `main_menu::MainMenu` | struct | — | `NSApplication.mainMenu` | — | The window's menu bar menu, published over dbusmenu | [13](13-app.md#menu-bar-menu) |
| `main_menu::Menu` | struct | `CommandMenu` | `NSMenu` | `UIMenu` | One top-level menu: a title and its items | [13](13-app.md#menu-bar-menu) |
| `main_menu::MenuHandle` | struct | — | ≈ `NSMenu` | ≈ `UIMenu` | Live handle to the window's menu: `update`, `set_menu_items` | [13](13-app.md#changing-the-menu-live) |
| `main_menu::MenuHandle::update` | method | — | `NSMenuItem` properties | `UIMenuBuilder.replace(menu:with:)` | Changes the item with this id and republishes the menu | [13](13-app.md#changing-the-menu-live) |
| `main_menu::MenuHandle::set_menu_items` | method | — | `NSMenu.items` | `UIMenuBuilder.replaceChildren(ofMenu:from:)` | Replaces the items of the top menu with this title | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_enabled` | fn | `.disabled(_:)` | `NSMenuItemValidation` | `UIControl.isEnabled` | Hook that enables or disables a menu item when the value changes | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_label` | fn | — | `NSMenuItem.title` | — | Hook that changes a menu item's label | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_checked` | fn | — | `NSMenuItem.state` | — | Hook that sets a menu item's checkbox | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_submenu_items` | fn | — | `NSMenuItem.submenu` | — | Hook that replaces a submenu's children | [13](13-app.md#changing-the-menu-live) |
| `main_menu::use_menu_update` | fn | — | — | — | Hook that applies any change to a menu item when a value changes | [13](13-app.md#changing-the-menu-live) |
| `main_menu::find_menu_shortcut` | fn | — | `NSMenu.performKeyEquivalent(with:)` | — | Finds the enabled action whose shortcut matches a key press | [13](13-app.md#changing-the-menu-live) |
| `extension::DbusMenuLayout` | type | — | — | — | dbusmenu `GetLayout` reply: revision and root item | [13](13-app.md#menu-bar-menu) |
| `extension::LayoutItem` | type | — | — | — | dbusmenu layout node: id, properties, children | [13](13-app.md#menu-bar-menu) |

## Undo

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `app::UndoManager` | struct | `EnvironmentValues.undoManager` | `UndoManager` | `UndoManager` | Undo and redo stacks for one `State`; `set`, `undo`, `redo` | [13](13-app.md#undo) |
| `app::use_undo` | fn | `EnvironmentValues.undoManager` | `UndoManager` | `UndoManager` | Hook that wraps a `State` and handles `UNDO` and `REDO` | [13](13-app.md#undo) |

## Settings and restoration

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `app::use_app_storage` | fn | `@AppStorage` | `UserDefaults` | `UserDefaults` | Hook that gives one key of `<app_id>.domain` as a `State` and saves it when it changes | [13](13-app.md#settings-and-restoration) |
| `app::load_defaults` | fn | — | `UserDefaults` | `UserDefaults` | Reads the app's settings from `<app_id>.domain` | [13](13-app.md#settings-and-restoration) |
| `app::save_defaults` | fn | — | `UserDefaults` | `UserDefaults` | Writes the app's settings to `<app_id>.domain` atomically | [13](13-app.md#settings-and-restoration) |
| `app::update_defaults` | fn | — | `UserDefaults` | `UserDefaults` | Loads the app's settings, changes them in a closure and saves them | [13](13-app.md#settings-and-restoration) |
| `app::use_defaults` | fn | — | `NSUserDefaultsController` | — | Hook that reads the settings and reloads them when the file changes | [13](13-app.md#settings-and-restoration) |
| `app::use_published` | fn | ≈ `onReceive(_:perform:)` | — | — | Hook that shows in a `State` the last value another thread sent | [13](13-app.md#updates-from-another-thread) |
| `app::Publisher` | struct | ≈ `PassthroughSubject` | — | — | The handle `use_published` gives to the thread; `send` hands a value over | [13](13-app.md#updates-from-another-thread) |
| `app::use_wake` | fn | — | ≈ `NSView.needsDisplay` | — | Hook that renders the component again each time a `WakeHandle` wakes | [13](13-app.md#updates-from-another-thread) |
| `app::use_watched` | fn | — | — | — | Hook that reruns a loader when any `.domain` file changes | [13](13-app.md#settings-and-restoration) |
| `app::load_restoration` | fn | `SceneStorage` | ≈ `NSWindowRestoration` | ≈ `UIStateRestoring` | Reads the app's saved state from `<app_id>.state` | [13](13-app.md#settings-and-restoration) |
| `app::save_restoration` | fn | `SceneStorage` | ≈ `NSWindowRestoration` | ≈ `UIStateRestoring` | Writes the app's state to `<app_id>.state` | [13](13-app.md#settings-and-restoration) |
| `app::read_toml` | fn | — | — | — | Reads a TOML file into `T`; default on missing or invalid | [13](13-app.md#settings-and-restoration) |
| `app::write_toml` | fn | — | — | — | Writes `T` as TOML through a temporary file and rename | [13](13-app.md#settings-and-restoration) |

## Single instance

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `app::launch` | fn | — | — | — | Owns the app's bus name or forwards activation to the running instance | [13](13-app.md#single-instance) |
| `app::Launch` | enum | — | — | — | Result of `launch`: `Primary(Instance)` or `Forwarded` | [13](13-app.md#single-instance) |
| `app::Instance` | struct | — | ≈ `NSApplicationDelegate.applicationShouldHandleReopen(_:hasVisibleWindows:)` | — | The primary instance; `listen` presents the window and runs forwarded actions | [13](13-app.md#single-instance) |

## Desktop services

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `status_item::StatusItem` | struct | `MenuBarExtra` | `NSStatusItem` | — | Status area icon with a menu, over StatusNotifierItem | [13](13-app.md#status-item) |
| `status_item::STATUS_ITEM_ACTIVATE` | constant | — | — | — | Action id sent when the status icon is clicked | [13](13-app.md#status-item) |
| `extension::Pixmaps` | type | — | — | — | StatusNotifierItem `IconPixmap` type: width, height, ARGB bytes | [13](13-app.md#status-item) |
| `notifications::Notification` | struct | — | ≈ `UNMutableNotificationContent` | ≈ `UNMutableNotificationContent` | A notification's app name, desktop entry, summary, body and action | [13](13-app.md#notifications) |
| `notifications::post` | fn | — | ≈ `UNUserNotificationCenter` | ≈ `UNUserNotificationCenter` | Posts a notification; the callback gets the activation token on click | [13](13-app.md#notifications) |
| `notifications::play_event_sound` | fn | — | ≈ `NSSound` | — | Plays a sound from the freedesktop sound theme | [13](13-app.md#notifications) |
| `secrets::Secrets` | struct | — | ≈ `SecItemCopyMatching(_:_:)` | ≈ `SecItemCopyMatching` | Session with the Secret Service keyring (the login Keychain on macOS): search, read, create, delete | [13](13-app.md#secrets) |
| `secrets::Item` | struct | — | — | — | A keyring item: object path, label and attributes | [13](13-app.md#secrets) |
| `secrets::Error` | enum | — | — | — | Keyring error: `Locked` (prompt showing), `Bus`, or `Keychain` (macOS `OSStatus`) | [13](13-app.md#secrets) |
| `portal::open_files` | fn | `.fileImporter(isPresented:allowedContentTypes:allowsMultipleSelection:onCompletion:)` | `NSOpenPanel` | `UIDocumentPickerViewController` | Async file or folder chooser through the portal | [13](13-app.md#portals) |
| `portal::save_file` | fn | `.fileExporter(isPresented:document:contentType:defaultFilename:onCompletion:)` | `NSSavePanel` | `UIDocumentPickerViewController` | Async save chooser through the portal | [13](13-app.md#portals) |
| `portal::open_uri` | fn | `OpenURLAction` | `NSWorkspace.open(_:)` | `UIApplication.open(_:options:completionHandler:)` | Opens a URI in its default app through the portal | [13](13-app.md#portals) |
| `portal::notify` | fn | — | ≈ `UNUserNotificationCenter` | ≈ `UNUserNotificationCenter` | Sends a notification through the notification portal | [13](13-app.md#portals) |
| `portal::file_uri` | fn | — | ≈ `URL.init(fileURLWithPath:)` | — | `ctappearance::file_uri`: a path as a percent-encoded `file://` URI | [13](13-app.md#portals) |
| `portal::file_path` | fn | — | — | — | `ctappearance::file_path`: a `file://` URI back into a path | [13](13-app.md#portals) |

## Animation

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `animation::Spring` | struct | `Spring` | ≈ `CASpringAnimation` | ≈ `CASpringAnimation` | A damped spring by perceptual duration and bounce, with initial velocity and settling duration; a Freya animated value | [14](14-animations.md#spring) |
| `animation::with_animation` | fn | `withAnimation(_:_:)` | ≈ `NSAnimationContext.runAnimationGroup(_:)` | ≈ `UIView.animate(withDuration:animations:)` | Tags the changes of its closure with an `Animation`; the outermost call of a turn wins; `Animation::None` is `withAnimation(nil)` | [14](14-animations.md#implicit-animation) |
| `animation::use_animated` | fn | ≈ `View.animation(_:value:)` | ≈ `NSAnimationContext` | ≈ `UIViewPropertyAnimator` | Hook returning a value that follows its target with the animation of a transaction or its default; a change midway keeps the velocity | [14](14-animations.md#implicit-animation) |
| `transition::Transition` | struct | `AnyTransition` | — | — | How a view looks when it is gone, on the way in and on the way out; `OPACITY`, `SCALE` and `IDENTITY`, built with `scale`, `offset`, `move_edge`, `combined` and `asymmetric` | [14](14-animations.md#transitions) |
| `transition::Transition::combined` | method | `AnyTransition.combined(with:)` | — | — | Both transitions at once | [14](14-animations.md#transitions) |
| `transition::Transition::move_edge` | fn | `.move(edge:)` | — | — | Starts or ends one of the view's own widths or heights beyond an edge | [14](14-animations.md#transitions) |
| `transition::Presence` | struct | ≈ `if` with `.transition(_:)` | — | — | Holds a view that comes and goes: `Some` shows it, `None` removes it after its transition; `.transition(t)` | [14](14-animations.md#transitions) |
| `transition::Presence::transition` | method | `.transition(_:)` | — | — | How the view comes and goes | [14](14-animations.md#transitions) |
| `transition::ContentTransition` | enum | `ContentTransition` | — | ≈ `UIView.transition(with:)` | How a changed view replaces the old one: `Identity` or `Opacity` | [14](14-animations.md#transitions) |
| `modifiers::StyleModifiers::content_transition` | method | `.contentTransition(_:)` | — | — | Crossfades the view when it changes inside `with_animation` | [14](14-animations.md#transitions) |
| `animation::use_reduced_motion` | fn | `EnvironmentValues.accessibilityReduceMotion` | `NSWorkspace.accessibilityDisplayShouldReduceMotion` | `UIAccessibility.isReduceMotionEnabled` | Hook returning the live reduced motion preference | [14](14-animations.md#reduced-motion) |
| `animation::motion_ms` | fn | — | — | — | A duration, or 0 when reduced motion is on | [14](14-animations.md#reduced-motion) |
| `animation::use_switch_progress` | fn | — | — | — | Hook easing 0–1 after a boolean state with an `Animation`; jumps with reduced motion | [14](14-animations.md#helpers) |
| `animation::use_alive` | fn | — | — | — | Hook flag that turns false when the component unmounts | [14](14-animations.md#helpers) |

## Backdrops and media

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `backdrop::Backdrop` | struct | — | — | — | Color backdrop from an image that gives its content a matching palette | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::BackdropSource` | enum | — | — | — | Where the backdrop's colors come from: `None`, `Colors`, `Image` | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::Accent` | enum | `.tint(_:)` | ≈ `NSColor.controlAccentColor` | `UIView.tintColor` | Accent inside a backdrop: `System`, `Own`, `FromBackdrop` | [15](15-backdrops-and-media.md#backdrop) |
| `backdrop::Tint` | enum | `.tint(_:)` | — | `UIView.tintColor` | A control's fill: `Accent`, `Color`, `Backdrop` gradient | [15](15-backdrops-and-media.md#tint) |
| `group::CompositingGroup` | struct | `compositingGroup()` | ≈ `NSView.canDrawSubviewsIntoLayer` | ≈ `CALayer.shouldRasterize` | Draws its children into one layer and applies a `GroupEffect` to it | [15](15-backdrops-and-media.md#compositing-groups) |
| `group::GroupEffect` | enum | ≈ `.opacity(_:)`, `.blur(radius:)`, `.colorEffect(_:)`, `.scrollEdgeEffectStyle(_:for:)` | ≈ `CALayer.filters` | ≈ `CALayer.filters` | What a group does to its layer: `Opacity`, `Blur`, `ColorMatrix`, `Shader`, `ScrollEdge` | [15](15-backdrops-and-media.md#compositing-groups) |
| `group::ShaderEffect` | struct | `Shader`, `.layerEffect(_:maxSampleOffset:)` | ≈ `CIKernel` | ≈ `CIKernel` | An SkSL image filter and its uniforms; a uniform named `time` follows the clock | [15](15-backdrops-and-media.md#compositing-groups) |
| `media::MediaButton` | struct | — | — | — | Large light pill button with an optional symbol, for media views | [15](15-backdrops-and-media.md#media) |
| `media::RoundButton` | struct | — | — | — | Round glass button with a symbol; filled light when selected | [15](15-backdrops-and-media.md#media) |
| `media::ScrimEdge` | enum | — | — | — | The solid edge of a scrim: `Leading` or `Bottom` | [15](15-backdrops-and-media.md#media) |
| `media::Scrim` | struct | — | — | — | Gradient that blends an image into the ground color | [15](15-backdrops-and-media.md#media) |
| `media::ground` | fn | — | — | — | The color under the content: backdrop average or window background | [15](15-backdrops-and-media.md#media) |
| `media::Material` | struct | `Material` | `NSVisualEffectView` | `UIVisualEffectView` | Blurred, tinted layer with a rim that fills its parent | [15](15-backdrops-and-media.md#media) |
| `media::TopGlass` | struct | ≈ `.scrollEdgeEffectStyle(_:for:)` | — | ≈ `UIScrollEdgeEffect.Style` | Progressive glass band along the top edge | [15](15-backdrops-and-media.md#media) |
| `media::use_artwork` | fn | ≈ `AsyncImage` | — | — | Hook that loads and decodes an image file in the background | [15](15-backdrops-and-media.md#media) |
| `media::use_artwork_from` | fn | ≈ `AsyncImage` | — | — | Hook that decodes image bytes from any loader in the background | [15](15-backdrops-and-media.md#media) |
| `glass_container::GlassContainer` | struct | `GlassEffectContainer` | `NSGlassEffectContainerView` | `UIGlassContainerEffect` | Row of glass shapes that merge when close and morph on change | [15](15-backdrops-and-media.md#glass-that-merges) |
| `WindowGlass` | struct | ≈ `.glassEffect(_:in:)` | ≈ `NSGlassEffectView` | ≈ `UIGlassEffect` | Draws its content over a window that is one glass panel; solid `surfaces.window` without compositor glass | [10](10-sidebar.md#glass) |
| `glass_container::Frame` | struct | — | — | — | A glass shape's rectangle and corner radius | [15](15-backdrops-and-media.md#glass-that-merges) |
| `glass_container::Glass` | struct | `Glass` | `NSGlassEffectView.Style` | ≈ `UIGlassEffect.Style` | The variant of a glass: `REGULAR` or `CLEAR` | [15](15-backdrops-and-media.md#glass-on-any-view) |
| `modifiers::StyleModifiers::glass_effect` | method | `.glassEffect(_:in:)` | `NSGlassEffectView` | `UIGlassEffect` | Puts glass in a shape behind the view: a `GlassContainer` of one item | [15](15-backdrops-and-media.md#glass-on-any-view) |

## Languages

| BarrierKit | Kind | SwiftUI | AppKit | UIKit | What it is | Chapter |
|---|---|---|---|---|---|---|
| `localize!` | macro | — | — | — | Creates an app's embedded translations, its `LOADER` and its `tr!` | [17](17-languages.md#localize) |
| `tr!` | macro | ≈ `LocalizedStringKey` | — | — | Returns the translated text for a Fluent id; not a key type | [17](17-languages.md#tr) |
| `i18n::load` | fn | — | — | — | Selects the requested languages into a Fluent loader | [17](17-languages.md#i18nload) |
| `i18n::use_app_language` | fn | — | — | — | Reads the app's chosen language from its `.domain` file, once | [17](17-languages.md#i18nuse_app_language) |
| `i18n::intern` | fn | — | — | — | Keeps an identifier built at run time as `&'static str`, once per distinct text | [17](17-languages.md#i18nintern) |

## Freya names you will meet

| Freya | SwiftUI | What it is |
|---|---|---|
| `App` | `App` | Trait of a root type with `fn render`; why the kit's app is `Application` |
| `Component` | `View` | A reusable view; it must be `PartialEq` |
| `IntoElement` | `some View` | Anything a view can return |
| `use_state` | `@State` | State that survives between renders |
| `rect()` | — | The basic box; `VStack` and `HStack` are built on it |
| `padding`, `margin`, `width`, `height`, `background`, `border`, `corner_radius`, `opacity` | `.padding`, `.frame`, `.background`, `.border`, `.cornerRadius`, `.opacity` | Set on the view's own node, not wrapped; every control takes them ([5](05-controls.md#modifiers-on-controls)) |
| `label()` | — | A single run of text; `Text` is built on it |
| `Label` | — | The type `label()` returns. The prelude exports the kit's [`Label`](02-first-app.md#text-and-stacks) in its place; name Freya's as `freya_core::prelude::Label` |
| `paragraph()` | — | Text with several styled spans |
