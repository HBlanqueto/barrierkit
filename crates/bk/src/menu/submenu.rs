use super::*;

pub(super) fn item_top(index: usize, items: &[MenuItem], m: MenuPalette) -> f32 {
    items[..index].iter().map(|item| item.height(m)).sum()
}

pub(super) fn submenu_positioner(opens_left: bool) -> (PopupAnchor, PopupGravity) {
    if opens_left {
        (PopupAnchor::TopLeft, PopupGravity::BottomLeft)
    } else {
        (PopupAnchor::TopRight, PopupGravity::BottomRight)
    }
}

pub(super) fn submenu_anchor_rect(
    index: usize,
    items: &[MenuItem],
    m: MenuPalette,
    parent_width: f32,
    shift: f32,
) -> (i32, i32, u32, u32) {
    (
        0,
        (item_top(index, items, m) + m.padding - shift) as i32,
        parent_width.max(1.0) as u32,
        m.item_height as u32,
    )
}

#[derive(Clone)]
pub(super) struct SubmenuCtx {
    pub(super) popup: Option<PopupHandle>,
    pub(super) popups: Option<Popups>,
    pub(super) m: MenuPalette,
    pub(super) size: (f32, f32),
    pub(super) palette_source: PaletteSource,
    pub(super) on_activate: Rc<dyn Fn(String)>,
    pub(super) on_dismiss_all: Rc<dyn Fn()>,
    pub(super) open_child_id: Rc<Cell<Option<PopupId>>>,
    pub(super) open_child_index: Rc<Cell<Option<usize>>>,
    pub(super) opens_left: bool,
}

impl SubmenuCtx {
    pub(super) fn close_child(
        &self,
        open_child_id: &Cell<Option<PopupId>>,
        open_child_index: &Cell<Option<usize>>,
    ) {
        if let Some(id) = open_child_id.get() {
            if let Some(popups) = &self.popups {
                popups.close(id);
            }
            open_child_id.set(None);
        }
        open_child_index.set(None);
    }

    fn child_view(&self, items: Vec<MenuItem>) -> MenuView {
        let on_activate = self.on_activate.clone();
        let on_dismiss_all = self.on_dismiss_all.clone();
        let view = MenuView::new(items, move |id| on_activate(id), move || on_dismiss_all());
        self.palette_source.inherit(view)
    }
}

pub(super) fn open_child_popup(
    ctx: &SubmenuCtx,
    index: usize,
    sub_items: &[MenuItem],
    parent_items: &[MenuItem],
) -> Option<PopupId> {
    let popup = ctx.popup.as_ref()?;
    let (open_child_id, open_child_index) = (&ctx.open_child_id, &ctx.open_child_index);
    if open_child_id.get().is_some() && open_child_index.get() == Some(index) {
        return open_child_id.get();
    }
    ctx.close_child(open_child_id, open_child_index);
    let m = ctx.m;
    let child_size = MenuView::preferred_size(sub_items, m);
    let open_child_id_set = open_child_id.clone();
    let open_child_index_set = open_child_index.clone();
    let dismissed_id = Rc::new(Cell::new(None::<PopupId>));
    let on_dismissed = {
        let dismissed_id = dismissed_id.clone();
        move || {
            let expected = dismissed_id.get();
            let current = open_child_id_set.get();
            if expected == current {
                open_child_id_set.set(None);
                open_child_index_set.set(None);
            }
        }
    };
    let inset = m.padding;
    let (anchor, gravity) = submenu_positioner(ctx.opens_left);
    let spec = PopupSpec {
        anchor_rect: submenu_anchor_rect(index, parent_items, m, ctx.size.0, popup.scroll_shift()),
        anchor,
        gravity,
        offset: (0, -(inset as i32)),
        size: (child_size.0 as u32, child_size.1 as u32),
        constraint_adjustment: PopupConstraintAdjustment {
            flip_x: true,
            slide_y: true,
            resize_y: true,
            ..PopupConstraintAdjustment::default()
        },
        grab: true,
        on_dismissed: Some(Rc::new(on_dismissed)),
    };
    let child_view = ctx
        .child_view(sub_items.to_vec())
        .with_popup_spec(spec.clone());
    let id = popup.open_child(spec, AppComponent::new(child_view));
    dismissed_id.set(Some(id));
    open_child_id.set(Some(id));
    open_child_index.set(Some(index));
    Some(id)
}
