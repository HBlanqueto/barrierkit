# macOS corners: radius and curve of every rounded element

**Goal:** give the radius and the curve of every rounded element of macOS 27.2.1 as tokens, a reference mask and a parity check, so the kit and Malaysia draw the same corner.
**Scope:** windows and sheets, popovers, menus, tooltips, the Dock, banners, Notification Center widgets, Control Center modules, Spotlight, the controls inside windows and the concentric rule that sets an inner radius from an outer one. Measured 2026-11-22 at scale 1 on a 2560 x 1440 display, light and dark. It excludes scale 2, which needs the user's `hidpi-720` run (section 8).
**Reader:** whoever changes `ctdesign` corner tokens, `ctappearance::corner`, a window or popover shape in Malaysia, or a rounded control in the kit.
**Input:** the probes in `~/src/probes` (`cornerwin`, `cornerpop`, `cornerctl`, `cornerconc`, `cornerref`) and the analysis in `~/src/probes/corners`; captures in `~/src/captures/corners`; the white and black pairs of `~/src/captures/shellblur/dark`.
**Output:** the tables below, `components.corner` in `crates/ctdesign/tengen/components.json`, `crates/ctappearance/src/corner.rs` with its measured tiles, and [Corners](../barrierkit/04-appearance-and-tokens.md#corners).

## Steps

1. Read the provenance rules (section 1), then the curve (section 2): a radius is meaningless without it.
2. Find the element in section 3 (windows), 4 (menus, popovers, tooltips), 5 (shell) or 6 (controls).
3. For a nested shape read the concentric rule (section 7).
4. To compare a capture with the mask or with the compositor, follow the parity procedure (section 9).
5. Read what is not measured and the exact state it needs (section 8) before claiming a value.
6. For what the kit draws and why, read section 12.

## 1. Method and provenance

Every value says where it comes from:

- **API.** `NSView.effectiveCornerRadii` of the frame view of a probe window, or of a probe view that returns `NSViewCornerConfiguration.corners(radius: .containerConcentric)` from `cornerConfiguration` (public since macOS 27.0). It is the radius the system uses for concentricity.
- **Layer.** `layer.cornerRadius` and `cornerCurve` of a control's own layer tree, read in the probe's process (`cornerctl`). The pixels of that layer are the drawn corner.
- **Measured.** The alpha edge of a window capture (`screencapture -x -o -l <id>`, raw, never converted) or the transmission of a surface over a white and a black backdrop. A window capture of a window that has a child window (popover, menu, sheet) returns the parent with the child composited, so the probe's parent is a clear borderless window and the alpha is the child's shape.
- **Inferred.** Said as such.

A corner tile is the top-left block of the alpha, with the straight edges at `x = edge_x` and `y = edge_y` (a window on the pixel grid has 0; a popover has 0.3). Coverage of a pixel is the area inside the shape. `ctappearance::corner::coverage` computes it with 16 sub-rows; the Python in `~/src/probes/corners/cornerlib.py` computes the same numbers (checked to four decimals). The fit error is the rms and the maximum of `shown - measured` over the tile.

The diagonal of a rounded corner is 0.2929 of its radius, for the circle and for every smoothed corner, because the middle of a smoothed corner is a circular arc of the nominal radius. So the diagonal gives the radius of an element whose tail is not resolved (the shell surfaces, section 5), within 2%.

## 2. The curve

`cornerCurve = .circular` is the exact circle. `.continuous` is the smoothed corner that Figma and Freya call `cornerSmoothing`: two Bezier tails around a circular arc of the nominal radius, reaching `(1 + smoothing)` times the radius along each edge.

CoreAnimation layers with each curve, one window each, captured at 1x (`cornerwin ca-k-<r>`, `ca-c-<r>`):

| Layer | Best fit (radius, smoothing) | Row-mean rms | Tile rms and max with `smoothing` 0.745 |
|---|---|---|---|
| circular 16 | circle 16.00 | 0.016 (tile 28) | 0.0155 and 0.13 |
| continuous 8 | 7.99, 0.765 | 0.013 | 0.0114 and 0.093 |
| continuous 16 | 16.00, 0.746 | 0.025 | 0.0138 and 0.130 |
| continuous 26 | 26.01, 0.743 | 0.032 | 0.0117 and 0.122 |
| continuous 40 | 40.00, 0.745 | 0.075 | not in the tests |

The fitted radius is the nominal one to 0.01, and the smoothing is 0.745 at every radius: `components.corner.smoothing`. The same layer rendered by the CPU (`CALayer.render(in:)` and SwiftUI `ImageRenderer` at 8x, `cornerref`) has a smoothing of 0.61, so a `cacheDisplay` or `ImageRenderer` capture of a continuous corner is not the one on screen. The kit's `style::continuous_corner` uses 0.52866 (`WINDOW_CORNER.expansion - 1`), which is neither; see section 10.

The superellipse that Malaysia's shape shader draws (`window_sdf`: extent `R = expansion * r`, exponent `n`) is a good stand-in for the on-screen continuous corner at its nominal radius:

| Shape | rms and max of `R = 1.52866 r`, `n = 3.34` | Best `(R, n)` in that form |
|---|---|---|
| continuous 8 | 0.0077 and 0.089 | 1.59 r, 3.5 (0.0076) |
| continuous 16 | 0.0113 and 0.190 | 1.48 r, 3.2 (0.0105) |
| continuous 26 | 0.0121 and 0.184 | 1.48 r, 3.2 (0.0117) |
| continuous 40 | 0.0150 and 0.312 | 1.40 r, 3.0 (0.0136) |

`WINDOW_CORNER.expansion` (1.52866) and `.exponent` (3.34) are therefore right for a layer. They are wrong for a window (section 3).

## 3. Windows and sheets

### 3.1 Radius by kind

`effectiveCornerRadii` of the frame view (`NSThemeFrame`) is 16 on all four corners for every kind of window of `cornerwin`, and so is the alpha of the capture (28 captures of 17 kinds, four corners each, light and dark, active and inactive, 480 x 320 and 900 x 600: the largest difference between any two is 0.031 of coverage, 8 levels, at the bottom corners of the HUD panel):

| Kind | API radius | Alpha edge |
|---|---|---|
| titled: standard, fixed size, close only, document, panel | 16 | identical |
| `fullSizeContentView` with hidden title (`titleless`) | 16 | identical |
| toolbar: automatic, unified, expanded, unified compact, preference | 16 | identical |
| sidebar with and without a toolbar (`NSSplitViewController`) | 16 | identical |
| utility panel, resizable utility panel | 16 | identical |
| HUD panel (`.hudWindow`) | 16 | identical (8 levels at the bottom corners) |
| tabbed window | 16 | identical |
| sheet: plain, closable, over a clear parent (`beginSheet`) | 16 | identical (the shape is read against the veil) |
| real Finder window (toolbar and sidebar) and the Claude window | not read | identical |
| 480 x 320 and 900 x 600 content | 16 | identical |
| active and inactive, light and dark | 16 | identical (difference 0) |
| full screen | 0 on all corners | square |
| borderless | none | square |
| alert (`NSAlert.window`, frame view `NSGlassContentFrameView`) | **26** | not captured: the glass is invisible to a window capture |

No window kind of macOS 27.2.1 has the 20, 26 or 15 that `WindowCorner` held for a compact toolbar, a unified toolbar and a utility panel. A window of a toolbar style does not get a bigger radius. `components.window.corner.unified_compact`, `unified` and `utility` are now 16.

### 3.2 Silhouette

The alpha of the window is a continuous corner (smoothing 0.745) of radius **16.8**, not 16: the API radius is the one of the content, and the silhouette that the capture shows, with its edge, is 0.8 larger. The fit over the 32 x 32 tile of the tests has rms 0.0143 and maximum 0.116 at 16.8 (rms 0.0105 over 44 x 44); with the API radius 16 it is 0.037 and 0.32. The window's sheet has the same shape, read against the 60% veil: radius 16.8, rms 0.0057, maximum 0.112. Against a continuous layer of radius 16.5 captured the same way, 39 of the 51 edge pixels of the tile differ by more than 8 levels and the largest difference is 47 levels, in the first row and column; the free fit (16.6, smoothing 0.86) lowers the rms to 0.0083. This is a limit: the 2x capture should say what the first row is.

Row-mean edge `x(j)` of the corner (the same in the four corners of every window kind), pixels from the window's left edge for the rows `j = 0 ... 11`:

```text
16.00 10.05 7.90 6.42 5.26 4.28 3.47 2.73 2.14 1.73 1.37 0.95
```

`CORNER_WINDOW` is `{ radius: 16 (API), fitted: 16.8, smoothing: 0.745 }`. A compositor draws the window with `fitted`: `R = fitted * expansion = 25.7`, exponent 3.34: rms 0.013 and maximum 0.14 (with the API radius: 0.030 and 0.35).

## 4. Popovers, menus and tooltips

| Element | Radius | Edge offset | Fit | Provenance |
|---|---|---|---|---|
| `NSPopover`, 240 x 160, arrow below | 19.7 | 0.298 and 0.302 on every straight edge | rms 0.0083, max 0.185 | measured; the same on the four sides of the arrow, in light and dark, at 400 x 300 |
| `NSPopover` corners on the arrow's side | 20.2 | 0.067 | rms 0.023 | measured |
| `NSPopover`, 112 x 60 body | 13.4 to 14.2 | 0 | rms 0.08 | measured; the radius shrinks with the body: not modelled |
| `NSMenu.popUp`, 134 x 141 and 134 x 333 | 12.5 light, 13.3 dark | 0.004 and 0.059 | rms 0.043 (glass rim in the alpha) | measured; the diagonal gives 12.3 |
| Tooltip (`toolTip`), 57 x 20 to 252 x 62 | 5.2 | 0 | rms 0.005 to 0.008, circle or smoothed alike | measured from the captures of the tooltip session |

The arrow of an `NSPopover` (240 x 160, arrow above, captured as the alpha of its clear host, 2026-11-22) is fitted with a body edge at 13.33 from the top of the window, two straight sides, a round tip and a concave fillet (`~/src/probes/cornerpop popover-minY-240x160`; fit by sampling the shape at 16x and Nelder-Mead over the 16 row widths): apex 12.41 above the body, 26.17 wide at the body, tip radius 4.46, fillet 4.83, rms 0.4 px (the tokens 25.42 x 13, 4, 4 give rms 1.4 px). On macOS the kit shows a real `NSPopover`, so they are not tokens; `components.popover` stays what the compositor's shader reads.

The popover radius is 20 (`components.popover.radius`); a popover whose straight edges fall 0.3 off the pixel grid fits 19.7. The menu radius is 12.5 within 0.5, so `components.shell.menu.panel.radius` is 12 and `CORNER_MENU.fitted` 12.5. The same menu in the menu bar of the shell gives 12.3 to 12.6 (section 5).

## 5. The desktop shell

White and black backdrop pairs of `shellblur` (dark appearance, 1x). `D`, the mean over the three channels of the white capture minus the black one, is 255 outside a surface and its transmission inside. The 50% contour of `D` around a corner, referred to the straight edges of the same image, gives the diagonal and so the radius (`~/src/probes/corners/shellcorner3.py`, `dockcorners.py`). The four corners agree to 0.1.

| Surface | Size | Radius | Corners used | Note |
|---|---|---|---|---|
| Dock | 1117.6 x 79.4 | 21.9 (0.275 of the thickness; first run 21.5, 0.271) | four, the same | `components.shell.dock.radius` 0.272 stays |
| Notification banner | 346.3 x 58.2 | 20.3 | three (the close button covers the fourth) | `components.shell.notifications.radius` 18 to 20 |
| Notification Center panel | 346.4 wide | 20.4 to 20.5 | the top two | the bottom corners are cut by the screen |
| Control Center module | 294.3 x 64.0 | 24.1 to 24.3 | four | `CORNER_CONTROL_CENTER_MODULE` |
| Spotlight panel | 638.7 x 80.7 | 24.3 to 24.8 | the top two | the bottom corners meet the results list; `components.shell.launcher.corner-radius` 16 to 24.5 |
| App menu of the menu bar | 309.4 x 305.3 | 12.3 to 12.6 | four | the same shape as `NSMenu` |
| Apps panel (tile of the `apps` capture) | 851 x 600 | 31.6 | the top two | not identified further |

The smoothing of the shell glass is not measured at 1x (the tails are below the noise); `components.corner.smoothing` is applied by inference.

## 6. Controls

`cornerctl` builds every control in a window that is never shown and reads the API and the layer tree. The light and dark dumps are identical. Sizes are mini, small, regular, large and extra large. "Layer" means the drawn corner is the layer's radius; "API" means only `effectiveCornerRadii` (the control draws its bezel with a cell).

| Control | Radii | Curve | Source |
|---|---|---|---|
| Push button: every bezel that has layers (push, flexible push, automatic, accessory bar, push disclosure, glass) | 4, 5, 6, 14, 18 | continuous | layer |
| Circular button, `borderShape` circle | 6.5, 10, 12, 14, 18 | circular | layer |
| Help button | 8, 10, 12, 14, 18 | circular | layer |
| Disclosure | 4, 5, 6, 6.5, 6.5 | not drawn by a layer | API only |
| Small square | 4, 5, 6, 11, 11 | not drawn by a layer | API only |
| Toolbar bezel | 4, 5, 6, 11.5, 11.5 | not drawn by a layer | API only |
| Badge | 4, 5, 6, 9, 9 | not drawn by a layer | API only |
| `borderShape` capsule | drawn 6.5, 10, 12, 14, 18 (half the height); the API reports 4, 5, 6, 7, 9 | continuous | layer |
| `borderShape` rounded rectangle | drawn 4, 5, 6, 7, 9; the API reports 6.5, 10, 12, 14, 18 | continuous | layer |
| Pop-up and pull-down | 4, 5, 6, 14, 18 | continuous | layer |
| Text field (bezel) | 4, 5, 6, 7, 9 | continuous | layer |
| Search field | the same layers as the text field (4, 5, 6, 7, 9), but the field is drawn as a capsule | the capture `~/src/captures/texts/aqua-white-focus-search@4x.png` shows round ends at every size, so the layer is not the drawn shape | pixels |
| Combo box button | 2.5, 3.5, 4.5, 5.5, 6.5 | continuous | layer |
| Checkbox | 3.5, 4.5, 5.5, 6.5, 6.5 | continuous | layer |
| Radio button | 6, 7, 8, 9, 9 | continuous | layer |
| Segmented control, container (rounded, textured rounded, capsule, automatic) | API 8, 10, 12, 14, 18; the layer of the control is 4, 5, 6, 14, 18 in every style | continuous | layer (the API radius is not what is drawn: the 4x captures of the bench agree with the layer) |
| Segmented control, container (round rect, textured square, small square, separated) | 4, 5, 6, 14, 18 | continuous | API and layer |
| Slider track and knob | 2, 2, 3, 3, 3 and 6, 7, 8, 10, 10 | continuous | layer |
| Progress bar | 3, 3, 4, 4, 4 | circular | layer |
| `NSBox` primary | 12 | continuous | layer |
| `NSBox` custom | the `cornerRadius` property (8 in the probe) | circular | layer |
| `NSGlassEffectView` | the `cornerRadius` property, read back at 8, 16 and 26 | continuous | layer |
| Color well: default 9 and 12, expanded 6 and 8, minimal 12 | | continuous, circular on the 8 | layer |

The combo button is concentric to its field: 6 - 1.5 = 4.5 at regular. `components.combo-box.button-radius` holds 4 (3, 4, 5, 7 at the other sizes), which is within 0.5 of the layers; it was left. Switches and steppers did not resize with `controlSize` in the probe and are not tabulated. `components.corner.controls` holds the arrays above; the existing button, checkbox and segmented tokens agree with them.

## 7. The concentric rule

Read from `NSViewCornerRadius.containerConcentric(_:)` in several hundred combinations of container, inset and minimum (`cornerconc`), and checked in SwiftUI with `ConcentricRectangle` in `containerShape(RoundedRectangle(cornerRadius:style: .continuous))` rendered at 8x (radius fitted to 0.01):

```text
inner = max(container - min(inset_x, inset_y), minimum)   if inset_x <= container and inset_y <= container
inner = minimum                                           otherwise
```

- **The insets are those of the corner:** the distances from the child's corner to the two container edges next to it. The smaller one counts: a child that touches one edge keeps the container's radius, whatever the other inset is (16 container, `dx` 16, `dy` 0: 16).
- **The corner region ends at the container's radius, inclusive:** at inset 16 in a 16 container the child keeps 16 - 16 = 0; at 17 it has the minimum. A child 100 wide in the middle of a capsule has radius 0 at its right corners.
- **`minimum`** is the floor, and it applies outside the region too (`containerConcentric(6)` gives 6 at 17).
- **Containers nest:** a child's radius is the container radius of its own children. In a container of 20 a child at inset 0 has 20 and its child at inset 6 has 14; in a container of 26 a child at inset 2 has 24 and its child at inset 6 has 18.
- **A capsule container** has radius half its height (12 at 24, 18 at 36, 30 at 60); a child at inset 2 gets 10, 16 and 28 on the corners that touch the ends.
- **A fixed or per-corner configuration** (`.fixed(r)`, `.capsule`, `.capsule(maximumRadius:)`, `uniformEdges`) returns its radii as given, `capsule` being half the shorter side up to the maximum, and an unconfigured view has `effectiveCornerRadii == nil`.
- **SwiftUI** gives the same numbers: `ConcentricRectangle` in a continuous container of 20, 26 or 40 at inset 0, 4, 8, 12 and 16 is `R - d` to 0.01, and `.concentric(minimum: .fixed(m))` is `max(R - d, m)`.

`ctappearance::corner::concentric(container, inset_x, inset_y, minimum)` is the rule; `CORNER_CONCENTRIC_REGION` (1.0) is the width of the region as a share of the container's radius.

## 8. Not measured, and what each needs

| Cell | What it needs |
|---|---|
| Everything at scale 2 (first row of the window, the tails, the popover's small body, the menu's rim) | the user's run of `~/src/probes/global-states.sh run hidpi-720` (it runs `states.d/hidpi-720/50-corners.sh`: windows, sheet, layers, popovers and menus at 0.5 pt) |
| Dock at every size and position | the `dock-small`, `dock-large`, `dock-left`, `dock-right`, `dock-nomagnify` and `dock-autohide` states (their `30-dock-corners.sh` reads the shell blur captures of the same state) |
| Light, Reduce Transparency and Increase Contrast cells of popovers, menus and sheets | the `light`, `rt-on`, `ic-on` and `rt-ic-on` states (`50-corners.sh`) |
| The rendered shape of an alert and of other glass windows (the API says 26) | a capture that sees glass: a region of the screen over a white and a black backdrop, as `shellblur` does |
| Menu bar items and the open pill (`components.shell.menubar.title-open.radius` 12), Mission Control thumbnails and the Spaces bar, Launchpad and app icon masks, lock screen elements | the `bar`, `mc` and `apps` pairs of `shellblur`, with geometry files that the shell session has not written for them; the icon plate is `components.icon-plate.corner` (0.2245), measured by the icons session |
| Stage Manager strips | Stage Manager is not enabled; changing that is a global setting |
| Toolbar items (platter, capsule, group), tab bar of a tabbed window, large title | the glass platter radius is 18 and a capsule (control metrics); not captured in pixels |
| Table rows, cards, group boxes in pixels | the corner of a row selection is 7 (control metrics, the lists session); not remeasured |
| The context menu and submenus as separate shapes | a probe that opens them with the HID tap |
| Switch, stepper and color well at every control size | the probe's `sizeToFit` did not resize them |
| A continuous corner whose radius is more than a quarter of the shorter side (Freya shortens the tails by Figma's rule; the regular push button, 6 in 24, is exactly at the limit) | a layer of 12, 16 and 20 in a shape 24 to 36 high, captured at 1x and 2x |
| The tab view's pill and box (the layers say 4 and 6, the bench 5 and 5), the stepper bezel, the icon button, collection tile, image well, settings card, menu item highlight, date picker segments and token | a layer dump per element and a capture over white and black; the layers of the tab view do not say which is which |

## 9. Parity

1. Capture the element over the same backdrop on macOS (`screencapture -x -o -l <id>` of a window, never converted; `cornerwin`, `cornerpop`, or a pair for a surface) and over a clear borderless parent for a child window.
2. Cut the top-left tile of 32 to 48 pixels at the corner (`corners()` of `cornerlib.py`, `tile_of` of `fitpop.py`), flip the other corners to look like it, and read the edge offsets from the alpha of the straight edges.
3. Draw the same tile with `corner::tile(radius, smoothing, size, edge_x, edge_y)` with the token's `fitted` radius and `CORNER_SMOOTHING`, or with the `superellipse_coverage` of `corner.rs`'s tests for the compositor's shape, or take the compositor's output over the same backdrop.
4. Pass when the rms is at most 0.02 of coverage and the maximum at most 0.2 of coverage (51 levels). The tolerance is the worst fit of a measured tile (0.0155 and 0.185) plus the quantisation of 8 bits; the fits of section 3 and 4 are 0.0083 to 0.0155.
5. Fail when the nominal radius fits worse than the fitted one by a factor of two (the window: 0.037 against 0.0143): the unit tests `the_superellipse_with_the_api_radius_is_too_square_for_a_window` and `a_window_silhouette_is_the_continuous_corner_of_the_silhouette_radius` say so.

## 10. Tokens and the kit

`components.corner` holds `smoothing` (0.745), `concentric-region` (1.0), a `{ radius, fitted, smoothing }` per element (`window`, `sheet`, `alert`, `popover`, `menu`, `tooltip`, `notification`, `notification-center`, `control-center-module`, `spotlight`) and the control arrays. A `radius` that already has a token is that token (`components.popover.radius`, `components.shell.notifications.radius`, ...), so no value lives twice.

Existing tokens that changed, and why Malaysia's drawing changes with them:

| Token | Was | Is | Measurement |
|---|---|---|---|
| `components.window.corner.unified-compact` | 20 | 16 | every window kind is 16 |
| `components.window.corner.unified` | 26 | 16 | idem |
| `components.window.corner.utility` | 15 | 16 | idem |
| `components.dialog.radius` | `components.window.corner.unified` (26) | 26, a literal | the alert is 26; the value does not change |
| `components.shell.notifications.radius` | 18 | 20 | banner 20.3 |
| `components.shell.menu.panel.radius` | 14 | 12 | menu 12.5 |
| `components.shell.launcher.corner-radius` | 16 | 24.5 | Spotlight 24.5 |

`components.window.corner.expansion` (1.52866) and `.exponent` (3.34) did not change: they fit a continuous layer. `components.window.corner.titlebar` (`radius.lg`, 16), `components.popover.radius` (20), `components.tooltip.radius` (5.25) and `components.shell.dock.radius` (0.272) are confirmed within the fit error. The kit's `continuous_corner` smoothing was 0.52866, not the on-screen 0.745; it is `CORNER_SMOOTHING` since 2026-11-22 ([Corners](../barrierkit/04-appearance-and-tokens.md#corners), section 12).

## 11. Every radius token and what the measurements say

"Kit" is `crates/bk`, "Malaysia" the compositor and shell at `/Users/humbe/src/malaysia`. A token that is a reference to another (`radius.button`) has the value of its target.

| Token (under `components` unless said) | Value | Used by | Verdict |
|---|---|---|---|
| `radius.xs`, `sm`, `button`, `md`, `lg` (tokens.json) | 4, 8, 6, 12, 16 | the controls that name them: kit | `button` 6 is the regular push, text, stepper and segmented radius (API 6); `sm` 8 is the sidebar item, icon button, settings card and menu item; `lg` 16 is the window; the others are design scale |
| `button.radius`, `mini`, `small`, `large`, `xl` | 6, 4, 5, 14, 18 | kit `Button` | agrees with the push button layers (4, 5, 6, 14, 18) |
| `button.bezel.*.radius` (small-square 0, textured-rounded 10, textured-square 4.5, inline 9, push-disclosure 6) | | kit | cell-drawn bezels: the API reports 4, 5, 6 for the small square (not what is drawn), so these keep their 4x pixel measurements |
| `icon-button.radius` | 8 | kit | not measured here |
| `text-field.radius`, `search-field`, `combo-box.*button-radius` | 6; capsule; 2.5, 3.5, 4.5, 5.5, 6.5 (were 3, 4, 4, 5, 7) | kit | the text field is 4, 5, 6, 7, 9 by size (`CORNER_CONTROLS.text_field`); the search field is a capsule (section 6); the combo button tokens are the layers' (section 12) |
| `stepper.*radius`, `segmented.*radius`, `segmented.segment-radius` | 6, 4, 5, 7, 9; 6, 4, 5, 14, 18 | kit | segmented agrees; the stepper layer is 0.5 and the shape is drawn by a cell: not compared |
| `checkbox.*radius` | 5.5, 3.5, 4.5, 6.5, 6.5 | kit | agrees with the layers |
| `group-box.radius`, `list-group.radius`, `collection.radius`, `image-well.radius` | 12, 12.5, 10, 8 | kit | `NSBox` primary is 12 in the API (agrees); the others are not compared |
| `table.row-radius`, `disclosure.row-radius` | 7, 7 | kit | measured in the lists session (control metrics); not remeasured |
| `tab-view.strip-radius`, `small`, `mini`, `pill-radius`, `box-radius`; `toolbar-tabs.container.radius`, `tab.radius` | 6, 5, 4, 5, 5; 12, 8 | kit | measured in the tab session; not remeasured |
| `date-picker.day-radius`, `segment-radius`, `token-field.token-radius`, `color-picker.expanded-radius`, `wheel-radius`, `media.button-radius` | 4.5, 3, 2, 5, 6, 10 | kit | the color well layers are 6 (expanded) and 8: not compared |
| `icon-plate.corner` | 0.2245 of the plate | kit, Malaysia (`dock/mod.rs:1124`) | not remeasured (icons session) |
| `window.corner.titlebar` | 16 (`radius.lg`) | Malaysia `window_style.rs:509`; the kit's `WindowGlass` hole is `CORNER_WINDOW.fitted` | agrees (API); the silhouette is 16.8 |
| `window.corner.unified-compact`, `unified`, `utility` | 20, 26, 15 | Malaysia `window_style.rs:509-516` | **wrong**: 16, 16, 16 |
| `window.corner.expansion`, `exponent` | 1.52866, 3.34 | kit `style::continuous_corner`; Malaysia `window_style.rs:522`, `scene_elements.rs:492,532,766` | agree with an on-screen continuous layer (rms 0.008 to 0.015); a window needs `CORNER_WINDOW.fitted` |
| `window.resize-corner`, `client-resize-corner` | 16, 12 | Malaysia | hit areas, not shapes |
| `window.titlebar.rail.pill-radius`, `reveal-corner` | 13, 96 | kit | not measured |
| `window.sidebar.item.radius`, `tile-radius`; `menu-popover.item-radius`; `settings.*` radii | 8, 4.5; 8; various | kit | not measured here |
| `dialog.radius` | 26 (was a reference to `unified`) | Malaysia `scene_elements.rs:1325`; the kit reads `CORNER_SHEET` and `CORNER_ALERT` | agrees with the alert frame (API 26); a plain sheet is 16.8 |
| `popover.radius`, `arrow-tip-radius` | 20, 4 | Malaysia `window_style.rs:426-449`; the kit reads only the arrow band | the radius agrees (19.7 to 20.2); the tip is not a corner measurement |
| `tooltip.radius` | 5.25 | kit `tooltip.rs:141` | agrees (5.2) |
| `shell.menu.panel.radius` | 14 | Malaysia `window_style.rs:430`, `scene_elements.rs:1007` | **wrong**: 12 (12.5 measured) |
| `shell.notifications.radius` | 18 | Malaysia `notifications/view.rs` | **wrong**: 20 (20.3 measured) |
| `shell.live-activity.radius` | 20 | Malaysia `menubar/live.rs:180` | agrees with the Notification Center panel (20.5) |
| `shell.launcher.corner-radius`, `result-radius` | 16, 10 | Malaysia `launcher.rs:157,324` | the panel was **wrong**: 24.5; the result row is not measured |
| `shell.dock.radius` | 0.272 of the thickness | Malaysia `dock/surface.rs:50`, `dock/icon.rs:121` | agrees (0.271 to 0.275) |
| `shell.dock.magnify-radius`, `rim-corner-light`, `rim-corner-dark` | 3, rim stops | Malaysia | magnification falloff and rim colors, not corner radii |
| `shell.menubar.title-open.radius`, `dock.label-radius`, `label-arrow-tip-radius`, `menu-arrow-tip-radius` | 12, 12, 3, 6 | Malaysia | the open pill is not measured |
| `settings.wallpaper.crop.corner`, `corner-stroke`, `preview.*` | 16, 3, various | kit | not measured |

A plain sheet in the kit was drawn with radius 26, right for an `Alert` and 9.2 too round for a sheet, whose silhouette is 16.8; a `Sheet` is now `CORNER_SHEET.fitted` and an `Alert` `CORNER_ALERT.fitted` (section 12).

## 12. The kit

The kit (`crates/bk`) draws a corner through one function, `style::continuous` (`Shape::corner_radius` and `style::continuous_corner` call it), whose smoothing is `CORNER_SMOOTHING` (0.745). Decided 2026-11-22, [Corners](../barrierkit/04-appearance-and-tokens.md#corners).

**Which curve.** The on-screen 0.745 applies to a Skia path because the kit draws what the user sees: the window server's pixels (section 2). The 0.61 of a CPU render (`CALayer.render`, `ImageRenderer`) is a different renderer and not the look on screen. A control is `Continuous` where its layer is (section 6) and `Circular` where its layer is circular or nobody measured it. A capsule and a circle are the same in both curves. Freya shortens the tails when the radius is over a quarter of the shorter side; CoreAnimation's clamp is not measured (section 8).

**Skia against the reference.** Skia's anti-aliasing is not the 16-row box filter of `ctappearance::corner::coverage`. The kit's corner against the reference tile (scale 1, edges on the pixel grid): rms 0.007 to 0.019 and maximum 0.065 to 0.142 at radius 2.5, 4, 6, 7, 8, 12.5, 16.8 and 26 (the worst is radius 7). Against the five measured tiles it has rms 0.014 to 0.018 and maximum 0.149 to 0.220, so the check against a capture of the kit's own pixels is rms 0.02 and maximum 0.25. The old 0.52866 has rms 0.025 to 0.036 on the same tiles.

**Radius by control: the kit before this change, what macOS gives, and the kit now.** Sizes are mini, small, regular, large, extra large. "Capsule" is a radius of half the height, which the kit draws as a `Capsule`.

| Element | Kit before | Measured | Kit now |
|---|---|---|---|
| Push button, pop-up, pull-down | 4, 5, 6, capsule 14, capsule 18; circular | 4, 5, 6, 14, 18; continuous | same radii; continuous |
| Segmented control, toggle group | 4, 5, 6, capsule, capsule; circular | layers 4, 5, 6, 14, 18; continuous | same radii; continuous |
| Text field, secure field | 6 at every size; circular | 4, 5, 6, 7, 9; continuous | 4, 5, 6, 7, 9 (`TextFieldStyle::sized`); continuous |
| Token field | field 6; token 2; circular | field as the text field; token not measured | field as the text field, continuous; token 2, circular |
| Search field | capsule | capsule in pixels (layers 4 to 9: not what is drawn) | capsule |
| Combo box field | 6 at every size; circular | 4, 5, 6, 7, 9 | 4, 5, 6, 7, 9 (`ComboBox::size`); continuous |
| Combo box button | 3, 4, 4, 5, 7; circular | 2.5, 3.5, 4.5, 5.5, 6.5; continuous | 2.5, 3.5, 4.5, 5.5, 6.5; continuous |
| Checkbox | 3.5, 4.5, 5.5, 6.5, 6.5; circular | the same; continuous | same radii; continuous |
| Radio, round button, help button | circle | circles 12 to 36 across | circle |
| Slider track and knob, switch, progress bar | capsules | half the height at every size | capsules |
| Group box | 12; circular | `NSBox` primary 12; continuous | 12; continuous |
| Tab view strip, pill, box | 6 (5, 4 at small, mini), 5, 5; circular | layers 4 and 6, continuous; the pair is not assigned | same radii; continuous |
| Toolbar tab | container 12 (not measured), tab 8; circular | `NSGlassEffectView` 8; continuous | container 12 circular, tab 8 continuous |
| Sidebar item selection | 8; circular | 8, continuous tail (1x capture of the SwiftUI sidebar: rms 0.025 at 0.745, 0.028 at 0.529, 0.034 circular; radius 6 gives 0.084) | 8; continuous |
| Table, outline row selection | 7; continuous at 0.529 | 7 continuous (control metrics) | 7; continuous at 0.745 |
| Form card | 12.5; continuous at 0.529 | 12.5 continuous (control metrics) | 12.5; continuous at 0.745 |
| Sheet | 26; continuous at 0.529 | silhouette 16.8, continuous | 16.8 (`CORNER_SHEET.fitted`); continuous at 0.745 |
| Alert | 26; continuous at 0.529 | 26 (API), glass not captured | 26 (`CORNER_ALERT.fitted`); continuous at 0.745 |
| Window glass hole (`WindowGlass`) | 16 | silhouette 16.8 | 16.8 (`CORNER_WINDOW.fitted`) |
| Tooltip | 5.25; circular | 5.2 | unchanged: 0.05 is inside the fit error |
| Popover, menu, notifications, Dock, Spotlight | not drawn by the kit | section 4 and 5 | not drawn by the kit |
| Icon button 8, collection tile 10, image well 8, settings card 12, menu item 8, date picker segment 3 and day 4.5, stepper 6, media button 10, titlebar rail pill 13, color well expanded 5 | circular | not measured here | unchanged |

**The concentric rule in the kit.** `style::concentric(outer, inset)` applies `ctappearance::corner::concentric` to each corner with the same inset on both sides and a minimum of 0. It replaces `radius - inset` in two places: the well of a `ColorPicker` (the capsule bezel of 12 with the swatch at inset 3 gives 9, the layer of `NSColorWell` default) and the clip and inner border of an `ImageWell` (8 at inset 4 gives 4). Where a cell draws a nested shape and it was measured, the measurement stays: the tab pill (5 in a strip of 6, inset 2; the rule gives 4); the menu item (8 in a menu of 12 at padding 4) agrees with the rule. A sidebar item is not nested: the sidebar is flat (corner radius 0). A form card (12.5) with a `GroupBox` (12) or a sheet with content cards at an inset over the container's radius has `minimum` (outside the corner region), so each keeps its own measured radius.
