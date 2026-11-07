use ctdesign::{BODY, BUTTON, SETTINGS, SPACE_1};
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Size};

use crate::i18n::text;
use crate::interaction::use_interaction;
use crate::menu::MenuItem;
use crate::platform::PopupAnchor;
use crate::popup_button::open_popover_menu;
use crate::style::{Palette, text};
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct MenuButton {
    label: &'static str,
    items: Vec<MenuItem>,
    on_activate: Option<Callback<String, ()>>,
}

impl MenuButton {
    pub fn new(label: &'static str, items: Vec<MenuItem>) -> Self {
        Self {
            label,
            items,
            on_activate: None,
        }
    }

    pub fn on_activate(mut self, callback: impl Into<Callback<String, ()>>) -> Self {
        self.on_activate = Some(callback.into());
        self
    }
}

impl Component for MenuButton {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let interaction = use_interaction();
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let on_activate = self.on_activate.clone();
        let background = if interaction.hovered() {
            palette.control_hover
        } else {
            palette.control_background
        };
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .a11y_builder(|node| node.set_has_popup(accesskit::HasPopup::Menu))
            .height(Size::px(BUTTON.height))
            .padding(Gaps::new(0.0, BUTTON.padding_x, 0.0, BUTTON.padding_x))
            .corner_radius(BUTTON.radius)
            .background(color(background))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(SPACE_1)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .on_press(move |_| {
                let field = *area.peek();
                let on_activate = on_activate.clone();
                open_popover_menu(
                    items.clone(),
                    PopupAnchor::BottomLeft,
                    (
                        field.origin.x.round() as i32,
                        field.origin.y.round() as i32,
                        field.size.width.round() as u32,
                        field.size.height.round() as u32,
                    ),
                    None,
                    move |id| {
                        if let Some(on_activate) = &on_activate {
                            on_activate.call(id);
                        }
                    },
                );
            })
            .child(text(self.label, BODY, palette.label))
            .child(symbol(
                Symbol::ChevronDown,
                SETTINGS.dropdown_chevron_glyph as f32,
                palette.secondary_label,
            ));
        interaction.focus_ring(button, BUTTON.radius)
    }
}

#[derive(PartialEq)]
pub struct ComboButton {
    label: &'static str,
    items: Vec<MenuItem>,
    on_press: Option<Callback<(), ()>>,
    on_activate: Option<Callback<String, ()>>,
}

impl ComboButton {
    pub fn new(label: &'static str, items: Vec<MenuItem>) -> Self {
        Self {
            label,
            items,
            on_press: None,
            on_activate: None,
        }
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
        let palette = Palette::current();
        let action = use_interaction();
        let arrow = use_interaction();
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let on_press = self.on_press.clone();
        let on_activate = self.on_activate.clone();
        let fill = |hovered: bool| {
            if hovered {
                color(palette.control_hover)
            } else {
                Color::TRANSPARENT
            }
        };
        let primary = action
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.label)
            .height(Size::fill())
            .padding(Gaps::new(0.0, BUTTON.padding_x, 0.0, BUTTON.padding_x))
            .center()
            .background(fill(action.hovered()))
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(text(self.label, BODY, palette.label));
        let menu = arrow
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(text!("more-options"))
            .a11y_builder(|node| node.set_has_popup(accesskit::HasPopup::Menu))
            .height(Size::fill())
            .padding(Gaps::new(
                0.0,
                BUTTON.menu_padding_x,
                0.0,
                BUTTON.menu_padding_x,
            ))
            .center()
            .background(fill(arrow.hovered()))
            .on_press(move |_| {
                let field = *area.peek();
                let on_activate = on_activate.clone();
                open_popover_menu(
                    items.clone(),
                    PopupAnchor::BottomLeft,
                    (
                        field.origin.x.round() as i32,
                        field.origin.y.round() as i32,
                        field.size.width.round() as u32,
                        field.size.height.round() as u32,
                    ),
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
                SETTINGS.dropdown_chevron_glyph as f32,
                palette.secondary_label,
            ));
        rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .height(Size::px(BUTTON.height))
            .corner_radius(BUTTON.radius)
            .overflow(Overflow::Clip)
            .background(color(palette.control_background))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(action.focus_ring(primary, BUTTON.radius))
            .child(
                rect()
                    .width(Size::px(ctdesign::HAIRLINE))
                    .height(Size::px(BUTTON.height / 2.0))
                    .background(color(palette.separator)),
            )
            .child(arrow.focus_ring(menu, BUTTON.radius))
    }
}

#[derive(PartialEq)]
pub struct HelpLink {
    on_press: Option<Callback<(), ()>>,
}

impl HelpLink {
    pub fn new() -> Self {
        Self { on_press: None }
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
        let palette = Palette::current();
        let interaction = use_interaction();
        let on_press = self.on_press.clone();
        let size = BUTTON.height;
        let background = if interaction.hovered() {
            palette.control_hover
        } else {
            palette.control_background
        };
        let button = interaction
            .attach(rect(), true)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(text!("help"))
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(size / 2.0)
            .center()
            .background(color(background))
            .maybe(interaction.pressed(), |el| el.scale(BUTTON.pressed_scale))
            .map(on_press, |el, on_press| {
                el.on_press(move |_| on_press.call(()))
            })
            .child(text("?", BODY, palette.label));
        interaction.focus_ring(button, size / 2.0)
    }
}

#[derive(PartialEq)]
pub struct ControlGroup {
    label: &'static str,
    children: Vec<Element>,
}

impl ControlGroup {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }
}

impl Component for ControlGroup {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let mut items = Vec::with_capacity(self.children.len() * 2);
        for (index, child) in self.children.iter().enumerate() {
            if index > 0 {
                items.push(
                    rect()
                        .width(Size::px(ctdesign::HAIRLINE))
                        .height(Size::px(BUTTON.height / 2.0))
                        .background(color(palette.separator))
                        .into_element(),
                );
            }
            items.push(child.clone());
        }
        rect()
            .a11y_role(AccessibilityRole::Group)
            .a11y_alt(self.label)
            .height(Size::px(BUTTON.height))
            .corner_radius(BUTTON.radius)
            .background(color(palette.control_background))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .children(items)
    }
}

#[derive(PartialEq)]
pub struct ShareLink {
    label: &'static str,
    content: String,
}

impl ShareLink {
    pub fn new(label: &'static str, content: impl Into<String>) -> Self {
        Self {
            label,
            content: content.into(),
        }
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
        MenuButton::new(self.label, items).on_activate(move |id: String| {
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
