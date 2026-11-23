# Window designs and window states: Swift against the kit

**Goal:** list the window designs a Swift app on macOS can use to draw its
own title bar, record how each one looks and behaves in every window state the
system has, and say which kit option reproduces it. Each design row ends in
a kit change or in a documented ceiling.
**Scope:** main windows on one display: the system title bar, the system
title bar with a toolbar, the hidden title bar (the app's bar beside the
native lights) and the borderless window (the app draws everything). States:
active, inactive, zoomed, tiled, full screen and minimized. It excludes
panels, HUDs, sheets, tabs, Stage Manager and several displays.
**Snapshot:** macOS 27.2, swiftc 6.4, repo `273c9fca`, 2026-11-23.
**Reader:** anyone who changes `WindowStyle`, `WindowToolbarStyle`,
`Titlebar`, `WindowChrome` or `crates/bk/src/platform/macos`.
**Input:** [fullscreen-titlebars.md](fullscreen-titlebars.md) (20 full-screen
designs, measured), [macos-windows.md](macos-windows.md) (windowed geometry
and lights), [macos-corners.md](macos-corners.md),
[macos-native-chrome.md](macos-native-chrome.md) and one probe pass per
design (section 4).
**Output:** the matrix of section 2, filled by the sessions of section 4, and
the kit changes they commit.

## 1. The states, by their system names

| State | How the user enters it | AppKit | SwiftUI | Kit today |
| ----- | ---------------------- | ------ | ------- | --------- |
| Active | click the window | `isKeyWindow`, `isMainWindow` | `@Environment(\.appearsActive)` | `WindowStatus::activated` |
| Inactive | another window is key | `windowDidResignKey` | `appearsActive == false` | `activated == false` |
| Zoomed | green button with Option, title-bar double click (setting "Zoom"), Window > Zoom | `zoom(_:)`, `isZoomed` | none | `WindowStatus::maximized` |
| Tiled | green button menu or Window > Move & Resize (halves, quarters, Fill, Center), drag to an edge | no public state | none | not reported |
| Full screen | green button, Control-Command-F | `toggleFullScreen(_:)`, `.fullScreen` in `styleMask` | `.windowFullScreenBehavior` | `WindowStatus::fullscreen` |
| Minimized | yellow button, Command-M | `miniaturize(_:)`, `isMiniaturized` | none | not reported (no surface) |

## 2. Designs against states

"Measured" cells point at their source. "Owed" cells are filled by the
session named in section 4.

| # | Swift design | Kit option | Active / inactive | Zoomed | Tiled | Full screen |
| - | ------------ | ---------- | ----------------- | ------ | ----- | ----------- |
| A | `WindowGroup`, no toolbar (system bar) | `WindowStyle::TitleBar` | same in Swift and kit (the kit asks AppKit for the bar): 32 px bar, lights at (9, 9), title ink at x 85 in gray (165 of 255 in Dark), corner 16 (diagonal inset 5 px), shadow 56 px active and 23 px inactive (`csd-ab/swift-A-*`, `csd-ab/kit-A-*`) | same in Swift and kit: Zoom fills the visible frame `0,30 2560×1325`; bar, lights, corner 16 and the 56 px shadow do not change; animation 21 fr / 333 ms | same in Swift and kit, nothing changes but the frame: Left `0,30 1280×1325`, Top Left `1280×662`, Fill and a drag to the top `2560×1325`, Center moves only; corner 16, shadow 56 px, lights and focus as windowed; Swift `isZoomed` is true after Fill only (`tile/swift-A-*`, `tile/kit-app-title-bar-*`) | same in Swift and kit: no shift, the 32 px title strip waits at `y −32` (alpha 0) and slides to `y 30` with the menu bar at the top edge (rows 1 and 2 of [fullscreen-titlebars.md](fullscreen-titlebars.md)) |
| B | `.toolbar{}` with `.windowToolbarStyle(.unified)`, `.unifiedCompact`, `.expanded` | `WindowStyle::HiddenTitleBar` and `Titlebar::toolbar_style` (`Unified`, `UnifiedCompact`, `Expanded`); recipe `full_screen.rs hidden-title-bar unified\|compact\|expanded` | Swift (`csd-ab/swift-B-*`): `.unified` 52 px, lights (19, 19), title white at the start, items 36 high; `.unifiedCompact` 40 px, lights (12, 13), items 24; `.expanded` 68 px = a 28 px title row (lights (9, 9), title gray at x 85, as A) over a 40 px toolbar row whose `.navigation` and `.primaryAction` items sit together at the start, 28 high. Corner 16 and shadow 56 / 23 px as A in all three. Kit (`csd-ab/kit-B-*`): unified and compact bars of the same heights and lights; `Expanded` added in this pass: 28 + 40 rows, native lights at (9, 9) through an `.expanded` empty `NSToolbar`, title ink at x 85, y 12 (Swift 11). Differs: the first item starts at x 86 in the unified bar (Swift 95), x 75 in the compact one, touching the lights (Swift 94), and x 7 in the expanded row (Swift 11); expanded items are the compact 24 (Swift 28); the title of the expanded row is the bar title color (234), not Swift's gray (165); the kit window background is 30 where Swift's bar is 41 | as A in Swift and kit: `0,30 2560×1325`, the bar keeps its height and items, corner 16, shadow 56 px (`*-zoomed*`) | Swift `.unified` as A, toolbar unchanged (`tile/swift-B-*`); the kit's B is a `HiddenTitleBar` window and tiles as C (`tile/kit-notes-hidden-title-bar-*`); a `run_window` window has no Window menu and its green-button menu is not in the AX tree, so `csd-ab` has no kit tile | Swift pins the bar and shifts the content by its height, 52 / 40 / 68 px (safe area `t52` in full screen), lights hidden; at the top edge the strip slides to `y 30` with the menu bar and shows the lights. `.windowToolbarFullScreenVisibility(.onHover)` is AppKit's `.autoHideToolbar` (row 11): strip at `y −52` alpha 0, shift 0, slides in at the edge (`csd-ab/swift-B-onhover-*`). Kit: the bar keeps its windowed place at the top, so the shift is the same 52 / 40 / 68 px; the expanded title row is empty, as in Swift. At the top edge AppKit's 32 px strip covers the bar; since `907de855` (ADR 0116) the glass controls of the first 52 px ride down beside the lights. Owed: the expanded toolbar row (28 to 68) does not ride, and `.onHover` has no kit option yet |
| C | `.windowStyle(.hiddenTitleBar)` (alone, with `.toolbar{}`, `.unifiedCompact`, `.toolbarBackgroundVisibility(.hidden, for: .windowToolbar)`, `.toolbar(removing: .title)`, `NavigationSplitView`), AppKit `titlebarAppearsTransparent` + `.fullSizeContentView` | `WindowStyle::HiddenTitleBar` | same. Native lights at window (18, 18) under a unified toolbar (also with the background hidden, the title removed or a split view), (11, 12) under `.unifiedCompact`, (8, 8) with no toolbar; safe-area top 52 / 40 / 32. The kit puts them at (18, 18) under `Unified` and, since this change, (11, 12) under `UnifiedCompact` (was 18, 18); the compact bar now leaves their width free. Inactive grays the native lights in both; corner 16, system shadow (`*-0-active`, `*-1-inactive`). Differs: a kit window with no `Titlebar` keeps (18, 18) (no app needs the bare strip), and the compact bar puts its first item right after the lights where Swift leaves 20 px (spacing shared by every compact bar, design B). Captures `~/Desktop/barrierkit-capturas/csd-c/`, harness `~/src/probes/fs/csd-states.sh` | same: frame = visible frame 0,30 2560x1325, lights keep their window offset, corner and shadow unchanged (`*-2-zoomed`); a double click on the kit bar zooms through AppKit | as A in Swift and kit: Left 0,30 1280x1325, Fill 0,30 2560x1325, lights keep their windowed offset, (8, 8) in Swift with no toolbar, (18, 18) in the kit; corner and shadow unchanged. The kit tiles from the Window menu of an `Application` and by an edge drag of a `run_window` window, which has no Window menu (`tile/swift-C-*`, `tile/kit-app-hidden-title-bar-*`, `tile/kit-notes-hidden-title-bar-*`, `csd-c/swift-*-4-tiled-fill`) | Swift: nothing (no toolbar) or the toolbar pinned with shift 52 / 40; at the edge it slides 30 px with the lights (rows 2, 4). Kit: mid-screen the bar stays at y 0 with no lights gap and no content shift. At the edge, before `907de855` the menu bar covered it and the lights showed at (18, 48) over blank content (`fs/kit-hidden-title-bar-2-fs-top-edge`); since then its glass controls ride beside the lights in AppKit's bar ([ADR 0116](../decisions.md#0116--in-full-screen-the-title-bar-row-of-a-hiddentitlebar-window-rides-with-appkits-bar)), whose style now follows `toolbar_style`. The content does not shift 52 / 40 as under Swift's pinned toolbar, by that ADR |
| D | `.windowStyle(.plain)`, borderless `NSWindow` | `WindowStyle::Plain` | kit lights drawn by the kit | zoom by the green light same as Swift; by title-bar double click does not zoom (not diagnosed) | Swift `.plain` never tiles: the Window menu and a drag do nothing (`tile/swift-D-*`). Kit tiles through Window > Fill, Center and Move & Resize of an `Application` with A's frames, keeping corner 16.8 and its 23 px shadow; not by a drag to an edge (the kit moves the window itself) nor from its lights; a `run_window` window has no Window menu (`tile/kit-app-plain-*`, `tile/kit-notes-plain-*`) | Swift `.plain` never enters; kit enters with one frame jump (2 bounds), hides menu bar and Dock for good |

Tiled is a ceiling: macOS 27.2 has no public tiled state. The SDK has
`isZoomed` and the full-screen tile behaviors only, the Window menu items
call private actions (`_zoomLeft:`, `_zoomFill:`), and AppKit posts only
`windowWillStartLiveResize`, `windowDidResize` and `windowDidEndLiveResize`.
Firefox needs a private API for the same state
([bug 2054999](https://bugzilla.mozilla.org/show_bug.cgi?id=2054999)). No
design looks different when tiled, so `WindowStatus` has no tiled field.
"Tiled windows have margins" is off on this Mac
(`EnableTiledWindowMargins = 0`); with margins on, nothing was measured.
Probe `~/src/probes/tile/`, captures `~/src/captures/tile/`.

Rows A and B: probe `~/src/probes/csd/` (`states.sh` drives one window
through every state, `greenmenu` presses the green button's menu,
`shape.py` reads corner and shadow, `swift-twins.swift` adds `.onHover` to the
`fs-swiftui` variants), captures `~/src/captures/csd-ab/`, display 2560×1440
at 1x, Dark. A SwiftUI `WindowGroup` with a toolbar opened no window at launch
from a fresh bundle; File > New Window opened it. Owed in the kit, none of them
a ceiling, since each has a public AppKit or SwiftUI counterpart: the leading
offset of the first toolbar item in the three bars, a 28 px item size for
`Expanded`, the gray title of a title row, the expanded toolbar row riding with
the menu bar (ADR 0116 carries only the first 52 px), and
`.windowToolbarFullScreenVisibility(.onHover)`, which the ADR 0116 session plans
through the focus overlay that already slides a bar in.

Rows 13 and 14 of [fullscreen-titlebars.md](fullscreen-titlebars.md)
(title-bar accessories) belong to design B; rows 15, 16 and 19 (kiosk, keep
the Dock, auto-hide the menu bar) to design D.

## 3. Rules every session follows

1. Build a Swift twin for the design and the kit window from the same recipe,
   and capture both in every state of section 1 (light, active and inactive).
   Use `~/src/probes/fs/` and its harness first; take
   `/tmp/barrierkit-screen.lock` (`mkdir`) for every capture.
2. Fill your row of section 2 with numbers (frame, content shift, corner,
   shadow, what the top edge reveals) and the capture names.
3. Implement in the kit what Swift does and the kit does not, with the
   SwiftUI name (glossary rule). A design Swift allows but no app needs stays
   out ("Nada por si acaso").
4. Record what cannot be reproduced without private API as a ceiling, with
   its cause.

## 4. Sessions

| Session | Designs | Files it owns |
| ------- | ------- | ------------- |
| `csd-titled-toolbar` | A and B | `titlebar.rs`, the `TitleBar` branch of `platform/macos` |
| `csd-hidden-titlebar` | C | the `HiddenTitleBar` branch of `platform/macos`, `chrome.rs` |
| `csd-plain` | D | the `Plain` branch of `platform/macos`, `traffic.rs` |
| `csd-tiling` | tiled column of A to D | `WindowStatus` if a public tiled signal exists |
