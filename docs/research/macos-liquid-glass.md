# macOS Liquid Glass

**Goal:** describe the glass material as a system, not as one blur token, and
show which parts Malaysia already models and which parts are missing.
**Scope:** the glass used by the menu panel, the Dock, Control Center and
notification cards, plus the grouping, tint and accessibility rules that
change it.
**Reader:** anyone who changes `ctappearance` glass, `DockSurface`,
`menu_glass` or the compositor's blur path.
**Input:** four-backdrop captures, the semantic colour dump and the existing
glass model in `ctappearance`.
**Output:** observed behaviour, a Malaysia comparison and ordered gaps.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the standard material model in
   [`macos-materials.md`](macos-materials.md).
3. Treat "glass" as a system of blur, tint, rim, lens, tone and grouping.
4. Do not reduce Liquid Glass to a single opacity or blur radius.
5. Do not copy Apple shaders, textures or SF Symbols.

---

## 1. What Liquid Glass is

Apple describes Liquid Glass as a material that:

- Has **no inherent colour**; it takes colour from the content behind it.
- **Lenses** or refracts that content instead of only blurring it.
- Carries a **specular rim** that changes with the light and the backdrop.
- **Adapts** its light or dark ink to what is underneath.
- Can be **tinted** with the accent, especially for prominent controls.
- Can be **interactive**, responding to press and hover.
- Can be **grouped** so nearby shapes render and morph as one.
- Is modified by Reduce Transparency and Increase Contrast.

The public descriptions are summarised in
[`styling-apple.md`](styling-apple.md). This file adds the observations from
the live desktop.

## 2. What was observed

The captured desktop does **not** make every surface glass:

| Surface | Observation |
|---|---|
| Menu bar | Clear, no blur, adaptive ink |
| Application menu | Dark glass, strong blur, rim |
| Dock | Dark glass, strong blur, rim and bevel |
| Control Center | Dark glass, strong blur, rim |
| Notification | Dark glass, rim |
| Normal window | Opaque semantic background |

The menu bar is the important exception. It is transparent but not frosted,
so "Liquid Glass everywhere" is false. The bar keeps the desktop sharp and
changes its ink; the panel below it is the frosted element.

The controlled-backdrop samples live in the file that owns each surface:
[macos-menu-bar.md](macos-menu-bar.md#4-measured-glass) and
[macos-dock.md](macos-dock.md#3-measured-glass). The checker result is a
single gray on both surfaces, so both blur. The menu is much darker than the
Dock, so they are different glass roles.

The static captures cannot prove **lensing**. A lens bends the image behind
the glass; a blur destroys its edges. The checker test shows the blur but
does not measure a displacement field. Treat lensing as observed only where
a live edge visibly bends, and mark it as an open question here.

## 3. The existing Malaysia model

Malaysia already has more than a blur token:

- `glass_blur` is a 0 to 1 setting.
- `liquid_glass(glass_blur) = glass_blur * glass_blur`, measured on the
  macOS 26 Liquid Glass slider: halfway, the blur has grown a quarter of the
  way and the tint has hardly moved.
- `GLASS_BLUR_SPREAD = 4.0` and `GLASS_BLUR_FULL = 0.6` widen the blur from
  its resting value to the full setting.
- `glass_soften(rest, amount)` interpolates between the resting blur and the
  spread.
- `DockSurface` carries `glass`, `outline`, `rim`, `divider`, `corner`,
  `rim_side`, `glow`, `lens`, `tone` and `soften`.
- `menu_glass` reuses the Dock frame but replaces the fill, removes the side
  rim, flattens the lens and adds a dark tone cap and chroma.
- `glass_chroma`, `glass_lightness`, `glass_rim_lift`,
  `glass_blur_passes`, `glass_edge` and `glass_shadow` describe the
  wallpaper interaction.
- `reduce_transparency` switches the Dock and menu to solid surfaces.

That is a strong base. The gaps are in the higher-level behaviours, not in
the existence of a glass type.

## 4. Comparison with Malaysia

| Element | macOS (observed) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Clear menu bar | Yes | `shell-bar` tinted, optional blur | `menubar/bar.rs` | Bar is not clear |
| Dark menu glass | `#2c2d2d` over black | `menu_glass` plus dark tone | `resolve.rs` | Tuning gap |
| Dock glass | `#0d0d0d` over black | `DockSurface` dark glass | `dock_surface` | Close |
| Strong blur | Checker becomes uniform | `glass_blur_passes`, `soften` | `resolve.rs` | Pass count unverified |
| Specular rim | Light/dark 1 px pair | `rim`, `rim_side`, `corner`, `glow` | `DockSurface` | Model exists |
| Lens | Not proven by static capture | `lens` parameters | `DockSurface` | Unverified, maybe unused |
| Adaptive ink | Yes | `glass-ink` roles | `tokens` | Fewer levels |
| Accent tint | Prominent controls | `take_accent_chroma` | `resolve.rs` | Only some surfaces |
| Grouping | Nearby shapes render together | No group | — | New concept |
| Morphing | Shapes morph as one | No morph | — | New concept |
| Interactive glass | Press and hover response | No interactive mode | — | New concept |
| Regular / clear / identity | Public variants | One glass plus `glass_blur` | — | Variants absent |
| Reduce Transparency | Makes glass frostier or opaque | Solid fallback | `resolve.rs` | Matches in principle |
| Increase Contrast | Stronger material | Contrast paths exist | `resolve.rs` | Verify |

The largest concrete gap is that the **menu bar is not clear**. The largest
conceptual gap is the **group and morph** behaviour: Malaysia has individual
glass surfaces, not a glass container that manages several shapes.

## 5. What not to copy

- Apple's lens shader, specular texture or glass rendering code.
- Apple's SF Symbols or application icons.
- A screenshot of a glass surface as a texture.
- Apple's exact HIG wording in user-visible text.
- The names `regular`, `clear` and `identity` as public API without checking
  the glossary rules.

## 6. Prioritized changes

1. **Make the default menu bar clear.** Use adaptive ink and no tint or blur.
   Keep the current tinted bar as the Reduce Transparency or opaque variant.
2. **Promote glass to roles.** Define at least `menu`, `dock`, `panel` and
   `card`, each with its own fill, blur, rim and tone. The measurements show
   the menu and Dock are not the same material.
3. **Add a glass group.** A container owns the blur, the rim and the tone for
   its children, so two nearby cards can merge. This is the feature that
   makes the system feel like Liquid Glass rather than frosted rectangles.
4. **Add morphing between grouped shapes.** Animate the container's shape,
   not each child independently.
5. **Add an interactive state.** Press and hover change the glass slightly,
   without a full re-render of the backdrop.
6. **Implement adaptive ink by luminance.** Sample the backdrop or use the
   known wallpaper tint, then choose light or dark glass ink. Do not use a
   fixed white label.
7. **Keep the quadratic `glass_blur`.** It is already measured against the
   macOS slider. Do not replace it with a linear ramp.
8. **Verify `lens`.** If no surface uses it, delete it. If the Dock uses it,
   add a live edge test that proves the displacement.

## 7. Open questions

- Does the macOS 27 menu panel actually lens, or only blur and tint?
- Is the menu bar clear in both light and dark mode?
- Does the Dock lens at rest or only under magnification?
- How does a glass group decide when two shapes merge?
- What is the exact interaction between Increase Contrast and the rim?
- Does Reduce Transparency keep the rim or remove it?
- Does the accent tint apply to the glass fill or to the label?
