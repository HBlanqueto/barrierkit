use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Size};

use crate::controls::{dimmed, shadowed};
use crate::i18n::text;
use crate::interaction::{control_state, use_interaction};
use crate::menu::MenuItem;
use crate::platform::PopupAnchor;
use crate::popup_button::open_popover_menu;
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{
    ComboButtonStyle, ControlGroupStyle, ControlState, HelpLinkStyle, LayoutDirection,
    MenuButtonStyle, Paints, Theme, resolve,
};
use crate::symbol::{Symbol, symbol};

#[derive(PartialEq)]
pub struct MenuButton {
    label: &'static str,
    items: Vec<MenuItem>,
    on_activate: Option<Callback<String, ()>>,
    style: Theme,
}

impl MenuButton {
    pub fn new(label: &'static str, items: Vec<MenuItem>) -> Self {
        Self {
            label,
            items,
            on_activate: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(MenuButtonStyle) -> MenuButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_activate(mut self, callback: impl Into<Callback<String, ()>>) -> Self {
        self.on_activate = Some(callback.into());
        self
    }
}

impl Component for MenuButton {
    fn render(&self) -> impl IntoElement {
        let style: MenuButtonStyle = resolve(&self.style);
        let interaction = use_interaction();
        let state = control_state(interaction, false);
        let paints = Paints::current(state, style.effect);
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let on_activate = self.on_activate.clone();
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(|node| node.set_has_popup(accesskit::HasPopup::Menu))
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .corner_radius(radius)
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.chevron_gap)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| {
                let field = *area.peek();
                let on_activate = on_activate.clone();
                open_popover_menu(
                    items.clone(),
                    PopupAnchor::BottomLeft,
                    crate::platform::anchor_rect(field),
                    None,
                    move |id| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.call(id);
                        }
                    },
                );
            })
            .child(
                with_font(label().text(self.label), style.font)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            )
            .child(symbol(
                Symbol::ChevronDown,
                style.chevron_size,
                paints.color(style.chevron.resolve(state)),
            ));
        interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), style.opacity),
            radius.top_left(),
        )
    }
}

#[derive(PartialEq)]
pub struct ComboButton {
    label: &'static str,
    items: Vec<MenuItem>,
    on_press: Option<Callback<(), ()>>,
    on_activate: Option<Callback<String, ()>>,
    style: Theme,
}

impl ComboButton {
    pub fn new(label: &'static str, items: Vec<MenuItem>) -> Self {
        Self {
            label,
            items,
            on_press: None,
            on_activate: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ComboButtonStyle) -> ComboButtonStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }

    pub fn on_activate(mut self, callback: impl Into<Callback<String, ()>>) -> Self {
        self.on_activate = Some(callback.into());
        self
    }
}

impl Component for ComboButton {
    fn render(&self) -> impl IntoElement {
        let style: ComboButtonStyle = resolve(&self.style);
        let action = use_interaction();
        let arrow = use_interaction();
        let group = Paints::current(ControlState::default(), style.effect);
        let action_paints = Paints::current(control_state(action, false), style.effect);
        let arrow_state = control_state(arrow, false);
        let arrow_paints = Paints::current(arrow_state, style.effect);
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let on_press = self.on_press.clone();
        let on_activate = self.on_activate.clone();
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let primary = action
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .height(Size::fill())
            .padding(Gaps::new(0.0, style.padding_x, 0.0, style.padding_x))
            .center()
            .painted(&action_paints, &style.segment)
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(
                with_font(label().text(self.label), style.font)
                    .max_lines(1)
                    .color(action_paints.fill(&style.label)),
            );
        let menu = arrow
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(text!("more-options"))
            .a11y_builder(|node| node.set_has_popup(accesskit::HasPopup::Menu))
            .height(Size::fill())
            .padding(Gaps::new(
                0.0,
                style.menu_padding_x,
                0.0,
                style.menu_padding_x,
            ))
            .center()
            .painted(&arrow_paints, &style.segment)
            .on_press(move |_| {
                let field = *area.peek();
                let on_activate = on_activate.clone();
                open_popover_menu(
                    items.clone(),
                    PopupAnchor::BottomLeft,
                    crate::platform::anchor_rect(field),
                    None,
                    move |id| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.call(id);
                        }
                    },
                );
            })
            .child(symbol(
                Symbol::ChevronDown,
                style.chevron_size,
                arrow_paints.color(style.chevron.resolve(arrow_state)),
            ));
        let (separator_width, separator_height) = style.separator_size;
        let combo = rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .height(Size::px(style.height))
            .corner_radius(radius)
            .overflow(Overflow::Clip)
            .painted(&group, &style.background)
            .bordered(&group, style.border.as_ref())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(action.focus_ring(primary, radius.top_left()))
            .child(
                rect()
                    .width(Size::px(separator_width))
                    .height(Size::px(separator_height))
                    .background(group.fill(&style.separator)),
            )
            .child(arrow.focus_ring(menu, radius.top_left()));
        dimmed(shadowed(combo, &style.shadows), style.opacity)
    }
}

#[derive(PartialEq)]
pub struct HelpLink {
    on_press: Option<Callback<(), ()>>,
    style: Theme,
}

impl HelpLink {
    pub fn new() -> Self {
        Self {
            on_press: None,
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(HelpLinkStyle) -> HelpLinkStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn on_press(mut self, callback: impl Into<Callback<(), ()>>) -> Self {
        self.on_press = Some(callback.into());
        self
    }
}

impl Default for HelpLink {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HelpLink {
    fn render(&self) -> impl IntoElement {
        let style: HelpLinkStyle = resolve(&self.style);
        let interaction = use_interaction();
        let paints = Paints::current(control_state(interaction, false), style.effect);
        let scale = crate::animation::use_animated(
            if interaction.pressed() {
                style.pressed_scale
            } else {
                1.0
            },
            style.animation,
        );
        let on_press = self.on_press.clone();
        let size = style.size;
        let radius = style
            .shape
            .corner_radius(size, size, LayoutDirection::LeftToRight);
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(text!("help"))
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(radius)
            .center()
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .maybe(scale != 1.0, |el| el.scale(scale))
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(
                with_font(label().text("?"), style.font)
                    .max_lines(1)
                    .color(paints.fill(&style.label)),
            );
        interaction.focus_ring(
            dimmed(shadowed(button, &style.shadows), style.opacity),
            radius.top_left(),
        )
    }
}

#[derive(PartialEq)]
pub struct ControlGroup {
    label: &'static str,
    children: Vec<Element>,
    style: Theme,
}

impl ControlGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            children: Vec::new(),
            style: Theme::tengen(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(ControlGroupStyle) -> ControlGroupStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

impl Component for ControlGroup {
    fn render(&self) -> impl IntoElement {
        let style: ControlGroupStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let (separator_width, separator_height) = style.separator_size;
        let mut items = Vec::with_capacity(self.children.len() * 2);
        for (index, child) in self.children.iter().enumerate() {
            if index > 0 {
                items.push(
                    rect()
                        .width(Size::px(separator_width))
                        .height(Size::px(separator_height))
                        .background(paints.fill(&style.separator))
                        .into_element(),
                );
            }
            items.push(child.clone());
        }
        let height = style.height;
        let group = rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .height(Size::px(height))
            .corner_radius(
                style
                    .shape
                    .corner_radius(height, height, LayoutDirection::LeftToRight),
            )
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .children(items);
        dimmed(shadowed(group, &style.shadows), style.opacity)
    }
}

#[derive(PartialEq)]
pub struct ShareLink {
    label: &'static str,
    content: String,
    style: Theme,
}

impl ShareLink {
    pub fn new(label: &'static str, content: impl Into<String>) -> Self {
        Self {
            label,
            content: content.into(),
            style: Theme::tengen(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(MenuButtonStyle) -> MenuButtonStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

pub fn mail_uri(content: &str) -> String {
    let body: String = content
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect();
    format!("mailto:?body={body}")
}

impl Component for ShareLink {
    fn render(&self) -> impl IntoElement {
        let content = self.content.clone();
        let is_link = content.starts_with("http://") || content.starts_with("https://");
        let mut items = vec![
            MenuItem::action("copy", text!("share-copy")),
            MenuItem::action("mail", text!("share-mail")),
        ];
        if is_link {
            items.push(MenuItem::action("open", text!("share-open-with")));
        }
        MenuButton {
            style: self.style.clone(),
            ..MenuButton::new(self.label, items)
        }
        .on_activate(move |id: String| {
            let content = content.clone();
            match id.as_str() {
                "copy" => {
                    if let Err(error) = freya_clipboard::prelude::Clipboard::set(content) {
                        tracing::warn!(?error, "BarrierKit: could not copy to the clipboard");
                    }
                }
                "mail" => {
                    spawn(async move { crate::portal::open_uri(&mail_uri(&content), false).await });
                }
                _ => {
                    spawn(async move { crate::portal::open_uri(&content, true).await });
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::mail_uri;

    #[test]
    fn the_mail_body_escapes_everything_but_unreserved_characters() {
        assert_eq!(mail_uri("Hola, año"), "mailto:?body=Hola%2C%20a%C3%B1o");
    }
}
