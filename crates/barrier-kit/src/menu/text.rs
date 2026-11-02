use super::*;

pub const MNEMONIC_MARK: char = '\u{2063}';

pub fn strip_mnemonic(label: &str) -> String {
    label.replace(MNEMONIC_MARK, "")
}

pub(super) fn item_label(text: &str, show_mnemonic: bool, size: f32, color: Rgba) -> Element {
    let split = text.split_once(MNEMONIC_MARK).and_then(|(before, rest)| {
        let letter = rest.chars().next()?;
        Some((before, letter, &rest[letter.len_utf8()..]))
    });
    match split {
        Some((before, letter, after)) if show_mnemonic => paragraph()
            .font_size(size)
            .color(theme::color(color))
            .max_lines(1)
            .span(Span::new(before.to_string()))
            .span(Span::new(letter.to_string()).text_decoration(TextDecoration::Underline))
            .span(Span::new(strip_mnemonic(after)))
            .into_element(),
        _ => label()
            .text(strip_mnemonic(text))
            .font_size(size)
            .color(theme::color(color))
            .max_lines(1)
            .into_element(),
    }
}

pub(super) fn font_families() -> Vec<Cow<'static, str>> {
    let mut families: Vec<Cow<'static, str>> = SANS_FAMILIES
        .iter()
        .map(|name| Cow::Borrowed(*name))
        .collect();
    families.extend(default_fonts());
    families
}

pub fn format_shortcut(text: &str) -> String {
    let text = text.trim();
    let (mut ctrl, mut alt, mut shift, mut sup) = (false, false, false, false);
    let mut key = String::new();
    for token in text.split('+').map(str::trim).filter(|t| !t.is_empty()) {
        match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" | "option" | "opt" => alt = true,
            "shift" => shift = true,
            "super" | "meta" | "cmd" | "command" => sup = true,
            "escape" | "esc" => key.push('⎋'),
            "left" => key.push('←'),
            "right" => key.push('→'),
            "up" => key.push('↑'),
            "down" => key.push('↓'),
            "return" | "enter" => key.push('↩'),
            "tab" => key.push('⇥'),
            _ => key.push_str(token),
        }
    }
    let mut out = String::new();
    if ctrl {
        out.push('⌃');
    }
    if alt {
        out.push('⌥');
    }
    if shift {
        out.push('⇧');
    }
    if sup {
        out.push('❖');
    }
    out.push_str(&key);
    out
}

pub fn measure_text(text: &str, size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let mut paragraph_style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(size);
    text_style.set_font_families(&font_families());
    paragraph_style.set_text_style(&text_style);

    let mut builder = ParagraphBuilder::new(&paragraph_style, crate::fonts::collection());
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(f32::MAX);
    paragraph.max_intrinsic_width()
}

pub(super) fn truncate_middle(text: &str, max_width: f32, font_size: f32) -> String {
    if max_width <= 0.0 || text.is_empty() || measure_text(text, font_size) <= max_width {
        return text.to_string();
    }
    let ellipsis = "…";
    let ellipsis_w = measure_text(ellipsis, font_size);
    let max_body = (max_width - ellipsis_w).max(0.0);
    let chars: Vec<char> = text.chars().collect();
    let mut left = chars.len() / 2;
    let mut right = chars.len() - left;
    while left > 0 || right > 0 {
        let body: String = chars
            .iter()
            .take(left)
            .chain(chars.iter().skip(chars.len() - right))
            .collect();
        if measure_text(&body, font_size) <= max_body {
            return format!("{body}{ellipsis}");
        }
        if left >= right && left > 0 {
            left -= 1;
        } else {
            right = right.saturating_sub(1);
        }
    }
    ellipsis.to_string()
}
