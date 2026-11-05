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

pub fn measure_text(text: &str, size: f32) -> f32 {
    measure_text_weight(text, size, FontWeight::NORMAL)
}

pub fn measure_text_weight(text: &str, size: f32, weight: FontWeight) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let mut paragraph_style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(size);
    text_style.set_font_style(FontStyle::new(weight.into(), Width::NORMAL, Slant::Upright));
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
