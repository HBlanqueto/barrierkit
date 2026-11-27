use std::path::PathBuf;
use std::{env, fs};

use serde_json::Value;

#[path = "build/tengen.rs"]
mod tengen;

use tengen::{Look, Tengen, entries, families, float, parse_color, rgba};

const THEMES: [(&str, &str); 3] = [
    ("LIGHT", "light"),
    ("DARK", "dark"),
    ("DARK_TINT", "dark-tint"),
];

const CONSTANTS: &str = r"
TITLEBAR_CLIENT_HEIGHT u32 size.titlebar-h
PANEL_GLOW_DEPTH f32 size.panel-glow-depth
FRAME_LINE_WIDTH f32 size.frame-line-width
TITLEBAR_DOUBLE_CLICK DoubleClick components.window.titlebar.double-click
WINDOW_CORNER WindowCorner components.window.corner
CORNER_SMOOTHING f32 components.corner.smoothing
NAVIGATION_PARALLAX f32 components.navigation.parallax
NAVIGATION_DIM f32 components.navigation.dim
SCROLL_POCKET_BLUR f32 components.scroll-pocket.blur
SCROLL_POCKET_SCALE_LIGHT f32 components.scroll-pocket.scale-light
SCROLL_POCKET_OFFSET_LIGHT f32 components.scroll-pocket.offset-light
SCROLL_POCKET_SCALE_DARK f32 components.scroll-pocket.scale-dark
SCROLL_POCKET_OFFSET_DARK f32 components.scroll-pocket.offset-dark
CORNER_CONCENTRIC_REGION f32 components.corner.concentric-region
CORNER_WINDOW CornerShape components.corner.window
CORNER_SHEET CornerShape components.corner.sheet
CORNER_ALERT CornerShape components.corner.alert
CORNER_MENU CornerShape components.corner.menu
CORNER_TOOLTIP CornerShape components.corner.tooltip
CORNER_CONTROLS CornerControls components.corner.controls
WINDOW_RESIZE WindowResize components.window edge=resize-edge corner=resize-corner
DIALOG_PADDING f32 components.dialog.padding
DIALOG_ALERT_WIDTH f32 components.dialog.alert-width
DIALOG_ALERT_PADDING f32 components.dialog.alert-padding
DIALOG_ALERT_SPACING f32 components.dialog.alert-spacing
DIALOG_ALERT_LINE_HEIGHT u32 components.dialog.alert-line-height
DIALOG_ALERT_ICON f32 components.dialog.alert-icon
DIALOG_ALERT_ICON_GAP f32 components.dialog.alert-icon-gap
DIALOG_ALERT_STACK_SPACING f32 components.dialog.alert-stack-spacing
DIALOG_ALERT_HELP_SIZE f32 components.dialog.alert-help-size
DIALOG_ALERT_HELP_INSET f32 components.dialog.alert-help-inset
DIALOG_ALERT_BADGE f32 components.dialog.alert-badge
DIALOG_ALERT_CAUTION Rgba components.dialog.alert-caution
DIALOG_ALERT_CAUTION_SIZE f32 components.dialog.alert-caution-size
DIALOG_ALERT_CAUTION_AT [f32;2] components.dialog.alert-caution-at
DIALOG_BUTTON_WIDTH f32 components.dialog.button-width
BUTTON_DESTRUCTIVE_FILL f32 components.button.destructive-fill
DIALOG_SHEET_WIDTH u32 components.dialog.sheet-width
DIALOG_SHEET_HEIGHT u32 components.dialog.sheet-height
DIALOG_SOLID_LIGHT Rgba components.dialog.solid-light
DIALOG_SOLID_DARK Rgba components.dialog.solid-dark
DIALOG_SOLID_SCRIM_LIGHT Rgba components.dialog.solid-scrim-light
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
SEARCH_FIELD SearchFieldMetrics components.search-field
TEXT_EDITOR TextEditorMetrics components.text-editor
ACCENT_RULE AccentRule components.settings.accent-rule
PROGRESS ProgressMetrics components.progress
    indeterminate_ms=indeterminate.duration spinner_ms=spinner.duration spinner_frames=spinner.frames fill_ms=fill.duration fill_curve=fill.curve
TOOLTIP TooltipMetrics components.tooltip delay_ms=delay
FULLSCREEN_GLASS FullscreenGlassMetrics components.window.titlebar.fullscreen-glass
FULLSCREEN_BAR FullscreenBarMetrics components.window.titlebar.fullscreen-bar
BACKDROP BackdropMetrics components.backdrop
BACKDROP_SCENES BackdropScenes components.backdrop-scenes
SYMBOL SymbolMetrics components.symbol
SYMBOL_SCALE_SIZE [f32;3] components.symbol.scale-size
SYMBOL_WEIGHT_STROKE_SMALL [f32;9] components.symbol.weight-stroke-small
SYMBOL_WEIGHT_STROKE_MEDIUM [f32;9] components.symbol.weight-stroke-medium
SYMBOL_WEIGHT_STROKE_LARGE [f32;9] components.symbol.weight-stroke-large
SYMBOL_HIERARCHY [f32;3] components.symbol.hierarchy
ICON_STYLE IconStyleMetrics components.icon-style
ICON_PLATE IconPlateMetrics components.icon-plate
ICON_PLATE_LOBE_DARK IconPlateLook components.icon-plate.lobe-dark
ICON_PLATE_LOBE_LIGHT IconPlateLook components.icon-plate.lobe-light
ICON_PLATE_GRAPHITE IconPlateLook components.icon-plate.graphite
ICON_PLATE_SILVER IconPlateLook components.icon-plate.silver
ICON_PLATE_CLEAR IconPlateLook components.icon-plate.clear
ICON_PLATE_CLEAR_DARK IconPlateLook components.icon-plate.clear-dark
ICON_PLATE_TINTED IconPlateLook components.icon-plate.tinted
ICON_PLATE_TINTED_DARK IconPlateLook components.icon-plate.tinted-dark
LEVEL_RING_RADII [f32;4] components.level-indicator.ring-radii
LEVEL_BOTTOM [f32;4] components.level-indicator.bottom
ICON_LOGO_CLEAR IconLogoEnds components.icon-logo.clear
ICON_LOGO_CLEAR_DARK IconLogoEnds components.icon-logo.clear-dark
ICON_LOGO_TINTED IconLogoEnds components.icon-logo.tinted
ICON_LOGO_TINTED_DARK IconLogoEnds components.icon-logo.tinted-dark
SLIDER_FAT SliderFatMetrics components.slider.fat
SLIDER_KEY_STEP f32 components.slider.keyboard-step
ICON_BUTTON IconButtonMetrics components.icon-button
STEPPER StepperMetrics components.stepper
HELP_LINK HelpLinkMetrics components.help-link
COMBO_BOX ComboBoxMetrics components.combo-box
POPOVER PopoverMetrics components.popover
GROUP_BOX GroupBoxMetrics components.group-box
COLOR_PICKER ColorPickerMetrics components.color-picker
CIRCULAR_SLIDER CircularSliderMetrics components.circular-slider
TOKEN_FIELD TokenFieldMetrics components.token-field
DATE_PICKER DatePickerMetrics components.date-picker
PATH_CONTROL PathControlMetrics components.path-control
DISCLOSURE DisclosureMetrics components.disclosure
FONT_PANEL FontPanelMetrics components.font-panel
SPELLING SpellingMetrics components.spelling
DRAG DragMetrics components.drag
IMAGE_WELL ImageWellMetrics components.image-well
LEVEL_INDICATOR LevelIndicatorMetrics components.level-indicator
GAUGE GaugeMetrics components.gauge
TOOLBAR_SEARCH_WIDTH f32 components.toolbar-search-width
BADGE BadgeMetrics components.badge
MEDIA MediaMetrics components.media
MATERIAL MaterialMetrics components.material
SCROLL_EDGE ScrollEdgeMetrics components.scroll-edge
SCROLL_BAR ScrollBarMetrics components.scroll-bar hide_delay_ms=hide-delay fade_in_ms=fade-in fade_out_ms=fade-out
GESTURE GestureMetrics components.gesture tap_interval_ms=tap-interval long_press_ms=long-press-duration
SCROLL_PHYSICS ScrollPhysicsMetrics components.scroll-physics velocity_window_ms=velocity-window
SCROLL_FLING_POWER i32 components.scroll-physics.fling-power
SCROLL_SETTLED_OVER f32 components.scroll-physics.settled-over
SCROLL_SETTLED_SPEED f32 components.scroll-physics.settled-speed
LIST_SECTION ListSectionMetrics components.list-section
LIST_GROUP ListGroupMetrics components.list-group
    separator_inset=separator.inset-left header_padding=section-header.padding
SPACE_1 f32 space.1
SPACE_2 f32 space.2
SPACE_3 f32 space.3
SPACE_4 f32 space.4
SPLIT_VIEW SplitViewMetrics components.split-view
SPLIT_PANES SplitPanesMetrics components.split-panes
TABLE TableMetrics components.table
COLLECTION CollectionMetrics components.collection
TAB_VIEW TabViewMetrics components.tab-view
TOOLBAR_TABS ToolbarTabsMetrics components.toolbar-tabs
    padding=container.padding gap=container.gap radius=container.radius icon=tab.icon
    icon_gap=tab.gap font_size=tab.font.size font_weight=tab.font.weight
ACCENT_MIN_CONTRAST_ON_WINDOW f32 appearance.contrast.accent-on-window
ON_ACCENT_MIN_CONTRAST f32 appearance.contrast.on-accent
ACCENT_CHROMA_GAIN f32 appearance.modes.accent.chroma-gain
WALLPAPER_TINTING_SIDEBAR f32 appearance.wallpaper-tinting.sidebar
WALLPAPER_TINTING_SIDEBAR_LIFT f32 appearance.wallpaper-tinting.sidebar-lift
WALLPAPER_TINTING_TARGET f32 appearance.wallpaper-tinting.target
TEXT_SIZES [f32;6] appearance.accessibility.text-sizes
MENU_DARK_INK_ABOVE f32 components.shell.menu.dark-ink-above
TRAFFIC_RING_WIDTH f32 components.window.traffic-lights.ring-width
TRAFFIC_MARGIN f32 components.window.traffic-lights.margin
TRAFFIC_GLYPH_FADE_MS u64 components.window.traffic-lights.glyph-fade-ms
TRAFFIC_CLOSE_MINIMIZE_DARKEN f32
    components.window.traffic-lights.styles.accent.close-minimize-darken
TRAFFIC_ZOOM_LIGHTEN f32 components.window.traffic-lights.styles.accent.zoom-lighten
TRAFFIC_GLYPHS TrafficGlyphs components.window.traffic-lights.glyphs
    close=close.segments minimize=minimize.segments zoom=zoom.triangles
    zoom_exit=zoom-exit.triangles plus=plus.segments
TRAFFIC_PRESS TrafficPress components.window.traffic-lights.pressed
BODY Font font.styles.body
LARGE_TITLE Font font.styles.display
TITLE Font font.styles.title-1
TITLE_2 Font font.styles.title-2
HEADLINE Font font.styles.headline
CALLOUT Font font.styles.callout
CAPTION Font font.styles.caption
TITLE_3 Font font.styles.title-3
TOOLBAR_TITLE Font font.styles.toolbar-title
TOOLBAR_TITLE_SUBTITLED Font font.styles.toolbar-title-subtitled
ABOUT_NAME Font font.styles.about-name
SUBHEADLINE Font font.styles.subheadline
FOOTNOTE Font font.styles.footnote
CAPTION_2 Font font.styles.caption-2
MONO Font font.styles.mono
TEXT_LINE_HEIGHT f32 font.line-height-scale
SHADOW ShadowStyle shadow.layers.light
SHADOW_DARK ShadowStyle shadow.layers.dark
WINDOW_SHADOW_LIGHT WindowShadow shadow.window.light
WINDOW_SHADOW_DARK WindowShadow shadow.window.dark
WINDOW_SHADOW_SPREAD f32 shadow.window.spread
KNOB_SHADOW ShadowLayer shadow.slint.knob y_offset=offset-y opacity==1.0
SIDEBAR SidebarMetrics components.window.sidebar width=size.sidebar-w padding_x=padding-x
    rim_width=rim-width traffic_x=traffic-x traffic_y=traffic-y app_icon=item.app-icon
    chevron=section.chevron
GLASS GlassMetrics components.window.sidebar.glass chroma_light=chroma.light
    chroma_dark=chroma.dark chroma_cap_light=chroma-cap.light chroma_cap_dark=chroma-cap.dark
    lightness_light=lightness.light lightness_dark=lightness.dark
    blur_passes_light=blur-passes.light blur_passes_dark=blur-passes.dark
    edge_intensity_light=edge.intensity.light edge_intensity_dark=edge.intensity.dark
    edge_span_light=edge.span.light edge_span_dark=edge.span.dark
    edge_chroma_light=edge.chroma.light edge_chroma_dark=edge.chroma.dark
    edge_lightness_light=edge.lightness.light edge_lightness_dark=edge.lightness.dark
    edge_cap_light=edge.cap.light edge_cap_dark=edge.cap.dark
    shadow_opacity=shadow.opacity
    rim_opacity=rim.opacity rim_lift_light=rim.lift.light rim_lift_dark=rim.lift.dark
    rim_lift_edge_light=rim.lift-edge.light rim_lift_edge_dark=rim.lift-edge.dark
    rim_chroma=rim.chroma rim_cap=rim.cap
    edge_offset=edge.offset content_span=content.span content_opacity=content.opacity
    shadow_span=shadow.span shadow_tint=shadow.tint
    shadow_lightness=shadow.lightness
TOOLGROUP ToolgroupMetrics components.window.toolgroup
MENU_POPOVER MenuPopoverMetrics components.menu-popover
DURATION_BASE u64 duration.base
DURATION_FAST_MS u64 duration.fast
DURATION_SLOW_MS u64 duration.slow
DURATION_LIST_ROW_MS u64 duration.list-row
DURATION_DISCLOSURE_MS u64 duration.disclosure
DURATION_OUTLINE_EXPAND_MS u64 duration.outline-expand
SPRING_SHEET_MS u64 spring.sheet-duration
SPRING_PUSH_MS u64 spring.push-duration
DESKTOP_SWITCH_OMEGA f32 spring.desktop-switch-omega
DESKTOP_SWITCH_DAMPING f32 spring.desktop-switch-damping
DESKTOP_GAP f32 spring.desktop-gap
DESKTOP_GAP_COLOR Rgba spring.desktop-gap-color
DESKTOP_BACKGROUND Rgba spring.desktop-background
CURSOR_FALLBACK_SIZE i32 cursor.fallback-size
CURSOR_FALLBACK_COLOR Rgba cursor.fallback-color
EXPANSION_OMEGA f32 spring.expansion-omega
EXPANSION_DAMPING f32 spring.expansion-damping
SPACES_STRIP_EXPAND_OMEGA f32 spring.spaces-strip-expand-omega
SPACES_STRIP_EXPAND_DAMPING f32 spring.spaces-strip-expand-damping
THEME_SHADOW_OPACITY f32 shadow.theme.opacity
SPACES_STRIP_SHADOW BoxShadow shadow.spaces-strip
EXPANSION_MARGIN_SIDE i32 components.expansion.margin-side
EXPANSION_MARGIN_TOP i32 components.expansion.margin-top
EXPANSION_MARGIN_BOTTOM i32 components.expansion.margin-bottom
EXPANSION_HOVER_RING_WIDTH i32 components.expansion.hover-ring-width
EXPANSION_TITLE_FONT Font components.expansion.title-font
EXPANSION_TITLE_HEIGHT i32 components.expansion.title-height
EXPANSION_TITLE_PADDING i32 components.expansion.title-padding
EXPANSION_TITLE_COLOR Rgba components.expansion.title-color
EXPANSION_TITLE_FILL Rgba components.expansion.title-fill
FULL_SCREEN_BACKGROUND Rgba components.full-screen.background
SPACES_STRIP_HEIGHT i32 components.spaces-strip.height
SPACES_STRIP_EXPANDED_HEIGHT i32 components.spaces-strip.expanded-height
SPACES_STRIP_TINT Rgba components.spaces-strip.tint
SPACES_STRIP_BLUR f32 components.spaces-strip.blur
SPACES_STRIP_SATURATION f32 components.spaces-strip.saturation
SPACES_STRIP_CAP f32 components.spaces-strip.cap
SPACES_STRIP_LINE Rgba components.spaces-strip.line
SPACES_THUMBNAIL_HEIGHT i32 components.spaces-strip.thumbnail-height
SPACES_THUMBNAIL_TOP i32 components.spaces-strip.thumbnail-top
SPACES_THUMBNAIL_GAP i32 components.spaces-strip.thumbnail-gap
SPACES_LABEL_CELL i32 components.spaces-strip.label-cell
SPACES_LABEL_PILL_HEIGHT i32 components.spaces-strip.label-pill-height
SPACES_LABEL_PILL_TOP i32 components.spaces-strip.label-pill-top
SPACES_LABEL_PILL_RADIUS f32 components.spaces-strip.label-pill-radius
SPACES_LABEL_PILL_LIFT f32 components.spaces-strip.label-pill-lift
SPACES_LABEL_FONT Font components.spaces-strip.label-font
SPACES_EXPANDED_LABEL_FONT Font components.spaces-strip.expanded-label-font
SPACES_LABEL_GAP i32 components.spaces-strip.label-gap
SPACES_LABEL_COLOR Rgba components.spaces-strip.label-color
SPACES_RING_WIDTH i32 components.spaces-strip.ring-width
SPACES_RING_GAP i32 components.spaces-strip.ring-gap
SPACES_ADD_SIZE i32 components.spaces-strip.add-size
SPACES_ADD_STROKE i32 components.spaces-strip.add-stroke
SPACES_ADD_INSET f32 components.spaces-strip.add-inset
SPACES_ADD_EXPANDED_CENTER f32 components.spaces-strip.add-expanded-center
SPACES_ADD_COLOR Rgba components.spaces-strip.add-color
SPACES_CLOSE_SIZE i32 components.spaces-strip.close-size
SPACES_CLOSE_COLOR Rgba components.spaces-strip.close-color
SPACES_CLOSE_CROSS_COLOR Rgba components.spaces-strip.close-cross-color
SPACES_CLOSE_CROSS_SIZE i32 components.spaces-strip.close-cross-size
SPACES_CLOSE_CROSS_STROKE f32 components.spaces-strip.close-cross-stroke
WINDOW_CASCADE_STEP i32 components.window.cascade-step
WINDOW_CENTER_TOP f32 components.window.center-top
WINDOW_TILE_MARGIN i32 components.window.tile-margin
MENUBAR_HEIGHT i32 components.shell.menubar.height
DOCK_LABEL_RADIUS f32 components.shell.dock.label-radius
DOCK_LABEL_ARROW_WIDTH f32 components.shell.dock.label-arrow-width
DOCK_LABEL_ARROW_HEIGHT f32 components.shell.dock.label-arrow-height
DOCK_LABEL_ARROW_TIP_RADIUS f32 components.shell.dock.label-arrow-tip-radius
DOCK_LABEL_ARROW_FILLET f32 components.shell.dock.label-arrow-fillet
DOCK_MENU_ARROW_TIP_RADIUS f32 components.shell.dock.menu-arrow-tip-radius
DOCK_MENU_ARROW_FILLET f32 components.shell.dock.menu-arrow-fillet
SETTINGS SettingsMetrics components.settings account_name_font=account.name
    account_detail_font=account.detail card_caption_font=card.caption
HAIRLINE f32 components.hairline
STACK_COLUMN_GAPS [f32;25] components.stack.column-gaps
SYSTEM_SIZE_ASCENT f32 components.system-size.ascent
SYSTEM_SIZE_DESCENT f32 components.system-size.descent
SYSTEM_SIZE_LEADING [f32;7] components.system-size.leading-sizes
STACK_TEXT_SIZES [f32;8] components.stack.text-sizes
STACK_TEXT_ABOVE [f32;8] components.stack.text-above
STACK_TEXT_BELOW [f32;8] components.stack.text-below
LABEL_ICON_WIDTH f32 components.label.icon-width
LABEL_ICON_HEIGHT f32 components.label.icon-height
LABEL_ICON_FRAME f32 components.label.icon-frame
LABEL_GAP f32 components.label.gap
CONTENT_UNAVAILABLE_ICON f32 components.content-unavailable.icon
CONTENT_UNAVAILABLE_ICON_LINE f32 components.content-unavailable.icon-line
CONTENT_UNAVAILABLE_ICON_GAP f32 components.content-unavailable.icon-gap
CONTENT_UNAVAILABLE_TITLE Font components.content-unavailable.title
CONTENT_UNAVAILABLE_DESCRIPTION Font components.content-unavailable.description
CONTENT_UNAVAILABLE_DESCRIPTION_GAP f32 components.content-unavailable.description-gap
CONTENT_UNAVAILABLE_ACTIONS_GAP f32 components.content-unavailable.actions-gap
DOMINANT_MIN_CHROMA f32 appearance.dominant.min-chroma
TITLEBAR_PADDING_X f32 components.window.titlebar.padding-x
TITLEBAR_SYSTEM_VEIL Rgba components.window.titlebar.system-veil
TITLEBAR_EXPANDED_TITLE_HEIGHT f32 components.window.titlebar.expanded-title.height
TITLEBAR_EXPANDED_TITLE_GAP f32 components.window.titlebar.expanded-title.gap
TITLEBAR_EXPANDED_TOOLBAR_BOTTOM f32 components.window.titlebar.expanded-title.toolbar-bottom
TITLEBAR_TOOLBAR_START TitlebarToolbarStart components.window.titlebar.toolbar-start
TITLEBAR_FULLSCREEN_START TitlebarToolbarStart components.window.titlebar.fullscreen-start
MENU_MARK_TEXT_GAP f32 components.shell.menu.mark.text-gap
MENU_ICON_TEXT_GAP f32 components.shell.menu.icon.text-gap
MENU_SUBMENU_INTENT_MS u64 components.shell.menu.submenu.intent-ms
MENU_SCROLL_TICK_MS u64 components.shell.menu.scroll-tick-ms
HUD_BACKGROUND Rgba components.hud.background
DOMINANT_LIGHTNESS [f32;2] appearance.dominant.lightness
";

const THEMED: &str = r"
BG_WINDOW Rgba themes.{theme}.bg-window
BG_SURFACE Rgba themes.{theme}.bg-surface
BG_SUNKEN Rgba themes.{theme}.bg-sunken
BG_RAISED Rgba themes.{theme}.bg-raised
BG_HOVER Rgba themes.{theme}.bg-hover
RING_TRACK Rgba themes.{theme}.ring-track
BAR_TRACK Rgba themes.{theme}.bar-track
LEVEL_TRACK Rgba themes.{theme}.level-track
LEVEL_BORDER Rgba themes.{theme}.level-border
SEARCH_CLEAR Rgba themes.{theme}.search-clear
DISABLED DisabledPalette themes.{theme}.disabled
GAUGE_CONTRAST GaugeContrast themes.{theme}.gauge-contrast
RELEVANCY_TRACK Rgba themes.{theme}.relevancy-track
RELEVANCY RelevancyPalette themes.{theme}.relevancy
RELEVANCY_CONTRAST RelevancyPalette themes.{theme}.relevancy-contrast
TOKEN_FILL Rgba themes.{theme}.token-fill
TOKEN_FILL_CONTRAST Rgba themes.{theme}.token-fill-contrast
CALENDAR CalendarPalette themes.{theme}.calendar
CALENDAR_CONTRAST CalendarPalette themes.{theme}.calendar-contrast
DISCLOSURE_TRIANGLE Rgba themes.{theme}.disclosure-triangle
DISCLOSURE_TRIANGLE_CONTRAST Rgba themes.{theme}.disclosure-triangle-contrast
FRAME_LINES FrameLines themes.{theme}.frame-lines
FRAME_LINES_CONTRAST FrameLines themes.{theme}.frame-lines-contrast
PRESSED PressedPalette themes.{theme}.pressed
PRESSED_CONTRAST PressedPalette themes.{theme}.pressed-contrast
DISABLED_CONTRAST DisabledPalette themes.{theme}.disabled-contrast
DATE_FIELD Rgba themes.{theme}.date-field
DATE_FIELD_BORDER Rgba themes.{theme}.date-field-border
DATE_FIELD_CONTRAST Rgba themes.{theme}.date-field-contrast
DATE_FIELD_BORDER_CONTRAST Rgba themes.{theme}.date-field-border-contrast
STEPPER_DIVIDER Rgba themes.{theme}.stepper-divider
STEPPER_DIVIDER_CONTRAST Rgba themes.{theme}.stepper-divider-contrast
BEZEL BezelPalette themes.{theme}.bezel
BEZEL_CONTRAST BezelPalette themes.{theme}.bezel-contrast
TRAFFIC_LIGHTS TrafficLooks themes.{theme}.traffic-lights
TRAFFIC_LIGHTS_CONTRAST TrafficLooks themes.{theme}.traffic-lights-contrast
CONTRAST ContrastPalette themes.{theme}.contrast
SHELL ShellPalette components.shell bar=menubar.bg text=menubar.text tile=dock.tile tile_active=dock.tile-active tile_text_active=dock.tile-text-active
GLASS_INK Rgba themes.{theme}.glass-ink
GLASS_INK_MUTED Rgba themes.{theme}.glass-ink-muted
GLASS_INK_FAINT Rgba themes.{theme}.glass-ink-faint
SIDEBAR Rgba themes.{theme}.sidebar-bg
TOOLBAR_BG Rgba themes.{theme}.toolbar-bg
TOOLBAR_LINE Rgba themes.{theme}.toolbar-line
SIDEBAR_GLASS_BASE Rgba themes.{theme}.sidebar-glass-base
SIDEBAR_GLASS_SOLID Rgba themes.{theme}.sidebar-glass-solid
TOOLGROUP_BG Rgba themes.{theme}.toolbar-glass-bg
SIDEBAR_PALETTE SidebarPalette components.window.sidebar.item item_ink=ink item_icon=icon-ink
    section_ink=components.window.sidebar.section.ink line=components.window.sidebar.line
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
    let tengen = Tengen::load(&manifest, &[manifest.join("src/lib.rs")]);

    let mut out = String::from(
        "// @generated by build.rs from tengen/tokens.json and tengen/components.json.\n\
         // Do not edit: change the JSON in crates/ctdesign/tengen and rebuild.\n\n",
    );
    out.push_str(&tengen.constants(CONSTANTS));
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
                // Reduce Transparency changes these two to their own colors, not to the flattened translucent ones.
                let path = if opaque && entry[0] != "SHELL" {
                    format!("{path}-reduced")
                } else {
                    path.clone()
                };
                let look = Look { theme, opaque };
                out.push_str(&tengen.constant(&name, entry[1], &path, &entry[3..], look));
            }
        }
    }
    emit_titlebar(&mut out, &tengen);
    emit_tints(&mut out, &tengen);
    emit_palettes(&mut out, &tengen);
    emit_specials(&mut out, &tengen);

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("tengen.rs"), out).expect("write tengen.rs");
}

fn emit_titlebar(out: &mut String, tengen: &Tengen) {
    let lights = "components.window.traffic-lights";
    let button = |kind: &str| format!("Button {{ kind: ButtonKind::{kind} }}");
    let int = |path: &str| tengen.number(path) as i64;
    out.push_str(&format!(
        "pub const TITLEBAR: Titlebar = Titlebar {{ thickness: {}, buttons: TitlebarButtons {{ order: [{}, {}, {}], diameter: {}, spacing: {}, inset: {}, inset_compact: {} }}, font: {} }};\n\n",
        int("size.titlebar-ssd-h"),
        button("Close"),
        button("Minimize"),
        button("Maximize"),
        int(&format!("{lights}.size")),
        int(&format!("{lights}.gap")),
        int(&format!("{lights}.inset")),
        int(&format!("{lights}.inset-compact")),
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
        (
            "WALLPAPER_TINTING_LIFT",
            "appearance.wallpaper-tinting.lift".to_owned(),
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
        out.push_str(&format!(
            "pub const {name}: Palette = Palette {{ titlebar_bg: {}, hairline: {}, inner_line: {inner_line}, panel_inner_line: {panel_inner_line}, outer_border: {}, title: {}, row_title: {}, text: {}, ink_strong: {}, ink_muted: {}, ink_faint: {}, line: {}, line_strong: {}, separator: {}, accent: {}, accent_inactive: {}, on_accent: {}, danger: {}, scrim: {}, selected_control: {}, card_border: {}, system_fill: {}, text_background: {}, selected_content: {}, control_disabled: {}, control: {}, accent_toggle: {}, separator_strong: {}, selected_content_inactive: {}, combo_button: {}, segment_divider: {}, accent_ring: {}, accent_toggle_disabled: {} }};\n\n",
            pair("titlebar-bg", "titlebar-bg-inactive"),
            rgba_of("titlebar-line"),
            rgba_of("window-outer"),
            pair("titlebar-title", "titlebar-title-inactive"),
            pair("titlebar-row-title", "titlebar-row-title-inactive"),
            pair("ink", "ink-muted"),
            rgba_of("ink-strong"),
            rgba_of("ink-muted"),
            rgba_of("ink-faint"),
            rgba_of("line"),
            rgba_of("line-strong"),
            rgba_of("separator"),
            rgba_of("accent"),
            rgba_of("accent-inactive"),
            rgba_of("on-accent"),
            rgba_of("danger"),
            rgba_of("scrim"),
            rgba_of("bg-selected-control"),
            rgba_of("card-border"),
            rgba_of("system-fill"),
            rgba_of("text-background"),
            rgba_of("selected-content"),
            rgba_of("bg-control-disabled"),
            rgba_of("bg-control"),
            rgba_of("accent-toggle"),
            rgba_of("separator-strong"),
            rgba_of("selected-content-inactive"),
            rgba_of("combo-button"),
            rgba_of("segment-divider"),
            rgba_of("accent-ring"),
            rgba_of("accent-toggle-disabled"),
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
    let accents = tengen
        .get("components.settings.accents")
        .as_array()
        .expect("ctdesign: accents must be a list");
    let colors_in = |accent: &Value, key: &str, theme: &str| {
        let fields: Vec<String> = accent[key]
            .as_object()
            .unwrap_or_else(|| panic!("ctdesign: accent {key} must be an object"))
            .iter()
            .map(|(field, text)| {
                let text = text.as_str().expect("ctdesign: accent colors are strings");
                let look = Look {
                    theme,
                    opaque: false,
                };
                format!(
                    "{}: {}",
                    field.replace('-', "_"),
                    rgba(tengen.color_of(text, look))
                )
            })
            .collect();
        format!("AccentColors {{ {} }}", fields.join(", "))
    };
    let entries: Vec<String> = accents
        .iter()
        .map(|accent| {
            format!(
                "Accent {{ name: {:?}, light: {}, dark: {}, light_contrast: {}, dark_contrast: {} }}",
                accent["name"].as_str().expect("ctdesign: accent name"),
                colors_in(accent, "light", "light"),
                colors_in(accent, "dark", "dark"),
                colors_in(accent, "light-contrast", "light"),
                colors_in(accent, "dark-contrast", "dark"),
            )
        })
        .collect();
    out.push_str(&format!(
        "pub const ACCENTS: [Accent; {}] = [{}];\n\n",
        entries.len(),
        entries.join(", ")
    ));
    let swatches: Vec<String> = (0..entries.len())
        .map(|index| format!("(ACCENTS[{index}].name, ACCENTS[{index}].light.control)"))
        .collect();
    out.push_str(&format!(
        "pub const ACCENT_SWATCHES: [(&str, Rgba); {}] = [{}];\n\n",
        swatches.len(),
        swatches.join(", ")
    ));
    for (constant, path) in [
        ("MULTICOLOR", "components.settings.multicolor"),
        ("SYSTEM_COLORS", "components.system-colors.light"),
        ("SYSTEM_COLORS_DARK", "components.system-colors.dark"),
        (
            "LEVEL_INDICATOR_COLORS",
            "components.level-indicator.colors",
        ),
        (
            "LEVEL_INDICATOR_COLORS_DARK",
            "components.level-indicator.colors-dark",
        ),
        (
            "LEVEL_INDICATOR_COLORS_CONTRAST",
            "components.level-indicator.colors-contrast",
        ),
        (
            "LEVEL_INDICATOR_COLORS_CONTRAST_DARK",
            "components.level-indicator.colors-contrast-dark",
        ),
    ] {
        let colors: Vec<String> = tengen
            .get(path)
            .as_array()
            .unwrap_or_else(|| panic!("ctdesign: {path} must be a list of colors"))
            .iter()
            .map(|color| {
                rgba(parse_color(
                    color.as_str().expect("ctdesign: colors are strings"),
                ))
            })
            .collect();
        out.push_str(&format!(
            "pub const {constant}: [Rgba; {}] = [{}];\n\n",
            colors.len(),
            colors.join(", ")
        ));
    }
}
