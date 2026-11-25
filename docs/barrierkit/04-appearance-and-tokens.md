# 4. Appearance and tokens

**Goal:** paint a view with the system's colors, type, symbols and sizes, follow
the user's appearance without a restart, and add a token when one is missing.
**Scope:** from the Tengen JSON files to the palette a view paints with. It
covers `style`, `theme`, `symbol`, fonts, the whole of `ctdesign` and the
whole of `ctappearance`. Backdrops and their tints are in
[15](15-backdrops-and-media.md); traffic lights as a control in
[9](09-titlebars.md); the language of the text in [17](17-languages.md); how
`build.rs` reads the JSON in [Tengen](tengen.md). The desktop's own settings
(wallpaper, menu bar, Dock, notifications), glass and tokens are not part of
the kit.
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
| **Appearance**: mode, accent, tints, contrast, motion | The user | `~/.config/malaysia/appearance.domain` |
| **Variants** and **styles** of a control: a preset such as `ButtonStyle::bordered_prominent`, tint, tinted backdrop, a `SliderStyle` | The app | The app's code ([styles](../../.claude/skills/barrierkit-app/styles.md)) |

The kit resolves the first two into a `Resolved` value
([Resolution](#resolution)) and the `Palette` that views paint with.

## The palette

`style::Palette` holds the semantic colors of the current context. A view or a
component paints only with them.

| Method | Returns |
|---|---|
| `Palette::current()` | The palette in effect here: the one a `Backdrop` provides ([15](15-backdrops-and-media.md)), otherwise `Palette::system()`. In an inactive window `accent` and `toggle_accent` become `accent_inactive`, `selected_content` becomes `selected_content_inactive`, and `on_accent` and `on_selected_content` follow them. |
| `Palette::system()` | The palette of the resolved appearance, without backdrop and without the active-window rule. |

| Field | Use | AppKit |
|---|---|---|
| `label` | Primary text | `NSColor.labelColor` |
| `secondary_label` | Secondary text | `NSColor.secondaryLabelColor` |
| `tertiary_label` | Tertiary text and placeholders | `NSColor.tertiaryLabelColor` |
| `separator` | Separators: black at 10% in light, white at 12% in dark, as `NSColor.separatorColor` measures | `NSColor.separatorColor` |
| `outline` | Hairline edge of a field, box, well or panel | — |
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
| `scrim` | Gray veil behind sheets and over a window under an overlay sidebar | — |
| `card_border` | Edge of a card | — |
| `system_fill` | Track of a checkbox, radio, switch or slider that is off | `NSColor.systemFillColor` |
| `control_background_disabled` | Face of a disabled control | — |
| `text_background` | Background of a text, search or token field | `NSColor.textBackgroundColor` |
| `toggle_accent`, `toggle_accent_disabled` | Fill of a checkbox, radio, switch or slider that is on, and of a disabled switch | — |
| `ring_accent` | Arc of a determinate ring | — |
| `selected_content`, `selected_content_inactive` | Selected rows in an active and in an inactive window | `NSColor.selectedContentBackgroundColor`, `NSColor.unemphasizedSelectedContentBackgroundColor` |
| `on_selected_content` | Ink on a selected row | — |
| `separator_strong` | Grid lines of a table | — |
| `combo_button` | Face of a combo button | — |
| `segment_divider` | Divider between segments | — |

With `contrast = true` ([The appearance file](#the-appearance-file)) the palette takes
the `contrast` colors of its theme (`ContrastPalette`), measured from `NSColor` and
from AppKit controls with Increase Contrast on: ink, separator, control border and
fill, system fill, text background, accent, selected content, toggle accent and focus
ring (`docs/research/macos-control-metrics.md`). Outlines use the separator.

## Pixel grid

SwiftUI lays every view on whole device pixels. Torin keeps fractions, which
soften edges and move centered views by a pixel, so after each layout the kit
rounds the edges of every area to the nearest device pixel (a half rounds up).
A view thinner than a pixel keeps one. Glyphs inside a text and the paths of a
symbol keep Skia's own placement.

## Text, shapes and colors

| Function | What it does |
|---|---|
| `style::text(content, face, ink) -> Label` | A one-line Freya `label` with the family, size, weight and line height of a `Font` token and the color `ink`. For text that wraps, use `Text` ([2](02-first-app.md)). |
| `style::continuous_corner(radius) -> CornerRadius` | A corner radius with macOS's continuous curve, smoothed as `CORNER_SMOOTHING` says (0.745, what the window server draws for `cornerCurve = .continuous`; [Corners](#corners)). |
| `style::panel_edges(body, radius, fill) -> Rect` | A card filled with `fill` and framed like a panel window: the glass inner line in dark (`panel_inner_line`: brighter top and bottom edges, inward glow), the window inner line otherwise, and the outer line, each one physical pixel wide. Same colours and widths as the compositor's panel frames. |
| `theme::color(rgba) -> Color` | Converts a Tengen `Rgba` to Freya's `Color`. |

## Symbols

`Image::system(symbol)` draws a line icon, like SwiftUI's
`Image(systemName:)`: by default at `BODY` in SF points (an SF symbol at
16 pt fills the kit's 20 px grid), in the `label` color of the `Palette`.
`.size(f32)` and `.color(Rgba)` change them. `.font(Font)` is SwiftUI's
`.font(_:)` on a symbol: the font's size in SF points and its weight. A symbol
lays out in its own SF frame, not a square: each one's width and height per
point, measured with SwiftUI from 10 to 32 pt, so a stack of symbols and text
spaces as SwiftUI's within 1 px (`tray` at 13 pt is 19 × 13). The icons
are drawn on a grid of `SYMBOL.grid` (16) with a stroke of `SYMBOL.stroke`
(1), like SF Symbols at regular weight and medium scale. Inside a `SymbolTile`
they take `SYMBOL.fine_stroke` (also 1).
`Bolt`, `CircleHalf`, `Play`, `Pause`, `Forward`, `Backward`, `Battery`, `Dot`, `DiamondFill` and the four `Triangle*` are solid shapes without a stroke.

A symbol is fitted to the SF Symbol at 16 pt, regular weight, medium scale, drawn at 20 px: one grid unit is 1.25 px and one em is 12.8 units. SF's regular stroke at medium scale is 1.31 pt on average (1.19 to 1.40 by symbol), which is 1.05 on the grid, so `SYMBOL.stroke` is 1. A symbol wider than 1.25 em does not fit the grid. The inventory of the SF names, their ink and what the kit has is in [Icons macOS uses](../research/macos-icons.md).

```rust
Image::system(Symbol::Folder)
    .size(TITLE.size as f32)
    .color(Palette::current().accent)
```

`Image::font_weight`, `Image::image_scale` and `Image::symbol_rendering_mode`
are SwiftUI's `.fontWeight(_:)`, `.imageScale(_:)` and `.symbolRenderingMode(_:)`
for one symbol. `SymbolWeight`, `ImageScale` and `SymbolRenderingMode` are in the
prelude.

```rust
Image::system(Symbol::InfoCircle)
    .font_weight(SymbolWeight::Semibold)
    .image_scale(ImageScale::Large)
    .symbol_rendering_mode(SymbolRenderingMode::Hierarchical)
```

| Modifier | Values | What the kit draws |
|---|---|---|
| `font_weight` | `UltraLight`, `Thin`, `Light`, `Regular` (default), `Medium`, `Semibold`, `Bold`, `Heavy`, `Black` | The stroke is `SYMBOL.stroke` times `SYMBOL_WEIGHT_STROKE_<scale>[weight]`: 0.29, 0.45, 0.76, 1, 1.22, 1.38, 1.60, 1.91 and 2.20 at medium scale. |
| `image_scale` | `Small`, `Medium` (default), `Large` | The box is `size` times `SYMBOL_SCALE_SIZE`: 0.793, 1 and 1.275. The stroke grows less than the box (0.90 and 1.10 of medium at regular weight), so it is a separate table per scale. |
| `symbol_rendering_mode` | `Monochrome` (default), `Hierarchical` | Hierarchical draws a second layer of 31 symbols at `SYMBOL_HIERARCHY[1]` (0.5) or `[2]` (0.25) of the color; the mark stays at 1. The other symbols draw the same in both modes. |

The tables were measured on macOS 27.2.1 with SwiftUI's `ImageRenderer` ([Icons macOS uses](../research/macos-icons.md#1-method-and-reference-frame)). A weight and a scale apply to `Image::system` only, and to one image: SwiftUI's environment versions that reach a whole subtree do not exist, so a control that draws a symbol keeps regular weight and medium scale. Palette and multicolor rendering are not drawn.

`Symbol` names the icon. `Image` and `Symbol` are in the prelude:

| Group | Variants |
|---|---|
| Navigation | `ChevronRight`, `ChevronLeft`, `ChevronUp`, `ChevronDown`, `ChevronUpDown`, `ChevronDoubleRight`, `GoBackward`, `GoForward` |
| Actions | `MagnifyingGlass`, `XMark`, `Plus`, `Minus`, `Ellipsis`, `Checkmark`, `Expand`, `QuestionMark` |
| Media | `Play`, `Pause`, `Shuffle`, `ChevronLeftForwardSlashChevronRight`, `Backward`, `Forward`, `SpeakerWave`, `Captions`, `Film`, `Tv` |
| Layout | `ListBullet`, `SquareGrid`, `SlidersVertical`, `SidebarLeft` |
| Places and things | `House`, `Folder`, `Document`, `Star`, `StarFill`, `Clock`, `Bookmark`, `CircleHalf`, `Accessibility`, `Display`, `MenubarDockRectangle`, `Photo`, `Flower`, `Moon`, `Bell`, `Bed`, `Person`, `Briefcase`, `Printer`, `Phone`, `SquareOnSquare`, `Keyboard` |
| Connections and power | `Wifi`, `Bluetooth`, `Network`, `Vpn`, `Lock`, `Bolt`, `Battery` |
| Row actions | `EllipsisCircle` |
| Shapes | `Square`, `Circle`, `CapsuleOnRectangle`, `CapsuleOnRectangleFill`, `Dot`, `DiamondFill`, `TriangleLeft`, `TriangleRight`, `TriangleUp`, `TriangleDown` |
| Sidebar and places | `ArrowDownCircle`, `ICloud`, `InternalDrive`, `ExternalDrive`, `Trash`, `Tag`, `MusicNote`, `DesktopComputer`, `Eject`, `PersonCropCircle` |
| Share and edit | `SquareAndArrowUp`, `SquareAndArrowDown`, `DocOnDoc`, `Scissors`, `ArrowUturnBackward`, `ArrowUturnForward`, `ArrowClockwise`, `ArrowCounterclockwise` |
| Find and organize | `ArrowUpArrowDown`, `Line3HorizontalDecrease`, `Heart`, `Flag`, `Key`, `Calendar`, `Envelope`, `PaperPlane`, `Mic`, `Camera`, `GearShape` |
| Arrows | `ArrowLeft`, `ArrowRight`, `ArrowUp`, `ArrowDown`, `ArrowUpForward`, `ArrowDownToLine`, `ArrowRightCircle`, `ArrowLeftCircle`, `ArrowUpCircle`, `ChevronRightCircle`, `ChevronLeftCircle` |
| Layout and text | `MacWindow`, `SidebarRight`, `TextAlignLeft`, `TextAlignCenter`, `TextAlignRight`, `TextJustify` |
| States | `InfoCircle`, `PlusCircle`, `MinusCircle`, `XMarkCircle`, `CheckmarkCircle`, `ExclamationMarkCircle`, `QuestionMarkCircle`, `ExclamationMarkTriangle`, `NoSign` |
| States, filled | `XMarkCircleFill`, `PlusCircleFill`, `MinusCircleFill`, `CheckmarkCircleFill`, `ExclamationMarkCircleFill`, `InfoCircleFill`, `QuestionMarkCircleFill`, `ExclamationMarkTriangleFill` |
| Links, edit and files | `Link`, `Pencil`, `SquareAndPencil`, `DocText`, `Tray`, `Repeat`, `CheckmarkSquare` |
| Privacy | `LockOpen`, `LockFill` |
| Sound and transport | `SpeakerSlash`, `SpeakerWave1`, `StopFill`, `PlayCircle`, `PauseCircle` |
| Window menus | `XMarkRectangle`, `PlusRectangle`, `RectangleOnRectangle`, `RectangleDashed`, `MacWindowOnRectangle` |
| Filled | `HeartFill`, `CameraFill` |

The `*Fill` states, `PersonCropCircle`, `LockFill`, `StopFill` and `Pencil` are solid shapes (the mark is cut out of them). `Wifi`, `MusicNote`, `Scissors`, `HeartFill`, `CameraFill`, `CapsuleOnRectangleFill` and `Ellipsis` are filled and stroked. `StarFill` is a solid star. Every symbol is fitted this way except `Checkmark`, `Star`, `StarFill`, `QuestionMark` and `Dot`: they keep their shapes because the checkbox, the menu check, the rating indicator, the help button and the date picker are sized to them ([Icons macOS uses](../research/macos-icons.md#33-the-65-older-symbols)). The kit caches the SVG of each symbol.

Add a symbol in four steps:

1. Find the SF name in [Icons macOS uses](../research/macos-icons.md) and read its ink at 16 pt.
2. Draw the path on the 16 grid so that its ink box is the SF's, divided by 1.25 px per unit; draw at stroke 1.0.
3. Add the variant to `Symbol`. If SF dims a layer in hierarchical mode, add it to `Symbol::dimmed` (the sub-paths to dim and the level).
4. Add the row with its Apple name to the [glossary](20-glossary.md#symbol-variants).

`Image` draws from an `ImageStyle`: `size`, `stroke` and `foreground`. `.size`
and `.color` override them for one image and a theme for every image; a
`SymbolTile` draws at `SYMBOL.fine_stroke`. The fields
are in [Styles and themes](../../crates/bk/src/styles/image.rs).

## Images and template rendering

Symbols are SVG. A symbol the kit does not have is an SVG too, never a PNG:
an SVG stays sharp at every scale and takes the theme's color.

`Image::data(bytes)` draws SVG, PNG or JPEG bytes. `Image::rgba(width,
height, rgba)` draws straight-alpha pixels, such as a status item's `Pixmap`.
Both draw the image's own colors by default.

`.frame(width, height)` gives an image that keeps its own colors a rectangle that
is not a square, such as an illustration, like SwiftUI's `.frame(width:height:)`.
A template image and a plate are square: they use `.size`.

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

### Icon styles

`.icon_style(IconStyle)` restyles an app icon the way macOS does: `Tinted`
maps black to a dark shade and white to a light shade of the icon color
(`icon_color`, or the accent), and `Clear` turns the luminance into
transparency over a gray. `Default` and `Dark` draw the icon untouched; a dark
icon is the theme's dark twin, which the shell picks. A gray icon color gives
monochrome icons. One icon serves every style, and no file is generated per
style. A shell passes `theme::resolved().icon_style`, the `icon_style` key
below.

`IconStyle::dark(tone, scheme_dark)` says whether the icons draw their dark
look: `Default` never does; the others follow `icon_tone`, where `Automatic`
follows the color scheme. `Resolved::icon_dark` holds the answer and
`Resolved::icon_tint` the icon color.

```rust
Image::data(include_bytes!("app.svg").as_slice())
    .icon_style(theme::resolved().icon_style)
    .size(64.0)
```

### App icons

`.app_icon()` draws an app's own icon the way macOS draws one that does not fill
the squircle: on a plate, on the 1024 grid, in the user's icon style and in the
current color scheme. The icon keeps its art; the plate comes from its colors.

1. The icon is read pixel by pixel in OKLCH. Hues that carry chroma form lobes.
2. A bright icon (mean lightness `ICON_PLATE.bright_lightness` or more) takes
   the hue of its darkest lobe with at least `lobe_min_share` of the chroma; a
   dark icon takes the lightest lobe. A bright gray icon gets the silver plate
   (`ICON_PLATE_SILVER`).
3. `icon_style` and `icon_dark` pick the plate: the icon's hue by default,
   near black for a dark `Dark`, translucent white for `Clear`, the icon color
   for `Tinted`, and translucent black for a dark `Clear` or `Tinted`.
4. The plate gets its edge. A light runs along the top and the bottom edge,
   turns each corner and ends in a point a little way down the side
   (`light_reach`, `light_side`): white on default and dark plates, the
   plate's own color lifted on tinted ones. Clear and tinted plates also get a
   closed outline, whole on the sides and at `outline_ends` on the top and the
   bottom, with the light just inside it.
5. The icon is scaled to `logo_scale` of the plate's body, centered on what it
   draws, with a soft shadow, and recolored by `Clear` or `Tinted`.

```rust
Image::data(include_bytes!("firefox.svg").as_slice())
    .app_icon()
    .size(64.0)
```

An icon that already is a plate takes `.icon_style(...)` instead. The plate and the ends of the icon's color
ramp are the `ICON_PLATE*` and `ICON_LOGO_*` tokens. `app_icon` works on SVG and
on PNG and is cached with the icon style.

Dark icons suit `Clear` best: the plate has the low luminance that becomes
the transparency. The ends of the tint and the clear gray are the
`ICON_STYLE` tokens.

With `.glass_plate()` a compositor that draws a clear icon's plate as glass
(Malaysia's Dock, `set_plate_frames`) gets only the logo: `app_icon` skips the
plate, and `icon_style` drops the dark body of the icon, whose alpha starts at
`ICON_STYLE.clear_cut` (`IconStyle::matrix_on_glass`). The plate's corner is
`ICON_PLATE.corner`, as a share of its body. The compositor moves the
lightness of what is behind the plate toward white by `ICON_PLATE_GLASS`, or
toward black by `ICON_PLATE_GLASS_DARK` with dark icons. A dark Dock is too dark
to darken, so there dark icons take `ICON_PLATE_GLASS_DARK_ON_DARK`, a smaller
step toward white than light icons. The plate stands out from any glass and gets
the same edge. Without `Clear` it changes nothing. A style rasterizes at the window's scale and is cached
like a template image. It does not apply to `Template` images or to symbols.

An icon whose `size` changes every frame, such as the Dock's under the pointer,
takes `.raster_size(largest)`: it is rasterized once at that size, with the
window's scale, and drawn at `size`, so no frame parses the SVG or plates the
icon again.

```rust
Image::data(include_bytes!("firefox.svg").as_slice())
    .app_icon()
    .size(magnified)
    .raster_size(64.0)
```

## Fonts

`register_font(bytes) -> bool` adds a typeface from the bytes of a TrueType or
OpenType file. Its family is then available to text, menus and the
font panel of that thread. It returns `false` when the bytes are not a font.
Call it on the thread that runs the window, before opening it:

```rust
bk::register_font(include_bytes!("../fonts/Display.otf"));
```

The type tokens name Adwaita Sans and Geist Mono ([Type](#type)), but text
shows the system's own faces, as a native app's does: `.AppleSystemUIFont` and
`.AppleSystemUIFontMonospaced` (SF and SF Mono) on macOS, fontconfig's
`sans-serif` and `monospace` (the desktop's default faces) on Linux. A family
the app names itself, or registers with `register_font`, stays as it is.

CoreText picks a face's optical size (`opsz`) and its `trak` tracking from the
point size; Skia pins both. So on macOS the kit lays out every type token as
CoreText does: a copy of the face at `opsz` equal to the size, and the `trak`
value as letter spacing. A 22 pt bold title is then as wide as in AppKit.
Set sizes with `Text::font`, or `Text::font_size` for a size outside the text
styles: it takes the face at that size and SwiftUI's line height for
`.system(size:)` (`SYSTEM_SIZE_*`, measured 8-96 pt).
Linux keeps the face as it is.

## Theme functions

`bk::theme` connects the resolved appearance with Freya.

| Function | What it does |
|---|---|
| `resolved() -> Resolved` | The resolved appearance of this point of the tree: the one `use_appearance` provides, otherwise the window's, otherwise it reads the files. |
| `use_appearance(adjust)` | Hook. Changes the appearance for this component and its children: `adjust` edits a copy of the user's `AppearanceConfig`. It follows later changes of the user's appearance. |
| `surface_background(resolved, kind) -> Rgba` | Background of a surface: the window color for `SurfaceKind::Toplevel`; the bar color for `SurfaceKind::LayerShell`, tinted when the tint covers the desktop. |
| `menu_palette(resolved, kind) -> MenuPalette` | Menu colors over that surface. A tinted surface lifts the menu background and recomputes its separator. |
| `popover_palette(resolved, kind) -> MenuPalette` | `menu_palette` with the metrics of a pop-up menu (`MENU_POPOVER`). |
| `apply_popover_metrics(palette) -> MenuPalette` | Replaces the padding, widths, item size and check mark of a `MenuPalette` with `MENU_POPOVER`. |
| `over_glass(palette) -> MenuPalette` | The palette of a menu or popover the compositor draws its glass under: a transparent `bg`, translucent inks (`glass_fg`, `glass_shortcut_color`, `glass_disabled_fg`) so the glass shows through the text, and `search_bg` the faint ink `glass_disabled_fg` as a translucent fill, so the search field reads as a field on the glass. The inks follow the glass the compositor reports under the popup (`limitless_focus_surface_v1.glass`): dark on a light panel and white on a dark one, whatever the colour scheme; until it reports, they are the scheme's. With Increase Contrast it returns the palette as it is; Reduce Transparency alone leaves the glass, as macOS does. |

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
([Styles and themes](../../.claude/skills/barrierkit-app/styles.md)).

| I want… | Like this | Where |
|---|---|---|
| A system color that follows light and dark (SwiftUI's `.red`) | `.fill(ShapeStyle::System(SystemColor::Red))`, `.foreground_style(ShapeStyle::System(SystemColor::Blue))` | [Styles and themes](../../.claude/skills/barrierkit-app/styles.md#value-types) |
| A control in another color | `.tint(Tint::Color(ACCENT_SWATCHES[4].1))` on `Switch`, `Checkbox`, `Slider`, `LevelSlider`, `RadioGroup`, `ProgressView` | [5](05-controls.md) |
| A control filled with the backdrop gradient | `.tint(Tint::Backdrop)` inside a `Backdrop` | [15](15-backdrops-and-media.md) |
| Another accent for a whole area | `Backdrop::new(..).accent(Accent::Own(color))` or `Accent::FromBackdrop`; it holds only while the user's accent is automatic | [15](15-backdrops-and-media.md) |
| Let the user pick a color | `ColorWell::new("Color", &ACCENT_SWATCHES, chosen)`, with `chosen: State<Option<usize>>` | [5](05-controls.md) |
| Light or dark in one area | `theme::use_appearance(..)` | [Theme functions](#theme-functions) |
| Any color, shape, size or motion of a control that takes a style (sliders, buttons, menu buttons, switch, checkbox, choices, stepper, toggle group, segmented control, tool group, text fields) | `.style(\|s\| ..)`, `.control_style(\|s: SliderStyle\| ..)` or `Theme` | [Styles and themes](../../.claude/skills/barrierkit-app/styles.md#which-controls-take-a-style) |
| The text color, background or shadow of a view | `.foreground_style(..)`, `.background_style(..)`, `.shadows(..)` | [Styles and themes](../../.claude/skills/barrierkit-app/styles.md#paint-on-views) |

In an inactive window every tint uses the inactive accent.

## User appearance: `ctappearance`

`ctappearance` reads the user's appearance, resolves it into colors and
watches it. The compositor, the shell, the portal and the kit share it, so all
of them agree. On Linux the kit's runner watches the files: when the appearance or the
wallpaper changes, every window repaints without a restart. On macOS the
system's appearance drives the repaint ([18](18-platform.md#the-macos-runner)).

### The system's light or dark

The desktop's light or dark wins over `color_scheme` (ADR 0114).
`load_appearance()` applies the scheme the runner recorded with
`set_system_scheme`, so every reader of the appearance, `use_appearance`
included, sees it. An app does not choose its own scheme; a part of a window
may still force one with `use_appearance`.

| System | Source | Live |
|---|---|---|
| macOS | `NSApp.effectiveAppearance`: Aqua or Dark Aqua | Key-value observing of `effectiveAppearance` |
| Linux | `org.freedesktop.appearance` `color-scheme` of the Settings portal: `1` dark, `2` light, `0` no preference | The portal's `SettingChanged` signal |

- A `dark_tint` file stays `dark_tint` when the system asks for dark.
- The system's scheme turns `automatic_scheme` off: the system already decides by the hour.
- With no preference, or no portal, `color_scheme` holds.
- The `shell` feature skips the portal: Malaysia's shell is the desktop, and its
  portal backend serves `color_scheme` from this file.
- The system's Increase Contrast and Reduce Motion, and Reduce Transparency on
  macOS, add to the file's keys ([18](18-platform.md#who-provides-what-on-each-platform)).

### The appearance file

`appearance_path()` is `config_dir()/appearance.domain`. `config_dir()` is
`$MALAYSIA_CONFIG_DIR`, or `$XDG_CONFIG_HOME/malaysia`
(`~/.config/malaysia`). A missing key takes its default; an invalid value takes
its default and logs a warning.

```toml
# ~/.config/malaysia/appearance.domain
color_scheme = "dark"
accent = "purple"
window_color = "accent"
tint_parts = ["sidebars", "window", "groups", "controls", "menus"]
```

| Key | Type (`AppearanceConfig` field) | Values | Default |
|---|---|---|---|
| `color_scheme` | `ColorScheme` | `light`, `dark`, `dark_tint`; [the system's](#the-systems-light-or-dark) light or dark wins | `light` |
| `automatic_scheme` | `bool` | Light by day and dark from `automatic_dark_from` until `automatic_dark_until`, local time; a `dark_tint` scheme stays dark-tint at night. Reading the domain puts the scheme of the hour in `color_scheme` | `false` |
| `automatic_dark_from`, `automatic_dark_until` | `u8` | Hours from 0 to 23 | `19`, `7` |
| `accent` | `Option<String>` | A name of `ACCENTS`: `blue`, `purple`, `pink`, `red`, `orange`, `yellow`, `green` or `graphite`; an unknown name is ignored. Absent means Multicolor: the controls are blue and apps keep their own accent | absent |
| `accent_color` | `Option<[f32; 3]>` | RGB from 0 to 1, a color macOS has no name for. One that equals the light `control` of a named accent, as the settings wrote before accents had names, reads as that accent | absent |
| `wallpaper_accent` | `bool` | Derives the accent from the wallpaper's dominant color; a monochrome wallpaper has none and falls back to graphite | `false` |
| `contrast` | `bool` | | `false` |
| `reduced_motion` | `bool` | | `false` |
| `reduce_transparency` | `bool` | | `false` |
| `scroll_bars` | `ScrollBars` | `automatic`, `when_scrolling`, `always` | `automatic` |
| `scroll_bar_click` | `ScrollBarClick` | `next_page`, `clicked_spot` | `next_page` |
| `window_color` | `WindowColorMode` | `neutral`, `accent` (an old `wallpaper` reads as `neutral`) | `neutral` |
| `tint_strength` | `TintStrength` | `subtle`, `normal`, `strong`, `intense` | `normal` |
| `tint_parts` | `TintParts` | list of `sidebars`, `window`, `groups`, `controls`, `menus`, `bars` (the menu bar and the Dock) | all but `bars` |
| `traffic_lights` | `TrafficLightStyle` | `standard`, `accent`, `wallpaper` | `wallpaper` |
| `wallpaper_tinting` | `bool` | | `true` |
| `wallpaper_tinting_in_light` | `bool` | | `false` |
| `show_borders` | `bool` | | `false` |
| `differentiate_without_color` | `bool` | | `false` |
| `text_size` | `f32` | a value of `TEXT_SIZES` | `1.0` |
| `text_sizes` | `TextSizes` | table with optional `barrierkit`, `gtk`, `qt`, each a value of `TEXT_SIZES` | empty: every toolkit uses `text_size` |
| `icon_style` | `IconStyle` | `default`, `dark`, `clear`, `tinted`; the retired `monochrome` reads as `tinted` with a graphite `icon_color` | `default` |
| `icon_tone` | `IconTone` | `light`, `dark`, `automatic` (follows the color scheme); `default` ignores it | `automatic` |
| `icon_color` | `Option<[f32; 3]>` | RGB from 0 to 1 for tinted icons and folders; absent means the accent | absent |

Notes on the values:

- The old value `"auto"` reads as `light`; `is_dark()` is true for `dark` and
  `dark_tint`.
- A number reads as the nearest value of its token list.
- A `sidebar_style` key from an older file is ignored: the sidebar has one look ([Glass](10-sidebar.md#glass)).
- `TintStrength::factor()` multiplies the accent tint and the wallpaper tint by
  `TINT_STRENGTH`: 0.5, 1, 1.5 or 2.
- `scroll_bars = "automatic"` is macOS «Automatically based on mouse or
  trackpad». On macOS it takes `NSScroller.preferredScrollerStyle`: legacy
  scrollers read as `always`, overlay ones as `when_scrolling`, and a change
  of mouse applies at once. On Linux the bar stays after a mouse wheel and
  hides after a touchpad scroll.
- `wallpaper` derives the lights from the wallpaper's dominant color, or from
  the accent while there is none. The old value `"macos"` still reads as
  `standard`, with a warning.
- `traffic_light_shape` is retired: the lights are discs, as in macOS 27. A file
  that still has it loads, and the next write removes it.
- `glass_blur` is retired: macOS has no blur strength to set, so every blur is
  the measured one of its surface. A file that still has it loads, and the next
  write removes it.
- The accessibility keys follow macOS System Settings › Accessibility ›
  Display. The kit applies three of them in its control styles:
  `show_borders` draws a `ControlBorder` line around icon buttons and tool
  groups; `differentiate_without_color` keeps the traffic-light
  symbols on while the window is focused; `text_size` scales `Text`, sidebar
  items (and their height) with `Font::scaled`; `Button` scales its text and
  symbol the same way when it draws them, and keeps its height. The other
  accessibility keys (`invert_colors`, `display_contrast`, the color filter, the
  pointer keys and `menu_bar_size`) belong to the desktop, not to the kit.

### Reading and writing

| Function | What it does |
|---|---|
| `config_dir()`, `data_dir()` | The desktop's shared folders: `$MALAYSIA_CONFIG_DIR` or `$XDG_CONFIG_HOME/malaysia`; `$MALAYSIA_DATA_DIR` or `$XDG_DATA_HOME/malaysia`. They hold `appearance.domain`, `window.domain`, `language.domain` and the wallpaper state; an app keeps its own settings in its own folder ([13](13-app.md#settings-and-restoration)). |
| `appearance_path()` | `config_dir()/appearance.domain`. |
| `wallpaper_state_path()` | `data_dir()/wallpaper.state`, where the wallpaper's dominant color and grid live. |
| `read_appearance()` | `Ok(None)` without a file, `Err` when the TOML does not parse. |
| `load_appearance()` | The file's config, or the default on any error. |
| `write_appearance(path, config)` | Writes the config and keeps other keys and comments. A file that does not parse as an appearance config is first copied to `.domain.bak`. The file is replaced atomically. |
| `replace_file(path, text)` | Writes `text` to a temporary file beside `path`, syncs it and renames it over `path`. Creates the directory with `create_private_dir`. |
| `create_private_dir(dir)` | Creates `dir` and its missing parents with mode `0700`, as the XDG Base Directory specification asks. |
| `load_dominant()` | Reads `dominant` from `wallpaper.state`. |
| `write_wallpaper_state(path, &WallpaperState)` | Writes `dominant` and `grid`; an empty state removes the file. A file that does not parse is first copied to `.state.bak`; one that cannot be read is left alone and the call fails. |
| `load_wallpaper_state()` | The whole `wallpaper.state`, or the default. `WallpaperState::grid_for(desktop)` picks a desktop's grid. |

The wallpaper each desktop shows and the user's collections are not part
of the kit.

### Resolution

`resolve(&config, dominant, native_lights) -> Resolved` turns the config and
the wallpaper's dominant color into colors. `native_lights` is true where the
system draws its own traffic lights: each backend passes its `NATIVE_LIGHTS`,
true on macOS, where only the graphite accent changes them. `dominant` comes from the backend's `dominant()`: `load_dominant` on Linux, `None` on macOS, where WindowServer tints the windows. `palette(scheme)` returns the base
`ctdesign::Palette` of a scheme (`LIGHT`, `DARK` or `DARK_TINT`).
`palette_in(scheme, contrast)` adds the scheme's `ContrastPalette` with
Increase Contrast; `Resolved::palette()` is that of a resolved appearance.
`traffic::looks(style, standard, accent, dominant)` returns the three traffic
lights of a style ([Traffic-light geometry](#traffic-light-geometry)).

| `Resolved` field | What it holds |
|---|---|
| `scheme` | The `ColorScheme`. |
| `accent_colors` | The `AccentColors` of the accent in this scheme and contrast: every color it decides ([Accents](#accents)). The fields below are copies of its members. |
| `control_accent` | `NSColor.controlAccentColor`: the fill of controls (`ShapeStyle::Accent`). |
| `toggle_accent`, `ring_accent`, `toggle_accent_disabled` | The fill of a checkbox, radio, switch and slider that is on, of a determinate ring, and of a disabled switch. |
| `accent`, `on_accent` | The accent for selected rows and its ink: `NSColor.selectedContentBackgroundColor` (`selected_content`). With a color macOS has no name for, the ink stays light and the fill is darkened until it reaches 4.5:1 (rule 3). |
| `automatic_accent` | True without `accent`, `accent_color` or a wallpaper accent: an `Accent::Own` or `Accent::FromBackdrop` keeps its color. |
| `focus_ring` | `NSColor.keyboardFocusIndicatorColor`: translucent, opaque with `contrast`. A color macOS has no name for moves until it reaches 3:1 on the window with `contrast` (rule 7). |
| `link` | The accent moved until it reaches 4.5:1 on the window and on content. |
| `window_tint` | Tint color and amount, or `None`: the accent, or the wallpaper after `behind_window`. |
| `wallpaper_tinting` | `backdrop::WallpaperTinting` of the wallpaper behind a window (`chroma` and `lift`), or `None` when it is off. `Resolved::behind_window(color)` tints the `tint_parts` with that color (rule 4). |
| `tint_parts` | Copy of the config. |
| `traffic_lights` | The `TrafficLooks` of the user's `traffic_lights` style, in this scheme and contrast. On macOS the style is `standard`, or `accent` with the graphite accent. |
| `contrast`, `reduced_motion`, `scroll_bars`, `scroll_bar_click` | Copies of the config. |
| `reduce_transparency` | `AppearanceConfig::transparency_reduced()`: true with `reduce_transparency` or `contrast`. |
| `show_borders`, `differentiate_without_color`, `icon_style` | Copies of the config. |
| `icon_dark`, `icon_tint` | `IconStyle::dark` for `icon_tone` and the scheme, and `icon_color` or the accent. |
| `text_size` | `AppearanceConfig::text_size_for(Toolkit::BarrierKit)`: the BarrierKit size, or the preferred `text_size`. |
| `menu`, `menu_parent` | `MenuPalette` and the window color a menu sits on. `Resolved::menu_separator(panel_bg)` gives the separator over another background. |
| `sidebar` | `SidebarSurface`: `bg`, its `SidebarPalette` and `glass`. `glass` is a `SidebarGlass` with `base` (the tone of the glass at rest, tinted with the accent when the accent is the window color), `solid` (the tone of a window that is not active) and `chroma` (the share of the wallpaper's chroma, `GLASS.accent_share` of it under an accent) and is `None` with reduced transparency ([Glass](10-sidebar.md#glass)). |
| `toolgroup` | `ToolgroupSurface`: `bg` and its `ToolgroupPalette`. |
| `surfaces` | `Surfaces`: `window`, `titlebar_inactive`, `toolbar`, `toolbar_line`, `sunken`, `surface`, `raised`, `control`, `hover`, `line`, already tinted except `toolbar_line` and `control`. `line` is the separator color. |

The menu bar, Dock, menu-glass and notification fields (`bar_tint`, `shell`,
`dock`, `menu_glass`, `notification_glass`) are not part of the kit.

The rules `resolve` applies:

1. `reduce_transparency` or `contrast` picks the `_OPAQUE` variants of the
   shell, menu, sidebar and toolgroup colors. In macOS the two are separate
   switches and Increase contrast alone keeps the blur of every material
   ([macOS materials](../research/macos-materials.md)); how the shell surfaces
   change with it was not measured, so the kit keeps them solid.
2. A tint takes the chroma of the tint color into each surface, in OKLab, so
   a surface keeps its lightness (`backdrop::take_chroma`, as the wallpaper
   tint does). The amount is `WINDOW_TINT_ACCENT` (light, dark), times the
   strength factor, times the `LAYER_FACTOR_*` of that layer, times
   `ACCENT_CHROMA_GAIN` for an accent. A part left out of `tint_parts` stays
   neutral.
3. `on_accent` is always the light ink, as in macOS. A named accent selects
   with its measured `selected_content`, even where the ink does not reach
   4.5:1, as macOS does. A color macOS has no name for, where the ink does not
   reach 4.5:1 (`ON_ACCENT_MIN_CONTRAST`), is darkened just enough
   (WCAG 2.2 §1.4.3). Without Increase Contrast, controls keep `control_accent` as chosen.
4. Wallpaper tinting is on with `wallpaper_tinting` and a neutral
   `window_color`, with any accent (malaysia ADR 0065), unless
   `reduce_transparency` or `contrast` is on. It
   applies in dark schemes, and in light ones with
   `wallpaper_tinting_in_light`. Its `chroma` is `WALLPAPER_TINTING` (light,
   dark) and its `lift` is `WALLPAPER_TINTING_LIFT`, both times
   `tint_strength`. As in macOS 26, `WallpaperTinting::take` tints only the
   window background: it takes `chroma` of the wallpaper's OKLab chroma
   (`backdrop::take_chroma`) and moves its lightness `lift` of the way to
   `WALLPAPER_TINTING_TARGET` (`backdrop::lift`). The layers over a macOS
   window are translucent, so every opaque layer of `tint_parts` takes the
   window's OKLab shift as it is, without `LAYER_FACTOR_*`; a translucent one,
   like `line`, already shows it and stays. The sidebar takes its own,
   `WallpaperTinting::sidebar`: `WALLPAPER_TINTING_SIDEBAR` times the chroma
   and `WALLPAPER_TINTING_SIDEBAR_LIFT` times the lift. `sidebar.glass.base`
   and `sidebar.glass.solid` take it too, saturating towards the glass cap,
   when `tint_parts` has `sidebars`.
5. The accent is, in this order: the named `accent`; `accent_color`; with
   `wallpaper_accent`, the wallpaper's dominant color, graphite while it has
   none; else Multicolor, which is blue for the controls.
   `automatic_accent` lets an area keep its own only for Multicolor.
6. Menu hover takes `control_accent`, as macOS does, and `accent` with
   Increase Contrast or a color macOS has no name for. Sidebar selection takes
   the accent's `sidebar_selection` (`accent` for a color macOS has no name
   for); the shell's active tile takes `accent`.
7. The focus ring is the measured `focus_ring` of the accent: translucent
   (49.8%) in normal contrast, opaque with `contrast`. A color macOS has no
   name for takes the rule's ring at that alpha; a translucent ring can fall
   under the 3:1 non-text contrast of WCAG 2.2 §1.4.11 on a light or busy
   surface, so with `contrast` that ring moves until it reaches
   `ACCENT_MIN_CONTRAST_ON_WINDOW` on the window. The accent tint of the
   window does not change a translucent ring; it does move an opaque one.

### Accents

The accent picker of System Settings has nine choices: Multicolor and eight
named colors. `ctdesign::ACCENTS` lists the eight, in the order of the picker,
each with the colors it decides in four looks: `light`, `dark`,
`light_contrast` and `dark_contrast`. Every value is measured on macOS 27.2.1
with the accent set per process (`-AppleAccentColor`), never in the system
([measurements](../research/macos-control-metrics.md#accent-and-selection)).
Multicolor is no entry: with no `accent` the controls are blue and
`automatic_accent` is true, so an app keeps its own accent.
macOS has its own value for every accent in every look, so one color for light and dark is wrong.

`Accent::colors(dark, contrast)` returns the `AccentColors` of one look:

| Field | macOS | The kit draws with it |
|---|---|---|
| `control` | `controlAccentColor` | Controls (`Resolved::control_accent`), menu highlight, the ring of the chosen swatch |
| `selected_content` | `selectedContentBackgroundColor` | Selected rows of lists and tables (`Resolved::accent`) |
| `selected_text` | `selectedTextBackgroundColor` | Selected text |
| `focus_ring` | `keyboardFocusIndicatorColor` | The keyboard focus ring: translucent, opaque with Increase Contrast |
| `toggle`, `toggle_disabled` | the fill of a checkbox, radio, switch and slider that is on, and of a disabled switch | `Resolved::toggle_accent`, `toggle_accent_disabled` |
| `ring` | the arc of a determinate ring | `Resolved::ring_accent` |
| `pressed`, `pressed_toggle` | a pressed default button; a pressed checkbox or radio that is on | `PressedPalette` |
| `segment_disabled` | the chosen segment of a disabled segmented control | `DisabledPalette` |
| `sidebar_selection` | the selected item of a sidebar | `Resolved::sidebar.palette.selected_bg` |

Blue's entries point to the `accent` tokens of the themes, except
`selected_text`. The other seven are measured one by one.

Three things hold in macOS for every accent. Increase Contrast makes
`selected_content` equal to `control`. It darkens `control` until white text on
it reaches about 4.6:1. `selected_text` is the `control` over the window at 30%:
over white in light and over `#5a5a5a` in dark (`ACCENT_RULE.selected_text_*`).

A color macOS has no name for, from `accent_color` or the wallpaper, has only
its `control`. `ctappearance::accent::derived` gives the rest by the rule fitted
to the named accents: `selected_text` is exact, `selected_content` and
`focus_ring` are an OKLab shift (`OklchShift`: lightness scale and offset,
chroma scale, hue shift) that stays within 6 levels of every named accent
(`ACCENT_RULE`), and the fills of a toggle, a pressed control and a disabled
one mix the `control` toward black in light and toward white in dark by a
measured amount per look (`ACCENT_RULE.*_mix`). The ring and the sidebar selection are the `control`. With Increase
Contrast the `control` is first darkened until white text reaches 4.5:1
(`ON_ACCENT_MIN_CONTRAST`), as rule 3 above asks. An `accent_color`
that equals the light `control` of a named accent is that accent, so a
configuration written before accents had names keeps its look and gains the
dark and Increase Contrast values.

`ColorWell::accents(label, chosen)` is the picker. Its swatches are 24 wide, 12 apart,
in the `control` of the current appearance; the chosen one has a ring 1 away and
3 wide in the accent, and its name below in the secondary label color, as in
System Settings. `MULTICOLOR` are the 25 stops of the Multicolor swatch, 15° apart from the
top clockwise.

### Glass over a backdrop

The compositor draws the glass of the menus, the Dock and the notifications;
the colour it shows over a backdrop is the compositor's.

`menu::panel_luma(panel)` is the luma of the panel and
`menu::dark_ink_over(luma)` says whether its ink is dark: above
`MENU_DARK_INK_ABOVE` (0.4) it is, below it the ink is light. A light panel
over black has luma 0.48 and takes dark ink; a dark one has 0.17 and takes
light ink.

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
   `limitless_focus_surface_v1` (version 3).
4. The kit keeps one `Resolved` per window: the global one passed through
   `behind_window(color)`. It recomputes it when the appearance changes and,
   when the event brings a new color, fades to it over `DURATION_BASE` with a
   smoothstep, one step per frame callback. Reduce motion swaps it at once.

### Watching

On Linux, `watch::AppearanceWatch::new()` watches `config_dir()` and `data_dir()` with
inotify. It is a file descriptor (`AsFd`) for an event loop; `drain()` returns
what changed as `AppearanceEvent`s:

| Event | File |
|---|---|
| `Appearance` | `appearance.domain` |
| `Wallpaper` | `wallpaper.state` |
| `WallpaperSelection` | `wallpaper-selection.state` in the data directory |
| `Defaults(name)` | any other `.domain` in the config folder or in a folder added with `watch_app_folder(path)`; `name` is its file name, `None` after a lost event |

If a watched folder is deleted or moved, `drain()` creates it again and watches it again.
The kit's runner adds the settings folder of each app ([13](13-app.md#settings-and-restoration)).

`AppearanceEvent::restyles()` is true for `Appearance` and `Wallpaper`. The kit's runner already
uses it; an app needs it only with its own event loop.

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

### Traffic-light geometry

`ctappearance::traffic` holds the geometry and look of the close, minimize and
zoom lights, shared by the kit and the compositor:

| Item | What it does |
|---|---|
| `inset(bar_height)` | Distance of the lights from the left edge of a bar: `TITLEBAR.buttons.inset` (9) in the 32 px bar and `inset_compact` (12) in the `TITLEBAR_CLIENT_HEIGHT` (40) bar, as measured on macOS. The lights stay centred vertically: 9 and 13 from the top. |
| `button_rect(index) -> (x, y, size)` | Rectangle of light `index` in a server-side titlebar. |
| `hit(point) -> Option<ButtonKind>` | The light under a point. |
| `TrafficInput` | `focused`, `group_hovered`, `pressed`, `zoom_supported`, `fullscreen` (minimize is disabled). |
| `visual(looks, input) -> [ButtonVisual; 3]` | The fill and ring stops, the glyph color, the scale and the dodge of each light for that input. |
| `ButtonVisual` | `fill` (4 stops, top to bottom), `ring` (3 stops), `glyph` (`None` without hover), `scale` and `dodge` (1.0 and 0.0 unless pressed). |
| `looks(style, standard, accent, dominant) -> TrafficLooks` | The `standard` look as it is, or recolored with the tone that `accent` or `wallpaper` gives. |
| `Glyph` | `Close`, `Minimize`, `ZoomEnter`, `ZoomExit` (full screen) and `Plus`; `Glyph::of(index, fixed_width, fullscreen)` picks the one of a light. |
| `rounded(triangle) -> [Corner; 3]` | The corners of a zoom triangle rounded on `TRAFFIC_GLYPHS.zoom_corner`. |
| `TrafficPointer` | Pointer state machine: `on_motion`, `on_group_enter`, `on_button_enter`, `on_leave_button`, `on_press`, `on_release` (returns the clicked light), `on_leave`, `hovered`, `pressed_visual`. |

```rust
// ✗ geometry computed again
let x = 8.0 + i as f64 * (12.0 + 8.0);
// ✓ the shared function
let (x, y, d) = ctappearance::traffic::button_rect(i);
```

### Corners

`ctappearance::corner` is the CPU reference of a rounded corner, shared by the kit and the compositor, with the measurements in [macOS corners](../research/macos-corners.md). A corner has a nominal `radius`, a `smoothing` (0 is the circle, `CORNER_SMOOTHING` the continuous curve of `cornerCurve = .continuous`) and the straight edges at `edge_x`, `edge_y` from the pixel grid:

| Item | What it does |
|---|---|
| `edge(radius, smoothing, y) -> f32` | Where the top-left corner starts at depth `y`: the arc and the two Bezier tails of Freya's smoothed corner. |
| `coverage(radius, smoothing, x, y, edge_x, edge_y) -> f32` | Share of the pixel at (`x`, `y`) inside the shape, 16 sub-rows. |
| `tile(radius, smoothing, size, edge_x, edge_y) -> Vec<f32>` | The coverage of a `size` x `size` corner, row by row: the mask to compare with a capture. |
| `concentric(container, inset_x, inset_y, minimum) -> f32` | The radius of a nested corner (`NSViewCornerRadius.containerConcentric`, SwiftUI `ConcentricRectangle`): the container's radius minus the smaller inset, `minimum` outside the container's corner region (`CORNER_CONCENTRIC_REGION`). |

Draw a window with `CORNER_WINDOW.fitted` (16.8), not its `radius` (16, the API's): the capture of a window is a continuous corner of the larger radius. The other elements have a `CornerShape` each (section [Windows and titlebars](#windows-and-titlebars)). The parity check, with its tolerance, is in [macOS corners](../research/macos-corners.md#9-parity).

```rust
// ✗ the API radius for the silhouette
let radius = WINDOW_CORNER.titlebar as f32;
// ✓ the measured silhouette
let radius = ctdesign::CORNER_WINDOW.fitted;
```

The kit draws every `CornerStyle::Continuous` shape through one function, `style::continuous`, with `CORNER_SMOOTHING`; no shape holds its own number. That smoothing, 0.745, is what the window server draws, not the 0.61 of a CPU render. A control is continuous where its layer in macOS is (push, pop-up, segmented, text field, combo box, checkbox, group box, tab view, sidebar item, rows, cards, sheet) and circular where it is not or nobody measured it. A text field and a combo box take their radius from `ControlSize` (4, 5, 6, 7, 9) and the combo button 2.5, 3.5, 4.5, 5.5, 6.5. A `Sheet` is `CORNER_SHEET.fitted` and an `Alert` `CORNER_ALERT.fitted`. A shape nested in a rounded one takes `style::concentric(outer, inset)` unless its radius was measured. The table of every radius, the Skia tolerance and the parity tests are in [macOS corners](../research/macos-corners.md#12-the-kit).

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
| `requested_languages(app_language, preferred)` | The app's language, else the system's, else what `preferred` returns. The kit passes its backend's list ([17](17-languages.md#how-the-language-is-chosen)). |
| `locale_languages()` | `LANGUAGE` followed by the first of `LC_ALL`, `LC_MESSAGES`, `LANG`, as tags such as `es-MX`. The `C` locale requests nothing. |

### Directories

`ctappearance::xdg` follows the XDG Base Directory specification and ignores
relative paths:

| Function | Returns |
|---|---|
| `config_home()` | `$XDG_CONFIG_HOME` or `~/.config` |
| `data_home()` | `$XDG_DATA_HOME` or `~/.local/share` |
| `cache_home()` | `$XDG_CACHE_HOME` or `~/.cache` |
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
   in `tokens.json`. A color per theme goes under `themes.light` and
   `themes.dark`; `themes.dark-tint` holds only the colors that differ from
   dark.
2. Add its field or constant and regenerate `docs/barrierkit/tengen.css`, as
   [Tengen](tengen.md#steps-to-add-a-constant) says.
3. Use it: `SPLIT_VIEW.key_step`.
4. Add its row to the [glossary](20-glossary.md).
5. Run `./dev.sh check`.

### Token types

| Type | What it is |
|---|---|
| `Rgba` | sRGB color with straight alpha, channels from 0 to 1. `TRANSPARENT`, `rgb8`, `rgba8`, `to_rgba8`, `premultiplied`, `mix` (keeps its own alpha), `blend` (mixes with alpha, premultiplied), `darken`, `lighten`, `to_linear`, `from_linear`, `luma` (Rec. 709 of the stored channels, `Rgba::LUMA`), `relative_luminance`, `contrast_ratio`, `to_hsl`, `from_hsl`. |
| `Font` | `family`, `size`, `line_height` (pixels) and `weight`. |
| `Palette` | Base colors of a theme: `titlebar_bg`, `hairline`, `inner_line`, `panel_inner_line`, `outer_border`, `title`, `row_title`, `text`, `ink_strong`, `ink_muted`, `ink_faint`, `line`, `line_strong`, `separator`, `accent`, `selected_content`, `accent_toggle`, `control`, `control_disabled`, `accent_inactive`, `on_accent`, `danger`, `scrim`, `selected_control`, `card_border`, `system_fill`, `text_background`, `separator_strong`, `selected_content_inactive`, `combo_button`, `segment_divider`, `accent_ring`, `accent_toggle_disabled`. |
| `ContrastPalette` | The Increase Contrast colors of a theme: `ink`, `ink_muted`, `ink_faint`, `separator`, `line_strong`, `bg_control`, `bg_control_disabled`, `combo_button`, `system_fill`, `text_background`, `accent`, `selected_content`, `accent_toggle`, `accent_toggle_disabled`, `accent_ring`, `separator_strong` and `focus_ring`; `Palette::contrasted` takes all but `focus_ring`; `CONTRAST_LIGHT`, `CONTRAST_DARK`, `CONTRAST_DARK_TINT`. |
| `DisabledPalette` | The fixed fills of disabled controls, measured from AppKit: `off`, `on`, `slider_fill`, `switch_off`, `stepper_chevron`, `segment` and `segment_label`; `DISABLED_LIGHT`, `DISABLED_DARK`, `DISABLED_DARK_TINT` and the `DISABLED_CONTRAST_*` set under Increase Contrast. |
| `PressedPalette` | The fills of pressed controls: `bezel`, `accent`, `toggle_accent` and `off`; `PRESSED_*` and `PRESSED_CONTRAST_*`. |
| `BezelPalette` | The square, textured and inline bezels of `ButtonStyle` and their pressed fills; `BEZEL_*` and `BEZEL_CONTRAST_*`. |
| `GaugeContrast` | `ring_track`, `level_track` and `level_border` of the gauges under Increase Contrast; `GAUGE_CONTRAST_*`. |
| `RelevancyPalette` | The `fill` behind the lines of a relevancy indicator and its `line`, measured from AppKit; `RELEVANCY_*` and `RELEVANCY_CONTRAST_*`. Its track is `RELEVANCY_TRACK_*`. |
| `CalendarPalette` | The card of a graphical date picker: `background`, `border`, `border_width`, `edge` (the line along its bottom) and `separator` (under the weekdays); `CALENDAR_*` and `CALENDAR_CONTRAST_*`. |
| `FocusPair` | A color for the focused window and one for the unfocused. |
| `PanelLine` | Inner line of a focused panel, menu or dialog: `sides`, `edges` (top and bottom) and the `glow` under the edges. |
| `TrafficLooks` | The traffic lights of a scheme: `close`, `minimize`, `zoom`, `inactive` (`TrafficLook`: `base`, `fill`, `ring`, `glyph`), the `halo` (`TrafficHalo`: `color`, `blur`, `dy`) and the `backdrop` that a pressed light brightens against. `TRAFFIC_LIGHTS_*` and `TRAFFIC_LIGHTS_CONTRAST_*`. |
| `ShellPalette` | Menu bar and Dock tiles: `bar`, `text`, `tile`, `tile_active`, `tile_text_active`. |
| `MenuPalette` | Colors and metrics of a menu: background, padding, item size, inks, inks over glass, hover, separator, check mark, header and search field. |
| `SidebarPalette` | Sidebar inks: item, icon, selection, inactive selection, section, count, line. |
| `ToolgroupPalette` | Toolbar group: `separator`, `rim`. |
| `ShadowLayer` | `blur`, `y_offset`, `opacity`, `color`. |
| `MaterialThickness` | `opacity` and `blur` of one material thickness. |
| `ShadowStyle` | `ambient` and `contact` layers and their scale when unfocused; `layers(focused)` returns both. |
| `WindowCorner` | Window radii (`titlebar`, `unified_compact`, `unified`, `utility`; all 16), curve `expansion` and superellipse `exponent`. |
| `CornerShape` | `radius` (the API's or the design value), `fitted` (the radius that draws the measured silhouette) and `smoothing` of one element. |
| `CornerControls` | The corner radius of each control at mini, small, regular, large and extra large: `push`, `circular`, `help`, `text_field`, `popup`, `combo_button`, `checkbox`, `radio`, `segmented`, `segmented_square`, `slider_track`, `slider_knob`, `progress_bar`. |
| `WindowStyle` | `corner_radius` and `superellipse_exponent` of a dialog or pop-up. |
| `WindowResize` | Width of the resize band at an `edge` and a `corner`. |
| `Titlebar` | Server-side titlebar: `thickness`, `buttons`, `font`. |
| `TitlebarButtons` | `order` of the three lights, `diameter`, `spacing`, `inset` (from the left of the 32 px bar), `inset_compact` (from the left of the `TITLEBAR_CLIENT_HEIGHT` bar). |
| `Button` | One traffic light: its `kind`. |
| `ButtonKind` | `Close`, `Minimize`, `Maximize`. |
| `TrafficGlyphs` | Glyph geometry: `stroke`, `zoom_corner`, `close`, `minimize`, `zoom`, `zoom_exit`, `plus`. |
| `Segment`, `Triangle` | A line from `a` to `b`; three points. |
| `DoubleClick` | `interval_ms` and pointer `slop` of a double click. |
| `SettingsStrength`, `SettingsCheck`, `SettingsWallpaper` | Parts of `SettingsMetrics`: the tint-strength scale, the check list and the Wallpaper pane (preview, tiles, color dots, the collection sheet and the custom framing sheet with its desktop preview). |
| `*Metrics` | Sizes of one component; see the tables below. |

### Spacing

| Constant | Value |
|---|---|
| `SPACE_1`, `SPACE_2`, `SPACE_3`, `SPACE_4` | 4, 8, 12, 16 |
| `STACK_COLUMN_GAPS` | The gap in a `VStack` between a view of one kind and the next, 5 × 5 (above, below) for text, symbol, label, checkbox and any other control, read from SwiftUI; `HStack` is always `SPACE_2` |
| `STACK_TEXT_SIZES`, `STACK_TEXT_ABOVE`, `STACK_TEXT_BELOW` | What a text of each size adds to the gap under it and over it beside a view that is not a text (`.largeTitle` 8 and 5, `.title` 5 and 3, `.title2` 3 and 2, `.title3` 1 and 0), read from SwiftUI on macOS 27.2 |
| `HAIRLINE` | 1, the width of a separator |

### Type

| Constant | Size / line height | Weight | SwiftUI |
|---|---|---|---|
| `LARGE_TITLE` | 26 / 31, pitch 32 | 400 | `Font.TextStyle.largeTitle` |
| `TITLE` | 22 / 26 | 400 | `.title` |
| `TITLE_2` | 17 / 21, pitch 22 | 400 | `.title2` |
| `HEADLINE` | 13 / 16 | 700 | `.headline` |
| `BODY` | 13 / 16 | 400 | `.body` |
| `CALLOUT` | 12 / 15 | 400 | `.callout` |
| `CAPTION` | 11 / 14 | 500 | the kit's own, between `.subheadline` and `.caption` |
| `TITLE_3` | 15 / 19, pitch 20 | 400 | `.title3` |
| `SUBHEADLINE` | 11 / 14 | 400 | `.subheadline` |
| `FOOTNOTE` | 10 / 13 | 400 | `.footnote` and `.caption` |
| `CAPTION_2` | 10 / 13 | 500 | `.caption2` |
| `MONO` | 13 / 16 | 400 | `Font.monospaced()` |
| `TOOLBAR_TITLE` | 15 / 16 | 600 | the window title in a unified toolbar; the line is what centers its glyphs where AppKit draws them (ink rows 19–31 in a 52 px bar) |
| `ABOUT_NAME` | 22 / 26 | 650 | the app name of `AboutView` |

The line is the height of a SwiftUI `Text` on macOS 27.2 (`fittingSize`):
13 pt is 16 high, 12 pt 15, 15 pt 19, 17 pt 21, 26 pt 31, 22 pt 26, 11 pt 14 and 10 pt 13. Sizes and
weights are `NSFont.preferredFont(forTextStyle:)` on macOS 27.2. A control's text is centered in its box, so the line height only matters where a text sets the height of a row.
A wrapped `Text` advances each line by the style's `line_pitch`: a point more than
the line in `.largeTitle`, `.title2` and `.title3`, as SwiftUI's (five lines of
`.title3` are 99 high), while one line keeps the line height. `glyph_lift` raises
the glyphs in their line where Skia sets them lower than CoreText: 1 in `TITLE`,
`TITLE_2` and `FOOTNOTE`.

All name Adwaita Sans and `MONO` names Geist Mono; text shows the system's
faces instead ([Fonts](#fonts)). `SANS_FAMILIES` is the fallback list after the
system's sans (Adwaita Sans, Inter,
Noto Sans CJK SC, system-ui, sans-serif). `TEXT_LINE_HEIGHT` (1.3) is the
factor that turns a font size into the height of one line when
the kit measures text without laying it out.

### Motion

| Constant | Value |
|---|---|
| `DURATION_FAST_MS`, `DURATION_BASE`, `DURATION_SLOW_MS` | 120, 250, 420 ms |
| `THEME_FADE_WAIT_MS` | 150 ms: longest wait for clients before the compositor's theme fade |
| `NAVIGATION_PARALLAX`, `NAVIGATION_DIM` | 0.3 of the width and a black of 0.13 laid over the page below a push (Mac Catalyst; the dimming is one reading) |
| `SCROLL_POCKET_BLUR`, `SCROLL_POCKET_SCALE_*`, `SCROLL_POCKET_OFFSET_*` | 3.4 pt, and the tone `scale * content + offset` (0 to 1 units) of a scroll pocket: 0.5 and 0.5349 in light, 0.4 and 0.0976 in dark; fitted to the pixels of macOS 27.2.1 ([Shader needs](../research/shader-needs.md)) |
| `SPRING_DAMPING` | 5; no reader in the kit ([14](14-animations.md#spring)) |
| `DURATION_LIST_ROW_MS`, `DURATION_MENU_CLOSE_MS`, `DURATION_POPOVER_IN_MS`, `DURATION_POPOVER_OUT_MS` | 300, 200, 210, 120 ms: rows inserted or removed, a menu closing, a popover in and out (Mac Catalyst; [UIKit animations](../research/uikit-animations.md#2-motion-table)) |
| `SPRING_SHEET_MS`, `SPRING_ALERT_MS`, `SPRING_PUSH_MS` | 344, 274, 251 ms: the perceptual duration of a critically damped spring (`bounce` 0) of a sheet and its detents, an alert, a push and pop; the last is a fit |

### Materials

`MATERIAL: MaterialMetrics` holds one `MaterialThickness` per
`styles::Thickness`. A `ShapeStyle::Material` paints the glass tint of its
context (the window color, or the `Backdrop` base) at `opacity`; the kit blurs
nothing, and `blur` is for the compositor. With Reduce Transparency the tint is
opaque.

| Field | `opacity` | `blur` |
|---|---|---|
| `ultra_thin` | 0.35 | 12 |
| `thin` | 0.5 | 20 |
| `regular` | 0.65 | 28 (`MEDIA.material_blur`) |
| `thick` | 0.8 | 36 |
| `ultra_thick` | 0.92 | 44 |

### Window materials

`MATERIAL_<NAME>_LIGHT` and `MATERIAL_<NAME>_DARK` (`<NAME>` is one of the 14
materials of `NSVisualEffectView.Material`: `TITLEBAR`, `SELECTION`, `MENU`,
`POPOVER`, `SIDEBAR`, `HEADER_VIEW`, `SHEET`, `WINDOW_BACKGROUND`,
`HUD_WINDOW`, `FULL_SCREEN_UI`, `TOOL_TIP`, `CONTENT_BACKGROUND`,
`UNDER_WINDOW_BACKGROUND`, `UNDER_PAGE_BACKGROUND`) are `MaterialSet`s of
`components.material.visual-effect`. A `MaterialSet` has the `active` look, the
`inactive` look and the looks under Reduce Transparency (`reduced`) and under
Reduce Transparency with Increase Contrast (`reduced_contrast`); Increase
Contrast alone changes no blur. A `MaterialLook` is:

| Field | Unit | Meaning |
|---|---|---|
| `sigma` | px at scale 1 | Gaussian of the blur; 0 for a solid. `components.material.visual-effect.blur-sigma`, 27.6 |
| `saturation` | factor | `colorSaturate` about the Rec. 709 luma, clamped to 0..1 |
| `veil` | encoded RGBA | colour and alpha mixed over the saturated backdrop |
| `cap` | 0..1 | per-channel minimum after the veil (light schemes; 1 is none) |
| `floor` | 0..1 | per-channel maximum after the cap (dark schemes; 0 is none) |
| `gain`, `bias` | factor, 0..1 | affine map after the floor; 1 and 0 except in `selection` |

The six SwiftUI thicknesses are `MaterialSet`s of the same four looks (in a window
that is not key only `.bar` turns solid):
`MATERIAL_ULTRA_THIN_LIGHT` and `_DARK`, `MATERIAL_THIN_*`, `MATERIAL_REGULAR_*`,
`MATERIAL_THICK_*`, `MATERIAL_ULTRA_THICK_*` and `MATERIAL_BAR_*`
(`components.material.swiftui`, sigma 59.2). They are the AppKit values with the
wider blur.

`ctappearance::material::looks(material, scheme)` returns the set of a
`material::Material` (dark tint takes the dark one). `over(&look, backdrop)`
is the colour a look shows over one blurred backdrop pixel. The values are numbers of the macOS 27.2.1 stack, measured at
scale 1 and in the display's device space; the compositor applies them in its
own output space ([research](../research/macos-materials.md#10-materials-in-windows-measured-2026-11-22)).
The render server's layer stack is the only source of these numbers: one veil and one cap per surface cannot tell the saturation, the cap and the floor apart.

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
| `ACCENT_CHROMA_GAIN` | `f32` | Gain (2.5) of the accent's chroma share, so the chroma-only tint looks as strong as the old sRGB mix did. |
| `WALLPAPER_TINTING`, `WALLPAPER_TINTING_LIFT`, `WALLPAPER_TINTING_TARGET`, `WALLPAPER_TINTING_SIDEBAR`, `WALLPAPER_TINTING_SIDEBAR_LIFT`, `WALLPAPER_TINTING_GRID` | `[f32; 2]`, `[f32; 2]`, `f32`, `f32`, `f32`, `u32` | Share of the wallpaper's OKLab chroma a window takes in light and dark (0.2, 0.15), and share of the way its lightness moves to the target (0, 0.14), measured on macOS 26 (malaysia ADR 0065); that target, L 0.43; the sidebar's share of the chroma (1.8 times) and of the lift (0.5 times); cells per side of the wallpaper grid. |
| `TINT_STRENGTH` | `[f32; 4]` | Factor of each `TintStrength`: 0.5, 1, 1.5, 2. |
| `ACCENT_MIN_CONTRAST_ON_WINDOW`, `ON_ACCENT_MIN_CONTRAST` | `f32` | 3 and 4.5: contrast of the accent on the window and of ink on the accent. |
| `DOMINANT_LIGHTNESS`, `DOMINANT_MIN_CHROMA` | `[f32; 2]`, `f32` | Lightness range and minimum chroma of a wallpaper's dominant color. |
| `ACCENTS` | `[Accent; 8]` | The named accents with their colors in four looks ([Accents](#accents)); blue, purple, pink, red, orange, yellow, green, graphite. The keyboard focus ring is each one's `focus_ring`. |
| `ACCENT_SWATCHES` | `[(&str, Rgba); 8]` | The name and light `control` of each accent. |
| `ACCENT_RULE` | `AccentRule` | The fitted rule for an accent without a name ([Accents](#accents)). |
| `MULTICOLOR` | `[Rgba; 25]` | The stops of the Multicolor swatch, from the top clockwise, measured on System Settings; the last is the first. |
| `LEVEL_INDICATOR_COLORS`, `LEVEL_INDICATOR_COLORS_DARK`, `LEVEL_INDICATOR_COLORS_CONTRAST`, `LEVEL_INDICATOR_COLORS_CONTRAST_DARK` | `[Rgba; 3]` | Normal, warning and critical colors of a level indicator, in light, dark and Increase Contrast. |
| `SHELL_LIGHT`, `SHELL_LIGHT_OPAQUE`, `SHELL_DARK`, `SHELL_DARK_OPAQUE`, `SHELL_DARK_TINT`, `SHELL_DARK_TINT_OPAQUE` | `ShellPalette` | Menu bar and Dock, translucent and opaque. |
| `THEME_SHADOW_OPACITY` | `f32` | Shadow opacity of a shell's theme. |
| `MENU_LIGHT`, `MENU_LIGHT_OPAQUE`, `MENU_DARK`, `MENU_DARK_OPAQUE`, `MENU_DARK_TINT`, `MENU_DARK_TINT_OPAQUE` | `MenuPalette` | Menus, translucent and opaque. |
| `SIDEBAR_LIGHT`, `SIDEBAR_LIGHT_OPAQUE`, `SIDEBAR_DARK`, `SIDEBAR_DARK_OPAQUE`, `SIDEBAR_DARK_TINT`, `SIDEBAR_DARK_TINT_OPAQUE` | `Rgba` | Sidebar background, measured on macOS 27: `#ededed` in light and `#262626` in dark; the `_OPAQUE` ones are the same with Reduce Transparency on: `#f7f7f7` in light and `#141414` in dark. |
| `TOOLBAR_BG_LIGHT`, `TOOLBAR_BG_DARK`, `TOOLBAR_BG_DARK_TINT` | `Rgba` | Background of the unified title bar beside a sidebar (`Surfaces::toolbar`): `#ffffff` in light and `#262626` in dark. |
| `TOOLBAR_LINE_LIGHT`, `TOOLBAR_LINE_DARK`, `TOOLBAR_LINE_DARK_TINT` | `Rgba` | The 1 px line under that bar (`Surfaces::toolbar_line`): `#0000001a` in light and `#ffffff13` in dark. |
| `SIDEBAR_GLASS_BASE_LIGHT`, `SIDEBAR_GLASS_BASE_DARK`, `SIDEBAR_GLASS_BASE_DARK_TINT` | `Rgba` | Tone of the sidebar glass at rest: a gray `#202020` in dark (L 0.245 in OKLab) and `#e5e7e8` in light (L 0.927), a little darker than the `#ebedee` measured on macOS so that the tint shows; dark tint is an estimate. |
| `SIDEBAR_GLASS_SOLID_LIGHT`, `SIDEBAR_GLASS_SOLID_DARK`, `SIDEBAR_GLASS_SOLID_DARK_TINT` | `Rgba` | Tone of the sidebar glass when its window is not active. Light is `#f1f2f3`, measured; dark is the tone at rest. |
| `SIDEBAR_PALETTE_LIGHT`, `SIDEBAR_PALETTE_DARK`, `SIDEBAR_PALETTE_DARK_TINT` | `SidebarPalette` | Sidebar inks per theme: the selection (`#0270f5` in light, `#007aff` in dark) and the rest. |
| `TOOLGROUP_BG_LIGHT`, `TOOLGROUP_BG_LIGHT_OPAQUE`, `TOOLGROUP_BG_DARK`, `TOOLGROUP_BG_DARK_OPAQUE`, `TOOLGROUP_BG_DARK_TINT`, `TOOLGROUP_BG_DARK_TINT_OPAQUE` | `Rgba` | Toolbar group background; with Reduce Transparency (`_OPAQUE`): `#ffffff` in light and `#333333` in dark, measured on macOS 27. |
| `TOOLGROUP_PALETTE_LIGHT`, `TOOLGROUP_PALETTE_DARK`, `TOOLGROUP_PALETTE_DARK_TINT` | `ToolgroupPalette` | Toolbar group lines per theme. |
| `RING_TRACK_*`, `BAR_TRACK_*`, `LEVEL_TRACK_*`, `LEVEL_BORDER_*`, `SEARCH_CLEAR_*`, `TOKEN_FILL_*`, `DISCLOSURE_TRIANGLE_*`, `DATE_FIELD_*`, `DATE_FIELD_BORDER_*`, `STEPPER_DIVIDER_*` | `Rgba` | One control's color per theme, read by its style; the last five have `_CONTRAST_*` twins. |

`SHELL_*_OPAQUE` and `MENU_*_OPAQUE` are their translucent twins painted over
the theme's window background; `SIDEBAR_*_OPAQUE` and `TOOLGROUP_BG_*_OPAQUE`
are the colors measured with Reduce Transparency. `resolve` picks them for
`reduce_transparency` or `contrast`.

### Windows and titlebars

| Constant | Type | Read by |
|---|---|---|
| `WINDOW_MIN_SIZE`, `WINDOW_DEFAULT_SIZE` | `(u32, u32)` | 480 × 360, which an app passes to `.min_size(Some(..))`, and 640 × 420, the default `WindowOptions::size` ([8](08-windows.md)) |
| `WINDOW_CORNER` | `WindowCorner` | `style::panel_edges` (`expansion`), the compositor |
| `CORNER_WINDOW`, `CORNER_SHEET`, `CORNER_ALERT`, `CORNER_POPOVER`, `CORNER_MENU`, `CORNER_TOOLTIP` | `CornerShape` | The kit (`CORNER_WINDOW`, `CORNER_SHEET`, `CORNER_ALERT`, `CORNER_MENU`, `CORNER_TOOLTIP`) and the compositor: the measured radius and curve of each element |
| `CORNER_CONTROLS` | `CornerControls` | The radii of the controls by size, read from the API |
| `CORNER_SMOOTHING`, `CORNER_CONCENTRIC_REGION` | `f32` | `style::continuous` and `ctappearance::corner::concentric`: the smoothing of the continuous curve (0.745) and the width of the concentric region as a share of the container's radius (1.0) |
| `WINDOW_RESIZE` | `WindowResize` | `platform` |
| `WINDOW_CLIENT_RESIZE` | `WindowResize` | The compositor: the resize band around a window that decorates itself |
| `DIALOG` | `WindowStyle` | No reader in the kit |
| `DIALOG_PADDING`, `DIALOG_ALERT_*`, `DIALOG_BUTTON_WIDTH`, `DIALOG_SHEET_WIDTH`, `DIALOG_SHEET_HEIGHT`, `DIALOG_SOLID_*` | `f32`, `u32`, `Rgba`, `[f32; 2]` | `views::feedback`, `styles` (`AlertStyle`, `SheetStyle`, `AboutViewStyle`) |
| `POPUP` | `WindowStyle` | The compositor, for pop-up corners; `platform` (the macOS popover arrow) |
| `PANEL_GLOW_DEPTH` | `f32` | `style::panel_edges` and the compositor: depth in points of the `PanelLine::glow` |
| `SHADOW`, `SHADOW_DARK` | `ShadowStyle` | `styles` (`WindowChromeStyle`), the compositor |
| `TITLEBAR` | `Titlebar` | Server-side titlebar of the compositor, `ctappearance::traffic`, `traffic`, `styles` (`TitlebarStyle`) |
| `TITLEBAR_CLIENT_HEIGHT`, `TITLEBAR_PADDING_X` | `u32`, `f32` | `styles` (`TitlebarStyle`), `chrome` ([9](09-titlebars.md)) |
| `TITLEBAR_TITLE_GAP` | `f32` | The compositor's titlebar |
| `TITLEBAR_EXPANDED_TITLE_HEIGHT`, `TITLEBAR_EXPANDED_TITLE_GAP`, `TITLEBAR_EXPANDED_TOOLBAR_BOTTOM` | `f32` | The expanded `Titlebar` (SwiftUI `.expanded`): a 28 high title row with the title 15 after the lights, and a toolbar row whose items sit 8 above its middle, 4 under the title row ([9](09-titlebars.md)) |
| `TITLEBAR_TOOLBAR_START` | `TitlebarToolbarStart` | Where the first toolbar item starts: 16 (unified) and 22 (compact) after the lights, 11 from the edge of the expanded toolbar row ([9](09-titlebars.md)) |
| `TITLEBAR_SYSTEM_VEIL` | `Rgba` | White at 9/255: what AppKit's title bar lays over the window material, 8 levels in Dark and none in Light ([9](09-titlebars.md)) |
| `TITLEBAR_FULLSCREEN_START` | `TitlebarToolbarStart` | Where the title or the first item starts in full screen, with no lights: 22 (unified), 14 (compact), 11 (expanded) from the edge ([9](09-titlebars.md)) |
| `TITLEBAR_DOUBLE_CLICK` | `DoubleClick` | `titlebar`, the compositor |
| `TITLEBAR_REVEAL` | `TitlebarRevealMetrics` | `chrome`, `styles` (`WindowChromeStyle`): the pill, reveal zone and corner, and resting edges of a hidden bar ([9](09-titlebars.md#window-chrome)) |
| `FULLSCREEN_BAR`, `FULLSCREEN_GLASS` | `FullscreenBarMetrics`, `FullscreenGlassMetrics` | `platform` (`FULLSCREEN_BAR`), `styles` (`TopGlassStyle`) |
| `HUD_BACKGROUND` | `Rgba` | Background of a HUD window (`platform`) |

### Traffic lights

| Constant | Type | What it is |
|---|---|---|
| `TRAFFIC_GLYPHS` | `TrafficGlyphs` | Geometry of the close, minimize, zoom and plus glyphs, in the 14 pt box of the light. |
| `TRAFFIC_RING_WIDTH`, `TRAFFIC_MARGIN` | `f32` | Width of the ring along the edge of a light, and the margin of its image around the 14 pt disc. |
| `TRAFFIC_GLYPH_FADE_MS` | `u64` | Cross-fade of the zoom glyph between its enter and exit shapes. |
| `TRAFFIC_PRESS` | `TrafficPress` | Scale and `dodge` of a pressed light. |
| `TRAFFIC_CLOSE_MINIMIZE_DARKEN`, `TRAFFIC_ZOOM_LIGHTEN` | `f32` | Shifts that derive tinted lights from one color. |

### Control metrics

| Constant | Type | Read by |
|---|---|---|
| `BUTTON`, `BUTTON_DESTRUCTIVE_FILL` | `ButtonMetrics`, `f32` | `styles`, `font_panel` |
| `ICON_BUTTON` | `IconButtonMetrics` | `controls`, `styles` |
| `SWITCH` | `SwitchMetrics` | `styles` |
| `CHECKBOX` | `CheckboxMetrics` | `styles` |
| `SLIDER`, `SLIDER_FAT`, `SLIDER_KEY_STEP` | `SliderMetrics`, `SliderFatMetrics`, `f32` | `styles`, `level` |
| `KNOB_SHADOW` | `ShadowLayer` | `styles` |
| `MATERIAL` | `MaterialMetrics` | `styles` ([Materials](#materials)) |
| `SEGMENTED` | `SegmentedMetrics` | `styles` |
| `STEPPER` | `StepperMetrics` | `styles` |
| `HELP_LINK` | `HelpLinkMetrics` | `styles` |
| `TOOLBAR_SEARCH_WIDTH` | `f32` | `styles` |
| `TEXT_FIELD`, `SEARCH_FIELD`, `TEXT_EDITOR` | `TextFieldMetrics`, `SearchFieldMetrics`, `TextEditorMetrics` | `styles` |
| `TOKEN_FIELD` | `TokenFieldMetrics` | `styles` |
| `SPELLING` | `SpellingMetrics` | `spelling` |
| `COMBO_BOX` | `ComboBoxMetrics` | `styles` |
| `PATH_CONTROL` | `PathControlMetrics` | `styles` |
| `DATE_PICKER` | `DatePickerMetrics` | `styles` |
| `IMAGE_WELL` | `ImageWellMetrics` | `styles` |
| `LEVEL_INDICATOR`, `GAUGE` | `LevelIndicatorMetrics`, `GaugeMetrics` | `styles`; `indicator` reads the number of stars |
| `LEVEL_BOTTOM`, `LEVEL_RING_RADII` | `[f32; 4]` | `styles` (the capacity indicator) |
| `PROGRESS` | `ProgressMetrics` | `styles` |
| `BADGE` | `BadgeMetrics` | `styles` (`BadgeStyle`) |
| `FOCUS_RING` | `FocusRingMetrics` | `interaction` |
| `SYMBOL` | `SymbolMetrics` | `symbol`, `styles` |
| `TOOLTIP` | `TooltipMetrics` | `tooltip` |
| `POPOVER` | `PopoverMetrics` | `popover`, the compositor |
| `MENU_POPOVER` | `MenuPopoverMetrics` | `theme`, `popup_button`, `styles` |
| `MENU_MARK_TEXT_GAP`, `MENU_ICON_TEXT_GAP`, `MENU_SUBMENU_INTENT_MS`, `MENU_SUBMENU_REVEAL_GAP`, `MENU_SCROLL_TICK_MS`, `MENU_SEARCH_ICON`, `MENU_SEARCH_ICON_GAP`, `MENU_SEARCH_CLEAR`, `MENU_SEARCH_CLEAR_GLYPH`, `MENU_SEARCH_PADDING_X`, `MENU_SEARCH_CLEAR_MIX`, `MENU_HELP_ICON` | `f32`, `u64`, `[f32; 2]` | `menu` ([7](07-menus-and-popovers.md)) |
| `MENU_SEPARATOR_INK_MIX` | `f32` | `Resolved::menu_separator` |
| `DRAG` | `DragMetrics` | `drag` |
| `GESTURE` | `GestureMetrics` | `gesture` |
| `SCROLL_PHYSICS` | `ScrollPhysicsMetrics` | `scroll_physics` |

### View metrics

| Constant | Type | Read by |
|---|---|---|
| `GLASS` | `GlassMetrics` | Per color scheme (`_light`, `_dark`): `chroma` (0.22, 0.05), the share of the wallpaper's OKLab chroma the glass takes; `chroma_cap` (0.05, 0.04), the chroma it saturates towards (`backdrop::take_chroma_capped`); `lightness` (0.00, 0.00), the share of the lightness of what is behind that shows through; `blur_passes` (4, 6), how many times the glass blurs what is behind it, also for the edge; `edge_intensity` (0.70, 0.70), `edge_span` (53, 60 logical pixels), `edge_chroma` (0.58, 0.45), `edge_lightness` (0.00, 0.10) and `edge_cap` (0.08, 0.18), `rim_lift` (0.03, 0.06) and `rim_lift_edge` (0.02, 0.05) and `edge_offset` (11 logical pixels outside the window): the wallpaper that shows along the left, top and bottom edges of a panel and fades inwards. Shared: `accent_share` (0.35), the part of `chroma` that stays when the accent tints the sidebar; `content_span` (200 logical pixels) and `content_opacity` (0.60): on a panel that does not reach the window's right edge, the glass's `base` veils the fill, growing smoothly from `content_span` before that edge to `content_opacity` at it, so the wallpaper's colours fade towards the content without disappearing; `rim_opacity` (1.00), `rim_chroma` (1.35) and `rim_cap` (0.25): the glass panel's border is derived from the fill under it, `rim_lift` lighter (`rim_lift_edge` more where the wallpaper shows) and `rim_chroma` times its chroma with its hue, up to `rim_cap`; `shadow_opacity` (0.13), `shadow_lightness` (0.20), `shadow_span` (25) and `shadow_tint` (1.00): the glass panel's shadow on the window, only in light; `glass_shadow` gives none in dark. Read `chroma` per scheme with `glass_chroma`, and the others with `ctshell::resolve` ([Glass](10-sidebar.md#glass)). Values chosen by the owner on the sidebar mockup |
| `SIDEBAR` | `SidebarMetrics` | `sidebar`, `styles` (the sidebar styles); `item_weight` and `item_selected_weight` (both 400) weigh the row labels |
| `TOOLGROUP` | `ToolgroupMetrics` | `compact`, `titlebar`, `views::split`, `styles` (`ToolGroupStyle`) |
| `SPLIT_VIEW` | `SplitViewMetrics` | `views::split`, `styles` |
| `SPLIT_PANES` | `SplitPanesMetrics` | `views::split_panes`, `styles` (`SplitPanesStyle`) |
| `TABLE` | `TableMetrics` | `views::table`, `styles` (`TableStyle`); `plain_row_height`, `row_size_font`, `row_size_icon`, `source_list_row_height`, `icon_box` and `icon_gap` are the classic `NSTableView` styles |
| `COLLECTION` | `CollectionMetrics` | `styles` (`CollectionViewStyle`) |
| `LIST_GROUP` | `ListGroupMetrics` | `views::form` |
| `LIST_SECTION` | `ListSectionMetrics` | `views::list` |
| `GROUP_BOX` | `GroupBoxMetrics` | `styles` (`GroupBoxStyle`) |
| `COLOR_PICKER` | `ColorPickerMetrics` | `color_picker`, `styles` (`ColorPickerStyle`) |
| `CIRCULAR_SLIDER` | `CircularSliderMetrics` | `styles` (`CircularSliderStyle`) |
| `DISCLOSURE` | `DisclosureMetrics` | `font_panel`, `styles` (`DisclosureGroupStyle`, `OutlineViewStyle`, `RuleEditorStyle`) |
| `TAB_VIEW` | `TabViewMetrics` | `views::tabs`, `styles` (`TabViewStyle`); the arrays hold light, dark, then both with Increase Contrast |
| `TOOLBAR_TABS` | `ToolbarTabsMetrics` | `styles` (`TabViewStyle`) |
| `SCROLL_BAR` | `ScrollBarMetrics` | Scroll views, tables and outlines, `styles` (`ScrollViewStyle`); `opacity` and `track_opacity` hold light, dark, then both with Increase Contrast |
| `SETTINGS` | `SettingsMetrics` | `app`, `patterns::settings`, `platform`, `titlebar`, `styles` |
| `FONT_PANEL` | `FontPanelMetrics` | `font_panel` |
| `MEDIA` | `MediaMetrics` | `media`, `styles`; `button_font_weight` is the weight of the `MediaButton` label |
| `BACKDROP` | `BackdropMetrics` | `backdrop`, `ctappearance::backdrop`, `styles` (`BackdropStyle`) |
| `BACKDROP_SCENES` | `BackdropScenes` | Preset gradient `dusk` (recipe `titlebar_over_gradient`) |
| `LABEL_ICON_WIDTH`, `LABEL_ICON_HEIGHT`, `LABEL_ICON_FRAME`, `LABEL_GAP` | `f32` | `Label`: the box of its symbol (16 × 16.5), the kit's symbol frame per point of the font (1.25) and the gap before the title (6), as SwiftUI lays out `Label("Doc", systemImage:)` |
| `CONTENT_UNAVAILABLE_ICON`, `CONTENT_UNAVAILABLE_ICON_LINE`, `CONTENT_UNAVAILABLE_ICON_GAP`, `CONTENT_UNAVAILABLE_TITLE`, `CONTENT_UNAVAILABLE_DESCRIPTION`, `CONTENT_UNAVAILABLE_DESCRIPTION_GAP`, `CONTENT_UNAVAILABLE_ACTIONS_GAP` | `f32`, `Font` | `ContentUnavailableView`: the symbol frame (45, the kit's frame for SF's 36 pt) and its 44 high line, the gaps above the title, the description and the actions (20, 6, 19) and the two fonts, as SwiftUI lays them out |

### Shell

The kit reads these shell values: `ctappearance` puts the glass in `Resolved`,
and the popover shapes and `pop_up_menu` draw the Dock's label and menu. The
shell's own tokens are not part of the kit.

| Constants | What they size |
|---|---|
| `MENUBAR_HEIGHT`, `MENUBAR_TINT_SPREAD` | The menu bar: its slim height, which the focus-mode bar takes, and the share of its width over which `WallpaperGrid::across` blurs the wallpaper |
| `MENU_GLASS_VEIL_LIGHT`, `MENU_GLASS_VEIL_DARK`, `MENU_GLASS_LIFT_LIGHT`, `MENU_GLASS_LIFT_DARK`, `MENU_GLASS_CAP_LIGHT`, `MENU_GLASS_CAP_DARK`, `MENU_GLASS_CHROMA`, `MENU_GLASS_SPREAD`, `NOTIFICATION_GLASS_VEIL_LIGHT`, `MENU_DARK_INK_ABOVE` | The glass of the menus and notifications and the luma above which a panel takes dark ink; `ctappearance` reads them into `Resolved` ([Glass over a backdrop](#glass-over-a-backdrop)) |
| `DOCK_RIM_LIGHT`, `DOCK_RIM_DARK`, `DOCK_DIVIDER_LIGHT`, `DOCK_DIVIDER_DARK`, `DOCK_RIM_CORNER_LIGHT`, `DOCK_RIM_CORNER_DARK`, `DOCK_RIM_SIDE_LIGHT`, `DOCK_RIM_SIDE_DARK`, `DOCK_LENS_LIGHT`, `DOCK_LENS_DARK`, `DOCK_RIM_GLOW`, `DOCK_RIM_GLOW_LIGHT`, `DOCK_TONE_LIGHT`, `DOCK_TONE_DARK` | The Dock's frame (the luminance of the rim as `[lift, share]` of the glass's, light and dark; how far a divider moves the glass's luminance; how the frame turns from rim to outline around a corner as `[decay, outline from, outline to]` in degrees; how much of the rim along the top and bottom stays on the left and right sides (`0.0` to `1.0`; the translucent Dock only, never the menus or the solid Dock); how the glass bends the backdrop along the frame's normal as `[depth, strength, split, blur]`, light and dark; the glow inside the rim pixel by pixel as shares of it, with its own steps for the translucent light Dock; and, for the translucent Dock, `DockTone`: the glass and outline taken from the colour behind the glass, its lightness bent down to `cap`, while the rim keeps that colour's hue, scaling it per channel by at most `gain` and making up the rest of its light in white) |
| `DOCK_LABEL_RADIUS` | The corner of the name label over a Dock icon |
| `DOCK_LABEL_ARROW_WIDTH`, `DOCK_LABEL_ARROW_HEIGHT`, `DOCK_LABEL_ARROW_TIP_RADIUS`, `DOCK_LABEL_ARROW_FILLET` | The arrow of that label, a popover with the `Label` shape |
| `DOCK_MENU_PADDING`, `DOCK_MENU_ITEM_PADDING_X`, `DOCK_MENU_SEPARATOR_MARGIN`, `DOCK_MENU_MIN_WIDTH` | An icon's menu: row metrics and width ([`pop_up_menu`](07-menus-and-popovers.md#pop_up_menu)) |
| `DOCK_MENU_ARROW_INSET`, `DOCK_MENU_ARROW_TIP_RADIUS`, `DOCK_MENU_ARROW_FILLET` | Its arrow: distance from the corner, tip and fillet of the `Menu` shape |

## Helpers

| Item | What it does |
|---|---|
| `init_tracing()` | Installs the `tracing` subscriber: logs to stdout filtered by `RUST_LOG` (default `info`), and to Tracy with the `tracy` feature ([19](19-profiling.md)). Call it once, first thing in `main`. |
| `ProbeExt::probe(id)` | Sets the accessibility id of a `rect`, so tests and tools can find the node: `rect().probe("sidebar-toggle")`. |

Next: [5. Controls](05-controls.md).
