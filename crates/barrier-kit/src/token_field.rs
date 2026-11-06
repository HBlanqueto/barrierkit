use freya_components::input::InputColorsThemePartial;
use freya_components::theming::macros::Preference;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{HAIRLINE, Rgba, TEXT, TEXT_FIELD, TOKEN_FIELD};

use crate::context_menu::edit_menu;
use crate::controls::{default_pre_key, text_style, themed_input};
use crate::style::{Palette, text};
use crate::theme::color;

#[derive(PartialEq)]
pub struct TokenField {
    tokens: State<Vec<String>>,
    placeholder: &'static str,
    width: Size,
}

impl TokenField {
    pub fn new(tokens: State<Vec<String>>, placeholder: &'static str) -> Self {
        Self {
            tokens,
            placeholder,
            width: Size::fill(),
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
}

pub fn commit_token(tokens: &mut Vec<String>, draft: &str) -> bool {
    let token = draft.trim().trim_end_matches(',').trim();
    if token.is_empty() {
        return false;
    }
    tokens.push(token.to_owned());
    true
}

pub(crate) fn token_chip(token: String, selected: bool) -> Rect {
    let palette = Palette::current();
    let (fill, ink) = if selected {
        (palette.accent, palette.on_accent)
    } else {
        (
            Rgba {
                a: TOKEN_FIELD.token_opacity,
                ..palette.accent
            },
            palette.label,
        )
    };
    rect()
        .height(Size::px(TOKEN_FIELD.token_height))
        .padding(Gaps::new(
            0.0,
            TOKEN_FIELD.token_padding_x,
            0.0,
            TOKEN_FIELD.token_padding_x,
        ))
        .corner_radius(TOKEN_FIELD.token_height / 2.0)
        .main_align(Alignment::Center)
        .background(color(fill))
        .a11y_role(AccessibilityRole::ListItem)
        .a11y_alt(token.clone())
        .a11y_builder(move |node| node.set_selected(selected))
        .child(text(token, TEXT, ink))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKey {
    Backspace,
    Delete,
    Left,
    Right,
}

pub fn token_selection(
    chosen: Option<usize>,
    count: usize,
    key: TokenKey,
) -> (Option<usize>, Option<usize>) {
    let last = count.checked_sub(1);
    match (key, chosen) {
        (TokenKey::Backspace | TokenKey::Delete, Some(index)) if index < count => {
            (None, Some(index))
        }
        (TokenKey::Backspace | TokenKey::Left, None) => (last, None),
        (TokenKey::Left, Some(index)) => (Some(index.saturating_sub(1)), None),
        (TokenKey::Right, Some(index)) => (Some(index + 1).filter(|next| *next < count), None),
        _ => (None, None),
    }
}

impl Component for TokenField {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut tokens = self.tokens;
        let mut draft = use_state(String::new);
        let mut chosen = use_state(|| None::<usize>);
        let input_id = use_hook(AccessibilityId::new_unique);
        let clear = Rgba {
            a: 0.0,
            ..palette.content_background
        };
        let transparent = Some(Preference::Specific(color(clear)));
        let input = themed_input(draft, self.placeholder, clear, clear, 0.0, 0.0)
            .width(Size::px(TOKEN_FIELD.input_min_width))
            .a11y_id(input_id)
            .theme_colors(InputColorsThemePartial {
                background: transparent.clone(),
                focus_background: transparent.clone(),
                border_fill: transparent.clone(),
                focus_border_fill: transparent,
                color: Some(Preference::Specific(color(palette.label))),
                placeholder_color: Some(Preference::Specific(color(palette.tertiary_label))),
            })
            .on_pre_key_down(move |event: Event<KeyboardEventData>| {
                let empty = draft.peek().is_empty();
                let count = tokens.peek().len();
                let key = match &event.key {
                    Key::Named(NamedKey::Backspace) if empty => Some(TokenKey::Backspace),
                    Key::Named(NamedKey::Delete) if empty => Some(TokenKey::Delete),
                    Key::Named(NamedKey::ArrowLeft) if empty => Some(TokenKey::Left),
                    Key::Named(NamedKey::ArrowRight) if empty => Some(TokenKey::Right),
                    _ => None,
                };
                if let Some(key) = key {
                    let (next, removed) = token_selection(*chosen.peek(), count, key);
                    if let Some(index) = removed {
                        tokens.write().remove(index);
                    }
                    chosen.set(next);
                    event.stop_propagation();
                    return false;
                }
                chosen.set_if_modified(None);
                match &event.key {
                    Key::Named(NamedKey::Enter) => {
                        let text = draft.peek().clone();
                        if commit_token(&mut tokens.write(), &text) {
                            draft.set(String::new());
                        }
                        false
                    }
                    Key::Character(c) if c == "," => {
                        let text = draft.peek().clone();
                        if commit_token(&mut tokens.write(), &text) {
                            draft.set(String::new());
                        }
                        event.stop_propagation();
                        false
                    }
                    _ => default_pre_key(&event),
                }
            });
        let line = |key: String, item: Element| {
            rect()
                .key(key)
                .height(Size::px(TEXT_FIELD.height))
                .main_align(Alignment::Center)
                .child(item)
                .into_element()
        };
        let chips = self
            .tokens
            .read()
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, token)| {
                let chip = token_chip(token.clone(), *chosen.read() == Some(index))
                    .on_press(move |_| {
                        chosen.set(Some(index));
                        input_id.request_focus();
                    })
                    .into_element();
                line(format!("{index}:{token}"), chip)
            });
        edit_menu(text_style(rect().width(self.width.clone())), input_id)
            .a11y_role(AccessibilityRole::List)
            .a11y_alt(self.placeholder)
            .min_height(Size::px(TEXT_FIELD.height))
            .padding(Gaps::new(
                0.0,
                TEXT_FIELD.padding_x,
                0.0,
                TEXT_FIELD.padding_x,
            ))
            .corner_radius(TEXT_FIELD.radius)
            .background(color(palette.content_background))
            .border(
                Border::new()
                    .width(HAIRLINE)
                    .alignment(BorderAlignment::Inner)
                    .fill(color(palette.separator)),
            )
            .direction(Direction::Horizontal)
            .content(Content::Wrap {
                wrap_spacing: Some(TOKEN_FIELD.gap),
            })
            .cross_align(Alignment::Center)
            .spacing(TOKEN_FIELD.gap)
            .children(chips)
            .child(line(String::from("input"), input.into_element()))
    }
}

#[cfg(test)]
mod tests {
    use super::commit_token;

    use super::{TokenKey, token_selection};

    #[test]
    fn backspace_selects_the_last_token_first_and_deletes_it_the_second_time() {
        assert_eq!(
            token_selection(None, 2, TokenKey::Backspace),
            (Some(1), None)
        );
        assert_eq!(
            token_selection(Some(1), 2, TokenKey::Backspace),
            (None, Some(1))
        );
        assert_eq!(token_selection(None, 0, TokenKey::Backspace), (None, None));
    }

    #[test]
    fn arrows_walk_the_tokens_and_leave_them_past_the_last() {
        assert_eq!(token_selection(None, 3, TokenKey::Left), (Some(2), None));
        assert_eq!(token_selection(Some(0), 3, TokenKey::Left), (Some(0), None));
        assert_eq!(
            token_selection(Some(1), 3, TokenKey::Right),
            (Some(2), None)
        );
        assert_eq!(token_selection(Some(2), 3, TokenKey::Right), (None, None));
    }

    #[test]
    fn a_token_is_the_trimmed_draft_without_its_comma_and_empty_drafts_add_nothing() {
        let mut tokens = Vec::new();
        assert!(commit_token(&mut tokens, "  ana@ejemplo.org, "));
        assert!(!commit_token(&mut tokens, " , "));
        assert_eq!(tokens, ["ana@ejemplo.org"]);
    }
}
