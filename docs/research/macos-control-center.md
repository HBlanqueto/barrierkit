# macOS Control Center

**Goal:** describe the Control Center panel that opens from the menu bar, so
BarrierKit can add an equivalent surface without guessing its frame.
**Scope:** the open panel, its module groups and its entry point in the menu
bar. It excludes the settings app, notification center and per-module
popovers.
**Reader:** anyone who adds a quick-settings panel to the shell.
**Input:** screenshots of three open states and one synthetic menu-bar click.
**Output:** measured frame, observed module layout, a Malaysia comparison
and ordered gaps.

## Steps

1. Read the limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the frame table for the panel.
3. Treat the module list as observed, not exhaustive.
4. Check the comparison before adding tokens.
5. Do not copy the system glyph artwork.

---

## 1. Entry point

Control Center has **no accessibility menu** on this host. A synthetic click
at `(2365, 15)` in the menu bar opened the panel. The click was delivered
through System Events and landed on the menu-bar agent, so the panel is owned
by a system agent rather than by the front application.

The panel opens directly below the menu bar and is anchored to the right
edge. It closes when focus leaves it, but the exact dismissal rule was not
measured.

## 2. Measured frame

| Property | Value |
|---|---:|
| Left | about 2250 |
| Right | about 2545 |
| Width | about 295 |
| Top | 30 |
| Bottom | about 565 |
| Height | about 535 |
| Right inset | about 15 |
| Corner radius | about 24 |
| Material | dark glass |

The top edge starts at the bar height, 30. The right inset is about 15 pt.
The panel is a tall single column, not a wide grid.

These values come from image inspection. Control Center exposes no
accessibility tree, so they are **inferred** at 1x.

## 3. Observed modules

The captured panel contains, from top to bottom:

- A connectivity group with Wi-Fi, Bluetooth and AirDrop.
- A media or now-playing group.
- A focus group.
- A screen-mirroring group.
- A display-brightness slider.
- A sound slider.

The exact grouping and the number of rows change with the system state. The
panel is a vertical set of module cards, each with its own rounded fill. The
inner cards are lighter than the panel background.

The panel uses the menu glass family. The 2026-11-20 pass sampled it over the
four backdrops: `#a3` over white, `#a3` over gray, `#26` over black and `#7f`
over checker in dark mode; `#fb` over white, `#c7` over gray, `#66` over
black and `#ba` over checker in light mode. Reduce Transparency leaves the
panel glass nearly unchanged (`#b1` over white in dark mode). The entry point
sits at x=2118 on the 2560x1440 display; the clock at x=2500 opens
Notification Center.

## 4. Comparison with Malaysia

Malaysia has no Control Center panel today.

| Element | macOS (inferred) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Menu-bar entry | Status item at right | Status items exist, no panel | `menubar/extras.rs` | No panel |
| Panel width | about 295 | Absent | — | New surface |
| Panel height | about 535 | Absent | — | New surface |
| Right inset | about 15 | Absent | — | New surface |
| Radius | about 24 | Absent | — | New surface |
| Material | dark glass | Menu and Dock glass exist | `menu_glass` | Reuse model |
| Connectivity group | Wi-Fi, Bluetooth, AirDrop | Individual status items | `menubar/status.rs` | No grouped control |
| Sliders | brightness, sound | Slider control exists | `bk` slider | No panel composition |
| Focus group | Present | No focus mode | — | New feature |
| Screen mirroring | Present | Absent | — | New feature |

The gap is a missing area, not a mismatch. Build it from existing controls
and the menu glass rather than from new primitives.

## 5. What not to copy

- Apple's Wi-Fi, Bluetooth, AirDrop, focus or mirroring glyphs.
- The module labels or their translations.
- Apple's private control-center agents.
- The exact spacing of Apple's module cards without measuring it.

Use project icons and the catalogue for labels.

## 6. Prioritized changes

1. **Add a panel with the measured frame.** Use a 295 pt width, a 535 pt
   height, a 15 pt right inset, a 30 pt top edge and a 24 pt radius. This
   gives a testable first version.
2. **Reuse the menu glass.** Control Center is another dark-glass surface.
   Do not create a third glass implementation.
3. **Start with four modules:** connectivity, media, brightness and sound.
   They cover the most common actions and exercise toggles and sliders.
4. **Add a menu-bar status item with a stable hit area.** The click target is
   the full bar height even when the icon is smaller.
5. **Add focus and mirroring later.** They need new state and are not
   required to validate the surface.
6. **Measure the module grid and gaps.** The current numbers describe the
   outer panel only.

## 7. Open questions

- What is the exact module card height and gap?
- Does the panel height grow with the module list or stay fixed?
- Does it use the same glass as the application menu or a lighter variant?
- What is the dismissal rule for a click inside another app?
- Does the panel have a light-mode variant with different rim values?
- The glass matrices for both modes are in
  [`macos-blur.md`](macos-blur.md); the panel shares the menu family with
  small offsets.

## 8. Implementation prompt

```text
You are fixing the Control Center glass values. Read this file and
macos-blur.md first. Work in ~/src/barrierkit and ~/src/malaysia, main
only: git pull --ff-only before editing and before each push, one commit
per session, English messages, no code comments.

Measured targets, dark / light:
- White: #a3 / #fbfbfb.
- Gray: #a3 / #c7c7c7.
- Black: #262626 / #666666.
- Checker: #7f7f7f / #bababa.
- RT: unchanged in dark mode (#b1 over white).

Tasks:
1. barrierkit: align the panel glass with the menu family offsets in
   macos-blur.md section 5; four-backdrop tests per scheme.
2. malaysia: after the rev bump, render the panel with the resolved glass.

Definition of done: ./dev.sh check passes in both repos; tests assert the
measured values; barrierkit main is pushed and malaysia pins the new rev.
Report changed tokens and commits.
```
