use super::*;

pub(super) fn item_top(index: usize, items: &[MenuItem], m: MenuPalette) -> f32 {
    let mut y = 0.0;
    for (i, item) in items.iter().enumerate() {
        if i == index {
            return y;
        }
        y += item.height(m);
    }
    0.0
}

pub(super) fn submenu_positioner(opens_left: bool) -> (PopupAnchor, PopupGravity, i32) {
    if opens_left {
        (PopupAnchor::TopLeft, PopupGravity::BottomLeft, 0)
    } else {
        (PopupAnchor::TopRight, PopupGravity::BottomRight, 0)
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

pub(super) fn close_child(
    open_child_id: Rc<Cell<Option<PopupId>>>,
    open_child_index: Rc<Cell<Option<usize>>>,
) {
    if let Some(id) = open_child_id.get() {
        if let Some(popups) = try_consume_root_context::<Popups>() {
            popups.close(id);
        }
        open_child_id.set(None);
    }
    open_child_index.set(None);
}

pub fn submenu_contents(loading: bool, items: &[MenuItem]) -> Vec<MenuItem> {
    if loading && items.is_empty() {
        vec![MenuItem::placeholder("Cargando…")]
    } else {
        items.to_vec()
    }
}

pub(super) fn announce_submenu(on_open: &Option<Rc<dyn Fn(String)>>, id: Option<&str>) {
    if let (Some(callback), Some(id)) = (on_open, id) {
        callback(id.to_string());
    }
}

pub(super) fn child_already_open(
    has_child: bool,
    open_child_index: &Cell<Option<usize>>,
    index: usize,
) -> bool {
    has_child && open_child_index.get() == Some(index)
}

#[derive(Clone)]
pub(super) struct SubmenuCtx {
    pub(super) popup: Option<PopupHandle>,
    pub(super) size: (f32, f32),
    pub(super) palette_source: PaletteSource,
    pub(super) on_activate: Rc<dyn Fn(String)>,
    pub(super) on_dismiss_all: Rc<dyn Fn()>,
    pub(super) on_open_submenu: Option<Rc<dyn Fn(String)>>,
    pub(super) submenu_source: Option<SubmenuSource>,
    pub(super) open_child_id: Rc<Cell<Option<PopupId>>>,
    pub(super) open_child_index: Rc<Cell<Option<usize>>>,
    pub(super) opens_left: bool,
}

impl SubmenuCtx {
    fn child_view(&self, items: Vec<MenuItem>) -> MenuView {
        let on_activate = self.on_activate.clone();
        let on_dismiss_all = self.on_dismiss_all.clone();
        let on_dismiss_for_action = on_dismiss_all.clone();
        let view = MenuView::new(
            items,
            move |id| {
                on_activate(id);
                on_dismiss_for_action();
            },
            move || on_dismiss_all(),
        );
        self.palette_source.inherit(view)
    }
}

pub(super) fn open_child_popup(
    ctx: &SubmenuCtx,
    index: usize,
    id: Option<&str>,
    sub_items: &[MenuItem],
    parent_items: &[MenuItem],
) -> Option<PopupId> {
    let popup = ctx.popup.as_ref()?;
    let (open_child_id, open_child_index) = (&ctx.open_child_id, &ctx.open_child_index);
    if child_already_open(open_child_id.get().is_some(), open_child_index, index) {
        return open_child_id.get();
    }
    close_child(open_child_id.clone(), open_child_index.clone());
    announce_submenu(&ctx.on_open_submenu, id);
    let m = ctx.palette_source.current();
    let initial_items = submenu_contents(id.is_some(), sub_items);
    let child_size = MenuView::preferred_size(&initial_items, m);
    let mut child_view = ctx.child_view(initial_items.clone());
    if let (Some(source), Some(id)) = (&ctx.submenu_source, id) {
        let source = source.clone();
        let id = id.to_string();
        let fallback = initial_items;
        child_view = child_view.with_items_source(move || {
            source(&id)
                .filter(|items| !items.is_empty())
                .unwrap_or_else(|| fallback.clone())
        });
    }
    if let Some(callback) = &ctx.on_open_submenu {
        let callback = callback.clone();
        child_view = child_view.with_open_submenu(move |id| callback(id));
    }
    if let Some(source) = &ctx.submenu_source {
        let source = source.clone();
        child_view = child_view.with_submenu_source(move |id| source(id));
    }
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
    let (anchor, gravity, dx) = submenu_positioner(ctx.opens_left);
    let spec = PopupSpec {
        anchor_rect: submenu_anchor_rect(index, parent_items, m, ctx.size.0, popup.scroll_shift()),
        anchor,
        gravity,
        offset: (dx, -(inset as i32)),
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
    child_view = child_view.with_popup_spec(spec.clone());
    let id = popup.open_child(spec, AppComponent::new(child_view));
    dismissed_id.set(Some(id));
    open_child_id.set(Some(id));
    open_child_index.set(Some(index));
    Some(id)
}

pub(super) fn open_reveal_popup(
    ctx: &SubmenuCtx,
    items: &[MenuItem],
    location: &SearchLocation,
    reveal_child_id: Rc<Cell<Option<PopupId>>>,
    (anchor_x, anchor_width): (f32, f32),
) -> Option<PopupId> {
    let popup = ctx.popup.as_ref()?;
    close_child(reveal_child_id.clone(), Rc::new(Cell::new(None)));
    let m = ctx.palette_source.current();
    let child_size = MenuView::preferred_size(items, m);
    let parent_width = ctx.size.0;
    let parent_left = if ctx.opens_left {
        anchor_x + anchor_width - parent_width
    } else {
        anchor_x
    };
    let reveal_x = parent_left - child_size.0 - 8.0;
    if reveal_x < 0.0 {
        return None;
    }
    let focused_index = location.indices.first().copied();
    let child_view = ctx
        .child_view(items.to_vec())
        .with_focused_index(focused_index);
    let reveal_child_id_set = reveal_child_id.clone();
    let on_dismissed = move || {
        reveal_child_id_set.set(None);
    };
    let spec = PopupSpec {
        anchor_rect: (0, 0, 1, 1),
        anchor: PopupAnchor::TopLeft,
        gravity: PopupGravity::BottomLeft,
        offset: (-8, 0),
        size: (child_size.0 as u32, child_size.1 as u32),
        constraint_adjustment: PopupConstraintAdjustment::default(),
        grab: false,
        on_dismissed: Some(Rc::new(on_dismissed)),
    };
    let id = popup.open_child(spec, AppComponent::new(child_view));
    reveal_child_id.set(Some(id));
    Some(id)
}
