use std::ops::Range;

use ctdesign::MONO;
use freya_core::prelude::*;

use crate::style::sized_font;
use crate::styles::{Paints, ShapeStyle};
use crate::theme::color;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub link: Option<String>,
    pub family: Option<String>,
    pub size: Option<f32>,
    pub foreground_style: Option<ShapeStyle>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontChoice {
    pub family: String,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRun {
    pub range: Range<usize>,
    pub style: RunStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Emphasis {
    Bold,
    Italic,
    Underline,
}

pub fn char_styles(runs: &[StyleRun], length: usize) -> Vec<RunStyle> {
    let mut styles = vec![RunStyle::default(); length];
    for run in runs {
        for style in styles
            .iter_mut()
            .take(run.range.end.min(length))
            .skip(run.range.start)
        {
            *style = run.style.clone();
        }
    }
    styles
}

pub fn runs_from(styles: &[RunStyle]) -> Vec<StyleRun> {
    let mut runs: Vec<StyleRun> = Vec::new();
    for (index, style) in styles.iter().enumerate() {
        if *style == RunStyle::default() {
            continue;
        }
        match runs.last_mut() {
            Some(run) if run.range.end == index && run.style == *style => run.range.end += 1,
            _ => runs.push(StyleRun {
                range: index..index + 1,
                style: style.clone(),
            }),
        }
    }
    runs
}

pub fn restyle_after_edit(runs: &[StyleRun], old: &str, new: &str) -> Vec<StyleRun> {
    let old: Vec<char> = old.chars().collect();
    let new: Vec<char> = new.chars().collect();
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let mut styles = char_styles(runs, old.len());
    let inherited = prefix
        .checked_sub(1)
        .map(|index| RunStyle {
            link: None,
            ..styles[index].clone()
        })
        .unwrap_or_default();
    let inserted = new.len() - prefix - suffix;
    styles.splice(
        prefix..old.len() - suffix,
        std::iter::repeat_n(inherited, inserted),
    );
    runs_from(&styles)
}

pub fn toggle(
    runs: &[StyleRun],
    length: usize,
    range: Range<usize>,
    emphasis: Emphasis,
) -> Vec<StyleRun> {
    let mut styles = char_styles(runs, length);
    let range = range.start.min(length)..range.end.min(length);
    let has = |style: &RunStyle| match emphasis {
        Emphasis::Bold => style.bold,
        Emphasis::Italic => style.italic,
        Emphasis::Underline => style.underline,
    };
    let on = !styles[range.clone()].iter().all(has);
    for style in &mut styles[range] {
        match emphasis {
            Emphasis::Bold => style.bold = on,
            Emphasis::Italic => style.italic = on,
            Emphasis::Underline => style.underline = on,
        }
    }
    runs_from(&styles)
}

pub fn font_at(runs: &[StyleRun], length: usize, index: usize) -> FontChoice {
    let style = char_styles(runs, length)
        .get(index)
        .cloned()
        .unwrap_or_default();
    FontChoice {
        family: style
            .family
            .unwrap_or_else(|| ctdesign::BODY.family.to_owned()),
        size: style.size.unwrap_or(ctdesign::BODY.size as f32),
        bold: style.bold,
        italic: style.italic,
    }
}

pub fn apply_font(
    runs: &[StyleRun],
    length: usize,
    range: Range<usize>,
    font: &FontChoice,
) -> Vec<StyleRun> {
    let mut styles = char_styles(runs, length);
    for style in &mut styles[range.start.min(length)..range.end.min(length)] {
        style.family = Some(font.family.clone());
        style.size = Some(font.size);
        style.bold = font.bold;
        style.italic = font.italic;
    }
    runs_from(&styles)
}

pub fn link_at(runs: &[StyleRun], index: usize) -> Option<&str> {
    runs.iter()
        .find(|run| run.range.contains(&index))
        .and_then(|run| run.style.link.as_deref())
}

type Delimiter = (&'static str, fn(&mut RunStyle));

const DELIMITERS: [Delimiter; 6] = [
    ("***", |style| {
        style.bold = true;
        style.italic = true;
    }),
    ("**", |style| style.bold = true),
    ("__", |style| style.bold = true),
    ("~~", |style| style.strikethrough = true),
    ("*", |style| style.italic = true),
    ("_", |style| style.italic = true),
];

/// Parses inline Markdown (`**bold**`, `*italic*`, `~~strike~~`, `` `code` ``, `[title](url)`, `\*`) into the plain text and its runs; markers without a closing mate stay literal.
pub fn markdown(source: &str) -> (String, Vec<StyleRun>) {
    let mut text = String::new();
    let mut styles = Vec::new();
    inline(source, &RunStyle::default(), &mut text, &mut styles);
    (text, runs_from(&styles))
}

fn inline(source: &str, base: &RunStyle, text: &mut String, styles: &mut Vec<RunStyle>) {
    let mut rest = source;
    let mut before = None;
    while let Some(first) = rest.chars().next() {
        let used = construct(rest, before, base, text, styles).unwrap_or_else(|| {
            text.push(first);
            styles.push(base.clone());
            first.len_utf8()
        });
        before = rest[..used].chars().last();
        rest = &rest[used..];
    }
}

fn push(text: &mut String, styles: &mut Vec<RunStyle>, content: &str, style: &RunStyle) {
    text.push_str(content);
    styles.extend(content.chars().map(|_| style.clone()));
}

// The bytes of `rest` a construct takes, after it wrote the construct's text and styles.
fn construct(
    rest: &str,
    before: Option<char>,
    base: &RunStyle,
    text: &mut String,
    styles: &mut Vec<RunStyle>,
) -> Option<usize> {
    let body = &rest[1..];
    match rest.chars().next()? {
        '\\' => {
            let escaped = body.chars().next().filter(char::is_ascii_punctuation)?;
            push(text, styles, &escaped.to_string(), base);
            return Some(1 + escaped.len_utf8());
        }
        '`' => {
            let end = body.find('`').filter(|end| *end > 0)?;
            let code = RunStyle {
                // Core Text has no "monospace" alias and Geist Mono is not installed on a Mac: Menlo is.
                family: Some(format!("Menlo, {}", MONO.family)),
                ..base.clone()
            };
            push(text, styles, &body[..end], &code);
            return Some(end + 2);
        }
        '[' => {
            let title_end = body.find("](")?;
            let url_end = title_end + 2 + body[title_end + 2..].find(')')?;
            let link = RunStyle {
                link: Some(body[title_end + 2..url_end].to_owned()),
                ..base.clone()
            };
            inline(&body[..title_end], &link, text, styles);
            return Some(url_end + 2);
        }
        _ => {}
    }
    DELIMITERS.iter().find_map(|(delimiter, apply)| {
        let inner = rest.strip_prefix(delimiter)?;
        let underscore = delimiter.starts_with('_');
        if inner.is_empty()
            || inner.starts_with(char::is_whitespace)
            || underscore && before.is_some_and(char::is_alphanumeric)
        {
            return None;
        }
        let close = closing(inner, delimiter)?;
        let mut style = base.clone();
        apply(&mut style);
        inline(&inner[..close], &style, text, styles);
        Some(delimiter.len() + close + delimiter.len())
    })
}

// A closing mate follows a non-space and, for a single mark or an underscore, is not part of a longer run.
fn closing(inner: &str, delimiter: &str) -> Option<usize> {
    let mark = delimiter.chars().next()?;
    inner
        .match_indices(delimiter)
        .map(|(at, _)| at)
        .find(|&at| {
            let after = &inner[at + delimiter.len()..];
            let before = inner[..at].chars().last();
            at > 0
                && before.is_some_and(|c| !c.is_whitespace())
                && (delimiter.len() > 1 && mark != '_'
                    || before != Some(mark) && !after.starts_with(mark))
                && (mark != '_' || !after.starts_with(char::is_alphanumeric))
        })
}

/// One Freya span for each run of equal style, with the link of the run; `link` colors links.
pub(crate) fn styled_spans(
    text: &str,
    runs: &[StyleRun],
    paints: &Paints,
    link: ctdesign::Rgba,
) -> Vec<(Span<'static>, Option<String>)> {
    if runs.is_empty() {
        return vec![(Span::new(text.to_owned()), None)];
    }
    let chars: Vec<char> = text.chars().collect();
    let styles = char_styles(runs, chars.len());
    let mut spans = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let style = &styles[start];
        let end = (start..chars.len())
            .find(|&index| styles[index] != *style)
            .unwrap_or(chars.len());
        let mut span = Span::new(chars[start..end].iter().collect::<String>());
        if style.bold {
            span = span.font_weight(FontWeight::BOLD);
        }
        if style.italic {
            span = span.font_slant(FontSlant::Italic);
        }
        let families: Vec<&str> = style
            .family
            .iter()
            .flat_map(|list| list.split(','))
            .map(str::trim)
            .collect();
        if let Some(size) = style.size {
            let first = families.first().copied().unwrap_or(ctdesign::BODY.family);
            span = sized_font(span, first, size);
        }
        for family in families {
            span = span.font_family(family.to_owned());
        }
        if let Some(foreground) = &style.foreground_style {
            span = span.color(color(paints.color(foreground)));
        }
        if style.link.is_some() {
            span = span.color(color(link));
        }
        if style.link.is_some() || style.underline {
            span = span.text_decoration(TextDecoration::Underline);
        } else if style.strikethrough {
            span = span.text_decoration(TextDecoration::LineThrough);
        }
        spans.push((span, style.link.clone()));
        start = end;
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bold(range: Range<usize>) -> StyleRun {
        StyleRun {
            range,
            style: RunStyle {
                bold: true,
                ..RunStyle::default()
            },
        }
    }

    #[test]
    fn typing_at_the_end_of_a_bold_run_continues_it_and_text_before_shifts_it() {
        assert_eq!(
            restyle_after_edit(&[bold(0..4)], "hola", "holas"),
            [bold(0..5)]
        );
        assert_eq!(
            restyle_after_edit(&[bold(2..4)], "hola", "¡hola"),
            [bold(3..5)]
        );
        assert_eq!(
            restyle_after_edit(&[bold(0..4)], "hola mundo", "ho mundo"),
            [bold(0..2)]
        );
    }

    #[test]
    fn a_link_covers_its_characters_and_not_the_one_after() {
        let runs = [StyleRun {
            range: 2..5,
            style: RunStyle {
                link: Some("https://example.org".into()),
                ..RunStyle::default()
            },
        }];
        assert_eq!(link_at(&runs, 2), Some("https://example.org"));
        assert_eq!(link_at(&runs, 5), None);
    }

    #[test]
    fn typing_after_a_link_does_not_extend_it() {
        let link = StyleRun {
            range: 0..3,
            style: RunStyle {
                link: Some("https://example.org".into()),
                ..RunStyle::default()
            },
        };
        assert_eq!(
            restyle_after_edit(std::slice::from_ref(&link), "web", "web!"),
            [link]
        );
    }

    #[test]
    fn a_font_applies_only_to_its_range_and_reads_back_from_it() {
        let font = FontChoice {
            family: "Serif".into(),
            size: 24.0,
            bold: true,
            italic: false,
        };
        let runs = apply_font(&[], 6, 2..4, &font);
        assert_eq!(font_at(&runs, 6, 3), font);
        assert_eq!(font_at(&runs, 6, 0).size, ctdesign::BODY.size as f32);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].range, 2..4);
    }

    #[test]
    fn typing_after_underlined_text_continues_the_underline() {
        let underlined = StyleRun {
            range: 0..2,
            style: RunStyle {
                underline: true,
                ..RunStyle::default()
            },
        };
        assert_eq!(
            toggle(&[], 2, 0..2, Emphasis::Underline),
            std::slice::from_ref(&underlined)
        );
        assert_eq!(
            restyle_after_edit(&[underlined], "ab", "abc")[0].range,
            0..3
        );
    }

    #[test]
    fn markdown_reads_emphasis_code_strike_and_links_into_runs() {
        let (text, runs) = markdown("a **b** _c_ `d` ~~e~~ [f](https://x.org)");
        assert_eq!(text, "a b c d e f");
        let styles = char_styles(&runs, text.chars().count());
        assert!(styles[2].bold && !styles[2].italic);
        assert!(styles[4].italic);
        assert!(
            styles[6]
                .family
                .as_deref()
                .is_some_and(|list| list.contains(MONO.family))
        );
        assert!(styles[8].strikethrough);
        assert_eq!(link_at(&runs, 10), Some("https://x.org"));
        assert_eq!(link_at(&runs, 1), None);
    }

    #[test]
    fn markdown_keeps_marks_without_a_mate_snake_case_and_escapes_literal() {
        for source in ["2 * 3 * 4", "snake_case_name", "**open", "a ` b", "[x](y"] {
            assert_eq!(markdown(source), (source.to_owned(), vec![]), "{source}");
        }
        assert_eq!(markdown("\\*x\\*"), ("*x*".to_owned(), vec![]));
    }

    #[test]
    fn markdown_nests_marks_and_takes_bold_italic_from_three_stars() {
        let (text, runs) = markdown("***x*** **a *b* c**");
        assert_eq!(text, "x a b c");
        let styles = char_styles(&runs, 7);
        assert!(styles[0].bold && styles[0].italic);
        assert!(styles[2].bold && !styles[2].italic);
        assert!(styles[4].bold && styles[4].italic);
    }

    #[test]
    fn toggling_bolds_a_mixed_range_and_unbolds_a_fully_bold_one() {
        assert_eq!(toggle(&[bold(0..2)], 6, 0..4, Emphasis::Bold), [bold(0..4)]);
        assert_eq!(
            toggle(&[bold(0..4)], 6, 1..3, Emphasis::Bold),
            [bold(0..1), bold(3..4)]
        );
    }
}
