# How the macOS desktop was observed

**Goal:** record the host, the method and the limits behind every
`macos-*.md` file in this directory, so another agent can tell a measured
value from an estimate.
**Scope:** measurement only. It starts at the host setup and ends at the
evidence index. It does not propose design changes.
**Reader:** anyone who edits the macOS research files or ports a value from
them into `ctdesign`.
**Input:** a live macOS session and the probes in `~/src/probes`.
**Output:** this method note plus the per-area findings.

## Steps

1. Read the environment table and the limits before using any number.
2. Follow the evidence link of the area you need.
3. Check whether the value is marked **measured**, **inferred** or
   **estimated**.
4. Re-run a probe with the commands in "Reproduce a measurement" when a
   value matters.
5. Do not copy SF Pro, SF Symbols, Apple artwork or Apple application icons
   into the repository.

---

## 1. Environment

| Item | Value |
|---|---|
| macOS | 27.2.1, build 26B131 |
| Hardware | Apple M4 |
| Display | VG27WQ3B; 1920x1080 at 180 Hz in the 2026-11-19 pass, 2560x1440 in the 2026-11-20 pass |
| Backing scale | 1.0, so one point equals one pixel |
| Visible frame 2026-11-20 | 2560x1410 usable, menu bar 30, Dock top edge about y=1342 |
| Appearance | Dark and light measured in the 2026-11-20 pass |
| Accent | Default (system blue) |
| Reduce Motion | Off |
| Reduce Transparency | Off; the live switch is `com.apple.universalaccess` `reduceTransparency` |
| Increase Contrast | Off |
| Differentiate Without Color | Off |
| System language | Spanish |
| Session | Only display; locking it would block the host |

The Spanish locale changes menu titles (`Archivo`, `Edición`,
`Visualización`, `Ir`, `Ventana`, `Ayuda`). Geometric values do not depend on
the locale, but text widths do.

## 2. Method

All evidence comes from public surfaces:

- `screencapture` for still images and for video at about 40.43 fps.
- An AVFoundation recorder (`~/src/probes/record.swift`) for 60 fps video.
- `~/src/probes/frames.swift` to extract 60 fps frames.
- Accessibility queries through `osascript` and System Events.
- A Swift probe (`~/src/probes/metrics.swift`) that reads public AppKit
  values: `NSColor`, `NSFont`, `NSScreen`, `NSWindow`, `NSWorkspace` and
  `NSAnimationContext`.
- Pixel sampling with `~/src/probes/sample.swift` over `NSBitmapImageRep`. Since
  2026-11-22 it reads the raw bytes; before that its mid-tones were too light
  ([`macos-blur.md`](macos-blur.md#91-2026-11-21-mid-tones-read-with-sample-were-too-light)).
- A controlled backdrop (`~/src/probes/backdrop.swift`) with white, gray,
  black, checker and colour-band regions.

No decompilation, class-dump, private framework, asset extraction or Apple
source code was used. Menu items were clicked through System Events when the
area exposed no accessibility tree. Spotlight and Control Center have no
useful accessibility tree on this host, so their measurements are visual.

## 3. Measurement conventions

- **Points and pixels are equal** on this host because the scale is 1.0.
  A 14 pt traffic light is 14 px wide in a capture.
- **Screen coordinates** start at the top-left of the display for capture
  sampling. AppKit window frames in `metrics.txt` use the bottom-left origin
  and are labelled as such.
- **Colours** are sRGB hex plus alpha when the source exposes alpha.
  A pixel sample from a screenshot has no alpha; it is the composited result
  over that backdrop.
- **Glass values** are reported over four controlled backdrops, never over
  one wallpaper. A single value over a photograph is not reusable.
- **Inferred** means the geometry is consistent with several frames but was
  not read from an API. **Estimated** means it was not captured at all and
  comes from a public Apple description or a comparison with a sibling area.

## 4. Evidence index

| Area | Files under `~/src/captures` |
|---|---|
| Colours, fonts, screens, windows | `metrics.txt` |
| Default reset | `default-dock-gray.png`, `default-menu-*.png`, `default-finder-menu-*.png`, `light-off-*-dock.png`, `light-on-*-dock*.png`, `reduced-*-dock.png`, `defaults-2026-11-19/` |
| Menu bar | `menu-over-white.png`, `menu-over-gray.png`, `menu-over-black.png`, `menu-over-checker.png`, `menu-over-bands.png`, `menu-crop.png`, `menubar-right.png`, `dark-menu-file*.png`, `anim-menu.mov` |
| Dock | `dock-hover.png`, `dock-hover2.png`, `dock-hover3.png`, `dock-context.png`, `anim-dock*.mov` |
| Spotlight | `spotlight-empty.png`, `spotlight-query.png`, `spotlight-results.png` |
| Launchpad | `launchpad.png` to `launchpad4.png` |
| Control Center | `control-center.png` to `control-center3.png` |
| Notifications | `notification-banner.png`, `notification-center.png`, `notification-center2.png` |
| Mission Control | `mission-control.png` |
| Settings and focus | `settings-appearance.png`, `settings-focus-tab.png`, `settings-focus-tab2.png`, `focus-crop.png`, `focus-crop-4x.png` |
| Blur and Liquid Glass matrix | `blur-*.png`, `defaults-2026-11-19-blur/` |
| 2026-11-20 modes pass | `d2-*.png`, `d3-*.png` (dark), `l2-*.png`, `l3-*.png` (light), `defaults-2026-11-20-modes/` |
| Materials in windows | `materials/*-full.png` (raw device RGB), `materials/*.json` (frames, flags, layer trees), `materials/materials-cells.json`, `materials/control-*`; probes `materials.swift`, `matcontrols.swift`; [`macos-materials.md`](macos-materials.md#10-materials-in-windows-measured-2026-11-22) |
| Standard app menus | `appmenus/{en,es,en-cmd,es-cmd}-{main,inspector,text}.json`, `appmenus/show.py` (probe `appmenus.swift`, [`macos-standard-menus.md`](macos-standard-menus.md)) |

These files stay outside the repository. They are local evidence, not
committed assets.

## 5. Limits

- **The system compositor draws glass, blur, refraction, shadows and
  transparency.** AppKit and SwiftUI only describe them (an
  `NSVisualEffectView`, a glass container, a platter). So `cacheDisplay`
  shows glass areas empty, `screencapture` shows them, and Reduce
  Transparency changes no `NSColor`. A measurement of material or shadow
  belongs to the compositor path (malaysia); a measurement of geometry, ink
  and fills belongs to the kit.

- **One display, scale 1.0.** No 2x capture was possible. Text rasterisation
  and one-pixel rims cannot be compared at Retina density. The display moved
  between 2560x1440 and 1920x1080 across passes; each pass states its
  resolution.
- **The wallpaper changed between research passes.** The 2026-11-19 matrix
  was recorded under a darker wallpaper, the 2026-11-20 matrix under the
  current picture wallpaper. The modes pass samples the wallpaper before and
  after every switch so its two matrices are comparable.
- **The iTerm2 window covers the full screen below the menu bar.** Samples
  from open desktop regions in some 2026-11-20 captures read the terminal
  background, not the wallpaper. Wallpaper controls are taken at y=15 behind
  the clear bar.
- **Menus over a backdrop window need two clicks in this session.** The first
  click is consumed by the previous panel or menu state; the second opens the
  menu. Control Center and the clock open on one click.
- **A stuck notification alert blocked clicks in the middle of the pass.**
  `killall UserNotificationCenter` cleared it. If menu clicks stop working,
  check `~/src/probes/wins` for a `UserNotificationCenter` window first.
- **The F-keys do not trigger Mission Control or Launchpad** through System
  Events keystrokes on this host. Their 2026-11-19 values stay as measured;
  no 2026-11-20 re-run exists.
- **The appearance was reset and restored.** The Dock geometry, Reduce
  Transparency and light/dark switches were measured in both states and then
  returned to the original values. The default matrix is in
  [`macos-defaults.md`](macos-defaults.md).
- **The session locks when idle.** A locked session (`CGSSessionScreenIsLocked`) captures only the
  wallpaper: no windows, no menu bar, no lock-screen layer. `shellstate` reports it and the capture
  drivers refuse to run. Probes that post events do not keep it unlocked.
- **Lock screen was not captured.** Locking the only session would have
  stopped the host. Lock values are marked estimated.
- **Control Center and Spotlight have no usable AX tree.** Their panels were
  measured from images.
- **Notification Center opens empty on this host.** Only the clock pill and
  the surrounding glass are measurable; no notification cards or widgets.
- **Animation timing is incomplete.** The recorder has about 1.5 s of
  startup latency. Menu open, Dock magnification and notification motion
  need a second pass with a pre-rolled recorder.
- **Dock contents and tile size are host state.** The 2026-11-20 pass ran at
  `tilesize=72`, magnification off. Geometry from the reset pass uses 64.
- **Window metrics come from fresh `NSWindow` instances**, not from Finder,
  Safari or Settings. The style values are valid for AppKit; real apps can
  add toolbars and split views.
- **Spanish labels** inflate some widths. Measure again with English titles
  before fixing a menu width.

## 6. Reproduce a measurement

```sh
# Colours, fonts, screens, window styles.
~/src/probes/metrics.swift > ~/src/captures/metrics.txt

# A still capture.
~/src/probes/shot.sh ~/src/captures/name.png

# One pixel in a capture (path command x y; y starts at the top).
~/src/probes/sample ~/src/captures/name.png pixel 100 40

# A 60 fps recording.
~/src/probes/record ~/src/captures/name.mov 3 60
~/src/probes/frames ~/src/captures/name.mov ~/src/captures/name 60 0 180

# Move and click without stealing focus.
~/src/probes/mouse move 160 1350
~/src/probes/mouse click 2365 15
```

The probes for controls, tables, alerts, menus, toolbars and tabs, and the
comparison with the kit, are in [`appkit-comparison.md`](appkit-comparison.md).

`sample.swift` treats `y=0` as the top row. Do not reintroduce the old flip.
zsh does not word-split `set -- $pair`; pass coordinates as separate
arguments.

## 7. Global states pass

**Goal:** measure every desktop surface in each global system state with one
command, without two sessions changing a setting under each other.
**Scope:** from the user's command to the restored settings. It does not
cover what each capture batch measures.
**Reader:** the user who runs the pass and the sessions that add batches.
**Input:** a Mac unlocked, in the settings of section 1 (Dark, Reduce
Transparency off, Increase Contrast off, 2560x1440 at scale 1), and no other
session holding `/tmp/barrierkit-screen.lock`.
**Output:** the captures of every batch, one folder per state, and the
settings back as they were.

### Steps

1. Read the plan and the time: `~/src/probes/global-states.sh --dry-run all`.
   It changes nothing.
2. Run `~/src/probes/global-states.sh all` and answer `y`. Leave the Mac alone:
   the screen flickers, the Dock restarts, the display mode changes twice, and the keyboard and
   mouse belong to the probes.
3. Read the table the runner prints at the end. A state that did not apply, a batch that
   failed or timed out, and a batch that skipped itself are cells that stay `not measured`.
4. Check the restore: `~/src/probes/shellstate` must print the original
   state (`"appearance":"Dark"`, `"increaseContrast":false`,
   `"reduceTransparency":false`, `"scale":1`, 2560x1440).

### What the runner does

1. It waits for the screen lock, holds it for the whole run and writes `global-states` as its owner.
2. It saves the original settings in `~/src/probes/.backup-global-states.json`. It refuses
   to overwrite a backup that a previous run left.
3. For each state, it changes only the settings that differ from the live ones, waits for them to
   settle, checks them against `shellstate`, and runs `states.d/<state>/*.sh` in alphabetical order
   ([contract](#the-contract-of-a-batch)).
4. At the end, on `Ctrl-C`, on `kill` and on an error, it stops the running batch, restores every setting and releases the lock.

`list` prints the states and what each one sets.
`set <state>` applies one state and keeps the lock until `restore`; `run <state>` runs the batches of
one state in the current settings and refuses when they are not that state.

| Setting | Command | Source |
|---|---|---|
| Light or Dark | `osascript` to System Events, `appearance preferences`, `dark mode` | the public scripting dictionary |
| Reduce Transparency, Increase Contrast | `defaults write com.apple.universalaccess reduceTransparency` and `increaseContrast` | the live switches of section 1 |
| Dock position, size, magnification, auto-hide | `defaults write com.apple.dock orientation`, `tilesize`, `largesize`, `magnification`, `autohide`, then `killall Dock` | [`macos-defaults.md`](macos-defaults.md) section 1 |
| Wallpaper | `osascript` to System Events, `picture of every desktop`; solid `#eeeeee` and `#111111` images made on first use | the public scripting dictionary |
| Display mode (1280x720 points at 2560x1440 px) | `~/src/probes/displaymode`, CoreGraphics `CGConfigureDisplayWithDisplayMode`, for the session only | `displaymode.swift`; `displayplacer` is not installed and not needed |

The runner verifies each change with `shellstate` and the read-backs of the Dock keys,
the wallpaper and the display mode. If a switch does not take (the
`universalaccess` write is the likely one: it may need Full Disk Access for the terminal), it prints
what differs and asks you to set it in System Settings, or `s` to skip the state.
With `--yes` it skips instead of asking.

### If the pass is interrupted

1. `Ctrl-C` restores by itself. Wait for `restored:` and the `shellstate` line.
2. After a closed terminal, a `kill -9` or a power loss, run `~/src/probes/global-states.sh restore`.
   While `.backup-global-states.json` exists, `all` and `backup` refuse to start.
3. The display mode is set for the login session only. A logout or a restart returns it to the mode saved in System Settings.
4. If a setting stays wrong, `restore` says which and keeps the backup. Set it in System Settings, then run `restore` again.
   The wallpaper's original store is copied to `.backup-global-states.json.wallpaper-index.plist`.
5. If the session locks (`locked:true`), the run stops and restores. Unlock and run `restore` if it printed an error.

### The contract of a batch

Each session leaves its batches in `~/src/probes/states.d/<state>/NN-name.sh`. The variables, the exit codes, the time limit and the rule
to skip the lock are in `~/src/probes/states.d/README.md`.
