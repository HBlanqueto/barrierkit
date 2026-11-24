use super::*;

pub const MNEMONIC_MARK: char = '\u{2063}';

// shortcut: cleared whole when full; a menu needs a few hundred widths, raise it if long menus thrash.
const REMEMBERED_WIDTHS: usize = 1024;

thread_local! {
    static WIDTHS: RefCell<HashMap<(String, u32, FontWeight), f32>> = RefCell::new(HashMap::new());
}

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
    WIDTHS.with(|widths| {
        let key = (text.to_owned(), size.to_bits(), weight);
        remembered(&mut widths.borrow_mut(), key, || {
            laid_out(text, size, weight, f32::MAX).max_intrinsic_width()
        })
    })
}

pub(crate) fn forget_widths() {
    WIDTHS.with(|widths| widths.borrow_mut().clear());
}

fn remembered(
    widths: &mut HashMap<(String, u32, FontWeight), f32>,
    key: (String, u32, FontWeight),
    measure: impl FnOnce() -> f32,
) -> f32 {
    if widths.len() >= REMEMBERED_WIDTHS && !widths.contains_key(&key) {
        widths.clear();
    }
    *widths.entry(key).or_insert_with(measure)
}

pub fn measure_wrapped(text: &str, size: f32, weight: FontWeight, width: f32) -> (f32, usize) {
    let paragraph = laid_out(text, size, weight, width);
    (paragraph.longest_line(), paragraph.line_number())
}

fn laid_out(text: &str, size: f32, weight: FontWeight, width: f32) -> Paragraph {
    let mut paragraph_style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(size);
    text_style.set_font_style(FontStyle::new(weight.into(), Width::NORMAL, Slant::Upright));
    text_style.set_font_families(&font_families());
    paragraph_style.set_text_style(&text_style);

    let mut builder = ParagraphBuilder::new(&paragraph_style, crate::fonts::collection());
    builder.add_text(text);
    let mut paragraph = builder.build();
    paragraph.layout(width);
    paragraph
}

pub(super) fn truncate_middle(text: &str, max_width: f32, font_size: f32) -> String {
    if max_width <= 0.0 || text.is_empty() || measure_text(text, font_size) <= max_width {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let shortened = |kept: usize| -> String {
        let head = kept.div_ceil(2);
        let tail = chars.len() - (kept - head);
        chars[..head]
            .iter()
            .chain(&['…'])
            .chain(&chars[tail..])
            .collect()
    };
    let (mut fits, mut overflows) = (0, chars.len());
    while overflows - fits > 1 {
        let kept = (fits + overflows) / 2;
        if measure_text(&shortened(kept), font_size) <= max_width {
            fits = kept;
        } else {
            overflows = kept;
        }
    }
    shortened(fits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembered_widths_measure_each_text_once_and_stay_bounded() {
        let key = |text: usize| (text.to_string(), 13f32.to_bits(), FontWeight::NORMAL);
        let mut widths = HashMap::new();
        for text in 0..REMEMBERED_WIDTHS {
            remembered(&mut widths, key(text), || 1.0);
        }
        assert_eq!(remembered(&mut widths, key(0), || 2.0), 1.0);
        assert_eq!(remembered(&mut widths, key(REMEMBERED_WIDTHS), || 3.0), 3.0);
        assert_eq!(widths.len(), 1);
    }
}
