use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use ctdesign::{ControlSize, MENU_POPOVER};
use freya_components::svg_viewer::SvgViewer;
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use torin::prelude::{Alignment, Area, Direction, Gaps, Size};

use crate::choices::Choice;
use crate::controls::{dimmed, opacity_when, shadowed};
use crate::elements::{Overrides, forward_key, forward_overrides};
use crate::environment::use_disabled;
use crate::interaction::{control_state, use_interaction};
use crate::menu::{CheckState, MenuIcon, MenuItem, MenuView};
use crate::platform::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupSpec, Popups};
use crate::style::with_font;
use crate::styles::Painted;
use crate::styles::{LayoutDirection, Paints, PopUpButtonStyle, Theme, resolve};
use crate::symbol::{Symbol, stroked_symbol};
use crate::theme::SurfaceKind;

#[derive(PartialEq)]
pub struct PopUpButton {
    label: Cow<'static, str>,
    items: Vec<(Cow<'static, str>, Option<MenuIcon>)>,
    section_headers: Vec<(usize, Cow<'static, str>)>,
    separators: Vec<usize>,
    selected: State<usize>,
    pulls_down: bool,
    disabled: bool,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl PopUpButton {
    pub fn new(label: impl Into<Cow<'static, str>>, selection: State<usize>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
            section_headers: Vec::new(),
            separators: Vec::new(),
            selected: selection,
            pulls_down: false,
            disabled: false,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn style(
        mut self,
        restyle: impl Fn(PopUpButtonStyle) -> PopUpButtonStyle + 'static,
    ) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.style(move |style: PopUpButtonStyle| style.sized(size))
    }

    pub fn item(mut self, label: impl Into<Cow<'static, str>>) -> Self {
        self.items.push((label.into(), None));
        self
    }

    pub fn icon_item(mut self, label: impl Into<Cow<'static, str>>, icon: MenuIcon) -> Self {
        self.items.push((label.into(), Some(icon)));
        self
    }

    pub fn section_header(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        self.section_headers.push((self.items.len(), title.into()));
        self
    }

    pub fn separator(mut self) -> Self {
        self.separators.push(self.items.len());
        self
    }

    pub fn pulls_down(mut self, pulls_down: bool) -> Self {
        self.pulls_down = pulls_down;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

forward_overrides!(PopUpButton);

forward_key!(PopUpButton);

impl Component for PopUpButton {
    forward_key!();

    fn render(&self) -> impl IntoElement {
        let style: PopUpButtonStyle = resolve(&self.style);
        let interaction = use_interaction();
        let disabled = use_disabled(self.disabled);
        let state = control_state(interaction, disabled);
        let paints = Paints::current(state);
        let choice = Choice(self.selected);
        let mut area = use_state(Area::default);
        let pulls_down = self.pulls_down;
        let current = if pulls_down {
            0
        } else {
            (0..self.items.len()).find(|&i| choice.is(i)).unwrap_or(0)
        };
        let items = self.items.clone();
        let section_headers = self.section_headers.clone();
        let separators = self.separators.clone();
        let selects: Vec<_> = (0..items.len()).map(|i| choice.select(i)).collect();
        let size = style.indicator_size;
        let leading_padding = style.leading_padding;
        let indicator_radius =
            style
                .indicator_shape
                .corner_radius(size, size, LayoutDirection::LeftToRight);
        let radius =
            style
                .shape
                .corner_radius(style.height, style.height, LayoutDirection::LeftToRight);
        let indicator = rect()
            .width(Size::px(size))
            .height(Size::px(size))
            .corner_radius(indicator_radius)
            .padding(Gaps::new(
                if pulls_down {
                    2.0 * style.pulldown_chevron.1
                } else {
                    0.0
                },
                0.0,
                0.0,
                0.0,
            ))
            .center()
            .painted(&paints, &style.indicator)
            .bordered(&paints, style.indicator_border.as_ref())
            .child(stroked_symbol(
                if pulls_down {
                    Symbol::ChevronDown
                } else {
                    Symbol::ChevronUpDown
                },
                if pulls_down {
                    style.pulldown_chevron.0
                } else {
                    style.chevron_size
                },
                style.chevron_stroke,
                paints.color(style.chevron.resolve(state)),
            ));

        let button = interaction
            .attach(rect(), !disabled)
            .a11y_role(AccessibilityRole::ComboBox)
            .a11y_alt(&*self.label)
            .height(Size::px(style.height))
            .padding(Gaps::new(0.0, style.trailing_padding, 0.0, leading_padding))
            .corner_radius(radius)
            .painted(&paints, &style.background)
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .spacing(style.gap)
            .on_sized(move |event: Event<SizedEventData>| area.set_if_modified(event.area))
            .maybe(!disabled, |button| {
                button.on_press(move |_| {
                    let mut content = *area.read();
                    content.origin.x += leading_padding;
                    open_menu(
                        &items,
                        &section_headers,
                        &separators,
                        current,
                        content,
                        selects.clone(),
                        pulls_down,
                    )
                })
            })
            .maybe_child(
                self.items
                    .get(current)
                    .and_then(|item| item.1.clone())
                    .and_then(|icon| button_icon(icon, style.icon_size)),
            )
            .child(
                with_font(
                    label().text(
                        self.items
                            .get(current)
                            .map(|item| item.0.clone())
                            .unwrap_or_default(),
                    ),
                    style.font,
                )
                .max_lines(1)
                .margin(Gaps::new(0.0, 0.0, 2.0 * style.label_lift, 0.0))
                .color(paints.fill(&style.label)),
            )
            .child(shadowed(indicator, &style.indicator_shadows));
        let opacity = opacity_when(disabled, style.opacity, style.disabled_opacity);
        let root = interaction.focus_ring(dimmed(button, opacity), radius.top_left());
        self.overrides.apply(root)
    }
}

fn button_icon(icon: MenuIcon, size: f32) -> Option<Element> {
    let side = Size::px(size);
    let width = Size::px(size * crate::menu::icon_aspect(&icon));
    match icon {
        MenuIcon::Svg(svg) => Some(
            SvgViewer::new(Bytes::from(svg.as_bytes().to_vec()))
                .width(width)
                .height(side)
                .show_loader(false)
                .into_element(),
        ),
        MenuIcon::Swatch(fill) => Some(
            rect()
                .width(side.clone())
                .height(side)
                .corner_radius(size / 2.0)
                .background(crate::theme::color(fill))
                .into_element(),
        ),
    }
}

fn open_menu(
    items: &[(Cow<'static, str>, Option<MenuIcon>)],
    section_headers: &[(usize, Cow<'static, str>)],
    separators: &[usize],
    current: usize,
    area: Area,
    selects: Vec<Callback<(), ()>>,
    pulls_down: bool,
) {
    let mut entries = Vec::new();
    let mut current_entry = 0;
    for (index, (label, icon)) in items.iter().enumerate() {
        if pulls_down && index == 0 {
            continue;
        }
        if separators.contains(&index) {
            entries.push(MenuItem::Separator);
        }
        for (_, title) in section_headers.iter().filter(|(at, _)| *at == index) {
            if !entries.is_empty() {
                entries.push(MenuItem::Separator);
            }
            entries.push(MenuItem::header(title.clone()));
        }
        if index == current {
            current_entry = entries.len();
        }
        entries.push(
            MenuItem::action(index.to_string(), label.clone())
                .with_check(CheckState::Checkbox(!pulls_down && index == current))
                .with_icon(icon.clone()),
        );
    }
    open_popover_menu(
        entries,
        if pulls_down {
            PopupAnchor::BottomLeft
        } else {
            PopupAnchor::TopLeft
        },
        crate::platform::anchor_rect(area),
        (!pulls_down).then_some(current_entry),
        move |id| {
            if let Some(select) = id.parse::<usize>().ok().and_then(|i| selects.get(i)) {
                select.call(());
            }
        },
    );
}

pub(crate) fn open_popover_menu(
    entries: Vec<MenuItem>,
    anchor: PopupAnchor,
    anchor_rect: (i32, i32, u32, u32),
    over: Option<usize>,
    on_activate: impl Fn(String) + 'static,
) {
    // The compositor draws its glass under a kit client's menus (Malaysia ADR 0071), in a window too.
    let palette = crate::theme::over_glass(crate::styles::popover_style());
    let size = MenuView::preferred_size(&entries, palette);
    let offset = over.map_or((0, 0), |index| {
        let above: f32 = entries[..index]
            .iter()
            .map(|item| item.height(palette))
            .sum();
        let inset =
            MENU_POPOVER.padding + crate::menu::last_leading_column_inset(&entries, palette);
        (
            -(inset.round() as i32),
            -((MENU_POPOVER.padding + above).round() as i32),
        )
    });
    let popups = Popups::get();
    let root = Rc::new(Cell::new(None));
    let close_root = {
        let (popups, root) = (popups.clone(), root.clone());
        move || {
            if let Some(id) = root.get() {
                popups.close(id);
            }
        }
    };
    let view = MenuView::new(entries, on_activate, close_root)
        .with_surface(SurfaceKind::Toplevel)
        .with_popover_metrics()
        .style(crate::theme::over_glass);
    let spec = PopupSpec {
        anchor_rect,
        anchor,
        gravity: PopupGravity::BottomRight,
        offset,
        size: (size.0.ceil() as u32, size.1.ceil() as u32),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_x: true,
            flip_y: over.is_none(),
            slide_x: true,
            slide_y: true,
            resize_y: false,
        },
        grab: true,
        on_dismissed: None,
    };
    root.set(Some(popups.open(spec, AppComponent::new(view))));
}
