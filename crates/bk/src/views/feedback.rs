use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use freya_core::integration::AppComponent;

use ctdesign::{
    DIALOG_ALERT_BADGE, DIALOG_ALERT_CAUTION, DIALOG_ALERT_CAUTION_AT, DIALOG_ALERT_CAUTION_SIZE,
    DIALOG_ALERT_HELP_INSET, DIALOG_ALERT_HELP_SIZE, DIALOG_BUTTON_WIDTH, DIALOG_SHEET_HEIGHT,
    DIALOG_SHEET_WIDTH, SPACE_2,
};

use crate::elements::{HStack, Overrides, forward_container, forward_overrides};
use crate::platform::{WindowControls, WindowOptions};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Content, Direction, Gaps, Position, Size};

use crate::ProbeExt;
use crate::buttons::HelpLink;
use crate::controls::{Button, Checkbox, ControlSize};
use crate::i18n::text;
use crate::style::text;
use crate::styles::Painted;
use crate::styles::{
    AlertStyle, ButtonStyle, ContentUnavailableViewStyle, ControlState, HelpLinkStyle,
    LayoutDirection, Paints, SheetStyle, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

pub struct ContentUnavailableView(
    Rect,
    Cow<'static, str>,
    Symbol,
    Option<Cow<'static, str>>,
    Theme,
);

impl ContentUnavailableView {
    pub fn new(title: impl Into<Cow<'static, str>>, symbol: Symbol) -> Self {
        Self::styled(title.into(), symbol, None, Theme::tengen())
    }

    pub fn search(query: &str) -> Self {
        Self::new(
            text!("no-results-for", query = query),
            Symbol::MagnifyingGlass,
        )
        .description(text!("check-the-spelling"))
    }

    pub fn description(self, description: impl Into<Cow<'static, str>>) -> Self {
        Self::styled(self.1, self.2, Some(description.into()), self.4)
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        let style: ContentUnavailableViewStyle = resolve(&self.4);
        self.0.children_mut().push(
            rect()
                .probe("content-unavailable-actions")
                .margin(Gaps::new(style.actions_gap, 0.0, 0.0, 0.0))
                .child(actions)
                .into_element(),
        );
        self
    }

    pub fn style(
        self,
        restyle: impl Fn(ContentUnavailableViewStyle) -> ContentUnavailableViewStyle + 'static,
    ) -> Self {
        Self::styled(self.1, self.2, self.3, self.4.style(restyle))
    }

    fn styled(
        title: Cow<'static, str>,
        icon: Symbol,
        description: Option<Cow<'static, str>>,
        theme: Theme,
    ) -> Self {
        let style: ContentUnavailableViewStyle = resolve(&theme);
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let mut view = rect()
            .expanded()
            .center()
            .direction(Direction::Vertical)
            .child(
                rect()
                    .probe("content-unavailable-symbol")
                    .height(Size::px(style.symbol_line))
                    .center()
                    .child(symbol(
                        icon,
                        style.symbol_size,
                        paints.color(style.symbol.resolve(state)),
                    )),
            )
            .child(
                rect()
                    .probe("content-unavailable-title")
                    .margin(Gaps::new(style.symbol_gap, 0.0, 0.0, 0.0))
                    .child(text(
                        title.clone(),
                        style.title_font,
                        paints.color(style.title.resolve(state)),
                    )),
            );
        if let Some(description) = description.clone() {
            view = view.child(
                rect()
                    .probe("content-unavailable-description")
                    .margin(Gaps::new(style.description_gap, 0.0, 0.0, 0.0))
                    .child(text(
                        description,
                        style.description_font,
                        paints.color(style.description.resolve(state)),
                    )),
            );
        }
        Self(view, title, icon, description, theme)
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
        let closed: Rc<Cell<bool>> = use_hook(Rc::default);
        let style: SheetStyle = resolve(&self.theme);
        let paints = Paints::current(ControlState::default(), style.effect);
        let theme = self.theme.clone();
        let parent = self.parent.0.clone();
        let registered = opened.clone();
        let unmounted = closed.clone();
        use_hook(move || {
            parent.open_sheet(
                AppComponent::from(move || {
                    sheet_window_root(shared, &theme, &registered, &unmounted)
                }),
                WindowOptions::new("", "sheet").size((DIALOG_SHEET_WIDTH, DIALOG_SHEET_HEIGHT)),
            )
        });
        use_drop(move || {
            closed.set(true);
            if let Some(window) = opened.borrow_mut().take() {
                window.close();
            }
            shared.set(rect().into_element());
        });
        let veil = rect()
            .position(Position::new_absolute().left(0.0).top(0.0))
            .width(Size::fill())
            .height(Size::percent(100.0))
            .layer(Layer::Overlay);
        let veiled_by_platform =
            try_consume_root_context::<crate::platform::backdrop::NativeSheets>()
                .is_some_and(|sheets| sheets.0);
        let veil = if veiled_by_platform {
            veil
        } else {
            veil.painted(&paints, &style.scrim)
        };
        veil.on_mouse_down(|event: Event<MouseEventData>| event.stop_propagation())
            .child(SheetPresence)
    }
}

pub(crate) fn sheet_window_root(
    content: State<Element>,
    theme: &Theme,
    opened: &Rc<RefCell<Option<WindowControls>>>,
    closed: &Rc<Cell<bool>>,
) -> Rect {
    let window = WindowControls::get();
    let registered = opened.clone();
    let own = window.clone();
    let unmounted = closed.get();
    use_hook(move || {
        if unmounted {
            own.close();
        } else {
            *registered.borrow_mut() = Some(own);
        }
    });
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
    title: Cow<'static, str>,
    message: Cow<'static, str>,
    confirm: Cow<'static, str>,
    cancel: Cow<'static, str>,
    on_confirm: Option<Callback<(), ()>>,
    on_cancel: Option<Callback<(), ()>>,
    suppressed: Option<State<bool>>,
    destructive: bool,
    icon: Option<Element>,
    extra_buttons: Vec<(Cow<'static, str>, Callback<(), ()>)>,
    on_help: Option<Callback<(), ()>>,
    accessory: Option<Element>,
    critical: bool,
    presented: Option<State<bool>>,
}

impl Alert {
    pub fn new(title: impl Into<Cow<'static, str>>, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            style: Theme::tengen(),
            title: title.into(),
            message: message.into(),
            confirm: text!("ok").into(),
            cancel: text!("cancel").into(),
            on_confirm: None,
            on_cancel: None,
            suppressed: None,
            destructive: false,
            icon: None,
            extra_buttons: Vec::new(),
            on_help: None,
            accessory: None,
            critical: false,
            presented: None,
        }
    }

    pub(crate) fn dismissing(mut self, presented: State<bool>) -> Self {
        self.presented = Some(presented);
        self
    }

    pub fn style(mut self, restyle: impl Fn(AlertStyle) -> AlertStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_element());
        self
    }

    pub fn critical(mut self) -> Self {
        self.critical = true;
        self
    }

    pub fn shows_help(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_help = Some(callback.into());
        self
    }

    pub fn accessory_view(mut self, view: impl IntoElement) -> Self {
        self.accessory = Some(view.into_element());
        self
    }

    pub fn add_button(
        mut self,
        label: impl Into<Cow<'static, str>>,
        callback: impl Into<Callback<(), ()>>,
    ) -> Self {
        self.extra_buttons.push((label.into(), callback.into()));
        self
    }

    pub fn suppression(mut self, suppressed: State<bool>) -> Self {
        self.suppressed = Some(suppressed);
        self
    }

    pub fn confirm(
        mut self,
        label: impl Into<Cow<'static, str>>,
        callback: impl Into<Callback<(), ()>>,
    ) -> Self {
        self.confirm = label.into();
        self.on_confirm = Some(callback.into());
        self
    }

    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn cancel(
        mut self,
        label: impl Into<Cow<'static, str>>,
        callback: impl Into<Callback<(), ()>>,
    ) -> Self {
        self.cancel = label.into();
        self.on_cancel = Some(callback.into());
        self
    }
}

#[derive(PartialEq)]
pub struct SheetButtons {
    cancel: Cow<'static, str>,
    on_cancel: Callback<(), ()>,
    confirm: Cow<'static, str>,
    on_confirm: Callback<(), ()>,
    confirm_disabled: bool,
    leading: Option<Element>,
    large: bool,
    equal_width: bool,
    overrides: Overrides,
}

impl SheetButtons {
    pub fn new(
        cancel: impl Into<Cow<'static, str>>,
        on_cancel: impl Into<Callback<(), ()>>,
        confirm: impl Into<Cow<'static, str>>,
        on_confirm: impl Into<Callback<(), ()>>,
    ) -> Self {
        Self {
            cancel: cancel.into(),
            on_cancel: on_cancel.into(),
            confirm: confirm.into(),
            on_confirm: on_confirm.into(),
            confirm_disabled: false,
            leading: None,
            large: false,
            equal_width: false,
            overrides: Overrides::default(),
        }
    }

    pub fn confirm_disabled(mut self, disabled: bool) -> Self {
        self.confirm_disabled = disabled;
        self
    }

    pub fn leading(mut self, view: impl IntoElement) -> Self {
        self.leading = Some(view.into_element());
        self
    }

    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    pub fn equal_width(mut self) -> Self {
        self.equal_width = true;
        self
    }
}

forward_overrides!(SheetButtons);

impl Component for SheetButtons {
    fn render(&self) -> impl IntoElement {
        let (large, equal_width) = (self.large, self.equal_width);
        let fit = move |button: Button| {
            let button = if large {
                button
                    .size(ControlSize::Large)
                    .width(Size::px(DIALOG_BUTTON_WIDTH))
            } else {
                button
            };
            if equal_width {
                button.width(Size::flex(1.0))
            } else {
                button
            }
        };
        let cancel = fit(Button::new(self.cancel.clone()).on_press(self.on_cancel.clone()));
        let confirm = fit(Button::new(self.confirm.clone())
            .style(ButtonStyle::bordered_prominent)
            .disabled(self.confirm_disabled)
            .on_press(self.on_confirm.clone()));
        let row = HStack::new()
            .width(Size::fill())
            .content(Content::flex())
            .spacing(SPACE_2);
        let root = if equal_width {
            row.child(cancel).child(confirm)
        } else {
            row.maybe_child(self.leading.clone())
                .child(rect().width(Size::flex(1.0)))
                .child(cancel)
                .child(confirm)
        };
        self.overrides.apply(root)
    }
}

impl Component for Alert {
    fn render(&self) -> impl IntoElement {
        let style: AlertStyle = resolve(&self.style);
        let sheet_style: SheetStyle = resolve(&Theme::tengen());
        let destructive = self.destructive;
        let state = ControlState::default();
        let paints = Paints::current(state, style.effect);
        let presented = self.presented;
        let dismiss = move || {
            if let Some(mut presented) = presented {
                presented.set(false);
            }
        };
        let call = |callback: &Option<Callback<(), ()>>| {
            let callback = callback.clone();
            move || {
                if let Some(cb) = &callback {
                    cb.call(());
                }
                dismiss();
            }
        };
        let (confirm, cancel) = (call(&self.on_confirm), call(&self.on_cancel));
        let (on_confirm, on_cancel) = (confirm.clone(), cancel.clone());

        let inset = sheet_style.padding - style.padding;
        let padding = style.padding;
        let gap = style.buttons_margin_top;
        let stacked = !self.extra_buttons.is_empty();
        let button_width = if stacked {
            Size::fill()
        } else {
            Size::flex(1.0)
        };
        let cancel_button = Button::new(self.cancel.clone())
            .size(ControlSize::Large)
            .width(button_width.clone())
            .on_press(move |_| on_cancel());
        let confirm_button = Button::new(self.confirm.clone())
            .style(move |button| {
                if destructive {
                    button.destructive()
                } else {
                    button.bordered_prominent()
                }
            })
            .size(ControlSize::Large)
            .width(button_width.clone())
            .on_press(move |_| on_confirm());
        let buttons = rect()
            .width(Size::fill())
            .margin(Gaps::new(gap, 0.0, 0.0, 0.0));
        let buttons = if stacked {
            buttons
                .direction(Direction::Vertical)
                .spacing(style.buttons_stack_spacing)
                .child(confirm_button)
                .children(self.extra_buttons.iter().map(|(label, callback)| {
                    let callback = callback.clone();
                    Button::new(label.clone())
                        .size(ControlSize::Large)
                        .width(button_width.clone())
                        .on_press(move |_| {
                            callback.call(());
                            dismiss();
                        })
                        .into_element()
                }))
                .child(cancel_button)
        } else {
            buttons
                .direction(Direction::Horizontal)
                .content(Content::flex())
                .spacing(style.buttons_spacing)
                .child(cancel_button)
                .child(confirm_button)
        };
        let icon = self.icon.clone().map(|app_icon| {
            let icon_box = rect()
                .width(Size::px(style.icon_size))
                .height(Size::px(style.icon_size))
                .margin(Gaps::new(0.0, 0.0, style.icon_gap - style.spacing, 0.0))
                .overflow(Overflow::Clip);
            if self.critical {
                let [x, y] = DIALOG_ALERT_CAUTION_AT;
                icon_box
                    .child(
                        rect()
                            .position(Position::new_absolute().left(x).top(y))
                            .child(symbol(
                                Symbol::ExclamationMarkTriangleFill,
                                DIALOG_ALERT_CAUTION_SIZE,
                                DIALOG_ALERT_CAUTION,
                            )),
                    )
                    .child(
                        rect()
                            .position(
                                Position::new_absolute()
                                    .left(style.icon_size - DIALOG_ALERT_BADGE)
                                    .top(style.icon_size - DIALOG_ALERT_BADGE),
                            )
                            .width(Size::px(DIALOG_ALERT_BADGE))
                            .height(Size::px(DIALOG_ALERT_BADGE))
                            .overflow(Overflow::Clip)
                            .child(app_icon),
                    )
            } else {
                icon_box.child(app_icon)
            }
        });
        let help = self.on_help.clone().map(|callback| {
            let inset = DIALOG_ALERT_HELP_INSET - padding;
            rect()
                .position(Position::new_absolute().top(inset).right(inset))
                .child(
                    HelpLink::new()
                        .style(|help| HelpLinkStyle {
                            glyph: (
                                help.glyph.0 * DIALOG_ALERT_HELP_SIZE / help.size,
                                help.glyph.1 * DIALOG_ALERT_HELP_SIZE / help.size,
                            ),
                            size: DIALOG_ALERT_HELP_SIZE,
                            ..help
                        })
                        .on_press(callback),
                )
        });
        Sheet::new(
            rect()
                .a11y_role(AccessibilityRole::AlertDialog)
                .a11y_alt(&*self.title)
                .a11y_focusable(true)
                .a11y_auto_focus(true)
                .width(Size::px(style.width - 2.0 * style.padding))
                .direction(Direction::Vertical)
                .on_global_key_down(move |event: Event<KeyboardEventData>| match event.key {
                    Key::Named(NamedKey::Enter) => confirm(),
                    Key::Named(NamedKey::Escape) => cancel(),
                    _ => {}
                })
                .maybe_child(help)
                .child(
                    rect()
                        .width(Size::fill())
                        .direction(Direction::Vertical)
                        .spacing(style.spacing)
                        .padding(Gaps::new(inset, inset, 0.0, inset))
                        .maybe_child(icon)
                        .child(text(
                            self.title.clone(),
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
                        ),
                )
                .maybe_child(self.accessory.clone().map(|view| {
                    rect()
                        .width(Size::fill())
                        .margin(Gaps::new(gap, 0.0, 0.0, 0.0))
                        .child(view)
                }))
                .maybe_child(self.suppressed.map(|mut suppressed| {
                    rect().margin(Gaps::new(gap, 0.0, 0.0, inset)).child(
                        Checkbox::new(text!("alert-suppress"))
                            .checked(*suppressed.read())
                            .on_toggle(move |checked| suppressed.set(checked)),
                    )
                }))
                .child(buttons),
        )
        .style(move |sheet| SheetStyle {
            padding,
            ..sheet.alert()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_search_title_carries_the_query() {
        assert!(text!("no-results-for", query = "wifi").contains("wifi"));
    }
}
