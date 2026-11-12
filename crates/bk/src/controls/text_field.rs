use super::*;

#[derive(Clone, PartialEq)]
pub struct SecureField {
    field: State<PasswordField>,
    label: &'static str,
    width: Size,
    auto_focus: bool,
    revealed: bool,
    on_submit: Option<Callback<Zeroizing<String>, ()>>,
    style: Theme,
    key: DiffKey,
}

impl SecureField {
    pub fn new(field: State<PasswordField>, label: &'static str) -> Self {
        Self {
            field,
            label,
            width: Size::fill(),
            auto_focus: false,
            revealed: false,
            on_submit: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
    }

    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
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

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
    }
}

impl KeyExt for SecureField {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SecureField {
    fn render(&self) -> impl IntoElement {
        let interaction = use_interaction();
        let style: SecureFieldStyle = resolve(&self.style);
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let mut field = self.field;
        let on_submit = self.on_submit.clone();
        let shown = if self.revealed {
            field.read().revealed()
        } else {
            field.read().masked()
        };
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let element = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::PasswordInput)
            .a11y_alt(self.label)
            .a11y_auto_focus(self.auto_focus)
            .width(self.width.clone())
            .height(Size::px(style.height))
            .padding(Gaps::new(
                margin_y,
                style.padding_x,
                margin_y,
                style.padding_x,
            ))
            .corner_radius(radius)
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .on_key_down(move |event: Event<KeyboardEventData>| {
                let secret = field.write().handle_key(&event.key, event.modifiers);
                if let Some(secret) = secret {
                    event.prevent_default();
                    if let Some(callback) = &on_submit {
                        callback.call(secret);
                    }
                }
            })
            .child(crate::style::text(
                shown,
                style.font,
                paints.color(style.text.resolve(state)),
            ));
        interaction.focus_ring(dimmed(element, style.opacity), radius.top_left())
    }
}

#[derive(PartialEq)]
pub struct TextField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    auto_focus: bool,
    autocorrection_disabled: bool,
    on_submit: Option<Callback<String, ()>>,
    style: Theme,
    key: DiffKey,
}

impl TextField {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            auto_focus: false,
            autocorrection_disabled: false,
            on_submit: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
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

impl KeyExt for TextField {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for TextField {
    fn render(&self) -> impl IntoElement {
        let style: TextFieldStyle = resolve(&self.style);
        let field = use_hook(AccessibilityId::new_unique);
        let focus = use_focus(field);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
        let ink = |paint: &StateValue<ShapeStyle>| paints.color(paint.resolve(state));
        let margin_y = (style.height - style.font.line_height as f32) / 2.0;
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let input = input(
            self.value,
            self.placeholder,
            [
                ink(&style.background),
                ink(&style.border),
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, style.padding_x, margin_y, style.padding_x),
        )
        .auto_focus(self.auto_focus);
        let input = match self.on_submit.clone() {
            Some(on_submit) => input.on_submit(move |text: String| on_submit.call(text)),
            None => input,
        };
        let mut index = use_state(|| None);
        let holder = use_state(ParagraphHolder::default);
        let padding_x = style.padding_x;
        let text_left = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(padding_x)));
        let left = text_left.clone();
        let value = self.value;
        let text = value.read().clone();
        let checked = !self.autocorrection_disabled;
        let ranges = checked
            .then(dictionary)
            .flatten()
            .map(|dictionary| {
                misspelled(&text, |word| known(dictionary, word))
                    .iter()
                    .map(|range| utf16_range(&text, range))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        edit_menu(
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
                .map(paints.blur(&style.background), |el, blur| {
                    el.corner_radius(radius).backdrop_blur(blur)
                })
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
        )
    }
}

#[derive(PartialEq)]
pub struct SearchField {
    value: State<String>,
    placeholder: &'static str,
    width: Size,
    auto_focus: bool,
    on_focus_change: Option<Callback<bool, ()>>,
    on_navigate: Option<Callback<NamedKey, ()>>,
    a11y_id: Option<AccessibilityId>,
    tokens: Option<State<Vec<String>>>,
    style: Theme,
    key: DiffKey,
}

impl SearchField {
    pub fn tokens(mut self, tokens: State<Vec<String>>) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            width: Size::fill(),
            auto_focus: false,
            on_focus_change: None,
            on_navigate: None,
            a11y_id: None,
            tokens: None,
            style: Theme::tengen(),
            key: DiffKey::None,
        }
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

impl KeyExt for SearchField {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for SearchField {
    fn render(&self) -> impl IntoElement {
        let style: SearchFieldStyle = resolve(&self.style);
        let token_style: TokenFieldStyle = resolve(&Theme::tengen());
        let mut value = self.value;
        let padding = style.padding_x;
        let icon = style.icon_size;
        let gap = style.icon_gap;
        let height = style.height;
        let a11y_id = use_hook(|| self.a11y_id.unwrap_or_else(AccessibilityId::new_unique));
        let focus = use_focus(a11y_id);
        let mut hovered = use_state(|| false);
        let state = ControlState {
            hovered: *hovered.read(),
            focused: !matches!(*focus.read(), Focus::Not),
            ..ControlState::default()
        };
        let paints = Paints::current(state, style.effect);
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
        let radius = style
            .shape
            .corner_radius(height, height, LayoutDirection::LeftToRight);
        let mut input = input(
            value,
            self.placeholder,
            [
                ink(&style.background),
                ink(&style.border),
                ink(&style.border),
                ink(&style.text),
                ink(&style.placeholder),
            ],
            radius,
            Gaps::new(margin_y, padding + icon + gap, margin_y, leading),
        )
        .a11y_id(a11y_id)
        .auto_focus(self.auto_focus);
        let on_navigate = self.on_navigate.clone();
        if on_navigate.is_some() || tokens.is_some() {
            input =
                input.on_pre_key_down(move |event: Event<KeyboardEventData>| match &event.key {
                    Key::Named(
                        key @ (NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Enter),
                    ) if on_navigate.is_some() => {
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
                });
        }
        if !value.read().is_empty() {
            input = input.trailing(
                rect()
                    .a11y_role(AccessibilityRole::Button)
                    .a11y_alt(text!("clear-search"))
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        if event.is_primary() {
                            value.set(String::new());
                        }
                    })
                    .child(symbol(Symbol::XMark, icon, ink(&style.clear_icon))),
            );
        }

        edit_menu(
            dimmed(
                text_style(rect().width(self.width.clone()), style.font),
                style.opacity,
            )
            .map(paints.blur(&style.background), |el, blur| {
                el.corner_radius(radius).backdrop_blur(blur)
            })
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
                        .left(padding)
                        .top((height - icon) / 2.0),
                )
                .child(symbol(Symbol::MagnifyingGlass, icon, ink(&style.icon))),
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
        })
    }
}

// Mirrors Input's default pre-handler for one line (freya-components 0.5.0, input.rs).
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

pub(crate) fn text_style(element: Rect, font: Font) -> Rect {
    element
        .font_family(font.family)
        .font_size(font.size as f32)
        .font_weight(font.weight as i32)
}

pub(crate) fn input(
    value: State<String>,
    placeholder: &'static str,
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

// freya-edit 0.5.0 set() checks the UTF-16 cursor against a byte length: https://docs.rs/crate/freya-edit/0.5.0/source/src/rope_editor.rs#278
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

#[cfg(test)]
mod tests {
    use super::cursor_may_pass_end;

    #[test]
    fn a_shorter_non_ascii_value_may_leave_the_cursor_past_the_end() {
        assert!(cursor_may_pass_end("Seville", "Málaga"));
        assert!(cursor_may_pass_end("añoo año", "año año"));
        assert!(cursor_may_pass_end("hello world", "😀"));
    }

    #[test]
    fn clearing_growing_or_ascii_values_keep_the_cursor_inside() {
        assert!(!cursor_may_pass_end("abc", ""));
        assert!(!cursor_may_pass_end("Málaga", "Malaga"));
        assert!(!cursor_may_pass_end("año", "años"));
    }
}
