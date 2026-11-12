use std::cell::RefCell;
use std::rc::Rc;

use freya_core::integration::AppComponent;

use ctdesign::{DIALOG_SHEET_HEIGHT, DIALOG_SHEET_WIDTH};

use crate::elements::forward_container;
use crate::platform::{WindowControls, WindowOptions};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::controls::{Button, Checkbox, ControlSize};
use crate::i18n::text;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    AlertStyle, ContentUnavailableViewStyle, ControlState, LayoutDirection, Paints, SheetStyle,
    Theme, resolve,
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

#[derive(PartialEq)]
pub struct Sheet {
    content: Element,
    theme: Theme,
}

// A window-modal sheet leaves minimize and zoom working but disables close, as in AppKit.
#[derive(Clone, Copy)]
pub(crate) struct OpenSheets(pub(crate) State<u32>);

impl OpenSheets {
    pub(crate) fn any() -> bool {
        try_consume_root_context::<OpenSheets>().is_some_and(|OpenSheets(open)| *open.read() > 0)
    }
}

// AppKit: under a sheet the window still drags by its title bar and minimizes and zooms.
pub(crate) const SHEET_TITLEBAR: Layer = Layer::OverlayLevel(2);
pub(crate) const SHEET_TRAFFIC_LIGHTS: Layer = Layer::OverlayLevel(3);

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
        Self {
            content: content.into_element(),
            theme: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(SheetStyle) -> SheetStyle + 'static) -> Self {
        self.theme = self.theme.style(restyle);
        self
    }
}

impl Component for Sheet {
    fn render(&self) -> impl IntoElement {
        let window = use_hook(|| {
            try_consume_root_context::<WindowControls>().filter(|controls| !controls.panel)
        });
        match window {
            Some(parent) => SheetWindow {
                content: self.content.clone(),
                theme: self.theme.clone(),
                parent: Parent(parent),
            }
            .into_element(),
            None => sheet(self.content.clone(), &resolve(&self.theme)).into_element(),
        }
    }
}

struct Parent(WindowControls);

impl PartialEq for Parent {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

// Like an AppKit sheet, the dialog is a modal child window (xdg-dialog-v1); the compositor
// keeps it centered on the window that opened it.
#[derive(PartialEq)]
struct SheetWindow {
    content: Element,
    theme: Theme,
    parent: Parent,
}

impl Component for SheetWindow {
    fn render(&self) -> impl IntoElement {
        let first = self.content.clone();
        // Global: the sheet window may render once more after this component unmounts.
        let mut shared = use_hook(move || State::create_global(first));
        if *shared.peek() != self.content {
            shared.set(self.content.clone());
        }
        let opened: Rc<RefCell<Option<WindowControls>>> = use_hook(Rc::default);
        let style: SheetStyle = resolve(&self.theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let theme = self.theme.clone();
        let parent = self.parent.0.clone();
        let registered = opened.clone();
        use_hook(move || {
            parent.open_sheet(
                AppComponent::from(move || sheet_window_root(shared, &theme, &registered)),
                WindowOptions::new("", "sheet").size((DIALOG_SHEET_WIDTH, DIALOG_SHEET_HEIGHT)),
            )
        });
        use_drop(move || {
            if let Some(window) = opened.borrow_mut().take() {
                window.close();
            }
            shared.set(rect().into_element());
        });
        rect()
            .position(Position::new_absolute().left(0.0).top(0.0))
            .width(Size::fill())
            .height(Size::percent(100.0))
            .layer(Layer::Overlay)
            .painted(&paints, &style.scrim)
            .on_mouse_down(|event: Event<MouseEventData>| event.stop_propagation())
            .child(SheetPresence)
    }
}

fn sheet_window_root(
    content: State<Element>,
    theme: &Theme,
    opened: &Rc<RefCell<Option<WindowControls>>>,
) -> Rect {
    let window = WindowControls::get();
    let registered = opened.clone();
    let own = window.clone();
    use_hook(move || *registered.borrow_mut() = Some(own));
    let style: SheetStyle = resolve(theme);
    let paints = Paints::current(ControlState::default(), style.effect);
    rect().expanded().painted(&paints, &style.background).child(
        rect()
            .padding(Gaps::new_all(style.padding))
            .on_sized(move |event: Event<SizedEventData>| {
                let size = event.area.size;
                window.fit((size.width.ceil() as u32, size.height.ceil() as u32));
            })
            .child(content.read().clone()),
    )
}

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
    destructive: bool,
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
            destructive: false,
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

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
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
        let destructive = self.destructive;
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

        Sheet::new(
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
                                .style(move |button| {
                                    if destructive {
                                        button.destructive()
                                    } else {
                                        button.bordered_prominent()
                                    }
                                })
                                .size(ControlSize::Large)
                                .width(Size::flex(1.0))
                                .on_press(move |_| on_confirm()),
                        ),
                ),
        )
    }
}
