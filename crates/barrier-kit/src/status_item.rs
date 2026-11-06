use futures_util::StreamExt;
use zbus::zvariant::OwnedObjectPath;

use crate::app_menu::MenuHandle;
use crate::menu::MenuItem;

pub const STATUS_ITEM_ACTIVATE: &str = "status-item.activate";

pub type Pixmaps = Vec<(i32, i32, Vec<u8>)>;

const ITEM_PATH: &str = "/StatusNotifierItem";
const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";

#[derive(Clone, Debug, PartialEq)]
pub struct StatusItem {
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub items: Vec<MenuItem>,
}

impl StatusItem {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        icon_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon_name: icon_name.into(),
            items: Vec::new(),
        }
    }

    pub fn items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }
}

pub fn bus_name(pid: u32, index: u32) -> String {
    format!("org.kde.StatusNotifierItem-{pid}-{index}")
}

struct ItemObject {
    item: StatusItem,
    menu: OwnedObjectPath,
    handle: MenuHandle,
}

#[zbus::interface(name = "org.kde.StatusNotifierItem")]
impl ItemObject {
    #[zbus(property, name = "Category")]
    fn category(&self) -> &str {
        "ApplicationStatus"
    }

    #[zbus(property, name = "Id")]
    fn id(&self) -> &str {
        &self.item.id
    }

    #[zbus(property, name = "Title")]
    fn title(&self) -> &str {
        &self.item.title
    }

    #[zbus(property, name = "Status")]
    fn status(&self) -> &str {
        "Active"
    }

    #[zbus(property, name = "WindowId")]
    fn window_id(&self) -> i32 {
        0
    }

    #[zbus(property, name = "IconName")]
    fn icon_name(&self) -> &str {
        &self.item.icon_name
    }

    #[zbus(property, name = "IconPixmap")]
    fn icon_pixmap(&self) -> Pixmaps {
        Vec::new()
    }

    #[zbus(property, name = "ToolTip")]
    fn tool_tip(&self) -> (String, Pixmaps, String, String) {
        (
            String::new(),
            Vec::new(),
            self.item.title.clone(),
            String::new(),
        )
    }

    #[zbus(property, name = "ItemIsMenu")]
    fn item_is_menu(&self) -> bool {
        false
    }

    #[zbus(property, name = "Menu")]
    fn menu(&self) -> OwnedObjectPath {
        self.menu.clone()
    }

    fn activate(&self, _x: i32, _y: i32) {
        self.handle.push_action(STATUS_ITEM_ACTIVATE.to_owned());
    }

    fn secondary_activate(&self, _x: i32, _y: i32) {}

    fn context_menu(&self, _x: i32, _y: i32) {}

    fn scroll(&self, _delta: i32, _orientation: String) {}
}

async fn register_with_watcher(connection: &zbus::Connection, name: &str) {
    let result = connection
        .call_method(
            Some(WATCHER),
            WATCHER_PATH,
            Some(WATCHER),
            "RegisterStatusNotifierItem",
            &(name,),
        )
        .await;
    if let Err(error) = result {
        tracing::debug!(%error, "BarrierKit: no StatusNotifierWatcher yet");
    }
}

pub(crate) async fn serve(
    connection: &zbus::Connection,
    item: StatusItem,
    menu: String,
    handle: MenuHandle,
    index: u32,
) {
    let Ok(menu) = OwnedObjectPath::try_from(menu) else {
        return;
    };
    let object = ItemObject { item, menu, handle };
    if let Err(error) = connection.object_server().at(ITEM_PATH, object).await {
        tracing::warn!(%error, "BarrierKit: could not export the status item");
        return;
    }
    let name = bus_name(std::process::id(), index);
    if let Err(error) = connection.request_name(name.as_str()).await {
        tracing::warn!(%error, "BarrierKit: could not own the status item name");
        return;
    }
    register_with_watcher(connection, &name).await;
    let Ok(bus) = zbus::fdo::DBusProxy::new(connection).await else {
        return;
    };
    let Ok(mut owners) = bus
        .receive_name_owner_changed_with_args(&[(0, WATCHER)])
        .await
    else {
        return;
    };
    let connection = connection.clone();
    connection
        .executor()
        .spawn(
            {
                let connection = connection.clone();
                async move {
                    // The watcher can start after the app (the shell restarts); register again then.
                    while let Some(change) = owners.next().await {
                        if change.args().is_ok_and(|args| args.new_owner.is_some()) {
                            register_with_watcher(&connection, &name).await;
                        }
                    }
                }
            },
            "status-item-watcher",
        )
        .detach();
}
