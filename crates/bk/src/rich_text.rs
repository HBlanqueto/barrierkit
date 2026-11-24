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
    let body = &rest[rest.chars().next()?.len_utf8()..];
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
