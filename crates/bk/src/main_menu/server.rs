use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use zbus::zvariant::{OwnedValue, Value};

use crate::main_menu::layout::{
    DbusMenuLayout, LayoutItem, LayoutNode, LayoutSnapshot, RegisteredMenu, Update,
    action_id_for_numeric, build_layout, diff_layout,
};
use crate::main_menu::{IdMap, MenuHandle};

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
    StatusItem {
        path: String,
        registered: Arc<RegisteredMenu>,
        item: crate::status_item::StatusItem,
        index: u32,
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

pub(crate) fn register_status_item(handle: MenuHandle, item: crate::status_item::StatusItem) {
    let index = NEXT_PATH_INDEX.fetch_add(1, Ordering::Relaxed);
    let path = format!("{PATH_PREFIX}{index}");
    handle.set_path(&path);
    send(Command::StatusItem {
        path,
        registered: RegisteredMenu::new(handle),
        item,
        index,
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
    let connection = match crate::session_bus::connection().await {
        Ok(c) => c,
        Err(error) => {
            tracing::warn!(%error, "barrier-dbusmenu: no session bus; menus are local only");
            return;
        }
    };

    let mut menus: HashMap<String, Arc<RegisteredMenu>> = HashMap::new();
    let mut id_maps: HashMap<String, IdMap> = HashMap::new();
    let mut status_items: HashMap<String, crate::status_item::Exported> = HashMap::new();

    while let Some(command) = commands.next().await {
        match command {
            Command::Register { path, registered } => {
                let Some(unique) = connection.unique_name().map(|s| s.to_string()) else {
                    tracing::warn!("barrier-dbusmenu: the session bus gave no unique name");
                    continue;
                };
                if export(&connection, &path, &registered, &mut menus, &mut id_maps).await {
                    registered
                        .handle
                        .deliver_address(crate::platform::MainMenuAddress {
                            service: unique,
                            path,
                        });
                }
            }
            Command::StatusItem {
                path,
                registered,
                item,
                index,
            } => {
                let own = match crate::session_bus::private_connection().await {
                    Ok(own) => own,
                    Err(error) => {
                        tracing::warn!(%error, "barrier-dbusmenu: no connection for the status item");
                        continue;
                    }
                };
                if export(&own, &path, &registered, &mut menus, &mut id_maps).await {
                    let handle = registered.handle.clone();
                    let served =
                        crate::status_item::serve(own, item, path.clone(), handle, index).await;
                    if let Some(exported) = served {
                        status_items.insert(path, exported);
                    }
                }
            }
            Command::Unregister { path } => {
                id_maps.remove(&path);
                let own = status_items.remove(&path);
                if menus.remove(&path).is_some() && own.is_none() {
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
                let connection = status_items
                    .get(&path)
                    .map_or(&connection, |item| &item.connection);
                let id_map = id_maps.entry(path.clone()).or_default();
                let snapshot = build_layout_from_handle(&registered, id_map);
                let revision = snapshot.revision;
                match diff_layout(&registered, snapshot) {
                    Update::None => {}
                    Update::Layout => emit_layout_updated(connection, &path, revision).await,
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

async fn export(
    connection: &zbus::Connection,
    path: &str,
    registered: &Arc<RegisteredMenu>,
    menus: &mut HashMap<String, Arc<RegisteredMenu>>,
    id_maps: &mut HashMap<String, IdMap>,
) -> bool {
    let iface = DbusMenuObject {
        registered: registered.clone(),
    };
    if !crate::session_bus::export(connection, path, iface).await {
        return false;
    }
    menus.insert(path.to_owned(), registered.clone());
    let mut id_map = IdMap::default();
    let snapshot = build_layout_from_handle(registered, &mut id_map);
    let revision = snapshot.revision;
    registered.layout().replace(snapshot);
    id_maps.insert(path.to_owned(), id_map);
    emit_layout_updated(connection, path, revision).await;
    true
}

fn unknown(id: i32) -> zbus::fdo::Error {
    zbus::fdo::Error::InvalidArgs(format!("no menu item with id {id}"))
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
    ) -> zbus::fdo::Result<DbusMenuLayout> {
        let guard = self.registered.layout();
        let Some(snapshot) = guard.as_ref() else {
            return Ok((0, (parent_id, HashMap::new(), Vec::new())));
        };
        let node = snapshot
            .nodes
            .get(&parent_id)
            .ok_or_else(|| unknown(parent_id))?;
        Ok((
            snapshot.revision,
            layout_node(snapshot, parent_id, node, recursion_depth, &property_names),
        ))
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

    fn get_property(&self, id: i32, name: String) -> zbus::fdo::Result<OwnedValue> {
        let guard = self.registered.layout();
        let node = guard
            .as_ref()
            .and_then(|s| s.nodes.get(&id))
            .ok_or_else(|| unknown(id))?;
        node.properties
            .get(&name)
            .cloned()
            .ok_or_else(|| zbus::fdo::Error::InvalidArgs(format!("item {id} has no {name}")))
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
