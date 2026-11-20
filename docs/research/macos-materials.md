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
[`macos-blur.md`](macos-blur.md). The shared result, measured at 2560x1440
under one wallpaper:

- The menu is **adaptive and scheme-dependent**: dark mode is a black veil
  capped bright (`#a3` over white, `#2b` over black); light mode is a white
  veil capped near white (`#f2` over white, `#75` over black). The ink follows
  the panel in both modes.
- The Dock is **scheme-biased**: dark glass in dark mode (`#be` over white,
  `#0d` over black), light glass in light mode (`#eb` over white, `#34` over
  black), still following the backdrop.
- Both blur strongly; the checker becomes a low-contrast texture or a single
  gray, depending on the surface.
- Neither is a single opacity. The bright and dark ends do not lie on one
  straight line.

The per-surface veil constants that reproduce these composites are in
[`macos-blur.md`](macos-blur.md#5-the-model-fit-one-veil-per-surface-per-mode).
The 2026-11-19 light values (`#eaeaea` Dock over white) belong to a darker
wallpaper state.

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

The measured matrix includes Reduce Transparency. The Dock becomes a **scheme
solid**: `#03` to `#0d` in dark mode and `#f1` to `#89` in light mode, both
over white and black backdrops. The menu and Control Center stay unchanged.
A single solid token per scheme is therefore accurate for the Dock; the
backdrop response that remains is a few units, not a material.

## 5. Comparison with Malaysia

| Element | macOS default | Malaysia before | After |
|---|---|---|---|
| Light Dock glass | `#ebebeb` over white | `#ffffff66` | `#b5b5b548` |
| Light Dock opaque | `#f1f1f1` over white | `#757575` | `#f1f1f1` |
| Dark Dock glass | `#bebebe` over white, `#0d` over black | `#2727274e` | unchanged |
| Menu glass | Two veil sets by scheme | fixed scheme glass | dark veil set only |
| Menu ink | Follows panel | follows scheme | follows the panel |
| Menu blur | Strong | `menu_glass` soften | verify |
| Dock rim | 3-step light/dark bevel | `rim` and `tone` | model exists |
| Label hierarchy | 84.7 / 54.9 / 24.7 / 9.8 | `ink`, `ink-muted`, `ink-faint` | fewer levels |
| Fill hierarchy | 9.8 / 7.8 / 4.7 / 2.7 | `bg-control`, `line` factors | different scheme |
| Reduce Transparency | Dock scheme solid; menu and CC unchanged | solid fallback | approximate |

## 6. What not to copy

- Apple material images or pre-rendered blur textures.
- SF Pro and SF Symbols.
- Apple's semantic names in public API. Use the glossary names.
- A photograph as the only backdrop for a glass test.

## 7. Prioritized changes

1. **Add the light menu veil set.** The dark tokens reproduce the dark
   matrix; a light scheme needs its own veil and cap (`#f2` over white,
   `#d1` over gray, `#75` over black). Targets in
   [`macos-blur.md`](macos-blur.md#5-the-model-fit-one-veil-per-surface-per-mode).
2. **Keep the corrected light Dock tokens.** `#b5b5b548` matches the measured
   light glass over white; verify it against `#ebebeb` and `#34` before
   changing it again.
3. **Keep the corrected light opaque token.** `#f1f1f1` is the measured light
   Reduce Transparency result over white; add the dark `#03` counterpart.
4. **Add four-backdrop tests for every glass token per scheme.** White, gray,
   black and checker, dark and light. A single wallpaper test cannot catch
   the adaptive menu.
5. **Add a quaternary label level.** macOS exposes four label levels; the
   current three-token set cannot express disabled or quaternary text.
6. **Verify the blur pass count and spread.** Use a checkerboard with a known
   square size and compare the edge energy.
7. **Test Reduce Transparency per scheme.** The fallback must replace the
   material with the scheme solid, not merely raise its opacity.

## 8. Open questions

- How many blur passes does each material use? The measured matrix is in
  [`macos-blur.md`](macos-blur.md); the pass count is still unknown.
- What is the rim width at 2x?
- Does the menu glass change with the wallpaper tint or only the window?
  [`macos-blur.md`](macos-blur.md) shows the menu is backdrop-driven and
  scheme-dependent; the wallpaper interaction is open.
- Do the notification and menu glass share the same composite? They do not:
  the menu and Control Center share one tone curve and Notification Center
  carries a lighter veil in both modes.
- Does light mode track a dark wallpaper and revert to the dark matrix?

## 9. Implementation prompt

```text
You are fixing the material model from measured values. Read this file and
macos-blur.md first. Work in ~/src/barrierkit and ~/src/malaysia, main
only: git pull --ff-only before editing and before each push, one commit
per session, English messages, no code comments.

Measured targets, dark / light:
- Menu: white #a3a3a3 / #f2f2f2, gray #92-94 / #d1d1d1, black #2b2b2b /
  #75-7b, checker #8b-8d / #bf-c6.
- Dock: white #bebebe / #ebebeb, gray #8b / #a6a6a6, black #0d-10 / #34,
  checker #6c / #91.
- RT: Dock scheme solid #03-0d / #f1-89; menu and CC unchanged.
- Label hierarchy: 84.7 / 54.9 / 24.7 / 9.8 per appearance.

Tasks:
1. barrierkit: add the light menu veil set; align the Dock light values
   with the targets; make the RT Dock resolve to the scheme solid.
2. barrierkit: four-backdrop tests per surface per scheme in
   ctappearance::resolve, asserting the targets within 8/255; keep the
   label hierarchy as the source for glass ink.
3. malaysia: after the rev bump, confirm the blur path and the RT
   early-out still hold.

Definition of done: ./dev.sh check passes in both repos; tests assert the
measured values; barrierkit main is pushed and malaysia pins the new rev.
Report changed tokens, test values and commits.
```
