# macOS materials, glass and vibrancy

**Goal:** turn the controlled-backdrop samples into a material model that
`ctappearance` can reproduce.
**Scope:** blur, tint, fill hierarchy and vibrancy on the menu panel, the Dock
and the notification card. It excludes the dynamic Liquid Glass behaviour,
which has its own file.
**Reader:** anyone who changes `ctappearance::resolve`, `glass_soften`,
`dock_surface` or `menu_glass`.
**Input:** four controlled backdrops and the semantic colour dump.
**Output:** measured composite values, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the backdrop table before using a glass value.
3. Pick a material by role, never by its apparent colour.
4. Check the fill hierarchy before adding a new token.
5. Do not copy Apple material images or SF Symbols.

---

## 1. What a material is

A macOS material is not a flat colour. It combines:

- A **blur** of the content behind it.
- A **tint** that lifts or lowers the blurred result.
- A **fill hierarchy** for text and glyphs.
- A **rim** or edge treatment.
- Accessibility overrides that change the material itself.

The measured surfaces show all of these except the rim in the numeric
samples. The blur is visible because a checkerboard becomes a single gray.
The tint is visible because black is lifted and white is lowered. The
hierarchy is visible because labels remain readable on both.

## 2. Measured composites

The composites are measured in the file that owns each surface:

- [macos-menu-bar.md](macos-menu-bar.md#4-measured-glass) for the bar and the
  application menu.
- [macos-dock.md](macos-dock.md#3-measured-glass) for the Dock.

This file does not repeat those tables. The shared result is that both
surfaces blur: a checker becomes a single gray. They differ in strength: the
menu is much darker than the Dock. Do not use one token for both.

## 3. Measured fill hierarchy

Light appearance:

| Role | Value |
|---|---|
| `label` | `#000000` at 84.7% |
| `secondaryLabel` | `#000000` at 49.8% |
| `tertiaryLabel` | `#000000` at 25.9% |
| `quaternaryLabel` | `#000000` at 9.8% |
| `separator` | `#000000` at 10% |
| `systemFill` | `#000000` at 9.8% |
| `secondarySystemFill` | `#000000` at 7.8% |
| `tertiarySystemFill` | `#000000` at 4.7% |

Dark appearance:

| Role | Value |
|---|---|
| `label` | `#ffffff` at 84.7% |
| `secondaryLabel` | `#ffffff` at 54.9% |
| `tertiaryLabel` | `#ffffff` at 24.7% |
| `quaternaryLabel` | `#ffffff` at 9.8% |
| `separator` | `#ffffff` at 12% |
| `systemFill` | `#ffffff` at 9.8% |
| `secondarySystemFill` | `#ffffff` at 7.8% |
| `tertiarySystemFill` | `#ffffff` at 4.7% |

Vibrancy uses this hierarchy on top of the material. The menu text is not a
fixed white: it is a label role that resolves against the current material
and appearance.

## 4. Accessibility overrides

Observed settings on this host were all off. The public API exposes them, and
they change the material rather than the app content:

- **Reduce Transparency** removes translucency and substitutes opaque
  surfaces.
- **Increase Contrast** strengthens text and separator contrast.
- **Differentiate Without Color** adds shapes or glyphs.
- **Reduce Motion** changes transitions, not the material.

The measured values in this file are the **standard** state. They are not
valid for Reduce Transparency or Increase Contrast.

## 5. Comparison with Malaysia

| Element | macOS (measured) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Menu glass over black | `#2c2d2d` | Dark base plus `#ffffff21` overlay | `menu_glass` | Close; verify tone lift |
| Menu glass over white | `#656565` | Light base plus `#f5f5f77a` overlay | `menu_glass` | Bright cap needs work |
| Menu blur | Strong, checker uniform | `glass_soften` and blur passes | `ctappearance` | Pass count unverified |
| Dock glass over black | `#0d0d0d` | `#2727274e` over base | `dock-bar` | Close, verify |
| Dock glass over white | `#bebebe` | `#ffffff66` in light mode | `dock-bar` | Light mode not captured |
| Rim | 1 px light/dark pair | `rim-light`, `rim-dark`, `tone` | `dock` and `menu` | Model exists |
| Label hierarchy | 84.7 / 54.9 / 24.7 / 9.8 | `ink`, `ink-muted`, `ink-faint` | `tokens` | Fewer levels |
| Fill hierarchy | 9.8 / 7.8 / 4.7 / 2.7 | `bg-control`, `line` factors | `layer-factors` | Different scheme |
| Reduce Transparency | Opaque substitution | `reduce_transparency` paths | `resolve.rs` | Matches in principle |
| Blur model | Blur plus tint plus rim | `glass_soften` plus `liquid_glass` | `resolve.rs` | Tuning gap |

The model is close. The **menu glass base** and the **number of label levels**
are the two concrete mismatches.

## 6. What not to copy

- Apple material images or pre-rendered blur textures.
- SF Pro and SF Symbols.
- Apple's semantic names in public API. Use the glossary names.
- A photograph as the only backdrop for a glass test.

## 7. Prioritized changes

1. **Add four-backdrop tests for every glass token.** White, gray, black and
   checker. A single wallpaper test cannot catch the menu panel's darkening.
2. **Tune the bright cap of the menu glass.** The dark end is already close:
   a 13% white overlay over black lands near the measured `#2c2d2d`. The
   over-white case is the gap, because the panel must fall to `#656565`
   instead of staying near white.
3. **Keep the Dock and menu materials separate.** Their composites differ by
   more than a small tint.
4. **Add a quaternary label level.** macOS exposes four label levels; the
   current three-token set cannot express disabled or quaternary text.
5. **Verify the blur pass count and spread.** Use a checkerboard with a
   known square size and compare the edge energy.
6. **Test Reduce Transparency.** The fallback must replace the material, not
   merely raise its opacity.

## 8. Open questions

- How many blur passes does each material use?
- What is the rim width at 2x?
- Does the menu glass change with the wallpaper tint?
- Do the notification and menu glass share the same composite?
- What does the light menu panel sample over the same four backdrops?
