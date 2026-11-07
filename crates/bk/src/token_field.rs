use ctdesign::Rgba;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Size};

use crate::context_menu::edit_menu;
use crate::controls::{default_pre_key, dimmed, inner_border, input, shadowed, text_style};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{ControlState, LayoutDirection, Paints, Theme, TokenFieldStyle, resolve};

#[derive(PartialEq)]
pub struct TokenField {
    tokens: State<Vec<String>>,
    placeholder: &'static str,
    width: Size,
    style: Theme,
}

impl TokenField {
    pub fn new(tokens: State<Vec<String>>, placeholder: &'static str) -> Self {
        Self {
            tokens,
            placeholder,
            width: Size::fill(),
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(TokenFieldStyle) -> TokenFieldStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
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

pub(crate) fn token_chip(token: String, selected: bool, style: &TokenFieldStyle) -> Rect {
    let paints = Paints::current(ControlState::default(), style.effect);
    let (fill, ink) = if selected {
        (&style.selected_token, &style.selected_token_label)
    } else {
        (&style.token, &style.token_label)
    };
    let height = style.token_height;
    rect()
        .height(Size::px(height))
        .padding(Gaps::new(
            0.0,
            style.token_padding_x,
            0.0,
            style.token_padding_x,
        ))
        .corner_radius(style.token_shape.corner_radius(
            height,
            height,
            LayoutDirection::LeftToRight,
        ))
        .main_align(Alignment::Center)
        .painted(&paints, fill)
        .a11y_role(AccessibilityRole::ListItem)
        .a11y_alt(token.clone())
        .a11y_builder(move |node| node.set_selected(selected))
        .child(
            with_font(label().text(token), style.font)
                .max_lines(1)
                .color(paints.fill(ink)),
        )
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
        let style: TokenFieldStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let ink = |paint| paints.color(paint);
        let mut tokens = self.tokens;
        let mut draft = use_state(String::new);
        let mut chosen = use_state(|| None::<usize>);
        let input_id = use_hook(AccessibilityId::new_unique);
        let state = ControlState::default();
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let input = input(
            draft,
            self.placeholder,
            [
                Rgba::TRANSPARENT,
                Rgba::TRANSPARENT,
                Rgba::TRANSPARENT,
                ink(style.text.resolve(state)),
                ink(style.placeholder.resolve(state)),
            ],
            CornerRadius::new_all(0.0),
            Gaps::new(margin_y, 0.0, margin_y, 0.0),
        )
        .width(Size::px(style.input_width))
        .a11y_id(input_id)
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
        let height = style.height;
        let line = |key: String, item: Element| {
            rect()
                .key(key)
                .height(Size::px(height))
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
                let chip = token_chip(token.clone(), *chosen.read() == Some(index), &style)
                    .on_press(move |_| {
                        chosen.set(Some(index));
                        input_id.request_focus();
                    })
                    .into_element();
                line(format!("{index}:{token}"), chip)
            });
        let field = edit_menu(
            text_style(rect().width(self.width.clone()), style.font),
            input_id,
            true,
            (true, true),
            None,
        )
        .a11y_role(AccessibilityRole::List)
        .a11y_alt(self.placeholder)
        .min_height(Size::px(style.height))
        .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
        .corner_radius(style.shape.corner_radius(
            style.height,
            style.height,
            LayoutDirection::LeftToRight,
        ))
        .painted(&paints, &style.background)
        .map(style.border.as_ref(), |el, border| {
            el.border(inner_border(&paints, border))
        })
        .direction(Direction::Horizontal)
        .content(Content::Wrap {
            wrap_spacing: Some(style.gap),
        })
        .cross_align(Alignment::Center)
        .spacing(style.gap)
        .children(chips)
        .child(line(String::from("input"), input.into_element()));
        dimmed(shadowed(field, &style.shadows), style.opacity)
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
