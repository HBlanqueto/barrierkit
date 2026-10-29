use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zbus::zvariant::{Array, OwnedValue, Value};

use crate::app_menu::{
    AppMenu, IdMap, Menu, check_state, dbusmenu_label, dbusmenu_shortcut, owned,
};
use crate::menu::MenuItem;

/// One dbusmenu layout item: `(id, properties, children)`.
pub type LayoutItem = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);

/// The value returned by `GetLayout`: `(revision, layout_item)`.
pub type DbusMenuLayout = (u32, LayoutItem);

/// Update record for `ItemsPropertiesUpdated`.
#[derive(Debug, Clone)]
pub struct PropertiesUpdate {
    pub id: i32,
    pub properties: HashMap<String, OwnedValue>,
}

/// State shared between the render thread and the D-Bus thread.
pub struct RegisteredMenu {
    pub handle: crate::app_menu::MenuHandle,
    pub(crate) last_layout: Mutex<Option<LayoutSnapshot>>,
}

impl RegisteredMenu {
    pub fn new(handle: crate::app_menu::MenuHandle) -> Arc<Self> {
        Arc::new(Self {
            handle,
            last_layout: Mutex::new(None),
        })
    }
}

/// Snapshot of a generated layout used to decide between
/// `ItemsPropertiesUpdated` and `LayoutUpdated`.
#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct LayoutSnapshot {
    pub revision: u32,
    pub nodes: HashMap<i32, LayoutNode>,
    pub root_children: Vec<i32>,
}

#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct LayoutNode {
    pub properties: HashMap<String, OwnedValue>,
    pub children: Vec<i32>,
    /// Original textual action id, if this node represents an action.
    pub action_id: Option<String>,
}

/// Generates the dbusmenu layout for `menu` using stable ids from `id_map`.
pub fn build_layout(
    menu: &AppMenu,
    id_map: &mut IdMap,
    revision: u32,
) -> (LayoutSnapshot, LayoutItem) {
    let mut snapshot = LayoutSnapshot {
        revision,
        nodes: HashMap::new(),
        root_children: Vec::new(),
    };
    // Reserve id 0 for the root so top-level menus start at 1 and never
    // collide with it.
    id_map.get("__root__");
    let root_id = 0;
    let mut root_children_values: Vec<OwnedValue> = Vec::new();
    for m in &menu.menus {
        let menu_key = format!("__menu__:{}", m.title);
        let menu_id = id_map.get(&menu_key);
        snapshot.root_children.push(menu_id);
        let item = layout_menu(menu_id, menu_key, m, id_map, &mut snapshot);
        root_children_values.push(
            OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(item))).unwrap(),
        );
    }
    snapshot.nodes.insert(
        root_id,
        LayoutNode {
            properties: HashMap::new(),
            children: snapshot.root_children.clone(),
            action_id: None,
        },
    );
    let root = (
        root_id,
        HashMap::<String, OwnedValue>::new(),
        root_children_values,
    );
    (snapshot, root)
}

fn layout_menu(
    id: i32,
    parent_key: String,
    menu: &Menu,
    id_map: &mut IdMap,
    snapshot: &mut LayoutSnapshot,
) -> LayoutItem {
    let mut props = HashMap::new();
    props.insert("label".to_string(), owned(dbusmenu_label(&menu.title)));
    props.insert("children-display".to_string(), owned("submenu"));
    let mut children = Vec::new();
    let mut child_ids = Vec::new();
    for (index, child) in menu.items.iter().enumerate() {
        let (child_item, child_id) = layout_item(child, &parent_key, index, id_map, snapshot);
        child_ids.push(child_id);
        children.push(
            OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(
                child_item,
            )))
            .unwrap(),
        );
    }
    snapshot.nodes.insert(
        id,
        LayoutNode {
            properties: props.clone(),
            children: child_ids,
            action_id: None,
        },
    );
    item_value(id, &props, children)
}

fn layout_item(
    item: &MenuItem,
    parent_key: &str,
    index: usize,
    id_map: &mut IdMap,
    snapshot: &mut LayoutSnapshot,
) -> (LayoutItem, i32) {
    match item {
        MenuItem::Separator => {
            let id = id_map.get(format!("__separator__:{parent_key}:{index}"));
            let mut props = HashMap::new();
            props.insert("type".to_string(), owned("separator"));
            snapshot.nodes.insert(
                id,
                LayoutNode {
                    properties: props.clone(),
                    children: Vec::new(),
                    action_id: None,
                },
            );
            (item_value(id, &props, Vec::new()), id)
        }
        MenuItem::Action {
            id: action_id,
            label,
            shortcut,
            enabled,
            check,
            ..
        } => {
            let id = id_map.get(action_id.as_str());
            let mut props = HashMap::new();
            props.insert("label".to_string(), owned(dbusmenu_label(label)));
            props.insert("enabled".to_string(), owned(*enabled));
            if let Some(shortcut) = shortcut {
                let keys = dbusmenu_shortcut(shortcut);
                if !keys.is_empty() {
                    let combo: Vec<Value<'static>> = keys.into_iter().map(Value::from).collect();
                    let array = Array::from(vec![Value::Array(Array::from(combo))]);
                    props.insert("shortcut".to_string(), owned(array));
                }
            }
            let (toggle_type, toggle_state) = check_state(check);
            if let Some(toggle_type) = toggle_type {
                props.insert("toggle-type".to_string(), owned(toggle_type));
                props.insert("toggle-state".to_string(), owned(toggle_state));
            }
            snapshot.nodes.insert(
                id,
                LayoutNode {
                    properties: props.clone(),
                    children: Vec::new(),
                    action_id: Some(action_id.clone()),
                },
            );
            (item_value(id, &props, Vec::new()), id)
        }
        MenuItem::Submenu {
            id: submenu_id,
            label,
            enabled,
            items,
            ..
        } => {
            let key = submenu_id
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("__submenu__:{label}"));
            let id = id_map.get(&key);
            let mut props = HashMap::new();
            props.insert("label".to_string(), owned(dbusmenu_label(label)));
            props.insert("enabled".to_string(), owned(*enabled));
            props.insert("children-display".to_string(), owned("submenu"));
            let mut children = Vec::new();
            let mut child_ids = Vec::new();
            for (i, child) in items.iter().enumerate() {
                let (child_item, child_id) = layout_item(child, &key, i, id_map, snapshot);
                child_ids.push(child_id);
                children.push(
                    OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(
                        child_item,
                    )))
                    .unwrap(),
                );
            }
            snapshot.nodes.insert(
                id,
                LayoutNode {
                    properties: props.clone(),
                    children: child_ids,
                    action_id: None,
                },
            );
            (item_value(id, &props, children), id)
        }
        // Headers and search fields are not part of app menus.
        _ => {
            let id = id_map.get("__ignored__");
            let props = HashMap::new();
            snapshot.nodes.insert(
                id,
                LayoutNode {
                    properties: props.clone(),
                    children: Vec::new(),
                    action_id: None,
                },
            );
            (item_value(id, &props, Vec::new()), id)
        }
    }
}

fn item_value(
    id: i32,
    props: &HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
) -> LayoutItem {
    (id, props.clone(), children)
}

/// Compares the new layout with the previous one and returns either a
/// `LayoutUpdated` signal or an `ItemsPropertiesUpdated` payload.
pub fn diff_layout(registered: &Arc<RegisteredMenu>, new: LayoutSnapshot) -> Update {
    let mut guard = registered.last_layout.lock().expect("layout lock poisoned");
    let old = guard.take();
    let same_structure = old.as_ref().is_some_and(|old| {
        let old_keys: std::collections::HashSet<_> = old.nodes.keys().copied().collect();
        let new_keys: std::collections::HashSet<_> = new.nodes.keys().copied().collect();
        old_keys == new_keys
            && old.root_children == new.root_children
            && old.nodes.iter().all(|(id, node)| {
                new.nodes
                    .get(id)
                    .map(|n| n.children == node.children)
                    .unwrap_or(false)
            })
    });
    let update = if same_structure {
        let mut updated = Vec::new();
        for (id, node) in &new.nodes {
            let old_props = old
                .as_ref()
                .and_then(|o| o.nodes.get(id).map(|n| n.properties.clone()))
                .unwrap_or_default();
            if old_props != node.properties {
                updated.push(PropertiesUpdate {
                    id: *id,
                    properties: node.properties.clone(),
                });
            }
        }
        if updated.is_empty() {
            Update::None
        } else {
            Update::Properties(updated)
        }
    } else {
        Update::Layout
    };
    *guard = Some(new);
    update
}

pub enum Update {
    None,
    Layout,
    Properties(Vec<PropertiesUpdate>),
}

/// Looks up the original textual id of an action from its numeric dbusmenu id
/// using the last published layout snapshot.
pub fn action_id_for_numeric(registered: &Arc<RegisteredMenu>, numeric_id: i32) -> Option<String> {
    let guard = registered.last_layout.lock().expect("layout lock poisoned");
    guard
        .as_ref()
        .and_then(|snapshot| snapshot.nodes.get(&numeric_id))
        .and_then(|node| node.action_id.clone())
}
