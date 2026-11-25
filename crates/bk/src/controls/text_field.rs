use ctdesign::{ControlSize, Rgba};
use freya_components::input::{
    Input, InputColorsThemePartial, InputLayoutThemePartial, InputValidator,
};
use freya_components::theming::macros::Preference;
use freya_core::prelude::*;
use std::borrow::Cow;
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};
use zeroize::Zeroizing;

use crate::context_menu::{Spelling, edit_menu};
use crate::controls::{dimmed, outer_hairline};
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::environment::use_disabled;
use crate::focus::use_focus_target;
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::layout::ambient;
use crate::password::PasswordField;
use crate::spelling::{InputParagraphs, Squiggles, char_index, use_misspellings};
use crate::style::text_style;
use crate::styles::{
    ControlState, Painted, Paints, SearchFieldStyle, SecureFieldStyle, ShapeStyle, StateValue,
    TextFieldStyle, Theme, TokenFieldStyle, resolve,
};
use crate::symbol::{Symbol, stroked_symbol};
use crate::theme::color;

#[derive(Clone, PartialEq)]
pub struct SecureField {
    field: State<PasswordField>,
    label: Cow<'static, str>,
    width: Size,
    auto_focus: bool,
    revealed: bool,
    disabled: bool,
    on_submit: Option<Callback<Zeroizing<String>, ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SecureField {
    pub fn new(field: State<PasswordField>, label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            field,
            label: label.into(),
            width: Size::fill(),
            auto_focus: false,
            revealed: false,
            disabled: false,
            on_submit: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SecureFieldStyle) -> SecureFieldStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn on_submit(mut self, callback: impl Into<Callback<Zeroizing<String>, ()>>) -> Self {
        self.on_submit = Some(callback.into());
        self
    }
}

forward_key!(SecureField);

forward_overrides!(SecureField);

impl Component for SecureField {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let field_style: TextFieldStyle = resolve(&self.style);
        let style: SecureFieldStyle = resolve(&self.style);
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state);
        let mut field = self.field;
        let on_submit = self.on_submit.clone();
        let shown = if self.revealed {
            field.read().revealed()
        } else {
            field.read().masked()
        };
        let height = field_style.height;
        let margin_y = (height - field_style.font.line_height as f32) / 2.0;
        let ink = paints.color(field_style.text.resolve(state));
        let radius = field_style.shape.corner_radius(height, height, ambient());
        // The kit's focus ring stands for the focused border of a text field.
        let unfocused = ControlState {
            focused: false,
            ..state
        };
        let element = interaction
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::PasswordInput)
            .a11y_alt(&*self.label)
            .a11y_auto_focus(self.auto_focus)
            .width(self.width.clone())
            .height(Size::px(height))
            .padding(Gaps::new(
                margin_y,
                field_style.padding_x,
                margin_y,
                field_style.padding_x,
            ))
            .corner_radius(radius)
            .painted(&paints, &field_style.background)
            .border(outer_hairline(
                paints.color(field_style.border.resolve(unfocused)),
            ))
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let unhandled_return =
                    on_submit.is_none() && event.key == Key::Named(NamedKey::Enter);
                if disabled || unhandled_return {
                    return;
                }
                let secret = field.write().handle_key(&event.key, event.modifiers);
                if let Some(secret) = secret {
                    event.prevent_default();
                    if let Some(callback) = &on_submit {
                        callback.call(secret);
                    }
                }
            })
            .child(if self.revealed {
                crate::style::text(shown, field_style.font, ink).into_element()
            } else {
                let (dot, pitch) = style.dot;
                let (inset, drop) = style.dot_offset;
                rect()
                    .height(Size::px(field_style.font.line_height as f32))
                    .padding(Gaps::new(0.0, 0.0, 0.0, inset))
                    .direction(Direction::Horizontal)
                    .cross_align(Alignment::Center)
                    .spacing(pitch - dot)
                    .offset_y(drop)
                    .children(shown.chars().map(|_| {
                        rect()
                            .width(Size::px(dot))
                            .height(Size::px(dot))
                            .corner_radius(dot / 2.0)
                            .background(color(ink))
                            .into_element()
                    }))
                    .into_element()
            });
        let root = interaction.focus_ring(dimmed(element, field_style.opacity), radius.top_left());
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct TextField {
    value: State<String>,
    placeholder: Cow<'static, str>,
    width: Size,
    auto_focus: bool,
    autocorrection_disabled: bool,
    disabled: bool,
    on_submit: Option<Callback<String, ()>>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl TextField {
    pub fn new(value: State<String>, placeholder: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value,
            placeholder: placeholder.into(),
            width: Size::fill(),
            auto_focus: false,
            autocorrection_disabled: false,
            disabled: false,
            on_submit: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: TextFieldStyle| style.sized(size))
    }

    pub fn autocorrection_disabled(mut self, disabled: bool) -> Self {
        self.autocorrection_disabled = disabled;
        self
    }

    pub fn on_submit(mut self, callback: impl Into<Callback<String, ()>>) -> Self {
        self.on_submit = Some(callback.into());
        self
    }

    pub fn style(mut self, restyle: impl Fn(TextFieldStyle) -> TextFieldStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }
}

forward_key!(TextField);

forward_overrides!(TextField);

impl Component for TextField {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: TextFieldStyle = resolve(&self.style);
        let field = use_hook(AccessibilityId::new_unique);
        let (state, _, mut hovered) = use_field_state(field, self.disabled);
        let disabled = state.disabled;
        let paints = Paints::current(state);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let radius = style
            .shape
            .corner_radius(style.height, style.height, ambient());
        let input = input(
            self.value,
            self.placeholder.clone(),
            [
                if disabled {
                    Rgba::TRANSPARENT
                } else {
                    ink(&style.background)
                },
                Rgba::TRANSPARENT,
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, style.padding_x, margin_y, style.padding_x),
        )
        .height(Size::px(style.height))
        .text_align(style.text_align)
        .enabled(!disabled)
        .auto_focus(self.auto_focus)
        .map(self.on_submit.clone(), |input, on_submit| {
            input.on_submit(move |text: String| on_submit.call(text))
        });
        let mut index = use_state(|| None);
        let holder = use_state(ParagraphHolder::default);
        let padding_x = style.padding_x;
        let text_left = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(padding_x)));
        let left = text_left.clone();
        let value = self.value;
        let checked = !self.autocorrection_disabled && !disabled;
        let ranges = use_misspellings(value, checked);
        let root = edit_menu(
            dimmed(
                text_style(rect().width(self.width.clone()), style.font),
                style.opacity,
            ),
            field,
            true,
            (true, true),
            checked.then_some(Spelling {
                value,
                index,
                styles: None,
            }),
        )
        .child(
            rect()
                .width(Size::fill())
                .corner_radius(radius)
                .border(outer_hairline(ink(&style.border)))
                .maybe(disabled, |el| el.background(color(ink(&style.background))))
                .on_pointer_enter(move |_| hovered.set(true))
                .on_pointer_leave(move |_| hovered.set(false))
                .on_secondary_down(move |event: Event<PressEventData>| {
                    if let (PressEventData::Mouse(mouse), Some(inner)) =
                        (event.data(), holder.peek().0.borrow().as_ref())
                    {
                        let x = (mouse.element_location.x as f32 - left.get())
                            * inner.scale_factor as f32;
                        let position = inner
                            .paragraph
                            .get_glyph_position_at_coordinate((x, 0.0))
                            .position;
                        index.set(Some(char_index(&value.peek(), position.max(0) as usize)));
                    }
                })
                .child(input.a11y_id(field))
                .maybe_child((!ranges.is_empty()).then(|| {
                    crate::glass::leaf(Squiggles {
                        holder: holder.read().clone(),
                        input: try_consume_root_context::<InputParagraphs>().map(|inputs| {
                            inputs.watch(field);
                            (field, inputs)
                        }),
                        text_left,
                        ranges,
                        color: ink(&style.misspelling),
                        offset: (0.0, 0.0),
                        layout: crate::glass::layout(
                            Size::fill(),
                            Size::fill(),
                            Position::new_absolute(),
                        ),
                    })
                })),
        );
        let root = root.a11y_builder(|node| {
            crate::accessibility::mark_field(node, &self.placeholder, !self.value.read().is_empty())
        });
        self.overrides.apply(root)
    }
}

#[derive(PartialEq)]
pub struct SearchField {
    value: State<String>,
    placeholder: Cow<'static, str>,
    width: Size,
    auto_focus: bool,
    on_focus_change: Option<Callback<bool, ()>>,
    on_navigate: Option<Callback<NamedKey, ()>>,
    a11y_id: Option<AccessibilityId>,
    tokens: Option<State<Vec<String>>>,
    disabled: bool,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl SearchField {
    pub fn tokens(mut self, tokens: State<Vec<String>>) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn new(value: State<String>, placeholder: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value,
            placeholder: placeholder.into(),
            width: Size::fill(),
            auto_focus: false,
            on_focus_change: None,
            on_navigate: None,
            a11y_id: None,
            tokens: None,
            disabled: false,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: SearchFieldStyle| style.sized(size))
    }

    pub fn style(
        mut self,
        restyle: impl Fn(SearchFieldStyle) -> SearchFieldStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn a11y_id(mut self, a11y_id: AccessibilityId) -> Self {
        self.a11y_id = Some(a11y_id);
        self
    }

    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn on_focus_change(mut self, callback: impl Into<Callback<bool, ()>>) -> Self {
        self.on_focus_change = Some(callback.into());
        self
    }

    pub fn on_navigate(mut self, callback: impl Into<Callback<NamedKey, ()>>) -> Self {
        self.on_navigate = Some(callback.into());
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
}

forward_key!(SearchField);

forward_overrides!(SearchField);

impl Component for SearchField {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: SearchFieldStyle = resolve(&self.style);
        let token_style: TokenFieldStyle = resolve(&Theme::tengen());
        let mut value = self.value;
        let padding = style.padding_x;
        let icon = style.icon_size;
        let (glyph, offset, clear) = (style.glyph, style.glyph_offset, style.clear_size);
        let gap = style.icon_gap;
        let height = style.height;
        let a11y_id = use_hook(|| self.a11y_id.unwrap_or_else(AccessibilityId::new_unique));
        let (state, focus, mut hovered) = use_field_state(a11y_id, self.disabled);
        let disabled = state.disabled;
        let paints = Paints::current(state);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let on_focus_change = use_reactive(&self.on_focus_change);
        let mut reported = use_state(|| false);
        use_side_effect(move || {
            let focused = focus.read().is_focused();
            if *reported.peek() != focused {
                reported.set(focused);
                if let Some(callback) = &*on_focus_change.peek() {
                    callback.call(focused);
                }
            }
        });
        let mut tokens_width = use_state(|| 0.0f32);
        let tokens = self.tokens;
        let shown_tokens = tokens
            .map(|tokens| tokens.read().clone())
            .unwrap_or_default();
        let leading = padding
            + icon
            + gap
            + if shown_tokens.is_empty() {
                0.0
            } else {
                *tokens_width.read() + gap
            };
        let margin_y = (height - style.font.line_height as f32) / 2.0;
        let radius = style.shape.corner_radius(height, height, ambient());
        let on_navigate = self.on_navigate.clone();
        let input = input(
            value,
            self.placeholder.clone(),
            [
                if disabled {
                    Rgba::TRANSPARENT
                } else {
                    ink(&style.background)
                },
                Rgba::TRANSPARENT,
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, padding + icon + gap, margin_y, leading),
        )
        .height(Size::px(height))
        .enabled(!disabled)
        .a11y_id(a11y_id)
        .auto_focus(self.auto_focus)
        .maybe(on_navigate.is_some() || tokens.is_some(), |input| {
            input.on_pre_key_down(move |event: Event<KeyboardEventData>| match &event.key {
                Key::Named(key @ (NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Enter))
                    if on_navigate.is_some() =>
                {
                    event.stop_propagation();
                    if let Some(on_navigate) = &on_navigate {
                        on_navigate.call(*key);
                    }
                    false
                }
                Key::Named(NamedKey::Backspace) if value.peek().is_empty() => {
                    if let Some(mut tokens) = tokens {
                        tokens.write().pop();
                    }
                    event.stop_propagation();
                    false
                }
                _ => default_pre_key(&event),
            })
        })
        .maybe(!value.read().is_empty(), |input| {
            input.trailing(
                rect()
                    .offset_x(style.clear_offset.0)
                    .offset_y(style.clear_offset.1)
                    .child(
                        rect()
                            .a11y_role(AccessibilityRole::Button)
                            .a11y_alt(text!("clear-search"))
                            .on_pointer_down(move |event: Event<PointerEventData>| {
                                if event.is_primary() && !disabled {
                                    value.set(String::new());
                                }
                            })
                            .width(Size::px(clear.0))
                            .height(Size::px(clear.0))
                            .corner_radius(clear.0 / 2.0)
                            .background(color(ink(&style.clear_background)))
                            .center()
                            .child(stroked_symbol(
                                Symbol::XMark,
                                clear.1,
                                clear.2,
                                ink(&style.clear_icon),
                            )),
                    ),
            )
        });

        let root = edit_menu(
            dimmed(
                text_style(rect().width(self.width.clone()), style.font),
                style.opacity,
            )
            .corner_radius(radius)
            .border(outer_hairline(ink(&style.border)))
            .maybe(disabled, |el| el.background(color(ink(&style.background))))
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false)),
            a11y_id,
            true,
            (true, true),
            None,
        )
        .child(input)
        .child(
            rect()
                .position(
                    Position::new_absolute()
                        .left(padding + (icon - glyph.0) / 2.0 + offset.0)
                        .top((height - glyph.0) / 2.0 + offset.1),
                )
                .child(stroked_symbol(
                    Symbol::MagnifyingGlass,
                    glyph.0,
                    glyph.1,
                    ink(&style.icon),
                )),
        )
        .maybe(!shown_tokens.is_empty(), |el| {
            el.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(padding + icon + gap)
                            .top((height - token_style.token_height) / 2.0),
                    )
                    .direction(Direction::Horizontal)
                    .spacing(token_style.gap)
                    .on_sized(move |event: Event<SizedEventData>| {
                        tokens_width.set_if_modified(event.area.width());
                    })
                    .children(shown_tokens.into_iter().map(|token| {
                        crate::token_field::token_chip(token, false, &token_style).into_element()
                    })),
            )
        });
        let root = root.a11y_builder(|node| {
            crate::accessibility::mark_field(node, &self.placeholder, !self.value.read().is_empty())
        });
        self.overrides.apply(root)
    }
}

// Freya's `Input` tracks its own hover and focus and reports neither.
pub(crate) fn use_field_state(
    field: AccessibilityId,
    disabled: bool,
) -> (ControlState, Memo<Focus>, State<bool>) {
    use_focus_target(field);
    let focus = use_focus(field);
    let disabled = use_disabled(disabled);
    let hovered = use_state(|| false);
    let state = ControlState {
        hovered: *hovered.read(),
        focused: !matches!(*focus.read(), Focus::Not),
        disabled,
        ..ControlState::default()
    };
    (state, focus, hovered)
}

// Mirrors Input's default pre-handler for one line (freya-components 0.5.3, input.rs).
pub(crate) fn default_pre_key(event: &Event<KeyboardEventData>) -> bool {
    match &event.key {
        Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Shift) => true,
        Key::Named(NamedKey::Tab) => false,
        _ => {
            event.stop_propagation();
            event.prevent_default();
            true
        }
    }
}

pub(crate) fn input(
    value: State<String>,
    placeholder: impl Into<Cow<'static, str>>,
    [background, border, focus_border, text, placeholder_color]: [Rgba; 5],
    corner_radius: CornerRadius,
    inner_margin: Gaps,
) -> Input {
    let (generation, mut typed) = use_input_generation(value);
    let color = |c: Rgba| Some(Preference::Specific(color(c)));
    Input::new(value)
        .key(generation)
        .on_validate(move |validator: InputValidator| typed.set(validator.text().clone()))
        .placeholder(placeholder)
        .width(Size::fill())
        .theme_colors(InputColorsThemePartial {
            background: color(background),
            focus_background: color(background),
            border_fill: color(border),
            focus_border_fill: color(focus_border),
            color: color(text),
            placeholder_color: color(placeholder_color),
        })
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(corner_radius)),
            padding: Some(Preference::Specific(inner_margin)),
        })
}

// freya-edit 0.5.3 set() checks the UTF-16 cursor against a byte length: https://docs.rs/crate/freya-edit/0.5.3/source/src/rope_editor.rs#278
fn use_input_generation(value: State<String>) -> (u32, State<String>) {
    let typed = use_state(String::new);
    let mut seen = use_state(|| value.peek().clone());
    let mut generation = use_state(|| 0u32);
    let current = value.read().clone();
    if current != *seen.peek() {
        if current != *typed.peek() && cursor_may_pass_end(&seen.peek(), &current) {
            *generation.write() += 1;
        }
        seen.set(current);
    }
    (*generation.peek(), typed)
}

fn cursor_may_pass_end(old: &str, new: &str) -> bool {
    let units = new.encode_utf16().count();
    old.encode_utf16().count().min(new.len()) > units
}
