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
| Display | VG27WQ3B, 1920x1080, 180 Hz |
| Backing scale | 1.0, so one point equals one pixel |
| Visible frame | 1920x948, menu bar 30, Dock reserve 102 |
| Appearance | Dark |
| Accent | Default (system blue) |
| Reduce Motion | Off |
| Reduce Transparency | Off |
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
- Pixel sampling with `~/src/probes/sample.swift` over `NSBitmapImageRep`.
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

These files stay outside the repository. They are local evidence, not
committed assets.

## 5. Limits

- **One display, scale 1.0.** No 2x capture was possible. Text rasterisation
  and one-pixel rims cannot be compared at Retina density. The display changed
  from 2560x1440 to 1920x1080 between passes; the default measurements are at
  1920x1080.
- **The appearance was reset and restored.** The Dock geometry, Reduce
  Transparency and light/dark switches were measured in both states and then
  returned to the original values. The default matrix is in
  [`macos-defaults.md`](macos-defaults.md).
- **Lock screen was not captured.** Locking the only session would have
  stopped the host. Lock values are marked estimated.
- **Control Center and Spotlight have no usable AX tree.** Their panels were
  measured from images.
- **Notification Center did not open visibly.** Only the banner is measured.
- **Animation timing is incomplete.** The recorder has about 1.5 s of
  startup latency. Menu open, Dock magnification and notification motion
  need a second pass with a pre-rolled recorder.
- **Dock contents and tile size are host state.** This host has
  `tilesize=128`, `largesize=128` and magnification on. Those are defaults
  for this machine, not universal macOS defaults.
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

`sample.swift` treats `y=0` as the top row. Do not reintroduce the old flip.
zsh does not word-split `set -- $pair`; pass coordinates as separate
arguments.
