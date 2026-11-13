use std::path::PathBuf;
use std::{env, fs};

use serde_json::Value;

#[path = "build/tengen.rs"]
mod tengen;

use tengen::{Look, THEMES, Tengen, families, float, parse_color, rgba};

const SOURCES: [&str; 3] = ["tengen/tokens.json", "tengen/components.json", "src/lib.rs"];

const CONSTANTS: &str = r"
TITLEBAR_CLIENT_HEIGHT u32 size.titlebar-h
PANEL_GLOW_DEPTH f32 size.panel-glow-depth
TITLEBAR_DOUBLE_CLICK DoubleClick components.window.titlebar.double-click
WINDOW_CORNER WindowCorner components.window.corner
WINDOW_RESIZE WindowResize components.window edge=resize-edge corner=resize-corner
WINDOW_CLIENT_RESIZE WindowResize components.window edge=client-resize-edge
    corner=client-resize-corner
DIALOG WindowStyle components.dialog corner_radius=radius superellipse_exponent=components.window.corner.exponent
DIALOG_PADDING f32 components.dialog.padding
DIALOG_ALERT_WIDTH f32 components.dialog.alert-width
BUTTON_DESTRUCTIVE_FILL f32 components.button.destructive-fill
DIALOG_SHEET_WIDTH u32 components.dialog.sheet-width
DIALOG_SHEET_HEIGHT u32 components.dialog.sheet-height
POPUP WindowStyle components.shell.menu.panel corner_radius=radius superellipse_exponent=components.window.corner.exponent
MENU_SEPARATOR_INK_MIX f32 components.shell.menu.separator.tint-ink-mix
LAYER_FACTOR_BG_SUNKEN f32 appearance.layer-factors.bg-sunken
LAYER_FACTOR_CONTROL f32 appearance.layer-factors.bg-control
LAYER_FACTOR_LINE f32 appearance.layer-factors.line
LAYER_FACTOR_SURFACE f32 appearance.layer-factors.bg-surface
LAYER_FACTOR_RAISED f32 appearance.layer-factors.bg-raised
SLIDER SliderMetrics components.slider
CHECKBOX CheckboxMetrics components.checkbox
SWITCH SwitchMetrics components.switch transition_ms=transition
SEGMENTED SegmentedMetrics components.segmented
FOCUS_RING FocusRingMetrics components.focus-ring
BUTTON ButtonMetrics components.button
TEXT_FIELD TextFieldMetrics components.text-field
PROGRESS ProgressMetrics components.progress
    indeterminate_ms=indeterminate.duration spinner_ms=spinner.duration
TOOLTIP TooltipMetrics components.tooltip delay_ms=delay
TITLEBAR_RAIL TitlebarRailMetrics components.window.titlebar.rail
FULLSCREEN_GLASS FullscreenGlassMetrics components.window.titlebar.fullscreen-glass
FULLSCREEN_BAND FullscreenGlassMetrics components.window.titlebar.fullscreen-band
FULLSCREEN_BAR FullscreenBarMetrics components.window.titlebar.fullscreen-bar
BACKDROP BackdropMetrics components.backdrop
BACKDROP_SCENES BackdropScenes components.backdrop-scenes
BREAKPOINT BreakpointMetrics components.breakpoint
SYMBOL SymbolMetrics components.symbol
SLIDER_FAT SliderFatMetrics components.slider.fat
SLIDER_KEY_STEP f32 components.slider.keyboard-step
ICON_BUTTON IconButtonMetrics components.icon-button
CHART ChartMetrics components.chart
STEPPER StepperMetrics components.stepper
COMBO_BOX ComboBoxMetrics components.combo-box
POPOVER PopoverMetrics components.popover
GROUP_BOX GroupBoxMetrics components.group-box
TOKEN_FIELD TokenFieldMetrics components.token-field
DATE_PICKER DatePickerMetrics components.date-picker
PATH_CONTROL PathControlMetrics components.path-control
DISCLOSURE DisclosureMetrics components.disclosure
FONT_PANEL FontPanelMetrics components.font-panel
SPELLING SpellingMetrics components.spelling
PRINT PrintMetrics components.print
DRAG DragMetrics components.drag
IMAGE_WELL ImageWellMetrics components.image-well
LEVEL_INDICATOR LevelIndicatorMetrics components.level-indicator
GAUGE GaugeMetrics components.gauge
TOOLBAR_SEARCH_WIDTH f32 components.toolbar-search-width
BADGE BadgeMetrics components.badge
MEDIA MediaMetrics components.media
MATERIAL MaterialMetrics components.material
SCROLL_BAR ScrollBarMetrics components.scroll-bar hide_delay_ms=hide-delay
LIST_GROUP ListGroupMetrics components.list-group
    separator_inset=separator.inset-left header_padding=section-header.padding
SPACE_1 f32 space.1
SPACE_2 f32 space.2
SPACE_3 f32 space.3
SPACE_4 f32 space.4
SPLIT_VIEW SplitViewMetrics components.split-view
SPLIT_PANES SplitPanesMetrics components.split-panes
FLOATING_PANE FloatingPaneMetrics components.floating-pane
TABLE TableMetrics components.table
COLLECTION CollectionMetrics components.collection
TOOLBAR_TABS ToolbarTabsMetrics components.toolbar-tabs
    padding=container.padding gap=container.gap radius=container.radius icon=tab.icon
    icon_gap=tab.gap font_size=tab.font.size font_weight=tab.font.weight
ACCENT_MIN_CONTRAST_ON_WINDOW f32 appearance.contrast.accent-on-window
ON_ACCENT_MIN_CONTRAST f32 appearance.contrast.on-accent
WALLPAPER_TINTING_GRID u32 appearance.wallpaper-tinting.grid
WALLPAPER_TINTING_SIDEBAR f32 appearance.wallpaper-tinting.sidebar
TEXT_SIZES [f32;6] appearance.accessibility.text-sizes
POINTER_SIZES [f32;7] appearance.accessibility.pointer-sizes
DISPLAY_CONTRAST [f32;7] appearance.accessibility.display-contrast
COLOR_FILTER_INTENSITY [f32;7] appearance.accessibility.color-filter-intensity
TRAFFIC_RIM_WIDTH f32 components.window.traffic-lights.rim.width
TRAFFIC_CLOSE_MINIMIZE_DARKEN f32
    components.window.traffic-lights.styles.accent.close-minimize-darken
TRAFFIC_ZOOM_LIGHTEN f32 components.window.traffic-lights.styles.accent.zoom-lighten
TRAFFIC_PRESSED_DARKEN f32 components.window.traffic-lights.pressed-darken
TRAFFIC_GLYPH_STANDARD Rgba components.window.traffic-lights.styles.standard.glyph
TRAFFIC_GLYPH_ON_DARK Rgba components.window.traffic-lights.styles.accent.glyph-on-dark
TRAFFIC_GLYPH_ON_LIGHT Rgba components.window.traffic-lights.styles.accent.glyph-on-light
TRAFFIC_GLYPHS TrafficGlyphs components.window.traffic-lights.glyphs box_size=box
    close=close.segments minimize=minimize.segments zoom=zoom.triangles plus=plus.segments
BODY Font font.styles.body
LARGE_TITLE Font font.styles.display
TITLE Font font.styles.title-1
TITLE_2 Font font.styles.title-2
HEADLINE Font font.styles.headline
CALLOUT Font font.styles.callout
CAPTION Font font.styles.caption
MONO Font font.styles.mono
TEXT_LINE_HEIGHT f32 font.line-height-scale
SHADOW ShadowStyle shadow.layers.light unfocused_blur_scale=shadow.unfocused.blur
    unfocused_offset_scale=shadow.unfocused.offset unfocused_opacity_scale=shadow.unfocused.opacity
SHADOW_DARK ShadowStyle shadow.layers.dark unfocused_blur_scale=shadow.unfocused.blur
    unfocused_offset_scale=shadow.unfocused.offset unfocused_opacity_scale=shadow.unfocused.opacity
KNOB_SHADOW ShadowLayer shadow.slint.knob y_offset=offset-y opacity==1.0
SIDEBAR SidebarMetrics components.window.sidebar width=size.sidebar-w inset=island.inset
    padding_x=island.padding-x radius=island.radius rim_width=island.rim-width
    inner_width=island.inner-width traffic_x=island.traffic-x
    traffic_y=island.traffic-y app_icon=item.app-icon chevron=section.chevron
GLASS GlassMetrics components.window.sidebar.glass
TOOLGROUP ToolgroupMetrics components.window.toolgroup
MENU_POPOVER MenuPopoverMetrics components.menu-popover
DURATION_BASE u64 duration.base
DURATION_FAST_MS u64 duration.fast
DURATION_SLOW_MS u64 duration.slow
THEME_FADE_WAIT_MS u64 duration.wait-max
SPRING_DAMPING f32 spring.damping
THEME_SHADOW_OPACITY f32 shadow.theme.opacity
MENUBAR_HEIGHT i32 components.shell.menubar.height
MENUBAR_HEIGHT_WIDE i32 components.shell.menubar.height-wide
MENUBAR_TINT_SPREAD f32 components.shell.menubar.tint-spread
MENUBAR_DARK_INK_ABOVE f32 components.shell.menubar.dark-ink-above
MENUBAR_OVER_LIGHT_INK Rgba components.shell.menubar.over-light.ink
MENUBAR_OVER_LIGHT_GLASS Rgba components.shell.menubar.over-light.glass
MENUBAR_OVER_LIGHT_SELECTION Rgba components.shell.menubar.over-light.selection
MENUBAR_OVER_DARK_INK Rgba components.shell.menubar.over-dark.ink
MENUBAR_OVER_DARK_GLASS Rgba components.shell.menubar.over-dark.glass
MENUBAR_OVER_DARK_SELECTION Rgba components.shell.menubar.over-dark.selection
MENUBAR_FONT_SIZE f32 components.shell.menubar.font-size
MENUBAR_FONT_SIZE_LARGE f32 components.shell.menubar.font-size-large
MENUBAR_APP_NAME_WEIGHT i32 components.shell.menubar.app-name.weight
MENUBAR_MARGIN i32 components.shell.menubar.margin
MENUBAR_MARGIN_RIGHT i32 components.shell.menubar.margin-right
MENUBAR_FADED_ALPHA f32 components.shell.menubar.faded-alpha
MENUBAR_TITLE_PADDING_X f32 components.shell.menubar.title-padding-x
MENUBAR_TEXT_LIFT f32 components.shell.menubar.text-lift
MENUBAR_TITLE_MAX_WIDTH f32 components.shell.menubar.title-max-width
MENUBAR_SYSTEM_MENU_MIN_WIDTH f32 components.shell.menubar.system-menu-min-width
MENUBAR_TITLE_BLEED f32 components.shell.menubar.title-open.bleed
MENUBAR_BATTERY_LOW f64 components.shell.menubar.battery.low
DOCK_TILESIZE i32 components.shell.dock.tilesize
DOCK_TILESIZE_MIN i32 components.shell.dock.tilesize-min
DOCK_TILESIZE_MAX i32 components.shell.dock.tilesize-max
DOCK_LARGESIZE i32 components.shell.dock.largesize
DOCK_EDGE_MARGIN f32 components.shell.dock.edge-margin
DOCK_SIDE_MARGIN f32 components.shell.dock.side-margin
DOCK_PADDING f32 components.shell.dock.padding
DOCK_PADDING_SCALE f32 components.shell.dock.padding-scale
DOCK_SEPARATOR f32 components.shell.dock.separator
DOCK_SEPARATOR_LENGTH f32 components.shell.dock.separator-length
DOCK_SEPARATOR_INK_MIX f32 components.shell.dock.separator-ink-mix
DOCK_RADIUS f32 components.shell.dock.radius
DOCK_MAGNIFY_RADIUS f32 components.shell.dock.magnify-radius
DOCK_MAGNIFY_MS u64 components.shell.dock.magnify-ms
DOCK_DOT f32 components.shell.dock.dot
DOCK_DOT_MIN f32 components.shell.dock.dot-min
DOCK_DOT_MAX f32 components.shell.dock.dot-max
DOCK_PROGRESS_HEIGHT f32 components.shell.dock.progress-height
DOCK_BOUNCE_MS u64 components.shell.dock.bounce-ms
DOCK_BOUNCE_HEIGHT f32 components.shell.dock.bounce-height
DOCK_LAUNCH_TIMEOUT_MS u64 components.shell.dock.launch-timeout-ms
DOCK_HIDE_MS u64 components.shell.dock.hide-ms
DOCK_MINIMIZE_MS u64 components.shell.dock.minimize-ms
DOCK_SLOT_MS u64 components.shell.dock.slot-ms
DOCK_MINIMIZE_HANDOFF_MS u64 components.shell.dock.minimize-handoff-ms
DOCK_REVEAL_STRIP i32 components.shell.dock.reveal-strip
DOCK_RECENTS usize components.shell.dock.recents
DOCK_ICON_BODY f32 components.shell.dock.icon-body
DOCK_FOLDER_CANVAS f32 components.shell.dock.folder-canvas
DOCK_RIM_DARK f32 components.shell.dock.rim-dark
DOCK_RIM_BRIGHT f32 components.shell.dock.rim-bright
DOCK_WINDOW_FRAME f32 components.shell.dock.window-frame
DOCK_WINDOW_ICON f32 components.shell.dock.window-icon
DOCK_WINDOW_BADGE f32 components.shell.dock.window-badge
DOCK_LABEL_GAP f32 components.shell.dock.label-gap
DOCK_LABEL_FONT_SIZE f32 components.shell.dock.label-font-size
DOCK_LABEL_PADDING_X f32 components.shell.dock.label-padding-x
DOCK_LABEL_PADDING_Y f32 components.shell.dock.label-padding-y
DOCK_LABEL_RADIUS f32 components.shell.dock.label-radius
ABOUT_WIDTH u32 components.shell.about.width
ABOUT_HEIGHT u32 components.shell.about.height
ABOUT_DRAWING_TOP f32 components.shell.about.drawing-top
ABOUT_DRAWING_WIDTH f32 components.shell.about.drawing-width
ABOUT_DRAWING_HEIGHT f32 components.shell.about.drawing-height
ABOUT_TITLE_GAP f32 components.shell.about.title-gap
ABOUT_TITLE_SIZE f32 components.shell.about.title-size
ABOUT_SPECS_GAP f32 components.shell.about.specs-gap
ABOUT_BUTTON_GAP f32 components.shell.about.button-gap
ABOUT_COLUMN_GAP f32 components.shell.about.column-gap
ABOUT_VALUE_WIDTH f32 components.shell.about.value-width
ABOUT_SPEC_ROW_GAP f32 components.shell.about.row-gap
ABOUT_FOOTER_BOTTOM f32 components.shell.about.footer-bottom
ABOUT_APP_WIDTH u32 components.shell.about-app.width
ABOUT_APP_HEIGHT u32 components.shell.about-app.height
ABOUT_APP_ICON f32 components.shell.about-app.icon
CONFIRM_WIDTH u32 components.shell.confirm.width
CONFIRM_HEIGHT u32 components.shell.confirm.height
CONFIRM_BUTTON_GAP f32 components.shell.confirm.button-gap
CONFIRM_ICON f32 components.shell.confirm.icon
CONFIRM_GLYPH f32 components.shell.confirm.glyph
FORCE_QUIT_WIDTH u32 components.shell.force-quit.width
FORCE_QUIT_HEIGHT u32 components.shell.force-quit.height
FORCE_QUIT_ROW_HEIGHT f32 components.shell.force-quit.row-height
FORCE_QUIT_ICON f32 components.shell.force-quit.icon
LOCK_CLOCK_FONT_SIZE f32 components.shell.lock.clock-font-size
LOCK_DATE_FONT_SIZE f32 components.shell.lock.date-font-size
LOCK_TOP_PERCENT f32 components.shell.lock.top-percent
LOCK_BOTTOM_MARGIN f32 components.shell.lock.bottom-margin
LOCK_SPACING f32 components.shell.lock.spacing
LOCK_AVATAR_SIZE f32 components.shell.lock.avatar-size
LOCK_KEYBOARD_WIDTH f32 components.shell.lock.keyboard-width
LOCK_FIELD_HEIGHT f32 components.shell.lock.field-height
LOCK_FIELD_WIDTH f32 components.shell.lock.field-width
LOCK_MASK_FONT_SIZE f32 components.shell.lock.mask-font-size
LOCK_INK Rgba components.shell.lock.ink
LOCK_FIELD_FILL Rgba components.shell.lock.field-fill
LOCK_AVATAR_FILL Rgba components.shell.lock.avatar-fill
LOCK_FIELD_ERROR Rgba components.shell.lock.field-error
LOCK_FIELD_INK Rgba components.shell.lock.field-ink
LOCK_FADE_MS u64 components.shell.lock.fade
LOCK_SETTLE_MS u64 components.shell.lock.settle
LOCK_RISE_MS u64 components.shell.lock.rise
LOCK_RISE_OFFSET f32 components.shell.lock.rise-offset
LOCK_LOGIN_MS u64 components.shell.lock.login
NOTIFICATION_WIDTH i32 components.shell.notifications.width
NOTIFICATION_RADIUS f32 components.shell.notifications.radius
NOTIFICATION_PADDING f32 components.shell.notifications.padding
NOTIFICATION_GAP f32 components.shell.notifications.gap
NOTIFICATION_MARGIN i32 components.shell.notifications.margin
NOTIFICATION_ICON_SIZE f32 components.shell.notifications.icon-size
NOTIFICATION_TEXT_GAP f32 components.shell.notifications.text-gap
NOTIFICATION_CLOSE_SIZE f32 components.shell.notifications.close-size
NOTIFICATION_STACK_PEEK f32 components.shell.notifications.stack-peek
NOTIFICATION_STACK_INSET f32 components.shell.notifications.stack-inset
NOTIFICATION_BODY_LINES usize components.shell.notifications.body-lines
NOTIFICATION_MAX_BANNERS usize components.shell.notifications.max-banners
NOTIFICATION_BANNER_MS u64 components.shell.notifications.banner-duration
NOTIFICATION_SLIDE_IN_MS u64 components.shell.notifications.slide-in-duration
NOTIFICATION_SLIDE_OUT_MS u64 components.shell.notifications.slide-out-duration
NOTIFICATION_SLIDE_BOUNCE f32 components.shell.notifications.slide-bounce
NOTIFICATION_EDGE f64 components.shell.notifications.edge
NOTIFICATION_SWIPE f64 components.shell.notifications.swipe
NOTIFICATION_WIDGET_SIZE f32 components.shell.notifications.widget-size
NOTIFICATION_WIDGET_RADIUS f32 components.shell.notifications.widget-radius
LIVE_ACTIVITY_WIDTH u32 components.shell.live-activity.width
LIVE_ACTIVITY_HEIGHT u32 components.shell.live-activity.height
LIVE_ACTIVITY_RADIUS f32 components.shell.live-activity.radius
LIVE_ACTIVITY_GAP i32 components.shell.live-activity.gap
LIVE_ACTIVITY_ENDED_MS u64 components.shell.live-activity.ended-duration
LAUNCHER_WIDTH i32 components.shell.launcher.width
LAUNCHER_HEIGHT i32 components.shell.launcher.height
LAUNCHER_PADDING i32 components.shell.launcher.padding
LAUNCHER_SEARCH_HEIGHT i32 components.shell.launcher.search-height
LAUNCHER_RESULT_HEIGHT i32 components.shell.launcher.result-height
LAUNCHER_RESULT_GAP i32 components.shell.launcher.result-gap
LAUNCHER_ICON_SIZE i32 components.shell.launcher.icon-size
LAUNCHER_MAX_RESULTS i32 components.shell.launcher.max-results
LAUNCHER_CORNER_RADIUS f32 components.shell.launcher.corner-radius
LAUNCHER_FONT_SIZE f32 components.shell.launcher.font-size
LAUNCHER_SUBTITLE_FONT_SIZE f32 components.shell.launcher.subtitle-font-size
LAUNCHER_SCALE_FROM f32 components.shell.launcher.scale-from
LAUNCHER_RESULT_RADIUS f32 components.shell.launcher.result-radius
LAUNCHER_OPEN_MS u64 components.shell.launcher.open-duration
LAUNCHER_CLOSE_MS u64 components.shell.launcher.close-duration
SETTINGS SettingsMetrics components.settings account_name_font=account.name
    account_detail_font=account.detail card_caption_font=card.caption
HAIRLINE f32 components.hairline
DOMINANT_MIN_CHROMA f32 appearance.dominant.min-chroma
CURSOR_SHAKE_SCALE f32 components.cursor-shake.scale
CURSOR_SHAKE_GROW_MS u64 components.cursor-shake.grow-ms
CURSOR_SHAKE_SHRINK_MS u64 components.cursor-shake.shrink-ms
CURSOR_SHAKE_TICK_MS u64 components.cursor-shake.tick-ms
CURSOR_SHAKE_MIN_DX f64 components.cursor-shake.min-dx
CURSOR_SHAKE_MIN_SWING f64 components.cursor-shake.min-swing
CURSOR_SHAKE_MAX_SWING_MS u64 components.cursor-shake.max-swing-ms
CURSOR_SHAKE_MAX_TOTAL_MS u64 components.cursor-shake.max-total-ms
CURSOR_SHAKE_REVERSALS u32 components.cursor-shake.reversals
CURSOR_SHAKE_ACTIVE_MS u64 components.cursor-shake.active-ms
BACKGROUND_BLUR_PASSES u32 components.background-blur.passes
BACKGROUND_BLUR_OFFSET f32 components.background-blur.offset
SCREENSHOT_BORDER_WIDTH i32 components.screenshot.border-width
SPACE_GAP i32 components.spaces.gap
SPACE_SLIDE_MS u64 components.spaces.slide-ms
TILE_MARGIN i32 components.tile.margin
TILE_GAP i32 components.tile.gap
TITLEBAR_PADDING_X f32 components.window.titlebar.padding-x
TITLEBAR_TITLE_GAP f32 components.window.titlebar.title-gap
MENU_MARK_TEXT_GAP f32 components.shell.menu.mark.text-gap
MENU_SUBMENU_INTENT_MS u64 components.shell.menu.submenu.intent-ms
MENU_SUBMENU_REVEAL_GAP f32 components.shell.menu.submenu.reveal-gap
MENU_SCROLL_TICK_MS u64 components.shell.menu.scroll-tick-ms
MENU_SEARCH_ICON_GAP f32 components.shell.menu.search.icon-gap
MENU_SEARCH_CLEAR f32 components.shell.menu.search.clear
MENU_SEARCH_CLEAR_GLYPH f32 components.shell.menu.search.clear-glyph
MENU_SEARCH_PADDING_X f32 components.shell.menu.search.padding-x
MENU_SEARCH_CLEAR_MIX f32 components.shell.menu.search.clear-mix
MENU_HELP_ICON f32 components.shell.menu.help-icon
GLASS_MASK Rgba components.glass-mask
HUD_BACKGROUND Rgba components.hud.background
SCREENSHOT_DIM Rgba components.screenshot.dim
SCREENSHOT_IDLE_DIM Rgba components.screenshot.idle-dim
SCREENSHOT_BORDER Rgba components.screenshot.border
SPACE_GAP_COLOR Rgba components.spaces.gap-color
FOCUS_BAR_BACKGROUND Rgba components.spaces.focus-bar
FOCUS_BAR_TEXT Rgba components.spaces.focus-bar-text
DESKTOP_CLEAR Rgba components.desktop-clear
WALLPAPER_FPS u32 components.wallpaper.fps
WALLPAPER_THUMBNAIL u32 components.wallpaper.thumbnail
WALLPAPER_FRAME u32 components.wallpaper.frame
SPACE_SLIDE_EASE [f32;4] components.spaces.slide-ease
DOMINANT_LIGHTNESS [f32;2] appearance.dominant.lightness
MENU_SEARCH_ICON [f32;2] components.shell.menu.search.icon
MENUBAR_BATTERY_ICON [f32;2] components.shell.menubar.battery.icon
";

const THEMED: &str = r"
BG_WINDOW Rgba themes.{theme}.bg-window
BG_CONTROL Rgba themes.{theme}.bg-control
BG_SURFACE Rgba themes.{theme}.bg-surface
BG_SUNKEN Rgba themes.{theme}.bg-sunken
BG_RAISED Rgba themes.{theme}.bg-raised
BG_HOVER Rgba themes.{theme}.bg-hover
SHELL ShellPalette components.shell bar=menubar.bg dock=dock.bg text=menubar.text tile=dock.tile
    tile_active=dock.tile-active tile_text_active=dock.tile-text-active
SIDEBAR Rgba themes.{theme}.bg-sunken
SIDEBAR_ISLAND Rgba themes.{theme}.sidebar-island-bg
SIDEBAR_GLASS_BASE Rgba themes.{theme}.sidebar-glass-base
SIDEBAR_GLASS_RIM Rgba themes.{theme}.sidebar-glass-rim
SIDEBAR_ISLAND_RIM Rgba components.window.sidebar.island.rim
SIDEBAR_ISLAND_INNER Rgba components.window.sidebar.island.inner
TOOLGROUP_BG Rgba themes.{theme}.toolbar-glass-bg
SIDEBAR_PALETTE SidebarPalette components.window.sidebar.item item_ink=ink item_icon=icon-ink
    section_ink=components.window.sidebar.section.ink line=components.window.sidebar.flat.line
TOOLGROUP_PALETTE ToolgroupPalette components.window.toolgroup
MENU MenuPalette components.shell.menu bg=panel.bg padding=panel.padding shadow=panel.shadow
    min_width=panel.min-width max_width=panel.max-width font_size=item.font-size
    shortcut_gap=item.shortcut-gap fg=item.fg shortcut_color=item.shortcut-color
    hover_bg=item.hover-bg hover_fg=item.hover-fg disabled_fg=item.disabled-fg
    bar_hover_bg=components.shell.menubar.hover-bg
    bar_title_open_bg=components.shell.menubar.title-open.bg
    bar_title_open_radius=components.shell.menubar.title-open.radius
    bar_title_open_height=components.shell.menubar.title-open.height leading_width=mark.width
";

const OPAQUE: [&str; 3] = ["SHELL", "SIDEBAR", "TOOLGROUP_BG"];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let read = |source: &str| {
        let path = manifest.join(source);
        println!("cargo:rerun-if-changed={}", path.display());
        fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("ctdesign: cannot read {}: {error}", path.display()))
    };
    let json = |source: &str| match serde_json::from_str(&read(source)) {
        Ok(Value::Object(object)) => object,
        _ => panic!("ctdesign: {source} is not a JSON object"),
    };
    let mut root = json(SOURCES[0]);
    root.extend(json(SOURCES[1]));
    let tengen = Tengen {
        root: Value::Object(root),
        structs: read(SOURCES[2]),
    };

    let mut out = String::from(
        "// @generated by build.rs from tengen/tokens.json and tengen/components.json.\n\
         // Do not edit: change the JSON in crates/ctdesign/tengen and rebuild.\n\n",
    );
    let default = Look {
        theme: tengen.string("default-theme"),
        opaque: false,
    };
    for entry in entries(CONSTANTS) {
        out.push_str(&tengen.constant(entry[0], entry[1], entry[2], &entry[3..], default));
    }
    for entry in entries(THEMED) {
        for (suffix, theme) in THEMES {
            let name = format!("{}_{suffix}", entry[0]);
            let path = entry[2].replace("{theme}", theme);
            for opaque in [false, true] {
                if opaque && !OPAQUE.contains(&entry[0]) {
                    continue;
                }
                let name = if opaque {
                    format!("{name}_OPAQUE")
                } else {
                    name.clone()
                };
                let look = Look { theme, opaque };
                out.push_str(&tengen.constant(&name, entry[1], &path, &entry[3..], look));
            }
        }
    }
    emit_titlebar(&mut out, &tengen, default);
    emit_tints(&mut out, &tengen);
    emit_palettes(&mut out, &tengen);
    emit_specials(&mut out, &tengen);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("tengen.rs"), out).expect("write tengen.rs");
}

fn entries(table: &str) -> Vec<Vec<&str>> {
    let mut entries: Vec<Vec<&str>> = Vec::new();
    for line in table.lines().filter(|line| !line.is_empty()) {
        if !line.starts_with(' ') {
            entries.push(Vec::new());
        }
        entries
            .last_mut()
            .expect("ctdesign: table starts indented")
            .extend(line.split_whitespace());
    }
    entries
}

fn emit_titlebar(out: &mut String, tengen: &Tengen, look: Look) {
    let lights = "components.window.traffic-lights";
    let button = |kind: &str, key: &str| {
        let color = tengen.color(&format!("{lights}.styles.standard.{key}"), look);
        format!(
            "Button {{ kind: ButtonKind::{kind}, color: {} }}",
            rgba(color)
        )
    };
    let int = |path: &str| tengen.number(path) as i64;
    out.push_str(&format!(
        "pub const TITLEBAR: Titlebar = Titlebar {{ thickness: {}, buttons: TitlebarButtons {{ order: [{}, {}, {}], diameter: {}, spacing: {}, corner_radius: {} }}, font: {} }};\n\n",
        int("size.titlebar-ssd-h"),
        button("Close", "close"),
        button("Minimize", "minimize"),
        button("Maximize", "zoom"),
        int(&format!("{lights}.size")),
        int(&format!("{lights}.gap")),
        int(&format!("{lights}.radius")),
        tengen.font("font.styles.headline"),
    ));
    for (name, key) in [
        ("WINDOW_MIN_SIZE", "min"),
        ("WINDOW_DEFAULT_SIZE", "default"),
    ] {
        out.push_str(&format!(
            "pub const {name}: (u32, u32) = ({}, {});\n\n",
            int(&format!("components.window.{key}-width")),
            int(&format!("components.window.{key}-height")),
        ));
    }
}

fn emit_tints(out: &mut String, tengen: &Tengen) {
    let modes = "appearance.modes";
    for (name, path) in [
        ("WINDOW_TINT_ACCENT", format!("{modes}.accent.windows")),
        (
            "WALLPAPER_TINTING",
            "appearance.wallpaper-tinting".to_owned(),
        ),
    ] {
        let mode = |theme: &str| float(tengen.number(&format!("{path}.{theme}")));
        out.push_str(&format!(
            "pub const {name}: [f32; 2] = [{}, {}];\n\n",
            mode("light"),
            mode("dark")
        ));
    }
    let strength = ["subtle", "normal", "strong", "intense"]
        .map(|key| float(tengen.number(&format!("appearance.tint-strength.{key}"))));
    out.push_str(&format!(
        "pub const TINT_STRENGTH: [f32; 4] = [{}];\n\n",
        strength.join(", ")
    ));
}

fn emit_palettes(out: &mut String, tengen: &Tengen) {
    for (name, theme) in THEMES {
        let look = Look {
            theme,
            opaque: false,
        };
        let color = |key: &str| tengen.color(&format!("themes.{theme}.{key}"), look);
        let rgba_of = |key: &str| rgba(color(key));
        let pair = |focused: &str, unfocused: &str| {
            format!(
                "FocusPair {{ focused: {}, unfocused: {} }}",
                rgba_of(focused),
                rgba_of(unfocused)
            )
        };
        let inner_line = match color("window-inner") {
            inner if inner.a == 0 => "None".to_string(),
            inner => format!("Some({})", rgba(inner)),
        };
        let panel_inner_line = match (color("panel-inner"), color("panel-inner-edge")) {
            (sides, edges) if sides.a == 0 && edges.a == 0 => "None".to_string(),
            (sides, edges) => format!(
                "Some(PanelLine {{ sides: {}, edges: {}, glow: {} }})",
                rgba(sides),
                rgba(edges),
                rgba_of("panel-glow")
            ),
        };
        let lights = "components.window.traffic-lights";
        let traffic = |key: &str| rgba(tengen.color(&format!("{lights}.{key}"), look));
        out.push_str(&format!(
            "pub const {name}: Palette = Palette {{ titlebar_bg: {}, hairline: {}, inner_line: {inner_line}, panel_inner_line: {panel_inner_line}, outer_border: {}, title: {}, text: {}, ink_strong: {}, ink_muted: {}, ink_faint: {}, line: {}, line_strong: {}, accent: {}, accent_inactive: {}, on_accent: {}, danger: {}, traffic_lights_inactive: {}, traffic_rim: TrafficRim {{ lit: {}, idle: {} }}, scrim: {}, selected_control: {}, card_border: {} }};\n\n",
            pair("titlebar-bg", "titlebar-bg-inactive"),
            rgba_of("titlebar-line"),
            rgba_of("window-outer"),
            pair("titlebar-title", "titlebar-title-inactive"),
            pair("ink", "ink-muted"),
            rgba_of("ink-strong"),
            rgba_of("ink-muted"),
            rgba_of("ink-faint"),
            rgba_of("line"),
            rgba_of("line-strong"),
            rgba_of("accent"),
            rgba_of("accent-inactive"),
            rgba_of("on-accent"),
            rgba_of("danger"),
            traffic("inactive"),
            traffic("rim.color"),
            traffic("rim.inactive"),
            rgba_of("scrim"),
            rgba_of("bg-selected-control"),
            rgba_of("card-border"),
        ));
        let bg = rgba(tengen.color(
            "components.shell.menu.panel.bg",
            Look {
                theme,
                opaque: true,
            },
        ));
        out.push_str(&format!(
            "pub const MENU_{name}_OPAQUE: MenuPalette = MenuPalette {{ bg: {bg}, ..MENU_{name} }};\n\n"
        ));
    }
}

fn emit_specials(out: &mut String, tengen: &Tengen) {
    let families: Vec<String> = families(tengen.string("font.families.sans"))
        .map(|family| format!("{family:?}"))
        .collect();
    out.push_str(&format!(
        "pub const SANS_FAMILIES: [&str; {}] = [{}];\n\n",
        families.len(),
        families.join(", ")
    ));
    let swatches: Vec<String> = tengen
        .get("components.settings.accent-swatches")
        .as_array()
        .expect("ctdesign: accent-swatches must be a list")
        .iter()
        .map(|swatch| {
            let field = |key: &str| {
                swatch[key]
                    .as_str()
                    .expect("ctdesign: accent swatch name and color")
            };
            format!(
                "({:?}, {})",
                field("name"),
                rgba(parse_color(field("color")))
            )
        })
        .collect();
    out.push_str(&format!(
        "pub const ACCENT_SWATCHES: [(&str, Rgba); {}] = [{}];\n\n",
        swatches.len(),
        swatches.join(", ")
    ));
    let palette = tengen.get("components.settings.accent-swatches");
    for (constant, path) in [
        ("CHART_SERIES", "components.chart.series"),
        ("MULTICOLOR", "components.settings.multicolor"),
        (
            "LEVEL_INDICATOR_COLORS",
            "components.level-indicator.colors",
        ),
    ] {
        let colors: Vec<String> = tengen
            .get(path)
            .as_array()
            .unwrap_or_else(|| panic!("ctdesign: {path} must be a list of swatch names"))
            .iter()
            .map(|name| {
                let swatch = palette
                    .as_array()
                    .and_then(|list| list.iter().find(|swatch| swatch["name"] == *name))
                    .unwrap_or_else(|| panic!("ctdesign: {path} {name} is not a swatch"));
                rgba(parse_color(swatch["color"].as_str().unwrap_or_default()))
            })
            .collect();
        out.push_str(&format!(
            "pub const {constant}: [Rgba; {}] = [{}];\n\n",
            colors.len(),
            colors.join(", ")
        ));
    }
}
