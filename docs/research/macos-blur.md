# macOS blur and Liquid Glass per element

**Goal:** classify every macOS shell surface by its material and give measured
values for its blur, tint, border, shadow and lensing in both appearances, so
a compositor can tell Liquid Glass from a general blur and can use one value
per mode instead of guessing.
**Scope:** the menu bar, the application menu panel, the Dock, Control Center,
Notification Center, Spotlight, notification banners, Mission Control,
Launchpad and one window's chrome. It ends at the classification table and the
implementation prompt.
**Reader:** anyone who changes `ctappearance` glass, the compositor blur path
or a Malaysia shell surface.
**Input:** the probes in `~/src/probes`, the four controlled backdrops, the
Reduce Transparency switch and dark and light mode measured under the same
wallpaper.
**Output:** this matrix, the per-mode model constants, the Liquid Glass
verdict, the code gaps and an implementation prompt.

## Steps

1. Read the host limits in
   [`macos-observations.md`](macos-observations.md).
2. Read the material model in
   [`macos-materials.md`](macos-materials.md) and the glass system in
   [`macos-liquid-glass.md`](macos-liquid-glass.md).
3. Use a value only with its backdrop and its appearance; never lift one
   colour alone.
4. Treat an element as Liquid Glass only with lensing, specular or adaptive
   ink evidence; a blur plus a tint is general translucency.
5. Check the corrections section before trusting a value from an earlier
   research file.

---

## 1. Method

All values come from pixel samples of 2560x1440, scale 1.0 captures. The
backdrops were white, gray (`#a3a3a3` as rendered), black, checker (8 px
cells) and a white/black boundary (`edgeh`, `edgev`). The boundary backdrop
is the refraction test: a refracting surface moves the transition midpoint; a
pure blur only softens it.

Dark and light mode were measured back to back under the same wallpaper, and
the wallpaper was sampled before each pass so a wallpaper change cannot
contaminate the comparison. The 2026-11-19 pass at 1920x1080 was recorded
under a darker wallpaper; where the two passes disagree the wallpaper is the
likely cause and the disagreement is listed in section 9.

There is no 2x capture on this host, so one-pixel rims are not comparable at
Retina density. The lock screen was not captured. This host has no desktop
widgets; Notification Center only exposes its clock pill.

## 2. The material classes observed

| Class | Surfaces |
|---|---|
| Clear | menu bar, Spaces bar, Mission Control background |
| Scheme-adaptive glass | application menu panel, Control Center, Spotlight field, notification banner, Notification Center, Dock |
| Scheme solid | Dock under Reduce Transparency |
| Wallpaper blur | Launchpad background |
| Translucent chrome | window toolbar and sidebar |
| Opaque | window content |

## 3. The measured matrix, dark mode

| Element | White | Gray | Black | Checker | RT | Ink |
|---|---|---|---|---|---|---|
| Menu bar | sharp white | sharp gray | sharp black | sharp 8 px cells | unchanged | white glyphs |
| Application menu | `#a3a3a3` | `#92` to `#94` | `#2b2b2b` | `#8b` to `#8d` | unchanged: `#a3` over white | follows the panel |
| Control Center | `#a3` to `#b1` | `#a3a3a3` | `#262626` | `#7f7f7f` | `#b1` over white | follows the panel |
| Dock | `#bebebe` | `#8b8b8b` | `#0d` to `#10` | `#6c6c6c` | solid `#03` to `#0d` | n/a |
| Notification Center clock pill | `#bcbcbc` | not measured | `#5c5c5c` | `#9f9f9f` | not measured | n/a |
| Notification banner | not measured | not measured | not measured | `#86` to `#8c` | not measured | white |
| Spotlight field | not measured | not measured | not measured | `#8d8d8d` | not measured | follows the panel |
| Finder window title/toolbar/sidebar/content | n/a | n/a | n/a | n/a | n/a | `#282828` / `#2f2f2f` / `#454f57` / `#282828` |

## 4. The measured matrix, light mode

| Element | White | Gray | Black | Checker | RT | Ink |
|---|---|---|---|---|---|---|
| Menu bar | sharp white | sharp gray | sharp black | sharp 8 px cells | unchanged | dark glyphs |
| Application menu | `#f2f2f2` | `#d1d1d1` | `#75` to `#7b` | `#bf` to `#c6` | unchanged: `#f2` over white | follows the panel: dark on `#75` |
| Control Center | `#fbfbfb` | `#c7c7c7` | `#666666` | `#bababa` | not measured | follows the panel |
| Dock | `#ebebeb` | `#a6a6a6` | `#343434` | `#919191` | solid `#f1` to `#89` | n/a |
| Notification Center clock pill | `#ededed` | not measured | `#9e9e9e` | `#d2d2d2` | not measured | n/a |
| Notification banner | not measured | not measured | not measured | `#b3b3b3` | not measured | dark |
| Finder window title/toolbar/sidebar/content | n/a | n/a | n/a | n/a | n/a | `#fafafa` / `#dbdbdb` / `#efefef` / `#f5f5f5` |

The menu bar stays a sharp 8 px pattern in both modes: clear, not blurred.
The Mission Control background over the wallpaper is pixel-identical to the
desktop on this host: macOS 27.2 does not dim or blur it.

## 5. The model fit: one veil per surface per mode

Treating each glass as a white or black veil plus a brightness cap fits every
measured value within a few units. A veil is `behind.mix(color, alpha)`; the
cap clamps the resulting luminance. These are the target constants for the
compositor:

| Surface | Mode | Veil | Cap | Fit check |
|---|---|---|---|---|
| Application menu | dark | white `a=0.2` | `0.64` | white `0.64` vs `#a3` 0.64; black `0.17` vs `#2b` 0.17 |
| Application menu | light | white `a≈0.45` | `≈0.95` | black `0.45` vs `#75` 0.46; gray `0.80` vs `#d1` 0.82 |
| Control Center | dark | white `a≈0.15` | `0.64` | black `0.15` vs `#26` 0.149; checker `0.57` vs `#7f` 0.50 |
| Control Center | light | white `a≈0.4` | `0.98` | black `0.40` vs `#66` 0.40; gray `0.78` vs `#c7` 0.78 |
| Dock | dark | black `a≈0.15` | `0.75` | white `0.85` vs `#be` 0.75; black `0.85` vs `#0d` 0.05 |
| Dock | light | white `a≈0.4` | `0.92` | black `0.40` vs `#34` 0.20; checker `0.70` vs `#91` 0.57 |
| NC clock pill | dark | white `a≈0.35` | `0.74` | black `0.35` vs `#5c` 0.36; white `0.74` vs `#bc` 0.74 |
| NC clock pill | light | white `a≈0.6` | `0.93` | black `0.60` vs `#9e` 0.62; checker `0.80` vs `#d2` 0.82 |
| Banner | dark | white `a≈0.45` | `0.55` | checker `0.52` vs `#86` 0.53 |
| Banner | light | white `a≈0.6` | `0.72` | checker `0.70` vs `#b3` 0.70 |

The Dock veil is a black veil in dark mode and a white veil in light mode,
which is why the two columns do not share a curve. Reduce Transparency
replaces the Dock glass with the scheme solid: `#03` to `#0d` in dark mode,
`#f1` to `#89` in light mode; the menu and Control Center stay unchanged.

## 6. Blur strength per element

The checker is 8 px cells. A surface that shows a flat colour has destroyed
that frequency; a surface that shows a faint dither has not.

- **Menu panel:** flat with a residual dither of about 3/255. Strong blur.
- **Control Center:** flat, same family as the menu.
- **Spotlight and banner:** flat, the same family.
- **Notification Center clock pill:** flattens the checker; its tone curve is
  lighter than the menu in both modes.
- **Dock:** flat, stronger than the menu at the same backdrop.
- **Window toolbar and sidebar:** checker fully flattened.
- **Launchpad:** the wallpaper detail is gone and the luminance lifts about
  36/255 (measured on the 2026-11-19 pass; the 2026-11-20 pass could not
  reopen Launchpad).

## 7. Borders, shadows and edge zones

- The **menu panel** edge is 1 px, light over light backdrops and dark at the
  corners; its shadow reaches about 30 px.
- The **Dock** top edge is a 1 px light line over a 1 to 2 px dark line,
  followed by the glass; its bottom ramp is up to 13 px over a bright
  backdrop. Both modes share the structure; the line values follow the mode
  (`#88` light mode, `#74` dark mode at the top line).
- The **Control Center** bottom edge is a 2 px light line with a shadow that
  fades over about 30 px on white.
- A **window** has a 1 px dark outline plus a 1 px light bevel and a 10 to
  15 px shadow.
- The **menu corner radius** stays about 6.

### 7.1 The boundary test for refraction

A white/black boundary at a known position was measured inside each surface
in both modes. A refracting edge would move the midpoint; a pure blur only
softens it.

| Surface | Boundary | Sample point | Midpoint | Displacement |
|---|---|---|---|---|
| Dock dark | horizontal at y=1355 | x=1300 | y≈1355 | 0 px |
| Dock light | horizontal at y=1355 | x=1300 | y≈1354.5 | within 1 px |
| Dock dark, 2026-11-19 | horizontal at y=1030 | gaps at x=250, 316, 1200, 1500, 1730 | y=1030 | 0 px |
| Dock dark, 2026-11-19 | vertical at x=250 | y=1000, 1060 | x=250 | 0 px |
| Banner dark, 2026-11-19 | horizontal at y=80 | x=1700 | y=80 to 81 | within 1 px |
| NC dark, 2026-11-19 | horizontal at y=120 | x=1700 | y=120 | within 1 px |

The 2026-11-20 light banner run is contaminated by the banner's own bottom
shadow and is not listed. No surface moved the boundary at 1x in either mode.
The blur is the same material from the rim inward; the only edge-specific
rendering is the rim itself.

## 8. Liquid Glass or general blur

The boundary tests are the verdict for both modes. No surface shifted a
white/black boundary, neither in the center nor next to its borders. There is
no measurable lensing or refraction at 1x in dark or light mode.

What the surfaces do have is the rest of the Liquid Glass description: a
specular 1 px edge, a strong blur and adaptive ink. The honest classification
on this host is **adaptive blur and tint materials with specular edges,
without proven lensing**. Do not call any of them lensing glass in the code
or the docs.

## 9. Corrections to the 2026-11-19 research

Measured under the same wallpaper, the two appearances disagree with three
claims of the 1920x1080 pass:

- **The menu panel is scheme-dependent.** Dark mode caps the panel at
  `#a3a3a3` and floors it at `#2b2b2b`; light mode renders `#f2f2f2` over
  white and `#75` to `#7b` over black. The earlier "scheme-independent" claim
  was recorded under a dark wallpaper, which the light mode evidently tracks
  when the wallpaper is dark. There are two veil sets, not one.
- **The Dock is scheme-dependent, not wallpaper-only.** Same wallpaper, dark
  mode `#bebebe` over white versus light mode `#ebebeb` over white. The Dock
  follows the appearance like the code already assumes; the wallpaper claim
  was an artifact of the dark wallpaper pass.
- **Reduce Transparency makes the Dock a scheme solid.** `#03` to `#0d` in
  dark mode, `#f1` to `#89` in light mode. The menu and Control Center stay
  unchanged in both modes.
- **Mission Control has no background material on this host.** Unchanged.
- **Notification Center carries a light veil even in dark mode** (`#5c` over
  black, `#bc` over white), lighter than the menu in both modes.

The Dock, menu and Notification Center values of the 2026-11-19 pass
(`#8d` menu over white, `#7f` Dock over checker) belong to the darker
wallpaper state of that day; treat them as wallpaper variants, not as
constants.

## 10. Code gaps against the current kit and compositor

Closed on 2026-11-20 in `ctdesign` and `ctappearance`; the tests in
`ctappearance::resolve` hold every value below within 8/255.

- **The earlier claim here was wrong.** The old menu tokens (`#ffffff33`, lift
  `-0.032`, cap `0.6434`) reproduced the `#8d` of the 2026-11-19 pass, not the
  current `#a3`: white was 22/255 off. The caps of section 5 are in sRGB gamma
  units and the token `cap` is OKLab lightness, so the two are not pasted
  across.
- **The menu has two sets** in `components.shell.menu`, picked by
  `resolve::menu_glass` from the scheme. Dark and dark tint: `#ffffff41`, lift
  `-0.084`, cap `0.706`; over white, gray, black and the checker they give
  `#a0`, `#98`, `#2c` and `#87`. Light: `#ffffff76`, lift `0.012`, cap
  `0.968`; they give `#f2`, `#d1`, `#7a` and `#bf`. Chroma and spread are
  shared.
- `ctappearance::menu::dark_ink_over` at `0.4` works for both matrices: the
  light panel over black is `0.48` (dark ink) and the dark panel over black is
  `0.17` (light ink).
- **The Dock** keeps its light tone (`#e9`, `#a8`, `#33`, `#8f`). The dark tone
  is now lift `0.032` and cap `0.822` (`#be`, `#87`, `#13`, `#6e`).
- **The notification banner** rides the menu glass in dark (checker `#87`) and
  has its own light veil, `NOTIFICATION_GLASS_VEIL_LIGHT` `#ffffff5e`, with
  the light menu lift and cap (checker `#b3`).
- **Reduce Transparency:** the Dock is the scheme solid (`#0a` dark, `#f1`
  light) over any backdrop, and the menu glass keeps its veil and tone.
  Section 3 gives `#03` to `#0d` for the dark solid and section 5 gives `#f1`
  to `#89` for the light one; one solid cannot be `#f1` over white and `#89`
  over black, so the light solid stays `#f1` until a capture settles it.
- **bk menus under Reduce Transparency:** `theme::over_glass` keeps the menu
  clear under Reduce Transparency alone and returns the solid palette only
  with Increase Contrast, which was not measured. The compositor's blur and
  glass gate is a separate change.
- Not built: Control Center, Spotlight and the Notification Center clock pill
  (no consumer), the Dock blur strength and the removal of the lens pass. No
  lensing pass is justified by the measurements.

## 11. Open questions

- What are the rim and blur values at 2x? A sub-pixel refraction zone could
  exist at 1x without being measurable; Retina captures are needed.
- Does the light mode track a dark wallpaper and revert to the dark matrix?
  The 2026-11-19 pass suggests it; verify with the current light wallpaper in
  dark mode and a dark wallpaper in light mode.
- Does Notification Center change under Reduce Transparency?
- Does the Launchpad lift depend on the wallpaper luminance?
- Is the compact Control Center panel the default or a host state?

## 12. Evidence

The captures are local and not committed:

- `~/src/captures/d3-{white,gray,black,checker}-menu.png` and
  `d3-*-cc.png`: dark mode matrix under the current wallpaper.
- `~/src/captures/l2-{white,gray,black,checker}-menu.png`,
  `l2-*-cc.png`, `l3-white-cc.png`, `l3-black-cc.png`: light mode matrix.
- `~/src/captures/d3-dock-eh1355.png`, `l3-dock-eh1355.png`: the boundary
  refraction tests in both modes.
- `~/src/captures/d2-rt-*.png`, `l3-rt-*.png`: Reduce Transparency passes.
- `~/src/captures/d2-{white,black,checker}-nc.png`, `l3-*-nc.png`:
  Notification Center clock pill.
- `~/src/captures/d2-checker-banner.png`, `l3-checker-banner.png`,
  `d2-checker-spotlight.png`: banner and Spotlight.
- `~/src/captures/d2-finder.png`, `l3-finder.png`: window chrome.
- `~/src/captures/l3-wallcheck.png`, `d3-wallcheck.png`: the wallpaper
  controls proving the two passes share one wallpaper.
- `~/src/captures/defaults-2026-11-20-modes/`: the backup before this pass.
- `~/src/captures/blur-*.png`: the 2026-11-19 pass at 1920x1080.

## 13. Implementation prompt

Hand this to one subagent per task and keep the sequence.

```text
You are fixing the macOS glass model from measured values. Read this file
first, then the touched crates. Work in ~/src/barrierkit and
~/src/malaysia, main only: git pull --ff-only before editing and before
each push, one commit per session, English messages, no code comments.

Measured targets, dark mode / light mode (backdrop behind the panel):
- Application menu: white #a3a3a3 / #f2f2f2, gray #92-94 / #d1d1d1,
  black #2b2b2b / #75-7b, checker #8b-8d / #bf-c6.
- Control Center: white #a3-b1 / #fbfbfb, gray #a3 / #c7c7c7,
  black #262626 / #666666, checker #7f / #bababa.
- Dock: white #bebebe / #ebebeb, gray #8b / #a6a6a6, black #0d-10 / #34,
  checker #6c / #91. RT dock: solid #03-0d / #f1-89; menu and CC unchanged
  under RT.
- Ink: dark ink on any panel above 0.4 luma, light ink below. The light
  panel over black is 0.46, the dark panel over black is 0.17.

Task 1, barrierkit: add the light veil set for the menu in
crates/ctdesign/tengen/components.json and resolve.rs so a light scheme
renders #75-7b over black and #f2 over white. The existing dark
tokens reproduce the old `#8d`, not `#a3`: refit them too.

Task 2, barrierkit: make the Dock and Reduce Transparency targets match
section 5 of this file. The Dock already picks by scheme; replace the
light values with the measured ones and make the RT dock a scheme solid.

Task 3, malaysia: verify the compositor feeds the same values in
crates/malaysia-compositor/src/blur.rs and focus_notice.rs after the
barrierkit rev bump (Cargo.toml, Cargo.lock, flake.nix, flake.lock with
nix flake update).

Tests: a four-backdrop test per surface per scheme in resolve.rs asserting
the eight target colours within 8/255. A pure logic test for the ink
threshold: 0.46 takes dark ink, 0.17 takes light ink.

Definition of done: ./dev.sh check passes in both repos; the tests assert
the measured values; barrierkit main is pushed and malaysia pins the new
rev; the report lists the changed tokens, the model inputs and outputs,
and the commits.
```
