use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use zbus::zvariant::{OwnedValue, Value};

use crate::app_menu::layout::{
    DbusMenuLayout, LayoutItem, LayoutNode, LayoutSnapshot, RegisteredMenu, Update,
    action_id_for_numeric, build_layout, diff_layout,
};
use crate::app_menu::{IdMap, MenuHandle};

const VERSION: u32 = 3;

const PATH_PREFIX: &str = "/MenuBar/";

static NEXT_PATH_INDEX: AtomicU32 = AtomicU32::new(0);

enum Command {
    Register {
        path: String,
        registered: Arc<RegisteredMenu>,
    },
    Unregister {
        path: String,
    },
    Update {
        path: String,
    },
}

static SERVER: OnceLock<UnboundedSender<Command>> = OnceLock::new();

fn send(command: Command) {
    let commands = SERVER.get_or_init(|| {
        let (commands, rx) = futures_channel::mpsc::unbounded();
        std::thread::Builder::new()
            .name("barrier-dbusmenu".to_string())
            .spawn(move || async_io::block_on(run(rx)))
            .expect("barrier-dbusmenu thread");
        commands
    });
    let _ = commands.unbounded_send(command);
}

pub fn register(handle: MenuHandle) {
    let index = NEXT_PATH_INDEX.fetch_add(1, Ordering::Relaxed);
    let path = format!("{PATH_PREFIX}{index}");
    handle.set_path(&path);
    send(Command::Register {
        path,
        registered: RegisteredMenu::new(handle),
    });
}

pub fn unregister(path: &str) {
    send(Command::Unregister {
        path: path.to_string(),
    });
}

pub fn update(path: &str) {
    send(Command::Update {
        path: path.to_string(),
    });
}

async fn run(mut commands: UnboundedReceiver<Command>) {
    let connection = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(error) => {
            tracing::warn!(%error, "barrier-dbusmenu: no session bus; menus are local only");
            return;
        }
    };

    let mut menus: HashMap<String, Arc<RegisteredMenu>> = HashMap::new();
    let mut id_maps: HashMap<String, IdMap> = HashMap::new();

    while let Some(command) = commands.next().await {
        match command {
            Command::Register { path, registered } => {
                let iface = DbusMenuObject {
                    registered: registered.clone(),
                };
                let Some(unique) = connection.unique_name().map(|s| s.to_string()) else {
                    continue;
                };
                if let Err(error) = connection.object_server().at(path.clone(), iface).await {
                    tracing::warn!(%error, path, "barrier-dbusmenu: could not register object");
                    continue;
                }
                menus.insert(path.clone(), registered.clone());
                let mut id_map = IdMap::default();
                let snapshot = build_layout_from_handle(&registered, &mut id_map);
                let revision = snapshot.revision;
                registered.layout().replace(snapshot);
                id_maps.insert(path.clone(), id_map);
                emit_layout_updated(&connection, &path, revision).await;
                registered
                    .handle
                    .deliver_address(crate::platform::AppMenuAddress {
                        service: unique,
                        path,
                    });
            }
            Command::Unregister { path } => {
                if menus.remove(&path).is_some() {
                    id_maps.remove(&path);
                    let _ = connection
                        .object_server()
                        .remove::<DbusMenuObject, _>(path.as_str())
                        .await;
                }
            }
            Command::Update { path } => {
                let Some(registered) = menus.get(&path).cloned() else {
                    continue;
                };
                let id_map = id_maps.entry(path.clone()).or_default();
                let snapshot = build_layout_from_handle(&registered, id_map);
                let revision = snapshot.revision;
                match diff_layout(&registered, snapshot) {
                    Update::None => {}
                    Update::Layout => emit_layout_updated(&connection, &path, revision).await,
                    Update::Properties(updated, removed) => {
                        if let Ok(iface_ref) = connection
                            .object_server()
                            .interface::<_, DbusMenuObject>(path.as_str())
                            .await
                            && let Err(error) = DbusMenuObject::items_properties_updated(
                                iface_ref.signal_emitter(),
                                updated,
                                removed,
                            )
                            .await
                        {
                            tracing::warn!(%error, "barrier-dbusmenu: could not emit ItemsPropertiesUpdated");
                        }
                    }
                }
            }
        }
    }
}

async fn emit_layout_updated(connection: &zbus::Connection, path: &str, revision: u32) {
    if let Ok(iface_ref) = connection
        .object_server()
        .interface::<_, DbusMenuObject>(path)
        .await
        && let Err(error) =
            DbusMenuObject::layout_updated(iface_ref.signal_emitter(), revision, 0).await
    {
        tracing::warn!(%error, "barrier-dbusmenu: could not emit LayoutUpdated");
    }
}

fn build_layout_from_handle(registered: &RegisteredMenu, id_map: &mut IdMap) -> LayoutSnapshot {
    let (menu, revision) = registered.handle.snapshot();
    build_layout(&menu, id_map, revision)
}

struct DbusMenuObject {
    registered: Arc<RegisteredMenu>,
}

impl DbusMenuObject {
    fn known(&self, id: i32) -> bool {
        self.registered
            .layout()
            .as_ref()
            .is_some_and(|snapshot| snapshot.nodes.contains_key(&id))
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl DbusMenuObject {
    #[zbus(property, name = "Version")]
    fn version(&self) -> u32 {
        VERSION
    }

    #[zbus(property, name = "TextDirection")]
    fn text_direction(&self) -> &str {
        "ltr"
    }

    #[zbus(property, name = "Status")]
    fn status(&self) -> &str {
        "normal"
    }

    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> DbusMenuLayout {
        let guard = self.registered.layout();
        let Some(snapshot) = guard.as_ref() else {
            return (0, (parent_id, HashMap::new(), Vec::new()));
        };
        let layout = match snapshot.nodes.get(&parent_id) {
            Some(node) => layout_node(snapshot, parent_id, node, recursion_depth, &property_names),
            None => (parent_id, HashMap::new(), Vec::new()),
        };
        (snapshot.revision, layout)
    }

    fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let guard = self.registered.layout();
        let Some(snapshot) = guard.as_ref() else {
            return Vec::new();
        };
        let ids = if ids.is_empty() {
            snapshot.nodes.keys().copied().collect()
        } else {
            ids
        };
        ids.into_iter()
            .filter_map(|id| {
                let node = snapshot.nodes.get(&id)?;
                Some((id, filtered(&node.properties, &property_names)))
            })
            .collect()
    }

    fn get_property(&self, id: i32, name: String) -> OwnedValue {
        let guard = self.registered.layout();
        guard
            .as_ref()
            .and_then(|s| s.nodes.get(&id))
            .and_then(|node| node.properties.get(&name).cloned())
            .unwrap_or_else(|| crate::app_menu::owned(""))
    }

    fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
        if event_id != "clicked" {
            return;
        }
        if let Some(action_id) = action_id_for_numeric(&self.registered, id)
            && !self.registered.handle.push_action(action_id.clone())
        {
            tracing::warn!(
                %action_id,
                "barrier-dbusmenu: no UI wake handle; dropped menu action"
            );
        }
    }

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        events
            .into_iter()
            .filter_map(|(id, event_id, data, timestamp)| {
                if !self.known(id) {
                    return Some(id);
                }
                self.event(id, event_id, data, timestamp);
                None
            })
            .collect()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let errors = ids.into_iter().filter(|id| !self.known(*id)).collect();
        (Vec::new(), errors)
    }

    #[zbus(signal)]
    async fn layout_updated(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn items_properties_updated(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        updated_props: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;
}

fn filtered(
    properties: &HashMap<String, OwnedValue>,
    names: &[String],
) -> HashMap<String, OwnedValue> {
    properties
        .iter()
        .filter(|(name, _)| names.is_empty() || names.contains(name))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn layout_node(
    snapshot: &LayoutSnapshot,
    id: i32,
    node: &LayoutNode,
    recursion_depth: i32,
    property_names: &[String],
) -> LayoutItem {
    let children: Vec<OwnedValue> = if recursion_depth == 0 {
        Vec::new()
    } else {
        node.children
            .iter()
            .filter_map(|child_id| {
                snapshot.nodes.get(child_id).map(|child_node| {
                    let child = layout_node(
                        snapshot,
                        *child_id,
                        child_node,
                        recursion_depth.saturating_sub(1),
                        property_names,
                    );
                    OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(child)))
                        .expect("a well-formed dbusmenu Structure always converts")
                })
            })
            .collect()
    };
    (id, filtered(&node.properties, property_names), children)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use zbus::zvariant::{OwnedValue, Value};

    use crate::app_menu::layout::{RegisteredMenu, Update, build_layout, diff_layout};
    use crate::app_menu::server::DbusMenuObject;
    use crate::app_menu::{AppMenu, IdMap, Menu, MenuHandle, MenuItem};
    use crate::menu::CheckState;

    fn action(id: &str) -> MenuItem {
        MenuItem::Action {
            id: id.to_string(),
            label: id.to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
            badge: None,
        }
    }

    fn registered(menu: AppMenu) -> Arc<RegisteredMenu> {
        let (ping, _) = sctk::reexports::calloop::ping::make_ping().expect("test ping");
        let wake = crate::platform::WakeHandle::from_ping(ping);
        RegisteredMenu::new(MenuHandle::new(menu, Some(wake)))
    }

    fn separator() -> MenuItem {
        MenuItem::Separator
    }

    #[test]
    fn path_counter_never_reuses_index() {
        let make = || {
            let handle = MenuHandle::new(AppMenu::new(vec![Menu::new("M", vec![])]), None);
            super::register(handle.clone());
            handle.path().unwrap()
        };
        let path_a = make();
        let path_b = make();
        super::unregister(&path_a);
        let path_c = make();
        assert_ne!(path_c, path_b, "path reused after unregister");
    }

    #[test]
    fn separators_get_distinct_stable_ids() {
        let menu = AppMenu::new(vec![Menu::new(
            "File",
            vec![
                action("a1"),
                separator(),
                action("a2"),
                separator(),
                action("a3"),
            ],
        )]);
        let mut id_map = IdMap::default();
        let snapshot = build_layout(&menu, &mut id_map, 1);
        let menu_id = snapshot.nodes[&0].children[0];
        let children = &snapshot.nodes[&menu_id].children;
        assert_eq!(children.len(), 5, "layout must have five children");
        let unique: HashSet<_> = children.iter().collect();
        assert_eq!(unique.len(), 5, "each child must have a unique id");
    }

    #[test]
    fn equal_submenu_labels_in_different_menus_get_distinct_ids() {
        let recent = || MenuItem::Submenu {
            id: None,
            label: "Recent".to_string(),
            enabled: true,
            items: vec![],
            icon: None,
        };
        let menu = AppMenu::new(vec![
            Menu::new("File", vec![recent()]),
            Menu::new("Edit", vec![recent()]),
        ]);
        let snapshot = build_layout(&menu, &mut IdMap::default(), 1);
        let submenus: HashSet<_> = snapshot.nodes[&0]
            .children
            .iter()
            .map(|menu| snapshot.nodes[menu].children[0])
            .collect();
        assert_eq!(submenus.len(), 2);
    }

    #[test]
    fn a_dropped_shortcut_is_sent_as_a_removed_property() {
        let mut with_shortcut = action("save");
        if let MenuItem::Action { shortcut, .. } = &mut with_shortcut {
            *shortcut = Some("Ctrl+S".to_string());
        }
        let mut id_map = IdMap::default();
        let registered = RegisteredMenu::new(MenuHandle::new(AppMenu::default(), None));
        let layout = |item: MenuItem, id_map: &mut IdMap| {
            build_layout(
                &AppMenu::new(vec![Menu::new("File", vec![item])]),
                id_map,
                1,
            )
        };
        diff_layout(&registered, layout(with_shortcut, &mut id_map));
        let Update::Properties(_, removed) =
            diff_layout(&registered, layout(action("save"), &mut id_map))
        else {
            panic!("only properties changed");
        };
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].1, vec!["shortcut".to_string()]);
    }

    #[test]
    fn event_resolves_against_published_snapshot() {
        let menu = AppMenu::new(vec![Menu::new("File", vec![action("a1"), action("a2")])]);
        let registered = registered(menu.clone());
        let handle = registered.handle.clone();

        let mut id_map = IdMap::default();
        let snapshot = build_layout(&menu, &mut id_map, 1);
        let id_a2 = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.action_id.as_deref() == Some("a2"))
            .map(|(id, _)| *id)
            .expect("a2 in snapshot");
        *registered.last_layout.lock().unwrap() = Some(snapshot);

        let mut new_menu = menu.clone();
        new_menu.menus[0].items.insert(0, action("a0"));
        handle.replace_menu(new_menu);

        let obj = DbusMenuObject {
            registered: registered.clone(),
        };
        obj.event(
            id_a2,
            "clicked".to_string(),
            OwnedValue::try_from(Value::I32(0)).unwrap(),
            0,
        );

        assert_eq!(handle.drain_actions(), vec!["a2".to_string()]);
    }

    #[test]
    fn event_does_not_run_on_dbus_thread() {
        let menu = AppMenu::new(vec![Menu::new("File", vec![action("a1")])]);
        let registered = registered(menu.clone());
        let handle = registered.handle.clone();

        let mut id_map = IdMap::default();
        let snapshot = build_layout(&menu, &mut id_map, 1);
        let id_a1 = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.action_id.as_deref() == Some("a1"))
            .map(|(id, _)| *id)
            .expect("a1 in snapshot");
        *registered.last_layout.lock().unwrap() = Some(snapshot);

        std::thread::spawn(move || {
            let obj = DbusMenuObject { registered };
            obj.event(
                id_a1,
                "clicked".to_string(),
                OwnedValue::try_from(Value::I32(0)).unwrap(),
                0,
            );
        })
        .join()
        .unwrap();

        assert_eq!(handle.drain_actions(), vec!["a1".to_string()]);
    }

    #[test]
    fn register_returns_without_blocking() {
        let handle = MenuHandle::new(AppMenu::new(vec![Menu::new("M", vec![])]), None);
        let start = Instant::now();
        super::register(handle.clone());
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "register blocked for {} ms",
            start.elapsed().as_millis()
        );
        assert!(handle.path().is_some());
    }
}
