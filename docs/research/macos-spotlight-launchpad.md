# macOS Spotlight and Launchpad

**Goal:** separate the two launchers that macOS presents: Spotlight, a compact
search overlay, and Launchpad, an application grid.
**Scope:** the visible panels, their search fields, result rows, categories
and grid. It excludes indexing, search ranking and application contents.
**Reader:** anyone who changes `malaysia-shell::launcher` or adds a search
overlay.
**Input:** screenshots of Spotlight at three states and Launchpad at four
states.
**Output:** measured geometry, a Malaysia comparison and ordered gaps.

## Steps

1. Read the limits in
   [`macos-observations.md`](macos-observations.md).
2. Use the Spotlight table for the compact overlay.
3. Use the Launchpad table for the grid.
4. Do not merge the two models into one list.
5. Do not copy Apple application icons or search results.

---

## 1. Two different surfaces

macOS has two launchers with different jobs:

- **Spotlight** is a compact, keyboard-first overlay. It opens over the
  current window, accepts a query in a single field, and shows a short list
  of results with category tabs.
- **Launchpad** is a full application grid. It opens as a large panel over a
  blurred desktop, groups apps by page, and has no visible result list.

Malaysia's `launcher.rs` is a single search-result list. It is closer to
Spotlight than to Launchpad, but it is opened from the Dock's Apps item and
uses the launcher size, not a Spotlight overlay.

## 2. Spotlight measurements

The panel is approximately **625 pt wide** and starts around **y=435** on a
1440 pt display. It is horizontally centred.

| Property | Value |
|---|---:|
| Panel width | about 625 |
| Search field height | about 44 |
| Result row height | about 36 |
| Panel material | dark glass |
| Result layout | icon, title, secondary line |
| Category tabs | present above the results |
| Search text alignment | leading |
| Result selection | one row highlighted |

Observations:

- The field is the full panel width minus the panel padding.
- The result list is compact. A row is about 36 pt, smaller than a standard
  macOS menu row.
- Category tabs sit above the list. They are text tabs, not a segmented
  control with a filled track.
- The panel is dark glass and lets the window behind it show through. The
  2026-11-20 pass sampled the empty field over the checker: `#8d8d8d` in dark
  mode, the same family as the menu. The panel top edge sits at about y=408
  on the 2560x1440 display.
- The first launch shows an onboarding card. Typing dismisses it. Synthetic
  clicks on the onboarding button did not register, so the dismissal was
  done with the keyboard.
- Spotlight has no accessibility tree on this host. All values in this
  section are visual and marked **inferred**.
- The open, close and query-update animations were not measured. The panel
  appeared between captures.

## 3. Launchpad measurements

The panel is approximately **620x480**, centred on the display.

| Property | Value |
|---|---:|
| Panel width | about 620 |
| Panel height | about 480 |
| Title | `Apps` |
| Top-right control | ellipsis button |
| Category tabs | present above the grid |
| Grid | 7 columns by 5 rows |
| Icon | about 56 |
| Search field | not visible in this capture |
| Background | blurred desktop |

Observations:

- Launchpad is a **grid**, not a list. The captured page has 7 columns and
  5 rows.
- The title `Apps` is centred at the top. An ellipsis button sits at the
  top-right.
- Category tabs sit under the title.
- Icons are about 56 pt and have labels below them.
- The panel is much larger than Spotlight but is not full screen on this
  display.
- No search field was visible. Do not assume a search field exists in this
  version or state.
- Page dots and drag-to-page were not captured.

**Measured 2026-11-22 (dark, raw).** The Spotlight pill and the Apps panel (the Launchpad of macOS 27 is a floating glass panel of 864 x 606 px at
848, 431, not a full-screen overlay) have the glass of the application menu panel: 146 over white, 129 over gray 146, 32 over black, 122
over the checker; the blur is 10.5 px and 8.0 px plus a halo of 64 px. The rest of the screen is not dimmed or blurred. Tables, shadows and
errors: [section 14](macos-blur.md#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22). Two system permission dialogs sit over the middle of the screen (1075 to 1558 by 205 to 691), so Spotlight's
flat windows are in its left part (x under 1060).

## 4. Comparison with Malaysia

| Element | macOS (inferred) | Malaysia today | Token or file | Gap |
|---|---|---|---|---|
| Spotlight panel width | about 625 | Launcher is 560 | `LAUNCHER_WIDTH` | Too narrow for Spotlight |
| Spotlight height | content-driven, compact | 460 fixed | `LAUNCHER_HEIGHT` | Too tall for Spotlight |
| Search field | about 44 | 44 | `LAUNCHER_SEARCH_HEIGHT` | Matches |
| Result row | about 36 | 44 | `LAUNCHER_RESULT_HEIGHT` | Malaysia rows are taller |
| Result gap | not measured | 4 | `LAUNCHER_RESULT_GAP` | Unverified |
| Result icon | not measured | 32 | `LAUNCHER_ICON_SIZE` | Unverified |
| Panel radius | not measured | 16 | `LAUNCHER_CORNER_RADIUS` | Unverified |
| Max results | not counted | 8 | `LAUNCHER_MAX_RESULTS` | Unverified |
| Open duration | not measured | 180 ms | `LAUNCHER_OPEN_MS` | Unverified |
| Close duration | not measured | 120 ms | `LAUNCHER_CLOSE_MS` | Unverified |
| Open scale | not measured | 0.96 | `LAUNCHER_SCALE_FROM` | Unverified |
| Launchpad grid | 7x5, 56 icons | Absent | — | No grid mode |
| Launchpad title | `Apps` | Absent | — | No grid mode |
| Launchpad categories | present | Absent | — | No grid mode |
| Launchpad background | blurred desktop | Absent | — | No grid mode |

The largest gap is structural: Malaysia has one list surface, while macOS has
a compact search overlay and a separate application grid.

## 5. What not to copy

- Apple application icons and their labels.
- Spotlight's private index or ranking rules.
- Apple's category names, onboarding copy or translations.
- A screenshot of Spotlight or Launchpad as a background.
- The `Apps` title as a hard-coded English string. It belongs in the
  catalogue.

## 6. Prioritized changes

1. **Add a Spotlight-style compact mode.** Use a 625 pt panel, a 44 pt field
   and about 36 pt rows. Keep it keyboard-first and independent of the Dock
   launcher.
2. **Add a Launchpad-style grid mode.** Use a 7x5 grid, about 56 pt icons,
   a centred title, category tabs and page dots. This is a new view, not a
   reskin of the result list.
3. **Split the tokens.** `LAUNCHER_*` currently describes one surface. Add
   `SPOTLIGHT_*` and `LAUNCHPAD_*` tokens so the two modes can diverge.
4. **Match the search field height.** The existing 44 pt field already
   matches. Reuse it in both modes.
5. **Measure the open and close animations.** Neither Spotlight nor
   Launchpad has reliable timing yet.
6. **Keep the category tabs text-only.** Do not build a segmented control
   from the settings kit; the captured tabs are a different control.

## 7. Open questions

- Is Spotlight's panel width fixed at 625 or content-driven?
- Is the Spotlight result row exactly 36 pt at 2x?
- Does Launchpad have a search field in another state or version?
- How many columns and rows does Launchpad use at other display sizes?
- Does Launchpad scroll horizontally by page or by pixel?
- The Launchpad F-key does not fire through System Events on this host; the
  2026-11-19 lift value (about 36/255) stands until a working trigger is
  found.

## 8. Implementation prompt

```text
You are aligning the launcher surfaces with measured values. Read this
file and macos-blur.md first. Work in ~/src/barrierkit and
~/src/malaysia, main only: git pull --ff-only before editing and before
each push, one commit per session, English messages, no code comments.

Measured targets:
- Spotlight panel: about 625 wide, 44 pt field, about 36 pt rows, glass
  #8d over checker in dark mode, menu family.
- Launchpad: about 620x480, 7x5 grid, about 56 icons, blurred desktop
  with a lift of about 36/255.

Tasks:
1. barrierkit: split the launcher tokens into spotlight and launchpad
   sets; align the spotlight glass with the menu family.
2. malaysia: after the rev bump, use the spotlight tokens for the search
   overlay and keep the dock launcher list as is.

Do not build the Launchpad grid in this pass; it has its own prompt.

Definition of done: ./dev.sh check passes in both repos; main is pushed
and malaysia pins the new rev. Report changed tokens and commits.
```
