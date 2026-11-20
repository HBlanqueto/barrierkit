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
- **Traffic lights** are three 14 pt rounded squares: close, minimise and
  zoom. Their order is fixed. The zoom button becomes a plus in some
  contexts.

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

Corner radius was not exposed by a public API and was not measured from a
screenshot in this pass. The live screenshots suggest a radius in the low
teens, but it is **estimated**, not measured. Do not treat the Malaysia
16/20/26 values as confirmed macOS values.

## 3. Traffic-light glyphs

- The glyph box is **8x8** inside the 14 pt button.
- Close is two 1.5 pt diagonal strokes.
- Minimise is one 1.5 pt horizontal stroke.
- Zoom is two triangles in the usual diagonal arrangement.
- The group uses a 0.5 pt rim in the live style.
- Inactive windows keep the same geometry and lose the colour.

The glyph path, fill and rim are tokens in
`crates/ctdesign/tengen/components.json` under
`components.window.traffic-lights`. The exact macOS colours are not in
`metrics.txt`; they were not sampled in this pass.

## 4. Behaviour

Observed or read from public APIs:

- Dragging the title bar moves the window.
- Double-clicking the title bar zooms or minimises according to the user
  setting. The double-click interval was not measured; the 400 ms and 4 pt
  values in `ctdesign` are project values.
- The traffic lights are separate controls. Group hover behaviour was not
  measured.
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
| Toolbar title bar | 66 | No equivalent; tool group is 36 | `tokens.size.toolgroup-h` | Tall toolbar style absent |
| Traffic diameter | 14 | 14 | `TITLEBAR.buttons.diameter` | Matches |
| Traffic gap | 9 | 9 | `TITLEBAR.buttons.spacing` | Matches |
| Traffic inset | 9 / 12 / 19 | Centred: `(bar - 14) / 2` | `ctappearance::traffic::inset` | Tall bars place lights too low |
| Corner radius | Not measured; low teens | 16 / 20 / 26 / 15 | `WINDOW_CORNER` | Needs measurement |
| Window background | `#1e1e1e` dark | Theme `bg-window` | `ctdesign` tokens | Verify against this table |
| Title font | 13 pt | `TITLEBAR` font | `ctdesign` | Verify face and weight |

The strongest mismatch is the inset rule. `traffic::inset` assumes the lights
are vertically centred. That is true for 32 and 40, but false for the 66 pt
toolbar style, where macOS puts them 19 pt from the top.

## 6. What not to copy

- SF Pro or SF Compact font files.
- Apple's traffic-light glyph assets.
- Apple window screenshots or app icons.
- The exact Apple title strings or menu names.

Copy geometry, contrast ratios and behaviour, not artwork.

## 7. Prioritized changes

1. **Split the traffic inset from the bar height.** Add a top inset that can
   differ from the horizontal one, and use 9 for 32, 12 for 40 and 19 for a
   66 pt toolbar. This is a small `ctappearance` change with a visible
   effect in tall windows.
2. **Measure the corner radius from a live window capture.** Use the
   existing `sample` probe on a high-contrast window edge. Only then compare
   it with `WINDOW_CORNER`.
3. **Add a toolbar title bar style** with a 66 pt bar and a 19 pt light
   inset, or document that BarrierKit deliberately stops at the compact
   style.
4. **Verify the title font and window background** in both light and dark
   mode against the semantic table.
5. **Capture the full-screen band.** It is currently a token-only area and
   has no measured frame.

## 8. Open questions

- What is the live window corner radius at 1x and at 2x?
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
