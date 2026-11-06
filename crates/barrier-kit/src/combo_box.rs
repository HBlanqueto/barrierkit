use freya_components::input::InputLayoutThemePartial;
use freya_components::theming::macros::Preference;
use freya_core::prelude::*;
use torin::prelude::{Area, Gaps, Position, Size};
use vow_design::{COMBO_BOX, TEXT, TEXT_FIELD};

use crate::controls::{default_pre_key, text_style, themed_input};
use crate::menu::MenuItem;
use crate::platform::PopupAnchor;
use crate::popup_button::open_popover_menu;
use crate::style::Palette;
use crate::symbol::{Symbol, symbol};
use crate::theme::color;

#[derive(PartialEq)]
pub struct ComboBox {
    value: State<String>,
    placeholder: &'static str,
    items: Vec<&'static str>,
    width: Size,
}

impl ComboBox {
    pub fn new(value: State<String>, placeholder: &'static str) -> Self {
        Self {
            value,
            placeholder,
            items: Vec::new(),
            width: Size::fill(),
        }
    }

    pub fn item(mut self, label: &'static str) -> Self {
        self.items.push(label);
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }
}

impl Component for ComboBox {
    fn render(&self) -> impl IntoElement {
        let palette = Palette::current();
        let value = self.value;
        let mut area = use_state(Area::default);
        let items = self.items.clone();
        let margin_y = (TEXT_FIELD.height - TEXT.line_height as f32) / 2.0;
        let trailing = COMBO_BOX.button_inset_x + COMBO_BOX.button_width + COMBO_BOX.text_gap;
        let input = themed_input(
            value,
            self.placeholder,
            palette.content_background,
            palette.separator,
            TEXT_FIELD.radius,
            TEXT_FIELD.padding_x,
        )
        .theme_layout(InputLayoutThemePartial {
            corner_radius: Some(Preference::Specific(CornerRadius::new_all(
                TEXT_FIELD.radius,
            ))),
            inner_margin: Some(Preference::Specific(Gaps::new(
                margin_y,
                trailing,
                margin_y,
                TEXT_FIELD.padding_x,
            ))),
        });
        let open = move || {
            let field = *area.peek();
            let entries = items
                .iter()
                .enumerate()
                .map(|(index, label)| MenuItem::action(index.to_string(), *label))
                .collect();
            let items = items.clone();
            open_popover_menu(
                entries,
                PopupAnchor::BottomLeft,
                (
                    field.origin.x.round() as i32,
                    field.origin.y.round() as i32,
                    field.size.width.round() as u32,
                    field.size.height.round() as u32,
                ),
                None,
                move |id| {
                    if let Some(label) = id.parse::<usize>().ok().and_then(|i| items.get(i)) {
                        let mut value = value;
                        value.set((*label).to_owned());
                    }
                },
            );
        };
        let opener = open.clone();
        let input = input.on_pre_key_down(move |event: Event<KeyboardEventData>| {
            if event.key == Key::Named(NamedKey::ArrowDown) {
                event.stop_propagation();
                opener();
                return false;
            }
            default_pre_key(&event)
        });
        text_style(rect().width(self.width.clone()))
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .child(input)
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .right(COMBO_BOX.button_inset_x)
                            .top(
                                ((area.read().size.height - COMBO_BOX.button_height) / 2.0)
                                    .max(0.0),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::px(COMBO_BOX.button_width))
                            .height(Size::px(COMBO_BOX.button_height))
                            .corner_radius(COMBO_BOX.button_radius)
                            .center()
                            .background(color(palette.control_background))
                            .a11y_role(AccessibilityRole::Button)
                            .a11y_alt(self.placeholder)
                            .on_press(move |_| open())
                            .child(symbol(
                                Symbol::ChevronDown,
                                COMBO_BOX.chevron,
                                palette.label,
                            )),
                    ),
            )
    }
}
