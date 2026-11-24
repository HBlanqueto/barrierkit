# 6. Status items

**Goal:** show the status items of other apps on a shell surface, activate
one and open its menu.
**Scope:** the tray host of `bk-shell`: `use_status_items`, `StatusHost`,
`TrayItem`, `Pixmap` and the D-Bus interfaces they speak. The app that
publishes an item is in
[13. App infrastructure](../barrierkit/13-app.md#status-item); the menu
entries are [`MenuItem`s](../barrierkit/07-menus-and-popovers.md#menu-items).
**Reader:** whoever writes the status area of Malaysia's menu bar.
**Input:** a layer surface ([3](03-layer-surfaces.md)) and a session bus.
**Output:** one icon per item. A press activates it; a secondary click opens
its menu and sends the chosen entry back to the app.

## Steps

1. Call `use_status_items()` in the surface's root component. It returns the
   items and the host that acts on them.
2. Draw one view per item, keyed with `.key(&item.key)`: its `pixmap` with
   `Image::rgba`, or its `title` when it has none.
3. Activate the item on press: `.on_press(move |_| host.activate(&key))`.
4. Open its menu on a secondary click with `.context_menu(item.menu.clone(), ..)`
   ([`.context_menu`](../barrierkit/07-menus-and-popovers.md#context_menu)),
   and send the chosen id with `host.click(&key, &id)`.
5. Run it on a compositor with a layer shell, nested as in
   [10](10-run-and-debug.md): `cargo run -p bk-shell --example tray`.
6. Start an app with a status item, such as a kit app with
   `WindowOptions::status_item`. Its icon appears on the surface.

The whole surface is
[`crates/bk-shell/examples/tray.rs`](../../crates/bk-shell/examples/tray.rs):

```rust
use bk::prelude::*;
use bk_shell::{
    Anchors, KeyboardInteractivity, Layer, LayerSpec, StatusHost, TrayItem, run_layer,
    use_status_items,
};
use ctdesign::{MENUBAR_HEIGHT, SYMBOL};
use freya_core::integration::AppComponent;

fn tray() -> impl IntoElement {
    let (items, host) = use_status_items();
    HStack::new().children(items.read().iter().map(|item| status_icon(item, &host)))
}

fn status_icon(item: &TrayItem, host: &StatusHost) -> impl IntoElement {
    let (key, activate, menu) = (item.key.clone(), host.clone(), host.clone());
    let menu_key = key.clone();
    rect()
        .key(&item.key)
        .on_press(move |_| activate.activate(&key))
        .context_menu(item.menu.clone(), move |id: String| {
            menu.click(&menu_key, &id)
        })
        .maybe_child(item.pixmap.as_ref().map(|pixmap| {
            Image::rgba(pixmap.width, pixmap.height, pixmap.rgba.clone()).size(SYMBOL.grid)
        }))
        .maybe_child(item.pixmap.is_none().then(|| Text::new(item.title.clone())))
}

fn main() -> Result<(), RunError> {
    run_layer(
        AppComponent::from(tray),
        LayerSpec {
            namespace: "tray".to_string(),
            layer: Layer::Top,
            anchors: Anchors::TOP,
            exclusive_zone: MENUBAR_HEIGHT,
            keyboard: KeyboardInteractivity::None,
            size: Some((0, MENUBAR_HEIGHT as u32)),
        },
    )
}
```

## The host

`use_status_items() -> (State<Vec<TrayItem>>, StatusHost)` is a hook. Its
first call starts the thread `barrier-status-items`, which hosts the
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/)
icons on the session bus:

1. It takes `org.kde.StatusNotifierWatcher` at `/StatusNotifierWatcher` if
   nobody owns the name. Otherwise it follows the running watcher: it reads
   `RegisteredStatusNotifierItems` and listens to
   `StatusNotifierItemRegistered` and `StatusNotifierItemUnregistered`.
2. It owns `org.kde.StatusNotifierHost-<pid>` and calls
   `RegisterStatusNotifierHost` with that name.
3. For each new item it reads the item and its menu
   ([below](#what-it-reads)), and listens to every signal of both. Each
   signal reads them again.
4. It sets the state to the items in the order they registered. An unchanged
   list does not re-render.
5. It drops an item when it unregisters or its bus name loses its owner.
   When the watcher it follows loses its owner, it takes the watcher again.

Each D-Bus call waits at most 500 ms. A method call that returns an error is
logged at `debug` level. Without a session bus the thread logs a warning and the list stays
empty. The thread stops and releases its names when the last `StatusHost`
clone drops.

The specification spells the interfaces `org.freedesktop.*`; the host uses
`org.kde.*`, as the app side does
([13](../barrierkit/13-app.md#status-item)).

### The watcher it serves

While the host owns the watcher, it serves `org.kde.StatusNotifierWatcher`:

| Member | Kind | What it does |
|---|---|---|
| `RegisterStatusNotifierItem(service)` | Method | `service` is an object path or a bus name. A path is keyed with the caller's unique name; a name must have an owner and is keyed with `/StatusNotifierItem`. Emits `StatusNotifierItemRegistered(key)` once per key |
| `RegisterStatusNotifierHost(service)` | Method | Adds the host and emits `StatusNotifierHostRegistered` |
| `RegisteredStatusNotifierItems` | Property | The keys, in order |
| `IsStatusNotifierHostRegistered` | Property | `true` once a host registered |
| `ProtocolVersion` | Property | `0` |
| `StatusNotifierItemUnregistered(key)` | Signal | Emitted for each item whose bus name loses its owner; `StatusNotifierHostUnregistered` likewise for a host |

### What it reads

One `GetAll` of `org.kde.StatusNotifierItem` fills a `TrayItem`:

| `TrayItem` field | Type | From |
|---|---|---|
| `key` | `String` | Bus name and object path; the id that `StatusHost` takes |
| `title` | `String` | `Title` |
| `icon_name` | `String` | `IconName`, or `AttentionIconName` when `Status` is `NeedsAttention` |
| `icon_theme_path` | `String` | KDE's `IconThemePath` |
| `pixmap` | `Option<Pixmap>` | The widest `IconPixmap` (or `AttentionIconPixmap`) whose bytes are width × height × 4 |
| `menu` | `Vec<MenuItem>` | The `com.canonical.dbusmenu` layout at the item's `Menu` path |

The host does not read `Id`, `Category`, `ToolTip`, `ItemIsMenu`, the overlay
icons or `WindowId`. It does not look `icon_name` up in an icon theme: draw
`pixmap`, or find the name yourself, in `icon_theme_path` first.

`Pixmap` is `WindowImage`: `width`, `height` and `rgba`, straight alpha,
converted from the wire's ARGB. `Image::rgba(width, height, rgba)` draws it
([Images](../barrierkit/04-appearance-and-tokens.md#images-and-template-rendering)).

The menu is `GetLayout(0, -1, [])`. `AboutToShow(0)` goes out once, when the
item registers: an app that rebuilds its menu there emits `LayoutUpdated`,
which would read the menu again without end. Each entry becomes a `MenuItem`:

| dbusmenu entry | `MenuItem` |
|---|---|
| `visible` is `false` | Dropped |
| `type` is `separator` | `MenuItem::Separator` |
| Children, or `children-display` is `submenu` | `MenuItem::submenu(label, items)` |
| `toggle-type` is `checkmark` | An action with `CheckState::Checkbox(toggle-state == 1)`; `CheckState::Mixed` when `toggle-state` is -1 |
| `toggle-type` is `radio` | An action with `CheckState::Radio(toggle-state == 1)` |
| Any other | `MenuItem::action(id, label)` |

The action id is the dbusmenu id as text. A label loses its mnemonic
underscores (`__` keeps one). `enabled` defaults to `true`.

## Acting on an item

| `StatusHost` method | Sends |
|---|---|
| `activate(key)` | `Activate(0, 0)` to the item |
| `secondary_activate(key)` | `SecondaryActivate(0, 0)` |
| `click(key, id)` | `Event(id, "clicked", 0, 0)` to the item's menu; an `id` that is not a number is ignored |

The coordinates are 0: the host does not know where the icon is on screen.
It sends no `ContextMenu` and no `Scroll`, so the shell shows `menu` itself,
as the example does.

Next: [7. Glass and blur](07-glass-and-blur.md).
