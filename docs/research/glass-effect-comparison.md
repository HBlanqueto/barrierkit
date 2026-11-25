# `glassEffect` against the kit: measured on macOS 27.2.1

**Goal:** know how far `.glass_effect(Glass, Shape)` ([Glass on any view](../barrierkit/15-backdrops-and-media.md#glass-on-any-view)) is from SwiftUI's `.glassEffect(_:in:)`, in light and dark, key and not key, nested, and under Reduce Transparency and Increase Contrast.
**Scope:** macOS arm64, 2560 x 1440 at scale 1, measured 2026-11-23. The kit's native path (`NSGlassEffectView` in a chrome, ADR 0100) and its Skia fallback. Not measured: scale 2, `Glass.tint`, `.interactive()`, `glassEffectUnion` (not built), macOS below 26.
**Reader:** whoever changes `glass_container.rs`, `StyleModifiers::glass_effect`, `platform/backdrop.rs` or the glass branch of `platform/macos/mod.rs`.
**Input:** a Mac in an unlocked session, `~/src/probes/glasseffect.swift` (SwiftUI twin), `~/src/probes/glass_probe.rs` (the kit's layouts), `~/src/probes/glasscmp/` (capture and pixel scripts, Python 3 standard library only).
**Output:** deltas per channel, what changed in the kit, and what stays different.

## Steps

1. Build the twin: `swiftc -O ~/src/probes/glasseffect.swift -o ~/src/probes/glasseffect`. Copy `glass_probe.rs` to `crates/bk/examples/` while you measure and build it with `cargo build -p bk --example glass_probe --example glass_effect`.
2. Light and dark are per process: the twin sets `NSAppearance`; the kit reads `color_scheme` from a temporary `MALAYSIA_CONFIG_DIR/appearance.domain`. Reduce Transparency and Increase Contrast are per process too (`-reduceTransparency 1`, `-increaseContrast 1` for the twin; `reduce_transparency = true`, `contrast = true` in the file for the kit). **No system setting changed**; `shellstate` read Reduce Transparency and Increase Contrast off before and after.
3. `python3 -I glasscmp/cap.py <dir> kind:layout:scheme:rt|ic|rtic|-:key|inactive ...` opens each app as a bundle, captures its window (`screencapture -l`, through the sRGB profile) and quits it. A `keeper` window makes the other one not key.
4. The layouts are the recipe (`crates/bk/examples/glass_effect.rs`), `bands` (flat rows of gray 0, 128, 255, red, green, blue, with empty 140 x 34 capsules), `shapes`, and `nested`. Read the pixels with `glasscmp/bands.py`, `analyze.py`, `full.py` and `profile.py` (raw PNG bytes, no colour conversion).

## 1. Like for like

**The earlier probe was not key.** A borderless `NSWindow` cannot become key, so the SwiftUI window of `glasscontainer` and of the first `glasseffect` drew the *inactive* glass (gray 128 gives 171 in light and 116 in clear dark, the inactive row of [macos-liquid-glass.md](macos-liquid-glass.md#41-state)). The twin now subclasses `NSWindow` with `canBecomeKey` true. Every number below is a key window unless it says inactive. The ceilings of `GlassContainer` in [appearance-matrix.md](appearance-matrix.md) were taken against that inactive twin; `glasscontainer.swift` has the same flaw.

Centre of a 140 x 34 capsule, R channel (the three channels agree), SwiftUI, kit, and kit minus SwiftUI:

| Backdrop | Variant | Light key | Dark key | Light inactive | Dark inactive |
|---|---|---|---|---|---|
| black | regular | 105, 105, 0 | 32, 32, 0 | 102, 102, 0 | 20, 20, 0 |
| gray 128 | regular | 180, 180, 0 | 133, 133, 0 | 171, 171, 0 | 127, 127, 0 |
| white | regular | 253, 253, 0 | 186, 186, 0 | 236, 236, 0 | 180, 180, 0 |
| red (213, 82, 81) | regular | (255, 144, 137), same, 0 | (235, 87, 88), same, 0 | (255, 128, 125), same, 0 | (229, 81, 81), same, 0 |
| black | clear | 32, 32, 0 | 32, 32, 0 | 71, 71, 0 | 25, 25, 0 |
| gray 128 | clear | 154, 154, 0 | 154, 154, 0 | 158, 158, 0 | 116, 116, 0 |
| white | clear | 255, 255, 0 | 255, 255, 0 | 244, 244, 0 | 207, 207, 0 |

- **Tone, tint, blur and lens: no residual.** The delta is 0 in every channel at every sampled point, and the profiles across the rim agree to the level: the outline is 1 px (backdrop 128 gives 64), the top edge goes 108 (ring), 191 (specular), 185, 182, 181, 179, 177 over about 8 px, the bottom edge has a 1 px highlight of 227 over a ring of 107. The kit's native path *is* the same filter, so there is nothing to tune. The clear glass is the same in light and dark when the window is key (black 32, white 255) and differs only when it is not.
- **Key and not key:** SwiftUI and the kit change together. Regular light over gray 128 goes 180 to 171, dark 133 to 127; clear dark 154 to 116, clear light 154 to 158.
- **Shapes** (`shapes`, mask of the pixels more than 3 levels from the backdrop): capsule, continuous rounded rectangles of radius 18 and 6, circle and rectangle have IoU 0.999 to 1.000 and the same bounding box within 1 px. Whole-window pixels differing by more than 3 levels: 0.0% in `bands`, 0.3% in `shapes` (the uneven shape), 1.6% in the recipe (the glyphs of `Text`).
- **Uneven shape: different.** SwiftUI follows `UnevenRoundedRectangle(4, 4, 22, 22)`: the two left corners are square-ish (256 of 256 pixels of the 16 x 16 corner square covered), the right ones round (192). The kit rounds all four to 22 (191, 192, 192, 191); IoU 0.965. `NSGlassEffectView` has `cornerRadius` and no shape ([header](https://developer.apple.com/documentation/appkit/nsglasseffectview)), so this stays.
- **Size: up to 1 px wider.** The kit rounds the frame out to whole logical pixels for the chrome (`floor`/`ceil` in `platform/backdrop.rs`) where SwiftUI keeps the fractional width of the `Text`: the capsules of the recipe are 80 px wide against 79.
- **Circle in a view that is not square: was different, fixed.** SwiftUI's `.circle` is the largest circle centered in the view; the kit drew a capsule of the whole frame (the "OK" badge, 49 x 46 against 46 x 45). `glass_effect(_, Shape::Circle)` now centers a square of the shorter side (`glass_frame` in `glass_container.rs`, both the native glass and the Skia one); the badge is within 1 px (bounding box x 30 to 76 against 30 to 75).

## 2. A glass inside another glass

SwiftUI composes them: each `glassEffect` is its own glass and the inner ones draw over the outer one (`nested`: a regular 300 x 180 glass of radius 28 with a clear capsule and two regular glasses of radius 10 inside, next to a clear one of radius 28).

- **Before:** a chrome held one radius and one variant (`Requests::add_backdrop`, as the wire of Malaysia does). The outer glass was requested and every inner one of another radius or variant was dropped without a fallback: the clear capsule and the two regular glasses did not exist, and their content drew straight on the outer glass. 12.5% (light) and 11.7% (dark) of the window differed by more than 3 levels, with deltas up to 213.
- **Now:** on macOS each glass of a chrome is its own native view with its own radius and variant (`Chrome::glasses`, `PlacedGlass`, the glass branch of `sync_chromes`); a chrome whose container has spacing 0 puts them in no `NSGlassEffectContainerView`, because a container merged and lost the third glass. 0.1% differ by more than 3 levels, 3.2% (light) and 1.7% (dark) by 1 to 3. The wire still holds one radius and one variant: Linux is unchanged, and only the macOS runner reads the per-glass list.
- **Fallback (Reduce Transparency, below macOS 26):** each `glass_effect` is its own container, so nested ones draw over each other. Under Reduce Transparency the regular glass is a flat white (light) or `#333333` (dark) with no outline, so a regular inside a regular is invisible in light (the third panel of `nested-light-rt-key-kit.png`); SwiftUI keeps a faint rim and the blurred rows under the outer glass.

## 3. Reduce Transparency and Increase Contrast

- **Found:** with the switch on, the kit's regular glass was the media solid (`MEDIA.material_tint`, 23, 23, 26) in light and dark. In light the label is black, so every label on a `glass_effect` was black on near black (deltas of -185 to -232 against SwiftUI, whose glass is near white). This is the setting that exists to improve legibility.
- **Fixed:** a regular `glass_effect` takes the toolbar's plain fill when the user reduces transparency (`resolved().toolgroup.bg`: `#ffffff` light, `#333333` dark, the `toolbar-glass-bg-reduced` token), as SwiftUI's glass does. A clear one keeps the dark solid.

Centre of the glass, regular, SwiftUI against the kit after the change:

| Backdrop | Light RT | Dark RT |
|---|---|---|
| black | 233 against 255 (+22) | 51 against 51 (0) |
| gray 128 | 248 against 255 (+7) | 51 against 51 (0) |
| white | 255 against 255 (0) | (60, 49, 49) against 51 (-9, +2, +2) |
| red | (255, 234, 219) against 255 (0, +21, +36) | (100, 34, 31) against 51 (-49, +17, +20) |
| green | (208, 255, 219) against 255 (+47, 0, +36) | (23, 59, 25) against 51 (+28, -8, +26) |

- **What stays different:** SwiftUI's Reduce Transparency glass still shows the backdrop through a blur of 50 px and a white or black fill, so it is tinted by what lies under it (up to 49 levels); the kit's is one flat colour. SwiftUI's clear is a dark plate that follows the backdrop (light: black 88, gray 76, white 64; dark: 24, 12, 0); the kit's is 23, 23, 26, 53 to 65 levels darker in light. Black text on it is illegible in both.
- **Increase Contrast:** the kit takes it as Reduce Transparency (`AppearanceConfig::transparency_reduced`), so it gets the same solid; SwiftUI keeps a translucent glass with a 0.45 white fill (light: black 185, gray 220, white 253; dark: 27, 40, 47). 25% of the `bands` window differs. Held as it is.

## 4. macOS below 26

Cannot run here. Evidence that the pre-26 path cannot call a missing symbol:

1. **Apple.** `NSGlassEffectView`, `NSGlassEffectContainerView` and `NSGlassEffectViewStyle` are `API_AVAILABLE(macos(26.0))` in the SDK header; `Glass` and `View.glassEffect(_:in:)` are `iOS 26.0, macOS 26.0` in `SwiftUICore.swiftinterface` ([Glass.clear](https://developer.apple.com/documentation/swiftui/glass/clear), [NSGlassEffectView](https://developer.apple.com/documentation/appkit/nsglasseffectview)). `effectIsInteractive` is macOS 27 and the kit does not call it.
2. **No link-time reference.** `nm -u target/debug/examples/glass_effect | grep NSGlass` prints nothing: objc2 looks classes up by name at run time. `LC_BUILD_VERSION` says `minos 11.0`.
3. **Run-time gate.** `platform/macos/blur.rs::glass_available()` is `AnyClass::get(c"NSGlassEffectView")` and `c"NSGlassEffectContainerView"`; `platform/macos/mod.rs` hands it to the app as `native_glass`. Without it `crate::glass::native_glass()` is false, `GlassContainer` never builds a `Declared::glass`, and `sync_chromes` never reaches `blur::glass` or `Glass::new`. The subclasses (`BKGlassView`, `BKGlassGroup`) register on their first use, which is inside that branch.
4. **What would run instead:** the Skia blob with the kit's blur or opaque solid (`glass_container.rs`).

## Not verified

- A Mac with macOS 25 or earlier; scale 2; `Glass.tint` and `.interactive()`; a wide-gamut display.
- Whether the per-process flags equal the system switches for this view: [macos-liquid-glass.md](macos-liquid-glass.md#134-other-surfaces-states-and-flags-read-2026-11-22) says they do for the glass filter; the system switch itself was not flipped.
- More than 8 glasses in a window: the ninth and later get no glass and no fallback (`MOST_CHROMES`, `bands` first drew 12).
