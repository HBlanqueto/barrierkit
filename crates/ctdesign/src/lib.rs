#![forbid(unsafe_code)]

mod dock_divider;
mod ease;
mod rgba;

pub use dock_divider::dock_divider;
pub use ease::{ease_in_ease_out, ease_in_out, ease_out, progress, smoothstep};
pub use rgba::Rgba;

pub const CIRCLE_EXPONENT: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficLook {
    pub base: Rgba,
    pub fill: [Rgba; 4],
    pub ring: [Rgba; 3],
    pub glyph: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficPress {
    pub scale: f32,
    pub dodge: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficHalo {
    pub color: Rgba,
    pub blur: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficLooks {
    pub close: TrafficLook,
    pub minimize: TrafficLook,
    pub zoom: TrafficLook,
    pub inactive: TrafficLook,
    pub halo: TrafficHalo,
    pub backdrop: Rgba,
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
    pub stroke: f32,
    pub zoom_corner: f32,
    pub close: &'static [Segment],
    pub minimize: &'static [Segment],
    pub zoom: &'static [Triangle],
    pub zoom_exit: &'static [Triangle],
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
    pub inset: u32,
    pub inset_compact: u32,
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
pub struct CornerShape {
    pub radius: f32,
    pub fitted: f32,
    pub smoothing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerControls {
    pub push: [f32; 5],
    pub circular: [f32; 5],
    pub help: [f32; 5],
    pub text_field: [f32; 5],
    pub popup: [f32; 5],
    pub combo_button: [f32; 5],
    pub checkbox: [f32; 5],
    pub radio: [f32; 5],
    pub segmented: [f32; 5],
    pub segmented_square: [f32; 5],
    pub slider_track: [f32; 5],
    pub slider_knob: [f32; 5],
    pub progress_bar: [f32; 5],
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
    pub row_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsMetrics {
    pub window_width: u32,
    pub window_height: u32,
    pub window_min_height: u32,
    pub search_padding_x: u32,
    pub search_icon: u32,
    pub search_glyph: f32,
    pub search_glyph_stroke: f32,
    pub search_glyph_dx: f32,
    pub search_glyph_dy: f32,
    pub search_clear: f32,
    pub search_clear_glyph: f32,
    pub search_clear_glyph_stroke: f32,
    pub search_clear_dx: f32,
    pub search_clear_dy: f32,
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
    pub tile_blue: Rgba,
    pub tile_green: Rgba,
    pub tile_cyan: Rgba,
    pub tile_black: Rgba,
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
    pub radio_spacing: u32,
    pub sidebar_width: u32,
    pub sidebar_lights_gap: u32,
    pub sidebar_search_gap: u32,
    pub sidebar_group_gap: u32,
    pub sidebar_padding_top: u32,
    pub strength: SettingsStrength,
    pub check: SettingsCheck,
    pub dropdown_chevron_glyph: u32,
    pub empty_padding_y: u32,
    pub empty_gap: u32,
    pub wallpaper: SettingsWallpaper,
    pub toolbar_gap: u32,
    pub toolbar_lights_gap: u32,
    pub toolbar_height: u32,
    pub toolbar_padding_x: u32,
    pub swatch_column: u32,
    pub swatch_name_gap: u32,
    pub swatch_ring_gap: u32,
    pub icon_card_size: u32,
    pub icon_card_ring_gap: u32,
    pub card_outline: Rgba,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlSize {
    Mini,
    Small,
    #[default]
    Regular,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderMetrics {
    pub height: f32,
    pub track: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    pub knob_pressed_scale: f32,
    pub knob_glass_alpha: f32,
    pub mini: SliderSize,
    pub small: SliderSize,
    pub large: SliderSize,
    pub xl: SliderSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderSize {
    pub height: f32,
    pub track: f32,
    pub knob_width: f32,
}

impl SliderMetrics {
    pub fn size(&self, size: ControlSize) -> SliderSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => SliderSize {
                height: self.height,
                track: self.track,
                knob_width: self.knob_width,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxMetrics {
    pub size: f32,
    pub radius: f32,
    pub mark: f32,
    pub mark_stroke: f32,
    pub mixed_mark: f32,
    pub mixed_stroke: f32,
    pub gap: f32,
    pub font_size: f32,
    pub radio_dot: f32,
    pub mini: CheckboxSize,
    pub small: CheckboxSize,
    pub large: CheckboxSize,
    pub xl: CheckboxSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxSize {
    pub size: f32,
    pub radius: f32,
    pub mark: f32,
    pub mark_stroke: f32,
    pub mixed_mark: f32,
    pub mixed_stroke: f32,
    pub gap: f32,
    pub font_size: f32,
    pub radio_dot: f32,
}

impl CheckboxMetrics {
    pub fn size(&self, size: ControlSize) -> CheckboxSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => CheckboxSize {
                size: self.size,
                radius: self.radius,
                mark: self.mark,
                mark_stroke: self.mark_stroke,
                mixed_mark: self.mixed_mark,
                mixed_stroke: self.mixed_stroke,
                gap: self.gap,
                font_size: self.font_size,
                radio_dot: self.radio_dot,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    pub inset: f32,
    pub knob_color: Rgba,
    pub knob_pressed_scale: f32,
    pub knob_glass_alpha: f32,
    pub transition_ms: u64,
    pub disabled_opacity: f32,
    pub disabled_opacity_contrast: f32,
    pub mini: SwitchSize,
    pub small: SwitchSize,
    pub large: SwitchSize,
    pub xl: SwitchSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchSize {
    pub width: f32,
    pub height: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    pub inset: f32,
}

impl SwitchMetrics {
    pub fn size(&self, size: ControlSize) -> SwitchSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => SwitchSize {
                width: self.width,
                height: self.height,
                knob_width: self.knob_width,
                knob_height: self.knob_height,
                inset: self.inset,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedMetrics {
    pub padding: f32,
    pub gap: f32,
    pub radius: f32,
    pub segment_radius: f32,
    pub segment_height: f32,
    pub segment_padding_x: f32,
    pub divider_height: f32,
    pub font_size: f32,
    pub font_weight: i32,
    pub mini: SegmentedSize,
    pub small: SegmentedSize,
    pub large: SegmentedSize,
    pub xl: SegmentedSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedSize {
    pub segment_height: f32,
    pub segment_padding_x: f32,
    pub radius: f32,
    pub divider_height: f32,
    pub font_size: f32,
}

impl SegmentedMetrics {
    pub fn size(&self, size: ControlSize) -> SegmentedSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => SegmentedSize {
                segment_height: self.segment_height,
                segment_padding_x: self.segment_padding_x,
                radius: self.radius,
                divider_height: self.divider_height,
                font_size: self.font_size,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
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
    pub mini: ButtonSize,
    pub small: ButtonSize,
    pub large: ButtonSize,
    pub xl: ButtonSize,
    pub popup: PopupSizes,
    pub bezel: BezelMetrics,
    pub circular_size: f32,
    pub circular_icon: f32,
    pub menu_padding_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonSize {
    pub height: f32,
    pub padding_x: f32,
    pub font_size: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezelRow {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezelMetrics {
    pub disabled_opacity: f32,
    pub small_square: BezelRow,
    pub textured_rounded: BezelRow,
    pub textured_square: BezelRow,
    pub inline: BezelRow,
    pub push_disclosure: BezelRow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezelPalette {
    pub square_top: Rgba,
    pub square_bottom: Rgba,
    pub square_border: Rgba,
    pub textured_top: Rgba,
    pub textured_bottom: Rgba,
    pub textured_border: Rgba,
    pub ring: Rgba,
    pub inline_fill: Rgba,
    pub square_pressed: Rgba,
    pub ring_pressed: Rgba,
    pub textured_pressed: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupSize {
    pub glyph: f32,
    pub glyph_stroke: f32,
    pub indicator: f32,
    pub trailing: f32,
    pub gap: f32,
    pub pulldown_glyph: f32,
    pub pulldown_drop: f32,
    pub label_lift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupSizes {
    pub mini: PopupSize,
    pub small: PopupSize,
    pub regular: PopupSize,
    pub large: PopupSize,
    pub xl: PopupSize,
}

impl ButtonMetrics {
    pub fn size(&self, size: ControlSize) -> ButtonSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => ButtonSize {
                height: self.height,
                padding_x: self.padding_x,
                font_size: self.font_size,
                radius: self.radius,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
}

impl PopupSizes {
    pub fn size(&self, size: ControlSize) -> PopupSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => self.regular,
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextFieldMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
    pub secure_dot: f32,
    pub secure_pitch: f32,
    pub secure_inset: f32,
    pub secure_drop: f32,
    pub plain_height: f32,
    pub plain_padding_x: f32,
    pub mini: TextFieldSize,
    pub small: TextFieldSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressMetrics {
    pub height: f32,
    pub frames: [f32; 5],
    pub ring_size: f32,
    pub ring_stroke: f32,
    pub ring_trim: f32,
    pub indeterminate_segment: f32,
    pub indeterminate_ms: u64,
    pub spinner_sizes: [f32; 5],
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
    pub sunken: f32,
    pub accent_lightness_on_dark: f32,
    pub accent_lightness_on_light: f32,
    pub accent_min_chroma: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextEditorMetrics {
    pub padding_x: f32,
    pub padding_y: f32,
    pub line_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextFieldSize {
    pub height: f32,
    pub font_size: u32,
    pub line_height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchFieldMetrics {
    pub mini: SearchFieldSize,
    pub small: SearchFieldSize,
    pub large: SearchFieldSize,
    pub xl: SearchFieldSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchFieldSize {
    pub height: f32,
    pub font_size: u32,
    pub line_height: u32,
    pub icon: f32,
    pub glyph: f32,
    pub glyph_stroke: f32,
    pub glyph_dx: f32,
    pub glyph_dy: f32,
    pub clear: f32,
    pub clear_glyph: f32,
    pub clear_glyph_stroke: f32,
    pub clear_dx: f32,
    pub clear_dy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipMetrics {
    pub delay_ms: u64,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub pointer_gap: f32,
    pub wrap_after: f32,
    pub wrap_width: f32,
    pub bg_light: Rgba,
    pub bg_dark: Rgba,
    pub bg_contrast_light: Rgba,
    pub bg_contrast_dark: Rgba,
    pub ink_light: Rgba,
    pub ink_dark: Rgba,
    pub ink_contrast_light: Rgba,
    pub ink_contrast_dark: Rgba,
}

/// Where a toolbar's first item starts: after the traffic lights in the unified and compact
/// bars, from the window edge in the expanded toolbar row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitlebarToolbarStart {
    pub unified: f32,
    pub compact: f32,
    pub expanded: f32,
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
    pub fine_stroke: f32,
    pub grid: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconPlateMetrics {
    pub neutral_chroma: f32,
    pub lobe_min_share: f32,
    pub bright_lightness: f32,
    pub body: f32,
    pub corner: f32,
    pub logo_scale: f32,
    pub shadow_far_offset: f32,
    pub shadow_far_opacity: f32,
    pub shadow_near_offset: f32,
    pub shadow_near_opacity: f32,
    pub logo_shadow_offset: f32,
    pub logo_shadow_opacity: f32,
    pub glass_shadow_factor: f32,
    pub edge_width: f32,
    pub light_reach: f32,
    pub light_sharpness: f32,
    pub light_side: f32,
    pub outline_ends: f32,
    pub outline_from: f32,
    pub outline_to: f32,
    pub tile_top_lift: f32,
    pub tile_bottom_lift: f32,
    pub dark_glyph_lightness: f32,
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
    pub light_top: f32,
    pub light_bottom: f32,
    pub light_tone: f32,
    pub outline_lightness: f32,
    pub outline_alpha: f32,
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
    pub clear_cut: f32,
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
    pub toolbar_icon: f32,
    pub toolbar_stroke: f32,
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
    pub star: f32,
    pub star_pitch: f32,
    pub star_drop: f32,
    pub stars: u32,
    pub segment_gap: f32,
    pub discrete_border: f32,
    pub discrete_border_contrast_light: f32,
    pub edge_softness: f32,
    pub relevancy_height: f32,
    pub relevancy_line: f32,
    pub relevancy_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DatePickerMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub stepper_width: f32,
    pub stepper_height: f32,
    pub stepper_chevron: f32,
    pub stepper_chevron_stroke: f32,
    pub stepper_chevron_offset_up: f32,
    pub stepper_chevron_offset_down: f32,
    pub stepper_separator_inset: f32,
    pub stepper_gap: f32,
    pub calendar_width: f32,
    pub calendar_height: f32,
    pub card_left: f32,
    pub card_top: f32,
    pub card_right: f32,
    pub card_bottom: f32,
    pub mini: DatePickerSize,
    pub small: DatePickerSize,
    pub large: DatePickerSize,
    pub xl: DatePickerSize,
    pub header_height: f32,
    pub weekday_height: f32,
    pub day_height: f32,
    pub column_width: f32,
    pub grid_left: f32,
    pub grid_gap: f32,
    pub title_inset: f32,
    pub title_lift: f32,
    pub weekday_font_size: u32,
    pub nav_width: f32,
    pub nav_height: f32,
    pub dot_size: f32,
    pub nav_pitch: f32,
    pub nav_inset: f32,
    pub selected_width: f32,
    pub selected_height: f32,
    pub selected_overhang: f32,
    pub selected_lift: f32,
    pub day_radius: f32,
    pub segment_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DatePickerSize {
    pub height: f32,
    pub stepper_gap: f32,
    pub stepper_width: f32,
    pub stepper_height: f32,
    pub stepper_chevron: f32,
    pub stepper_chevron_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenFieldMetrics {
    pub padding_x: f32,
    pub token_height: f32,
    pub token_drop: f32,
    pub token_padding_x: f32,
    pub token_label_lift: f32,
    pub token_radius: f32,
    pub gap: f32,
    pub input_min_width: f32,
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
pub struct CircularSliderMetrics {
    pub size: f32,
    pub dash_from: f32,
    pub dash_to: f32,
    pub dash_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPickerMetrics {
    pub inset: f32,
    pub ring_shift: f32,
    pub disabled_opacity: f32,
    pub expanded_width: f32,
    pub expanded_radius: f32,
    pub expanded_gap: f32,
    pub wheel_size: f32,
    pub wheel_radius: f32,
    pub wheel_diameter: f32,
    pub wheel_center: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupBoxMetrics {
    pub radius: f32,
    pub padding: f32,
    pub label_inset: f32,
    pub label_height: f32,
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
    pub inset: f32,
    pub lead: f32,
    pub label_gap: f32,
    pub separator_margin: f32,
    pub list_chevron: f32,
    pub list_chevron_stroke: f32,
    pub list_chevron_x: f32,
    pub list_chevron_open_x: f32,
    pub list_chevron_open_y: f32,
    pub group_lead: f32,
    pub group_chevron: f32,
    pub group_chevron_stroke: f32,
    pub group_chevron_x: f32,
    pub group_chevron_open_x: f32,
    pub group_chevron_open_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpellingMetrics {
    pub dot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontPanelMetrics {
    pub sizes: [f32; 14],
    pub size_column: f32,
    pub name_columns: f32,
    pub visible_rows: f32,
    pub preview_rows: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopoverMetrics {
    pub radius: f32,
    pub arrow_width: f32,
    pub arrow_height: f32,
    pub arrow_tip_radius: f32,
    pub arrow_fillet: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxMetrics {
    pub button_width: f32,
    pub button_height: f32,
    pub button_inset_x: f32,
    pub button_radius: f32,
    pub text_gap: f32,
    pub chevron: f32,
    pub chevron_stroke: f32,
    pub mini: ComboBoxSize,
    pub small: ComboBoxSize,
    pub large: ComboBoxSize,
    pub xl: ComboBoxSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxSize {
    pub height: f32,
    pub font_size: u32,
    pub line_height: u32,
    pub button_width: f32,
    pub button_height: f32,
    pub button_inset_x: f32,
    pub button_radius: f32,
    pub chevron: f32,
    pub chevron_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelpLinkMetrics {
    pub glyph: f32,
    pub glyph_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepperMetrics {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub chevron: f32,
    pub chevron_stroke: f32,
    pub chevron_offset_up: f32,
    pub chevron_offset_down: f32,
    pub separator_inset: f32,
    pub separator_drop: f32,
    pub gap: f32,
    pub mini: StepperSize,
    pub small: StepperSize,
    pub large: StepperSize,
    pub xl: StepperSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepperSize {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub chevron: f32,
    pub chevron_stroke: f32,
}

impl StepperMetrics {
    pub fn size(&self, size: ControlSize) -> StepperSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            ControlSize::Regular => StepperSize {
                width: self.width,
                height: self.height,
                radius: self.radius,
                chevron: self.chevron,
                chevron_stroke: self.chevron_stroke,
            },
            ControlSize::Large => self.large,
            ControlSize::ExtraLarge => self.xl,
        }
    }
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
pub struct MaterialLook {
    pub sigma: f32,
    pub saturation: f32,
    pub veil: Rgba,
    pub cap: f32,
    pub floor: f32,
    pub gain: f32,
    pub bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialSet {
    pub active: MaterialLook,
    pub inactive: MaterialLook,
    pub reduced: MaterialLook,
    pub reduced_contrast: MaterialLook,
}

pub const RIM_BINS: usize = 35;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassLook {
    pub blur: f32,
    pub blur_opacity: [f32; 4],
    pub blur_distance: [f32; 4],
    pub white: f32,
    pub black: f32,
    pub saturation: f32,
    pub max_luma: f32,
    pub fill: Rgba,
    pub refraction: [f32; 2],
    pub tone_delta: [f32; 9],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassRim {
    pub angle_1: [f32; 12],
    pub angle_2: [f32; 12],
    pub distance_1: [f32; RIM_BINS],
    pub distance_2: [f32; RIM_BINS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassScale {
    pub refraction_amount: [f32; 4],
    pub refraction_height: [f32; 4],
    pub blur_opacity_0: [f32; 4],
    pub blur_opacity_1: [f32; 4],
    pub blur_distance_0: [f32; 4],
    pub max_luma: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassSet {
    pub active: GlassLook,
    pub inactive: GlassLook,
    pub reduced: GlassLook,
    pub increased_contrast: GlassLook,
    pub reduced_contrast: GlassLook,
    pub rim: GlassRim,
    pub scale: GlassScale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassLens {
    pub scale: f32,
    pub decay: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassTint {
    pub blur: [f32; 3],
    pub blur_opacity: [f32; 4],
    pub blur_fill_opacity: [f32; 3],
    pub blur_fill_radius: f32,
    pub fill_rgb: [f32; 3],
    pub fill_alpha: [f32; 3],
    pub max_luma: [f32; 3],
    pub read_halvings: [f32; 2],
    pub tone_shift_t25: [f32; 9],
    pub tone_shift_t50: [f32; 9],
    pub tone_shift_t75: [f32; 9],
    pub tone_shift_t100: [f32; 9],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarMetrics {
    pub thumb: f32,
    pub inset: f32,
    pub min_length: f32,
    pub opacity: [f32; 4],
    pub track: f32,
    pub track_opacity: [f32; 4],
    pub edge_height: f32,
    pub hide_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureMetrics {
    pub tap_distance: f32,
    pub tap_interval_ms: f32,
    pub long_press_ms: f32,
    pub long_press_distance: f32,
    pub drag_distance: f32,
    pub magnify_delta: f32,
    pub rotate_degrees: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPhysicsMetrics {
    pub fling_distance: f32,
    pub fling_exponent: f32,
    pub min_speed: f32,
    pub velocity_window_ms: f32,
    pub band: f32,
    pub touch_band: f32,
    pub band_speed: f32,
    pub return_omega: f32,
    pub refresh_trigger: f32,
    pub refresh_hold: f32,
    pub refresh_band: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListSectionMetrics {
    pub header_height: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListGroupMetrics {
    pub radius: f32,
    pub row_min_height: f32,
    pub row_padding_x: f32,
    pub row_padding_y: f32,
    pub separator_inset: f32,
    pub padding: [f32; 4],
    pub header_padding: [f32; 4],
    pub highlight: f32,
    pub label_min_share: f32,
    pub section_spacing: f32,
    pub columns_gap: f32,
    pub columns_spacing: f32,
    pub max_width: f32,
    pub header_weight: u16,
    pub popup_height: f32,
    pub popup_indicator: f32,
    pub popup_gap: f32,
    pub popup_trailing: f32,
    pub date_height: f32,
    pub slider_share: f32,
    pub control_overhang: f32,
    pub capacity_frame: f32,
    pub dark_accent_lift: [f32; 3],
    pub card: [Rgba; 4],
    pub separator_line: [Rgba; 4],
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
    pub grab: f32,
    pub pane_min: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableMetrics {
    pub row_height: f32,
    pub header_height: f32,
    pub padding_x: f32,
    pub plain_padding_x: f32,
    pub header_padding_x: f32,
    pub column_shift: f32,
    pub row_inset: f32,
    pub row_radius: f32,
    pub body_inset: f32,
    pub header_line: f32,
    pub header_separator: f32,
    pub zebra: f32,
    pub zebra_contrast: [f32; 2],
    pub min_column_width: f32,
    pub resize_handle: f32,
    pub sorted_weight: u32,
    pub sort_arrow: f32,
    pub sort_arrow_stroke: f32,
    pub sort_arrow_inset: f32,
    pub sort_arrow_drop: f32,
    pub plain_row_height: f32,
    pub row_size_font: [f32; 3],
    pub row_size_icon: [f32; 3],
    pub source_list_row_height: [f32; 3],
    pub source_list_inset: f32,
    pub icon_box: f32,
    pub icon_gap: f32,
    pub label_row_height: f32,
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
    pub tab_padding_x: f32,
    pub tab_padding_top: f32,
    pub tab_padding_bottom: f32,
    pub tab_radius: f32,
    pub icon: f32,
    pub icon_gap: f32,
    pub font_size: f32,
    pub font_weight: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabViewSize {
    pub strip_height: f32,
    pub strip_inset: f32,
    pub strip_radius: f32,
    pub strip_padding_x: f32,
    pub page_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabViewMetrics {
    pub strip_height: f32,
    pub strip_inset: f32,
    pub strip_radius: f32,
    pub strip_padding_x: f32,
    pub page_gap: f32,
    pub small: TabViewSize,
    pub mini: TabViewSize,
    pub strip_padding_y: f32,
    pub strip_opacity: [f32; 4],
    pub pill_inset: f32,
    pub pill_inset_x: f32,
    pub pill_radius: f32,
    pub pill_fill: [Rgba; 4],
    pub pill_highlight: [Rgba; 4],
    pub pill_ring: [f32; 4],
    pub pill_highlight_depth: f32,
    pub box_radius: f32,
    pub box_inset_x: f32,
    pub box_inset_far: f32,
    pub box_inset_near: f32,
    pub box_fill_opacity: f32,
    pub box_border_opacity: [f32; 4],
    pub line_border_opacity: [f32; 4],
    pub left_strip_top: f32,
    pub bezel_inset: [f32; 4],
    pub bezel_page_inset: [f32; 4],
}

impl TabViewMetrics {
    pub fn size(&self, size: ControlSize) -> TabViewSize {
        match size {
            ControlSize::Mini => self.mini,
            ControlSize::Small => self.small,
            _ => TabViewSize {
                strip_height: self.strip_height,
                strip_inset: self.strip_inset,
                strip_radius: self.strip_radius,
                strip_padding_x: self.strip_padding_x,
                page_gap: self.page_gap,
            },
        }
    }
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
    pub content_span: f32,
    pub content_opacity: f32,
    pub shadow_opacity: f32,
    pub shadow_span: f32,
    pub shadow_tint: f32,
    pub shadow_lightness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarMetrics {
    pub width: u32,
    pub padding_x: u32,
    pub rim_width: f32,
    pub traffic_x: u32,
    pub traffic_y: u32,
    pub toggle_inset: f32,
    pub item_height: u32,
    pub item_radius: u32,
    pub item_padding_x: u32,
    pub item_text_padding_x: u32,
    pub item_gap: u32,
    pub item_text_size: u32,
    pub item_weight: u16,
    pub item_selected_weight: u16,
    pub item_icon: u32,
    pub item_glyph: f32,
    pub item_glyph_stroke: f32,
    pub app_icon: u32,
    pub item_tile_radius: f32,
    pub section_height: u32,
    pub section_margin_bottom: u32,
    pub section_padding_x: u32,
    pub section_size: u32,
    pub section_weight: u16,
    pub chevron: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupMetrics {
    pub height: u32,
    pub item_width: f32,
    pub padding_x: f32,
    pub separator_w: u32,
    pub separator_h: u32,
    pub compact_width: u32,
    pub compact_height: u32,
    pub compact_inset: u32,
    pub compact_separator_h: u32,
    pub expanded_width: u32,
    pub expanded_height: u32,
    pub expanded_inset: u32,
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
pub struct CalendarPalette {
    pub background: Rgba,
    pub border: Rgba,
    pub border_width: f32,
    pub edge: Rgba,
    pub separator: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelevancyPalette {
    pub fill: Rgba,
    pub line: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaugeContrast {
    pub ring_track: Rgba,
    pub level_track: Rgba,
    pub level_border: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentColors {
    pub control: Rgba,
    pub selected_content: Rgba,
    pub selected_text: Rgba,
    pub focus_ring: Rgba,
    pub toggle: Rgba,
    pub toggle_disabled: Rgba,
    pub ring: Rgba,
    pub pressed: Rgba,
    pub pressed_toggle: Rgba,
    pub segment_disabled: Rgba,
    pub sidebar_selection: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklchShift {
    pub lightness_scale: f32,
    pub lightness_offset: f32,
    pub chroma_scale: f32,
    pub hue_shift: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccentRule {
    pub selected_content_light: OklchShift,
    pub selected_content_dark: OklchShift,
    pub focus_ring_light: OklchShift,
    pub focus_ring_dark: OklchShift,
    pub focus_ring_contrast_light: OklchShift,
    pub focus_ring_contrast_dark: OklchShift,
    pub selected_text_mix: f32,
    pub selected_text_base_light: Rgba,
    pub selected_text_base_dark: Rgba,
    pub toggle_mix: [f32; 4],
    pub toggle_disabled_mix: [f32; 4],
    pub pressed_mix: [f32; 4],
    pub pressed_toggle_mix: [f32; 4],
    pub segment_disabled_mix: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Accent {
    pub name: &'static str,
    pub light: AccentColors,
    pub dark: AccentColors,
    pub light_contrast: AccentColors,
    pub dark_contrast: AccentColors,
}

impl Accent {
    pub fn colors(&self, dark: bool, contrast: bool) -> AccentColors {
        match (dark, contrast) {
            (false, false) => self.light,
            (true, false) => self.dark,
            (false, true) => self.light_contrast,
            (true, true) => self.dark_contrast,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressedPalette {
    pub bezel: Rgba,
    pub accent: Rgba,
    pub toggle_accent: Rgba,
    pub off: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisabledPalette {
    pub off: Rgba,
    pub on: Rgba,
    pub slider_fill: Rgba,
    pub switch_off: Rgba,
    pub stepper_chevron: Rgba,
    pub segment: Rgba,
    pub segment_label: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastPalette {
    pub ink: Rgba,
    pub ink_muted: Rgba,
    pub ink_faint: Rgba,
    pub separator: Rgba,
    pub line_strong: Rgba,
    pub bg_control: Rgba,
    pub bg_control_disabled: Rgba,
    pub combo_button: Rgba,
    pub system_fill: Rgba,
    pub text_background: Rgba,
    pub accent: Rgba,
    pub selected_content: Rgba,
    pub accent_toggle: Rgba,
    pub accent_toggle_disabled: Rgba,
    pub accent_ring: Rgba,
    pub separator_strong: Rgba,
    pub focus_ring: Rgba,
}

impl Palette {
    pub fn contrasted(self, contrast: ContrastPalette) -> Self {
        Self {
            ink_strong: contrast.ink,
            ink_muted: contrast.ink_muted,
            ink_faint: contrast.ink_faint,
            separator: contrast.separator,
            line_strong: contrast.line_strong,
            control: contrast.bg_control,
            control_disabled: contrast.bg_control_disabled,
            combo_button: contrast.combo_button,
            system_fill: contrast.system_fill,
            text_background: contrast.text_background,
            accent: contrast.accent,
            selected_content: contrast.selected_content,
            accent_toggle: contrast.accent_toggle,
            accent_toggle_disabled: contrast.accent_toggle_disabled,
            accent_ring: contrast.accent_ring,
            separator_strong: contrast.separator_strong,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub titlebar_bg: FocusPair,
    pub hairline: Rgba,
    pub inner_line: Option<Rgba>,
    pub panel_inner_line: Option<PanelLine>,
    pub outer_border: Rgba,
    pub title: FocusPair,
    /// The title of a title row (no toolbar, or SwiftUI's `.expanded`), dimmer than `title`.
    pub row_title: FocusPair,
    pub text: FocusPair,
    pub ink_strong: Rgba,
    pub ink_muted: Rgba,
    pub ink_faint: Rgba,
    pub line: Rgba,
    pub line_strong: Rgba,
    pub separator: Rgba,
    pub accent: Rgba,
    pub selected_content: Rgba,
    pub control_disabled: Rgba,
    pub control: Rgba,
    pub accent_toggle: Rgba,
    pub separator_strong: Rgba,
    pub selected_content_inactive: Rgba,
    pub combo_button: Rgba,
    pub segment_divider: Rgba,
    pub accent_ring: Rgba,
    pub accent_toggle_disabled: Rgba,
    pub accent_inactive: Rgba,
    pub on_accent: Rgba,
    pub danger: Rgba,
    pub scrim: Rgba,
    pub selected_control: Rgba,
    pub card_border: Rgba,
    pub system_fill: Rgba,
    pub text_background: Rgba,
}
/// The Dock's glass and its frame, which the compositor draws (malaysia's `dock_frame.glsl`). `rim`
/// is the rim's luminance as `[lift, share]` of the glass's, `divider` the shift of the glass's
/// luminance that marks a divider (positive lights it, negative darkens it), `corner` how the frame
/// turns from rim to outline around a corner: `[decay, outline from, outline to]` in degrees,
/// `rim_side` how much of the rim along the top and bottom stays on the left and right sides, and
/// `glow` the glow inside the rim pixel by pixel as shares of it. With `tone` the frame takes its
/// colours from what is behind the glass, and the rim keeps their hue: the glass is scaled per
/// channel toward the luminance `rim` gives, as far as `DockTone::gain`, and white makes up the
/// rest, instead of overlaid with white or black. `lens` is how
/// the glass bends the backdrop along the frame's normal, as the Liquid Glass config's
/// `liquid-glass.frag` does: `[depth, strength]`, both as shares of the Dock's thickness. It never
/// splits the colour channels. These are design values, not measured on macOS. `soften` is how
/// many times wider than the clear Dock's the Gaussian that softens the backdrop spreads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockSurface {
    pub glass: Rgba,
    pub outline: Rgba,
    pub rim: [f32; 2],
    pub divider: f32,
    pub corner: [f32; 3],
    pub rim_side: f32,
    pub glow: [f32; 6],
    pub lens: [f32; 2],
    pub tone: Option<DockTone>,
    pub soften: f32,
}

/// The translucent Dock's frame as macOS draws it over a wallpaper, measured on its Dock: the
/// glass takes the backdrop's OKLab chroma times `chroma` and its own lightness plus `lift`, bent
/// down to `cap` with its chroma, and the outline is the glass with its lightness and chroma times
/// `outline`. `gain` is the most the rim, and the light of a clear icon's plate, scale their glass
/// per channel, fitted to macOS's rim over the Tahoe wallpaper, top and bottom; past it they light
/// in white, so over a saturated wallpaper they keep a white base instead of the wallpaper's pure
/// colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTone {
    pub chroma: f32,
    pub lift: f32,
    pub outline: [f32; 2],
    pub cap: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellGlass {
    pub sigma: f32,
    pub halo_sigma: f32,
    pub halo_weight: f32,
    pub tone: [f32; 12],
    pub chroma: [f32; 5],
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
    pub mark_stroke: f32,
    pub mark_dash: f32,
    pub mark_dash_inset: f32,
    pub chevron_size: f32,
    pub chevron_gap: f32,
    pub chevron_inset: f32,
    pub chevron_drop: f32,
    pub chevron_scroll: f32,
    pub chevron_color: Rgba,
    pub icon_size: f32,
    pub header_font_size: f32,
    pub header_weight: i32,
    pub header_height: f32,
    pub search_height: f32,
    pub search_margin_x: f32,
    pub search_margin_y: f32,
    pub search_bg: Rgba,
    pub glass_fg: Rgba,
    pub glass_shortcut_color: Rgba,
    pub glass_disabled_fg: Rgba,
}

// The veil a SwiftUI sidebar lays over its glass for a selection the focus is not on, as measured on macOS 26 over two wallpapers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassSelection {
    pub unfocused: Rgba,
    pub inactive: Rgba,
}

pub const SIDEBAR_GLASS_SELECTION_LIGHT: GlassSelection = GlassSelection {
    unfocused: Rgba::rgba8(0, 0, 0, 0.094),
    inactive: Rgba::rgba8(0, 0, 0, 0.047),
};

pub const SIDEBAR_GLASS_SELECTION_DARK: GlassSelection = GlassSelection {
    unfocused: Rgba::rgba8(255, 255, 255, 0.133),
    inactive: Rgba::rgba8(255, 255, 255, 0.063),
};

// A SwiftUI toolbar item on glass: the circle under the pointer and under a press, inset in the capsule, as measured on macOS 27.
// An item alone on its glass is lit over the whole capsule instead, with the `alone_` hover (Resonance's sidebar toggle, 2026-11-23).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolbarItemHighlight {
    pub size: f32,
    pub hover_light: Rgba,
    pub hover_dark: Rgba,
    pub alone_hover_light: Rgba,
    pub alone_hover_dark: Rgba,
    pub pressed_light: Rgba,
    pub pressed_dark: Rgba,
}

pub const TOOLBAR_ITEM_HIGHLIGHT: ToolbarItemHighlight = ToolbarItemHighlight {
    size: 32.0,
    hover_light: Rgba::rgba8(0, 0, 0, 0.048),
    hover_dark: Rgba::rgba8(255, 255, 255, 0.047),
    alone_hover_light: Rgba::rgba8(0, 0, 0, 0.042),
    alone_hover_dark: Rgba::rgba8(255, 255, 255, 0.105),
    pressed_light: Rgba::rgba8(0, 0, 0, 0.02),
    pressed_dark: Rgba::rgba8(255, 255, 255, 0.142),
};

// A SwiftUI control on its own glass, as measured on macOS 27: its label at full strength, a toolbar item's glass `height`,
// the toolbar segmented control's thumbs (`inset` from the glass, `gap` apart) and the toolbar search field's padding
// before its icon box and gap after it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassControl {
    pub ink_light: Rgba,
    pub ink_dark: Rgba,
    pub height: f32,
    pub inset: f32,
    pub gap: f32,
    pub segment_padding_x: f32,
    pub selected_light: Rgba,
    pub selected_dark: Rgba,
    pub hover_light: Rgba,
    pub hover_dark: Rgba,
    pub search_padding_x: f32,
    pub search_gap: f32,
}

pub const GLASS_CONTROL: GlassControl = GlassControl {
    ink_light: Rgba::rgba8(0, 0, 0, 1.0),
    ink_dark: Rgba::rgba8(255, 255, 255, 1.0),
    height: 36.0,
    inset: 4.0,
    gap: 2.0,
    segment_padding_x: 11.0,
    selected_light: Rgba::rgba8(0, 0, 0, 0.081),
    selected_dark: Rgba::rgba8(255, 255, 255, 0.11),
    hover_light: Rgba::rgba8(0, 0, 0, 0.04),
    hover_dark: Rgba::rgba8(255, 255, 255, 0.11),
    search_padding_x: 9.0,
    search_gap: 8.0,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarPalette {
    pub item_ink: Rgba,
    pub item_icon: Rgba,
    pub selected_bg: Rgba,
    pub selected_ink: Rgba,
    pub selected_icon_rim: Rgba,
    pub inactive_selected_bg: Rgba,
    pub inactive_ink: Rgba,
    pub inactive_selected_ink: Rgba,
    pub inactive_bg: Rgba,
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
