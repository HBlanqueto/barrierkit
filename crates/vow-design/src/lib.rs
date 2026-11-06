#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self::rgba8(0, 0, 0, 0.0);

    pub fn mix(self, other: Self, amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * amount,
            g: self.g + (other.g - self.g) * amount,
            b: self.b + (other.b - self.b) * amount,
            a: self.a,
        }
    }

    pub fn darken(self, amount: f32) -> Self {
        self.mix(Self::rgb8(0, 0, 0), amount)
    }

    pub fn lighten(self, amount: f32) -> Self {
        self.mix(Self::rgb8(255, 255, 255), amount)
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 1.0)
    }

    pub const fn rgba8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    pub fn to_rgba8(self) -> [u8; 4] {
        [
            (self.r * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.g * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.b * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn premultiplied(self) -> [f32; 4] {
        [self.r * self.a, self.g * self.a, self.b * self.a, self.a]
    }

    pub fn to_linear(self) -> Self {
        let decode = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        Self {
            r: decode(self.r),
            g: decode(self.g),
            b: decode(self.b),
            a: self.a,
        }
    }

    pub fn from_linear(linear: Self) -> Self {
        let encode = |channel: f32| {
            let channel = channel.clamp(0.0, 1.0);
            if channel <= 0.003_130_8 {
                12.92 * channel
            } else {
                1.055 * channel.powf(1.0 / 2.4) - 0.055
            }
        };
        Self {
            r: encode(linear.r),
            g: encode(linear.g),
            b: encode(linear.b),
            a: linear.a,
        }
    }

    pub fn relative_luminance(self) -> f32 {
        let linear = self.to_linear();
        0.2126 * linear.r + 0.7152 * linear.g + 0.0722 * linear.b
    }

    pub fn contrast_ratio(self, other: Self) -> f32 {
        let a = self.relative_luminance();
        let b = other.relative_luminance();
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let lightness = (max + min) / 2.0;
        let delta = max - min;
        if delta == 0.0 {
            return (0.0, 0.0, lightness);
        }
        let saturation = if lightness > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };
        let hue = if max == self.r {
            ((self.g - self.b) / delta + if self.g < self.b { 6.0 } else { 0.0 }) / 6.0
        } else if max == self.g {
            ((self.b - self.r) / delta + 2.0) / 6.0
        } else {
            ((self.r - self.g) / delta + 4.0) / 6.0
        };
        (hue, saturation, lightness)
    }

    pub fn from_hsl(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        }
        let (r, g, b) = if saturation == 0.0 {
            (lightness, lightness, lightness)
        } else {
            let q = if lightness < 0.5 {
                lightness * (1.0 + saturation)
            } else {
                lightness + saturation - lightness * saturation
            };
            let p = 2.0 * lightness - q;
            (
                hue_to_rgb(p, q, hue + 1.0 / 3.0),
                hue_to_rgb(p, q, hue),
                hue_to_rgb(p, q, hue - 1.0 / 3.0),
            )
        };
        Self { r, g, b, a: alpha }
    }
}

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
    pub compact: u32,
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
pub struct SettingsCheck {
    pub gap: u32,
    pub row_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsMetrics {
    pub window_width: u32,
    pub window_height: u32,
    pub window_min_height: u32,
    pub search_height: u32,
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
    pub preview_window_radius: u32,
    pub preview_light: u32,
    pub preview_tint_window_x: u32,
    pub preview_tint_window_y: u32,
    pub preview_tint_side_w: u32,
    pub preview_panel_h: u32,
    pub preview_panel_w: u32,
    pub preview_menu_x: u32,
    pub preview_menu_y: u32,
    pub preview_menu_w: u32,
    pub preview_menu_h: u32,
    pub preview_menu_padding: u32,
    pub preview_menu_line_h: u32,
    pub preview_menu_line_radius: u32,
    pub preview_app_w: u32,
    pub preview_app_h: u32,
    pub preview_app_inset: u32,
    pub preview_app_side_w: u32,
    pub preview_dots_inset: u32,
    pub preview_dot_gap: u32,
    pub preview_dot_radius: u32,
    pub preview_radius: u32,
    pub preview_wall_angle: u32,
    pub preview_wall_light: f32,
    pub preview_wall_dark: f32,
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
    pub sidebar_padding_top: u32,
    pub strength: SettingsStrength,
    pub check: SettingsCheck,
    pub dropdown_chevron_glyph: u32,
    pub empty_padding_y: u32,
    pub empty_icon: u32,
    pub empty_gap: u32,
    pub toolbar_gap: u32,
    pub toolbar_height: u32,
    pub toolbar_padding_top: u32,
    pub toolbar_padding_x: u32,
    pub swatch_column: u32,
    pub swatch_name_gap: u32,
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub knob_width: f32,
    pub knob_height: f32,
    pub inset: f32,
    pub travel: f32,
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
    pub offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
    pub font_size: f32,
    pub font_weight: i32,
    pub pressed_scale: f32,
    pub small_height: f32,
    pub small_padding_x: f32,
    pub small_font_size: f32,
    pub small_radius: f32,
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
    pub width_wide: f32,
    pub actions_width: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub title_size: f32,
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
    pub hover_scale: f32,
    pub preview_delay_ms: u64,
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
pub struct TableMetrics {
    pub row_height: f32,
    pub header_height: f32,
    pub padding_x: f32,
    pub zebra: f32,
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
pub struct SidebarMetrics {
    pub width: u32,
    pub inset: u32,
    pub padding_x: u32,
    pub radius: u32,
    pub rim_width: f32,
    pub inner_width: f32,
    pub blur: u32,
    pub traffic_x: u32,
    pub traffic_y: u32,
    pub item_height: u32,
    pub item_radius: u32,
    pub item_padding_x: u32,
    pub item_gap: u32,
    pub item_text_size: u32,
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
    pub item_padding_x: f32,
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
    pub mark_dot: f32,
    pub mark_check: f32,
    pub chevron_size: f32,
    pub chevron_color: Rgba,
    pub icon_size: f32,
    pub header_font_size: f32,
    pub header_weight: i32,
    pub header_height: f32,
    pub search_height: f32,
    pub search_margin_y: f32,
    pub search_ring_width: f32,
    pub search_radius: f32,
    pub search_bg: Rgba,
    pub search_ring: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LockPalette {
    pub bg_surface: Rgba,
    pub ink: Rgba,
    pub ink_muted: Rgba,
    pub danger: Rgba,
    pub accent: Rgba,
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

include!(concat!(env!("OUT_DIR"), "/tengen.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn premultiplied_opaque_is_unchanged() {
        let c = super::Rgba::rgb8(128, 64, 255);
        let [r, g, b, a] = c.premultiplied();
        assert!((r - c.r).abs() < 0.004);
        assert!((g - c.g).abs() < 0.004);
        assert!((b - c.b).abs() < 0.004);
        assert!((a - c.a).abs() < 0.001);
    }

    #[test]
    fn premultiplied_honours_alpha() {
        let c = super::Rgba::rgba8(255, 128, 0, 0.5);
        let [r, g, b, a] = c.premultiplied();
        assert!((r - 0.5).abs() < 0.004, "red should be halved");
        assert!((g - 0.25).abs() < 0.004, "green should be halved");
        assert!(b < 0.004);
        assert!((a - 0.5).abs() < 0.001);
    }
}
