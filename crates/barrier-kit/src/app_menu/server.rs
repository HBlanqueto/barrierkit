use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use async_channel::{Receiver, Sender};
use zbus::zvariant::{OwnedValue, Value};

use crate::app_menu::layout::{
    DbusMenuLayout, RegisteredMenu, Update, action_id_for_numeric, build_layout, diff_layout,
};
use crate::app_menu::{AppMenu, IdMap, MenuHandle};

const VERSION: u32 = 3;

/// The path prefix for per-window menus: `/MenuBar/<n>`.
const PATH_PREFIX: &str = "/MenuBar/";

/// Monotonic counter for menu paths so unregistering one never reuses it.
static NEXT_PATH_INDEX: AtomicU32 = AtomicU32::new(0);

/// Commands sent from the render thread to the D-Bus thread.
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

/// Per-process D-Bus state. Only one thread and one session connection is
/// created, shared by every window of the app.
struct Server {
    commands: Sender<Command>,
    menus: Mutex<HashMap<String, Weak<RegisteredMenu>>>,
}

static SERVER: OnceLock<Server> = OnceLock::new();

impl Server {
    fn get() -> &'static Server {
        SERVER.get_or_init(|| {
            let (commands, rx) = async_channel::unbounded();
            std::thread::Builder::new()
                .name("barrier-dbusmenu".to_string())
                .spawn(move || {
                    async_io::block_on(run(rx));
                })
                .expect("barrier-dbusmenu thread");
            Server {
                commands,
                menus: Mutex::new(HashMap::new()),
            }
        })
    }

    fn register(handle: MenuHandle) {
        let server = Self::get();
        let index = NEXT_PATH_INDEX.fetch_add(1, Ordering::Relaxed);
        let path = format!("{PATH_PREFIX}{index}");
        handle.set_path(&path);
        let registered = RegisteredMenu::new(handle);
        server
            .menus
            .lock()
            .expect("menu registry poisoned")
            .insert(path.clone(), Arc::downgrade(&registered));
        let _ = server.commands.send_blocking(Command::Register {
            path: path.clone(),
            registered,
        });
    }

    fn unregister(path: &str) {
        let server = Self::get();
        server
            .menus
            .lock()
            .expect("menu registry poisoned")
            .remove(path);
        let _ = server.commands.send_blocking(Command::Unregister {
            path: path.to_string(),
        });
    }

    fn update(path: &str) {
        let server = Self::get();
        let _ = server.commands.send_blocking(Command::Update {
            path: path.to_string(),
        });
    }
}

/// Registers a menu on the session bus. The D-Bus address is delivered
/// asynchronously to the UI thread through [`MenuHandle::deliver_address`].
pub fn register(handle: MenuHandle) {
    Server::register(handle)
}

pub fn unregister(path: &str) {
    Server::unregister(path);
}

pub fn update(path: &str) {
    Server::update(path);
}

async fn run(commands: Receiver<Command>) {
    let connection = match zbus::Connection::session().await {
        Ok(c) => c,
        Err(error) => {
            tracing::warn!(%error, "barrier-dbusmenu: no session bus; menus are local only");
            return;
        }
    };

    let mut menus: HashMap<String, Arc<RegisteredMenu>> = HashMap::new();
    let mut id_maps: HashMap<String, IdMap> = HashMap::new();

    while let Ok(command) = commands.recv().await {
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
                let (snapshot, _layout) = build_layout_from_handle(&registered, &mut id_map);
                let revision = snapshot.revision;
                let _ = registered
                    .last_layout
                    .lock()
                    // Only fails if a panic already poisoned the mutex; no external data here.
                    .expect("layout lock poisoned")
                    .replace(snapshot);
                id_maps.insert(path.clone(), id_map);
                if let Ok(iface_ref) = connection
                    .object_server()
                    .interface::<_, DbusMenuObject>(path.as_str())
                    .await
                    && let Err(error) =
                        DbusMenuObject::layout_updated(iface_ref.signal_emitter(), revision, 0)
                            .await
                {
                    tracing::warn!(%error, "barrier-dbusmenu: could not emit initial LayoutUpdated");
                }
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
                let (snapshot, _layout) = build_layout_from_handle(&registered, id_map);
                let revision = snapshot.revision;
                match diff_layout(&registered, snapshot) {
                    Update::None => {}
                    Update::Layout => {
                        if let Ok(iface_ref) = connection
                            .object_server()
                            .interface::<_, DbusMenuObject>(path.as_str())
                            .await
                            && let Err(error) = DbusMenuObject::layout_updated(
                                iface_ref.signal_emitter(),
                                revision,
                                0,
                            )
                            .await
                        {
                            tracing::warn!(%error, "barrier-dbusmenu: could not emit LayoutUpdated");
                        }
                    }
                    Update::Properties(updated) => {
                        if let Ok(iface_ref) = connection
                            .object_server()
                            .interface::<_, DbusMenuObject>(path.as_str())
                            .await
                        {
                            let payload: Vec<(i32, HashMap<String, OwnedValue>)> =
                                updated.into_iter().map(|u| (u.id, u.properties)).collect();
                            if let Err(error) = DbusMenuObject::items_properties_updated(
                                iface_ref.signal_emitter(),
                                payload,
                                Vec::new(),
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
}

fn build_layout_from_handle(
    registered: &Arc<RegisteredMenu>,
    id_map: &mut IdMap,
) -> (
    crate::app_menu::layout::LayoutSnapshot,
    crate::app_menu::layout::LayoutItem,
) {
    let (menu, revision) = registered.handle.snapshot();
    build_layout(&menu, id_map, revision)
}

struct DbusMenuObject {
    registered: Arc<RegisteredMenu>,
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
        _property_names: Vec<String>,
    ) -> DbusMenuLayout {
        let guard = self
            .registered
            .last_layout
            .lock()
            .expect("layout lock poisoned");
        let Some(snapshot) = guard.as_ref() else {
            return (1, empty_root());
        };
        let revision = snapshot.revision;
        if parent_id == 0 {
            let children: Vec<OwnedValue> = if recursion_depth == 0 {
                Vec::new()
            } else {
                snapshot
                    .root_children
                    .iter()
                    .filter_map(|child_id| {
                        snapshot.nodes.get(child_id).map(|child_node| {
                            let child = layout_node(
                                snapshot,
                                *child_id,
                                child_node,
                                recursion_depth.saturating_sub(1),
                            );
                            OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(
                                child,
                            )))
                            // A well-formed dbusmenu Structure always converts; invariant.
                            .expect("a well-formed dbusmenu Structure always converts")
                        })
                    })
                    .collect()
            };
            return (revision, (0, HashMap::new(), children));
        }
        if let Some(node) = snapshot.nodes.get(&parent_id) {
            #[allow(clippy::needless_borrow)]
            return (
                revision,
                layout_node(&snapshot, parent_id, node, recursion_depth),
            );
        }
        (revision, empty_subtree(parent_id))
    }

    fn get_group_properties(
        &self,
        ids: Vec<i32>,
        _property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let guard = self
            .registered
            .last_layout
            .lock()
            .expect("layout lock poisoned");
        ids.into_iter()
            .filter_map(|id| {
                guard
                    .as_ref()
                    .and_then(|s| s.nodes.get(&id))
                    .map(|node| (id, node.properties.clone()))
            })
            .collect()
    }

    fn get_property(&self, id: i32, name: String) -> OwnedValue {
        let guard = self
            .registered
            .last_layout
            .lock()
            .expect("layout lock poisoned");
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

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<(i32, Vec<String>)> {
        for (id, event_id, data, timestamp) in events {
            self.event(id, event_id, data, timestamp);
        }
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        true
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> Vec<(i32, bool)> {
        ids.into_iter().map(|id| (id, true)).collect()
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

fn empty_root() -> crate::app_menu::layout::LayoutItem {
    (
        0i32,
        HashMap::<String, OwnedValue>::new(),
        Vec::<OwnedValue>::new(),
    )
}

fn empty_subtree(id: i32) -> crate::app_menu::layout::LayoutItem {
    (
        id,
        HashMap::<String, OwnedValue>::new(),
        Vec::<OwnedValue>::new(),
    )
}

fn layout_node(
    snapshot: &crate::app_menu::layout::LayoutSnapshot,
    id: i32,
    node: &crate::app_menu::layout::LayoutNode,
    recursion_depth: i32,
) -> crate::app_menu::layout::LayoutItem {
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
                    );
                    OwnedValue::try_from(Value::Structure(zbus::zvariant::Structure::from(child)))
                        .unwrap()
                })
            })
            .collect()
    };
    (id, node.properties.clone(), children)
}

/// Test-only peer-to-peer server handle (Commit B, ADR 0040 §10).
///
/// Lets integration tests in other crates serve a kit menu over a private
/// `UnixStream::pair` connection without needing a session bus.
#[derive(Clone)]
#[doc(hidden)]
pub struct TestMenuServer {
    registered: Arc<RegisteredMenu>,
    path: String,
}

impl TestMenuServer {
    /// Creates a server for `menu` that queues "clicked" events on its handle.
    pub fn new(
        menu: AppMenu,
        path: impl Into<String>,
        wake: Option<crate::platform::WakeHandle>,
    ) -> Self {
        let handle = crate::app_menu::MenuHandle::new(menu, wake);
        let registered = RegisteredMenu::new(handle);
        Self {
            registered,
            path: path.into(),
        }
    }

    /// The handle shared with the D-Bus object.
    pub fn handle(&self) -> MenuHandle {
        self.registered.handle.clone()
    }

    /// Registers the dbusmenu object on `connection` and emits the initial
    /// `LayoutUpdated`.
    pub async fn serve(&self, connection: &zbus::Connection) -> zbus::Result<()> {
        connection
            .object_server()
            .at(
                self.path.clone(),
                DbusMenuObject {
                    registered: self.registered.clone(),
                },
            )
            .await?;
        let mut id_map = IdMap::default();
        let (menu, _) = self.registered.handle.snapshot();
        let (snapshot, _) = build_layout(&menu, &mut id_map, 1);
        let revision = snapshot.revision;
        *self
            .registered
            .last_layout
            .lock()
            .expect("layout lock poisoned") = Some(snapshot);
        if let Ok(iface_ref) = connection
            .object_server()
            .interface::<_, DbusMenuObject>(self.path.as_str())
            .await
        {
            DbusMenuObject::layout_updated(iface_ref.signal_emitter(), revision, 0).await?;
        }
        Ok(())
    }

    /// Updates the cached properties and emits `ItemsPropertiesUpdated`.
    pub async fn update_properties(
        &self,
        connection: &zbus::Connection,
        updated: Vec<(i32, HashMap<String, OwnedValue>)>,
    ) -> zbus::Result<()> {
        {
            let mut guard = self
                .registered
                .last_layout
                .lock()
                .expect("layout lock poisoned");
            if let Some(snapshot) = guard.as_mut() {
                for (id, props) in &updated {
                    if let Some(node) = snapshot.nodes.get_mut(id) {
                        node.properties.extend(props.clone());
                    }
                }
            }
        }
        let iface_ref = connection
            .object_server()
            .interface::<_, DbusMenuObject>(self.path.as_str())
            .await?;
        DbusMenuObject::items_properties_updated(iface_ref.signal_emitter(), updated, Vec::new())
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    use zbus::zvariant::{OwnedValue, Value};

    use crate::app_menu::server::DbusMenuObject;
    use crate::app_menu::{AppMenu, IdMap, Menu, MenuHandle, MenuItem, layout::build_layout};
    use crate::menu::CheckState;
    use crate::platform::WakeHandle;

    fn action(id: &str) -> MenuItem {
        MenuItem::Action {
            id: id.to_string(),
            label: id.to_string(),
            shortcut: None,
            enabled: true,
            check: CheckState::None,
            icon: None,
        }
    }

    fn separator() -> MenuItem {
        MenuItem::Separator
    }

    fn test_wake() -> WakeHandle {
        let (ping, _) = sctk::reexports::calloop::ping::make_ping().expect("test ping");
        WakeHandle::from_ping(ping)
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
        let (snapshot, _root) = build_layout(&menu, &mut id_map, 1);
        let menu_id = snapshot.root_children[0];
        let children = &snapshot.nodes[&menu_id].children;
        assert_eq!(children.len(), 5, "layout must have five children");
        let unique: HashSet<_> = children.iter().collect();
        assert_eq!(unique.len(), 5, "each child must have a unique id");
    }

    #[test]
    fn event_resolves_against_published_snapshot() {
        let menu = AppMenu::new(vec![Menu::new("File", vec![action("a1"), action("a2")])]);
        let server = super::TestMenuServer::new(menu.clone(), "/MenuBar/0", Some(test_wake()));
        let handle = server.handle();

        // Publish the original layout snapshot.
        let mut id_map = IdMap::default();
        let (snapshot, _) = build_layout(&menu, &mut id_map, 1);
        let id_a2 = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.action_id.as_deref() == Some("a2"))
            .map(|(id, _)| *id)
            .expect("a2 in snapshot");
        *server.registered.last_layout.lock().unwrap() = Some(snapshot);

        // Mutate the live menu: insert a new item at the front.
        let mut new_menu = menu.clone();
        new_menu.menus[0].items.insert(0, action("a0"));
        handle.replace_menu(new_menu);

        // Click the old numeric id of a2.
        let obj = DbusMenuObject {
            registered: server.registered.clone(),
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
        let server = super::TestMenuServer::new(menu.clone(), "/MenuBar/0", Some(test_wake()));
        let handle = server.handle();

        let mut id_map = IdMap::default();
        let (snapshot, _) = build_layout(&menu, &mut id_map, 1);
        let id_a1 = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.action_id.as_deref() == Some("a1"))
            .map(|(id, _)| *id)
            .expect("a1 in snapshot");
        *server.registered.last_layout.lock().unwrap() = Some(snapshot);

        std::thread::spawn(move || {
            let obj = DbusMenuObject {
                registered: server.registered.clone(),
            };
            obj.event(
                id_a1,
                "clicked".to_string(),
                OwnedValue::try_from(Value::I32(0)).unwrap(),
                0,
            );
        })
        .join()
        .unwrap();

        // The action is queued for the UI thread instead of running on the D-Bus thread.
        assert_eq!(handle.drain_actions(), vec!["a1".to_string()]);
    }

    #[test]
    // shortcut: no simula un bus que retiene la respuesta — mejora: mock de zbus con canal retenido.
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
