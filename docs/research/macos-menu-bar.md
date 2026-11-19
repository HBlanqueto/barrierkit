# macOS menu bar and application menus

**Goal:** describe the menu bar, the title-open state and the application menu
panel with measurements that can be ported to `ctdesign`.
**Scope:** the top bar, the left application menus, the right status area and
one open application menu. It excludes menu extras that are separate panels.
**Reader:** anyone who changes `malaysia-shell::menubar` or the menu tokens.
**Input:** accessibility frames, controlled-backdrop screenshots and a 60 fps
menu recording.
**Output:** measured geometry, glass values, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the bar table for heights and item widths.
3. Use the glass table only with its backdrop; never lift one value alone.
4. Check the comparison table before changing a menu token.
5. Do not copy SF Pro, SF Symbols or Apple menu artwork.

---

## 1. Structure

The menu bar has three regions:

- **Left:** the Apple menu, the front app name and the app menus.
- **Right:** status items owned by system agents, including Control Center
  and the clock.
- **Panel:** a separate window below the bar. It contains items, separators,
  shortcuts, check marks and submenu chevrons.

The bar itself is not a material. On this host it is **clear**: the desktop
or window behind it shows through unchanged, with no blur and no tint. The
panel is a strong dark glass.

## 2. Measured bar geometry

The bar is **30 pt** tall across the whole 2560 pt width. Accessibility
frames for the left items, in screen coordinates:

| Item | x | Width | Height |
|---|---:|---:|---:|
| Apple | 10 | 34 | 30 |
| Finder | 43 | 62 | 30 |
| Archivo | 104 | 67 | 30 |
| Edición | 170 | 65 | 30 |
| Visualización | 234 | 100 | 30 |
| Ir | 333 | 29 | 30 |
| Ventana | 361 | 70 | 30 |
| Ayuda | 430 | 59 | 30 |

Observations:

- The left margin is **10**.
- Each title is centred in a hit area that reaches the full bar height.
- Widths grow with the Spanish labels. They are not fixed tokens.
- The app name (`Finder`) is bold; the menu titles are regular.
- The font is **13 pt**, matching `NSFont.menuBarFont`.
- The right status area did not expose an accessibility menu. A synthetic
  click at `(2365, 15)` opened Control Center, which confirms that the bar
  owns the hit area even when no AX element is present.

The bar's **wide** variant was not observed. Malaysia has a 38 pt wide mode;
macOS on this host stayed at 30.

## 3. Measured panel geometry

The `Archivo` menu was opened over a controlled backdrop:

| Property | Value |
|---|---:|
| Panel left | 99 |
| Panel right | 412 |
| Panel width | 314 |
| Panel top | 31 |
| Panel bottom | about 644 |
| Item height | 24 |
| Separator band | 11 |
| Font | 13 |

The panel starts one point below the 30 pt bar. Its width is content-driven:
314 for this File menu, not a fixed panel width. Item height is exactly 24.
A separator occupies 11 pt, which is consistent with a 1 pt line plus 5 pt
above and below.

The selected first row sampled `#80bcff` and `#81bdff` in the capture. That
is much lighter than a solid accent fill, so the panel is blending the
selection with the glass or the capture was taken in a light appearance.
Treat the exact value as scheme-dependent and re-sample in both appearances.

The open animation is **instant** at 60 fps: the panel appears between two
consecutive frames. There is no scale, fade or slide to copy.

## 4. Measured glass

The bar and the panel behave differently over the same backdrops:

| Backdrop | Bar sample | Panel sample |
|---|---|---|
| White | `#ffffff` | `#656565` |
| Gray | `#a3a3a3` | `#606161` |
| Black | `#000000` | `#2c2d2d` |
| Checker | Sharp checker | Uniform `#5c5c5c` |

Conclusions:

- The **bar is transparent**. White stays white, black stays black and the
  checker keeps its edges.
- The **panel blurs heavily**. The checker loses its edges and collapses to
  a single gray.
- The panel is a **dark glass** in this dark-mode session: it lifts black to
  `#2c2d2d` and pulls white down to `#656565`.
- The panel is not a simple opacity layer. The four results do not lie on
  one straight line from black to white, which is what a pure tint would
  produce. A blur plus a light tint plus a rim is the smallest model that
  fits.

Malaysia's current menu model is a raised surface plus an optional
`menu-glass` overlay. Its light value is `#f5f5f77a`, its dark value is
`#ffffff21`. A 13% white overlay over black lands near `#212121`, which is
close to the measured `#2c2d2d`; the dark end is plausible. The hard case is
the bright end: the model must cap a bright backdrop down to `#656565`
without turning the panel into a flat white sheet.

## 5. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Bar height | 30 | 30 | `MENUBAR_HEIGHT` | Matches |
| Wide bar | Not observed | 38 | `MENUBAR_HEIGHT_WIDE` | Unverified |
| Font | 13 | 13 | `MENUBAR_FONT_SIZE` | Matches |
| App-name weight | Bold | 700 | `menubar.app-name.weight` | Matches |
| Left margin | 10 | 10 | `MENUBAR_MARGIN` | Matches |
| Bar background | Clear, adaptive ink | `shell-bar` `#f2f4f7e6` / `#1e1e1ee6` | `themes.*.shell-bar` | Bar is tinted and can blur |
| Bar blur | None | Optional `BarBlur` | `menubar/bar.rs` | Remove for the default style |
| Panel width | 314 for File | 220 to 640 | `shell.menu.panel` | Range is fine |
| Item height | 24 | 24 | `shell.menu.item.height` | Matches |
| Separator | 11 total | 1 + 2x5 | `shell.menu.separator` | Matches |
| Panel glass | Dark glass, strong blur | Raised surface plus overlay | `shell.menu.glass` | Wrong base model |
| Selected row | `#80bcff` sampled | Accent `#2f6fe4` | `accent` | Too saturated and dark |
| Open animation | Instant | No menu animation token | — | Matches by default |

## 6. What not to copy

- SF Pro Text and the macOS menu font files.
- SF Symbols used by menu items.
- Apple menu items or their translations.
- The Apple logo glyph as an asset. Draw a project mark instead.
- Any screenshot of an Apple menu.

## 7. Prioritized changes

1. **Make the default bar clear.** The measured bar has no tint and no blur.
   Keep `shell-bar` only for the opaque or accessibility fallback. This is
   the largest visual difference and it touches every screen.
2. **Rebuild the menu glass as a dark glass.** The panel needs a blurred
   backdrop, a dark tint that lifts black to about `#2c2d2d`, a lighter tint
   over white, and a rim. A single neutral overlay cannot match all four
   backdrop samples.
3. **Keep the 24 pt item and 11 pt separator.** These already match; do not
   change them while fixing the surface.
4. **Use content-driven menu widths.** The 314 pt File menu is a useful
   check. Keep the 220 pt minimum, but do not force a default width.
5. **Re-sample the selected row in light and dark.** The `#80bcff` value is
   not a usable token until the scheme is known.
6. **Add a menu-bar contrast test over white, gray and black.** The bar must
   keep readable ink on every backdrop because it has no material.

## 8. Open questions

- What is the panel corner radius? It was not sampled.
- Does the panel use a different glass in light mode?
- Does the app name use the same weight as a bold menu title or a heavier
  face?
- Are the right status items spaced by a token or by each agent's view?
- Does the wide bar exist on macOS 27, and if so, which setting enables it?
