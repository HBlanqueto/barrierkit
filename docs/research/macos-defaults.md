# macOS default desktop appearance

**Goal:** reset the live macOS appearance to its default geometry, measure the
default Dock, top bar and menu glass, and compare them with the Malaysia and
BarrierKit defaults.
**Scope:** Dock geometry and glass, menu-bar geometry and ink, application
menu geometry and glass, and Reduce Transparency. It excludes application
content and window chrome.
**Reader:** anyone who changes `ctdesign` Dock or menu tokens, `ctappearance`
glass or the Malaysia shell defaults.
**Input:** a backed-up macOS session at 1920x1080, controlled backdrops and
the switches for light/dark and Reduce Transparency.
**Output:** the measured matrix, the corrections applied to the repositories
and the remaining gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the reset procedure before measuring.
3. Read the Dock table and the menu table; do not mix their backdrops.
4. Compare a token with the "before" and "after" columns.
5. Read the per-element blur matrix and the corrections in
   [`macos-blur.md`](macos-blur.md) before trusting a glass value.
6. Check the remaining gaps before claiming the desktop is identical.

---

## 1. Reset and restore

The current display changed from 2560x1440 to **1920x1080 at 180 Hz** between
research passes. All values in this file are at 1920x1080, scale 1.0.

The appearance was backed up before the reset:

```sh
mkdir -p ~/src/captures/defaults-2026-11-19
defaults export com.apple.dock ~/src/captures/defaults-2026-11-19/com.apple.dock.plist
defaults export com.apple.universalaccess ~/src/captures/defaults-2026-11-19/com.apple.universalaccess.plist
defaults export NSGlobalDomain ~/src/captures/defaults-2026-11-19/NSGlobalDomain.plist
```

The Dock appearance keys were deleted, not the pinned applications:

```sh
for key in tilesize largesize magnification autohide orientation mineffect \
  show-recents minimize-to-application autohide-delay autohide-time-modifier; do
  defaults delete com.apple.dock "$key" 2>/dev/null || true
done
killall Dock
```

After the measurements the original Dock preferences were restored with
`defaults import`. The Reduce Transparency and light/dark switches were
returned to their original values (dark, transparency allowed).

The default Dock on this host is therefore the macOS default with the user's
pinned applications, not a factory application set.

## 2. Default Dock geometry

| Property | Measured |
|---|---:|
| Visible tile | 64 |
| Slot | 68x84 |
| Slot spacing | 68 |
| Visible icon body | 52x52 |
| Panel left | 172 |
| Panel right | 1747 |
| Panel width | 1576 |
| Panel top | 991 |
| Panel bottom | 1074 |
| Panel height | 84 |
| Panel radius | about 22 |
| Panel edge margin | 6 |
| Panel inner padding | about 9 |
| Icon body inset | 8 horizontal, 16 vertical |

The slot is wider than the icon body and taller than it. The extra space is
the Dock's hit area and the label/indicator room, not icon artwork.

The default app set on this host has 22 tiles and 2 separators. That is why
the panel is 1576 wide; the Dock is content-sized, not screen-wide.

## 3. Default Dock glass

The panel was sampled in the gap between two app icons, away from the rim.
The four backdrops are the controlled white, gray, black and checker windows.

| State | White | Gray | Black | Checker |
|---|---|---|---|---|
| Dark, transparency allowed | `#bebebe` | `#8a8a8a` | `#0d0d0d` | `#7f7f7f` |
| Dark, Reduce Transparency | `#020202` | not measured | `#0a0a0a` | not measured |
| Light, transparency allowed | `#eaeaea` | `#a5a5a5` | `#323232` | `#979797` |
| Light, Reduce Transparency | `#f1f1f1` | not measured | `#888888` | not measured |

The Dock does not have one fixed colour. Even with Reduce Transparency on it
still follows the backdrop a little. A single opaque token cannot describe
both the white and the black case.

The light Dock rim over the controlled backdrops:

| Backdrop | Edge sequence |
|---|---|
| White | `#e4e4e4`, `#f0f0f0`, `#eaeaea` |
| Gray | `#939393`, `#ababab`, `#a5a5a5` |
| Black | `#101010`, `#3b3b3b`, `#323232` |

The rim is a three-step light/dark bevel, not a one-pixel black outline. In
dark mode over gray the same edge reads `#919191`, `#808080`, `#919191`.

## 4. Default menu bar

The bar is **30 pt** high. The left margin is **10**, and the font is **13**.
The default item frames, with the Spanish Finder menu, are:

| Item | x | Width |
|---|---:|---:|
| Apple | 10 | 34 |
| Finder | 43 | 62 |
| Archivo | 104 | 67 |
| Edición | 170 | 65 |
| Visualización | 234 | 100 |
| Ir | 333 | 29 |
| Ventana | 361 | 70 |
| Ayuda | 430 | 59 |

The bar is **clear** in dark mode over white, gray and black. A white
backdrop stays white, a black backdrop stays black, and a checker stays
sharp. In dark mode the ink stays white; light mode uses the theme's dark
ink. The bar does not switch ink for the window behind it.

The wall paper still feeds the bar's selection and translucent styles. The
default measured state is a clear bar, so the default style is `floating`.

## 5. Default application menu

The Finder `Archivo` menu was opened over the controlled backdrops:

| Property | Measured |
|---|---:|
| Left | 98 |
| Right | 414 |
| Width | 317 |
| Top | 31 |
| Bottom | 659 |
| Height | 629 |
| Item height | 24 |
| Separator band | 11 |
| Corner radius | about 6 |
| Panel edge | 1 px |

The iTerm2 `Shell` menu is 356 wide. The width is content-driven.

The menu glass is **adaptive**. In dark mode it is light over a light
backdrop and dark over a dark backdrop:

| State | White | Gray | Black | Checker |
|---|---|---|---|---|
| Dark, transparency allowed | `#8d8d8d` | `#8d8d8d` | `#2b2b2b` | about `#80` |
| Dark, Reduce Transparency | `#8d8d8d` | not measured | `#2b2b2b` | not measured |
| Light, transparency allowed | not measured | not measured | `#9a9a9a` | not measured |
| Light, Reduce Transparency | `#fefefe` | not measured | `#9a9a9a` | not measured |

The ink follows the panel: dark text on the light panel, white text on the
dark panel. A split backdrop with white on top and black below switched the
panel and the ink at the split line. A checker produced a light panel with
dark text and a faint dither.

The measured menu is therefore not the same material as the menu bar. The bar
is clear and scheme-inked; the menu is adaptive and backdrop-inked.

## 6. Defaults and the repositories

The values below were the defaults before this pass and the corrections
applied in the same commit.

| Element | macOS default | Malaysia and BarrierKit before | After |
|---|---:|---|---|
| Dock tile | 64 | 48 | 64 |
| Dock large size | 64 | 64 | 64 |
| Dock edge margin | 6 | 4 | 6 |
| Dock padding base | about 9 | 4 plus 4.44% of tile | 6 plus 4.44% of tile |
| Dock slot gap | 4 | none | 4 |
| Dock vertical padding | 10 at 64 | derived from the same padding | 15.625% of the tile |
| Dock light glass | `#eaeaea` over white | `#ffffff66` | `#b5b5b548` |
| Dock light opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dock dark glass | `#0d0d0d` over black | `#2727274e` | unchanged |
| Menu bar style | clear | `translucent` | `floating` |
| Menu bar height | 30 | 30 | 30 |
| Menu bar margin | 10 | 10 | 10 |
| Menu bar font | 13 | 13 | 13 |
| Menu item height | 24 | 24 | 24 |
| Menu separator band | 11 | 1 + 2x5 | 1 + 2x5 |
| Menu ink | follows panel | follows colour scheme | still scheme-based |

The Dock horizontal padding after the change is 8.84 at a 64 tile: the 6
base plus 4.44% of 64. The 4 px slot gap and the 15.625% vertical padding make
the default panel 84 high and put the slots 68 apart, matching the measured
geometry.

## 7. Borders and dynamic behaviour

- The Dock rim is a light/dark bevel. A single outline token is not enough.
- The menu panel edge is about 1 px and follows the panel lightness.
- The menu ink is dynamic with the panel, not with the colour scheme.
- The menu bar ink is fixed by the colour scheme, not by the window behind it.
- The Dock still follows the backdrop with Reduce Transparency on. A fixed
  solid token is an approximation.
- The window inner border was not measured in this pass. It remains a gap.

## 7b. Corrections from the blur pass

A later pass measured every shell surface over white, gray, black, checker
and a grid. It corrects three values above; the details and evidence are in
[`macos-blur.md`](macos-blur.md).

- The light Dock matrix (`#eaeaea`, `#a5a5a5`, `#323232`, light opaque
  `#f1f1f1`) was recorded under a different wallpaper state. The Dock glass
  follows the wallpaper, not the scheme; with the dark wallpaper both modes
  give the dark matrix and Reduce Transparency gives solid `#020202` to
  `#0a0a0a`.
- The menu panel is scheme-independent: `#8d8d8d` over white and `#2b2b2b`
  over black in both modes, with the ink following the panel. Reduce
  Transparency leaves it unchanged.
- Reduce Transparency leaves Control Center unchanged too.

## 8. Remaining gaps

1. The menu ink follows the panel as a whole. macOS switches it at the split
   line of a split backdrop; Malaysia takes one ink per popup from the average
   of what lies behind it.
2. Reduce Transparency still leaves the Dock backdrop-dependent. The single
   solid tokens are an approximation, not the measured state.
3. The menu corner radius is about 6 on this host, below the 12 used by the
   menus. It needs a second measurement at 2x.
4. The top-bar right status area was not measured. Its item spacing is still
   a script default.

## 9. Evidence

The captures are local and not committed:

- `~/src/captures/default-dock-gray.png`
- `~/src/captures/default-menu-white.png`
- `~/src/captures/default-menu-gray3.png`
- `~/src/captures/default-menu-black.png`
- `~/src/captures/default-menu-checker.png`
- `~/src/captures/default-menu-split.png`
- `~/src/captures/default-menu-graybands.png`
- `~/src/captures/default-finder-menu-white.png`
- `~/src/captures/default-finder-menu-black.png`
- `~/src/captures/light-off-*-dock.png`
- `~/src/captures/light-on-*-dock*.png`
- `~/src/captures/reduced-*-dock.png`
- `~/src/captures/defaults-2026-11-19/`
