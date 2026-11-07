use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use freya_core::elements::image::ImageHandle;
use freya_core::prelude::*;
use freya_engine::prelude::{AlphaType, ColorType, Data, ImageInfo, raster_from_data};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use torin::prelude::Size;
use zbus::object_server::SignalEmitter;
use zbus::proxy::CacheProperties;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::app_menu::layout::{DbusMenuLayout, LayoutItem};
use crate::menu::{CheckState, MenuItem};
use crate::status_item::Pixmaps;

const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const ITEM: &str = "org.kde.StatusNotifierItem";
const ITEM_PATH: &str = "/StatusNotifierItem";
const DBUSMENU: &str = "com.canonical.dbusmenu";
const CALL_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, PartialEq)]
pub struct Pixmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrayItem {
    pub key: String,
    pub title: String,
    pub icon_name: String,
    pub icon_theme_path: String,
    pub pixmap: Option<Pixmap>,
    pub menu: Vec<MenuItem>,
}

enum Event {
    Registered(String),
    Unregistered(String),
    Vanished(String),
    Changed(String),
    Activate(String),
    SecondaryActivate(String),
    Click(String, i32),
}

#[derive(Clone)]
pub struct StatusHost {
    events: UnboundedSender<Event>,
}

impl StatusHost {
    pub fn activate(&self, key: &str) {
        let _ = self.events.unbounded_send(Event::Activate(key.to_owned()));
    }

    pub fn secondary_activate(&self, key: &str) {
        let _ = self
            .events
            .unbounded_send(Event::SecondaryActivate(key.to_owned()));
    }

    pub fn click(&self, key: &str, id: &str) {
        if let Ok(id) = id.parse() {
            let _ = self.events.unbounded_send(Event::Click(key.to_owned(), id));
        }
    }
}

pub fn use_status_items() -> (State<Vec<TrayItem>>, StatusHost) {
    let mut items = use_state(Vec::new);
    let host = use_hook(|| {
        let (events, events_rx) = futures_channel::mpsc::unbounded();
        let (updates, mut updates_rx) = futures_channel::mpsc::unbounded();
        let worker_events = events.clone();
        std::thread::Builder::new()
            .name("barrier-status-items".to_string())
            .spawn(move || async_io::block_on(run(worker_events, events_rx, updates)))
            .expect("barrier-status-items thread");
        spawn(async move {
            while let Some(next) = updates_rx.next().await {
                items.set(next);
            }
        });
        StatusHost { events }
    });
    (items, host)
}

pub fn pixmap_image(pixmap: &Pixmap, size: f32) -> Option<Element> {
    let info = ImageInfo::new(
        (pixmap.width as i32, pixmap.height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let raster = raster_from_data(
        &info,
        Data::new_copy(&pixmap.rgba),
        pixmap.width as usize * 4,
    )?;
    Some(
        image(ImageHandle::new(raster, Bytes::new()))
            .width(Size::px(size))
            .height(Size::px(size))
            .into_element(),
    )
}

fn item_key(sender: &str, service: &str) -> String {
    if service.starts_with('/') {
        format!("{sender}{service}")
    } else {
        format!("{service}{ITEM_PATH}")
    }
}

fn split_key(key: &str) -> Option<(&str, &str)> {
    let slash = key.find('/')?;
    Some((&key[..slash], &key[slash..]))
}

fn largest_pixmap(pixmaps: &[(i32, i32, Vec<u8>)]) -> Option<Pixmap> {
    let (width, height, argb) = pixmaps
        .iter()
        .filter(|(width, height, data)| {
            *width > 0 && *height > 0 && data.len() == (*width * *height * 4) as usize
        })
        .max_by_key(|(width, ..)| *width)?;
    let rgba = argb
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[a, r, g, b]| [*r, *g, *b, *a])
        .collect();
    Some(Pixmap {
        width: *width as u32,
        height: *height as u32,
        rgba,
    })
}

fn plain_label(label: &str) -> String {
    let mut plain = String::with_capacity(label.len());
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '_' {
            if chars.peek() == Some(&'_') {
                plain.push('_');
                chars.next();
            }
        } else {
            plain.push(c);
        }
    }
    plain
}

fn property<T: TryFrom<OwnedValue>>(
    properties: &HashMap<String, OwnedValue>,
    name: &str,
) -> Option<T> {
    properties
        .get(name)
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| T::try_from(value).ok())
}

fn menu_items(children: Vec<OwnedValue>) -> Vec<MenuItem> {
    children
        .into_iter()
        .filter_map(|child| {
            let (id, properties, children): LayoutItem = child.try_into().ok()?;
            if property::<bool>(&properties, "visible") == Some(false) {
                return None;
            }
            if property::<String>(&properties, "type").as_deref() == Some("separator") {
                return Some(MenuItem::Separator);
            }
            let label = plain_label(&property::<String>(&properties, "label").unwrap_or_default());
            let enabled = property::<bool>(&properties, "enabled").unwrap_or(true);
            if !children.is_empty()
                || property::<String>(&properties, "children-display").as_deref() == Some("submenu")
            {
                return Some(MenuItem::submenu(label, menu_items(children)).with_enabled(enabled));
            }
            let on = property::<i32>(&properties, "toggle-state") == Some(1);
            let check = match property::<String>(&properties, "toggle-type").as_deref() {
                Some("checkmark") => CheckState::Checkbox(on),
                Some("radio") => CheckState::Radio(on),
                _ => CheckState::None,
            };
            Some(
                MenuItem::action(id.to_string(), label)
                    .with_enabled(enabled)
                    .with_check(check),
            )
        })
        .collect()
}

async fn timed<T>(future: impl Future<Output = T>) -> Option<T> {
    let timeout = async {
        async_io::Timer::after(CALL_TIMEOUT).await;
        None
    };
    futures_util::future::select(Box::pin(async { Some(future.await) }), Box::pin(timeout))
        .await
        .factor_first()
        .0
}

struct Watcher {
    items: Arc<Mutex<Vec<String>>>,
    hosts: Vec<String>,
    events: UnboundedSender<Event>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    async fn register_status_notifier_item(
        &mut self,
        service: &str,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) {
        let sender = header
            .sender()
            .map(|name| name.to_string())
            .unwrap_or_default();
        let key = item_key(&sender, service);
        {
            let Ok(mut items) = self.items.lock() else {
                return;
            };
            if items.contains(&key) {
                return;
            }
            items.push(key.clone());
        }
        let _ = Self::status_notifier_item_registered(&emitter, &key).await;
        let _ = self.events.unbounded_send(Event::Registered(key));
    }

    async fn register_status_notifier_host(
        &mut self,
        service: &str,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) {
        if !self.hosts.iter().any(|host| host == service) {
            self.hosts.push(service.to_owned());
        }
        let _ = Self::status_notifier_host_registered(&emitter).await;
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items
            .lock()
            .map(|items| items.clone())
            .unwrap_or_default()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        !self.hosts.is_empty()
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

struct Entry {
    item: TrayItem,
    menu: Option<OwnedObjectPath>,
    _signals: Vec<zbus::Task<()>>,
}

async fn proxy<'a>(
    connection: &zbus::Connection,
    bus: &'a str,
    path: &'a str,
    interface: &'a str,
) -> Option<zbus::Proxy<'a>> {
    zbus::proxy::Builder::new(connection)
        .destination(bus)
        .ok()?
        .path(path)
        .ok()?
        .interface(interface)
        .ok()?
        .cache_properties(CacheProperties::No)
        .build()
        .await
        .ok()
}

async fn fetch(
    connection: &zbus::Connection,
    key: &str,
) -> Option<(TrayItem, Option<OwnedObjectPath>)> {
    let (bus, path) = split_key(key)?;
    let item = proxy(connection, bus, path, ITEM).await?;
    let text = |name: &'static str| {
        let item = item.clone();
        async move {
            timed(item.get_property::<String>(name))
                .await
                .and_then(Result::ok)
                .unwrap_or_default()
        }
    };
    let attention = text("Status").await == "NeedsAttention";
    let (icon_name, pixmap_name) = if attention {
        ("AttentionIconName", "AttentionIconPixmap")
    } else {
        ("IconName", "IconPixmap")
    };
    let pixmaps: Pixmaps = timed(item.get_property(pixmap_name))
        .await
        .and_then(Result::ok)
        .unwrap_or_default();
    let menu_path: Option<OwnedObjectPath> =
        timed(item.get_property("Menu")).await.and_then(Result::ok);
    let menu = match &menu_path {
        Some(menu_path) => layout(connection, bus, menu_path.as_str()).await,
        None => Vec::new(),
    };
    Some((
        TrayItem {
            key: key.to_owned(),
            title: text("Title").await,
            icon_name: text(icon_name).await,
            icon_theme_path: text("IconThemePath").await,
            pixmap: largest_pixmap(&pixmaps),
            menu,
        },
        menu_path,
    ))
}

async fn layout(connection: &zbus::Connection, bus: &str, path: &str) -> Vec<MenuItem> {
    let Some(menu) = proxy(connection, bus, path, DBUSMENU).await else {
        return Vec::new();
    };
    let _ = timed(menu.call::<_, _, bool>("AboutToShow", &(0i32,))).await;
    let layout: Option<DbusMenuLayout> =
        timed(menu.call("GetLayout", &(0i32, -1i32, Vec::<String>::new())))
            .await
            .and_then(Result::ok);
    layout
        .map(|(_, (_, _, children))| menu_items(children))
        .unwrap_or_default()
}

async fn watch_item(
    connection: &zbus::Connection,
    key: &str,
    menu: Option<&OwnedObjectPath>,
    events: &UnboundedSender<Event>,
) -> Vec<zbus::Task<()>> {
    let Some((bus, path)) = split_key(key) else {
        return Vec::new();
    };
    let mut sources = vec![(path.to_owned(), ITEM)];
    if let Some(menu) = menu {
        sources.push((menu.to_string(), DBUSMENU));
    }
    let mut tasks = Vec::new();
    for (path, interface) in sources {
        let Some(proxy) = proxy(connection, bus, &path, interface).await else {
            continue;
        };
        let Ok(mut signals) = proxy.receive_all_signals().await else {
            continue;
        };
        let (events, key) = (events.clone(), key.to_owned());
        tasks.push(connection.executor().spawn(
            async move {
                while signals.next().await.is_some() {
                    let _ = events.unbounded_send(Event::Changed(key.clone()));
                }
            },
            "status-item-signals",
        ));
    }
    tasks
}

async fn own_watcher(
    connection: &zbus::Connection,
    events: &UnboundedSender<Event>,
) -> Option<Arc<Mutex<Vec<String>>>> {
    let items = Arc::new(Mutex::new(Vec::new()));
    let watcher = Watcher {
        items: items.clone(),
        hosts: Vec::new(),
        events: events.clone(),
    };
    if !crate::session_bus::export(connection, WATCHER_PATH, watcher).await {
        return None;
    }
    if let Err(error) = connection
        .request_name_with_flags(WATCHER, zbus::fdo::RequestNameFlags::DoNotQueue.into())
        .await
    {
        tracing::info!(%error, "BarrierKit: another StatusNotifierWatcher runs; hosting its items");
        let _ = connection
            .object_server()
            .remove::<Watcher, _>(WATCHER_PATH)
            .await;
        return None;
    }
    Some(items)
}

async fn follow_watcher(connection: &zbus::Connection, events: &UnboundedSender<Event>) {
    let Some(watcher) = proxy(connection, WATCHER, WATCHER_PATH, WATCHER).await else {
        return;
    };
    let registered: Vec<String> = timed(watcher.get_property("RegisteredStatusNotifierItems"))
        .await
        .and_then(Result::ok)
        .unwrap_or_default();
    for key in registered {
        let _ = events.unbounded_send(Event::Registered(key));
    }
    for (signal, added) in [
        ("StatusNotifierItemRegistered", true),
        ("StatusNotifierItemUnregistered", false),
    ] {
        let Ok(mut stream) = watcher.receive_signal(signal).await else {
            continue;
        };
        let events = events.clone();
        connection
            .executor()
            .spawn(
                async move {
                    while let Some(message) = stream.next().await {
                        if let Ok(key) = message.body().deserialize::<String>() {
                            let event = if added {
                                Event::Registered(key)
                            } else {
                                Event::Unregistered(key)
                            };
                            let _ = events.unbounded_send(event);
                        }
                    }
                },
                "status-watcher-signals",
            )
            .detach();
    }
}

async fn watch_names(connection: &zbus::Connection, events: &UnboundedSender<Event>) {
    let Ok(bus) = zbus::fdo::DBusProxy::new(connection).await else {
        return;
    };
    let Ok(mut changes) = bus.receive_name_owner_changed().await else {
        return;
    };
    let events = events.clone();
    connection
        .executor()
        .spawn(
            async move {
                while let Some(change) = changes.next().await {
                    if let Ok(args) = change.args()
                        && args.new_owner.is_none()
                    {
                        let _ = events.unbounded_send(Event::Vanished(args.name.to_string()));
                    }
                }
            },
            "status-item-names",
        )
        .detach();
}

async fn call(
    connection: &zbus::Connection,
    bus: &str,
    path: &str,
    interface: &str,
    method: &str,
    body: impl serde::Serialize + zbus::zvariant::DynamicType,
) {
    let result =
        timed(connection.call_method(Some(bus), path, Some(interface), method, &body)).await;
    if let Some(Err(error)) = result {
        tracing::debug!(%error, method, "BarrierKit: status item call failed");
    }
}

async fn run(
    events: UnboundedSender<Event>,
    mut incoming: UnboundedReceiver<Event>,
    updates: UnboundedSender<Vec<TrayItem>>,
) {
    let connection = match crate::session_bus::connection().await {
        Ok(connection) => connection,
        Err(error) => {
            tracing::warn!(%error, "BarrierKit: no session bus; no status items");
            return;
        }
    };
    let owned = own_watcher(&connection, &events).await;
    if owned.is_none() {
        follow_watcher(&connection, &events).await;
    }
    watch_names(&connection, &events).await;
    let host = format!("org.kde.StatusNotifierHost-{}", std::process::id());
    if connection.request_name(host.as_str()).await.is_ok() {
        call(
            &connection,
            WATCHER,
            WATCHER_PATH,
            WATCHER,
            "RegisterStatusNotifierHost",
            (host.as_str(),),
        )
        .await;
    }
    let mut order: Vec<String> = Vec::new();
    let mut entries: HashMap<String, Entry> = HashMap::new();
    while let Some(event) = incoming.next().await {
        match event {
            Event::Registered(key) | Event::Changed(key) => {
                let Some((item, menu)) = fetch(&connection, &key).await else {
                    continue;
                };
                if let Some(entry) = entries.get_mut(&key) {
                    entry.item = item;
                    entry.menu = menu;
                } else {
                    let signals = watch_item(&connection, &key, menu.as_ref(), &events).await;
                    order.push(key.clone());
                    entries.insert(
                        key,
                        Entry {
                            item,
                            menu,
                            _signals: signals,
                        },
                    );
                }
            }
            Event::Unregistered(key) => {
                order.retain(|known| *known != key);
                entries.remove(&key);
            }
            Event::Vanished(name) => {
                let gone: Vec<String> = order
                    .iter()
                    .filter(|key| split_key(key).is_some_and(|(bus, _)| bus == name))
                    .cloned()
                    .collect();
                for key in gone {
                    order.retain(|known| *known != key);
                    entries.remove(&key);
                    if let Some(items) = &owned {
                        if let Ok(mut items) = items.lock() {
                            items.retain(|known| *known != key);
                        }
                        if let Ok(watcher) = connection
                            .object_server()
                            .interface::<_, Watcher>(WATCHER_PATH)
                            .await
                        {
                            let _ = Watcher::status_notifier_item_unregistered(
                                watcher.signal_emitter(),
                                &key,
                            )
                            .await;
                        }
                    }
                }
            }
            Event::Activate(key) => {
                if let Some((bus, path)) = split_key(&key) {
                    call(&connection, bus, path, ITEM, "Activate", (0i32, 0i32)).await;
                }
                continue;
            }
            Event::SecondaryActivate(key) => {
                if let Some((bus, path)) = split_key(&key) {
                    call(
                        &connection,
                        bus,
                        path,
                        ITEM,
                        "SecondaryActivate",
                        (0i32, 0i32),
                    )
                    .await;
                }
                continue;
            }
            Event::Click(key, id) => {
                if let (
                    Some((bus, _)),
                    Some(Entry {
                        menu: Some(menu), ..
                    }),
                ) = (split_key(&key), entries.get(&key))
                {
                    let data = OwnedValue::from(0i32);
                    call(
                        &connection,
                        bus,
                        menu.as_str(),
                        DBUSMENU,
                        "Event",
                        (id, "clicked", data, 0u32),
                    )
                    .await;
                }
                continue;
            }
        }
        let snapshot = order
            .iter()
            .filter_map(|key| entries.get(key).map(|entry| entry.item.clone()))
            .collect();
        let _ = updates.unbounded_send(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_registered_by_path_is_keyed_by_its_sender() {
        assert_eq!(
            item_key(":1.7", "/org/ayatana/NotificationItem/x"),
            ":1.7/org/ayatana/NotificationItem/x"
        );
        assert_eq!(
            item_key(":1.7", "org.kde.StatusNotifierItem-4-0"),
            "org.kde.StatusNotifierItem-4-0/StatusNotifierItem"
        );
        assert_eq!(split_key(":1.7/a/b"), Some((":1.7", "/a/b")));
    }

    #[test]
    fn the_largest_valid_pixmap_turns_from_argb_into_rgba() {
        let pixmaps = vec![
            (1, 1, vec![255, 1, 2, 3]),
            (2, 2, vec![9; 3]),
            (2, 1, vec![128, 10, 20, 30, 0, 0, 0, 0]),
        ];
        let pixmap = largest_pixmap(&pixmaps).unwrap();
        assert_eq!((pixmap.width, pixmap.height), (2, 1));
        assert_eq!(&pixmap.rgba[..4], [10, 20, 30, 128]);
    }

    #[test]
    fn mnemonic_underscores_are_dropped_and_doubled_ones_kept() {
        assert_eq!(plain_label("_Abrir"), "Abrir");
        assert_eq!(plain_label("mi__archivo"), "mi_archivo");
    }
}
