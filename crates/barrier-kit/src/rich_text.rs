use std::ops::Range;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub link: Option<String>,
    pub family: Option<String>,
    pub size: Option<f32>,
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
    pub style: TextStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Emphasis {
    Bold,
    Italic,
}

pub fn char_styles(runs: &[StyleRun], length: usize) -> Vec<TextStyle> {
    let mut styles = vec![TextStyle::default(); length];
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

pub fn runs_from(styles: &[TextStyle]) -> Vec<StyleRun> {
    let mut runs: Vec<StyleRun> = Vec::new();
    for (index, style) in styles.iter().enumerate() {
        if *style == TextStyle::default() {
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
        .map(|index| TextStyle {
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
    let has = |style: &TextStyle| match emphasis {
        Emphasis::Bold => style.bold,
        Emphasis::Italic => style.italic,
    };
    let on = !styles[range.clone()].iter().all(has);
    for style in &mut styles[range] {
        match emphasis {
            Emphasis::Bold => style.bold = on,
            Emphasis::Italic => style.italic = on,
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
            .unwrap_or_else(|| vow_design::TEXT.family.to_owned()),
        size: style.size.unwrap_or(vow_design::TEXT.size as f32),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bold(range: Range<usize>) -> StyleRun {
        StyleRun {
            range,
            style: TextStyle {
                bold: true,
                ..TextStyle::default()
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
            style: TextStyle {
                link: Some("https://example.org".into()),
                ..TextStyle::default()
            },
        }];
        assert_eq!(link_at(&runs, 2), Some("https://example.org"));
        assert_eq!(link_at(&runs, 5), None);
    }

    #[test]
    fn typing_after_a_link_does_not_extend_it() {
        let link = StyleRun {
            range: 0..3,
            style: TextStyle {
                link: Some("https://example.org".into()),
                ..TextStyle::default()
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
        assert_eq!(font_at(&runs, 6, 0).size, vow_design::TEXT.size as f32);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].range, 2..4);
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
