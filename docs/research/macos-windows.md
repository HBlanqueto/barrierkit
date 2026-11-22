# macOS windows, title bars and traffic lights

**Goal:** describe the measurable window chrome that BarrierKit must match:
frame, content, title bar, traffic lights and semantic colours.
**Scope:** the window frame and its title bar. It excludes toolbars with
custom content, sheets, dialogs, popovers, tabs and full-screen transitions.
**Reader:** anyone who changes `ctappearance`, `ctdesign.window` or the
client-side decoration path.
**Input:** fresh `NSWindow` instances and screenshots of Settings.
**Output:** measured metrics, a Malaysia comparison and ordered gaps.

## Steps

1. Read the environment and limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the window styles table to pick a title bar height.
3. Check the traffic-light inset rule before changing `traffic::inset`.
4. Compare a proposed token with the last table.
5. Keep Apple artwork out of the repository.

---

## 1. Structure

A macOS window is a frame rectangle split into a title bar and a content
rectangle:

- The **frame** includes the title bar and any border.
- The **content** rectangle is what the app draws into.
- The **title bar** holds traffic lights, the title and, in a toolbar window,
  the toolbar.
- **Traffic lights** are three 14 pt discs: close, minimise and zoom. Their
  order is fixed. The zoom glyph is the full-screen arrows, or a plus in a
  window that cannot go full screen and while Option is held.

AppKit also exposes a unified compact style and a toolbar style. They are the
same chrome with different heights and insets.

## 2. Measured geometry

All values are points; on this host one point is one pixel. Frames use the
AppKit bottom-left origin. Insets in the table are from the top-left of the
frame.

| Style | Frame | Content | Title bar | Close button | Inset | Gap |
|---|---:|---:|---:|---|---:|---:|
| Standard (`style=15`) | 640x420 | 640x388 | 32 | `9,397 14x14` | 9 | 9 |
| Utility (`style=3`) | 400x300 | 400x268 | 32 | `9,277 14x14` | 9 | 9 |
| Unified compact | 640x420 | 640x380 | 40 | `12,393 14x14` | 12 | 9 |
| Toolbar | 640x420 | 640x354 | 66 | `19,387 14x14` | 19 | 9 |

Rules that follow from the four rows:

- The button diameter is **14** in every style.
- The gap between buttons is **9**.
- The standard and utility title bar is **32**.
- The compact title bar is **40**, and the lights move to **12** from the
  left.
- The toolbar title bar is **66**, and the lights move to **19** from the
  top-left. They are **not** vertically centred in a 66 pt bar; centring
  would put them at 26.
- The traffic-light group starts at the left inset, so the close button is
  the leftmost element.

The title bar font reported by AppKit is `titleBar`, **13 pt**. The semantic
window colours are:

| Role | Light | Dark |
|---|---|---|
| `windowBackground` | `#ffffff` | `#1e1e1e` |
| `underPageBackground` | `#f6f6f6` | `#282828` |
| `separator` | `#000000` at 10% | `#ffffff` at 12% |
| `windowFrameText` | `#000000` at 84.7% | `#ffffff` at 84.7% |
| `selectedContentBackground` | `#0064e1` | `#0059d1` |

A live Finder window sampled at 2560x1440 adds the chrome values the API does
not expose:

| Region | Dark | Light |
|---|---|---|
| Title bar | `#282828` | `#fafafa` |
| Toolbar | `#2f2f2f` | `#dbdbdb` |
| Sidebar | `#454f57` | `#efefef` |
| Content | `#282828` | `#f5f5f5` |
| Frame outline | 1 px dark plus 1 px light bevel | same structure |

Corner radius, measured on 2026-11-22 ([macOS corners](macos-corners.md)):
`NSView.effectiveCornerRadii` of the frame view is **16** on every corner of
every window kind (titled, toolbar styles, sidebar, utility, HUD, sheet,
tabbed, document, Finder), light and dark, active and inactive, and the alpha
of the window capture agrees with it. The shape is a continuous corner of
radius 16.8 (smoothing 0.745); full screen is square. The earlier 20, 26 and
15 for a compact toolbar, a unified toolbar and a utility panel were never
measured and are wrong.

## 3. Traffic-light glyphs

Measured on 2026-11-20 with `~/src/probes/lights.swift`; the whole look and the
method are in
[macOS control metrics](macos-control-metrics.md#traffic-lights-macos-27).

- The glyphs are drawn in the 14 pt disc itself, not in an 8 pt box.
- Close is two 2 pt diagonal strokes with round caps, 5 pt long on each axis.
- Minimise is a 7.5 × 2 pt bar with square ends.
- Zoom is two right triangles of 5 pt legs, corners rounded 0.3, in opposite
  corners of a 6 pt square (the full-screen arrows); in a full-screen window the
  triangles meet at the center instead (the exit glyph); the plus is two
  7.5 × 2 pt bars.
- The ring along the edge is 0.5 pt wide.
- Inactive windows keep the geometry and show translucent gray discs.

The glyph paths, the ring and the colors of every state are tokens:
`components.window.traffic-lights` in `crates/ctdesign/tengen/components.json`
and `traffic-lights` in each theme of `tokens.json`.

## 4. Behaviour

Observed or read from public APIs:

- Dragging the title bar moves the window.
- Double-clicking the title bar zooms or minimises according to the user
  setting. The double-click interval was not measured; the 400 ms and 4 pt
  values in `ctdesign` are project values.
- The traffic lights are separate controls. Hovering any of them shows the
  glyphs of all three; the fill does not change.
- A full-screen window hides the normal title bar and uses a full-screen
  glass band. That band was not captured.

The default `NSAnimationContext` duration on this host is **250 ms**. Window
open, close and zoom animations use it unless an app overrides it. No
frame-by-frame window animation was captured.

## 5. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Standard title bar | 32 | 32 (`titlebar-ssd-h`) | `tokens.size.titlebar-ssd-h` | Matches |
| Client title bar | 40 | 40 (`titlebar-h`) | `tokens.size.titlebar-h` | Matches |
| Toolbar title bar | 66 | No equivalent; tool group is 36 | `tokens.size.toolgroup-h` | Absent by decision: no consumer |
| Traffic diameter | 14 | 14 | `TITLEBAR.buttons.diameter` | Matches |
| Traffic gap | 9 | 9 | `TITLEBAR.buttons.spacing` | Matches |
| Traffic inset | 9 / 12 / 19 | 9 in the 32 bar, 12 in the 40 bar, centred vertically (9 and 13 from the top) | `ctappearance::traffic::inset`, `TITLEBAR.buttons.inset`, `inset_compact` | Matches 32 and 40; 19 not implemented |
| Traffic look | Discs with a gradient, a 0.5 ring, a halo and glyphs, per appearance | The same, from `TRAFFIC_LIGHTS_*` | `ctdesign` tokens `traffic-lights` | Matches within the limits in [macOS control metrics](macos-control-metrics.md#traffic-lights-macos-27) |
| Corner radius | 16 (API), silhouette 16.8, continuous, every window kind; full screen 0; alert 26 | 16 / 16 / 16 / 16 since 2026-11-22 (was 16 / 20 / 26 / 15) | `WINDOW_CORNER`, `CORNER_WINDOW` | Draw the silhouette radius `CORNER_WINDOW.fitted` ([macOS corners](macos-corners.md#3-windows-and-sheets)) |
| Window background | `#1e1e1e` dark | Theme `bg-window` | `ctdesign` tokens | Verify against this table |
| Title font | 13 pt | `TITLEBAR` font | `ctdesign` | Verify face and weight |

The inset rule is solved for the two bars BarrierKit has: `traffic::inset`
gives 9 and 12 from the left and the lights stay centred, which is also where
macOS puts them from the top in the 32 and 40 pt bars. It is false for the 66 pt
toolbar style, where macOS puts them 19 pt from the left and from the top.
BarrierKit has no such bar, so the 19 is kept here and not in a token.

## 6. What not to copy

- SF Pro or SF Compact font files.
- Apple's traffic-light glyph assets.
- Apple window screenshots or app icons.
- The exact Apple title strings or menu names.

Copy geometry, contrast ratios and behaviour, not artwork.

## 7. Prioritized changes

1. **Split the traffic inset from the bar height.** Done for 9 (32) and 12
   (40): `traffic::inset` returns the horizontal inset from
   `TITLEBAR.buttons.inset` and `inset_compact`, and the vertical position
   stays centred. The 19 for a 66 pt toolbar is not added (item 3).
2. **Measure the corner radius from a live window capture.** Done on
   2026-11-22 with `cornerwin` ([macOS corners](macos-corners.md)); the 20,
   26 and 15 of `WINDOW_CORNER` are now 16.
3. **Add a toolbar title bar style** with a 66 pt bar and a 19 pt light
   inset, or document that BarrierKit deliberately stops at the compact
   style. Decided: it stops at the compact style
   ([9. Title bars](../barrierkit/09-titlebars.md#traffic-lights)); the 19
   stays here.
4. **Verify the title font and window background** in both light and dark
   mode against the semantic table.
5. **Capture the full-screen band.** It is currently a token-only area and
   has no measured frame.

## 8. Open questions

- What is the live window corner radius at 2x? At 1x it is 16 (API) and 16.8 (silhouette); the first row of the corner is the open point ([macOS corners](macos-corners.md#8-not-measured-and-what-each-needs)).
- Does the toolbar style centre the title vertically or align it with the
  toolbar row?
- Do inactive windows dim the title and the lights by the same amount?
- Which animation curve does AppKit use for the 250 ms open and close?

## 9. Implementation prompt

```text
You are fixing the window chrome from measured values. Read this file
first. Work in ~/src/barrierkit and ~/src/malaysia, main only:
git pull --ff-only before editing and before each push, one commit per
session, English messages, no code comments.

Measured targets, dark / light:
- Title bar #282828 / #fafafa.
- Toolbar #2f2f2f / #dbdbdb.
- Sidebar #454f57 / #efefef.
- Content #282828 / #f5f5f5.
- Frame outline: 1 px dark plus 1 px light bevel, both modes.

Tasks:
1. barrierkit: verify the theme titlebar and window tokens against the
   table; fix only the ones that differ by more than 8/255. Do not change
   the matched geometry (32/40/66 bars, 14 buttons, gap 9).
2. barrierkit: add a test asserting the four chrome values per scheme.

Do not change the traffic-light inset rule in this pass; it has its own
prompt.

Definition of done: ./dev.sh check passes; tests assert the measured
values; main is pushed. Report changed tokens and commits.
```
