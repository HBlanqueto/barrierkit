# 4. Appearance and tokens

**Goal:** paint a view with the system's colors, type, symbols and sizes, follow
the user's appearance without a restart, and add a token when one is missing.
**Scope:** from the Tengen JSON files to the palette a view paints with. It
covers `style`, `theme`, `symbol`, fonts, the whole of `ctdesign` and the
whole of `ctappearance`. Backdrops and their tints are in
[15](15-backdrops-and-media.md); traffic lights as a control in
[9](09-titlebars.md); the language of the text in [17](17-languages.md); how
`build.rs` reads the JSON in [Tengen](../architecture/tengen.md).
**Reader:** anyone who designs an app or a component.
**Input:** an app from [2](02-first-app.md). **Output:** a view that uses only
tokens and the palette, and follows the user's appearance.

## Steps

1. Read the palette inside the component: `let palette = Palette::current();`
   ([The palette](#the-palette)).
2. Write text with a type token: `text("Storage", HEADLINE, palette.label)`
   ([Text, shapes and colors](#text-shapes-and-colors)).
3. Size and space with tokens: `.padding(SPACE_3)`,
   `.corner_radius(continuous_corner(BUTTON.radius))`
   ([Tokens](#tokens-ctdesign)).
4. Convert palette colors for Freya with `color`:
   `.background(color(palette.content_background))`.
5. Add a symbol:
   `Image::system(Symbol::Folder).size(SYMBOL.grid).color(palette.secondary_label)`
   ([Symbols](#symbols)). A one-color image takes the theme's color with
   `.rendering_mode(TemplateRenderingMode::Template)`
   ([Images and template rendering](#images-and-template-rendering)).
6. If the view needs a value Tengen does not have, add it:
   [Adding a token](#adding-a-token).
7. Run `./dev.sh check`. `scripts/check-design-tokens.sh` rejects color
   literals and UI measurements outside `ctdesign`.

The result:

```rust
use bk::prelude::*;
use bk::style::continuous_corner;
use ctdesign::{BUTTON, SYMBOL};

fn storage_row() -> impl IntoElement {
    let palette = Palette::current();
    HStack::new()
        .padding(SPACE_3)
        .corner_radius(continuous_corner(BUTTON.radius))
        .background(color(palette.content_background))
        .child(
            Image::system(Symbol::Folder)
                .size(SYMBOL.grid)
                .color(palette.secondary_label),
        )
        .child(text("Storage", HEADLINE, palette.label))
}
```

## Where a value comes from

| Source | Who decides | Where it lives |
|---|---|---|
| **Tokens**: base colors, spacing, type, durations, per-component metrics | The design system | `crates/ctdesign/tengen/tokens.json` and `components.json` |
| **Appearance**: mode, accent, tints, sidebar style, contrast, motion | The user | `~/.config/malaysia/appearance.domain` |
| **Variants** and **styles** of a control: a preset such as `ButtonStyle::bordered_prominent`, tint, tinted backdrop, a `SliderStyle` | The app | The app's code ([styles](../../agents/styles.md)) |

The kit resolves the first two into a `Resolved` value
([Resolution](#resolution)) and the `Palette` that views paint with.

## The palette

`style::Palette` holds the semantic colors of the current context. A view or a
component paints only with them.

| Method | Returns |
|---|---|
| `Palette::current()` | The palette in effect here: the one a `Backdrop` provides ([15](15-backdrops-and-media.md)), otherwise `Palette::system()`. In an inactive window `accent` becomes `accent_inactive`, and `on_accent` follows it. |
| `Palette::system()` | The palette of the resolved appearance, without backdrop and without the active-window rule. |

| Field | Use | AppKit |
|---|---|---|
| `label` | Primary text | `NSColor.labelColor` |
| `secondary_label` | Secondary text | `NSColor.secondaryLabelColor` |
| `tertiary_label` | Tertiary text and placeholders | `NSColor.tertiaryLabelColor` |
| `separator` | Separators | `NSColor.separatorColor` |
| `control_border` | Edge of a control | — |
| `window_background` | Window background | `NSColor.windowBackgroundColor` |
| `content_background` | Background of lists, tables and groups | `NSColor.controlBackgroundColor` |
| `control_background` | Face of a control at rest | `NSColor.controlColor` |
| `control_hover` | Face of a control under the pointer | — |
| `selected_control` | Face of a selected control | `NSColor.selectedControlColor` |
| `accent` | Accent | `NSColor.controlAccentColor` |
| `accent_inactive` | Accent in an inactive window | — |
| `on_accent` | Ink on the accent | `NSColor.alternateSelectedControlTextColor` |
| `destructive` | Destructive actions | `NSColor.systemRed` |
| `raised_background` | Raised surfaces | — |
| `sunken_background` | Sunken surfaces | — |
| `scrim` | Dimming behind sheets | — |
| `card_border` | Edge of a card | — |

With `contrast = true` ([The appearance file](#the-appearance-file)) each ink
moves up one level: `secondary_label` takes the strong ink and
`tertiary_label` the muted one. Separators and control borders use the muted
ink, which reaches 3:1 on the window, as the HIG asks for «Increase Contrast».

## Text, shapes and colors

| Function | What it does |
|---|---|
| `style::text(content, face, ink) -> Label` | A one-line Freya `label` with the family, size, weight and line height of a `Font` token and the color `ink`. For text that wraps, use `Text` ([2](02-first-app.md)). |
| `style::continuous_corner(radius) -> CornerRadius` | A corner radius with Apple's continuous curve, smoothed as `WINDOW_CORNER.expansion` says (malaysia ADR 0059). |
| `style::glass_edge(body, radius, fill, strength) -> Rect` | The inner line of `panel_edges` without the outer line and without the inward glow, with its alpha multiplied by `strength` (the Dock: `DOCK_RIM_DARK` to `DOCK_RIM_BRIGHT` by the lightness of the wallpaper behind it) and softened by a Gaussian of `DOCK_RIM_BLUR` physical pixels. |
| `style::panel_edges(body, radius, fill) -> Rect` | A card filled with `fill` and framed like a panel window: the glass inner line in dark (`panel_inner_line`: brighter top and bottom edges, inward glow), the window inner line otherwise, and the outer line, each one physical pixel wide. Same colours and widths as the compositor's panel frames. |
| `theme::color(rgba) -> Color` | Converts a Tengen `Rgba` to Freya's `Color`. |

## Symbols

`Image::system(symbol)` draws a line icon, like SwiftUI's
`Image(systemName:)`. It is `BODY.size` pixels square, in the `label` color
of the `Palette`. `.size(f32)` and `.color(Rgba)` change them. The icons
are drawn on a grid of `SYMBOL.grid` (16) with a stroke of `SYMBOL.stroke`
(1.4), like SF Symbols at regular weight.

```rust
Image::system(Symbol::Folder)
    .size(TITLE.size as f32)
    .color(Palette::current().accent)
```

`Symbol` names the icon. `Image` and `Symbol` are in the prelude:

| Group | Variants |
|---|---|
| Navigation | `ChevronRight`, `ChevronLeft`, `ChevronUp`, `ChevronDown`, `ChevronUpDown`, `ChevronDoubleRight`, `GoBackward`, `GoForward`, `ArrowDown` |
| Actions | `MagnifyingGlass`, `XMark`, `Plus`, `Minus`, `Ellipsis`, `Checkmark`, `Expand`, `QuestionMark` |
| Media | `Play`, `Pause`, `Shuffle`, `ChevronLeftForwardSlashChevronRight`, `Backward`, `Forward`, `SpeakerWave`, `Captions`, `Film`, `Tv` |
| Layout | `ListBullet`, `SquareGrid`, `SlidersVertical`, `SidebarLeft` |
| Places and things | `House`, `Folder`, `Document`, `Star`, `StarFill`, `Clock`, `Bookmark`, `CircleHalf`, `Accessibility`, `Display`, `MenubarDockRectangle`, `Photo`, `Moon`, `Bell`, `Bed`, `Person`, `Briefcase`, `Printer`, `Phone`, `SquareOnSquare`, `Keyboard` |
| Connections and power | `Wifi`, `Bluetooth`, `Network`, `Lock`, `Bolt`, `Battery` |
| Row actions | `EllipsisCircle` |
| Shapes | `Square`, `Circle` |

`StarFill` is the only filled symbol. The kit caches the SVG of each symbol.

`Image` draws from an `ImageStyle`: `size` and `foreground`, which `.size`
and `.color` override for one image and a theme for every image. The fields
are in [Styles and themes](../../crates/bk/src/styles/text.rs).

## Images and template rendering

Symbols are SVG. A symbol the kit does not have is an SVG too, never a PNG:
an SVG stays sharp at every scale and takes the theme's color.

`Image::data(bytes)` draws SVG, PNG or JPEG bytes. `Image::rgba(width,
height, rgba)` draws straight-alpha pixels, such as a status item's `Pixmap`.
Both draw the image's own colors by default.

`.rendering_mode(TemplateRenderingMode::Template)` keeps only the image's
alpha and paints it in the foreground color, like SwiftUI's
`.renderingMode(.template)` and AppKit's `isTemplate`. The foreground is
`label` unless `.color` changes it, so the image is black in light and white
in dark, and changes with the appearance.

```rust
Image::data(include_bytes!("wifi-symbolic.svg").as_slice())
    .rendering_mode(TemplateRenderingMode::Template)
    .size(SYMBOL.grid)
```

| Image | Mode |
|---|---|
| A symbol: `Image::system`, or an SVG drawn with `currentColor` | Always template |
| A one-color icon in SVG or PNG, such as a freedesktop `-symbolic` icon | `Template` |
| An app icon, a photo or an illustration | `Original`, the default |

A template image rasterizes at the window's scale and is cached by bytes,
size and color.

## Fonts

`register_font(bytes) -> bool` adds a typeface from the bytes of a TrueType or
OpenType file. Its family is then available to text, menus, printing and the
font panel of that thread. It returns `false` when the bytes are not a font.
Call it on the thread that runs the window, before opening it:

```rust
bk::register_font(include_bytes!("../fonts/Display.otf"));
```

The type tokens use Adwaita Sans and Geist Mono ([Type](#type)). The dev shell
installs them.

## Theme functions

`bk::theme` connects the resolved appearance with Freya.

| Function | What it does |
|---|---|
| `resolved() -> Resolved` | The resolved appearance of this point of the tree: the one `use_appearance` provides, otherwise the window's, otherwise it reads the files. |
| `use_appearance(adjust)` | Hook. Changes the appearance for this component and its children: `adjust` edits a copy of the user's `AppearanceConfig`. It follows later changes of the user's appearance. |
| `use_shell_theme()` | Hook. Gives Freya's own components a theme built from the resolved shell colors, and updates it when the appearance changes. |
| `for_scheme(dark) -> Theme` | Freya theme for the default light or dark appearance. |
| `surface_background(resolved, kind) -> Rgba` | Background of a surface: the window color for `SurfaceKind::Toplevel`; the bar color for `SurfaceKind::LayerShell`, tinted when the tint covers the desktop. |
| `menu_palette(resolved, kind) -> MenuPalette` | Menu colors over that surface. A tinted surface lifts the menu background and recomputes its separator. |
| `popover_palette(resolved, kind) -> MenuPalette` | `menu_palette` with the metrics of a pop-up menu (`MENU_POPOVER`). |
| `apply_popover_metrics(palette) -> MenuPalette` | Replaces the padding, widths, item size and check mark of a `MenuPalette` with `MENU_POPOVER`. |

`SurfaceKind` has two variants: `Toplevel` (an app window) and `LayerShell`
(a shell surface such as the menu bar).

Force dark mode in one part of a window:

```rust
use bk::theme::use_appearance;
use ctappearance::ColorScheme;

fn dark_panel() -> impl IntoElement {
    use_appearance(|config| config.color_scheme = ColorScheme::Dark);
    storage_row()
}
```

## Custom colors in an app

An app picks a variant, or changes the style of a control that takes one
([Styles and themes](../../agents/styles.md)).

| I want… | Like this | Where |
|---|---|---|
| A control in another color | `.tint(Tint::Color(ACCENT_SWATCHES[4].1))` on `Switch`, `Checkbox`, `Slider`, `LevelSlider`, `RadioGroup`, `ProgressView` | [5](05-controls.md) |
| A control filled with the backdrop gradient | `.tint(Tint::Backdrop)` inside a `Backdrop` | [15](15-backdrops-and-media.md) |
| Another accent for a whole area | `Backdrop::new(..).accent(Accent::Own(color))` or `Accent::FromBackdrop`; it holds only while the user's accent is automatic | [15](15-backdrops-and-media.md) |
| Let the user pick a color | `ColorWell::new("Color", &ACCENT_SWATCHES)` | [5](05-controls.md) |
| Light or dark in one area | `theme::use_appearance(..)` | [Theme functions](#theme-functions) |
| Any color, shape, size or motion of a control that takes a style (sliders, buttons, menu buttons, switch, checkbox, choices, stepper, toggle group, segmented control, tool group, text fields) | `.style(\|s\| ..)`, `.control_style(\|s: SliderStyle\| ..)` or `Theme` | [Styles and themes](../../agents/styles.md#which-controls-take-a-style) |
| The text color, background or shadow of a view | `.foreground_style(..)`, `.background_style(..)`, `.shadows(..)` | [Styles and themes](../../agents/styles.md#paint-on-views) |

In an inactive window every tint uses the inactive accent.

## User appearance: `ctappearance`

`ctappearance` reads the user's appearance, resolves it into colors and
watches it. The compositor, the shell, the portal and the kit share it, so all
of them agree. The kit's runner watches the files: when the appearance or the
wallpaper changes, every window repaints without a restart.

### The appearance file

`appearance_path()` is `config_dir()/appearance.domain`. `config_dir()` is
`$MALAYSIA_CONFIG_DIR`, or `$XDG_CONFIG_HOME/malaysia`
(`~/.config/malaysia`). A missing key takes its default; an invalid value takes
its default and logs a warning.

```toml
# ~/.config/malaysia/appearance.domain
color_scheme = "dark"
accent_color = [0.18, 0.44, 0.89]
window_color = "accent"
tint_parts = ["sidebars", "window", "groups", "controls", "menus"]
```

| Key | Type (`AppearanceConfig` field) | Values | Default |
|---|---|---|---|
| `color_scheme` | `ColorScheme` | `light`, `dark`, `dark_tint` | `light` |
| `accent_color` | `Option<[f32; 3]>` | RGB from 0 to 1; absent means automatic: the theme's accent, and apps keep their own | absent |
| `contrast` | `bool` | | `false` |
| `reduced_motion` | `bool` | | `false` |
| `reduce_transparency` | `bool` | | `false` |
| `scroll_bars` | `ScrollBars` | `automatic`, `when_scrolling`, `always` | `automatic` |
| `scroll_bar_click` | `ScrollBarClick` | `next_page`, `clicked_spot` | `next_page` |
| `window_color` | `WindowColorMode` | `neutral`, `accent` (an old `wallpaper` reads as `neutral`) | `neutral` |
| `tint_strength` | `TintStrength` | `subtle`, `normal`, `strong`, `intense` | `normal` |
| `tint_parts` | `TintParts` | list of `sidebars`, `window`, `groups`, `controls`, `menus`, `bars` (the menu bar and the Dock) | all but `bars` |
| `sidebar_style` | `SidebarStyle` | `island`, `flat` | `island` |
| `traffic_lights` | `TrafficLightStyle` | `auto`, `standard`, `accent` (an old `wallpaper` reads as `accent`) | `auto` |
| `traffic_light_shape` | `TrafficLightShape` | corner radius from 0 to 7 on the 14 light; the old `square` reads as 4 and `round` as 7 | `4` |
| `wallpaper_tinting` | `bool` | | `true` |
| `wallpaper_tinting_in_light` | `bool` | | `false` |
| `invert_colors` | `bool` | | `false` |
| `show_borders` | `bool` | | `false` |
| `differentiate_without_color` | `bool` | | `false` |
| `display_contrast` | `f32` | a value of `DISPLAY_CONTRAST` | `1.0` |
| `text_size` | `f32` | a value of `TEXT_SIZES` | `1.0` |
| `text_sizes` | `TextSizes` | table with optional `barrierkit`, `gtk`, `qt`, each a value of `TEXT_SIZES` | empty: every toolkit uses `text_size` |
| `menu_bar_size` | `MenuBarSize` | `default`, `large` | `default` |
| `shake_to_locate` | `bool` | | `true` |
| `pointer_size` | `f32` | a value of `POINTER_SIZES` | `1.0` |
| `color_filters` | `bool` | | `false` |
| `color_filter` | `ColorFilter` | `grayscale`, `red_green`, `green_red`, `blue_yellow` | `grayscale` |
| `color_filter_intensity` | `f32` | a value of `COLOR_FILTER_INTENSITY` | `1.0` |

Notes on the values:

- The old value `"auto"` reads as `light`; `is_dark()` is true for `dark` and
  `dark_tint`.
- A number outside its token list reads as the nearest end of the list.
- `TintStrength::factor()` multiplies the accent tint and the wallpaper tint by
  `TINT_STRENGTH`: 0.5, 1, 1.5 or 2.
- `scroll_bars = "automatic"` keeps the bar on screen after a mouse wheel and
  hides it after a touchpad scroll, like macOS «Automatically based on mouse or
  trackpad».
- `traffic_lights = "auto"` follows `window_color`
  (`AppearanceConfig::traffic_light_style`). The old value `"macos"` still
  reads as `standard`, with a warning.
- `TrafficLightShape::corner_radius(diameter)` gives the radius of a light of
  that size; `TrafficLightShape::new(radius)` stops at `ROUND`.
- The accessibility keys follow macOS System Settings › Accessibility ›
  Display. The kit applies three of them in `ControlStyle::accessible`:
  `show_borders` draws a `ControlBorder` line around icon buttons, tool groups
  and search fields; `differentiate_without_color` keeps the traffic-light
  symbols on while the window is focused; `text_size` scales `Text`, sidebar
  items (and their height) with `Font::scaled`. The compositor applies
  `invert_colors`, `display_contrast`, the color filter and the pointer keys;
  the shell applies `menu_bar_size` (malaysia ADR 0066).

### Reading and writing

| Function | What it does |
|---|---|
| `config_dir()`, `data_dir()` | `$MALAYSIA_CONFIG_DIR` or `$XDG_CONFIG_HOME/malaysia`; `$MALAYSIA_DATA_DIR` or `$XDG_DATA_HOME/malaysia`. |
| `appearance_path()` | `config_dir()/appearance.domain`. |
| `wallpaper_state_path()` | `data_dir()/wallpaper.state`, where the wallpaper's dominant color and grid live. |
| `read_appearance()` | `Ok(None)` without a file, `Err` when the TOML does not parse. |
| `load_appearance()` | The file's config, or the default on any error. |
| `write_appearance(path, config)` | Writes the config and keeps other keys and comments. A file that does not parse as an appearance config is first copied to `.domain.bak`. The file is replaced atomically. |
| `replace_file(path, text)` | Writes `text` to a temporary file beside `path`, syncs it and renames it over `path`. Creates the directory with mode `0700`. |
| `load_dominant()` | Reads `dominant` from `wallpaper.state`. |
| `write_wallpaper_state(path, &WallpaperState)` | Writes `dominant` and `grid`; an empty state removes the file. |
| `load_wallpaper_state()` | The whole `wallpaper.state`, or the default. `WallpaperState::grid_for(desktop)` picks a desktop's grid. |

### The wallpaper files

Malaysia's wallpaper lives in three files. System Settings writes the first two
and `malaysia-wallpaper` reads them and writes the third.

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

### Resolution

`resolve(&config, dominant) -> Resolved` turns the config and the wallpaper's
dominant color into colors. `palette(scheme)` returns the base
`ctdesign::Palette` of a scheme (`LIGHT`, `DARK` or `DARK_TINT`).
`lights_for(style, accent, dominant)` returns the three traffic-light colors of
a style.

| `Resolved` field | What it holds |
|---|---|
| `scheme` | The `ColorScheme`. |
| `accent`, `on_accent` | The accent and its ink. |
| `automatic_accent` | True without `accent_color`: an `Accent::Own` or `Accent::FromBackdrop` keeps its color. |
| `focus_ring` | The accent moved until it reaches 3:1 on the window. |
| `link` | The accent moved until it reaches 4.5:1 on the window and on content. |
| `window_tint` | Tint color and amount, or `None`: the accent, or the wallpaper after `behind_window`. |
| `bar_tint` | Tint of the menu bar and the Dock with the `bars` part: the accent tint, or the dominant color by `wallpaper_tinting`. |
| `wallpaper_tinting` | Amount of the wallpaper behind a window, or `None` when it is off. `Resolved::behind_window(color)` tints the `tint_parts` with that color, as an accent tint does. |
| `tint_parts` | Copy of the config. |
| `lights`, `traffic_style`, `traffic_shape` | The traffic-light colors, style and shape. |
| `contrast`, `reduced_motion`, `scroll_bars`, `scroll_bar_click` | Copies of the config. |
| `reduce_transparency` | `AppearanceConfig::transparency_reduced()`: true with `reduce_transparency` or `contrast`. |
| `show_borders`, `differentiate_without_color`, `menu_bar_size` | Copies of the config. |
| `text_size` | `AppearanceConfig::text_size_for(Toolkit::BarrierKit)`: the BarrierKit size, or the preferred `text_size`. |
| `shell` | `ShellPalette`: menu bar and Dock colors. |
| `menu`, `menu_parent` | `MenuPalette` and the window color a menu sits on. `Resolved::menu_separator(panel_bg)` gives the separator over another background. |
| `sidebar` | `SidebarSurface`: `style`, `bg`, `rim`, `inner`, its `SidebarPalette` and `glass`. `glass` is a `SidebarGlass` with `base` (the solid tone, tinted with the accent when the accent is the window color), `solid` (`base` with the wallpaper tint) and `chroma` (the share of the wallpaper's chroma, `GLASS.accent_share` of it under an accent) and `rim_tint` (the accent, under an accent: the island's border takes its chroma from it instead of from the wallpaper), and is `None` with reduced transparency ([ADR 0065](../decisions/0065-sidebar-glass.md)). |
| `toolgroup` | `ToolgroupSurface`: `bg` and its `ToolgroupPalette`. |
| `surfaces` | `Surfaces`: `window`, `titlebar_inactive`, `sunken`, `surface`, `raised`, `control`, `hover`, `line`, already tinted. |

The rules `resolve` applies:

1. `reduce_transparency` or `contrast` picks the `_OPAQUE` variants of the
   shell, menu, sidebar and toolgroup colors. macOS turns Reduce transparency
   on with Increase contrast.
2. A tint mixes the tint color into each surface. The amount is
   `WINDOW_TINT_ACCENT` (light, dark), times the strength factor, times the
   `LAYER_FACTOR_*` of that layer. A part left out of `tint_parts` stays
   neutral.
3. `on_accent` is always the light ink, as in macOS. An accent where it does
   not reach 4.5:1 (`ON_ACCENT_MIN_CONTRAST`) is darkened just enough
   (WCAG 2.2 §1.4.3).
4. Wallpaper tinting is on with `wallpaper_tinting` and a neutral
   `window_color`, with any accent (malaysia ADR 0065), unless
   `reduce_transparency` or `contrast` is on. It
   applies in dark schemes, and in light ones with
   `wallpaper_tinting_in_light`. The amount is `WALLPAPER_TINTING` (light,
   dark) times `tint_strength`, and it reaches the same `tint_parts` and
   `LAYER_FACTOR_*` as an accent tint, with `WALLPAPER_TINTING_SIDEBAR` for
   the sidebar. As in macOS it is a chroma mix in OKLab
   (`backdrop::take_chroma`): each gray keeps its lightness and takes that
   share of the wallpaper's hue. `sidebar.glass.solid` takes the same mix from
   `sidebar.glass.base` when `tint_parts` has `sidebars`.
5. Without `accent_color` the accent is the theme's, and `automatic_accent`
   lets an area keep its own.
6. Menu hover, sidebar selection and the shell's active tile take the accent.

### Wallpaper behind a window

Each window takes the average color of the wallpaper behind it into the
parts of `tint_parts`, with the strength of `tint_strength` (malaysia ADR
0065). `ctappearance::tinting::WallpaperGrid` does the same
from a grid of average colors:

1. `malaysia-wallpaper` builds it with `WallpaperGrid::from_rgba8(rgba8,
   width, height)`: `WALLPAPER_TINTING_GRID` cells per side and the image's
   `aspect`. It writes it to `wallpaper.state` next to `dominant`.
2. The compositor reads it with `load_wallpaper_state()` and takes
   `grid_for(desktop)`, the grid of the window's desktop. `grid.behind(output,
   window)` averages the cells behind the window rectangle, weighted by area,
   with the wallpaper scaled to cover the output.
3. The compositor sends that color to the window with the `wallpaper` event of
   `infinity_focus_surface_v1` (version 3).
4. The kit keeps one `Resolved` per window: the global one passed through
   `behind_window(color)`. It recomputes it when the appearance changes and,
   when the event brings a new color, fades to it over `DURATION_BASE` with a
   smoothstep, one step per frame callback. Reduce motion swaps it at once.

### Watching

`watch::AppearanceWatch::new()` watches `config_dir()` and `data_dir()` with
inotify. It is a file descriptor (`AsFd`) for an event loop; `drain()` returns
what changed as `AppearanceEvent`s:

| Event | File |
|---|---|
| `Appearance` | `appearance.domain` |
| `Wallpaper` | `wallpaper.state` |
| `WallpaperSelection` | `wallpaper-selection.state` in the data directory |
| `Defaults` | any other `.domain` in the config folder |

`restyles()` is true for `Appearance` and `Wallpaper`. The kit's runner already
uses it; an app needs it only with its own event loop.

### Menu bar

`menubar::MenubarConfig` is `menubar.domain`: `app_menus` (default `true`)
shows the active app's own menus. Malaysia's compositor and menu bar both read
it. `show_wifi`, `show_bluetooth`, `show_battery`, `show_battery_percentage`
and `show_vpn` (all `true`) choose the status menus of the bar; System
Settings writes them. `show_printer` (`true`) shows a printer while a print job
is queued, on the bar and on the lock screen.

`style` is a `MenubarStyle`: `solid`, `translucent` (default) or `floating`.
`bar_height` is a `MenubarHeight`: `slim` (default, `MENUBAR_HEIGHT`) or
`wide` (`MENUBAR_HEIGHT_WIDE`). `MenubarConfig::height()` returns the height
in logical pixels; the bar and the lock screen both use it. An unknown value
falls back to the default. `menubar::colors_over(wallpaper)` returns the
`MenubarColors` of a translucent or floating bar over that wallpaper color:
`ink`, `glass` and `selection`, the translucent pill behind an open menu
title. It takes the `MENUBAR_OVER_LIGHT_*` set when the wallpaper, darkened by the
glass, is brighter than `MENUBAR_DARK_INK_ABOVE` (relative luminance) and
`MENUBAR_OVER_DARK_*` otherwise, whatever the color scheme.
`WallpaperGrid::across` returns the wallpaper color along the bar, blurred
horizontally by `MENUBAR_TINT_SPREAD` of the bar width.

### Window behavior

`window::read_window_behavior()` reads `config_dir()/window.domain` into a
`WindowBehavior`:

| Key | Type | Values | Default |
|---|---|---|---|
| `double_click_titlebar` | `DoubleClickAction` | `fill`, `minimize`, `zoom`, `none` | `zoom` |
| `green_button` | `GreenButtonAction` | `focus` (also `fullscreen`), `zoom`, `none` | `focus` |

`green_action(action, fixed_width)` and `double_click_action(action, zoomable)`
turn them into a `TitlebarAction`: `Focus`, `Maximize`, `Minimize` or `None`.
A fixed-width window maximizes instead of entering focus mode; a double click
never enters focus mode. The titlebar uses them ([9](09-titlebars.md)).

`window::write_double_click_titlebar(path, action)` rewrites only
`double_click_titlebar` in that file and keeps the compositor's other keys.
System Settings uses it.

### Dock

`dock::DockConfig` is `dock.domain`. Its keys are those of
`defaults write com.apple.dock`:

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
| `launchanim` | bool | `true` |
| `autohide-delay` | seconds | `0.0` |
| `autohide-time-modifier` | factor | `1.0` |

`DockConfig::tilesize()` clamps `tilesize` to `DOCK_TILESIZE_MIN` and
`DOCK_TILESIZE_MAX`. `largesize()` is the magnified size: at least
`DOCK_MAGNIFY_MIN` times `tilesize()`, so icons still grow at the largest size,
and equal to `tilesize()` without magnification. An unknown orientation or
effect falls back to the default.

### Traffic-light geometry

`ctappearance::traffic` holds the geometry and look of the close, minimize and
zoom lights, shared by the kit and the compositor:

| Item | What it does |
|---|---|
| `inset(bar_height)` | Distance of the lights from the left and top edges of a bar. |
| `button_rect(index) -> (x, y, size)` | Rectangle of light `index` in a server-side titlebar. |
| `group_rect() -> (x, y, width, height)` | Rectangle of the three lights. |
| `hit(point) -> Option<ButtonKind>` | The light under a point. |
| `TrafficInput` | `focused`, `group_hovered`, `pressed`, `zoom_supported`. |
| `visual(lights, style, inactive, rim, input) -> [ButtonVisual; 3]` | Fill, rim and glyph color of each light for that input. |
| `ButtonVisual` | `fill`, `rim`, `glyph`; `ButtonVisual::idle(fill, rim)` is a light at rest. |
| `glyph_svg(index, color, fixed_width, pixels)` | SVG of the glyph of a light; a fixed-width window gets a plus instead of the zoom arrows. |
| `TrafficPointer` | Pointer state machine: `on_motion`, `on_group_enter`, `on_button_enter`, `on_leave_button`, `on_press`, `on_release` (returns the clicked light), `on_leave`, `hovered`, `pressed_visual`. |

```rust
// ✗ geometry computed again
let x = 8.0 + i as f64 * (12.0 + 8.0);
// ✓ the shared function
let (x, y, d) = ctappearance::traffic::button_rect(i);
```

### Backdrop colors

`ctappearance::backdrop` derives colors from an image, for tinted backdrops
([15](15-backdrops-and-media.md)) and for the wallpaper's dominant color:

| Item | What it does |
|---|---|
| `palette_from_pixels(rgba8) -> Option<ImagePalette>` | Three representative colors (`colors`) and the most vivid one (`vivid`), by k-means in OKLab. |
| `dominant_from_pixels(rgba8) -> Option<Rgba>` | The dominant chromatic color, with lightness kept inside `DOMINANT_LIGHTNESS`. `None` for a grey image. |
| `Backdrop` | Three gradient `layers` and a `base`. `from_colors`, `solid`, `mix`, `average`, `ink`, `accent(vivid)`, `highlights`. |
| `ink_on(background) -> Rgba` | The light or dark ink with more contrast on that background. |
| `Oklab` | A color in OKLab: `from_rgba`, `to_rgba`. |

### Language

`ctappearance::language` finds the language the user asked for
([17](17-languages.md)):

| Function | What it does |
|---|---|
| `read_language(path)` | The `language = "…"` of a `.domain` file; an empty value is `None`. |
| `system_language()` | `read_language` of `config_dir()/language.domain`. |
| `requested_languages(app_language)` | The app's language, else the system's, else `LANGUAGE` followed by the first of `LC_ALL`, `LC_MESSAGES`, `LANG`, as tags such as `es-MX`. The `C` locale requests nothing. |

### Directories

`ctappearance::xdg` follows the XDG Base Directory specification and ignores
relative paths:

| Function | Returns |
|---|---|
| `config_home()` | `$XDG_CONFIG_HOME` or `~/.config` |
| `data_home()` | `$XDG_DATA_HOME` or `~/.local/share` |
| `data_dirs()` | `$XDG_DATA_DIRS` or `/usr/local/share` and `/usr/share` |
| `user_dir(kind)` | `XDG_<kind>_DIR` from `user-dirs.dirs`, for example `user_dir("DOWNLOAD")` |

`ctappearance::ini` reads and edits the INI files that GTK 3 (`settings.ini`)
and qt5ct and qt6ct read, and keeps every other line:

| Function | Does |
|---|---|
| `value(ini, section, key)` | The value of `key` in `[section]`, or `None` |
| `set(ini, section, key, Some(value))` | Sets the key in place, adding the section if missing |
| `set(ini, section, key, None)` | Removes the key |

## Tokens: `ctdesign`

`ctdesign` is Tengen in Rust. `build.rs` reads `tengen/tokens.json` and
`tengen/components.json` and writes one constant per token into
`$OUT_DIR/tengen.rs`. The struct of each constant is in `src/lib.rs`. An app
imports them from `ctdesign`; the prelude re-exports the type and spacing
tokens.

### Adding a token

A color, size or duration never goes into `bk`. It goes into Tengen:

1. Add the key: under its component in `components.json`, or as a base value
   in `tokens.json`. A color per theme goes under `themes.light`,
   `themes.dark` and `themes.dark-tint`.
2. A new field of an existing struct: add `pub name: Type,` to the struct in
   `src/lib.rs`. `build.rs` finds the key by name (`_` becomes `-`, and
   prefixes match nested objects).
3. A new constant: add a line `NAME Type json.path` to `CONSTANTS` in
   `build.rs`, plus its struct in `src/lib.rs` if it is a new type. A line in
   `THEMED` makes one constant per theme (`_LIGHT`, `_DARK`, `_DARK_TINT`).
   Write `field=path` after the path when a key has another name.
4. Use it: `SPLIT_VIEW.key_step`.
5. Add its row to the [glossary](20-glossary.md).
6. Run `./dev.sh check`.

### Token types

| Type | What it is |
|---|---|
| `Rgba` | sRGB color with straight alpha, channels from 0 to 1. `TRANSPARENT`, `rgb8`, `rgba8`, `to_rgba8`, `premultiplied`, `mix` (keeps its own alpha), `darken`, `lighten`, `to_linear`, `from_linear`, `relative_luminance`, `contrast_ratio`, `to_hsl`, `from_hsl`. |
| `Font` | `family`, `size`, `line_height` (pixels) and `weight`. |
| `Palette` | Base colors of a theme: `titlebar_bg`, `hairline`, `inner_line`, `panel_inner_line`, `outer_border`, `title`, `text`, `ink_strong`, `ink_muted`, `ink_faint`, `line`, `line_strong`, `accent`, `accent_inactive`, `on_accent`, `danger`, `traffic_lights_inactive`, `traffic_rim`, `scrim`, `selected_control`, `card_border`. |
| `FocusPair` | A color for the focused window and one for the unfocused. |
| `PanelLine` | Inner line of a focused panel, menu or dialog: `sides`, `edges` (top and bottom) and the `glow` under the edges. |
| `TrafficRim` | Rim of a traffic light: `lit` and `idle`. |
| `ShellPalette` | Menu bar and Dock: `bar`, `dock` (the Dock's glass), `text`, `tile`, `tile_active`, `tile_text_active`. |
| `MenuPalette` | Colors and metrics of a menu: background, padding, item size, inks, hover, separator, check mark, header and search field. |
| `SidebarPalette` | Sidebar inks: item, icon, selection, inactive selection, hover, section, count, line. |
| `ToolgroupPalette` | Toolbar group: `separator`, `rim`. |
| `ShadowLayer` | `blur`, `y_offset`, `opacity`, `color`. |
| `MaterialThickness` | `opacity` and `blur` of one material thickness. |
| `ShadowStyle` | `ambient` and `contact` layers and their scale when unfocused; `layers(focused)` returns both. |
| `WindowCorner` | Window radii (`titlebar`, `unified_compact`, `unified`, `utility`), curve `expansion` and superellipse `exponent`. |
| `WindowStyle` | `corner_radius` and `superellipse_exponent` of a dialog or pop-up. |
| `WindowResize` | Width of the resize band at an `edge` and a `corner`. |
| `Titlebar` | Server-side titlebar: `thickness`, `buttons`, `font`. |
| `TitlebarButtons` | `order` of the three lights, `diameter`, `spacing`, `corner_radius`. |
| `Button` | One traffic light: its `kind` and `color`. |
| `ButtonKind` | `Close`, `Minimize`, `Maximize`. |
| `TrafficGlyphs` | Glyph geometry: `box_size`, `stroke`, `close`, `minimize`, `zoom`, `plus`. |
| `Segment`, `Triangle` | A line from `a` to `b`; three points. |
| `DoubleClick` | `interval_ms` and pointer `slop` of a double click. |
| `SettingsStrength`, `SettingsCheck`, `SettingsWallpaper` | Parts of `SettingsMetrics`: the tint-strength scale, the check list and the Wallpaper pane (preview, tiles, color dots, the collection sheet and the custom framing sheet with its desktop preview). |
| `*Metrics` | Sizes of one component; see the tables below. |

### Spacing

| Constant | Value |
|---|---|
| `SPACE_1`, `SPACE_2`, `SPACE_3`, `SPACE_4` | 4, 8, 12, 16 |
| `HAIRLINE` | 1, the width of a separator |

### Type

| Constant | Size / line height | Weight | SwiftUI |
|---|---|---|---|
| `LARGE_TITLE` | 26 / 32 | 700 | `Font.TextStyle.largeTitle` |
| `TITLE` | 22 / 26 | 650 | `.title` |
| `TITLE_2` | 15 / 20 | 600 | `.title2` |
| `HEADLINE` | 13 / 18 | 700 | `.headline` |
| `BODY` | 13 / 18 | 400 | `.body` |
| `CALLOUT` | 12 / 16 | 400 | `.callout` |
| `CAPTION` | 11 / 14 | 500 | `.caption` |
| `MONO` | 13 / 16 | 400 | `Font.monospaced()` |

All use Adwaita Sans, the sans of GNOME, so `bk` text matches GTK; `MONO`
uses Geist Mono. `SANS_FAMILIES` is the fallback list (Adwaita Sans, Inter,
Noto Sans CJK SC, system-ui, sans-serif). `TEXT_LINE_HEIGHT` (1.3) is the
factor that turns a font size into the height of one line when
the kit measures text without laying it out.

### Motion

| Constant | Value |
|---|---|
| `DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS` | 120, 220, 420 ms |
| `THEME_FADE_WAIT_MS` | 150 ms: longest wait for clients before the compositor's theme fade |
| `SPRING_DAMPING` | 5, damping of the kit's spring ([14](14-animations.md)) |

### Materials

`MATERIAL: MaterialMetrics` holds one `MaterialThickness` per
`styles::Thickness`. A `ShapeStyle::Material` paints the glass tint of its
context (the window color, or the `Backdrop` base) at `opacity` and blurs what
lies under the part by `blur`. With Reduce Transparency the tint is opaque and
nothing is blurred.

| Field | `opacity` | `blur` |
|---|---|---|
| `ultra_thin` | 0.35 | 12 |
| `thin` | 0.5 | 20 |
| `regular` | 0.65 | 28 (`MEDIA.material_blur`) |
| `thick` | 0.8 | 36 |
| `ultra_thick` | 0.92 | 44 |

### Colors

| Constants | Type | What they are |
|---|---|---|
| `LIGHT`, `DARK`, `DARK_TINT` | `Palette` | Base palette of each theme. |
| `BG_WINDOW_LIGHT`, `BG_WINDOW_DARK`, `BG_WINDOW_DARK_TINT` | `Rgba` | Window background. |
| `BG_SURFACE_LIGHT`, `BG_SURFACE_DARK`, `BG_SURFACE_DARK_TINT` | `Rgba` | Content and group background. |
| `BG_SUNKEN_LIGHT`, `BG_SUNKEN_DARK`, `BG_SUNKEN_DARK_TINT` | `Rgba` | Sunken background. |
| `BG_RAISED_LIGHT`, `BG_RAISED_DARK`, `BG_RAISED_DARK_TINT` | `Rgba` | Raised background (menus). |
| `BG_CONTROL_LIGHT`, `BG_CONTROL_DARK`, `BG_CONTROL_DARK_TINT` | `Rgba` | Control face. |
| `BG_HOVER_LIGHT`, `BG_HOVER_DARK`, `BG_HOVER_DARK_TINT` | `Rgba` | Control face under the pointer. |
| `LAYER_FACTOR_BG_SUNKEN`, `LAYER_FACTOR_SURFACE`, `LAYER_FACTOR_RAISED`, `LAYER_FACTOR_CONTROL`, `LAYER_FACTOR_LINE` | `f32` | Share of the window tint each layer takes: 1.4, 1, 0.6, 1.2, 1.2. |
| `WINDOW_TINT_ACCENT` | `[f32; 2]` | Accent tint amount in light and dark. |
| `WALLPAPER_TINTING`, `WALLPAPER_TINTING_SIDEBAR`, `WALLPAPER_TINTING_GRID` | `[f32; 2]`, `f32`, `u32` | Share of the wallpaper's OKLab chroma a window takes in light and dark, measured on macOS 26 (ADR 0065); how much more the sidebar takes; cells per side of the wallpaper grid. |
| `TINT_STRENGTH` | `[f32; 4]` | Factor of each `TintStrength`: 0.5, 1, 1.5, 2. |
| `ACCENT_MIN_CONTRAST_ON_WINDOW`, `ON_ACCENT_MIN_CONTRAST` | `f32` | 3 and 4.5: contrast of the accent on the window and of ink on the accent. |
| `DOMINANT_LIGHTNESS`, `DOMINANT_MIN_CHROMA` | `[f32; 2]`, `f32` | Lightness range and minimum chroma of a wallpaper's dominant color. |
| `ACCENT_SWATCHES` | `[(&str, Rgba); 8]` | The accent choices: blue, purple, pink, red, orange, yellow, green, graphite. |
| `MULTICOLOR` | `[Rgba; 8]` | The stops of the rainbow swatch, from the accent swatches, red to red. |
| `CHART_SERIES` | `[Rgba; 7]` | Series colors of charts ([16](16-charts.md)). |
| `LEVEL_INDICATOR_COLORS` | `[Rgba; 3]` | Normal, warning and critical colors of a level indicator. |
| `SHELL_LIGHT`, `SHELL_LIGHT_OPAQUE`, `SHELL_DARK`, `SHELL_DARK_OPAQUE`, `SHELL_DARK_TINT`, `SHELL_DARK_TINT_OPAQUE` | `ShellPalette` | Menu bar and Dock, translucent and opaque. |
| `THEME_SHADOW_OPACITY` | `f32` | Shadow opacity of `use_shell_theme`. |
| `MENU_LIGHT`, `MENU_LIGHT_OPAQUE`, `MENU_DARK`, `MENU_DARK_OPAQUE`, `MENU_DARK_TINT`, `MENU_DARK_TINT_OPAQUE` | `MenuPalette` | Menus, translucent and opaque. |
| `SIDEBAR_LIGHT`, `SIDEBAR_LIGHT_OPAQUE`, `SIDEBAR_DARK`, `SIDEBAR_DARK_OPAQUE`, `SIDEBAR_DARK_TINT`, `SIDEBAR_DARK_TINT_OPAQUE` | `Rgba` | Flat sidebar background. |
| `SIDEBAR_ISLAND_LIGHT`, `SIDEBAR_ISLAND_DARK`, `SIDEBAR_ISLAND_DARK_TINT` | `Rgba` | Island sidebar background. |
| `SIDEBAR_GLASS_BASE_LIGHT`, `SIDEBAR_GLASS_BASE_DARK`, `SIDEBAR_GLASS_BASE_DARK_TINT` | `Rgba` | Solid tone of the sidebar glass, with the lightness of the app's window. Only the dark one is measured; the other two are estimates. |
| `SIDEBAR_GLASS_RIM_LIGHT`, `SIDEBAR_GLASS_RIM_DARK`, `SIDEBAR_GLASS_RIM_DARK_TINT` | `Rgba` | White line of the sidebar glass's rim; the compositor tints it with the wallpaper's chroma. Dark mode is measured (alpha 0.04 on average, 0.10 at most); light is an estimate. |
| `SIDEBAR_ISLAND_RIM_LIGHT`, `SIDEBAR_ISLAND_RIM_DARK`, `SIDEBAR_ISLAND_RIM_DARK_TINT` | `Rgba` | Island sidebar outer rim. |
| `SIDEBAR_ISLAND_INNER_LIGHT`, `SIDEBAR_ISLAND_INNER_DARK`, `SIDEBAR_ISLAND_INNER_DARK_TINT` | `Rgba` | Island sidebar inner line. |
| `SIDEBAR_PALETTE_LIGHT`, `SIDEBAR_PALETTE_DARK`, `SIDEBAR_PALETTE_DARK_TINT` | `SidebarPalette` | Sidebar inks per theme. |
| `TOOLGROUP_BG_LIGHT`, `TOOLGROUP_BG_LIGHT_OPAQUE`, `TOOLGROUP_BG_DARK`, `TOOLGROUP_BG_DARK_OPAQUE`, `TOOLGROUP_BG_DARK_TINT`, `TOOLGROUP_BG_DARK_TINT_OPAQUE` | `Rgba` | Toolbar group background. |
| `TOOLGROUP_PALETTE_LIGHT`, `TOOLGROUP_PALETTE_DARK`, `TOOLGROUP_PALETTE_DARK_TINT` | `ToolgroupPalette` | Toolbar group lines per theme. |

An `_OPAQUE` constant is its translucent twin painted over the theme's window
background, for `reduce_transparency`.

### Windows and titlebars

| Constant | Type | Read by |
|---|---|---|
| `WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` | `(u32, u32)` | `WindowOptions`: 480 × 360 and 640 × 420 ([8](08-windows.md)) |
| `WINDOW_CORNER` | `WindowCorner` | `style::continuous_corner`, the compositor |
| `WINDOW_RESIZE` | `WindowResize` | `platform` |
| `WINDOW_CLIENT_RESIZE` | `WindowResize` | The compositor: the resize band around a window that decorates itself |
| `DIALOG`, `DIALOG_PADDING`, `DIALOG_ALERT_WIDTH` | `WindowStyle`, `f32`, `f32` | `views::feedback` |
| `POPUP` | `WindowStyle` | The compositor, for pop-up corners |
| `PANEL_GLOW_DEPTH` | `f32` | The compositor: depth in points of the `PanelLine::glow` |
| `SHADOW`, `SHADOW_DARK` | `ShadowStyle` | `styles` (`WindowChromeStyle`), the compositor |
| `NATIVE_APP_ID_PREFIX` | `&str` | `io.github.hblanqueto.malaysia.`: app ids of the kit's apps (`platform`, `titlebar`) |
| `ALERT_LAYER_NAMESPACE` | `&str` | `malaysia-dialog`: layer-shell namespace of shell alerts |
| `NOTIFICATIONS_LAYER_NAMESPACE` | `&str` | `malaysia-notifications`: layer-shell namespace of banners and Notification Center |
| `TITLEBAR` | `Titlebar` | Server-side titlebar of the compositor, `ctappearance::traffic` |
| `TITLEBAR_CLIENT_HEIGHT`, `TITLEBAR_PADDING_X` | `u32`, `f32` | `styles` (`TitlebarStyle`), `chrome` ([9](09-titlebars.md)) |
| `TITLEBAR_TITLE_GAP` | `f32` | The compositor's titlebar |
| `TITLEBAR_DOUBLE_CLICK` | `DoubleClick` | `titlebar`, the compositor |
| `TITLEBAR_RAIL` | `TitlebarRailMetrics` | `chrome`, `styles` (`WindowChromeStyle`); `title_angle` (−90) turns the rail title |
| `FULLSCREEN_BAR`, `FULLSCREEN_GLASS`, `FULLSCREEN_BAND` | `FullscreenBarMetrics`, `FullscreenGlassMetrics` | `platform`, `styles` (`FullscreenBarStyle`, `TopGlassStyle`) |
| `GLASS_MASK` | `Rgba` | Base color of the mask that fades glass bars |
| `HUD_BACKGROUND` | `Rgba` | Background of a HUD window (`platform`) |

### Traffic lights

| Constant | Type | What it is |
|---|---|---|
| `TRAFFIC_GLYPHS` | `TrafficGlyphs` | Geometry of the close, minimize, zoom and plus glyphs. |
| `TRAFFIC_GLYPH_STANDARD`, `TRAFFIC_GLYPH_ON_DARK`, `TRAFFIC_GLYPH_ON_LIGHT` | `Rgba` | Glyph ink for the standard style and on tinted lights. |
| `TRAFFIC_RIM_WIDTH` | `f32` | Rim width. |
| `TRAFFIC_PRESSED_DARKEN` | `f32` | Darkening of a pressed light. |
| `TRAFFIC_CLOSE_MINIMIZE_DARKEN`, `TRAFFIC_ZOOM_LIGHTEN` | `f32` | Shifts that derive tinted lights from one color. |

### Control metrics

| Constant | Type | Read by |
|---|---|---|
| `BUTTON` | `ButtonMetrics` | `controls`, `compact`, `views::window_tabs`, `font_panel` |
| `ICON_BUTTON` | `IconButtonMetrics` | `compact`, `views::window_tabs` |
| `SWITCH` | `SwitchMetrics` | `controls`, `choices`, `level` |
| `CHECKBOX` | `CheckboxMetrics` | `controls` |
| `SLIDER`, `SLIDER_FAT`, `SLIDER_KEY_STEP` | `SliderMetrics`, `SliderFatMetrics`, `f32` | `styles`, `level` |
| `KNOB_SHADOW` | `ShadowLayer` | `level`, `styles` |
| `MATERIAL` | `MaterialMetrics` | `styles` ([Materials](#materials)) |
| `SEGMENTED` | `SegmentedMetrics` | `controls`, `compact` |
| `STEPPER` | `StepperMetrics` | `styles` |
| `TOOLBAR_SEARCH_WIDTH` | `f32` | `compact` |
| `TEXT_FIELD` | `TextFieldMetrics` | `controls`, `sidebar` |
| `TOKEN_FIELD` | `TokenFieldMetrics` | `styles` |
| `SPELLING` | `SpellingMetrics` | `spelling` |
| `COMBO_BOX` | `ComboBoxMetrics` | `styles` |
| `PATH_CONTROL` | `PathControlMetrics` | `styles` |
| `DATE_PICKER` | `DatePickerMetrics` | `styles` |
| `IMAGE_WELL` | `ImageWellMetrics` | `styles` |
| `LEVEL_INDICATOR`, `GAUGE` | `LevelIndicatorMetrics`, `GaugeMetrics` | `styles`; `indicator` reads the number of stars |
| `PROGRESS` | `ProgressMetrics` | `styles` |
| `BADGE` | `BadgeMetrics` | `styles` (`BadgeStyle`) |
| `FOCUS_RING` | `FocusRingMetrics` | `interaction` |
| `SYMBOL` | `SymbolMetrics` | `symbol`, `styles` |
| `TOOLTIP` | `TooltipMetrics` | `tooltip`, `styles` (`ChartStyle`) |
| `POPOVER` | `PopoverMetrics` | `popover`, the compositor |
| `MENU_POPOVER` | `MenuPopoverMetrics` | `theme`, `popup_button`, `styles` |
| `MENU_MARK_TEXT_GAP`, `MENU_SUBMENU_INTENT_MS`, `MENU_SUBMENU_REVEAL_GAP`, `MENU_SCROLL_TICK_MS`, `MENU_SEARCH_ICON`, `MENU_SEARCH_ICON_GAP`, `MENU_SEARCH_CLEAR`, `MENU_SEARCH_CLEAR_GLYPH`, `MENU_SEARCH_PADDING_X`, `MENU_SEARCH_CLEAR_MIX`, `MENU_HELP_ICON` | `f32`, `u64`, `[f32; 2]` | `menu` ([7](07-menus-and-popovers.md)) |
| `MENU_SEPARATOR_INK_MIX` | `f32` | `Resolved::menu_separator` |
| `DRAG` | `DragMetrics` | `drag` |

### View metrics

| Constant | Type | Read by |
|---|---|---|
| `GLASS` | `GlassMetrics` | `chroma_light` (0.25) and `chroma_dark` (0.40) and `accent_share` (0.35): the part of `chroma` that stays when the accent tints the sidebar; `rim_chroma` (0.65): the share of the wallpaper's OKLab chroma that the sidebar's glass and its rim take; `chroma_cap_light` (0.05) and `chroma_cap_dark` (0.04): the chroma the glass saturates towards (`backdrop::take_chroma_capped`), so a vivid wallpaper tints it without turning it into a window; `blur_passes` (4): how much wider than the other blurs the glass blurs the wallpaper ([ADR 0065](../decisions/0065-sidebar-glass.md)) |
| `SIDEBAR` | `SidebarMetrics` | `sidebar`, `styles` (the sidebar styles); `item_weight` (500) and `item_selected_weight` (600) weigh the row labels |
| `TOOLGROUP` | `ToolgroupMetrics` | `controls`, `compact` |
| `SPLIT_VIEW` | `SplitViewMetrics` | `views::split`, `styles` |
| `SPLIT_PANES` | `SplitPanesMetrics` | `views::split_panes`, `styles` (`SplitPanesStyle`) |
| `FLOATING_PANE` | `FloatingPaneMetrics` | `views::floating_pane`, `styles` (`FloatingPaneStyle`) |
| `TABLE` | `TableMetrics` | `views::table` |
| `COLLECTION` | `CollectionMetrics` | `views::collection` |
| `LIST_GROUP` | `ListGroupMetrics` | `views::form` |
| `GROUP_BOX` | `GroupBoxMetrics` | `views::group` |
| `DISCLOSURE` | `DisclosureMetrics` | `views::group`, `views::tree`, `font_panel`, `styles` (`RuleEditorStyle`) |
| `TOOLBAR_TABS` | `ToolbarTabsMetrics` | `views::tabs` |
| `SCROLL_BAR` | `ScrollBarMetrics` | Scroll views, `styles` (`ScrollViewStyle`) |
| `BREAKPOINT` | `BreakpointMetrics` | `adaptive`: 620 and 900 px |
| `SETTINGS` | `SettingsMetrics` | `patterns::settings`, `sidebar`, `styles` |
| `FONT_PANEL` | `FontPanelMetrics` | `font_panel` |
| `PRINT` | `PrintMetrics` | `print` |
| `CHART` | `ChartMetrics` | `charts`, `styles` (`ChartStyle`); `hover_opacity` is the highlight of the hovered row of horizontal bars |
| `MEDIA` | `MediaMetrics` | `media`, `styles`; `button_font_weight` is the weight of the `MediaButton` label |
| `BACKDROP` | `BackdropMetrics` | `backdrop`, `sidebar`, `ctappearance::backdrop`, `styles` (`BackdropStyle`) |
| `BACKDROP_SCENES` | `BackdropScenes` | Preset gradient `dusk` (recipe `titlebar_over_gradient`) |

### Shell

`malaysia-shell` reads these; the kit does not, except `MENUBAR_HEIGHT` for the
focus-mode bar.

| Constants | What they size |
|---|---|
| `MENUBAR_HEIGHT`, `MENUBAR_HEIGHT_WIDE`, `MENUBAR_TINT_SPREAD`, `MENUBAR_DARK_INK_ABOVE`, `MENUBAR_OVER_LIGHT_INK`, `MENUBAR_OVER_LIGHT_GLASS`, `MENUBAR_OVER_LIGHT_SELECTION`, `MENUBAR_OVER_DARK_INK`, `MENUBAR_OVER_DARK_GLASS`, `MENUBAR_OVER_DARK_SELECTION`, `MENUBAR_FONT_SIZE`, `MENUBAR_APP_NAME_WEIGHT`, `MENUBAR_MARGIN`, `MENUBAR_MARGIN_RIGHT`, `MENUBAR_FADED_ALPHA`, `MENUBAR_TITLE_PADDING_X`, `MENUBAR_TEXT_LIFT`, `MENUBAR_TITLE_MAX_WIDTH`, `MENUBAR_SYSTEM_MENU_MIN_WIDTH`, `MENUBAR_TITLE_BLEED` | The menu bar |
| `FOCUS_BAR_BACKGROUND`, `FOCUS_BAR_TEXT` | The menu bar in focus mode |
| `DOCK_TILESIZE`, `DOCK_TILESIZE_MIN`, `DOCK_TILESIZE_MAX`, `DOCK_LARGESIZE` | Dock icon sizes: default, 16 and 128 px, and the default magnified size |
| `DOCK_EDGE_MARGIN`, `DOCK_SIDE_MARGIN`, `DOCK_PADDING`, `DOCK_PADDING_SCALE`, `DOCK_RADIUS` | Dock bar: gap to the screen edge, side margin as a share of the screen, padding = `DOCK_PADDING` + `DOCK_PADDING_SCALE` × size, corner radius as a share of its thickness |
| `DOCK_SEPARATOR`, `DOCK_SEPARATOR_LENGTH`, `DOCK_SEPARATOR_INK_MIX`, `DOCK_MAGNIFY_RADIUS`, `DOCK_MAGNIFY_MIN`, `DOCK_MAGNIFY_MS` | Separator slot and line as shares of the size, the share of ink mixed into the glass for the line; magnification reach in icons, the least it grows as a multiple of the size, and its fade |
| `DOCK_DOT`, `DOCK_DOT_MIN`, `DOCK_DOT_MAX`, `DOCK_PROGRESS_HEIGHT`, `DOCK_WINDOW_FRAME`, `DOCK_WINDOW_ICON`, `DOCK_WINDOW_BADGE` | Open-app dot, progress bar, the frame and icon of a minimized window before its picture arrives, and the app icon on the picture |
| `DOCK_ICON_BODY`, `DOCK_FOLDER_CANVAS`, `DOCK_RIM_DARK`, `DOCK_RIM_BRIGHT`, `DOCK_RIM_BLUR`, `DOCK_LAYER_NAMESPACE` | The share of a slot an icon's body takes (macOS's 824 on 1024), the share its canvas takes for the Downloads folder so that the folder is as tall as that body, how strong the Dock's inner line is over a dark and over a bright wallpaper, the blur that softens that line, and the Dock's layer-shell namespace |
| `DOCK_BOUNCE_MS`, `DOCK_BOUNCE_HEIGHT`, `DOCK_LAUNCH_TIMEOUT_MS`, `DOCK_HIDE_MS`, `DOCK_REVEAL_STRIP`, `DOCK_RECENTS`, `DOCK_MINIMIZE_MS`, `DOCK_SLOT_MS`, `DOCK_MINIMIZE_HANDOFF_MS` | Launch bounce, automatic hiding, the number of recent apps, the minimize effect, how long a minimized window's slot takes to open or close, and how long the compositor keeps drawing the finished thumbnail while the Dock's tile takes over |
| `DOCK_LABEL_GAP`, `DOCK_LABEL_FONT_SIZE`, `DOCK_LABEL_PADDING_X`, `DOCK_LABEL_PADDING_Y`, `DOCK_LABEL_RADIUS` | The name label over the icon under the pointer |
| `NOTIFICATION_WIDTH`, `NOTIFICATION_RADIUS`, `NOTIFICATION_PADDING`, `NOTIFICATION_GAP`, `NOTIFICATION_MARGIN`, `NOTIFICATION_ICON_SIZE`, `NOTIFICATION_TEXT_GAP`, `NOTIFICATION_CLOSE_SIZE`, `NOTIFICATION_STACK_PEEK`, `NOTIFICATION_STACK_INSET`, `NOTIFICATION_BODY_LINES`, `NOTIFICATION_MAX_BANNERS`, `NOTIFICATION_BANNER_MS`, `NOTIFICATION_SLIDE_IN_MS`, `NOTIFICATION_SLIDE_OUT_MS`, `NOTIFICATION_SLIDE_BOUNCE`, `NOTIFICATION_EDGE`, `NOTIFICATION_SWIPE`, `NOTIFICATION_WIDGET_SIZE`, `NOTIFICATION_WIDGET_RADIUS`, `LIVE_ACTIVITY_WIDTH`, `LIVE_ACTIVITY_HEIGHT`, `LIVE_ACTIVITY_RADIUS`, `LIVE_ACTIVITY_GAP`, `LIVE_ACTIVITY_ENDED_MS` | Notification banners, Notification Center, its widgets and the Live Activities of the menu bar (ADR 0068 in malaysia; the banner radius is provisional until measured on Tahoe, the Live Activity size is measured on Apple's WWDC25 video) |
| `LAUNCHER_WIDTH`, `LAUNCHER_HEIGHT`, `LAUNCHER_PADDING`, `LAUNCHER_SEARCH_HEIGHT`, `LAUNCHER_RESULT_HEIGHT`, `LAUNCHER_RESULT_GAP`, `LAUNCHER_RESULT_RADIUS`, `LAUNCHER_ICON_SIZE`, `LAUNCHER_MAX_RESULTS`, `LAUNCHER_CORNER_RADIUS`, `LAUNCHER_FONT_SIZE`, `LAUNCHER_SUBTITLE_FONT_SIZE`, `LAUNCHER_SCALE_FROM`, `LAUNCHER_OPEN_MS`, `LAUNCHER_CLOSE_MS` | The launcher |
| `LOCK_CLOCK_FONT_SIZE`, `LOCK_DATE_FONT_SIZE`, `LOCK_TOP_PERCENT`, `LOCK_BOTTOM_MARGIN`, `LOCK_SPACING`, `LOCK_AVATAR_SIZE`, `LOCK_KEYBOARD_WIDTH`, `LOCK_FIELD_HEIGHT`, `LOCK_FIELD_WIDTH`, `LOCK_MASK_FONT_SIZE`, `LOCK_INK`, `LOCK_FIELD_FILL`, `LOCK_AVATAR_FILL`, `LOCK_FIELD_ERROR`, `LOCK_FIELD_INK` | The lock screen over the wallpaper (Malaysia ADR 0074) |
| `LOCK_FADE_MS`, `LOCK_SETTLE_MS`, `LOCK_RISE_MS`, `LOCK_RISE_OFFSET`, `LOCK_LOGIN_MS` | The transition from the desktop to the lock screen: the fade, the menu bar closing up, the clock rising and the login appearing |
| `ABOUT_WIDTH`, `ABOUT_HEIGHT`, `ABOUT_DRAWING_TOP`, `ABOUT_DRAWING_WIDTH`, `ABOUT_DRAWING_HEIGHT`, `ABOUT_TITLE_GAP`, `ABOUT_TITLE_SIZE`, `ABOUT_SPECS_GAP`, `ABOUT_SPEC_ROW_GAP`, `ABOUT_COLUMN_GAP`, `ABOUT_VALUE_WIDTH`, `ABOUT_BUTTON_GAP`, `ABOUT_FOOTER_BOTTOM` | The «About this computer» window |
| `ABOUT_APP_WIDTH`, `ABOUT_APP_HEIGHT`, `ABOUT_APP_ICON` | The «About ‹app›» window and its icon |
| `CONFIRM_WIDTH`, `CONFIRM_HEIGHT`, `CONFIRM_BUTTON_GAP`, `CONFIRM_ICON`, `CONFIRM_GLYPH` | The restart, shut down and log out confirmation: its size, the gap between its buttons, and its round icon and glyph |
| `FORCE_QUIT_WIDTH`, `FORCE_QUIT_HEIGHT`, `FORCE_QUIT_ROW_HEIGHT`, `FORCE_QUIT_ICON` | The «Force Quit» window, its rows and their app icons |
| `MENUBAR_BATTERY_ICON`, `MENUBAR_BATTERY_LOW` | The battery in the menu bar: its size, and the percentage from which it turns red, as on macOS |

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

## Helpers

| Item | What it does |
|---|---|
| `init_tracing()` | Installs the `tracing` subscriber: logs to stderr filtered by `RUST_LOG` (default `info`), and to Tracy with the `tracy` feature ([19](19-profiling.md)). Call it once, first thing in `main`. |
| `ProbeExt::probe(id)` | Sets the accessibility id of a `rect`, so tests and tools can find the node: `rect().probe("sidebar-toggle")`. |

Next: [5. Controls](05-controls.md).
