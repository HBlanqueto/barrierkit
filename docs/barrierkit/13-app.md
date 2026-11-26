# 13. App infrastructure

**Goal:** give an app what any desktop app has: actions with shortcuts, a menu
bar menu, saved settings, undo, a single instance and the desktop services.
**Scope:** starts with an app that already opens a window
([2. Your first app](02-first-app.md)) and ends with the services in
`bk::app`, `main_menu`, `status_item`, `notifications`, `secrets` and
`portal`. `Application` is in
[2](02-first-app.md#what-application-does-for-you), `WindowOptions` and the
scenes in [8. Windows](08-windows.md), `MenuItem` and `CheckState` in
[7. Menus and popovers](07-menus-and-popovers.md), translated text (`tr!`) in
[17. Languages](17-languages.md).
**Reader:** anyone preparing an app for real use.
**Input:** an app with a window and some state. **Output:** the same app with
an Edit menu, a shortcut, undo and settings that survive a restart.

## Steps

The prelude holds `Application`, `Commands`, `CommandGroup`,
`CommandGroupPlacement` and `use_app_storage`. Import the rest from `bk::app`
and `bk::main_menu`.

1. Declare each command once, as a constant `Action` with its shortcut:

   ```rust
   const RESET: Action =
       Action::new("reset", || tr!("reset")).shortcut(Shortcut::ctrl(ShortcutKey::Char('r')));
   ```

2. Add the actions to the [standard menus](#standard-menus). The Edit menu
   already has Undo and Redo; put yours after them:

   ```rust
   Application::new("com.example.counter")
       .window("Counter", counter)
       .commands(Commands::new().group(CommandGroup::after(
           CommandGroupPlacement::UndoRedo,
           vec![RESET.item(true)],
       )))
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

6. Keep a setting with `use_app_storage`. It gives a `State` that saves
   itself:

   ```rust
   let mut step = use_app_storage("step", 1);
   step.set(2);
   ```

7. Optional: make the app single instance with `launch` before `run()`
   ([Single instance](#single-instance)).

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
| `Shortcut::DEFAULT_ACTION`, `CANCEL_ACTION` | Enter and Escape with no modifier, SwiftUI's `.defaultAction` and `.cancelAction` |
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
   window's `MainMenu` whose shortcut text matches. A popup or utility window
   uses its parent's menu.
2. When one matches, the kit dispatches its id to `use_menu_action` and the key
   does not reach the app.
3. While a text field has focus, Ctrl+A, C, V, X, Y and Z (without Alt or
   Super) go to the field and never fire a menu shortcut.

A shortcut that is in no menu and on no `Button` ([5](05-controls.md#buttons))
does nothing. A disabled item does not fire its shortcut.

The kit does not route an action to the focused view, and it has no quit call
([ADR 0085](../decisions.md)). The close veto and the open-URL event are in
[8](08-windows.md#documents-and-file-dialogs).

The kit declares these actions:

| Action | Shortcut | Handled by |
|---|---|---|
| `UNDO` | Ctrl+Z | [`use_undo`](#undo) |
| `REDO` | Ctrl+Shift+Z | [`use_undo`](#undo) |
| `GO_BACK`, `GO_FORWARD` | Alt+Left, Alt+Right | [11. Views](11-views.md) |
| `TOGGLE_SIDEBAR` | Ctrl+Alt+S | [10. Sidebar](10-sidebar.md) |

## Standard menus

`Commands` gives the windows of an `Application` the menus that SwiftUI gives
an app. Titles, order and shortcuts come from a SwiftUI app measured on
macOS 27.2.1 in English and Spanish
([the measurement](../research/macos-standard-menus.md)). Every `Application`
has them (ADR 0115); a scene whose options carry a `MainMenu` keeps that one.
They are data over `MenuItem`, so no app builds the same menus by hand.
An app that draws its own title bar sets `HiddenTitleBar` in
`.options(..)`: its window gets no kit `Titlebar`, and keeps the menus.

1. To change them, pass `.commands(Commands::new()..)` to the `Application`.
2. Add your own menus with `.menu(Menu::new(title, items))` (SwiftUI's
   `CommandMenu`). They sit between View and Window.
3. Add to a standard group, or replace it, with
   `.group(CommandGroup::after(CommandGroupPlacement::NewItem, items))`
   (`before`, `after` and `replacing`).
4. Add the Find submenu with `.text_editing()` (SwiftUI's
   `TextEditingCommands`).
5. Handle your own ids with `use_menu_action`. The kit handles the ones marked
   so below; for the others call `use_menu_enabled(id, true)` once the app
   serves them.

The menus, in order, and the groups you can place items in:

| Menu | Groups, in order | Standard items |
|---|---|---|
| the app name | `AppInfo`, `AppSettings`, `AppTermination` | About *App*, Settings…, Quit *App* |
| File | `NewItem`, `SaveItem` | New Window, Close |
| Edit | `UndoRedo`, `Pasteboard`, `TextEditing` | Undo, Redo; Cut, Copy, Paste, Delete, Select All; Find ▸ (only with `.text_editing()`) |
| View | | Enter Full Screen |
| your `.menu(..)` menus | | |
| Window | `WindowSize`, `SingleWindowList`, `WindowArrangement`, `WindowList` | Minimize, Zoom; one item per `Window` scene; Bring All to Front; one item per open window |
| Help | `Help` | *App* Help |

A menu without items is not published. The Spanish titles are in the
[measurement](../research/macos-standard-menus.md#the-measured-menus).

Each action is a constant in `bk::app`. A shortcut is the macOS one with ⌘ as
Ctrl, ⌥ as Alt and ⇧ as Shift; Help is F1, which does not depend on the
keyboard layout.

| Action | Shortcut | Handled by | Enabled |
|---|---|---|---|
| `ABOUT` | none | the kit: opens the About window | always |
| `NEW_WINDOW` | Ctrl+N | the kit: opens another window of the first `WindowGroup` | always; the item exists only with a `WindowGroup` without a value, as in SwiftUI |
| `CLOSE_WINDOW` | Ctrl+W | the kit: closes this window | always |
| `SETTINGS` | Ctrl+, | the kit: opens the `Settings` scene | always; the item exists only with a `Settings` scene, as in SwiftUI |
| `QUIT` | Ctrl+Q | the kit: closes every window, so the process ends | always |
| `MINIMIZE` | Ctrl+M | the kit | always |
| `ZOOM` | none | the kit: maximizes or restores | not for a fixed-size window |
| `BRING_ALL_TO_FRONT` | none | the kit | always |
| `TOGGLE_FULL_SCREEN` | F11 (⌃⌘F on macOS) | the kit: enters or leaves full screen | not for a fixed-size window |
| `UNDO`, `REDO` | Ctrl+Z, Ctrl+Shift+Z | [`use_undo`](#undo) | while a stack has a value |
| `CUT`, `COPY`, `PASTE`, `SELECT_ALL` | Ctrl+X, C, V, A | a focused text field, else the app | while a text field is focused, or after `use_menu_enabled` |
| `DELETE` | none | the app | after `use_menu_enabled` |
| `FIND`, `FIND_AND_REPLACE`, `FIND_NEXT`, `FIND_PREVIOUS`, `USE_SELECTION_FOR_FIND`, `JUMP_TO_SELECTION` | Ctrl+F, Ctrl+Alt+F, Ctrl+G, Ctrl+Shift+G, Ctrl+E, Ctrl+J | the app | after `use_menu_enabled` |
| `HELP` | F1 (⌘? on macOS until the app serves it) | the app | after `use_menu_enabled` |

Undo, Redo, Cut, Copy, Paste and Select All go to a focused text field first,
as AppKit sends them to the first responder: the field gets its own shortcut.
Otherwise they reach the app's `use_menu_action`. The kit routes no other
command to the focused view ([ADR 0085](../decisions.md)), so Delete, Find and
Help start disabled until the app serves them.

On macOS the menus are AppKit's: the app menu carries the bundle name,
Settings… and Services; Enter Full Screen, Minimize, Zoom, Bring All to Front
and a Help the app does not serve are AppKit's own items, so AppKit adds Fill,
Center, Move & Resize and Full Screen Tile to Window and a search field to Help
(ADR 0115).

About *App* is AppKit's standard panel on macOS, filled from the bundle. On
Linux `ABOUT` opens the scene with the id `ABOUT_WINDOW`: the kit's own shows
the app name in an [`AboutView`](12-patterns.md#aboutview). To show an icon, a
version and a copyright, declare a `Window::new(ABOUT_WINDOW, title, content)`
scene; yours opens instead.

```swift
// ✗ Button("Copy") { }.keyboardShortcut("c") in a CommandGroup, enabled by the responder chain
// ✓ use_menu_enabled(COPY.id, *selection.read() != 0) and a use_menu_action that copies
```

### The Window menu

The kit lists the open windows itself: Wayland has no window list
([8. Windows](08-windows.md#scenes)). Each scene window publishes the items
of its own Window menu when the list changes:

1. A `Window` scene has one item, `window-open:<id>`, that opens or fronts it.
2. Each open window of a `WindowGroup` has an item, `window-activate:<n>`,
   that fronts it. The item of the window that owns the menu is checked.
3. Titles are the ones the scene declared. `set_title` does not change them.

### Quit

`QUIT` closes every window of the app, one by one. When a window's `Document`
has unsaved changes, `QUIT` closes only that window, which asks first; the user
quits again after answering ([8](08-windows.md#documents-and-file-dialogs)).
The kit has no function that quits.

## Menu bar menu

`MainMenu` is the window's menu bar menu: a list of `Menu`, each with a title
and its `MenuItem`s. `WindowOptions::menu` gives it to the window.

| Item | What it is |
|---|---|
| `MainMenu::new(menus)` | The menu bar menu; `menus` is a public field |
| `Menu::new(title, items)` | One top-level menu; `title` and `items` are public fields |

The kit publishes the menu on the session bus:

1. It exports `com.canonical.dbusmenu` at `/MenuBar/<n>`. Labels escape `_`
   as `__`; shortcuts become key lists (`Control`, `Shift`, `S`); checkbox
   and radio marks become `toggle-type` and `toggle-state`.
2. If the compositor offers `org_kde_kwin_appmenu_manager`, the kit sends it
   the bus name and path. The Malaysia menu bar then shows the menu.
3. Without that protocol the menu still exists on the bus, and its shortcuts
   still work.

`LayoutItem` and `DbusMenuLayout` are the dbusmenu types of `GetLayout`:
`(id, properties, children)` and `(revision, root)`. They live in
`platform/linux/dbusmenu/layout.rs` and the shell reaches them through
`bk::extension`. An app does not build them.

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
| `use_app_storage(key, default)` | `key` of `<app_id>.domain`, any `T` with `Serialize`, `Deserialize`, `Clone` and `PartialEq`; returns a `State<T>` that saves the key when the value changes |
| `load_defaults::<T>(app_id)`, `save_defaults(app_id, &value)` | `<app_id>.domain` in the app's settings folder |
| `use_defaults::<T>(app_id)` | The same; returns a `Memo<T>` that reloads when the file changes |
| `update_defaults::<T>(app_id, \|value\| ..)` | The same; loads the file, runs the closure on the value and saves it |
| `load_restoration::<T>(app_id)`, `save_restoration(app_id, &value)` | `<app_id>.state` in the app's state folder; `SettingsWindow` keeps its own state in `<app_id>.settings.state` beside it |
| `read_toml::<T>(path)`, `write_toml(path, &value)` | Any path; the base of the others |
| `use_watched(load)` | Runs `load` again when a settings file changes; returns a `Memo` |

- `use_app_storage` is SwiftUI's `@AppStorage`: one key, read once when the
  view appears and written only when the value changes, keeping the other keys
  of the file. It uses the id of the running app (`Application::new` or
  `run_window`); before an app runs, as in a test, the value lives in memory
  only. It does not reload when another process changes the file
  ([`recipes/app_storage.rs`](../../crates/bk/examples/app_storage.rs)).
- Reading a missing file gives `T::default()`.
- Reading a file that does not parse logs a warning, copies it to `<file>.bak`
  and gives `T::default()`.
- Writing creates the folder (mode 0700), writes a temporary file beside the
  file and renames it over the file. `update_defaults` and `use_app_storage`
  hold `<file>.lock` while they read and save, so two processes keep each
  other's keys.
- Each app has its own folders, named by its `app_id`, like macOS's
  `~/Library/Preferences/<bundle id>.plist`, so an app built on the kit never
  writes into Malaysia's folder:

  | System | Settings | State |
  |---|---|---|
  | Linux | `$XDG_CONFIG_HOME/<app_id>/` (`~/.config/<app_id>/`) | `$XDG_DATA_HOME/<app_id>/` (`~/.local/share/<app_id>/`) |
  | macOS | `~/Library/Application Support/<app_id>/` | The same |

  A shell component follows the same rule; to keep a file in the desktop's
  folder it calls `read_toml` and `write_toml` with that path.

- A file saved before in the desktop's folder (`config_dir()`, `data_dir()`)
  moves to the app's folder the first time it is read. If the move fails, the
  kit keeps reading the old file.
- `use_defaults` reloads when its own `<app_id>.domain` changes and when
  `wallpaper.state` changes. `use_watched` cannot know which file `load` reads,
  so it reloads when any `.domain` file changes in the desktop's config folder
  or in a folder that `use_defaults` watches, except `appearance.domain`.
  An app that calls only `use_defaults` ignores the
  settings other apps save. On macOS the kit looks for changed `.domain`
  files once a second.

On Linux the kit saves and restores the window size and the maximized state
([8. Windows](08-windows.md#window-options)); on macOS it does not.

## Updates from another thread

State belongs to the window's thread. A thread that watches D-Bus, a file or a
socket hands its values over with `use_published`, or only asks for a redraw
with `use_wake`.

```rust
// ✗ a mailbox, a wakeup and a closure written by hand in every hook
// ✓
let focus = use_published(None, |focus| watch_focus(move |next| focus.send(next)));
let revision = use_wake(|wake| watch_theme(move || wake.wake()));
```

| Function | Effect |
|---|---|
| `use_published(initial, start)` | Returns a `State<T>`. `start` runs once and receives a `Publisher<T>`; `send(value)` from any thread sets the state on the window's thread, only if it changed |
| `Publisher::send(value)` | Keeps the last value and wakes the window. A burst of sends shows only the last one |
| `use_wake(start)` | Runs `start` once with a `WakeHandle`; every `wake()` renders the component again. Returns how many wakes there have been, to use as a cache key |

Both hooks stop listening when the component goes away.

`T` must be `Clone + PartialEq + Send`. Both hooks follow the rules of hooks:
call them on every render, in the same order.

## Single instance

`launch(app_id)` makes the first run of the app the only one. Call it in
`main`, before `run()`:

```rust
let instance = match launch("com.example.notes") {
    Ok(Launch::Forwarded) => return Ok(()),
    Ok(Launch::Primary(instance)) => Some(instance),
    Err(_) => None,
};
```

1. `launch` exports `org.freedesktop.Application` at the path made from
   `app_id`, with `.` as `/` and `-` as `_` (`/com/example/notes`), and asks
   for the bus name `app_id`.
2. If the name is free, it returns `Launch::Primary(Instance)`.
3. If another process owns it, `launch` calls `Activate` on it with its
   `XDG_ACTIVATION_TOKEN`, and returns `Launch::Forwarded`. The app then exits.
4. In the first process, `instance.listen()` handles what arrives:
   `Activate` brings the window to the front (xdg-activation, with or without a
   token), `Open(uris)` hands each URI to `on_open_url`
   ([8](08-windows.md#documents-and-file-dialogs)) and then activates, and
   `ActivateAction(name)` dispatches `name` to `use_menu_action`.
   Call `listen()` once, from a component of the window, for example in
   `use_hook`.

Without a session bus, `launch` returns `Err` and the app runs without a
single instance. On macOS there is no bus: `launch` takes an exclusive lock on
`$TMPDIR/<app_id>.lock` and listens on the socket `$TMPDIR/<app_id>.sock`; a second run
connects to the socket, which is the activation, and returns `Launch::Forwarded`. The
first run brings its window to the front and activates the app. `launch` returns `Err`
when the socket path is too long (about 100 bytes), and the system drops the lock when
the process ends, so a crash leaves nothing behind. Launch Services already keeps a
bundled app to one process when it is opened from the Finder or the Dock; the lock also
covers a bare binary and `open -n`. A second run forwards no action and no files; the files
the Finder opens reach `on_open_url` through the app delegate.

## Status item

`StatusItem` is an icon with a menu in the desktop's status area.
`WindowOptions::status_item` publishes it with the window:

```rust
let icon = StatusItem::new("mail", "Mail", "mail-unread")
    .items(vec![MenuItem::action("new", tr!("new-message"))]);
Application::new("com.example.mail")
    .window("Mail", inbox)
    .options(|window| window.status_item(icon))
    .run()
```

| Item | What it is |
|---|---|
| `StatusItem::new(id, title, icon_name)` | `id` and `title` for the host; `icon_name` from the icon theme |
| `.items(items)` | Its menu |
| `STATUS_ITEM_ACTIVATE` | The action id (`"status-item.activate"`) sent when the icon is clicked |
| `Pixmaps` | The `IconPixmap` type of the specification (`bk::extension`, Linux); the kit sends it empty |

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

On macOS the item is an `NSStatusItem` (`platform/macos/status.rs`) that lives as long as
its window. `icon_name` is looked up with `NSImage.imageNamed` (an image of the app's
bundle or a system one) and drawn as a template; without one the button shows `title`.
With items, a click opens the menu and its actions reach `use_menu_action` with their id;
without items, a click sends `STATUS_ITEM_ACTIVATE`. The items are fixed when the window
opens, as on Linux.

## Notifications

`notifications::post(notice, clicked)` posts a notification through
`org.freedesktop.Notifications`, on a background thread.

| `Notification` field | What it is |
|---|---|
| `app_name: &'static str` | The app name the server shows |
| `desktop_entry: &'static str` | The `desktop-entry` hint, so the server finds the icon |
| `summary`, `body` | The text |
| `action` | The label of the default action |

`clicked: FnOnce(Option<String>)` runs when the user clicks the notification.
It gets the xdg-activation token the server sends, so the app can bring its
window to the front. It never runs if the notification closes unclicked.
Without a notification server, `post` logs a warning and does nothing.

On macOS `post` uses `UNUserNotificationCenter` (`platform/macos/notifications.rs`). The
first call asks the user's permission. `summary` and `body` are the title and the text; the
click on the banner runs `clicked(None)` (there is no activation token), and the system
brings the app forward. `app_name`, `desktop_entry` and `action` are not used: the app's
bundle gives the name and the icon, and the banner has no action button. A process
without a bundle identifier (a bare binary from `cargo run`) cannot post: `post` logs a
warning and does nothing, so run the app from an `.app` bundle.

`play_event_sound(name)` plays a sound from the freedesktop sound theme
(`"message-new-email"`) with `canberra-gtk-play`. On macOS it plays the system
`NSSound` of that name (`"Glass"`).

## Secrets

`Secrets` is the Linux keyring, through the Secret Service API
(`org.freedesktop.secrets`). On macOS it is the login Keychain: each item is a generic
password whose account is the label and whose service holds the attributes, so `search`
reads the kit's items and filters them by attributes; `Item::path` is the same for all.

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
later. `Error::Bus` wraps any other D-Bus error; on macOS `Error::Keychain` holds the
Keychain's `OSStatus`, and `Locked` does not occur (the system asks for the password itself). Secrets travel in plain over
the session bus, which is local to the user; the copies the kit makes are
zeroed.

## Portals

`bk::portal` talks to `org.freedesktop.portal.Desktop`. The calls are
`async`: run them with `spawn` from a component. Failures log a warning.

| Function | Portal | Returns |
|---|---|---|
| `open_files(title, multiple, directory)` | `FileChooser.OpenFile` | `Option<Vec<PathBuf>>`; `None` if cancelled |
| `save_file(title, current_name)` | `FileChooser.SaveFile` | `Option<PathBuf>` |
| `open_uri(uri, ask)` | `OpenURI`; `ask` lets the user pick the app | — |
| `notify(id, title, body)` | `Notification.AddNotification`; on macOS a banner with `id` as its identifier (see [Notifications](#notifications)) | — |

For filters, a start folder and, on macOS, a sheet of its window, use `FileDialog` ([8](08-windows.md#documents-and-file-dialogs)); `open_files` and `save_file` keep the portal's arguments.

`file_uri(path)` turns a path into a `file://` URI escaped like GLib;
`file_path(uri)` does the reverse and returns `None` for other schemes.

## Clipboard

The kit serves Freya's clipboard over `wl_data_device` on Linux and
`NSPasteboard` on macOS. The app uses
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

### Typed clipboard

Module `pasteboard`. `Pasteboard::general()` is the system clipboard and `Pasteboard::named("name")` a
private board; recipe [43](../../.claude/skills/barrierkit-app/recipes.md#43-copy-paste-and-drag-typed-data) runs it.
An app copies more than text (rich text, images, files), as `NSPasteboard` and SwiftUI's `Transferable` do.

```rust
use bk::drag::DragPayload;
use bk::pasteboard::Pasteboard;

let board = Pasteboard::general();
board.copy(&DragPayload::Html { text: "Hello".into(), html: "<b>Hello</b>".into() });
let text: Option<String> = board.paste();
let anything: Option<DragPayload> = board.paste();
```

`copy` replaces the board and returns false when the platform cannot hold the type. `paste` reads the richest
item (files, then an image, then rich text, then text) and gives `None` when the item is not a `T`. A type of the
app implements `Transferable` (`payload` and `from_payload`) to travel as one of the four `DragPayload` shapes.

| Payload | macOS (`NSPasteboard`) | Linux |
|---|---|---|
| `Text` | `public.utf8-plain-text` | the Wayland clipboard |
| `Html { text, html }` | both types | both: `text/html` and the text |
| `Image(png)` | `public.png` and `public.tiff`; a TIFF alone reads as PNG | not held: `copy` returns false |
| `Files(paths)` | one `public.file-url` item per path, with the path as text | not held: `copy` returns false |
| a named board | private to the process, shared with others that use the name | not held: false and `None` |

Linux reads text only. The read has no size cap on macOS: the system already holds the bytes.

## Drag and drop

Files dropped on a window arrive as Freya file events. On macOS the content
view is the `NSDraggingDestination` ([18](18-platform.md#the-macos-runner)).

1. While a drag is over the window, every element with
   `on_global_file_hover` gets the cursor position.
2. On drop, the kit reads the `text/uri-list` offer (the file URLs of the
   dragging pasteboard on macOS), turns each `file://` URI into a path with
   `file_path`, and sends one `on_file_drop` event per path
   to the element under the cursor; its `file_paths` holds that one path.
3. `on_global_file_hover_cancelled` fires when the drag leaves.

```rust
rect().on_file_drop(move |event: Event<FileEventData>| {
    if let Some(path) = event.file_paths.first().cloned() {
        open(path);
    }
})
```

Only files are received. Dragging files or text out of the app, and the image
well that accepts dropped images, are in [5. Controls](05-controls.md).

Next: [14. Animations](14-animations.md).
