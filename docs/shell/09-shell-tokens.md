# 9. Shell tokens

**Goal:** size and color a desktop surface with the tokens only the desktop
reads, and add one when it is missing.
**Scope:** `ctshell::design`: its families, who reads each, its metric types,
and the path from a key of the Tengen JSON to a constant. The kit's tokens and
the build they share are in [Tengen](../barrierkit/tengen.md) and
[4. Appearance and tokens](../barrierkit/04-appearance-and-tokens.md#tokens-ctdesign).
**Reader:** whoever changes a measurement of Malaysia's shell, compositor or
wallpaper.
**Input:** `crates/ctdesign/tengen/components.json` and `tokens.json`.
**Output:** a constant in `ctshell::design`, and its CSS variable in
`docs/barrierkit/tengen.css`.

## Steps to add a token

1. Pick the crate. A value the kit reads goes in `ctdesign`
   ([Tengen](../barrierkit/tengen.md#steps-to-add-a-constant)); a value only
   the desktop reads goes here.
2. Add the key to `components.json`, under its [family](#families).
3. Add a line to the `CONSTANTS` table of `crates/ctshell/build.rs`,
   `NAME type path`:

   ```text
   DOCK_SLOT_GAP f32 components.shell.dock.slot-gap
   ```

4. If the type is a new struct, declare it in `crates/ctshell/src/design.rs`,
   one `pub field: Type,` per line and the closing `}` at the start of a line.
   Each field reads its key by the rules of
   [Tengen](../barrierkit/tengen.md#steps-to-add-a-constant), steps 3 and 4.
5. Build with `cargo build -p ctshell`. A missing key, struct or field stops
   the build with `ctdesign: …` and the path it looked for.
6. Regenerate the CSS: `scripts/tengen-css.sh > docs/barrierkit/tengen.css`.
7. Add its row to the [shell glossary](glossary.md).
8. Use it as `ctshell::design::NAME` and run `./dev.sh check`
   ([Checks](#checks)).

## How the build reads the table

`crates/ctshell/build.rs` includes `crates/ctdesign/build/tengen.rs` with
`#[path]`, so both crates read the JSON the same way:

1. `Tengen::load` merges `tokens.json` and `components.json` and reads the
   structs of `crates/ctdesign/src/lib.rs` and `crates/ctshell/src/design.rs`.
   `cargo:rerun-if-changed` watches the four files.
2. `constants(CONSTANTS)` writes one `pub const` per line of the table into
   `$OUT_DIR/tengen.rs`.
3. `src/design.rs` includes that file with `include!`.

| Type in the table | Value |
|---|---|
| `f32`, `f64`, `u16`, `u32`, `u64`, `i32`, `usize` | A number, or the dotted path of another key |
| `Rgba` | A hex color, or a token name in braces such as `{ink}` |
| `Font` | The name of a style of `font.styles`, or an object with `family`, `size`, `line-height` and `weight` |
| `[f32;N]`, `[Rgba;N]` | A list; the type has no space, because the table splits on spaces |
| A struct | Its fields, read under the path |

The table has no `THEMED` counterpart: a `{token}` color resolves in the
default theme, `light`. A value that changes per theme goes in the `THEMED`
table of `ctdesign`.

## Families

| Family | Key | Constants | Read by |
|---|---|---|---|
| Menu bar | `components.shell.menubar` | `MENUBAR_*` | `malaysia-shell` |
| Dock | `components.shell.dock` | `DOCK_*` | `malaysia-shell`; System Settings (`DOCK_TILESIZE*`) |
| Notifications | `components.shell.notifications`, `live-activity` | `NOTIFICATION_*`, `LIVE_ACTIVITY_*` | `malaysia-shell`; the compositor (`NOTIFICATION_WIDTH`) |
| Launcher | `components.shell.launcher` | `LAUNCHER_*` | `malaysia-shell` |
| Lock screen | `components.shell.lock` | `LOCK_*` | `malaysia-shell` |
| Dialogs | `components.shell.about`, `about-app`, `confirm`, `force-quit` | `ABOUT_*`, `ABOUT_APP_*`, `CONFIRM_*`, `FORCE_QUIT_*` | `malaysia-shell` |
| Focus bar | `components.spaces.focus-bar`, `focus-bar-text` | `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | `malaysia-shell` |
| Corners | `components.corner` | `CORNER_NOTIFICATION`, `CORNER_NOTIFICATION_CENTER`, `CORNER_CONTROL_CENTER_MODULE`, `CORNER_SPOTLIGHT` | The compositor and `malaysia-shell` |
| Window motion | `components.window.expand-min-ms`, `expand-max-ms` | `WINDOW_EXPAND_MIN_MS`, `WINDOW_EXPAND_MAX_MS` | The compositor |
| Spaces and tiling | `components.spaces`, `tile` | `SPACE_GAP`, `SPACE_GAP_COLOR`, `SPACE_SLIDE_MS`, `SPACE_SLIDE_EASE`, `TILE_MARGIN`, `TILE_GAP` | The compositor |
| Pointer | `components.cursor-shake` | `CURSOR_SHAKE_*` | The compositor |
| Blur and screenshots | `components.background-blur`, `screenshot` | `BACKGROUND_BLUR_*`, `SCREENSHOT_*` | The compositor |
| Desktop | `components.desktop-clear` | `DESKTOP_CLEAR` | The compositor, `malaysia-wallpaper` and System Settings |
| Glass mask | `components.glass-mask` | `GLASS_MASK` | — |
| Wallpaper | `components.wallpaper` | `WALLPAPER_FPS`, `WALLPAPER_THUMBNAIL`, `WALLPAPER_FRAME` | `malaysia-wallpaper` |
| Shell glass | `components.shell-glass` | `SHELL_GLASS_*` | The compositor ([7](07-glass-and-blur.md#measured-pipeline)) |
| Liquid Glass | `components.liquid-glass` | `LIQUID_GLASS_*` | The compositor ([7](07-glass-and-blur.md#liquid-glass)) |

`GLASS_MASK` is the base color of the mask that fades glass bars.

`MENUBAR_HEIGHT`, `MENUBAR_TINT_SPREAD`, the Dock's frame, label and menu,
`MENU_*` and `SHELL_*` stay in `ctdesign`, because the kit reads them
([4](../barrierkit/04-appearance-and-tokens.md#shell)).

`src/design.rs` also writes these by hand:

| Item | What it is |
|---|---|
| `MENUBAR_LAYER_NAMESPACE`, `DOCK_LAYER_NAMESPACE`, `WALLPAPER_LAYER_NAMESPACE`, `NOTIFICATIONS_LAYER_NAMESPACE`, `ALERT_LAYER_NAMESPACE` | `malaysia-menubar`, `malaysia-dock`, `malaysia-wallpaper`, `malaysia-notifications` (banners and Notification Center) and `malaysia-dialog` (shell alerts): the layer-shell namespaces the compositor matches |
| `CIRCLE_EXPONENT` | 2.0, the superellipse exponent of a circle |
| `RIM_BINS` | 35, the distance bins of a `GlassRim` |
| `dock_divider(surface, backdrop)` | The white or black overlay that moves the glass's luminance over `backdrop` by `surface.divider` |

### Shell

| Constants | What they size |
|---|---|
| `MENUBAR_HEIGHT_WIDE`, `MENUBAR_DARK_INK_ABOVE`, `MENUBAR_CLASSIC_ALPHA`, `MENUBAR_OVER_LIGHT_INK`, `MENUBAR_OVER_LIGHT_GLASS`, `MENUBAR_OVER_LIGHT_SELECTION`, `MENUBAR_OVER_DARK_INK`, `MENUBAR_OVER_DARK_GLASS`, `MENUBAR_OVER_DARK_SELECTION` | The menu bar's wide height and its colors per style ([8](08-desktop-settings.md#menu-bar)) |
| `MENUBAR_FONT_SIZE`, `MENUBAR_FONT_SIZE_LARGE`, `MENUBAR_APP_NAME_WEIGHT`, `MENUBAR_MARGIN`, `MENUBAR_MARGIN_RIGHT`, `MENUBAR_FADED_ALPHA`, `MENUBAR_TITLE_PADDING_X`, `MENUBAR_TEXT_LIFT`, `MENUBAR_TITLE_MAX_WIDTH`, `MENUBAR_SYSTEM_MENU_MIN_WIDTH`, `MENUBAR_TITLE_BLEED` | The menu bar's text and titles |
| `MENUBAR_BATTERY_ICON`, `MENUBAR_BATTERY_LOW`, `MENUBAR_BATTERY_TRACK`, `MENUBAR_BATTERY_BODY_WIDTH`, `MENUBAR_BATTERY_BODY_HEIGHT`, `MENUBAR_BATTERY_DIGIT_PITCH`, `MENUBAR_BATTERY_DIGIT_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_HEIGHT`, `MENUBAR_BATTERY_HUNDRED_GAP`, `MENUBAR_BATTERY_BOLT_GAP`, `MENUBAR_BATTERY_LEVEL_MIN` | The battery, drawn as on macOS 27 with the percentage inside: its size, the level from which it turns red, the opacity of its empty part, and the digits, the hundred mark and the bolt knocked out of its body |
| `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | The menu bar in focus mode |
| `DOCK_TILESIZE`, `DOCK_TILESIZE_MIN`, `DOCK_TILESIZE_MAX`, `DOCK_LARGESIZE` | Icon sizes: default (64), 16 and 128 px, and the default magnified size (64) |
| `DOCK_EDGE_MARGIN`, `DOCK_SIDE_MARGIN`, `DOCK_PADDING`, `DOCK_PADDING_SCALE`, `DOCK_SLOT_GAP`, `DOCK_VERTICAL_PADDING`, `DOCK_RADIUS` | The bar: gap to the screen edge, side margin as a share of the screen, horizontal padding (`DOCK_PADDING` + `DOCK_PADDING_SCALE` × size), the gap between slots, vertical padding as a share of the tile, and the corner radius as a share of its thickness |
| `DOCK_SEPARATOR`, `DOCK_SEPARATOR_LENGTH`, `DOCK_MAGNIFY_RADIUS`, `DOCK_MAGNIFY_MIN`, `DOCK_MAGNIFY_MS` | The separator's slot and line as shares of the size; the magnification's reach in icons, the least it grows as a multiple of the size, and its fade |
| `DOCK_DOT`, `DOCK_DOT_MIN`, `DOCK_DOT_MAX`, `DOCK_PROGRESS_HEIGHT`, `DOCK_WINDOW_FRAME`, `DOCK_WINDOW_ICON`, `DOCK_WINDOW_BADGE`, `DOCK_ICON_BODY` | The open-app dot, the progress bar, a minimized window's frame and icon before its picture arrives, the app icon on the picture, and the share of a slot an icon's body takes |
| `DOCK_BOUNCE_MS`, `DOCK_BOUNCE_HEIGHT`, `DOCK_LAUNCH_TIMEOUT_MS`, `DOCK_HIDE_MS`, `DOCK_REVEAL_STRIP`, `DOCK_RECENTS`, `DOCK_MINIMIZE_MS`, `DOCK_SLOT_MS`, `DOCK_MINIMIZE_HANDOFF_MS` | The launch bounce, automatic hiding, the number of recent apps, the minimize effect, how long a slot takes to open or close, and how long the compositor keeps drawing the finished thumbnail while the Dock's tile takes over |
| `DOCK_LABEL_GAP`, `DOCK_LABEL_FONT_SIZE`, `DOCK_LABEL_PADDING_X`, `DOCK_LABEL_PADDING_Y`, `DOCK_MENU_GAP` | The name label over the icon under the pointer, and the gap from an icon to its menu's tip ([`pop_up_menu`](../barrierkit/07-menus-and-popovers.md#pop_up_menu)) |
| `NOTIFICATION_WIDTH`, `NOTIFICATION_RADIUS`, `NOTIFICATION_PADDING`, `NOTIFICATION_GAP`, `NOTIFICATION_MARGIN`, `NOTIFICATION_ICON_SIZE`, `NOTIFICATION_TEXT_GAP`, `NOTIFICATION_CLOSE_SIZE`, `NOTIFICATION_TITLE_FONT`, `NOTIFICATION_SHADOW`, `NOTIFICATION_SHADOW_BLUR`, `NOTIFICATION_VEIL_SPREAD`, `NOTIFICATION_EMPTY_TOP`, `NOTIFICATION_EMPTY_GAP`, `NOTIFICATION_STACK_PEEK`, `NOTIFICATION_STACK_INSET`, `NOTIFICATION_BODY_LINES` | A banner and Notification Center |
| `NOTIFICATION_MAX_BANNERS`, `NOTIFICATION_BANNER_MS`, `NOTIFICATION_SLIDE_IN_MS`, `NOTIFICATION_SLIDE_OUT_MS`, `NOTIFICATION_SLIDE_BOUNCE`, `NOTIFICATION_EDGE`, `NOTIFICATION_SWIPE` | How many banners show, how long, how they slide, and the edge swipe that opens Notification Center |
| `LIVE_ACTIVITY_WIDTH`, `LIVE_ACTIVITY_HEIGHT`, `LIVE_ACTIVITY_RADIUS`, `LIVE_ACTIVITY_GAP`, `LIVE_ACTIVITY_ENDED_MS` | The Live Activities of the menu bar |
| `LAUNCHER_WIDTH`, `LAUNCHER_HEIGHT`, `LAUNCHER_PADDING`, `LAUNCHER_SEARCH_HEIGHT`, `LAUNCHER_RESULT_HEIGHT`, `LAUNCHER_RESULT_GAP`, `LAUNCHER_RESULT_RADIUS`, `LAUNCHER_ICON_SIZE`, `LAUNCHER_MAX_RESULTS`, `LAUNCHER_CORNER_RADIUS`, `LAUNCHER_FONT_SIZE`, `LAUNCHER_SUBTITLE_FONT_SIZE`, `LAUNCHER_SCALE_FROM`, `LAUNCHER_OPEN_MS`, `LAUNCHER_CLOSE_MS` | The launcher |
| `LOCK_CLOCK_FONT_SIZE`, `LOCK_DATE_FONT_SIZE`, `LOCK_TOP_PERCENT`, `LOCK_BOTTOM_MARGIN`, `LOCK_SPACING`, `LOCK_AVATAR_SIZE`, `LOCK_KEYBOARD_WIDTH`, `LOCK_FIELD_HEIGHT`, `LOCK_FIELD_WIDTH`, `LOCK_MASK_FONT_SIZE`, `LOCK_INK`, `LOCK_FIELD_FILL`, `LOCK_AVATAR_FILL`, `LOCK_FIELD_ERROR`, `LOCK_FIELD_INK` | The lock screen over the wallpaper |
| `LOCK_FADE_MS`, `LOCK_SETTLE_MS`, `LOCK_RISE_MS`, `LOCK_RISE_OFFSET`, `LOCK_LOGIN_MS` | From the desktop to the lock screen: the fade, the menu bar closing up, the clock rising and the login appearing |
| `ABOUT_WIDTH`, `ABOUT_HEIGHT`, `ABOUT_DRAWING_TOP`, `ABOUT_DRAWING_WIDTH`, `ABOUT_DRAWING_HEIGHT`, `ABOUT_TITLE_GAP`, `ABOUT_TITLE_SIZE`, `ABOUT_SPECS_GAP`, `ABOUT_SPEC_ROW_GAP`, `ABOUT_COLUMN_GAP`, `ABOUT_VALUE_WIDTH`, `ABOUT_BUTTON_GAP`, `ABOUT_FOOTER_BOTTOM` | The «About this computer» window |
| `ABOUT_APP_WIDTH`, `ABOUT_APP_HEIGHT`, `ABOUT_APP_ICON` | The «About ‹app›» window and its icon |
| `CONFIRM_WIDTH`, `CONFIRM_HEIGHT`, `CONFIRM_ICON`, `CONFIRM_GLYPH` | The restart, shut down and log out confirmation, and its round icon and glyph |
| `FORCE_QUIT_WIDTH`, `FORCE_QUIT_HEIGHT`, `FORCE_QUIT_ROW_HEIGHT`, `FORCE_QUIT_ICON` | The «Force Quit» window, its rows and their app icons |

### Compositor

| Constants | What they are |
|---|---|
| `CORNER_NOTIFICATION`, `CORNER_NOTIFICATION_CENTER`, `CORNER_CONTROL_CENTER_MODULE`, `CORNER_SPOTLIGHT` | The measured radius and curve of each element ([Corners](../barrierkit/04-appearance-and-tokens.md#corners)); the shell reads them too |
| `WINDOW_EXPAND_MIN_MS`, `WINDOW_EXPAND_MAX_MS` | How long a window takes to maximize or restore: 300 ms for the shortest move, 560 ms across the whole workspace |
| `SPACE_GAP`, `SPACE_GAP_COLOR`, `SPACE_SLIDE_MS`, `SPACE_SLIDE_EASE` | The gap and the slide between Spaces |
| `TILE_MARGIN`, `TILE_GAP` | The margins of tiled windows |
| `CURSOR_SHAKE_SCALE`, `CURSOR_SHAKE_GROW_MS`, `CURSOR_SHAKE_SHRINK_MS`, `CURSOR_SHAKE_TICK_MS` | The cursor that grows when the pointer is shaken |
| `CURSOR_SHAKE_MIN_DX`, `CURSOR_SHAKE_MIN_SWING`, `CURSOR_SHAKE_MAX_SWING_MS`, `CURSOR_SHAKE_MAX_TOTAL_MS`, `CURSOR_SHAKE_REVERSALS`, `CURSOR_SHAKE_ACTIVE_MS` | What counts as a shake: the shortest step and swing in points, the longest swing and gesture, the reversals it takes, and how long the cursor stays large |
| `BACKGROUND_BLUR_PASSES`, `BACKGROUND_BLUR_OFFSET` | The dual Kawase blur behind surfaces: 3 halvings, spread 3.0 ([7](07-glass-and-blur.md#who-draws-the-blur)) |
| `SCREENSHOT_DIM`, `SCREENSHOT_IDLE_DIM`, `SCREENSHOT_BORDER`, `SCREENSHOT_BORDER_WIDTH` | The screenshot selection overlay |
| `DESKTOP_CLEAR` | The color behind everything |
| `SHELL_GLASS_LEVELS`, `SHELL_GLASS_CHROMA_KNOTS`, `SHELL_GLASS_<SURFACE>_DARK` | The [measured pipeline](07-glass-and-blur.md#measured-pipeline): the 12 luma levels, the 5 chroma knots, and one `ShellGlass` per surface (`SPACES_BAR` is a `MaterialLook`) |
| `LIQUID_GLASS_<STYLE>_<SCHEME>`, `LIQUID_GLASS_TINT_<STYLE>_<SCHEME>`, `LIQUID_GLASS_LENS`, `LIQUID_GLASS_BLUR_CAP`, `LIQUID_GLASS_UNION` | [Liquid Glass](07-glass-and-blur.md#liquid-glass): a `GlassSet` and a `GlassTint` per style (`REGULAR`, `CLEAR`) and scheme (`LIGHT`, `DARK`), the lens, the cap of the blur sigma and the spacing share of a smooth union |

### Wallpaper

`malaysia-wallpaper` reads `WALLPAPER_FPS`, the frames per second of an
animated wallpaper; `WALLPAPER_THUMBNAIL`, the side of a thumbnail (the
spec's x-large); and `WALLPAPER_FRAME`, the side of a still of a dynamic image
([8](08-desktop-settings.md#wallpaper-catalog)).

## Metric types

The scalar, color and list types are those of `ctdesign`
([Token types](../barrierkit/04-appearance-and-tokens.md#token-types));
`CornerShape` and `MaterialLook` come from `ctdesign` too. `src/design.rs`
declares the rest.

`ShellGlass` is one surface's glass: `sigma` and `halo_sigma` in px,
`halo_weight`, `tone` (12 levels at `SHELL_GLASS_LEVELS`) and `chroma`
(5 gains at `SHELL_GLASS_CHROMA_KNOTS`).

### Liquid Glass types

A `GlassSet` holds the looks `active` (a key window or a non-activating
panel), `inactive`, `reduced` (Reduce Transparency), `increased_contrast` and
`reduced_contrast`, its `rim` and its `scale`. A `GlassLook` is:

| Field | Unit | Meaning |
|---|---|---|
| `blur` | Half-resolution texels | The Gaussian sigma: 2 × `blur` × the share, in px, capped by `LIQUID_GLASS_BLUR_CAP` |
| `blur_opacity`, `blur_distance` | Share, px | The share of `blur` at each distance; negative is inside |
| `white`, `black` | 0 to 1.5 | The output for a white and a black input |
| `saturation` | Factor | Saturation about the luma |
| `max_luma` | Factor | The tone is scaled by `1 - (1 - max_luma) × luma` |
| `fill` | RGBA | Mixed over the tone by its alpha |
| `refraction` | px | `[amount, height]` of the lens |
| `tone_delta` | Levels of 255 | A nine-point correction along the luma |

| Type | Fields |
|---|---|
| `GlassRim` | `angle_1`, `angle_2` (12 angle bins of 7.5°) and `distance_1`, `distance_2` (`RIM_BINS` distance bins): two rim profiles and their weight by angle |
| `GlassScale` | `refraction_amount`, `refraction_height`, `blur_opacity_0`, `blur_opacity_1`, `blur_distance_0`, `max_luma`: each `[slope, offset, a, b]`, the value `slope × shorter + offset` kept between `a` and `b` |
| `GlassLens` | `scale` and `decay` of the lens's exponential profile |
| `GlassTint` | The Appearance › Liquid Glass slider: `blur`, `blur_fill_opacity`, `fill_rgb`, `fill_alpha` and `max_luma` as knots at 0, 0.5 and 1; `blur_opacity`; `blur_fill_radius`; `read_halvings`, the positions at which the read halves its resolution; `tone_shift_t25` to `tone_shift_t100`, nine-point shifts of `tone_delta` |

## Checks

`./dev.sh check` runs these over the tokens:

| Check | Fails when |
|---|---|
| `scripts/check-design-tokens.sh` | Production code in `crates/*/src`, outside `ctdesign`, holds a color, radius, duration or UI measurement literal ([the patterns](../barrierkit/tengen.md#checks)). `ctshell/src` is checked too |
| `scripts/tengen-css.sh \| diff -u docs/barrierkit/tengen.css -` | `docs/barrierkit/tengen.css` differs from the JSON |

`scripts/check-client-only-deps.sh` also fails if `ctshell` pulls Freya or
Skia.

Next: [10. Run and debug](10-run-and-debug.md).
