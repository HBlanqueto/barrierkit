use super::*;

#[derive(PartialEq)]
pub struct ToolGroup {
    children: Vec<Element>,
    probe: Option<&'static str>,
    style: Theme,
    key: DiffKey,
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
        }
    }

    pub fn style(mut self, restyle: impl Fn(ToolGroupStyle) -> ToolGroupStyle + 'static) -> Self {
        self.style = self.style.style(restyle);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_element));
        self
    }

    pub fn key(mut self, key: impl Into<DiffKey>) -> Self {
        self.key = key.into();
        self
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

impl Component for ToolGroup {
    fn render(&self) -> impl IntoElement {
        crate::compact::use_toolbar_items(true);
        let style: ToolGroupStyle = resolve(&self.style);
        let paints = Paints::current(ControlState::default(), style.effect);
        let vertical = style.axis == Axis::Vertical;
        let slot = style.item_size;
        let (along, across) = style.separator_size;
        let (separator_width, separator_height) = if vertical {
            (across, along)
        } else {
            (along, across)
        };

        let mut items: Vec<Element> = Vec::with_capacity(self.children.len());
        let last = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().cloned().enumerate() {
            items.push(
                rect()
                    .width(Size::px(slot))
                    .height(Size::px(slot))
                    .center()
                    .child(child)
                    .into_element(),
            );
            if let Some(separator) = style.separator.as_ref().filter(|_| i != last) {
                items.push(
                    rect()
                        .width(Size::px(separator_width))
                        .height(Size::px(separator_height))
                        .background(paints.fill(separator))
                        .into_element(),
                );
            }
        }

        let padding = style.padding;
        let root = if vertical {
            rect()
                .direction(Direction::Vertical)
                .width(Size::px(slot))
                .padding(Gaps::new(padding, 0.0, padding, 0.0))
        } else {
            rect()
                .direction(Direction::Horizontal)
                .height(Size::px(slot))
                .padding(Gaps::new(0.0, padding, 0.0, padding))
        };
        let root = root
            .cross_align(Alignment::Center)
            .children(items)
            .map(self.probe, ProbeExt::probe)
            .painted(&paints, &style.background)
            .bordered(&paints, style.border.as_ref())
            .corner_radius(
                style
                    .shape
                    .corner_radius(slot, slot, LayoutDirection::LeftToRight),
            );
        dimmed(shadowed(root, &style.shadows), style.opacity)
    }
}
