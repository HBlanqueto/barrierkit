//! Design constants: the single place for radii, shadows, colours and margins.

#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
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

    pub fn relative_luminance(self) -> f32 {
        fn linear(channel: f32) -> f32 {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
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
pub struct FocusPair {
    pub focused: Rgba,
    pub unfocused: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarPosition {
    Left,
    Right,
    Top,
    Bottom,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleRotation {
    Horizontal,
    BottomToTop,
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
    pub margin_start: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Titlebar {
    pub position: TitlebarPosition,
    pub thickness: u32,
    pub buttons: TitlebarButtons,
    pub font: Font,
    pub rotation: TitleRotation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowStyle {
    pub corner_radius: u32,
    pub superellipse_exponent: f32,
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
    pub tick: u32,
    pub label_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsCheck {
    pub gap: u32,
    pub column_gap: u32,
    pub row_gap: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsMetrics {
    pub label_column: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub window_min_height: u32,
    pub search_height: u32,
    pub search_padding_x: u32,
    pub search_icon: u32,
    pub search_gap: u32,
    pub account_avatar: u32,
    pub account_gap: u32,
    pub account_height: u32,
    pub account_padding_x: u32,
    pub account_name_font: Font,
    pub account_detail_font: Font,
    pub section_gap: u32,
    pub section_icon_bg: Rgba,
    pub group_radius: u32,
    pub group_row_min_height: u32,
    pub group_row_padding_x: u32,
    pub group_row_padding_y: u32,
    pub group_title_margin_top: u32,
    pub group_content_padding_x: u32,
    pub swatch_size: u32,
    pub swatch_gap: u32,
    pub swatch_name_font: Font,
    pub card_width: u32,
    pub card_height: u32,
    pub card_radius: u32,
    pub card_gap: u32,
    pub card_ring: u32,
    pub card_ring_gap: u32,
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
    pub preview_shadow: &'static str,
    pub radio_size: u32,
    pub radio_dot: u32,
    pub radio_gap: u32,
    pub radio_spacing: u32,
    pub popup_height: u32,
    pub popup_chevron: u32,
    pub popup_light_gap: u32,
    pub popup_text_gap: u32,
    pub selected_dark_mix: f32,
    pub sidebar_lights_gap: u32,
    pub sidebar_search_gap: u32,
    pub sidebar_item_gap: u32,
    pub sidebar_padding_top: u32,
    pub sidebar_padding_bottom: u32,
    pub sidebar_app_icon_glyph: u32,
    pub row_gap: u32,
    pub strength: SettingsStrength,
    pub check: SettingsCheck,
    pub dropdown_preview_light: u32,
    pub dropdown_chevron_glyph: u32,
    pub empty_padding_y: u32,
    pub empty_icon: u32,
    pub empty_gap: u32,
    pub toolbar_gap: u32,
    pub toolbar_height: u32,
    pub toolbar_padding_top: u32,
    pub toolbar_padding_x: u32,
    pub content_padding_top: u32,
    pub content_padding_bottom: u32,
    pub content_gap: u32,
    pub group_title_margin_bottom: u32,
    pub group_title_indent: u32,
    pub group_desc_gap: u32,
    pub swatch_column: u32,
    pub swatch_name_gap: u32,
    pub card_outline: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderMetrics {
    pub height: f32,
    pub track: f32,
    pub knob: f32,
    pub track_color: Rgba,
    pub fill_color: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxMetrics {
    pub size: f32,
    pub radius: f32,
    pub border: Rgba,
    pub on: Rgba,
    pub check_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub knob: f32,
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextFieldMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarMetrics {
    pub default_island: bool,
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
    pub count_size: u32,
    pub selected_weight: u16,
    pub item_icon: u32,
    pub app_icon: u32,
    pub section_height: u32,
    pub section_margin_top: u32,
    pub section_padding_x: u32,
    pub section_size: u32,
    pub section_weight: u16,
    pub chevron: u32,
    pub chevron_stroke: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupMetrics {
    pub height: u32,
    pub padding_x: u32,
    pub button_w: u32,
    pub button_h: u32,
    pub separator_w: u32,
    pub separator_h: u32,
    pub flat_button_w: u32,
    pub flat_button_h: u32,
    pub flat_radius: u32,
    pub icon: u32,
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
pub struct Palette {
    pub titlebar_bg: FocusPair,
    pub hairline: Rgba,
    pub inner_line: Option<Rgba>,
    pub outer_border: Rgba,
    pub title: FocusPair,
    pub text: FocusPair,
    pub ink_strong: Rgba,
    pub ink_muted: Rgba,
    pub ink_faint: Rgba,
    pub line: Rgba,
    pub line_strong: Rgba,
    pub accent: Rgba,
    pub on_accent: Rgba,
    pub danger: Rgba,
    pub traffic_lights_inactive: Rgba,
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
    pub item_gap: f32,
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
    pub submenu_offset: f32,
    pub fg: Rgba,
    pub shortcut_color: Rgba,
    pub hover_bg: Rgba,
    pub hover_fg: Rgba,
    pub bar_hover_bg: Rgba,
    pub bar_hover_fg: Rgba,
    pub bar_title_open_bg: Rgba,
    pub bar_title_open_radius: f32,
    pub bar_title_open_height: f32,
    pub disabled_fg: Rgba,
    pub separator_color: Rgba,
    pub separator_height: f32,
    pub separator_margin_x: f32,
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
    pub hover_bg: Rgba,
    pub section_ink: Rgba,
    pub count_ink: Rgba,
    pub line: Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolgroupPalette {
    pub ink: Rgba,
    pub disabled_ink: Rgba,
    pub separator: Rgba,
}

pub const CURSOR_SHAKE_SCALE: f32 = 3.0;

pub const CURSOR_SHAKE_GROW_MS: u64 = 180;

pub const CURSOR_SHAKE_SHRINK_MS: u64 = 220;

pub const CURSOR_SHAKE_TICK_MS: u64 = 16;

pub const SCREENSHOT_DIM: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

pub const SCREENSHOT_IDLE_DIM: [f32; 4] = [0.0, 0.0, 0.0, 0.15];

pub const SCREENSHOT_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 0.9];

pub const SCREENSHOT_BORDER_WIDTH: i32 = 1;

pub const SPACE_GAP: i32 = 48;

pub const SPACE_GAP_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub const SPACE_SLIDE_MS: u64 = 480;

pub const FOCUS_BAR_BACKGROUND: Rgba = Rgba::rgb8(0, 0, 0);

pub const FOCUS_BAR_TEXT: Rgba = Rgba::rgb8(255, 255, 255);

pub const SPACE_SLIDE_EASE: [f32; 4] = [0.32, 0.72, 0.0, 1.0];

pub const WALLPAPER_DIR: &str = "/run/current-system/sw/share/wallpapers/solid-colors";

pub const NATIVE_APP_ID_PREFIX: &str = "io.github.hblanqueto.malaysia.";

pub const WALLPAPER: &str = "Solid Aqua Blue.png";

include!(concat!(env!("OUT_DIR"), "/tengen.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn traffic_glyphs_have_the_expected_shapes() {
        assert_eq!(super::TRAFFIC_GLYPHS.close.len(), 2);
        assert_eq!(super::TRAFFIC_GLYPHS.minimize.len(), 1);
        assert_eq!(super::TRAFFIC_GLYPHS.zoom.len(), 2);
        assert_eq!(super::TRAFFIC_GLYPHS.plus.len(), 2);
    }

    #[test]
    fn generated_shadows_match_the_tokens() {
        let tokens: serde_json::Value = serde_json::from_str(include_str!("../tengen/tokens.json"))
            .expect("tokens.json should parse");

        for (style, theme) in [(super::SHADOW, "light"), (super::SHADOW_DARK, "dark")] {
            for (layer, name) in [(style.ambient, "ambient"), (style.contact, "contact")] {
                let json = &tokens["shadow"]["layers"][theme][name];
                assert_eq!(u64::from(layer.blur), json["blur"].as_u64().unwrap());
                assert_eq!(
                    i64::from(layer.y_offset),
                    json["y-offset"].as_i64().unwrap()
                );
                assert!(
                    (f64::from(layer.opacity) - json["opacity"].as_f64().unwrap()).abs() < 1e-6
                );
                let hex = json["color"].as_str().unwrap().trim_start_matches('#');
                let expected = [
                    u8::from_str_radix(&hex[0..2], 16).unwrap(),
                    u8::from_str_radix(&hex[2..4], 16).unwrap(),
                    u8::from_str_radix(&hex[4..6], 16).unwrap(),
                ];
                assert_eq!(layer.color.to_rgba8()[..3], expected);
            }
        }

        let unfocused = &tokens["shadow"]["unfocused"];
        let scales = [
            (
                f64::from(super::SHADOW.unfocused_blur_scale),
                unfocused["blur"].as_f64().unwrap(),
            ),
            (
                f64::from(super::SHADOW.unfocused_offset_scale),
                unfocused["offset"].as_f64().unwrap(),
            ),
            (
                f64::from(super::SHADOW.unfocused_opacity_scale),
                unfocused["opacity"].as_f64().unwrap(),
            ),
        ];
        for (generated, json) in scales {
            assert!((generated - json).abs() < 1e-6);
        }

        let [ambient, contact] = super::SHADOW.layers(false);
        assert_eq!(
            ambient.blur,
            (super::SHADOW.ambient.blur as f32 * super::SHADOW.unfocused_blur_scale).round() as u32
        );
        assert_eq!(
            contact.y_offset,
            (super::SHADOW.contact.y_offset as f32 * super::SHADOW.unfocused_offset_scale).round()
                as i32
        );
    }

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

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn sidebar_metrics_come_from_tokens() {
        assert_eq!(super::SIDEBAR.inset, 8);
        assert_eq!(super::SIDEBAR.padding_x, 10);
        assert_eq!(super::SIDEBAR.radius, 12);
        assert_eq!(super::SIDEBAR.width, 220);
        assert!(super::SIDEBAR.default_island);
        assert_eq!(super::SIDEBAR.item_padding_x, 8);
        assert_eq!(super::SIDEBAR.item_gap, 8);
        assert_eq!(super::SIDEBAR.item_text_size, 13);
        assert_eq!(super::SIDEBAR.section_padding_x, 8);
        assert!((super::SIDEBAR.chevron_stroke - 1.8).abs() < 1e-6);
        assert_eq!(super::TOOLGROUP.height, 34);
        assert_eq!(super::TOOLGROUP.padding_x, 3);
        assert_eq!(super::TOOLGROUP.flat_button_w, 30);
        assert_eq!(super::TOOLGROUP.flat_button_h, 28);
        assert_eq!(super::TOOLGROUP.flat_radius, 8);
        assert_eq!(super::TOOLGROUP.icon, 16);
    }

    #[test]
    fn settings_metrics_and_swatches_match_components() {
        assert_eq!(super::SETTINGS.label_column, 150);
        assert_eq!(super::SETTINGS.window_width, 760);
        assert_eq!(super::SETTINGS.window_height, 620);
        assert_eq!(super::SETTINGS.window_min_height, 480);
        assert_eq!(super::SETTINGS.search_height, 30);
        assert_eq!(super::SETTINGS.search_padding_x, 10);
        assert_eq!(super::SETTINGS.search_icon, 16);
        assert_eq!(super::SETTINGS.account_avatar, 36);
        assert_eq!(super::SETTINGS.account_gap, 10);
        assert_eq!(super::SETTINGS.account_name_font, super::HEADLINE);
        assert_eq!(super::SETTINGS.account_detail_font, super::CALLOUT);
        assert_eq!(super::SETTINGS.section_gap, 14);
        assert_eq!(
            super::SETTINGS.section_icon_bg,
            super::Rgba::rgb8(0x2b, 0x2b, 0x2b)
        );
        assert_eq!(super::SETTINGS.group_radius, 16);
        assert_eq!(super::SETTINGS.group_row_min_height, 44);
        assert_eq!(super::SETTINGS.group_row_padding_x, 14);
        assert_eq!(super::SETTINGS.group_row_padding_y, 10);
        assert_eq!(super::SETTINGS.group_title_margin_top, 16);
        assert_eq!(super::SETTINGS.group_content_padding_x, 16);
        assert_eq!(super::SETTINGS.swatch_size, 20);
        assert_eq!(super::SETTINGS.swatch_gap, 9);
        assert_eq!(super::SETTINGS.swatch_name_font, super::CAPTION);
        assert_eq!(super::SETTINGS.card_width, 70);
        assert_eq!(super::SETTINGS.card_height, 46);
        assert_eq!(super::SETTINGS.card_radius, 8);
        assert_eq!(super::SETTINGS.card_gap, 10);
        assert_eq!(super::SETTINGS.card_ring, 2);
        assert_eq!(super::SETTINGS.card_ring_gap, 2);
        assert_eq!(super::SETTINGS.card_caption_gap, 5);
        assert_eq!(super::SETTINGS.card_caption_font, super::CALLOUT);
        assert_eq!(super::SETTINGS.preview_window_radius, 5);
        assert_eq!(super::SETTINGS.preview_light, 5);
        assert_eq!(super::SETTINGS.preview_tint_window_x, 10);
        assert_eq!(super::SETTINGS.preview_tint_window_y, 9);
        assert_eq!(super::SETTINGS.preview_tint_side_w, 14);
        assert_eq!(super::SETTINGS.preview_panel_h, 4);
        assert_eq!(super::SETTINGS.preview_panel_w, 35);
        assert_eq!(super::SETTINGS.preview_menu_x, 5);
        assert_eq!(super::SETTINGS.preview_menu_y, 6);
        assert_eq!(super::SETTINGS.preview_menu_w, 26);
        assert_eq!(super::SETTINGS.preview_menu_h, 20);
        assert_eq!(super::SETTINGS.preview_menu_padding, 2);
        assert_eq!(super::SETTINGS.preview_menu_line_h, 2);
        assert_eq!(super::SETTINGS.preview_menu_line_radius, 1);
        assert_eq!(super::SETTINGS.preview_app_w, 32);
        assert_eq!(super::SETTINGS.preview_app_h, 21);
        assert_eq!(super::SETTINGS.preview_app_inset, 5);
        assert_eq!(super::SETTINGS.preview_app_side_w, 9);
        assert_eq!(super::SETTINGS.preview_dots_inset, 2);
        assert_eq!(super::SETTINGS.preview_dot_gap, 1);
        assert_eq!(super::SETTINGS.preview_dot_radius, 1);
        assert_eq!(super::SETTINGS.preview_radius, 3);
        assert_eq!(super::SETTINGS.preview_wall_angle, 160);
        assert!((super::SETTINGS.preview_wall_light - 0.25).abs() < 1e-6);
        assert!((super::SETTINGS.preview_wall_dark - 0.40).abs() < 1e-6);
        assert_eq!(super::SETTINGS.radio_size, 16);
        assert_eq!(super::SETTINGS.radio_dot, 6);
        assert_eq!(super::SETTINGS.radio_gap, 7);
        assert_eq!(super::SETTINGS.radio_spacing, 16);
        assert_eq!(super::SETTINGS.popup_height, 26);
        assert_eq!(super::SETTINGS.popup_chevron, 18);
        assert_eq!(super::SETTINGS.popup_light_gap, 5);
        assert_eq!(super::SETTINGS.popup_text_gap, 6);
        assert!((super::SETTINGS.selected_dark_mix - 0.5).abs() < 1e-6);
        assert_eq!(super::SETTINGS.sidebar_lights_gap, 18);
        assert_eq!(super::SETTINGS.sidebar_search_gap, 12);
        assert_eq!(super::SETTINGS.sidebar_item_gap, 2);
        assert_eq!(super::SETTINGS.toolbar_gap, 12);
        assert_eq!(super::SETTINGS.toolbar_height, 52);
        assert_eq!(super::SETTINGS.toolbar_padding_top, 8);
        assert_eq!(super::SETTINGS.toolbar_padding_x, 14);
        assert_eq!(super::SETTINGS.sidebar_padding_top, 16);
        assert_eq!(super::SETTINGS.sidebar_padding_bottom, 10);
        assert_eq!(super::SETTINGS.account_height, 48);
        assert_eq!(super::SETTINGS.account_padding_x, 6);
        assert_eq!(super::SETTINGS.search_gap, 6);
        assert_eq!(super::SETTINGS.row_gap, 12);
        assert_eq!(super::SETTINGS.strength.width, 200);
        assert_eq!(super::SETTINGS.strength.tick, 4);
        assert_eq!(super::SETTINGS.strength.label_gap, 4);
        assert_eq!(super::SETTINGS.check.gap, 7);
        assert_eq!(super::SETTINGS.check.column_gap, 18);
        assert_eq!(super::SETTINGS.check.row_gap, 8);
        assert_eq!(super::SETTINGS.dropdown_preview_light, 12);
        assert_eq!(super::SETTINGS.dropdown_chevron_glyph, 10);
        assert_eq!(super::SETTINGS.empty_padding_y, 48);
        assert_eq!(super::SETTINGS.empty_icon, 28);
        assert_eq!(super::SETTINGS.empty_gap, 6);
        assert_eq!(super::SETTINGS.sidebar_app_icon_glyph, 14);
        assert_eq!(super::SIDEBAR.selected_weight, 500);
        assert_eq!(super::MENU_POPOVER.padding, 5.0);
        assert_eq!(super::MENU_POPOVER.min_width, 180.0);
        assert_eq!(super::MENU_POPOVER.check, 12.0);
        assert_eq!(super::SETTINGS.content_padding_top, 4);
        assert_eq!(super::SETTINGS.content_padding_bottom, 24);
        assert_eq!(super::SETTINGS.content_gap, 8);
        assert_eq!(super::SETTINGS.group_title_margin_bottom, 2);
        assert_eq!(super::SETTINGS.group_title_indent, 12);
        assert_eq!(super::SETTINGS.group_desc_gap, 2);
        assert_eq!(super::SETTINGS.swatch_column, 24);
        assert_eq!(super::SETTINGS.swatch_name_gap, 5);
        assert_eq!(super::SETTINGS.card_outline.to_rgba8(), [0, 0, 0, 0x40]);
        assert_eq!(super::ACCENT_SWATCHES.len(), 8);
        assert_eq!(super::ACCENT_SWATCHES[0].0, "Azul");
        assert_eq!(
            super::ACCENT_SWATCHES[1].1,
            super::Rgba::rgb8(0x8e, 0x5b, 0xd8)
        );
        assert_eq!(super::SLIDER.height, 20.0);
        assert_eq!(super::SLIDER.track, 4.0);
        assert_eq!(super::SLIDER.knob, 20.0);
        assert_eq!(super::CHECKBOX.size, 16.0);
        assert_eq!(super::CHECKBOX.check_stroke, 2.2);
        assert_eq!(super::HEADLINE.size, 13);
        assert_eq!(super::HEADLINE.weight, 600);
        assert_eq!(super::CALLOUT.size, 12);
        assert_eq!(super::CALLOUT.weight, 400);
        assert_eq!(super::CAPTION.size, 11);
        assert_eq!(super::CAPTION.weight, 500);
    }

    #[test]
    fn sidebar_island_opaque_matches_bg_sunken() {
        assert_eq!(super::SIDEBAR_ISLAND_LIGHT_OPAQUE, super::SIDEBAR_LIGHT);
        assert_eq!(super::SIDEBAR_ISLAND_DARK_OPAQUE, super::SIDEBAR_DARK);
        assert_eq!(
            super::SIDEBAR_ISLAND_DARK_TINT_OPAQUE,
            super::SIDEBAR_DARK_TINT
        );
        assert_eq!(super::SIDEBAR_ISLAND_LIGHT_OPAQUE.a, 1.0);
        assert_eq!(super::SIDEBAR.rim_width, 1.0);
    }

    #[test]
    fn sidebar_island_light_has_no_rim_inner_or_shadow() {
        assert_eq!(super::SIDEBAR_ISLAND_RIM_LIGHT.a, 0.0);
        assert_eq!(super::SIDEBAR_ISLAND_INNER_LIGHT.a, 0.0);
    }

    #[test]
    fn sidebar_island_dark_has_gray_rim() {
        assert_eq!(
            super::SIDEBAR_ISLAND_RIM_DARK.to_rgba8(),
            [0xff, 0xff, 0xff, 0x21]
        );
        assert_eq!(
            super::SIDEBAR_ISLAND_RIM_DARK_TINT.to_rgba8(),
            [0xff, 0xff, 0xff, 0x21]
        );
    }
}
