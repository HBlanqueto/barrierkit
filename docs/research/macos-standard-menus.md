# The standard menus of a macOS app

**Goal:** record the titles, order, shortcuts and enabled state of the menus
that SwiftUI builds for an app, so the kit's `Commands` copies them.
**Scope:** the menus of a SwiftUI `App` with a `WindowGroup`, a `Window` and a
`Settings` scene, plus the items that `TextEditingCommands`,
`TextFormattingCommands`, `SidebarCommands`, `ToolbarCommands` and
`InspectorCommands` add. It does not cover how a menu looks (see
[`macos-menu-bar.md`](macos-menu-bar.md)) or the menus of an app with documents.
**Reader:** anyone who changes `bk::app::Commands` or the standard actions.
**Input:** the probe `~/src/probes/appmenus.swift` and its dumps in
`~/src/captures/appmenus`.
**Output:** one table per menu, the Spanish titles, the Linux mapping and what
was not measured.

## Steps

1. Build the probe and run it per language:

   ```sh
   cd ~/src/probes
   swiftc -O -parse-as-library appmenus.swift -o appmenus
   ./appmenus.sh ~/src/captures/appmenus/en en
   ./appmenus.sh ~/src/captures/appmenus/es es
   ./appmenus.sh ~/src/captures/appmenus/en-cmd en -AppMenusCommands 1
   ```

2. Read `<prefix>-main.json` (the group's window is key), `-inspector.json`
   (the `Window` scene is key) and `-text.json` (a text view is first
   responder). `python3 ~/src/captures/appmenus/show.py <file>` prints one.
3. Copy only titles, order and key equivalents. The kit draws its own menus.

## Method

The probe is a SwiftUI `App` with `WindowGroup(id: "main")`,
`Window("Inspector", id: "inspector")` and `Settings`, and no `.commands`. It
runs as an app bundle, so its window is key. After launch it activates the
app, opens the `Window` scene, calls `NSMenu.update()` on every menu so the
items validate, and writes every item of `NSApp.mainMenu`: title, key
equivalent, modifier mask, `isEnabled`, `isHidden`, `isAlternate`, `state`,
action selector and submenu. Only public API is read.

| Item | Value |
|---|---|
| macOS | 27.2.1 (build 26B131) |
| Languages | `en` and `es`, per process with `-AppleLanguages "(en)"`; no system setting changes |
| App name in the dumps | `Probe` (the bundle name) |

A positional argument on the command line makes AppKit open it as a file and
the app shows no window. The probe takes its output path as
`-AppMenusOut <prefix>`, a defaults key.

## The measured menus

Shortcuts are written as macOS draws them. **Kit** is the id of the
[`bk::app`](../barrierkit/13-app.md#standard-menus) action and its shortcut on
Linux: ⌘ becomes Ctrl, ⌥ becomes Alt and ⇧ stays Shift, as for `UNDO` and
`TOGGLE_SIDEBAR`. A dash in **Kit** means the item is not built.

### App menu (titled with the app name)

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| About *App* | Acerca de *App* | none | — (the app shows its `AboutView`) |
| Settings… | Ajustes… | ⌘, | `SETTINGS`, Ctrl+, |
| Services ▸ | Servicios ▸ | none | — (no Linux equivalent) |
| Hide *App* | Ocultar *App* | ⌘H | — (Wayland has no hide) |
| Hide Others | Ocultar otras apps | ⌥⌘H | — |
| Show All | Mostrar todo | none, disabled | — |
| Quit *App* | Salir de *App* | ⌘Q | `QUIT`, Ctrl+Q |

Separators sit after About, after Settings…, after Services and after Show All.

### File

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| New Window | Nueva ventana | ⌘N | `NEW_WINDOW`, Ctrl+N |
| Close | Cerrar | ⌘W | `CLOSE_WINDOW`, Ctrl+W |
| Close All | Cerrar todo | ⌥⌘W, an alternate (hidden until Option is held) | — (the kit has no alternates) |

New Window exists because the app has a `WindowGroup`. Two separators sit
next to each other between New Window and Close; what group they close was not
checked.

### Edit

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| Undo | Deshacer | ⌘Z | `UNDO`, Ctrl+Z |
| Redo | Rehacer | ⇧⌘Z | `REDO`, Ctrl+Shift+Z |
| Cut | Cortar | ⌘X | `CUT`, Ctrl+X |
| Copy | Copiar | ⌘C | `COPY`, Ctrl+C |
| Paste | Pegar | ⌘V | `PASTE`, Ctrl+V |
| Delete | Eliminar | none | `DELETE` |
| Select All | Seleccionar todo | ⌘A | `SELECT_ALL`, Ctrl+A |
| Writing Tools ▸ (13 items) | Herramientas de escritura ▸ | none | — (Apple Intelligence) |
| AutoFill ▸ | Autorrelleno ▸ | none | — |
| Start Dictation… | Iniciar dictado… | none | — |
| Emoji & Symbols | Emojis y símbolos | the dump gives ⌘ plus a space on the visible item and ⌃⌘Space and fn E on hidden copies | — |

Separators sit after Redo, after Select All and after Writing Tools. With a window
key and nothing selected, Undo, Redo, Cut, Copy, Paste and Delete are
disabled and Select All is enabled only when a text view is first responder
(`-text.json`). Writing Tools items are disabled without a text view.

### Edit ▸ Find (added by `TextEditingCommands`)

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| Find… | Buscar… | ⌘F | `FIND`, Ctrl+F |
| Find and Replace… | Buscar y reemplazar… | ⌥⌘F | `FIND_AND_REPLACE`, Ctrl+Alt+F |
| Find Next | Buscar siguiente | ⌘G | `FIND_NEXT`, Ctrl+G |
| Find Previous | Buscar anterior | ⇧⌘G | `FIND_PREVIOUS`, Ctrl+Shift+G |
| Use Selection for Find | Usar selección para buscar | ⌘E (disabled without a selection) | `USE_SELECTION_FOR_FIND`, Ctrl+E |
| Jump to Selection | Ir a la selección | ⌘J | `JUMP_TO_SELECTION`, Ctrl+J |

The submenu is titled Find (Buscar). The same command also adds Spelling and
Grammar, Substitutions, Transformations and Speech submenus after it; the kit
has none of them (the text view's own context menu has the spelling items).

### View, Format, and the sidebar, toolbar and inspector items

The View menu of the probe is empty. `SidebarCommands`, `ToolbarCommands` and
`InspectorCommands` fill it:

| Item (en) | Shortcut | Kit |
|---|---|---|
| Show Toolbar | ⌥⌘T | — |
| Customize Toolbar… | none | — |
| Show Sidebar | ⌃⌘S | — (`TOGGLE_SIDEBAR`, Ctrl+Alt+S, is the kit's own) |
| Show Inspector | ⌃⌘I | — |

`TextFormattingCommands` adds a Format menu with Font ▸ (Show Fonts ⌘T, Bold
⌘B, Italic ⌘I, Underline ⌘U, Larger ⌘+, Smaller ⌘-, Show Colors ⇧⌘C, Copy Style
⌥⌘C, Paste Style ⌥⌘V) and Text ▸ (alignment, writing direction, ruler). The kit
builds none of it. Alignment shortcuts depend on the keyboard layout (the Spanish
layout gave ⇧⌘ñ, ⇧⌘ç and ⇧⌘´).

### Window

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| Minimize | Minimizar | ⌘M | `MINIMIZE`, Ctrl+M |
| Zoom | Zoom | none | `ZOOM` |
| *each* `Window` *scene* | its title | none | one item per `Window` scene |
| Bring All to Front | Traer todo al frente | none | `BRING_ALL_TO_FRONT` |
| *each open group window* | its title, ✓ on the key window | none | one item per open window of a `WindowGroup` |

Order: Minimize, Zoom, a separator, the `Window` scenes, a separator, Bring All
to Front, a separator, the open windows. A `Window` scene is listed even when
its window is closed (the first run had no window at all); the probe did not
click the item, so what it does with an open window was not tested. The open
windows of a `WindowGroup` are listed below, and the key one has state 1 (the
check). The `Window` scene's own window was not repeated in that list.

### Help

| Item (en) | Item (es) | Shortcut | Kit |
|---|---|---|---|
| *App* Help | Ayuda de *App* | ⌘? | `HELP`, F1 |

`⌘?` is ⇧⌘/ on a US layout and depends on the layout, so the kit uses F1, the
Linux help key. A hidden Toggle Sidebar item (⌥⌘S, disabled) sits in the menu.
The search field at the top of the menu is not an item of `NSMenu.items`.

## Not measured

- The Settings window's own menu state: `showSettingsWindow:` did not open it
  from the probe.
- Fill, Center, Move & Resize and Full Screen Tile in the Window menu, and
  Enter Full Screen in View: none appeared for the probe's windows.
- Close All with Option held, and the menus of an app with documents.
- How a menu looks open: [`macos-menu-bar.md`](macos-menu-bar.md).
- The title the Window menu shows after `window.title` changes.
