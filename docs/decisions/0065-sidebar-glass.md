# 0065 — The sidebar is a glass panel

Status: accepted (2026-11-13). Nothing is implemented yet; the order is in «Plan».

## Goal and scope

This ADR fixes what the kit does so that a sidebar looks like the glass of
macOS Tahoe: which colors it computes, what it paints, what it sends to the
compositor and what it does when the compositor cannot draw the glass. It
starts when `Sidebar` is laid out in a `SplitView` and ends when the kit has
sent its panel or painted its solid fill. The material itself, its rim and its
states are drawn by the compositor, and are in
[Malaysia ADR 0078](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions/0078-sidebar-glass-panels.md).
Readers: whoever changes `Sidebar`, `SplitView`, `ctdesign` or `ctappearance`.

## Context

1. `Sidebar` paints `Resolved.sidebar.bg` as an opaque fill. Today the
   tokens `bg-sunken` and `sidebar-island-bg` have alpha 1, so the translucent
   and the opaque variants are the same color
   ([0038 in Malaysia](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisions/0038-reduce-transparency-by-default.md)).
2. The wallpaper already tints the sidebar. When `tint_parts.sidebars` is on,
   `Resolved` moves `sidebar.bg` toward the wallpaper's dominant color with
   `take_chroma`, by `wallpaper-tinting` times `WALLPAPER_TINTING_SIDEBAR`
   (`ctappearance::backdrop`, `resolve.rs`).
3. Over a `Backdrop` the sidebar is clear or tinted
   ([0057](0057-glass-and-sidebar-over-backdrop.md)).
4. The kit reaches the compositor through `infinity_window_style_v1` for the
   window's bar and, with the `shell` feature, for card frames. A plain blur
   region (`BlurRegion`) exists for the Dock and the lock screen.
5. A capture of macOS shows that the sidebar keeps the lightness of the app's
   solid layer and takes about 0.4 of the wallpaper's chroma, and that its rim
   is a white line, like the inner border of a window, that takes more of the
   wallpaper's chroma than the fill does. Only the compositor sees the
   wallpaper, so it draws the material and the rim (Malaysia ADR 0078).

## Decision

1. **No public API.** `Sidebar` and `SplitView` do it. As in AppKit, where the
   glass of a sidebar is automatic, an app has nothing to call. An app that
   needs to change the material gets a field in `SidebarStyle`
   ([0064](0064-styles-and-themes.md)) when it asks for one.
2. **The kit computes two colors** and sends them with the panel:
   1. `base`, a neutral solid with the app's lightness. It is the new token
      `sidebar-glass-base`: `#1e1e1f` in dark mode, measured, and `#f4f4f6` in
      light mode, an estimate.
   2. `solid`, `base` with the wallpaper's tint: `take_chroma(base, tint,
      factor × WALLPAPER_TINTING_SIDEBAR)`, the same path as today's tint. With
      `tint_parts.sidebars` off, `solid` is `base`.
3. **What the kit paints, by case:**

   | Case | The panel | Sends |
   |---|---|---|
   | The compositor offers `infinity_window_style_v1` version 6, and transparency is allowed | Transparent. The rows go on top | `base`, `solid`, the corner and the rectangles |
   | No such compositor | `solid`, opaque, with the fixed rim `sidebar-island-rim` | Nothing |
   | `reduce_transparency` or `contrast` | `Resolved.sidebar.bg`, the neutral opaque tone, with no wallpaper tint and the fixed rim | An empty panel list |

   While the window is not focused the kit does nothing extra: the compositor
   flattens the panel and removes the island's rim (Malaysia ADR 0078).
4. **The window has a hole under the panel.** The window background is not
   painted where the panel is, so what the compositor draws shows through. The
   ring around an island stays `bg-window`, the color of the detail pane.
5. **Island and flat.** An island panel is the rounded rectangle inset by
   `SIDEBAR.inset`, with `SIDEBAR.radius`. A flat panel is the whole column with
   radius 0; the compositor clips it to the window's corner. The flat sidebar's
   1 px line is painted by the kit in every case.
6. **When it sends.** On the first layout and whenever the width, the visibility,
   the narrow-window overlay or the window size change. It does not send the same
   panel twice. A hidden sidebar sends an empty list.
7. **Tokens.** `sidebar-glass-base` goes in `tokens.json`. The weights the
   compositor uses (`chroma` and `rim_chroma`) go in `components.window.sidebar.glass` and come
   out as `ctdesign::GLASS`.
   Light and dark share them.
8. **Over a `Backdrop`** the sidebar stays as 0057 says. It sends no panel.
9. **Plain blur is unchanged.** `BlurRegion` stays for the Dock and the lock.

## Plan

1. **Protocol.** Version 6 of `infinity-window-style-v1.xml`, and `bk` binds
   `1..=6`. The request is sent but nothing draws it.
2. **Tokens and colors.** `sidebar-glass-base`, `GLASS`, and `base` and `solid`
   in `ctappearance`. Tests for `solid` with the tint on and off.
3. **Publishing.** `Sidebar` and `SplitView` send the panel, the window gets
   its hole, and the three cases of step 3 paint.
4. **Documentation.** Chapter 10 (sidebar), chapter 4 (tokens) and the
   glossary, in the commit of each step.

Malaysia's compositor phases follow in its ADR 0078; its `rev` of barrierkit
moves after step 1.

## Consequences

- With a compatible compositor, the sidebar's lightness stays solid and only
  its color comes from the wallpaper.
- Without one, the sidebar is a solid panel with the wallpaper's tint, as
  before but with a lighter base in dark mode (`#1e1e1f` instead of `#141414`).
- Every `Sidebar` in a `SplitView` sends requests while it resizes.
- The kit holds two more colors per theme.

## Rejected alternatives

- **A translucent fill and a plain blur region.** It lets the wallpaper's
  lightness through, and a region cannot say a corner.
- **A public switch for the glass.** Nobody asks for it today.
- **The kit paints the rim in the glass case.** The compositor draws it with
  the window frame's inner border, so both share one line.
- **Tinting the neutral solid in «Reduce transparency».** A solid sidebar stays
  neutral, focused or not.
