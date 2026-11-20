# macOS materials, glass and vibrancy

**Goal:** turn the controlled-backdrop samples into a material model that
`ctappearance` can reproduce.
**Scope:** blur, tint, fill hierarchy and vibrancy on the menu panel, the Dock
and the notification card. It excludes the dynamic grouping behaviour, which
has its own file.
**Reader:** anyone who changes `ctappearance::resolve`, `glass_soften`,
`dock_surface` or `menu_glass`.
**Input:** four controlled backdrops and the semantic colour dump.
**Output:** the measured material model, a Malaysia comparison and ordered
gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the complete default matrix in
   [`macos-defaults.md`](macos-defaults.md).
3. Pick a material by role, never by its apparent colour.
4. Check the fill hierarchy before adding a new token.
5. Read the per-element blur and lensing matrix in
   [`macos-blur.md`](macos-blur.md).
6. Do not copy Apple material images or SF Symbols.

---

## 1. What a material is

A macOS material is not a flat colour. It combines:

- A **blur** of the content behind it.
- A **tint** that lifts or lowers the blurred result.
- A **fill hierarchy** for text and glyphs.
- A **rim** or edge treatment.
- Accessibility overrides that change the material itself.

The white, gray, black and checker backdrops separate these parts. The checker
shows the blur. The black and white results show the tone curve. The label
roles show the fill hierarchy. The rim is visible as a light/dark bevel.

## 2. Measured composites

The menu and Dock tables live in their area files, and the full matrix is in
[`macos-defaults.md`](macos-defaults.md). The shared result is:

- The menu is **adaptive**: light over a bright backdrop, dark over a dark
  one, with the ink following the panel.
- The Dock is **scheme-biased**: dark glass in dark mode, light glass in light
  mode, still following the backdrop.
- Both blur strongly; the checker becomes a low-contrast texture or a single
  gray, depending on the surface.
- Neither is a single opacity. The bright and dark ends do not lie on one
  straight line.

The measured light Dock glass over white, gray and black is `#eaeaea`,
`#a5a5a5` and `#323232`. The dark Dock glass is `#bebebe`, `#8a8a8a` and
`#0d0d0d`. The light token was corrected from `#ffffff66` to `#b5b5b548`.

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

The public API exposes four settings that change the material:

- **Reduce Transparency** removes translucency and substitutes opaque or
  frosted surfaces.
- **Increase Contrast** strengthens text and separator contrast.
- **Differentiate Without Color** adds shapes or glyphs.
- **Reduce Motion** changes transitions, not the material.

The measured matrix includes Reduce Transparency. The Dock still follows the
backdrop a little even when it is on; a single solid token is an
approximation. The menu stays adaptive in dark mode, and in light mode it
becomes a near-white surface.

## 5. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Light Dock glass | `#eaeaea` over white | `#ffffff66` | `#b5b5b548` |
| Light Dock opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dark Dock glass | `#0d0d0d` over black | `#2727274e` | unchanged |
| Menu glass | Adaptive light/dark | fixed scheme glass | adaptive model |
| Menu ink | Follows panel | follows scheme | still scheme-based |
| Menu blur | Strong | `menu_glass` soften | verify |
| Dock rim | 3-step light/dark bevel | `rim` and `tone` | model exists |
| Label hierarchy | 84.7 / 54.9 / 24.7 / 9.8 | `ink`, `ink-muted`, `ink-faint` | fewer levels |
| Fill hierarchy | 9.8 / 7.8 / 4.7 / 2.7 | `bg-control`, `line` factors | different scheme |
| Reduce Transparency | Opaque/frosted substitution | solid fallback | approximate |

## 6. What not to copy

- Apple material images or pre-rendered blur textures.
- SF Pro and SF Symbols.
- Apple's semantic names in public API. Use the glossary names.
- A photograph as the only backdrop for a glass test.

## 7. Prioritized changes

1. **Keep the corrected light Dock token.** `#b5b5b548` matches the measured
   light glass over white and black; the old `#ffffff66` was too bright.
2. **Keep the corrected light opaque token.** `#f1f1f1` is the measured light
   Reduce Transparency result over white.
3. **Add four-backdrop tests for every glass token.** White, gray, black and
   checker. A single wallpaper test cannot catch the adaptive menu.
4. **Add a quaternary label level.** macOS exposes four label levels; the
   current three-token set cannot express disabled or quaternary text.
5. **Verify the blur pass count and spread.** Use a checkerboard with a known
   square size and compare the edge energy.
6. **Test Reduce Transparency.** The fallback must replace the material, not
   merely raise its opacity.

## 8. Open questions

- How many blur passes does each material use? The measured matrix is in
  [`macos-blur.md`](macos-blur.md); the pass count is still unknown.
- What is the rim width at 2x?
- Does the menu glass change with the wallpaper tint or only the window?
  [`macos-blur.md`](macos-blur.md) shows the menu is backdrop-driven and
  scheme-independent.
- Do the notification and menu glass share the same composite? They do not:
  the menu and Control Center share one tone curve, Notification Center is
  darker, and the Dock follows the wallpaper.
- What does the light menu panel sample over the same four backdrops? It
  samples the same values as the dark panel; see the corrections in
  [`macos-blur.md`](macos-blur.md#7-corrections-to-the-earlier-research).
