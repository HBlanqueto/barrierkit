# 13. App infrastructure

**Goal:** give an app what any desktop app has: actions with shortcuts, a menu
bar menu, saved settings, undo, a single instance and the desktop services.
**Scope:** starts with an app that already opens a window
([2. Your first app](02-first-app.md)) and ends with the services in
`barrier_kit::app`, `app_menu`, `status_item`, `notifications`, `secrets` and
`portal`. `Application` and `Scene` are in
[2](02-first-app.md#what-application-does-for-you), `WindowOptions` in
[8. Windows](08-windows.md), `MenuItem` and `CheckState` in
[7. Menus and popovers](07-menus-and-popovers.md), translated text (`tr!`) in
[17. Languages](17-languages.md).
**Reader:** anyone preparing an app for real use.
**Input:** an app with a window and some state. **Output:** the same app with
an Edit menu, a shortcut, undo and settings that survive a restart.

## Steps

1. Declare each command once, as a constant `Action` with its shortcut:

   ```rust
   const RESET: Action =
       Action::new("reset", || tr!("reset")).shortcut(Shortcut::ctrl(ShortcutKey::Char('r')));
   ```

2. Put the actions in the window's menu. The kit's `UNDO` and `REDO` go next to
   yours:

   ```rust
   Application::new("counter")
       .window("Counter", counter)
       .options(|window| {
           window.menu(AppMenu::new(vec![Menu::new(
               tr!("edit"),
               vec![UNDO.item(true), REDO.item(true), RESET.item(true)],
           )]))
       })
       .run()
   ```

3. Wrap the state that the user can undo with `use_undo`, and change it through
   the returned `UndoManager`:

   ```rust
   let count = use_state(|| 0);
   let history = use_undo(count);
   Button::new(tr!("add")).on_press(move |_| history.set(*count.peek() + 1))
   ```

4. Handle your own actions in a component with `use_menu_action`. The menu
   item and the shortcut both arrive here:

   ```rust
   use_menu_action(move |id| {
       if id == RESET.id {
           history.set(0);
       }
   });
   ```

5. Keep the menu in step with the state. `use_menu_enabled` publishes only when
   the value changes:

   ```rust
   use_menu_enabled(RESET.id, *count.read() != 0);
   ```

6. Read the settings with `use_defaults` and write them with `save_defaults`:

   ```rust
   #[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
   struct Settings {
       step: i32,
   }

   let settings = use_defaults::<Settings>("counter");
   save_defaults("counter", &Settings { step: 2 })?;
   ```

7. Optional: make the app single instance with `launch` before `run()`
   ([Single instance](#single-instance)).

[ADR 0047, annex A](../decisions/0047-barrierkit-in-layers.md#annex-a--how-an-app-is-written)
puts these pieces together in a complete app.

## Actions and shortcuts

An `Action` is a command with an id, a translated title and an optional
shortcut. One constant serves the menu bar menu, the shortcut, context menus
and the status item menu.

| Item | Signature | What it does |
|---|---|---|
| `Action::new` | `const fn new(id: &'static str, title: fn() -> &'static str) -> Action` | Creates the action; `title` runs each time the item is built, so it follows the language |
| `.shortcut` | `const fn shortcut(self, Shortcut) -> Action` | Adds the shortcut |
| `.id`, `.shortcut` | public fields | The id compared in `use_menu_action`; the shortcut, if any |
| `.title()` | `fn title(&self) -> &'static str` | The current title |
| `.item` | `fn item(&self, enabled: bool) -> MenuItem` | A menu item with the title and the shortcut text |
| `.item_with` | `fn item_with(&self, enabled: bool, check: CheckState) -> MenuItem` | The same, with a checkbox or radio mark |

`Shortcut` is a set of modifiers and one `ShortcutKey`:

| Item | What it is |
|---|---|
| `Shortcut::new(modifiers, key)` | Any `Modifiers` (`CONTROL`, `ALT`, `SHIFT`, `META`) and a key |
| `Shortcut::ctrl(key)` | `Shortcut::new(Modifiers::CONTROL, key)` |
| `.text()` | The menu text, modifiers in the order Ctrl, Alt, Shift, Super: `Ctrl+Shift+Z` |
| `.matches(modifiers, key)` | Whether a key press is this shortcut; letters match in both cases |
| `ShortcutKey::Char(c)` | A character key: `'r'`, `','` |
| `ShortcutKey::Left`, `Right` | The arrow keys |
| `ShortcutKey::Named(NamedKey)` | Any other named key: `ArrowUp`, `F11`, `PageDown` |

`use_menu_action(handler)` registers `handler: impl Fn(&str)` for the life of
the component. Every handler of the window receives every action id, on the
interface thread. The id comes from a menu bar item, a shortcut, a status item
menu, `STATUS_ITEM_ACTIVATE` or an `ActivateAction` sent to the
[single instance](#single-instance).

How a shortcut reaches the app:

1. On a key press, the kit looks for an **enabled** `MenuItem::Action` of the
   window's `AppMenu` whose shortcut text matches. A popup or utility window
   uses its parent's menu.
2. When one matches, the kit dispatches its id to `use_menu_action` and the key
   does not reach the app.
3. While a text field has focus, Ctrl+A, C, V, X, Y and Z (without Alt or
   Super) go to the field and never fire a menu shortcut.

A shortcut that is in no menu does nothing. A disabled item does not fire its
shortcut.

The kit declares these actions:

| Action | Shortcut | Handled by |
|---|---|---|
| `UNDO` | Ctrl+Z | [`use_undo`](#undo) |
| `REDO` | Ctrl+Shift+Z | [`use_undo`](#undo) |
| `GO_BACK`, `GO_FORWARD` | Alt+Left, Alt+Right | [11. Views](11-views.md) |
| `TOGGLE_SIDEBAR` | Ctrl+Alt+S | [10. Sidebar](10-sidebar.md) |

## Menu bar menu

`AppMenu` is the window's menu bar menu: a list of `Menu`, each with a title
and its `MenuItem`s. `WindowOptions::menu` gives it to the window.

| Item | What it is |
|---|---|
| `AppMenu::new(menus)` | The menu bar menu; `menus` is a public field |
| `Menu::new(title, items)` | One top-level menu; `title` and `items` are public fields |

The kit publishes the menu on the session bus:

1. It exports `com.canonical.dbusmenu` at `/MenuBar/<n>`. Labels escape `_`
   as `__`; shortcuts become key lists (`Control`, `Shift`, `S`); checkbox
   and radio marks become `toggle-type` and `toggle-state`.
2. If the compositor offers `org_kde_kwin_appmenu_manager`, the kit sends it
   the bus name and path. The Malaysia menu bar then shows the menu.
3. Without that protocol the menu still exists on the bus, and its shortcuts
   still work.

`app_menu::layout::LayoutItem` and `DbusMenuLayout` are the dbusmenu types of
`GetLayout`: `(id, properties, children)` and `(revision, root)`. An app does
not build them.

### Changing the menu live

The window's `MenuHandle` is a root context. A component takes it with
`try_consume_root_context::<MenuHandle>()`:

| Method | What it does |
|---|---|
| `update(id, \|item\| ..)` | Changes the action or submenu with that id: label, `enabled`, `check`, children |
| `set_menu_items(title, items)` | Replaces the items of the top-level menu with that title, for example a list of profiles |

Both publish only when something changed. A submenu gets an id with
`MenuItem::submenu(..).with_id(id)`.

From a component, the hooks are shorter. Each one publishes only when its value
changes between renders:

| Hook | Changes |
|---|---|
| `use_menu_enabled(id, enabled)` | `enabled` of an action or a submenu |
| `use_menu_label(id, label)` | The label of an action |
| `use_menu_checked(id, checked)` | The checkbox of an action |
| `use_submenu_items(id, items)` | The children of a submenu |
| `use_menu_update(id, value, apply)` | Any change: `apply: fn(&mut MenuItem, &T)` runs when `value` changes |

`find_menu_shortcut(menu, modifiers, key)` returns the id of the enabled
action whose shortcut matches the key press. The kit uses it for step 1 of the
shortcut path; an app that handles keys itself can call it too.

## Undo

`use_undo(state)` wraps a `State<T>` and returns its `UndoManager<T>`
(`T: Clone`). The manager is `Copy`, so closures capture it freely.

| Method | What it does |
|---|---|
| `set(value)` | Saves the current value on the undo stack, clears the redo stack and writes `value` |
| `undo()` | Goes back one value; no effect on an empty stack |
| `redo()` | Goes forward one value; no effect on an empty stack |
| `can_undo()`, `can_redo()` | Whether each stack has values |

`use_undo` also handles the `UNDO` and `REDO` actions, from the menu or the
shortcut, and disables their menu items while their stack is empty. A change
written straight to the `State` bypasses the history.

## Settings and restoration

Settings are user preferences. Restoration is the state the app finds again
after a restart: the selected pane, a search, the sidebar width. Both are a
struct with `Serialize`, `Deserialize` and `Default`, stored as TOML.

| Function | File |
|---|---|
| `load_defaults::<T>(app_id)`, `save_defaults(app_id, &value)` | `~/.config/malaysia/<app_id>.domain` |
| `use_defaults::<T>(app_id)` | The same; returns a `Memo<T>` that reloads when the file changes |
| `load_restoration::<T>(app_id)`, `save_restoration(app_id, &value)` | `~/.local/share/malaysia/<app_id>.state` |
| `read_toml::<T>(path)`, `write_toml(path, &value)` | Any path; the base of the others |
| `use_watched(load)` | Runs `load` again when a settings file changes; returns a `Memo` |

- Reading a missing file gives `T::default()`.
- Reading a file that does not parse logs a warning, copies it to `<file>.bak`
  and gives `T::default()`.
- Writing creates the folder, writes `<file>.tmp` and renames it over the file.
- `MALAYSIA_CONFIG_DIR` and `MALAYSIA_DATA_DIR` replace the two folders.
  Without them they are `malaysia/` under the XDG config and data homes.
- `use_defaults` and `use_watched` reload when any `.domain` file in the
  config folder changes, except `appearance.domain`.

The window size and the maximized state are saved and restored by the kit
([8. Windows](08-windows.md)).

## Single instance

`launch(app_id)` makes the first run of the app the only one. Call it in
`main`, before `run()`:

```rust
let instance = match launch("io.github.hblanqueto.malaysia.notes") {
    Ok(Launch::Forwarded) => return Ok(()),
    Ok(Launch::Primary(instance)) => Some(instance),
    Err(_) => None,
};
```

1. `launch` exports `org.freedesktop.Application` at the path made from
   `app_id` (`/io/github/hblanqueto/malaysia/notes`) and asks for the bus name
   `app_id`.
2. If the name is free, it returns `Launch::Primary(Instance)`.
3. If another process owns it, `launch` calls `Activate` on it with its
   `XDG_ACTIVATION_TOKEN`, and returns `Launch::Forwarded`. The app then exits.
4. In the first process, `instance.listen()` handles what arrives:
   `Activate` brings the window to the front (xdg-activation, with or without a
   token) and `ActivateAction(name)` dispatches `name` to `use_menu_action`.
   Call `listen()` once, from a component of the window, for example in
   `use_hook`.

Without a session bus, `launch` returns `Err` and the app runs without a
single instance. `Open` (opening files) is accepted and ignored.

## Status item

`StatusItem` is an icon with a menu in the desktop's status area.
`WindowOptions::status_item` publishes it with the window:

```rust
let icon = StatusItem::new("mail", "Mail", "mail-unread")
    .items(vec![MenuItem::action("new", tr!("new-message"))]);
Application::new("mail")
    .window("Mail", inbox)
    .options(|window| window.status_item(icon))
    .run()
```

| Item | What it is |
|---|---|
| `StatusItem::new(id, title, icon_name)` | `id` and `title` for the host; `icon_name` from the icon theme |
| `.items(items)` | Its menu |
| `STATUS_ITEM_ACTIVATE` | The action id (`"status-item.activate"`) sent when the icon is clicked |
| `Pixmaps` | The `IconPixmap` type of the specification; the kit sends it empty |

It follows the
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/)
protocol:

1. The kit exports `org.kde.StatusNotifierItem` at `/StatusNotifierItem` on
   its own bus connection, named `org.kde.StatusNotifierItem-<pid>-<n>`, and
   its menu as dbusmenu on the same connection.
2. It registers with `org.kde.StatusNotifierWatcher`. If the watcher appears
   later (the shell starts after the app or restarts), it registers again.
3. Menu items reach `use_menu_action` with their id. A click on the icon
   arrives as `STATUS_ITEM_ACTIVATE`.

The specification spells the interfaces `org.freedesktop.*`; KDE,
libayatana-appindicator and waybar use `org.kde.*`, and so does the kit.

## Notifications

`notifications::post(notice, clicked)` posts a notification through
`org.freedesktop.Notifications`, on a background thread.

| `Notice` field | What it is |
|---|---|
| `app_name: &'static str` | The app name the server shows |
| `desktop_entry: &'static str` | The `desktop-entry` hint, so the server finds the icon |
| `summary`, `body` | The text |
| `action` | The label of the default action |

`clicked: FnOnce(Option<String>)` runs when the user clicks the notification.
It gets the xdg-activation token the server sends, so the app can bring its
window to the front. It never runs if the notification closes unclicked.
Without a notification server, `post` logs a warning and does nothing.

`play_event_sound(name)` plays a sound from the freedesktop sound theme
(`"message-new-email"`) with `canberra-gtk-play`.

## Secrets

`Secrets` is the Linux keyring, through the Secret Service API
(`org.freedesktop.secrets`).

| Item | What it does |
|---|---|
| `Secrets::open()` | Opens a session; returns `Result<Secrets, Error>` |
| `search(&attributes)` | Finds items by attributes (`HashMap<&str, &str>`); unlocks locked ones |
| `secret(&item)` | Reads the secret as `Zeroizing<Vec<u8>>` |
| `create(label, &attributes, secret)` | Creates an item in the default collection, replacing one with the same attributes |
| `set_secret(&item, secret)` | Changes the secret |
| `delete(&item)` | Deletes the item |

An `Item` has a `path`, a `label` and its `attributes`. `Error::Locked` means
the keyring asked to be unlocked and is showing its own prompt: try again
later. `Error::Bus` wraps any other D-Bus error. Secrets travel in plain over
the session bus, which is local to the user; the copies the kit makes are
zeroed.

## Portals

`barrier_kit::portal` talks to `org.freedesktop.portal.Desktop`. The calls are
`async`: run them with `spawn` from a component. Failures log a warning.

| Function | Portal | Returns |
|---|---|---|
| `open_files(title, multiple, directory)` | `FileChooser.OpenFile` | `Option<Vec<PathBuf>>`; `None` if cancelled |
| `save_file(title, current_name)` | `FileChooser.SaveFile` | `Option<PathBuf>` |
| `open_uri(uri, ask)` | `OpenURI`; `ask` lets the user pick the app | — |
| `notify(id, title, body)` | `Notification.AddNotification` | — |
| `print(title, document)` | `Print`: `document(&PageSetup)` returns the PDF bytes | — |

`print` asks for the page setup first, calls `document` with it and sends the
PDF. `PageSetup` and the PDF helpers are in
[6. Text and editing](06-text-and-editing.md).

`file_uri(path)` turns a path into a `file://` URI with percent-encoding;
`file_path(uri)` does the reverse and returns `None` for other schemes.

## Clipboard

The kit serves Freya's clipboard over `wl_data_device`. The app uses
`Clipboard` from `freya-clipboard`, which it adds to its `Cargo.toml`:

```rust
use freya_clipboard::prelude::Clipboard;

let copied = Clipboard::set("text".to_string());
let pasted: Result<String, ClipboardError> = Clipboard::get();
```

Both calls read a root context, so they run inside the app: in a component or
an event handler. Without a clipboard they return `ClipboardError::NotAvailable`.
Text fields already copy, cut and
paste ([6. Text and editing](06-text-and-editing.md)).

## Drag and drop

Files dropped on a window arrive as Freya file events:

1. While a drag is over the window, every element with
   `on_global_file_hover` gets the cursor position.
2. On drop, the kit reads the `text/uri-list` offer, turns each `file://` URI
   into a path with `file_path`, and sends one `on_file_drop` event per path
   to the element under the cursor.
3. `on_global_file_hover_cancelled` fires when the drag leaves.

```rust
rect().on_file_drop(move |event: Event<FileEventData>| {
    if let Some(path) = event.file_path.clone() {
        open(path);
    }
})
```

Only files are received. Dragging files or text out of the app, and the image
well that accepts dropped images, are in [5. Controls](05-controls.md).

Next: [14. Animations](14-animations.md).
