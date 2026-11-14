#![forbid(unsafe_code)]

mod dock_divider;
mod rgba;

pub use dock_divider::dock_divider;
pub use rgba::Rgba;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficRim {
    pub lit: Rgba,
    pub idle: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusPair {
    pub focused: Rgba,
    pub unfocused: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Font {
    pub family: &'static str,
    pub size: u32,
    pub line_height: u32,
    pub weight: u16,
}

impl Font {
    pub fn scaled(self, factor: f32) -> Self {
        let scale = |value: u32| (value as f32 * factor).round() as u32;
        Self {
            size: scale(self.size),
            line_height: scale(self.line_height),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Close,
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Button {
    pub kind: ButtonKind,
    pub color: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub a: (f32, f32),
    pub b: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle(pub [(f32, f32); 3]);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficGlyphs {
    pub box_size: f32,
    pub stroke: f32,
    pub close: &'static [Segment],
    pub minimize: &'static [Segment],
    pub zoom: &'static [Triangle],
    pub plus: &'static [Segment],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubleClick {
    pub interval_ms: u32,
    pub slop: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitlebarButtons {
    pub order: [Button; 3],
    pub diameter: u32,
    pub spacing: u32,
    pub corner_radius: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Titlebar {
    pub thickness: u32,
    pub buttons: TitlebarButtons,
    pub font: Font,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowCorner {
    pub titlebar: u32,
    pub unified_compact: u32,
    pub unified: u32,
    pub utility: u32,
    pub expansion: f32,
    pub exponent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowStyle {
    pub corner_radius: u32,
    pub superellipse_exponent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowResize {
    pub edge: f32,
    pub corner: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLayer {
    pub blur: u32,
    pub y_offset: i32,
    pub opacity: f32,
    pub color: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsStrength {
    pub width: u32,
    pub track: u32,
    pub tick: u32,
    pub tick_gap: u32,
    pub label_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsWallpaper {
    pub preview_width: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub dot: u32,
    pub sheet_width: u32,
    pub row_fits: usize,
    pub shuffle_scrim: f32,
    pub shuffle_icon: f32,
    pub field_width: u32,
    pub filter_width: u32,
    pub filter_control: f32,
    pub filter_inset: f32,
    pub filter_chevron: f32,
    pub filter_close_icon: f32,
    pub filter_scope_tint: f32,
    pub edge_fade: f32,
    pub kind_size: f32,
    pub kind_icon: f32,
    pub kind_inset: f32,
    pub source_width: u32,
    pub source_height: u32,
    pub source_radius: u32,
    pub crop_width: u32,
    pub crop_side: u32,
    pub crop_aspect: f32,
    pub crop_dim: f32,
    pub crop_frame: f32,
    pub crop_guide: f32,
    pub crop_corner: u32,
    pub crop_corner_stroke: u32,
    pub crop_zoom_max: f32,
    pub crop_zoom_step: f32,
    pub crop_zoom_button: u32,
    pub crop_zoom_label: u32,
    pub crop_soft: f32,
    pub crop_blurry: f32,
    pub crop_preview_bar: f32,
    pub crop_preview_dock_width: f32,
    pub crop_preview_dock_bottom: f32,
    pub crop_preview_dock_tiles: u32,
    pub crop_preview_dock_gap: f32,
    pub crop_preview_dock_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsCheck {
    pub gap: u32,
    pub row_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsMetrics {
    pub window_width: u32,
    pub window_height: u32,
    pub window_min_height: u32,
    pub search_padding_x: u32,
    pub search_icon: u32,
    pub search_gap: u32,
    pub search_popover_gap: u32,
    pub search_popover_max_height: u32,
    pub search_result_height: u32,
    pub search_header_height: u32,
    pub account_avatar: u32,
    pub account_gap: u32,
    pub account_height: u32,
    pub account_padding_x: u32,
    pub account_margin_bottom: u32,
    pub account_name_font: Font,
    pub account_detail_font: Font,
    pub section_icon_bg: Rgba,
    pub swatch_size: u32,
    pub swatch_gap: u32,
    pub option_column: u32,
    pub card_width: u32,
    pub card_height: u32,
    pub card_radius: u32,
    pub card_gap: u32,
    pub card_caption_gap: u32,
    pub card_caption_font: Font,
    pub card_caption_selected_weight: u32,
    pub preview_window_radius: u32,
    pub preview_window_x: u32,
    pub preview_window_y: u32,
    pub preview_light: u32,
    pub preview_menubar_dot: u32,
    pub preview_menubar_dot_inset: u32,
    pub preview_menu_x: u32,
    pub preview_menu_y: u32,
    pub preview_menu_w: u32,
    pub preview_menu_h: u32,
    pub preview_menu_padding: u32,
    pub preview_menu_accent_h: u32,
    pub preview_menu_line_h: u32,
    pub preview_menu_line_radius: u32,
    pub preview_app_inset: u32,
    pub preview_dots_x: u32,
    pub preview_dots_y: u32,
    pub preview_dot_gap: u32,
    pub preview_radius: u32,
    pub preview_bar_h: u32,
    pub preview_bar_padding: u32,
    pub preview_bar_item_w: u32,
    pub preview_bar_item_gap: u32,
    pub radio_size: u32,
    pub radio_dot: u32,
    pub radio_gap: u32,
    pub radio_spacing: u32,
    pub popup_height: u32,
    pub popup_chevron: u32,
    pub popup_text_gap: u32,
    pub sidebar_width: u32,
    pub sidebar_lights_gap: u32,
    pub sidebar_search_gap: u32,
    pub sidebar_group_gap: u32,
    pub sidebar_padding_top: u32,
    pub strength: SettingsStrength,
    pub check: SettingsCheck,
    pub dropdown_chevron_glyph: u32,
    pub empty_padding_y: u32,
    pub empty_icon: u32,
    pub empty_gap: u32,
    pub wallpaper: SettingsWallpaper,
    pub toolbar_gap: u32,
    pub toolbar_height: u32,
    pub toolbar_padding_top: u32,
    pub toolbar_padding_x: u32,
    pub swatch_column: u32,
    pub swatch_name_gap: u32,
    pub swatch_ring_gap: u32,
    pub icon_card_size: u32,
    pub icon_card_radius: u32,
    pub icon_card_ring_gap: u32,
    pub card_outline: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderMetrics {
    pub height: f32,
    pub track: f32,
    pub knob_width: f32,
    pub knob_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxMetrics {
    pub size: f32,
    pub radius: f32,
    pub mark: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    pub inset: f32,
    pub knob_color: Rgba,
    pub transition_ms: u64,
    pub disabled_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedMetrics {
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub segment_radius: f32,
    pub segment_height: f32,
    pub segment_padding_x: f32,
    pub font_size: f32,
    pub font_weight: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusRingMetrics {
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
    pub font_size: f32,
    pub font_weight: i32,
    pub pressed_scale: f32,
    pub large_height: f32,
    pub small_height: f32,
    pub small_padding_x: f32,
    pub small_font_size: f32,
    pub small_radius: f32,
    pub menu_padding_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextFieldMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressMetrics {
    pub height: f32,
    pub ring_size: f32,
    pub ring_stroke: f32,
    pub indeterminate_segment: f32,
    pub indeterminate_ms: u64,
    pub spinner_size: f32,
    pub spinner_spokes: u32,
    pub spinner_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropScenes {
    pub dusk: [Rgba; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropMetrics {
    pub angles: [f32; 3],
    pub opacity: f32,
    pub stops: u32,
    pub lightness: [f32; 2],
    pub chroma: f32,
    pub base_darken: f32,
    pub base_lightness: [f32; 2],
    pub base_chroma: f32,
    pub samples: u32,
    pub clusters: u32,
    pub ink_secondary: f32,
    pub ink_tertiary: f32,
    pub glass: f32,
    pub glass_strong: f32,
    pub island: f32,
    pub sunken: f32,
    pub accent_lightness_on_dark: f32,
    pub accent_lightness_on_light: f32,
    pub accent_min_chroma: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BreakpointMetrics {
    pub medium: f32,
    pub wide: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipMetrics {
    pub delay_ms: u64,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub pointer_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitlebarRailMetrics {
    pub width: f32,
    pub actions_width: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub title_size: f32,
    pub title_angle: f32,
    pub pill_inset: f32,
    pub pill_padding_x: f32,
    pub pill_padding_y: f32,
    pub pill_radius: f32,
    pub reveal_zone: f32,
    pub reveal_corner: f32,
    pub edge_line: f32,
    pub edge_accent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FullscreenBarMetrics {
    pub content_top: f32,
    pub band_line: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FullscreenGlassMetrics {
    pub height: f32,
    pub blur: f32,
    pub layers: usize,
    pub content_top: f32,
    pub tint: &'static [(f32, f32)],
    pub mask: &'static [(f32, f32)],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolMetrics {
    pub stroke: f32,
    pub grid: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconPlateMetrics {
    pub full_chroma: f32,
    pub neutral_chroma: f32,
    pub lobe_min_share: f32,
    pub bright_lightness: f32,
    pub rim_lightness: f32,
    pub rim_chroma: f32,
    pub rim_width: f32,
    pub body: f32,
    pub logo_scale: f32,
    pub shadow_far_offset: f32,
    pub shadow_far_opacity: f32,
    pub shadow_near_offset: f32,
    pub shadow_near_opacity: f32,
    pub logo_shadow_offset: f32,
    pub logo_shadow_opacity: f32,
    pub glass_shadow_factor: f32,
    pub outline_width: f32,
    pub outline_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconPlateLook {
    pub top_lightness: f32,
    pub top_chroma_scale: f32,
    pub top_chroma_cap: f32,
    pub bottom_lightness: f32,
    pub bottom_chroma_scale: f32,
    pub bottom_chroma_cap: f32,
    pub bottom_hue_shift: f32,
    pub opacity_top: f32,
    pub opacity_bottom: f32,
    pub rim_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconLogoEnds {
    pub dark_lightness: f32,
    pub dark_chroma_scale: f32,
    pub dark_chroma_cap: f32,
    pub light_lightness: f32,
    pub light_chroma_scale: f32,
    pub light_chroma_cap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconStyleMetrics {
    pub tint_dark_lightness: f32,
    pub tint_dark_chroma: f32,
    pub tint_light_lightness: f32,
    pub tint_light_chroma: f32,
    pub clear_floor: f32,
    pub clear_range: f32,
    pub clear_alpha_gain: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderFatMetrics {
    pub height: f32,
    pub fill_min: f32,
    pub lead_icon_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconButtonMetrics {
    pub size: f32,
    pub radius: f32,
    pub icon: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartMetrics {
    pub height: f32,
    pub line_width: f32,
    pub point_size: f32,
    pub rule_width: f32,
    pub grid_width: f32,
    pub area_opacity: f32,
    pub bar_gap: f32,
    pub bar_radius: f32,
    pub ticks: u32,
    pub y_axis_width: f32,
    pub axis_gap: f32,
    pub legend_swatch: f32,
    pub legend_gap: f32,
    pub sector_inner: f32,
    pub sector_inset: f32,
    pub sector_radius: f32,
    pub unselected_opacity: f32,
    pub heat_gap: f32,
    pub heat_radius: f32,
    pub azimuth: f32,
    pub inclination: f32,
    pub drag_degrees: f32,
    pub scroll_min_span: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub wheel_zoom: f32,
    pub stream_hole: f32,
    pub hover_opacity: f32,
    pub box_scale: f32,
    pub ambient_light: f32,
    pub depth_scale: f32,
    pub height_3d: f32,
    pub guide_rings: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaugeMetrics {
    pub size: f32,
    pub stroke: f32,
    pub arc: f32,
    pub marker: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragMetrics {
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageWellMetrics {
    pub size: f32,
    pub radius: f32,
    pub inset: f32,
    pub drop_border: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelIndicatorMetrics {
    pub height: f32,
    pub radius: f32,
    pub star: f32,
    pub stars: u32,
    pub segment_gap: f32,
    pub relevancy_height: f32,
    pub relevancy_line: f32,
    pub relevancy_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DatePickerMetrics {
    pub calendar_width: f32,
    pub calendar_height: f32,
    pub header_height: f32,
    pub weekday_height: f32,
    pub day_radius: f32,
    pub segment_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenFieldMetrics {
    pub token_height: f32,
    pub token_padding_x: f32,
    pub gap: f32,
    pub input_min_width: f32,
    pub token_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathControlMetrics {
    pub height: f32,
    pub gap: f32,
    pub chevron: f32,
    pub icon: f32,
    pub icon_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupBoxMetrics {
    pub radius: f32,
    pub padding: f32,
    pub label_inset: f32,
    pub fill_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisclosureMetrics {
    pub indent: f32,
    pub row_height: f32,
    pub chevron: f32,
    pub row_radius: f32,
    pub icon: f32,
    pub column_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintMetrics {
    pub dpi: f32,
    pub margin_x: f32,
    pub margin_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpellingMetrics {
    pub dot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontPanelMetrics {
    pub sizes: [f32; 14],
    pub size_column: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopoverMetrics {
    pub radius: f32,
    pub arrow_width: f32,
    pub arrow_height: f32,
    pub arrow_tip_radius: f32,
    pub arrow_fillet: f32,
    pub padding: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxMetrics {
    pub button_width: f32,
    pub button_height: f32,
    pub button_inset_x: f32,
    pub button_radius: f32,
    pub text_gap: f32,
    pub chevron: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepperMetrics {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub chevron: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeMetrics {
    pub height: f32,
    pub padding_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaMetrics {
    pub button_height: f32,
    pub button_padding_x: f32,
    pub button_radius: f32,
    pub button_font_size: f32,
    pub button_font_weight: i32,
    pub button_fill: Rgba,
    pub button_ink: Rgba,
    pub round: f32,
    pub round_icon: f32,
    pub round_glass: Rgba,
    pub round_rim: Rgba,
    pub material_blur: f32,
    pub material_tint: Rgba,
    pub material_rim: Rgba,
    pub scrim_side: &'static [(f32, f32)],
    pub scrim_bottom: &'static [(f32, f32)],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialMetrics {
    pub ultra_thin: MaterialThickness,
    pub thin: MaterialThickness,
    pub regular: MaterialThickness,
    pub thick: MaterialThickness,
    pub ultra_thick: MaterialThickness,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialThickness {
    pub opacity: f32,
    pub blur: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarMetrics {
    pub thumb: f32,
    pub inset: f32,
    pub min_length: f32,
    pub opacity: f32,
    pub track: f32,
    pub track_opacity: f32,
    pub always_opacity: f32,
    pub edge_height: f32,
    pub hide_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListGroupMetrics {
    pub radius: f32,
    pub row_min_height: f32,
    pub row_padding_x: f32,
    pub separator_inset: f32,
    pub padding: [f32; 4],
    pub header_padding: [f32; 4],
    pub highlight: f32,
    pub label_min_share: f32,
    pub stack_below: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitViewMetrics {
    pub sidebar_min: f32,
    pub sidebar_max: f32,
    pub collapse_below: f32,
    pub inspector: f32,
    pub grip: f32,
    pub grip_line: f32,
    pub grip_line_inset: f32,
    pub key_step: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitPanesMetrics {
    pub divider: f32,
    pub pane_min: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingPaneMetrics {
    pub title_height: f32,
    pub outline: f32,
    pub resize_band: f32,
    pub shadow: Rgba,
    pub shadow_blur: f32,
    pub shadow_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableMetrics {
    pub row_height: f32,
    pub header_height: f32,
    pub padding_x: f32,
    pub zebra: f32,
    pub min_column_width: f32,
    pub resize_handle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollectionMetrics {
    pub tile: f32,
    pub icon: f32,
    pub gap: f32,
    pub radius: f32,
    pub selection: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolbarTabsMetrics {
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub tab_min_width: f32,
    pub tab_padding_top: f32,
    pub tab_padding_bottom: f32,
    pub tab_radius: f32,
    pub icon: f32,
    pub icon_gap: f32,
    pub font_size: f32,
    pub font_weight: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassMetrics {
    pub chroma_light: f32,
    pub chroma_dark: f32,
    pub chroma_cap_light: f32,
    pub chroma_cap_dark: f32,
    pub lightness_light: f32,
    pub lightness_dark: f32,
    pub accent_share: f32,
    pub blur_passes_light: u32,
    pub blur_passes_dark: u32,
    pub edge_intensity_light: f32,
    pub edge_intensity_dark: f32,
    pub edge_span_light: f32,
    pub edge_span_dark: f32,
    pub edge_chroma_light: f32,
    pub edge_chroma_dark: f32,
    pub edge_lightness_light: f32,
    pub edge_lightness_dark: f32,
    pub edge_cap_light: f32,
    pub edge_cap_dark: f32,
    pub rim_opacity: f32,
    pub rim_lift_light: f32,
    pub rim_lift_dark: f32,
    pub rim_lift_edge_light: f32,
    pub rim_lift_edge_dark: f32,
    pub rim_chroma: f32,
    pub rim_cap: f32,
    pub edge_offset: f32,
    pub shadow_opacity_light: f32,
    pub shadow_opacity_dark: f32,
    pub shadow_span: f32,
    pub shadow_tint: f32,
    pub shadow_lightness_light: f32,
    pub shadow_lightness_dark: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarMetrics {
    pub width: u32,
    pub inset: u32,
    pub padding_x: u32,
    pub radius: u32,
    pub rim_width: f32,
    pub inner_width: f32,
    pub traffic_x: u32,
    pub traffic_y: u32,
    pub item_height: u32,
    pub item_radius: u32,
    pub item_padding_x: u32,
    pub item_gap: u32,
    pub item_text_size: u32,
    pub item_weight: u16,
    pub item_selected_weight: u16,
    pub item_icon: u32,
    pub app_icon: u32,
    pub section_height: u32,
    pub section_margin_top: u32,
    pub section_padding_x: u32,
    pub section_size: u32,
    pub section_weight: u16,
    pub chevron: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupMetrics {
    pub height: u32,
    pub padding_x: u32,
    pub separator_w: u32,
    pub separator_h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowStyle {
    pub ambient: ShadowLayer,
    pub contact: ShadowLayer,
    pub unfocused_blur_scale: f32,
    pub unfocused_offset_scale: f32,
    pub unfocused_opacity_scale: f32,
}

impl ShadowStyle {
    pub fn layers(self, focused: bool) -> [ShadowLayer; 2] {
        if focused {
            return [self.ambient, self.contact];
        }
        let unfocused = |layer: ShadowLayer| ShadowLayer {
            blur: (layer.blur as f32 * self.unfocused_blur_scale).round() as u32,
            y_offset: (layer.y_offset as f32 * self.unfocused_offset_scale).round() as i32,
            opacity: layer.opacity * self.unfocused_opacity_scale,
            ..layer
        };
        [unfocused(self.ambient), unfocused(self.contact)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelLine {
    pub sides: Rgba,
    pub edges: Rgba,
    pub glow: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub titlebar_bg: FocusPair,
    pub hairline: Rgba,
    pub inner_line: Option<Rgba>,
    pub panel_inner_line: Option<PanelLine>,
    pub outer_border: Rgba,
    pub title: FocusPair,
    pub text: FocusPair,
    pub ink_strong: Rgba,
    pub ink_muted: Rgba,
    pub ink_faint: Rgba,
    pub line: Rgba,
    pub line_strong: Rgba,
    pub accent: Rgba,
    pub accent_inactive: Rgba,
    pub on_accent: Rgba,
    pub danger: Rgba,
    pub traffic_lights_inactive: Rgba,
    pub traffic_rim: TrafficRim,
    pub scrim: Rgba,
    pub selected_control: Rgba,
    pub card_border: Rgba,
}
/// The Dock's glass and its frame, which the compositor draws (malaysia's `dock_frame.glsl`). `rim`
/// is the rim's luminance as `[lift, share]` of the glass's, `divider` the shift of the glass's
/// luminance that marks a divider (positive lights it, negative darkens it), `corner` how the frame
/// turns from rim to outline around a corner: `[decay, outline from, outline to]` in degrees, and
/// `glow` the glow inside the rim pixel by pixel as shares of it. With `tone` the frame takes its
/// colours from what is behind the glass, and the rim keeps their hue: the glass is scaled per
/// channel up to the luminance `rim` gives, instead of overlaid with white or black. `lens` is how
/// the glass bends the backdrop along the frame's normal, as the Liquid Glass config's
/// `liquid-glass.frag` does: `[depth, strength]`, both as shares of the Dock's thickness. It never
/// splits the colour channels. These are design values, not measured on macOS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockSurface {
    pub glass: Rgba,
    pub outline: Rgba,
    pub rim: [f32; 2],
    pub divider: f32,
    pub corner: [f32; 3],
    pub glow: [f32; 6],
    pub lens: [f32; 2],
    pub tone: Option<DockTone>,
}

/// The translucent Dock's frame as macOS draws it over a wallpaper, measured on its Dock: the
/// glass takes the backdrop's OKLab chroma times `chroma` and its own lightness plus `lift`, and the
/// outline is the glass with its lightness and chroma times `outline`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTone {
    pub chroma: f32,
    pub lift: f32,
    pub outline: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellPalette {
    pub bar: Rgba,
    pub text: Rgba,
    pub tile: Rgba,
    pub tile_active: Rgba,
    pub tile_text_active: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuPopoverMetrics {
    pub item_height: f32,
    pub item_radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub item_padding_start: f32,
    pub check: f32,
    pub leading_column: f32,
    pub swatch_grid_size: f32,
    pub swatch_grid_dot: f32,
    pub swatch_grid_gap: f32,
    pub separator_margin: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuPalette {
    pub bg: Rgba,
    pub padding: f32,
    pub shadow: Rgba,
    pub min_width: f32,
    pub max_width: f32,
    pub item_height: f32,
    pub item_radius: f32,
    pub item_padding_x: f32,
    pub font_size: f32,
    pub shortcut_gap: f32,
    pub fg: Rgba,
    pub shortcut_color: Rgba,
    pub hover_bg: Rgba,
    pub hover_fg: Rgba,
    pub bar_hover_bg: Rgba,
    pub bar_title_open_bg: Rgba,
    pub bar_title_open_radius: f32,
    pub bar_title_open_height: f32,
    pub disabled_fg: Rgba,
    pub separator_color: Rgba,
    pub separator_height: f32,
    pub separator_margin_y: f32,
    pub leading_width: f32,
    pub mark_inset: f32,
    pub mark_dot: f32,
    pub mark_check: f32,
    pub chevron_size: f32,
    pub chevron_color: Rgba,
    pub icon_size: f32,
    pub header_font_size: f32,
    pub header_weight: i32,
    pub header_height: f32,
    pub search_height: f32,
    pub search_margin_x: f32,
    pub search_margin_y: f32,
    pub search_bg: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarPalette {
    pub item_ink: Rgba,
    pub item_icon: Rgba,
    pub selected_bg: Rgba,
    pub selected_ink: Rgba,
    pub selected_icon_rim: Rgba,
    pub inactive_selected_bg: Rgba,
    pub inactive_ink: Rgba,
    pub hover_bg: Rgba,
    pub section_ink: Rgba,
    pub count_ink: Rgba,
    pub line: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupPalette {
    pub separator: Rgba,
    pub rim: Rgba,
}

pub const NATIVE_APP_ID_PREFIX: &str = "io.github.hblanqueto.malaysia.";
pub const ALERT_LAYER_NAMESPACE: &str = "malaysia-dialog";
pub const NOTIFICATIONS_LAYER_NAMESPACE: &str = "malaysia-notifications";
pub const MENUBAR_LAYER_NAMESPACE: &str = "malaysia-menubar";
pub const DOCK_LAYER_NAMESPACE: &str = "malaysia-dock";
pub const WALLPAPER_LAYER_NAMESPACE: &str = "malaysia-wallpaper";

include!(concat!(env!("OUT_DIR"), "/tengen.rs"));
