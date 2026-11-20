# macOS menu bar and application menus

**Goal:** describe the menu bar, the title-open state and the application menu
panel with measurements that can be ported to `ctdesign`.
**Scope:** the top bar, the left application menus, the right status area and
one open application menu. It excludes menu extras that are separate panels.
**Reader:** anyone who changes `malaysia-shell::menubar` or the menu tokens.
**Input:** accessibility frames, controlled-backdrop screenshots and a menu
recording.
**Output:** measured geometry, the adaptive glass model, a Malaysia comparison
and ordered gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the full default matrix in
   [`macos-defaults.md`](macos-defaults.md).
3. Read the menu row and the scheme-independence correction in
   [`macos-blur.md`](macos-blur.md).
4. Use the bar table for heights and item widths.
5. Use the glass table only with its backdrop; never lift one value alone.
6. Do not copy SF Pro, SF Symbols or Apple menu artwork.

---

## 1. Structure

The menu bar has three regions:

- **Left:** the Apple menu, the front app name and the app menus.
- **Right:** status items owned by system agents, including Control Center
  and the clock.
- **Panel:** a separate window below the bar. It contains items, separators,
  shortcuts, check marks and submenu chevrons.

The bar is **clear**: the desktop or window behind it shows through unchanged,
with no blur and no tint. The panel is a separate adaptive glass surface, not
a darker copy of the bar.

## 2. Default bar geometry

The bar is **30 pt** tall across the 1920 pt display. Accessibility frames for
the Spanish Finder menu, in screen coordinates:

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
- The right status area has no accessibility menu. A synthetic click at
  `(2365, 15)` opened Control Center, so the bar owns the hit area even when
  no AX element exists.

The wide variant was not observed. Malaysia has a 38 pt wide mode; macOS on
this host stayed at 30.

## 3. Default panel geometry

The Finder `Archivo` menu over a controlled white backdrop:

| Property | Value |
|---|---:|
| Panel left | 98 |
| Panel right | 414 |
| Panel width | 317 |
| Panel top | 31 |
| Panel bottom | 659 |
| Panel height | 629 |
| Item height | 24 |
| Separator band | 11 |
| Corner radius | about 6 |
| Font | 13 |

The panel starts one point below the bar. Its width is content-driven: 317 for
this File menu and 356 for the iTerm2 `Shell` menu. Item height is exactly 24.
A separator occupies 11 pt, a 1 pt line plus 5 pt above and below.

The selected first row sampled `#80bcff` and `#81bdff`; the exact value is
scheme- and backdrop-dependent. Re-sample it in both appearances before using
it as a token.

The open animation is **instant** at 60 fps: the panel appears between two
consecutive frames. There is no scale, fade or slide to copy.

## 4. Adaptive glass

The panel does not have one fixed dark colour. In dark mode it is light over a
light backdrop and dark over a dark backdrop:

| Backdrop | Panel | Ink |
|---|---|---|
| White | `#8d8d8d` | dark |
| Gray | `#8d8d8d` | dark |
| Black | `#2b2b2b` | white |
| Checker | about `#80` | dark |
| Split white/black | light top, dark bottom | switches at the split |

The full matrix, including Reduce Transparency and light mode, is in
[`macos-defaults.md`](macos-defaults.md#5-default-application-menu).

The bar is the opposite case: it stays clear and its ink follows the colour
scheme, not the window behind it. A white backdrop therefore gives white text
on white in dark mode. That is the measured macOS behaviour, not a rendering
bug in the probe.

The panel is scheme-independent too: over black it is `#2b2b2b` with white
ink even in light mode, and over white it is `#8d8d8d` with dark ink even in
dark mode. See the corrections in
[`macos-blur.md`](macos-blur.md#8-corrections-to-the-earlier-research).

Malaysia's menu glass is one veil and one tone for both colour schemes
(`MENU_GLASS_VEIL`, `MENU_GLASS_LIFT`, `MENU_GLASS_CAP`). Over white, black and
the checker it composites to `#8d8d8d`, `#2b2b2b` and `#878787`; the tests in
`ctappearance::resolve` pin the four backdrops. The ink follows the panel: the
compositor reports the panel luma of each popup
(`infinity_focus_surface_v1.glass`) and `theme::over_glass` takes dark ink
above `MENU_DARK_INK_ABOVE` and white below it.

## 5. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Bar height | 30 | 30 | 30 |
| Wide bar | Not observed | 38 | 38 |
| Font | 13 | 13 | 13 |
| App-name weight | Bold | 700 | 700 |
| Left margin | 10 | 10 | 10 |
| Bar style | Clear | `translucent` | `floating` |
| Bar blur | None except the open pill | `BarBlur::Bar` | selection only |
| Panel width | 317 for File | 220 to 640 | unchanged |
| Item height | 24 | 24 | 24 |
| Separator | 11 total | 1 + 2x5 | unchanged |
| Panel glass | Adaptive light/dark | fixed scheme glass | one adaptive tone |
| Panel ink | Follows panel | follows scheme | follows the panel |
| Panel radius | about 6 | 12 | needs review |
| Open animation | Instant | no menu animation | matches |

## 6. What not to copy

- SF Pro Text and the macOS menu font files.
- SF Symbols used by menu items.
- Apple menu items or their translations.
- The Apple logo glyph as an asset. Draw a project mark instead.
- Any screenshot of an Apple menu.

## 7. Prioritized changes

1. **Keep the clear bar.** The default is now `floating`: no bar fill and blur
   only under the open title pill. This matches the measured clear bar.
2. **Make the menu ink follow the panel.** Done: the compositor reports the
   panel luma and the kit picks dark or white ink from it.
3. **Keep the 24 pt item and 11 pt separator.** These already match.
4. **Use content-driven widths.** The 317 pt File menu and the 356 pt Shell
   menu are useful checks. Keep the 220 pt minimum.
5. **Review the panel radius.** The measured corner is about 6, below the
   current 12. Confirm at 2x before changing the token.
6. **Add a menu-bar contrast test over white, gray and black.** The clear bar
   with scheme ink must be accepted as measured, not "fixed" by adding tint.

## 8. Open questions

- Does the panel use the same adaptive glass in light mode over a dark
  window?
- What is the panel radius at 2x?
- Does the app name use a heavier face than a bold menu title?
- Are the right status items spaced by a token or by each agent's view?
- Does the wide bar exist on macOS 27, and which setting enables it?
