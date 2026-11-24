use super::*;
use crate::titlebar::WindowToolbarStyle;

#[derive(PartialEq)]
pub struct ToolGroup {
    children: Vec<Element>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
    overrides: Overrides,
}

impl Default for ToolGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolGroup {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            probe: None,
            style: Theme::tengen(),
            key: DiffKey::None,
            overrides: Overrides::default(),
        }
    }

    pub fn style(mut self, restyle: impl Fn(ToolGroupStyle) -> ToolGroupStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }
}

impl ChildrenExt for ToolGroup {
    fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl KeyExt for ToolGroup {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl ProbeExt for ToolGroup {
    fn probe(mut self, id: &'static str) -> Self {
        self.probe = Some(id);
        self
    }
}

forward_overrides!(ToolGroup);

impl Component for ToolGroup {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        crate::compact::use_toolbar_items_or(WindowToolbarStyle::Unified);
        crate::compact::use_toolbar_item_alone(self.children.len() == 1);
        let style: ToolGroupStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let vertical = style.axis == Axis::Vertical;
        let (length, thickness) = style.item_size;
        let (slot_width, slot_height) = if vertical {
            (thickness, length)
        } else {
            (length, thickness)
        };
        let (along, across) = style.separator_size;
        let (separator_width, separator_height) = if vertical {
            (across, along)
        } else {
            (along, across)
        };

        let last = self.children.len().saturating_sub(1);
        let items = self
            .children
            .iter()
            .cloned()
            .enumerate()
            .flat_map(|(i, child)| {
                let slot = rect()
                    .width(Size::px(slot_width))
                    .height(Size::px(slot_height))
                    .center()
                    .child(child)
                    .into_element();
                let separator = style
                    .separator
                    .as_ref()
                    .filter(|_| i != last)
                    .map(|separator| {
                        rect()
                            .width(Size::px(separator_width))
                            .height(Size::px(separator_height))
                            .background(paints.fill(separator))
                            .into_element()
                    });
                std::iter::once(slot).chain(separator)
            });

        let padding = if self.children.len() == 1 {
            style.padding_alone
        } else {
            style.padding
        };
        let root = if vertical {
            rect()
                .direction(Direction::Vertical)
                .width(Size::px(thickness))
                .padding(Gaps::new(padding, 0.0, padding, 0.0))
        } else {
            rect()
                .direction(Direction::Horizontal)
                .height(Size::px(thickness))
                .padding(Gaps::new(0.0, padding, 0.0, padding))
        };
        let corner_radius = style
            .shape
            .corner_radius(slot_width, slot_height, ambient());
        let root = root
            .cross_align(Alignment::Center)
            .children(items)
            .map(self.probe, ProbeExt::probe);
        if crate::glass::native_glass() {
            let count = self.children.len() as f32;
            let separators = if style.separator.is_some() {
                count - 1.0
            } else {
                0.0
            };
            let along = 2.0 * padding + count * slot_width + separators * separator_width;
            // The native glass draws its HAIRLINE rim outside its frame, and the group's size includes the rim.
            let glass_size = if vertical {
                (
                    thickness,
                    2.0 * padding + count * slot_height + separators * separator_height,
                )
            } else {
                (along, thickness)
            };
            let glass_size = (glass_size.0 - 2.0 * HAIRLINE, glass_size.1 - 2.0 * HAIRLINE);
            let (width, height) = if vertical {
                (Size::px(thickness), Size::auto())
            } else {
                (Size::auto(), Size::px(thickness))
            };
            let glass = rect()
                .width(width)
                .height(height)
                .corner_radius(corner_radius)
                .child(crate::platform::backdrop::Declared::glass(
                    style
                        .shape
                        .corner_radius(glass_size.0, glass_size.1, ambient())
                        .top_left(),
                    glass_size,
                    (HAIRLINE, HAIRLINE),
                    crate::platform::backdrop::GlassLook::default(),
                ))
                .child(root);
            return self.overrides.apply(dimmed(glass, style.opacity));
        }
        let root = root
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .corner_radius(corner_radius);
        self.overrides
            .apply(dimmed(shadowed(root, &style.shadows), style.opacity))
    }
}
