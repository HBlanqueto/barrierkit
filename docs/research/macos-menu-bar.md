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

The panel does not have one fixed colour. Measured at 2560x1440 under the same
wallpaper, the two appearances give two veil sets:

| Backdrop | Dark panel | Dark ink | Light panel | Light ink |
|---|---|---|---|---|
| White | `#a3a3a3` | dark | `#f2f2f2` | dark |
| Gray | `#92` to `#94` | dark | `#d1d1d1` | dark |
| Black | `#2b2b2b` | white | `#75` to `#7b` | dark |
| Checker | `#8b` to `#8d` | dark | `#9f` to `#c6` | dark |
| Split white/black | light top, dark bottom | switches at the split | light top, dark bottom | switches at the split |

The full matrix, including Reduce Transparency, is in
[`macos-blur.md`](macos-blur.md#3-the-measured-matrix-dark-mode) and
[`macos-blur.md`](macos-blur.md#4-the-measured-matrix-light-mode).

The bar is the opposite case: it stays clear and its ink follows the colour
scheme, not the window behind it. A white backdrop therefore gives white text
on white in dark mode. That is the measured macOS behaviour, not a rendering
bug in the probe.

The panel is **scheme-dependent**: over black it is `#2b2b2b` in dark mode but
`#75` to `#7b` in light mode. The earlier "scheme-independent" claim was
recorded under a dark wallpaper; see the corrections in
[`macos-blur.md`](macos-blur.md#9-corrections-to-the-2026-11-19-research).

**Re-read 2026-11-22.** The panel values above were read with `sample` and are shifted
([`macos-blur.md` section 9.1](macos-blur.md#91-2026-11-21-mid-tones-read-with-sample-were-too-light)).
Raw, in dark mode, the panel shows 146 over white, 129 over gray, 32 over black and 122 over the 8 px checker, with a
blur of 9.5 px plus a wide halo, a luma tone and a chroma gain: [section 14](macos-blur.md#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22). The bar
itself is identical to the backdrop to 0 levels over every backdrop of the series. Light mode was not measured.

The ink follows the panel in both modes: `ctappearance::menu::dark_ink_over`
takes dark ink above `MENU_DARK_INK_ABOVE` (0.4) and white below it. The
light panel over black is 0.46 luma and the dark panel over black is 0.17,
so the threshold works for both matrices.

Malaysia's menu glass has one veil, lift and cap for the light scheme and one
for the dark ones (`MENU_GLASS_VEIL_LIGHT` and `_DARK`, and likewise
`MENU_GLASS_LIFT_*` and `MENU_GLASS_CAP_*`). Over white, black and the checker
the dark set composites to `#a0`, `#2c` and `#87`, and the light set to `#f2`,
`#7a` and `#bf`; the tests in `ctappearance::resolve` pin the four backdrops in
both.

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
| Panel glass | Dark matrix `#a3`/`#2b`; light matrix `#f2`/`#75` | fixed scheme glass | dark adaptive tone only |
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
7. **Add the light veil set.** The dark tokens reproduce the dark matrix;
   a light scheme needs its own veil and cap so the panel renders `#f2` over
   white, `#d1` over gray and `#75` over black. The targets and model are in
   [`macos-blur.md`](macos-blur.md#5-the-model-fit-one-veil-per-surface-per-mode).

## 8. Open questions

- Does the panel use the same adaptive glass in light mode over a dark
  window?
- What is the panel radius at 2x?
- Does the app name use a heavier face than a bold menu title?
- Are the right status items spaced by a token or by each agent's view?
- Does the wide bar exist on macOS 27, and which setting enables it?

## 9. Implementation prompt

```text
You are fixing the menu bar and application menu glass from measured values.
Read this file and macos-blur.md first. Work in ~/src/barrierkit and
~/src/malaysia, main only: git pull --ff-only before editing and before
each push, one commit per session, English messages, no code comments.

Measured targets at 2560x1440, dark / light:
- Panel over white: #a3a3a3 / #f2f2f2.
- Panel over gray: #92-94 / #d1d1d1.
- Panel over black: #2b2b2b / #75-7b.
- Panel over checker: #8b-8d / #bf-c6.
- Ink: dark above 0.4 panel luma, white below; the light panel over black
  is 0.46, the dark panel over black is 0.17.
- Bar: clear in both modes, no blur, scheme ink only.

Tasks:
1. barrierkit: add the light veil and cap for the menu glass in
   crates/ctdesign/tengen/components.json and ctappearance, keeping the
   dark tokens unchanged. Four-backdrop tests per scheme in
   ctappearance::resolve asserting the eight targets within 8/255.
2. malaysia: after the barrierkit rev bump, verify
   malaysia-compositor/blur.rs and focus_notice.rs feed the resolved glass
   and the ink decision unchanged. The shell already follows the panel;
   do not add a wallpaper proxy: the split-backdrop measurement proves the
   ink follows the window behind the panel.
3. Keep the clear bar as measured; do not add a bar tint or blur.

Definition of done: ./dev.sh check passes in both repos; the light matrix
renders the four targets; barrierkit main is pushed and malaysia pins the
new rev. Report the changed tokens, the test values and the commits.
```
