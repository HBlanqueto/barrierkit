use super::*;

#[derive(Clone, Copy)]
pub(super) struct Color {
    pub(super) r: u8,
    pub(super) g: u8,
    pub(super) b: u8,
    pub(super) a: u8,
}

pub(super) fn color(root: &Value, path: &str, file: &str) -> Color {
    let text = string(root, path, file);
    parse_color(text, path, file)
}

pub(super) fn parse_color(text: &str, path: &str, file: &str) -> Color {
    let hex = text
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("vow-design: `{path}` in {file} is not a #rrggbb(aa) colour"));
    let channel = |index: usize| -> u8 {
        u8::from_str_radix(&hex[index..index + 2], 16).unwrap_or_else(|_| {
            panic!("vow-design: `{path}` in {file} has an invalid colour `{text}`")
        })
    };
    match hex.len() {
        6 => Color {
            r: channel(0),
            g: channel(2),
            b: channel(4),
            a: 255,
        },
        8 => Color {
            r: channel(0),
            g: channel(2),
            b: channel(4),
            a: channel(6),
        },
        _ => panic!("vow-design: `{path}` in {file} has an invalid colour `{text}`"),
    }
}

pub(super) fn rgba(color: Color) -> String {
    if color.a == 255 {
        format!(
            "Rgba::rgb8(0x{:02x}, 0x{:02x}, 0x{:02x})",
            color.r, color.g, color.b
        )
    } else {
        format!(
            "Rgba::rgba8(0x{:02x}, 0x{:02x}, 0x{:02x}, {:.6})",
            color.r,
            color.g,
            color.b,
            f64::from(color.a) / 255.0
        )
    }
}

pub(super) fn theme_color(tokens: &Value, theme: &str, key: &str) -> Color {
    color(tokens, &format!("themes.{theme}.{key}"), TOKENS)
}

pub(super) fn opaque_over(color: Color, bg: Color) -> Color {
    if color.a == 255 {
        return color;
    }
    let alpha = f32::from(color.a) / 255.0;
    let blend =
        |fg: u8, back: u8| (f32::from(fg) * alpha + f32::from(back) * (1.0 - alpha)).round() as u8;
    Color {
        r: blend(color.r, bg.r),
        g: blend(color.g, bg.g),
        b: blend(color.b, bg.b),
        a: 255,
    }
}

pub(super) fn default_theme(tokens: &Value) -> &str {
    string(tokens, "default-theme", TOKENS)
}

pub(super) fn color_ref(tokens: &Value, theme: &str, text: &str, path: &str, file: &str) -> Color {
    let name = text
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(text);
    if value(tokens, &format!("themes.{theme}"), TOKENS)
        .get(name)
        .is_some()
    {
        theme_color(tokens, theme, name)
    } else {
        parse_color(text, path, file)
    }
}

pub(super) fn shadow(tokens: &Value, theme: &str) -> String {
    let path = format!("shadow.layers.{theme}");
    let layer = |name: &str| {
        let path = format!("{path}.{name}");
        format!(
            "ShadowLayer {{ blur: {}, y_offset: {}, opacity: {}, color: {} }}",
            integer(tokens, &format!("{path}.blur"), TOKENS),
            integer(tokens, &format!("{path}.y-offset"), TOKENS),
            float(number(tokens, &format!("{path}.opacity"), TOKENS)),
            rgba(color(tokens, &format!("{path}.color"), TOKENS)),
        )
    };
    let scale = |key: &str| number(tokens, &format!("shadow.unfocused.{key}"), TOKENS);
    format!(
        "ShadowStyle {{\n\
         \x20   ambient: {},\n\
         \x20   contact: {},\n\
         \x20   unfocused_blur_scale: {},\n\
         \x20   unfocused_offset_scale: {},\n\
         \x20   unfocused_opacity_scale: {},\n\
         }}",
        layer("ambient"),
        layer("contact"),
        scale("blur"),
        scale("offset"),
        scale("opacity"),
    )
}
