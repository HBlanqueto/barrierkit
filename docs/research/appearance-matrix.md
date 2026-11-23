# Appearance matrix: light and dark, with blur and without

**Goal:** know how each surface looks in light and dark, with the system's blur and with Reduce Transparency on, and how far it is from SwiftUI or AppKit.
**Scope:** macOS arm64, the kit's examples (`agents/recipes`) against probes in `~/src/probes`, measured 2026-11-22 at scale 1 on a 2560 x 1440 display.
**Reader:** whoever changes a material, a glass, a title bar or a popup in `platform/macos/`.
**Input:** a Mac in an unlocked session, `cargo build -p bk --examples`, `~/src/probes`, numpy and Pillow.
**Output:** the share of pixels that differ per element, in each cell, and the ceiling each one is held to.

## Steps

1. Light and dark are per process: the probe sets `NSAppearance`, the kit reads `appearance.domain` from a temporary `MALAYSIA_CONFIG_DIR`. No system setting changes.
2. Blur is the system's default. Reduce Transparency is the system's switch: the script reads it, refuses to run when it differs from `--transparency`, and never changes it. The user turns it on and off in System Settings > Accessibility > Display.
3. Run `scripts/appearance-matrix.py --transparency blur`. It takes `/tmp/barrierkit-screen.lock`, opens each probe and each example as an app bundle, captures their windows (`screencapture -l`, through the sRGB profile) and prints the share of pixels that differ by more than 8 levels, per element. A window that is not key (its red light is gray) is captured again.
4. The five twins (notes, tasks, form, controls, standalone) are the bench `crates/bk/tests/kit/swiftui_twin.rs`: references in `crates/bk/tests/reference/swiftui-<twin>-<cell>.png`, cells `light`, `dark`, `light-rt`, `dark-rt` (a cell without a reference is skipped). `scripts/twin-reference.sh <twin> <light|dark> <out.png>` captures a reference.
5. With Reduce Transparency on: `scripts/appearance-matrix.py --transparency rt --references --live-compare DIR`, after `--live-capture DIR` was started before the flip (it waits for the user, captures the same running examples again, and the batch compares them with fresh ones).

## Cells

| Surface | Probe | Example | Elements |
|---|---|---|---|
| Notes, title bar drawn by the kit (`Plain`) | `csdtwin` | `notes_sidebar plain` | lights, sidebar toggle and rows, title, toolbar group, edges, empty state |
| Notes, native lights (`HiddenTitleBar`) | `csdtwin` | `notes_sidebar hidden-title-bar` | the same |
| Toolbar glass over scrolling rows | `glasstwin` | `glass_toolbar` | toolbar group, title bar veil, rows |
| `GlassContainer`, regular and clear | `glasscontainer` | `glass_container` | the two pairs, rows |
| Popover and its arrow | `matrixpopups popover` | `popups` | arrow, body |
| Sheet and its veil | `matrixpopups sheet` | `popups sheet` | sheet, veil |
| Notes, tasks, form, controls, standalone | `csdtwin`, `listtwin`, `formtwin`, `controlstwin`, `standalone` | the bench, `TestingRunner` | per element, the ceilings of `swiftui_twin.rs` |

## Blur, light and dark (2026-11-22)

| Surface | Element | Light | Dark | Ceiling | Cause of what is left |
|---|---|---|---|---|---|
| Notes, both frames | lights (`Plain`) | 8.3 | 10.9 | 12 | the kit draws its own lights; with native lights (`HiddenTitleBar`) 0.0 |
| | sidebar toggle | 23.5 | 20.5 | 26 | the toggle has no ring; the glass of SwiftUI's capsule |
| | sidebar rows | 7.5 | 7.7 | 10 | icon badges and glyph weight |
| | title | 9.0 | 8.6 | 11 | glyph weight (SF Pro against Inter) |
| | toolbar group | 13.9 | 13.9 | 16 | glass lens and rim |
| | backgrounds and the edge under the title bar | 0.0 | 0.0 | 1 | |
| | empty state | 3.0 | 3.0 | 5 | |
| Toolbar glass | toolbar group | 64 | 99 | 100 | see below |
| | title bar veil | 99 | 99 | 100 | see below |
| | rows | 10.1 | 10.1 | 12 | SwiftUI draws a scroller the kit does not |
| `GlassContainer` | regular pair | 30 | 27 | 32 | the glyph ink of `IconButton` and the lens highlight; varies between runs with the key state |
| | clear pair | 16 | 28 | 30 | the same |
| Popover | arrow | 25 | 60 | 62 | NSPopover is Liquid Glass on macOS 26; see below |
| | body | 4 | 86 | 88 | the same |
| Sheet | sheet and veil | 0.0 | 0.0 | 2 | |

The bench for the five twins holds its per-element ceilings in `swiftui_twin.rs`; the blur cells are its `light` and `dark`.

## What the numbers say

- **Sheet veil.** AppKit already veils the window under `beginSheet`: the kit drew its own scrim over it, so a light parent was 184 where AppKit's is 204. The kit now draws no scrim when the platform veils (`Host::native_sheets`, `NativeSheets`). Sheet and veil are 0.0% in light and dark.
- **Toolbar glass.** SwiftUI's scroll-edge effect under a toolbar is hard and as deep as the bar (52 px, an 8 px blend); `TopGlass` is the full-screen bar glass of Malaysia, soft and 96 px deep (`FULLSCREEN_GLASS`). A window-toolbar variant of `TopGlass` (hard edge, depth of the title bar) is not built. In dark the kit's veil is also lighter (143 against 116 over the red row). Held at a ceiling until then.
- **Popover.** The popover of macOS 26 is Liquid Glass; `NSGlassEffectView` takes no shape but a corner radius, so the arrow cannot be given to it. The kit draws its own fill under the arrow-shaped material: white in light (matches, 4%), dark `popover_style().bg` (26) against NSPopover's 56 over a 30 backdrop. A native material under a clear kit fill was tried: it comes out gray (222 in light, 94 in dark) and is worse.
- **GlassContainer.** With the media `RoundButton` the glass was doubled (the button paints its own); `IconButton` is the SwiftUI twin of the probe. What is left is the ink and the lens.

## Not in the matrix

| Surface | Why |
|---|---|
| Sidebar vibrancy, any window that shows what lies behind it | The material blurs the screen behind the window, and the Mac in use keeps an always-on-top app there: a backdrop window cannot go behind the compared windows. The material is the same `NSVisualEffectView` on both sides by construction |
| Menus | The kit's menus on macOS are the Dock-style `MenuView`, not `NSMenu`: its content is the bench `menu` (`appkit-compare.py menu`) and its shape the popover's `Menu` shape |
| Tooltips | They need a hover; `~/src/probes/tooltip.sh` does it and the kit's `.help` has no equivalent driver yet |
| Title bar `TitleBar` (SSD) | The native bar is taller than the twin's full-size content: the sizes differ |
| Tasks, form and controls as real windows | They are the bench; a real window and the `TestingRunner` agree on the notes twin to the pixel count |
