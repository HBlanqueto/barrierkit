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
| A | `WindowGroup`, no toolbar (system bar) | `WindowStyle::TitleBar` | measured, same ([macos-windows.md](macos-windows.md)) | animation same, 21 fr / 333 ms; look owed | owed | same as Swift rows 1 and 2 of [fullscreen-titlebars.md](fullscreen-titlebars.md); the kit `Titlebar` disappears |
| B | `.toolbar{}` with `.windowToolbarStyle(.unified)`, `.unifiedCompact`, `.expanded` | `Titlebar::toolbar_style` (`Unified`, `UnifiedCompact`; no `Expanded`) | kit variants in [09](../barrierkit/09-titlebars.md#variants-in-a-normal-window); Swift twin owed | owed | owed | Swift pins the toolbar and shifts the content 52 / 40 / 68 px (rows 3 to 8); `.autoHideToolbar` hides it (row 11); kit has neither |
| C | `.windowStyle(.hiddenTitleBar)`, AppKit `titlebarAppearsTransparent` + `.fullSizeContentView` | `WindowStyle::HiddenTitleBar` | measured, kit bar beside native lights | animation same; look owed | owed | lights at the top edge as in Swift; the kit bar's place in full screen is being changed by the running session `kit-csd-drag` |
| D | `.windowStyle(.plain)`, borderless `NSWindow` | `WindowStyle::Plain` | kit lights drawn by the kit | zoom by the green light same as Swift; by title-bar double click does not zoom (not diagnosed) | owed: no Move & Resize menu on a kit light | Swift `.plain` never enters; kit enters with one frame jump (2 bounds), hides menu bar and Dock for good |

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
