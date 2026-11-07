use super::*;

#[derive(Default)]
pub(super) struct HoverState {
    pub(super) last: Option<(Instant, CursorPoint)>,
    pub(super) pending: Option<HoverPending>,
    pub(super) generation: u64,
}

#[derive(Clone, Copy)]
pub(super) struct HoverPending {
    pub(super) generation: u64,
    pub(super) target_index: usize,
    pub(super) from: CursorPoint,
    pub(super) to_top: CursorPoint,
    pub(super) to_bottom: CursorPoint,
}

pub(super) fn point_in_triangle(
    p: CursorPoint,
    a: CursorPoint,
    b: CursorPoint,
    c: CursorPoint,
) -> bool {
    let sign = |p1: CursorPoint, p2: CursorPoint, p3: CursorPoint| {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    };
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

pub(super) fn first_selectable(items: &[MenuItem]) -> Option<usize> {
    items.iter().position(|item| item.selectable())
}

pub(super) fn last_selectable(items: &[MenuItem]) -> Option<usize> {
    items.iter().rposition(|item| item.selectable())
}

pub(super) fn next_selectable(
    items: &[MenuItem],
    current: Option<usize>,
    delta: isize,
) -> Option<usize> {
    let len = items.len();
    if len == 0 {
        return None;
    }
    let start = current
        .map(|i| (i as isize + delta).rem_euclid(len as isize) as usize)
        .unwrap_or_else(|| if delta > 0 { 0 } else { len - 1 });
    let mut index = start;
    loop {
        if items[index].selectable() {
            return Some(index);
        }
        index = (index as isize + delta.signum()).rem_euclid(len as isize) as usize;
        if index == start {
            return None;
        }
    }
}

pub(super) fn activate_at(items: &[MenuItem], index: usize) -> Option<String> {
    match items.get(index)? {
        MenuItem::Action { id, enabled, .. } if *enabled => Some(id.clone()),
        MenuItem::SearchResult { item, .. } => activate_at(std::slice::from_ref(item), 0),
        _ => None,
    }
}

pub(super) fn activate_or_open(ctx: &SubmenuCtx, items: &[MenuItem], index: usize) {
    if let Some(id) = activate_at(items, index) {
        (ctx.on_activate)(id);
        (ctx.on_dismiss_all)();
    } else if let Some(MenuItem::Submenu { id, items: sub, .. }) = items.get(index) {
        open_child_popup(ctx, index, id.as_deref(), sub, items);
    }
}
