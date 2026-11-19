# macOS Dock

**Goal:** describe the Dock slab, its icons, magnification, label and context
menu with values BarrierKit can compare against `ctdesign.shell.dock`.
**Scope:** a bottom Dock that is visible, magnification on and auto-hide off.
It excludes side Docks, hidden reveal, minimisation and launch bounce.
**Reader:** anyone who changes `malaysia-shell::dock` or the Dock tokens.
**Input:** a live Dock, controlled-backdrop screenshots and hover captures.
**Output:** measured geometry, glass values, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the host Dock defaults before comparing tile sizes.
3. Use the glass table with its backdrop; do not lift one value.
4. Check the comparison table before changing a Dock token.
5. Do not copy Apple application icons.

---

## 1. Host state

The Dock preferences on this machine:

| Key | Value |
|---|---:|
| `tilesize` | 128 |
| `largesize` | 128 |
| `magnification` | 1 |
| `autohide` | 0 |

These are **host state**, not a claim about the macOS default. They explain
why the slab and the slots are large. Compare geometry at the same tile size
before calling a token wrong.

## 2. Measured geometry

| Property | Value |
|---|---:|
| Panel left | 101 |
| Panel right | 2458 |
| Panel width | 2358 |
| Panel top | 1309 |
| Panel bottom | 1434 |
| Panel height | 126 |
| Panel radius | about 28 |
| Slot width | 109 |
| Slot height | 125 |
| Slot spacing | 110 |
| Visible icon body | about 76x80 |
| Icon baseline | y = 1309 |

The panel is almost the full screen width because the Dock contents need
2358 pt. The height is 126, which is one point more than the slot height.
The radius is about **28**, or roughly 22% of the height.

The visible icon body is **76x80**, but macOS application icons contain
transparent margins. Do not compare that box with the icon plate size
directly. Compare the **slot** and the panel height, which are measurable
without knowing the artwork.

## 3. Measured glass

The Dock is a translucent dark slab in this dark-mode session. Its composite
over four controlled backdrops:

| Backdrop | Dock sample |
|---|---|
| White | `#bebebe` |
| Gray | `#8a8a8a` |
| Black | `#0d0d0d` |
| Checker | `#7f7f7f` |

The checker collapses to a single gray, so the Dock blurs strongly. The
values are not a single opacity:

- Over white the slab is **25% darker** than white.
- Over black it is **5% lighter** than black.
- Over gray it is about **15% darker** than the gray.
- The checker result sits between the gray and black results, which is what
  a blur plus a dark tint produces.

The slab has a **1 px outline** and a lighter bevel:

- Dark rim sampled around `#808080` over gray.
- Light rim sampled around `#919191` over gray.
- The rim is visible on the top edge and the rounded corners.

The rim is not a flat border. It is a light/dark pair that changes with the
backdrop. A single `dock-outline` token cannot reproduce both samples.

## 4. Icons, magnification and label

- The Dock uses the **full-height slot** for hit testing. The visible icon
  body is smaller than the slot.
- With magnification on, the Finder icon measured about **88x102** under the
  pointer, or about **1.3x** its resting body box. The rounded rectangle
  around it grows with the icon.
- The label `Finder` appears above the icon on hover. The label has a small
  arrow pointing at the icon.
- The label uses the menu-bar title-open shape: a pill with a pointer, not a
  plain rectangle.
- A right-click opens a context menu above the Dock. The context menu shares
  the application menu glass. It was captured but not measured in this pass.
- The Dock is **not hidden**. No reveal strip, hide timing or auto-hide
  animation was measured.
- No launch bounce, minimisation or recents stack was measured.

The magnification timing is not reliable yet. The recorder starts about
1.5 s after the command, so the first frames miss the cursor placement. Use a
pre-rolled recorder and a fixed cursor position before fitting a curve.

## 5. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Tile size | 128 on this host | 48 default, 16 to 128 range | `DOCK_TILESIZE`, `DOCK_TILESIZE_MAX` | Host state; verify intended default |
| Large size | 128 | 64 | `DOCK_LARGESIZE` | Magnified ceiling is lower |
| Panel height at 128 tiles | 126 | Derived from tile plus padding | `dock/geometry.rs` | Compare at equal tile |
| Panel radius | about 28 | 27.2% of thickness | `DOCK_RADIUS` | Malaysia is rounder |
| Slot spacing | 110 | Tile plus padding | `dock/geometry.rs` | Compare at equal tile |
| Visible icon body | 76x80 | 80.5% of the short side | `DOCK_ICON_BODY` | Icon art margins make this uncertain |
| Magnification | about 1.3x | 1.3 minimum | `DOCK_MAGNIFY_MIN` | Matches the observed floor |
| Magnify radius | Not measured | 3 tiles | `DOCK_MAGNIFY_RADIUS` | Unverified |
| Magnify duration | Not measured | 120 ms | `DOCK_MAGNIFY_MS` | Unverified |
| Glass | Dark blur plus dark tint | `dock-bar` light `#ffffff66`, dark `#2727274e` | `themes.*.dock-bar` | Light mode and tint model need work |
| Outline | 1 px dark plus light bevel | `dock-outline` plus rim tokens | `themes.*.dock-outline` | Bevel model differs |
| Label | Pill plus pointer, 13 pt | 13 pt, 18/5 padding, 12 radius | `DOCK_LABEL_*` | Close; verify height |
| Running dot | Not measured | 4.4% of tile, 2 to 5 px | `DOCK_DOT` | Unverified |
| Context menu | Shares menu glass | Dock menu tokens | `DOCK_MENU_*` | Verify with menu glass |

The largest known mismatch is the **radius ratio**: about 22% on macOS versus
27.2% in Malaysia at the same thickness. The largest unknown is the **icon
body**, because the visible pixels exclude transparent icon margins.

## 6. What not to copy

- Finder, Safari, Mail or any Apple application icon.
- Apple's running-indicator artwork or badges.
- Apple's Dock context-menu labels and translations.
- A macOS Dock screenshot as a texture.

Use the project's own application icons and draw the indicator from tokens.

## 7. Prioritized changes

1. **Match the radius ratio.** Use about **0.222** of the slab thickness
   instead of **0.272**, or add a separate slab-radius token. This is visible
   on every Dock corner.
2. **Model the rim as a light/dark pair.** The measured bevel is not a single
   outline. The existing `rim-light`, `rim-dark`, `rim-side` and `rim-corner`
   tokens are the right place; add a backdrop sample test.
3. **Decide the default tile size.** The host uses 128. Malaysia defaults to
   48. If the goal is the macOS look, the default must be verified against a
   clean macOS account, not this host.
4. **Measure magnification properly.** Pre-roll the recorder, hold the
   cursor on one slot, and fit the height curve. The observed floor is 1.3x;
   the curve, overshoot and duration are still unknown.
5. **Verify the label height and pointer.** The macOS label is a pill with a
   pointer and sits about 10 pt above the icon. Malaysia has the same shape;
   compare the measured height.
6. **Measure auto-hide and launch bounce.** They are token-only today, so a
   regression cannot be detected.

## 8. Open questions

- What is the clean-install `tilesize` on macOS 27?
- Does the Dock radius scale linearly with tile size?
- Does the icon body scale with the slot or with the artwork's safe area?
- Is the magnification curve a spring, and does it overshoot?
- Does auto-hide use the same glass and rim as the visible Dock?
