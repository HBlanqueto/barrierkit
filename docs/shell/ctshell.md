# ctshell

**Goal:** read and write the desktop's own settings, and draw its surfaces
with the measured glass and the shell tokens.
**Scope:** `ctshell`, the crate without Freya that Malaysia's compositor,
portal, System Settings and shell share: the wallpaper, menu bar, Dock and
notification settings, the glass of the desktop surfaces and
`ctshell::design`. The user's appearance it builds on is in
[4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md#user-appearance-ctappearance).
**Reader:** whoever works on Malaysia's compositor, portal, System Settings or
shell.
**Input:** the files in `ctappearance::config_dir()` and `data_dir()`, and the
Tengen JSON. **Output:** typed settings, the colours a glass shows and the
shell's constants.

## Steps

1. Depend on `ctshell`, beside `ctappearance` and `ctdesign`:
   `ctshell = { git = "https://github.com/HBlanqueto/barrierkit", rev = "…" }`.
2. Read a settings file with its function or with `toml`: the
   [wallpaper files](#the-wallpaper-files), [`menubar.domain`](#menu-bar),
   [`dock.domain`](#dock) or the [notification settings](#notifications).
   Write it back with the writer of the same module, or with
   `ctappearance::replace_file`.
3. Take the colours of a shell surface from the resolved appearance
   ([The shell colors of `Resolved`](#the-shell-colors-of-resolved)) and draw
   its glass with [Glass over a backdrop](#glass-over-a-backdrop),
   [Shell glass](#shell-glass) or [Liquid Glass](#liquid-glass).
4. Size it with [`ctshell::design`](#tokens-ctshelldesign).
5. Run `./dev.sh check`: `scripts/check-client-only-deps.sh` fails if
   `ctshell` pulls Freya or Skia.

## The wallpaper files

Malaysia's wallpaper lives in three files. System Settings writes the first two
and `malaysia-wallpaper` reads them and writes the third. `ctshell::wallpaper`
holds the first two and the wallpaper catalog; the third is `ctappearance`'s
([4](../barrierkit/04-appearance-and-tokens.md#reading-and-writing)).

| File | Functions | Contents |
|---|---|---|
| `data_dir()/wallpaper-selection.state` | `wallpaper_selection_path`, `read_wallpaper_selection`, `write_wallpaper_selection` | `WallpaperSelection`: what each desktop shows and how |
| `config_dir()/wallpaper.domain` | `wallpaper_config_path`, `read_wallpaper_config`, `write_wallpaper_config` | `WallpaperConfig`: the user's collections |
| `data_dir()/wallpaper.state` | `wallpaper_state_path`, `load_wallpaper_state`, `write_wallpaper_state` | `WallpaperState`: dominant color, tint grids, outputs and the shuffle |

Every map of the selection is keyed by `desktop_key`: a desktop number from 1,
or `ALL_DESKTOPS` (`"default"`) for every desktop. A desktop without its own key
shows the shared one with its scale, crop, appearance and pause. Crops can also be
keyed `"<desktop>/<output>"`, which wins over the desktop's own.

| Field | Holds |
|---|---|
| `desktops` | A `WallpaperSource` as text: `builtin`, a `file://` URI, `shuffle:colors` or `shuffle:collection:<name>`. No shared key shows the built-in wallpaper. |
| `scale`, `scales` | `WallpaperScale` of the shared wallpaper and of each desktop's own: `Fill`, `Fit`, `Stretch`, `Center` or `Custom`. |
| `crop` | `[x, y, width, height]`, fractions of the image. With `Custom` it covers the screen like `Fill`. |
| `appearance` | `WallpaperAppearance` (`Light` or `Dark`): the still a dynamic wallpaper keeps instead of following the system appearance or the time. |
| `paused` | Desktop keys whose animated wallpaper stands still. |
| `shuffle` | `WallpaperShuffle`: `Login`, `Hourly` or `Daily`. |

Edit it through its methods so the keys stay consistent: `set_source`,
`make_own`, `set_scale`, `set_crop`, `set_appearance` and `set_paused` write;
`source`, `scale_for`, `crop_for`, `appearance_for` and `paused_for` read with the fallbacks above.
`set_source(None, _)` sets the wallpaper of every desktop: it drops each
desktop's own choices. `replace_source(old, new)` rewrites only the desktops
that show `old`; with `None` they lose that wallpaper. `make_own(desktop)` gives a desktop a copy of the shared
choices, and `show_on_all_desktops(desktop)` makes a desktop's choices the
shared ones, like macOS «Show on all Spaces».

```toml
scale = "custom"
shuffle = "daily"

[desktops]
default = "file:///run/current-system/sw/share/wallpapers/bandura.jpg"
2 = "shuffle:collection:Viajes"

[crop]
default = [0.2, 0.15, 0.55, 0.62]
"default/HDMI-A-1" = [0.3, 0.1, 0.5, 0.62]
```

`WallpaperKind::of` and `wallpapers_in` decide which files count as
wallpapers: PNG, JPEG, HEIC and HEIF images, MP4, MOV, WebM and MKV videos, and
GLSL shaders (`.frag`, `.glsl`). `system_wallpapers_dir` is the first
`$XDG_DATA_DIRS/wallpapers` and `custom_colors_dir` holds the user's own colors.
`wallpaper_colors` lists the colors: the system's `solid-colors`, then the
user's.

Previews follow the [thumbnail spec](https://specifications.freedesktop.org/thumbnail-spec/latest/):
`thumbnail_path` is the x-large PNG named by the MD5 of `file_uri` (escaped as
GLib does), valid while `thumbnail_is_fresh`. `thumbnail_image_size` reads the
size of the original image from its `Thumb::Image::Width` and `Height`. `frame_path` holds the light and the
dark still of a dynamic wallpaper; System Settings treats a HEIC with both as
dynamic.

## Menu bar

`ctshell::menubar::MenubarConfig` is `menubar.domain`: `app_menus` (default `true`)
shows the active app's own menus. Malaysia's compositor and menu bar both read
it. `show_wifi`, `show_bluetooth`, `show_battery`, `show_battery_percentage`
and `show_vpn` (all `true`) choose the status menus of the bar; System
Settings writes them. `show_printer` (`true`) shows a printer while a print job
is queued, on the bar and on the lock screen.

`style` is a `MenubarStyle`: `solid` (Classic), `translucent` or `floating` (default: a clear bar, with only the open title pill blurred).
`menubar::classic_bar(theme_bar, reduce_transparency)` returns the fill of the Classic
bar: the theme's `shell-bar` at `MENUBAR_CLASSIC_ALPHA`, or opaque with
reduced transparency.
`bar_height` is a `MenubarHeight`: `slim` (default, `MENUBAR_HEIGHT`) or
`wide` (`MENUBAR_HEIGHT_WIDE`). `MenubarConfig::height()` returns the height
in logical pixels; the bar and the lock screen both use it. An unknown value
falls back to the default. `menubar::colors_over(wallpaper)` returns the
`MenubarColors` of a translucent or floating bar over that wallpaper color:
`ink`, `glass` and `selection`, the translucent pill behind an open menu
title. It takes the `MENUBAR_OVER_LIGHT_*` set when the wallpaper, darkened by the
glass, is brighter than `MENUBAR_DARK_INK_ABOVE` (relative luminance) and
`MENUBAR_OVER_DARK_*` otherwise, whatever the color scheme.
`ctappearance::tinting::WallpaperGrid::across` returns the wallpaper color along the bar, blurred
horizontally by `MENUBAR_TINT_SPREAD` of the bar width.

## Dock

`ctshell::dock::DockConfig` is `dock.domain`. Its keys are those of
`defaults write com.apple.dock`, except `show-downloads`: macOS keeps the
Downloads folder in `persistent-others`, and the Dock turns this key off when
the folder is dragged out.

| Key | Type | Default |
|---|---|---|
| `autohide` | bool | `false` |
| `orientation` | `DockOrientation`: `left`, `bottom`, `right` | `bottom` |
| `magnification` | bool | `false` |
| `tilesize` | integer | `DOCK_TILESIZE` |
| `largesize` | integer | `DOCK_LARGESIZE` |
| `mineffect` | `MinimizeEffect`: `genie`, `scale`, `suck` | `genie` |
| `show-process-indicators` | bool | `true` |
| `static-only` | bool | `false` |
| `minimize-to-application` | bool | `false` |
| `show-recents` | bool | `true` |
| `show-downloads` | bool | `true` |
| `launchanim` | bool | `true` |
| `autohide-delay` | seconds | `0.0` |
| `autohide-time-modifier` | factor | `1.0` |

`DockConfig::tilesize()` clamps `tilesize` to `DOCK_TILESIZE_MIN` and
`DOCK_TILESIZE_MAX`. `largesize()` is the magnified size: at least
`DOCK_MAGNIFY_MIN` times `tilesize()`, so icons still grow at the largest size,
and equal to `tilesize()` without magnification. An unknown orientation or
effect falls back to the default.

## Notifications

`notifications::NotificationsConfig` is the TOML form of the desktop's
notification settings. A missing key takes its default; an invalid enum value
takes its default with a warning.

| Field | Type | Default |
|---|---|---|
| `show_previews` | `ShowPreviews`: `always`, `when-unlocked`, `never` | `always` |
| `when_locked` | bool | `true` |
| `apps` | map of app id to `AppNotifications` | empty |
| `focus` | map of `Focus` (`do-not-disturb`, `sleep`, `personal`, `work`) to `FocusSettings` | empty |
| `widgets` | list of `Widget`: `calendar`, `batteries` | both |

`NotificationsConfig::app(id)` and `focus(focus)` return the entry or its
default; `app_mut` and `focus_mut` insert the default first.

| `AppNotifications` field | Type | Default |
|---|---|---|
| `allow`, `desktop`, `center`, `lock_screen` | bool | `true` |
| `style` | `AlertStyle`: `temporary`, `persistent` | `temporary` |
| `critical` | bool | `false` |
| `time_sensitive`, `sound`, `badge` | bool | `true` |
| `grouping` | `Grouping`: `automatic`, `off` | `automatic` |
| `muted_until` | `Option<i64>` | absent |

`AppNotifications::muted(now)` is true while `muted_until` is after `now`.
`FocusSettings` has `allowed_apps` (empty) and `time_sensitive` (`true`).

## The shell colors of `Resolved`

`ctappearance::resolve` fills these fields of `Resolved` for the shell and the
compositor; the rest of `Resolved` is in
[4](../barrierkit/04-appearance-and-tokens.md#resolution). Malaysia passes
`false` as `native_lights`.

| `Resolved` field | What it holds |
|---|---|
| `bar_tint` | Tint of the menu bar and the Dock with the `bars` part: the accent tint, or the dominant color by `wallpaper_tinting`. |
| `shell` | `ShellPalette`: menu bar and Dock colors. |
| `dock` | `DockSurface` of the Dock: its glass, outline, rim and tone (`DOCK_TONE_LIGHT` or `DOCK_TONE_DARK`) by scheme. With transparency reduced, the scheme's solid and no tone. |
| `menu_glass` | `DockSurface` of the menus: the `_LIGHT` veil, lift and cap of `MENU_GLASS_*` in the light scheme, the `_DARK` ones in dark and dark tint. Transparency does not change its veil or tone. |
| `notification_glass` | `DockSurface` of the notifications: the tone and blur of `menu_glass` in the Dock's frame, with the menus' dark veil in dark and `NOTIFICATION_GLASS_VEIL_LIGHT` in light. With transparency reduced, `dock`. |

## Glass over a backdrop

`ctshell::menu::panel_over(&surface, behind)` returns the colour a
`DockSurface` shows over `behind`, as the compositor's shader computes it. The
Dock, the menus and the notifications share it:

1. The veil mixes over the backdrop at its alpha.
2. The tone adds `lift` to the lightness of the result, in OKLab.
3. A soft minimum keeps that lightness under `cap`.
4. The backdrop's hue comes back, saturated by `chroma` and scaled down by the
   lightness the cap removed.

A surface without a tone is its veil alone: the Dock with transparency reduced.

Each surface takes its numbers from the scheme:

| Surface | Light | Dark and dark tint |
|---|---|---|
| `menu_glass` | `MENU_GLASS_VEIL_LIGHT`, `MENU_GLASS_LIFT_LIGHT`, `MENU_GLASS_CAP_LIGHT` | `MENU_GLASS_VEIL_DARK`, `MENU_GLASS_LIFT_DARK`, `MENU_GLASS_CAP_DARK` |
| `notification_glass` | `NOTIFICATION_GLASS_VEIL_LIGHT` with the light lift and cap of the menus | the dark veil, lift and cap of the menus |
| `dock` | the theme's `DOCK_GLASS` with `DOCK_TONE_LIGHT` | the theme's `DOCK_GLASS` with `DOCK_TONE_DARK` |

`MENU_GLASS_CHROMA` and `MENU_GLASS_SPREAD` are the same in every scheme. With
transparency reduced the Dock is the scheme's solid (`DOCK_GLASS_SOLID`) over
any backdrop and the notifications take it; the menu glass keeps its veil and
tone.

The ink a panel takes is in
[4](../barrierkit/04-appearance-and-tokens.md#glass-over-a-backdrop).

## Shell glass

`design::ShellGlass` is the glass of one desktop surface in one scheme, as macOS 27.2.1 composes it, and
`ctshell::shell_glass` is its reference function ([the measurements](../research/macos-blur.md#14-desktop-surfaces-as-a-pipeline-measured-2026-11-22)). The tokens are
`components.shell-glass.<surface>.<scheme>`; today `menu` (also the banner), `spotlight`, `apps`, `notification-card`, `control-center`,
`control-center-panel` and `dock` (glass), and `spaces-bar` (a `MaterialLook`: use `ctappearance::material::over`), dark only. Light, Reduce
Transparency and Increase Contrast have no token: they were not measured.
A veil and a cap per surface miss the menu and the Dock by over 6 levels; this pipeline misses by 1.7 at most.

1. The backdrop is blurred by `sigma` px, plus `halo_weight` of a wide `halo_sigma` px blur when the weight is not 0
   (the compositor can get the same sigma from a Gaussian on a downsampled backdrop, or from a Dual Kawase of
   `kawase_sigma(passes, offset)`).
2. `over(glass, blurred)` takes the blurred pixel in the display's device values, the bytes the compositor writes on the measured
   panel, and returns `tone(Y) + chroma(Y) (c - Y)` per channel: `Y` is its Rec. 709 luma, `tone` the 12-point gray response
   (`SHELL_GLASS_LEVELS`), `chroma` the gain of the colour difference at five luma knots (`SHELL_GLASS_CHROMA_KNOTS`). It does not clamp.

The menu bar needs no token: it is identical to the backdrop. `MENU_GLASS_*`, `DOCK_TONE_*`, `DOCK_GLASS_*` and
`NOTIFICATION_GLASS_VEIL_LIGHT` were fitted to the shifted numbers of
[section 9.1](../research/macos-blur.md#91-2026-11-21-mid-tones-read-with-sample-were-too-light) and are superseded by `SHELL_GLASS_*`;
they stay until malaysia moves.

## Panel glass per scheme

`ctshell::resolve` reads the per-scheme values of `ctdesign::GLASS`
([4](../barrierkit/04-appearance-and-tokens.md#view-metrics)) for the
compositor, which draws the sidebar's glass panel
([Glass](../barrierkit/10-sidebar.md#glass)). `ctappearance::glass_chroma`
gives the chroma.

| Function | Returns |
|---|---|
| `glass_lightness(scheme)` | The share of the lightness behind the glass that shows through |
| `glass_rim_lift(scheme)` | How much lighter the panel's border is than the fill under it, and how much more where the wallpaper shows |
| `glass_blur_passes(scheme)` | How many times the glass blurs what is behind it |
| `glass_edge(scheme) -> GlassEdge` | The wallpaper along the left, top and bottom edges: `intensity`, `span`, `chroma`, `lightness`, `cap` and `offset` |
| `glass_shadow(scheme) -> Option<GlassShadow>` | The panel's shadow on the window: `opacity`, `span`, `tint`, `lightness`; `None` in dark |

`material::blur_region(line, sigma)` is the Gaussian over one line of a region
that sees only itself: the weights are renormalised at the border, so nothing
outside the region reaches the pixels inside, as macOS blurs with
`inputNormalizeEdges`.

## Traffic-light group

`ctshell::traffic` adds two items to the geometry of
`ctappearance::traffic` ([4](../barrierkit/04-appearance-and-tokens.md#traffic-light-geometry))
for the compositor's server-side title bar:

| Item | What it does |
|---|---|
| `group_rect() -> (x, y, width, height)` | Rectangle of the three lights. |
| `glyph_svg(glyph, color)` | SVG of a glyph, 14 pt. |

## Tokens: `ctshell::design`

`ctshell::design` holds the tokens that only the desktop reads. Its
`build.rs` reuses `crates/ctdesign/build/tengen.rs` over the same Tengen JSON
([Tengen](../barrierkit/tengen.md)) with its own `CONSTANTS` table, and reads
the structs of `crates/ctdesign/src/lib.rs` and `src/design.rs`. To add one,
follow [Steps to add a constant](../barrierkit/tengen.md#steps-to-add-a-constant)
with the table in `crates/ctshell/build.rs`.

| Values | Key | Constant |
|---|---|---|
| Shell | `components.shell`: `menubar`, `dock`, `notifications`, `live-activity`, `launcher`, `lock`, `about`, `about-app`, `confirm`, `force-quit` | `MENUBAR_*`, `DOCK_*`, `NOTIFICATION_*`, `LIVE_ACTIVITY_*`, `LAUNCHER_*`, `LOCK_*`, `ABOUT_*`, `CONFIRM_*`, `FORCE_QUIT_*` |
| Desktop | `components.corner`, `window`, `spaces`, `tile`, `wallpaper`, `cursor-shake`, `background-blur`, `screenshot`, `glass-mask`, `desktop-clear` | `CORNER_NOTIFICATION`, `CORNER_NOTIFICATION_CENTER`, `CORNER_CONTROL_CENTER_MODULE`, `CORNER_SPOTLIGHT`, `WINDOW_EXPAND_*`, `FOCUS_BAR_*`, `SPACE_GAP*`, `SPACE_SLIDE_*`, `TILE_*`, `WALLPAPER_FPS`, `WALLPAPER_THUMBNAIL`, `WALLPAPER_FRAME`, `CURSOR_SHAKE_*`, `BACKGROUND_BLUR_*`, `SCREENSHOT_*`, `GLASS_MASK`, `DESKTOP_CLEAR` |
| Shell glass | `components.shell-glass` | `SHELL_GLASS_*`, `ShellGlass` |
| Liquid Glass | `components.liquid-glass` | `LIQUID_GLASS_*`, `GlassSet`, `GlassLook`, `GlassRim`, `GlassScale`, `GlassLens`, `GlassTint`, `RIM_BINS` |

`MENUBAR_HEIGHT`, `MENUBAR_TINT_SPREAD`, the Dock's frame, label and menu,
`MENU_*` and `SHELL_*` stay in `ctdesign`: the kit reads them
([Tengen](../barrierkit/tengen.md#value-map)).

### Windows, corners and layers

| Constant | Type | Read by |
|---|---|---|
| `CORNER_NOTIFICATION`, `CORNER_NOTIFICATION_CENTER`, `CORNER_CONTROL_CENTER_MODULE`, `CORNER_SPOTLIGHT` | `ctdesign::CornerShape` | The compositor and the shell: the measured radius and curve of each element |
| `WINDOW_EXPAND_MIN_MS`, `WINDOW_EXPAND_MAX_MS` | `u64` | The compositor: how long a window takes to maximize or restore, from the shortest move (300 ms) to one across the whole workspace (560 ms) |
| `ALERT_LAYER_NAMESPACE` | `&str` | `malaysia-dialog`: layer-shell namespace of shell alerts |
| `NOTIFICATIONS_LAYER_NAMESPACE` | `&str` | `malaysia-notifications`: layer-shell namespace of banners and Notification Center |
| `MENUBAR_LAYER_NAMESPACE`, `DOCK_LAYER_NAMESPACE`, `WALLPAPER_LAYER_NAMESPACE` | `&str` | `malaysia-menubar`, `malaysia-dock`, `malaysia-wallpaper`: the layer-shell namespaces of the menu bar, the Dock and the wallpaper |
| `GLASS_MASK` | `Rgba` | Base color of the mask that fades glass bars |
| `CIRCLE_EXPONENT` | `f32` | The superellipse exponent of a circle, 2 |
| `dock_divider(surface, backdrop)` | fn | The white or black overlay that moves the glass's luminance over `backdrop` by `surface.divider` |

### Liquid Glass

`design::LIQUID_GLASS_<STYLE>_<SCHEME>` (`REGULAR` or `CLEAR`, `LIGHT` or `DARK`) are
`GlassSet`s of `components.liquid-glass`. A `GlassSet` has the looks `active`
(a key window or a non-activating panel), `inactive` (a non-key window),
`reduced` (Reduce Transparency), `increased_contrast` and `reduced_contrast`
(both), the `rim` tables and the `scale` rules. A `GlassLook` is:

| Field | Unit | Meaning |
|---|---|---|
| `blur` | half-resolution texels | Gaussian sigma; 2 x `blur` x the share, in pixels, capped by `LIQUID_GLASS_BLUR_CAP` |
| `blur_opacity`, `blur_distance` | share, px | the share of `blur` at each distance (negative inside) |
| `white`, `black` | 0..1.5 | output for a white and a black input |
| `saturation`, `max_luma` | factor | saturation about the luma; the tone is scaled by `1 - (1 - max_luma) x luma` |
| `fill` | RGBA | mixed over the tone by its alpha |
| `refraction` | px | `[amount, height]` of the lens |
| `tone_delta` | levels of 255 | nine-point correction of the blur fill |

`ctshell::liquid_glass::looks(glass, scheme)` returns the set;
`adapted(look, &set.scale, shorter)` applies the size rules to the active look.
`face`, `blur_sigma`, `lens_offset`, `rim_levels` and `smooth_union` are the
stages and `render(set, tint, amount, state, &backdrop, &shape)` the CPU reference over an
`Image`. Values are device values at scale 1 as macOS 27.2.1 draws them; the
compositor applies them in its own output space
([research](../research/macos-liquid-glass.md#6-the-model)). The layer tree names each part, which one veil and one cap cannot: the lens, the blur profile, the state and the size.

`LIQUID_GLASS_TINT_<STYLE>_<SCHEME>` are the `GlassTint` of the Appearance › Liquid
Glass slider (`AppearanceConfig::liquid_glass_tint`, 0 to 1): `blur`, `blur_fill_opacity`,
`fill_rgb`, `fill_alpha` and `max_luma` are knots at 0, 0.5 and 1, `read_halvings` the
positions at which the read through the backdrop halves its resolution, and
`tone_shift_t25` to `_t100` nine-point shifts of `tone_delta`.
`liquid_glass::tint(glass, scheme)` returns it and
`tinted(&set, &tint, amount, shorter)` the active look at that position, with the
blur fill (`Tinted`); `render(set, tint, amount, state, &backdrop, &shape)` takes both.
The slider is a function of one number, so a compositor can apply it to every glass it draws. At 0 the look is the unchanged one
([research](../research/macos-liquid-glass.md#13-the-appearance--liquid-glass-slider-nsglasstintamount-measured-2026-11-22)).

### Shell

`malaysia-shell` reads these.

| Constants | What they size |
|---|---|
| `MENUBAR_HEIGHT_WIDE`, `MENUBAR_DARK_INK_ABOVE`, `MENUBAR_CLASSIC_ALPHA`, `MENUBAR_OVER_LIGHT_INK`, `MENUBAR_OVER_LIGHT_GLASS`, `MENUBAR_OVER_LIGHT_SELECTION`, `MENUBAR_OVER_DARK_INK`, `MENUBAR_OVER_DARK_GLASS`, `MENUBAR_OVER_DARK_SELECTION`, `MENUBAR_FONT_SIZE`, `MENUBAR_FONT_SIZE_LARGE`, `MENUBAR_APP_NAME_WEIGHT`, `MENUBAR_MARGIN`, `MENUBAR_MARGIN_RIGHT`, `MENUBAR_FADED_ALPHA`, `MENUBAR_TITLE_PADDING_X`, `MENUBAR_TEXT_LIFT`, `MENUBAR_TITLE_MAX_WIDTH`, `MENUBAR_SYSTEM_MENU_MIN_WIDTH`, `MENUBAR_TITLE_BLEED` | The menu bar |
| `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | The menu bar in focus mode |
| `DOCK_TILESIZE`, `DOCK_TILESIZE_MIN`, `DOCK_TILESIZE_MAX`, `DOCK_LARGESIZE` | Dock icon sizes: default, 16 and 128 px, and the default magnified size |
| `DOCK_EDGE_MARGIN`, `DOCK_SIDE_MARGIN`, `DOCK_PADDING`, `DOCK_PADDING_SCALE`, `DOCK_SLOT_GAP`, `DOCK_VERTICAL_PADDING`, `DOCK_RADIUS` | Dock bar: gap to the screen edge, side margin as a share of the screen, horizontal padding = `DOCK_PADDING` + `DOCK_PADDING_SCALE` × size, the gap between icon slots in pixels, vertical padding as a share of the tile, and corner radius as a share of its thickness |
| `DOCK_SEPARATOR`, `DOCK_SEPARATOR_LENGTH`, `DOCK_MAGNIFY_RADIUS`, `DOCK_MAGNIFY_MIN`, `DOCK_MAGNIFY_MS` | Separator slot and line as shares of the size; magnification reach in icons, the least it grows as a multiple of the size, and its fade |
| `DOCK_DOT`, `DOCK_DOT_MIN`, `DOCK_DOT_MAX`, `DOCK_PROGRESS_HEIGHT`, `DOCK_WINDOW_FRAME`, `DOCK_WINDOW_ICON`, `DOCK_WINDOW_BADGE` | Open-app dot, progress bar, the frame and icon of a minimized window before its picture arrives, and the app icon on the picture |
| `DOCK_ICON_BODY` | The share of a slot an icon's body takes (macOS's 824 on 1024) |
| `DOCK_BOUNCE_MS`, `DOCK_BOUNCE_HEIGHT`, `DOCK_LAUNCH_TIMEOUT_MS`, `DOCK_HIDE_MS`, `DOCK_REVEAL_STRIP`, `DOCK_RECENTS`, `DOCK_MINIMIZE_MS`, `DOCK_SLOT_MS`, `DOCK_MINIMIZE_HANDOFF_MS` | Launch bounce, automatic hiding, the number of recent apps, the minimize effect, how long a slot takes to open or close (a minimized window's, the gap of a dragged icon, and the flight back to its slot of one dropped away from the Dock), and how long the compositor keeps drawing the finished thumbnail while the Dock's tile takes over |
| `DOCK_LABEL_GAP`, `DOCK_LABEL_FONT_SIZE`, `DOCK_LABEL_PADDING_X`, `DOCK_LABEL_PADDING_Y` | The name label over the icon under the pointer |
| `DOCK_MENU_GAP` | An icon's menu: tip to icon ([`pop_up_menu`](../barrierkit/07-menus-and-popovers.md#pop_up_menu)) |
| `NOTIFICATION_WIDTH`, `NOTIFICATION_RADIUS`, `NOTIFICATION_PADDING`, `NOTIFICATION_GAP`, `NOTIFICATION_MARGIN`, `NOTIFICATION_ICON_SIZE`, `NOTIFICATION_TEXT_GAP`, `NOTIFICATION_CLOSE_SIZE`, `NOTIFICATION_TITLE_FONT`, `NOTIFICATION_SHADOW`, `NOTIFICATION_SHADOW_BLUR`, `NOTIFICATION_VEIL_SPREAD`, `NOTIFICATION_EMPTY_TOP`, `NOTIFICATION_EMPTY_GAP`, `NOTIFICATION_STACK_PEEK`, `NOTIFICATION_STACK_INSET`, `NOTIFICATION_BODY_LINES`, `NOTIFICATION_MAX_BANNERS`, `NOTIFICATION_BANNER_MS`, `NOTIFICATION_SLIDE_IN_MS`, `NOTIFICATION_SLIDE_OUT_MS`, `NOTIFICATION_SLIDE_BOUNCE`, `NOTIFICATION_EDGE`, `NOTIFICATION_SWIPE`, `LIVE_ACTIVITY_WIDTH`, `LIVE_ACTIVITY_HEIGHT`, `LIVE_ACTIVITY_RADIUS`, `LIVE_ACTIVITY_GAP`, `LIVE_ACTIVITY_ENDED_MS` | Notification banners, Notification Center and the Live Activities of the menu bar (ADR 0068 in malaysia; the banner width, radius and right inset are measured on Tahoe, the Live Activity size is measured on Apple's WWDC25 video) |
| `LAUNCHER_WIDTH`, `LAUNCHER_HEIGHT`, `LAUNCHER_PADDING`, `LAUNCHER_SEARCH_HEIGHT`, `LAUNCHER_RESULT_HEIGHT`, `LAUNCHER_RESULT_GAP`, `LAUNCHER_RESULT_RADIUS`, `LAUNCHER_ICON_SIZE`, `LAUNCHER_MAX_RESULTS`, `LAUNCHER_CORNER_RADIUS`, `LAUNCHER_FONT_SIZE`, `LAUNCHER_SUBTITLE_FONT_SIZE`, `LAUNCHER_SCALE_FROM`, `LAUNCHER_OPEN_MS`, `LAUNCHER_CLOSE_MS` | The launcher |
| `LOCK_CLOCK_FONT_SIZE`, `LOCK_DATE_FONT_SIZE`, `LOCK_TOP_PERCENT`, `LOCK_BOTTOM_MARGIN`, `LOCK_SPACING`, `LOCK_AVATAR_SIZE`, `LOCK_KEYBOARD_WIDTH`, `LOCK_FIELD_HEIGHT`, `LOCK_FIELD_WIDTH`, `LOCK_MASK_FONT_SIZE`, `LOCK_INK`, `LOCK_FIELD_FILL`, `LOCK_AVATAR_FILL`, `LOCK_FIELD_ERROR`, `LOCK_FIELD_INK` | The lock screen over the wallpaper (Malaysia ADR 0074) |
| `LOCK_FADE_MS`, `LOCK_SETTLE_MS`, `LOCK_RISE_MS`, `LOCK_RISE_OFFSET`, `LOCK_LOGIN_MS` | The transition from the desktop to the lock screen: the fade, the menu bar closing up, the clock rising and the login appearing |
| `ABOUT_WIDTH`, `ABOUT_HEIGHT`, `ABOUT_DRAWING_TOP`, `ABOUT_DRAWING_WIDTH`, `ABOUT_DRAWING_HEIGHT`, `ABOUT_TITLE_GAP`, `ABOUT_TITLE_SIZE`, `ABOUT_SPECS_GAP`, `ABOUT_SPEC_ROW_GAP`, `ABOUT_COLUMN_GAP`, `ABOUT_VALUE_WIDTH`, `ABOUT_BUTTON_GAP`, `ABOUT_FOOTER_BOTTOM` | The «About this computer» window |
| `ABOUT_APP_WIDTH`, `ABOUT_APP_HEIGHT`, `ABOUT_APP_ICON` | The «About ‹app›» window and its icon |
| `CONFIRM_WIDTH`, `CONFIRM_HEIGHT`, `CONFIRM_ICON`, `CONFIRM_GLYPH` | The restart, shut down and log out confirmation: its size, and its round icon and glyph |
| `FORCE_QUIT_WIDTH`, `FORCE_QUIT_HEIGHT`, `FORCE_QUIT_ROW_HEIGHT`, `FORCE_QUIT_ICON` | The «Force Quit» window, its rows and their app icons |
| `MENUBAR_BATTERY_ICON`, `MENUBAR_BATTERY_LOW`, `MENUBAR_BATTERY_TRACK`, `MENUBAR_BATTERY_BODY_WIDTH`, `MENUBAR_BATTERY_BODY_HEIGHT`, `MENUBAR_BATTERY_DIGIT_PITCH`, `MENUBAR_BATTERY_DIGIT_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_GAP`, `MENUBAR_BATTERY_BOLT_GAP`, `MENUBAR_BATTERY_LEVEL_MIN` | The battery in the menu bar, drawn as on macOS 27 with the percentage inside: its size, the percentage from which it turns red, the opacity of the ink in its empty part, and the metrics of the digits, the hundred mark and the bolt knocked out of its body |

### Compositor

`malaysia-compositor` reads these:

| Constants | What they are |
|---|---|
| `CURSOR_SHAKE_SCALE`, `CURSOR_SHAKE_GROW_MS`, `CURSOR_SHAKE_SHRINK_MS`, `CURSOR_SHAKE_TICK_MS` | The cursor that grows when the pointer is shaken |
| `CURSOR_SHAKE_MIN_DX`, `CURSOR_SHAKE_MIN_SWING`, `CURSOR_SHAKE_MAX_SWING_MS`, `CURSOR_SHAKE_MAX_TOTAL_MS`, `CURSOR_SHAKE_REVERSALS`, `CURSOR_SHAKE_ACTIVE_MS` | What counts as a shake: the shortest step and swing in points, the longest swing and whole gesture, the reversals it takes, and how long after one the cursor stays large |
| `BACKGROUND_BLUR_PASSES`, `BACKGROUND_BLUR_OFFSET` | The compositor's dual Kawase blur behind surfaces: halvings and sample spread (malaysia ADR 0071) |
| `SCREENSHOT_DIM`, `SCREENSHOT_IDLE_DIM`, `SCREENSHOT_BORDER`, `SCREENSHOT_BORDER_WIDTH` | The screenshot selection overlay |
| `SPACE_GAP`, `SPACE_GAP_COLOR`, `SPACE_SLIDE_MS`, `SPACE_SLIDE_EASE` | The gap and slide between Spaces |
| `TILE_MARGIN`, `TILE_GAP` | Margins of tiled windows |
| `DESKTOP_CLEAR` | The color behind everything |
| `WALLPAPER_FPS`, `WALLPAPER_THUMBNAIL`, `WALLPAPER_FRAME` | Wallpaper: frames per second of animated wallpapers, size of a thumbnail (the spec's x-large) and of a still of a dynamic image |

Next: [Glossary](glossary.md).
