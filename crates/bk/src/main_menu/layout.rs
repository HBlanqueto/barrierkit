use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use zbus::zvariant::OwnedValue;

use crate::main_menu::{IdMap, MainMenu, check_state, dbusmenu_label, owned};
use crate::menu::{MenuItem, shortcut_keys};

pub type LayoutItem = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

pub type DbusMenuLayout = (u32, LayoutItem);

pub(crate) type Properties = (i32, HashMap<String, OwnedValue>);

pub(crate) type RemovedProperties = (i32, Vec<String>);

pub(crate) struct RegisteredMenu {
    pub(crate) handle: crate::main_menu::MenuHandle,
    pub(crate) last_layout: Mutex<Option<LayoutSnapshot>>,
}

impl RegisteredMenu {
    pub(crate) fn new(handle: crate::main_menu::MenuHandle) -> Arc<Self> {
        Arc::new(Self {
            handle,
            last_layout: Mutex::new(None),
        })
    }

    pub(crate) fn layout(&self) -> MutexGuard<'_, Option<LayoutSnapshot>> {
        self.last_layout.lock().expect("layout lock poisoned")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutSnapshot {
    pub(crate) revision: u32,
    pub(crate) nodes: HashMap<i32, LayoutNode>,
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutNode {
    pub(crate) properties: HashMap<String, OwnedValue>,
    pub(crate) children: Vec<i32>,
    pub(crate) action_id: Option<String>,
}

pub(crate) fn build_layout(menu: &MainMenu, id_map: &mut IdMap, revision: u32) -> LayoutSnapshot {
    let mut snapshot = LayoutSnapshot {
        revision,
        nodes: HashMap::new(),
    };
    let mut root_children = Vec::new();
    for m in &menu.menus {
        let key = format!("menu:{}", m.title);
        if m.title.is_empty() {
            root_children.extend(layout_items(&m.items, &key, id_map, &mut snapshot));
            continue;
        }
        let id = id_map.get(&key);
        let children = layout_items(&m.items, &key, id_map, &mut snapshot);
        snapshot
            .nodes
            .insert(id, submenu_node(&m.title, true, children));
        root_children.push(id);
    }
    snapshot.nodes.insert(
        0,
        LayoutNode {
            properties: HashMap::from([("children-display".to_string(), owned("submenu"))]),
            children: root_children,
            action_id: None,
        },
    );
    snapshot
}

fn submenu_node(label: &str, enabled: bool, children: Vec<i32>) -> LayoutNode {
    let properties = HashMap::from([
        ("label".to_string(), owned(dbusmenu_label(label))),
        ("enabled".to_string(), owned(enabled)),
        ("children-display".to_string(), owned("submenu")),
    ]);
    LayoutNode {
        properties,
        children,
        action_id: None,
    }
}

fn layout_items(
    items: &[MenuItem],
    parent_key: &str,
    id_map: &mut IdMap,
    snapshot: &mut LayoutSnapshot,
) -> Vec<i32> {
    let mut ids = Vec::new();
    for (index, item) in items.iter().enumerate() {
        match item {
            MenuItem::Palette { items } => {
                let key = format!("{parent_key}:palette{index}");
                ids.extend(layout_items(items, &key, id_map, snapshot));
            }
            item => ids.extend(layout_item(item, parent_key, index, id_map, snapshot)),
        }
    }
    ids
}

fn layout_item(
    item: &MenuItem,
    parent_key: &str,
    index: usize,
    id_map: &mut IdMap,
    snapshot: &mut LayoutSnapshot,
) -> Option<i32> {
    let (id, node) = match item {
        MenuItem::Separator => (
            id_map.get(format!("separator:{parent_key}:{index}")),
            LayoutNode {
                properties: HashMap::from([("type".to_string(), owned("separator"))]),
                children: Vec::new(),
                action_id: None,
            },
        ),
        MenuItem::Action {
            id: action_id,
            label,
            shortcut,
            enabled,
            check,
            ..
        } => {
            let mut properties = HashMap::from([
                ("label".to_string(), owned(dbusmenu_label(label))),
                ("enabled".to_string(), owned(*enabled)),
            ]);
            let keys = shortcut.as_deref().map(shortcut_keys).unwrap_or_default();
            if !keys.is_empty() {
                properties.insert("shortcut".to_string(), owned(vec![keys]));
            }
            let (toggle_type, toggle_state) = check_state(check);
            if let Some(toggle_type) = toggle_type {
                properties.insert("toggle-type".to_string(), owned(toggle_type));
                properties.insert("toggle-state".to_string(), owned(toggle_state));
            }
            (
                unique_id(id_map, snapshot, action_id),
                LayoutNode {
                    properties,
                    children: Vec::new(),
                    action_id: enabled.then(|| action_id.clone()),
                },
            )
        }
        MenuItem::Submenu {
            id,
            label,
            enabled,
            items,
            ..
        } => {
            let key = id
                .clone()
                .unwrap_or_else(|| format!("submenu:{parent_key}/{label}"));
            let children = layout_items(items, &key, id_map, snapshot);
            (
                unique_id(id_map, snapshot, &key),
                submenu_node(label, *enabled, children),
            )
        }
        MenuItem::Header { .. }
        | MenuItem::SearchField { .. }
        | MenuItem::SearchResult { .. }
        | MenuItem::Palette { .. } => {
            return None;
        }
    };
    snapshot.nodes.insert(id, node);
    Some(id)
}

// dbusmenu ids name one item each; an action listed twice gets a second, stable id.
fn unique_id(id_map: &mut IdMap, snapshot: &LayoutSnapshot, key: &str) -> i32 {
    let mut id = id_map.get(key);
    let mut copy = 1;
    while snapshot.nodes.contains_key(&id) {
        id = id_map.get(format!("{key}#{copy}"));
        copy += 1;
    }
    id
}

pub(crate) fn diff_layout(registered: &RegisteredMenu, new: LayoutSnapshot) -> Update {
    let mut guard = registered.layout();
    let old = guard.replace(new);
    let new = guard.as_ref().expect("snapshot just stored");
    let Some(old) = old.filter(|old| {
        old.nodes.len() == new.nodes.len()
            && old.nodes.iter().all(|(id, node)| {
                new.nodes
                    .get(id)
                    .is_some_and(|n| n.children == node.children)
            })
    }) else {
        return Update::Layout;
    };
    let mut updated = Vec::new();
    let mut removed = Vec::new();
    for (id, node) in &new.nodes {
        let old_properties = &old.nodes[id].properties;
        if *old_properties != node.properties {
            updated.push((*id, node.properties.clone()));
        }
        let gone: Vec<String> = old_properties
            .keys()
            .filter(|key| !node.properties.contains_key(*key))
            .cloned()
            .collect();
        if !gone.is_empty() {
            removed.push((*id, gone));
        }
    }
    if updated.is_empty() {
        Update::None
    } else {
        Update::Properties(updated, removed)
    }
}

pub(crate) enum Update {
    None,
    Layout,
    Properties(Vec<Properties>, Vec<RemovedProperties>),
}

pub(crate) fn action_id_for_numeric(
    registered: &RegisteredMenu,
    numeric_id: i32,
) -> Option<String> {
    registered
        .layout()
        .as_ref()
        .and_then(|snapshot| snapshot.nodes.get(&numeric_id))
        .and_then(|node| node.action_id.clone())
}
