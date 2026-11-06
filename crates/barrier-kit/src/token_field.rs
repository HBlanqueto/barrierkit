use freya_components::input::InputColorsThemePartial;
use freya_components::theming::macros::Preference;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};
use vow_design::{HAIRLINE, Rgba, TEXT, TEXT_FIELD, TOKEN_FIELD};

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

impl Component for TokenField {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut tokens = self.tokens;
        let mut draft = use_state(String::new);
        let clear = Rgba {
            a: 0.0,
            ..palette.content_background
        };
        let transparent = Some(Preference::Specific(color(clear)));
        let input = themed_input(draft, self.placeholder, clear, clear, 0.0, 0.0)
            .width(Size::px(TOKEN_FIELD.input_min_width))
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
                    Key::Named(NamedKey::Backspace) if empty => {
                        tokens.write().pop();
                        event.stop_propagation();
                        false
                    }
                    _ => default_pre_key(&event),
                }
            });
        let line = |item: Element| {
            rect()
                .height(Size::px(TEXT_FIELD.height))
                .main_align(Alignment::Center)
                .child(item)
                .into_element()
        };
        let chips = self.tokens.read().clone().into_iter().map(|token| {
            let chip = rect()
                .height(Size::px(TOKEN_FIELD.token_height))
                .padding(Gaps::new(
                    0.0,
                    TOKEN_FIELD.token_padding_x,
                    0.0,
                    TOKEN_FIELD.token_padding_x,
                ))
                .corner_radius(TOKEN_FIELD.token_height / 2.0)
                .main_align(Alignment::Center)
                .background(color(Rgba {
                    a: TOKEN_FIELD.token_opacity,
                    ..palette.accent
                }))
                .a11y_role(AccessibilityRole::ListItem)
                .a11y_alt(token.clone())
                .child(text(token, TEXT, palette.label))
                .into_element();
            line(chip)
        });
        text_style(rect().width(self.width.clone()))
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
            .child(line(input.into_element()))
    }
}

#[cfg(test)]
mod tests {
    use super::commit_token;

    #[test]
    fn a_token_is_the_trimmed_draft_without_its_comma_and_empty_drafts_add_nothing() {
        let mut tokens = Vec::new();
        assert!(commit_token(&mut tokens, "  ana@ejemplo.org, "));
        assert!(!commit_token(&mut tokens, " , "));
        assert_eq!(tokens, ["ana@ejemplo.org"]);
    }
}
