# macOS Dock

**Goal:** describe the default Dock slab, its icons, glass, label and context
menu with values BarrierKit can compare against `ctdesign.shell.dock`.
**Scope:** a bottom Dock with the default geometry, magnification off and
auto-hide off. It excludes side Docks, hidden reveal, minimisation and launch
bounce.
**Reader:** anyone who changes `malaysia-shell::dock` or the Dock tokens.
**Input:** a reset Dock at 1920x1080, controlled backdrops and hover captures.
**Output:** default geometry, glass references, a Malaysia comparison and
ordered gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the complete default matrix in
   [`macos-defaults.md`](macos-defaults.md).
3. Use the glass table with its backdrop; do not lift one value.
4. Check the comparison table before changing a Dock token.
5. Do not copy Apple application icons.

---

## 1. Reset state

The Dock appearance keys were deleted and the Dock was restarted, keeping the
user's pinned applications. The default state used for this file is:

| Key | Effective value |
|---|---:|
| `tilesize` | 64 |
| `largesize` | 64 |
| `magnification` | off |
| `autohide` | off |
| `orientation` | bottom |

The original host state was `tilesize=128`, `largesize=128`, magnification on.
It was restored after the pass. The 128 values are not the default.

## 2. Default geometry

| Property | Value |
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

The panel is content-sized. The 1576 width comes from 22 tiles, 2 separators
and the edge padding on this host. The radius is about **0.262** of the panel
height, not 0.222.

The icon body is 52 square inside a 64 tile and a 68x84 slot. The extra slot
space is the hit area and the label room.

## 3. Default glass

The complete four-backdrop matrix is in
[`macos-defaults.md`](macos-defaults.md#3-default-dock-glass). The short
version for dark mode with transparency allowed:

| Backdrop | Dock sample |
|---|---|
| White | `#bebebe` |
| Gray | `#8a8a8a` |
| Black | `#0d0d0d` |
| Checker | `#7f7f7f` |

The Dock is a blurred dark glass in dark mode. In light mode the same
backdrops give `#eaeaea`, `#a5a5a5`, `#323232` and `#979797`; the light token
was corrected to `#b5b5b548` in this pass.

The rim is a three-step light/dark bevel, not a one-pixel outline. In dark
mode over gray it reads `#919191`, `#808080`, `#919191`; in light mode over
white it reads `#e4e4e4`, `#f0f0f0`, `#eaeaea`.

## 4. Icons, magnification and label

- The Dock uses the full slot for hit testing. The visible icon body is
  smaller than the slot.
- Magnification was measured in the earlier 128 state: the Finder icon grew to
  about **1.3x** under the pointer. The default state has magnification off.
- The label `Finder` appears above the icon on hover. It is a pill with a
  pointer, not a plain rectangle.
- A right-click opens a context menu above the Dock. It shares the application
  menu glass and was captured but not measured.
- The Dock is visible by default. Auto-hide and launch bounce were not
  measured.
- The magnification timing is still unreliable: the recorder starts about
  1.5 s after the command. Use a pre-rolled recorder and a fixed cursor before
  fitting a curve.

## 5. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---:|---|---:|
| Tile size | 64 | 48 | 64 |
| Large size | 64 | 64 | 64 |
| Edge margin | 6 | 4 | 6 |
| Padding base | about 9 | 4 plus 4.44% of tile | 6 plus 4.44% of tile |
| Panel height at 64 | 84 | 77.7 | 84 |
| Panel radius | about 22 | 27.2% of thickness | 27.2% of thickness |
| Slot spacing | 68 | 64 | 68 |
| Visible icon body | 52 | 80.5% of the short side | unchanged |
| Magnification floor | about 1.3x | 1.3 | 1.3 |
| Light glass | `#eaeaea` over white | `#ffffff66` | `#b5b5b548` |
| Light opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dark glass | `#0d0d0d` over black | `#2727274e` | unchanged |

The radius ratio is already close: 22/84 is 0.262 and the token is 0.272.
The new tile, padding, slot gap and vertical padding close the size gap.

## 6. What not to copy

- Finder, Safari, Mail or any Apple application icon.
- Apple's running-indicator artwork or badges.
- Apple's Dock context-menu labels and translations.
- A macOS Dock screenshot as a texture.

Use the project's own application icons and draw the indicator from tokens.

## 7. Prioritized changes

1. **Keep the slot gap and vertical padding.** The 4 px gap and the 15.625%
   vertical padding match the measured slot and panel height.
2. **Keep the radius ratio.** 0.272 is within one pixel of the measured
   0.262. Do not change it to 0.222.
3. **Model the rim as a light/dark pair.** The measured bevel is not a single
   outline. The existing `rim-light`, `rim-dark`, `rim-side` and `rim-corner`
   tokens are the right place; add a backdrop sample test.
4. **Measure magnification properly.** The 1.3x floor is known; the curve,
   overshoot and duration are not.
5. **Measure auto-hide and launch bounce.** They are token-only today.

## 8. Open questions

- Does the clean macOS default use 64 or another tile value at 2x?
- Does the Dock radius scale linearly with tile size?
- Does the icon body scale with the slot or with the artwork's safe area?
- Is the magnification curve a spring, and does it overshoot?
- Does auto-hide use the same glass and rim as the visible Dock?
