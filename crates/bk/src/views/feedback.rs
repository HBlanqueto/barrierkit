use crate::elements::forward_container;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::controls::{Button, Checkbox, ControlSize};
use crate::i18n::text;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    AlertStyle, ButtonStyle, ContentUnavailableViewStyle, ControlState, LayoutDirection, Paints,
    SheetStyle, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

pub struct ContentUnavailableView(Rect, &'static str, Symbol, Theme);

impl ContentUnavailableView {
    pub fn new(title: &'static str, symbol: Symbol) -> Self {
        Self::styled(title, symbol, Theme::tengen())
    }

    pub fn style(
        self,
        restyle: impl Fn(ContentUnavailableViewStyle) -> ContentUnavailableViewStyle + 'static,
    ) -> Self {
        Self::styled(self.1, self.2, self.3.style(restyle))
    }

    fn styled(title: &'static str, icon: Symbol, theme: Theme) -> Self {
        let style: ContentUnavailableViewStyle = resolve(&theme);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let view = rect()
            .width(Size::fill())
            .direction(Direction::Vertical)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(style.padding_y, 0.0, style.padding_y, 0.0))
            .spacing(style.spacing)
            .child(symbol(
                icon,
                style.symbol_size,
                paints.color(style.symbol.resolve(state)),
            ))
            .child(text(
                title,
                style.font,
                paints.color(style.title.resolve(state)),
            ));
        Self(view, title, icon, theme)
    }
}

forward_container!(ContentUnavailableView);

pub struct Sheet(Rect, Element, Theme);

// A window-modal sheet leaves minimize and zoom working but disables close, as in AppKit.
#[derive(Clone, Copy)]
pub(crate) struct OpenSheets(pub(crate) State<u32>);

impl OpenSheets {
    pub(crate) fn any() -> bool {
        try_consume_root_context::<OpenSheets>().is_some_and(|OpenSheets(open)| *open.read() > 0)
    }
}

#[derive(PartialEq)]
struct SheetPresence;

impl Component for SheetPresence {
    fn render(&self) -> impl IntoElement {
        let open = use_hook(try_consume_root_context::<OpenSheets>);
        use_hook(move || {
            if let Some(OpenSheets(mut open)) = open {
                *open.write() += 1;
            }
        });
        use_drop(move || {
            if let Some(OpenSheets(mut open)) = open {
                let count = *open.peek();
                open.set(count.saturating_sub(1));
            }
        });
        rect()
    }
}

impl Sheet {
    pub fn new(content: impl IntoElement) -> Self {
        let content = content.into_element();
        let theme = Theme::tengen();
        Self(sheet(content.clone(), &resolve(&theme)), content, theme)
    }

    pub fn style(self, restyle: impl Fn(SheetStyle) -> SheetStyle + 'static) -> Self {
        let theme = self.2.style(restyle);
        Self(sheet(self.1.clone(), &resolve(&theme)), self.1, theme)
    }
}

forward_container!(Sheet);

fn sheet(content: impl IntoElement, style: &SheetStyle) -> Rect {
    let paints = Paints::current(ControlState::default(), style.effect);
    rect()
        .position(Position::new_absolute().left(0.0).top(0.0))
        .width(Size::fill())
        .height(Size::percent(100.0))
        .layer(Layer::Overlay)
        .painted(&paints, &style.scrim)
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .child(SheetPresence)
        .child(crate::controls::shadowed(
            rect()
                .a11y_role(AccessibilityRole::Dialog)
                .padding(Gaps::new_all(style.padding))
                .corner_radius(
                    style
                        .shape
                        .corner_radius(0.0, 0.0, LayoutDirection::LeftToRight),
                )
                .painted(&paints, &style.background)
                .map(style.border.as_ref(), |el, (paint, width)| {
                    el.border(
                        Border::new()
                            .width(*width)
                            .alignment(BorderAlignment::Outer)
                            .fill(color(paints.color(paint))),
                    )
                })
                .child(content),
            &style.shadows,
        ))
}

#[derive(PartialEq)]
pub struct Alert {
    style: Theme,
    title: &'static str,
    message: String,
    confirm: &'static str,
    cancel: &'static str,
    on_confirm: Option<Callback<(), ()>>,
    on_cancel: Option<Callback<(), ()>>,
    suppressed: Option<State<bool>>,
}

impl Alert {
    pub fn new(title: &'static str, message: impl Into<String>) -> Self {
        Self {
            style: Theme::tengen(),
            title,
            message: message.into(),
            confirm: text!("ok"),
            cancel: text!("cancel"),
            on_confirm: None,
            on_cancel: None,
            suppressed: None,
        }
    }

    pub fn style(mut self, restyle: impl Fn(AlertStyle) -> AlertStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn suppression(mut self, suppressed: State<bool>) -> Self {
        self.suppressed = Some(suppressed);
        self
    }

    pub fn confirm(mut self, label: &'static str, callback: impl Into<Callback<(), ()>>) -> Self {
        self.confirm = label;
        self.on_confirm = Some(callback.into());
        self
    }

    pub fn cancel(mut self, label: &'static str, callback: impl Into<Callback<(), ()>>) -> Self {
        self.cancel = label;
        self.on_cancel = Some(callback.into());
        self
    }
}

impl Component for Alert {
    fn render(&self) -> impl IntoElement {
        let style: AlertStyle = resolve(&self.style);
        let sheet_style: SheetStyle = resolve(&Theme::tengen());
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let call = |callback: &Option<Callback<(), ()>>| {
            let callback = callback.clone();
            move || {
                if let Some(cb) = &callback {
                    cb.call(());
                }
            }
        };
        let (confirm, cancel) = (call(&self.on_confirm), call(&self.on_cancel));
        let (on_confirm, on_cancel) = (confirm.clone(), cancel.clone());

        sheet(
            rect()
                .a11y_role(AccessibilityRole::AlertDialog)
                .a11y_alt(self.title)
                .a11y_focusable(true)
                .a11y_auto_focus(true)
                .width(Size::px(style.width - 2.0 * sheet_style.padding))
                .direction(Direction::Vertical)
                .spacing(style.spacing)
                .on_global_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::Enter) => confirm(),
                    Key::Named(NamedKey::Escape) => cancel(),
                    _ => {}
                })
                .child(text(
                    self.title,
                    style.title_font,
                    paints.color(style.title.resolve(state)),
                ))
                .child(
                    text(
                        self.message.clone(),
                        style.message_font,
                        paints.color(style.message.resolve(state)),
                    )
                    .max_lines(None),
                )
                .maybe_child(self.suppressed.map(|mut suppressed| {
                    Checkbox::new(text!("alert-suppress"))
                        .checked(*suppressed.read())
                        .on_toggle(move |checked| suppressed.set(checked))
                }))
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Horizontal)
                        .content(Content::flex())
                        .margin(Gaps::new(style.buttons_margin_top, 0.0, 0.0, 0.0))
                        .spacing(style.buttons_spacing)
                        .child(
                            Button::new(self.cancel)
                                .size(ControlSize::Large)
                                .width(Size::flex(1.0))
                                .on_press(move |_| on_cancel()),
                        )
                        .child(
                            Button::new(self.confirm)
                                .style(ButtonStyle::bordered_prominent)
                                .size(ControlSize::Large)
                                .width(Size::flex(1.0))
                                .on_press(move |_| on_confirm()),
                        ),
                ),
            &sheet_style,
        )
    }
}
