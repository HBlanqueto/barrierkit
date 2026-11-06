use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashSet;
use std::io::Write;
use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Once, OnceLock};

use freya_core::data::LayoutData;
use freya_core::element::{ElementExt, RenderContext};
use freya_core::elements::paragraph::{ParagraphElement, ParagraphHolderInner};
use freya_core::node_id::NodeId;
use freya_core::prelude::*;
use freya_core::tree::{DiffModifies, Tree};
use freya_engine::prelude::{Paint, PaintStyle, Paragraph as SkParagraph};
use spellbook::Dictionary;
use vow_design::{Rgba, SPELLING};

static DICTIONARY: OnceLock<Option<Dictionary>> = OnceLock::new();
static PERSONAL: OnceLock<(PathBuf, HashSet<String>)> = OnceLock::new();

thread_local! {
    static ACCEPTED: State<HashSet<String>> = State::create_global(HashSet::new());
}

pub(crate) fn dictionary() -> Option<&'static Dictionary> {
    static LOAD: Once = Once::new();
    LOAD.call_once(|| {
        std::thread::spawn(|| {
            let _ = DICTIONARY.set(load());
        });
    });
    DICTIONARY.get()?.as_ref()
}

fn load() -> Option<Dictionary> {
    let dirs = dictionary_dirs();
    let names: Vec<(PathBuf, String)> = dirs
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?.strip_suffix(".dic")?.to_owned();
            Some((path.with_extension(""), name))
        })
        .collect();
    let listed: Vec<String> = names.iter().map(|(_, name)| name.clone()).collect();
    let chosen = crate::i18n::languages()
        .iter()
        .find_map(|tag| dictionary_name(&listed, tag))?;
    let (base, _) = names.iter().find(|(_, name)| *name == chosen)?;
    let personal = personal_list(&chosen);
    let words = std::fs::read_to_string(&personal).unwrap_or_default();
    let _ = PERSONAL.set((personal, personal_words(&words)));
    let aff = std::fs::read_to_string(base.with_extension("aff")).ok()?;
    let dic = std::fs::read_to_string(base.with_extension("dic")).ok()?;
    match Dictionary::new(&aff, &dic) {
        Ok(dictionary) => Some(dictionary),
        Err(error) => {
            tracing::warn!(%error, dictionary = %chosen, "BarrierKit: could not read the dictionary");
            None
        }
    }
}

// Hunspell's search path: DICPATH first, then the hunspell and myspell data dirs (hunspell(1)).
fn dictionary_dirs() -> Vec<PathBuf> {
    let paths = |name: &str| {
        std::env::var_os(name)
            .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
            .unwrap_or_default()
    };
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    let mut data_dirs = paths("XDG_DATA_DIRS");
    if data_dirs.is_empty() {
        data_dirs = vec!["/usr/local/share".into(), "/usr/share".into()];
    }
    let mut dirs = paths("DICPATH");
    for base in data_home.into_iter().chain(data_dirs) {
        dirs.push(base.join("hunspell"));
        dirs.push(base.join("myspell"));
    }
    dirs
}

// Enchant's per-user word list, shared with GTK and GNOME apps (enchant(5)).
fn personal_list(name: &str) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_default()
        .join("enchant")
        .join(format!("{name}.dic"))
}

pub fn personal_words(list: &str) -> HashSet<String> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn known(dictionary: &Dictionary, word: &str) -> bool {
    dictionary.check(word)
        || PERSONAL
            .get()
            .is_some_and(|(_, words)| words.contains(word))
        || ACCEPTED.with(|accepted| accepted.read().contains(word))
}

pub(crate) fn ignore(word: String) {
    ACCEPTED.with(|accepted| {
        let mut accepted = *accepted;
        accepted.write().insert(word);
    });
}

pub(crate) fn learn(word: String) {
    if let Some((path, _)) = PERSONAL.get() {
        let saved = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
            })
            .and_then(|mut file| writeln!(file, "{word}"));
        if let Err(error) = saved {
            tracing::warn!(%error, "BarrierKit: could not save the learned word");
        }
    }
    ignore(word);
}

pub fn dictionary_name(names: &[String], tag: &str) -> Option<String> {
    let tag = tag.replace('-', "_");
    let language = tag.split('_').next().unwrap_or(&tag);
    if names.contains(&tag) {
        return Some(tag);
    }
    let mut regional: Vec<&String> = names
        .iter()
        .filter(|name| *name == language || name.starts_with(&format!("{language}_")))
        .collect();
    regional.sort();
    regional.first().map(|name| (*name).clone())
}

pub fn words(text: &str) -> Vec<Range<usize>> {
    let chars: Vec<char> = text.chars().collect();
    let joins = |index: usize| {
        matches!(chars[index], '\'' | '’')
            && index > 0
            && chars[index - 1].is_alphanumeric()
            && chars
                .get(index + 1)
                .is_some_and(|next| next.is_alphanumeric())
    };
    let mut found = Vec::new();
    let mut start = None;
    for index in 0..=chars.len() {
        let inside = index < chars.len() && (chars[index].is_alphanumeric() || joins(index));
        match (inside, start) {
            (true, None) => start = Some(index),
            (false, Some(from)) => {
                if !chars[from..index].iter().any(char::is_ascii_digit) {
                    found.push(from..index);
                }
                start = None;
            }
            _ => {}
        }
    }
    found
}

pub fn misspelled(text: &str, known: impl Fn(&str) -> bool) -> Vec<Range<usize>> {
    let chars: Vec<char> = text.chars().collect();
    words(text)
        .into_iter()
        .filter(|range| !known(&chars[range.clone()].iter().collect::<String>()))
        .collect()
}

pub fn word_at(text: &str, index: usize) -> Option<Range<usize>> {
    words(text)
        .into_iter()
        .find(|range| range.start <= index && index <= range.end)
}

pub fn replace_chars(text: &str, range: Range<usize>, with: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    chars[..range.start].iter().collect::<String>()
        + with
        + &chars[range.end..].iter().collect::<String>()
}

pub(crate) fn utf16_range(text: &str, range: &Range<usize>) -> Range<usize> {
    let offset = |end: usize| text.chars().take(end).map(char::len_utf16).sum();
    offset(range.start)..offset(range.end)
}

pub(crate) fn char_index(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    text.chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= utf16
        })
        .count()
}

pub(crate) struct Suggestion {
    pub word: String,
    pub range: Range<usize>,
    pub guesses: Vec<String>,
}

pub(crate) fn suggestion(text: &str, index: usize) -> Option<Suggestion> {
    let dictionary = dictionary()?;
    let range = word_at(text, index)?;
    let word: String = text.chars().skip(range.start).take(range.len()).collect();
    if known(dictionary, &word) {
        return None;
    }
    let mut guesses = Vec::new();
    dictionary.suggest(&word, &mut guesses);
    Some(Suggestion {
        word,
        range,
        guesses,
    })
}

#[derive(PartialEq)]
pub(crate) struct Squiggles {
    pub holder: ParagraphHolder,
    pub input: Option<AccessibilityId>,
    pub text_left: Rc<Cell<f32>>,
    pub ranges: Vec<Range<usize>>,
    pub color: Rgba,
    pub offset: (f32, f32),
    pub layout: LayoutData,
}

fn input_paragraph(tree: &Tree, field: AccessibilityId) -> Option<(NodeId, &ParagraphElement)> {
    let root = tree
        .accessibility_state
        .iter()
        .find_map(|(node, state)| (state.a11y_id == field).then_some(*node))?;
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        let element: Option<&dyn ElementExt> = tree.elements.get(&node).map(|e| &**e);
        if let Some(paragraph) = element.and_then(|e| (e as &dyn Any).downcast_ref()) {
            return Some((node, paragraph));
        }
        pending.extend(tree.children.get(&node).into_iter().flatten().rev());
    }
    None
}

impl Squiggles {
    fn draw(&self, context: &RenderContext, paragraph: &SkParagraph, (left, top): (f32, f32)) {
        let dot = SPELLING.dot * context.scale_factor as f32;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(PaintStyle::Fill);
        paint.set_color(crate::theme::color(self.color));
        for range in &self.ranges {
            for rect in paragraph.get_rects_for_range(
                range.clone(),
                freya_engine::prelude::RectHeightStyle::Tight,
                freya_engine::prelude::RectWidthStyle::Tight,
            ) {
                let y = top + rect.rect.bottom + dot / 2.0;
                let width = rect.rect.width() - dot;
                let count = (width / (dot * 2.0)).floor().max(0.0);
                let start = left + rect.rect.left + dot / 2.0 + (width - count * dot * 2.0) / 2.0;
                for step in 0..=count as usize {
                    let x = start + step as f32 * dot * 2.0;
                    context.canvas.draw_circle((x, y), dot / 2.0, &paint);
                }
            }
        }
    }

    fn draw_in_input(&self, context: &RenderContext, field: AccessibilityId) {
        let Some((node, element)) = input_paragraph(context.tree, field) else {
            return;
        };
        let inner = element
            .sk_paragraph
            .0
            .borrow()
            .as_ref()
            .map(|inner| ParagraphHolderInner {
                paragraph: inner.paragraph.clone(),
                scale_factor: inner.scale_factor,
            });
        let (Some(layout), Some(inner)) = (context.tree.layout.get(&node), inner) else {
            return;
        };
        let area = context.layout_node.visible_area();
        let scale = context.scale_factor as f32;
        let origin = layout.visible_area().origin;
        self.text_left.set((origin.x - area.min_x()) / scale);
        let paragraph = inner.paragraph.clone();
        *self.holder.0.borrow_mut() = Some(inner);
        context.canvas.save();
        context.canvas.clip_rect(
            freya_engine::prelude::Rect::new(
                area.min_x(),
                area.min_y(),
                area.max_x(),
                area.max_y(),
            ),
            None,
            None,
        );
        self.draw(context, &paragraph, (origin.x, origin.y));
        context.canvas.restore();
    }
}

impl ElementExt for Squiggles {
    fn changed(&self, other: &std::rc::Rc<dyn ElementExt>) -> bool {
        !crate::style_diff(self, other).is_empty()
    }

    fn diff(&self, other: &std::rc::Rc<dyn ElementExt>) -> DiffModifies {
        crate::style_diff(self, other)
    }

    fn layout(&'_ self) -> Cow<'_, LayoutData> {
        Cow::Borrowed(&self.layout)
    }

    fn render(&self, context: RenderContext) {
        if let Some(field) = self.input {
            self.draw_in_input(&context, field);
            return;
        }
        let Some(inner) = self
            .holder
            .0
            .borrow()
            .as_ref()
            .map(|inner| inner.paragraph.clone())
        else {
            return;
        };
        let area = context.layout_node.visible_area();
        let scale = context.scale_factor as f32;
        let origin = (
            area.min_x() + self.offset.0 * scale,
            area.min_y() + self.offset.1 * scale,
        );
        self.draw(&context, &inner, origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_keep_inner_apostrophes_and_skip_numbers() {
        let text = "l’eau don't 3rd  año.";
        let found: Vec<String> = words(text)
            .into_iter()
            .map(|r| text.chars().skip(r.start).take(r.len()).collect())
            .collect();
        assert_eq!(found, ["l’eau", "don't", "año"]);
    }

    #[test]
    fn only_words_missing_from_the_dictionary_are_misspelled() {
        let dictionary = Dictionary::new("SET UTF-8\n", "2\nhola\nmundo\n").unwrap();
        let known = |word: &str| dictionary.check(word);
        assert_eq!(misspelled("hola mundu, mundo", known), vec![(5..10)]);
    }

    #[test]
    fn the_personal_list_skips_blank_and_comment_lines() {
        let words = personal_words("# enchant\nmalaysia\n\n  vow  \n");
        assert_eq!(words, HashSet::from(["malaysia".into(), "vow".into()]));
    }

    #[test]
    fn a_language_tag_picks_its_exact_dictionary_then_its_first_regional_one() {
        let names = ["en_GB", "en_US", "es_MX", "es_ES"].map(String::from);
        assert_eq!(dictionary_name(&names, "en-US").as_deref(), Some("en_US"));
        assert_eq!(dictionary_name(&names, "es").as_deref(), Some("es_ES"));
        assert_eq!(dictionary_name(&names, "fr"), None);
    }

    #[test]
    fn utf16_offsets_round_trip_past_astral_characters() {
        let text = "a😀bc";
        assert_eq!(utf16_range(text, &(2..4)), 3..5);
        assert_eq!(char_index(text, 3), 2);
        assert_eq!(replace_chars(text, 2..4, "x"), "a😀x");
    }
}
