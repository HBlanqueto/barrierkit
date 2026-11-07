# How other toolkits and standards customize control appearance

**Goal:** show how Flutter, Compose, Material, GTK/libadwaita, Qt, CSS, DTCG tokens and AwesomeWM theme and override controls, with a source for every claim.
**Scope:** research only; the decision built on it is
[ADR 0064](../decisions/0064-styles-and-themes.md) and the guide for agents is
[`docs/ai/styles.md`](../ai/styles.md).
**Reader:** anyone who changes the styling system.
**Input:** the question in the first section. **Output:** findings and
"Lessons for BarrierKit".

## Steps

1. Read the section of the toolkit or topic you need.
2. Follow its source links; primary sources are official documentation.
3. Check the lesson against [ADR 0064](../decisions/0064-styles-and-themes.md)
   before changing the design.

Date: 2026-11-07. Scope: how toolkits and standards let a control's look be set
at theme level, inherited by a subtree, or overridden on one instance, and how
complete alternative themes are built from those variables.

Every claim carries a source. **[primary]** means official docs, specs or
source code. **[secondary]** means a blog post or forum thread, used only as a
pointer or for a maintainer's stated position. Where a page could not be read
by machine, this file says so.

---

## 1. Flutter (Material library)

### 1.1 Theme data and inheritance
- `Theme` passes its `ThemeData` (colors, typography and more) to its child and
  every widget below it. `Theme.of(context)` returns the closest enclosing
  `ThemeData`, and any widget that calls it rebuilds when the theme changes.
  **[primary]** https://api.flutter.dev/flutter/material/Theme-class.html
- `MaterialApp` has `theme`, `darkTheme`, `highContrastTheme` and
  `highContrastDarkTheme`, plus `themeMode`. High contrast is a full theme
  slot, not a filter. **[primary]**
  https://api.flutter.dev/flutter/material/MaterialApp-class.html
- `DefaultTextStyle` sets the text style of a subtree for `Text` widgets that
  have no explicit style. `DefaultTextStyle.merge` inherits the style in scope
  and overrides selected fields. **[primary]**
  https://api.flutter.dev/flutter/widgets/DefaultTextStyle-class.html

### 1.2 Theme extensions (custom tokens with animation)
- `ThemeExtension<T>` is "an interface that defines custom additions to a
  ThemeData object". Subclasses implement `copyWith` and `lerp`; `lerp`
  "will ensure smooth transitions of properties when switching themes".
  **[primary]** https://api.flutter.dev/flutter/material/ThemeExtension-class.html
- Extensions are passed as an iterable to `ThemeData(extensions: …)` or
  `copyWith`, and read with the `extension<T>()` convenience method.
  **[primary]** https://api.flutter.dev/flutter/material/ThemeData/extensions.html

### 1.3 Animated theme changes
- `AnimatedTheme` animates `ThemeData` changes, and `MaterialApp` wraps its
  content in one. Theme changes therefore cross-fade by interpolating every
  field. **[primary]** https://api.flutter.dev/flutter/material/Theme-class.html
- `MaterialApp.themeAnimationStyle` overrides the duration (default 200 ms) and
  curve (default `Curves.linear`). `AnimationStyle.noAnimation` disables the
  theme animation. **[primary]**
  https://api.flutter.dev/flutter/material/MaterialApp/themeAnimationStyle.html
- `AnimationStyle` has `curve`, `duration`, `reverseCurve` and
  `reverseDuration`. A `Duration.zero` disables that direction. It has
  `copyWith` and a static `lerp`. Widgets that accept it include
  `ExpansionTile`, `PopupMenuButton`, `showModalBottomSheet` and `MaterialApp`.
  **[primary]** https://api.flutter.dev/flutter/animation/AnimationStyle-class.html

### 1.4 Component themes and shapes as types (Slider)
- `SliderThemeData` "overrides the default values of visual properties for
  descendant Slider widgets". It is set globally in `ThemeData.sliderTheme`,
  and a `SliderTheme` widget overrides it for a subtree. `SliderTheme.of` reads
  the active one, and `copyWith` customizes it. **[primary]**
  https://api.flutter.dev/flutter/material/SliderThemeData-class.html
- It has state-specific colors (`activeTrackColor`, `inactiveTrackColor`,
  `disabledActiveTrackColor`, `thumbColor`, `disabledThumbColor`,
  `overlayColor`, `valueIndicatorColor` and more) and `trackHeight`. It also
  holds pluggable shape objects: `trackShape`, `thumbShape`, `overlayShape`,
  `tickMarkShape` and `valueIndicatorShape`, plus range-slider equivalents.
  `thumbSize` and `mouseCursor` are `WidgetStateProperty` values. **[primary]**
  (same URL)
- A shape is a class with a paint method. `SliderComponentShape.paint` receives
  the canvas context and center, plus `activationAnimation`, `enableAnimation`,
  `isDiscrete`, `labelPainter`, `parentBox`, `sliderTheme`, `textDirection`,
  `value`, `textScaleFactor` and `sizeWithOverflow`. The shape gets the
  animation progress of state changes, the whole theme and the text direction.
  **[primary]** https://api.flutter.dev/flutter/material/SliderComponentShape/paint.html
- `SliderTrackShape` has `getPreferredRect`, which positions the thumb and tick
  marks relative to the track, and `paint`. Shipped subclasses include
  `RoundedRectSliderTrackShape` (the default), `RectangularSliderTrackShape` and
  `GappedSliderTrackShape`. **[primary]**
  https://api.flutter.dev/flutter/material/SliderTrackShape-class.html

### 1.5 Per-state resolution: ButtonStyle and WidgetStateProperty
- `WidgetStateProperty<T>` is an interface "for classes that resolve to a value
  of type T based on a widget's interactive 'state'". The state is a set of
  `WidgetState` values such as `focused`, `hovered` and `pressed`. You build one
  with `resolveWith(callback)` or `fromMap`. **[primary]**
  https://api.flutter.dev/flutter/widgets/WidgetStateProperty-class.html
- `MaterialStateProperty` is a deprecated typedef for `WidgetStateProperty`,
  deprecated after v3.19.0-0.3.pre. **[primary]**
  https://api.flutter.dev/flutter/material/MaterialStateProperty.html
- `ButtonStyle` fields are mostly `WidgetStateProperty`: `backgroundColor`,
  `foregroundColor`, `overlayColor`, `shadowColor`, `surfaceTintColor`,
  `iconColor`, `textStyle`, `padding`, `minimumSize`, `fixedSize`,
  `maximumSize`, `elevation`, `shape`, `side` and `mouseCursor`. It also has
  `animationDuration`, `splashFactory`, `visualDensity`, `tapTargetSize` and
  `backgroundBuilder`/`foregroundBuilder`. Every field defaults to null, which
  means "use the default". In the doc example, the resolver returns a
  translucent primary color when `states.contains(WidgetState.pressed)` and
  returns `null` otherwise, falling back to the component default. `merge`
  fills null fields from another style. **[primary]**
  https://api.flutter.dev/flutter/material/ButtonStyle-class.html
- Precedence: "Non-null properties of this style override the corresponding
  properties in themeStyleOf and defaultStyleOf." A `WidgetStateProperty` that
  resolves to non-null overrides the same way. The order is instance `style` >
  component theme (`themeStyleOf`) > widget default (`defaultStyleOf`, derived
  from the theme's `colorScheme` and `textTheme`). The merge is per field, per
  state. **[primary]**
  https://api.flutter.dev/flutter/material/ButtonStyleButton/style.html and
  https://api.flutter.dev/flutter/material/ButtonStyleButton-class.html

### 1.6 Shape, shadow and blur primitives
- `ShapeBorder` has `getOuterPath`, `getInnerPath`, `paint`, `dimensions`,
  `scale`, `lerpFrom` and `lerpTo`. The lerp methods work across classes
  ("possibly of another class"), so a theme can animate from a circle to a
  rounded rectangle. **[primary]**
  https://api.flutter.dev/flutter/painting/ShapeBorder-class.html
- `RoundedSuperellipseBorder` gives rounded corners "following the shape of an
  RSuperellipse", which "mimics the rounded rectangle style commonly seen in
  iOS design". **[primary]**
  https://api.flutter.dev/flutter/painting/RoundedSuperellipseBorder-class.html
- `BackdropFilter` "applies a filter to the existing painted content and then
  paints child". It is bounded by the nearest clip; without a clip it covers
  the full screen. It is "relatively expensive, especially if the filter is
  non-local, such as a blur". Filters that share a `BackdropGroup` key can be
  merged by the engine into one operation. It has an `enabled` flag.
  **[primary]** https://api.flutter.dev/flutter/widgets/BackdropFilter-class.html

### 1.7 Platform accessibility signals
- `MediaQueryData` has `disableAnimations`, `highContrast`, `boldText`,
  `invertColors`, `accessibleNavigation`, `textScaler` and
  `platformBrightness`. These are inputs that widgets and themes read.
  **[primary]** https://api.flutter.dev/flutter/widgets/MediaQueryData-class.html

---

## 2. Jetpack Compose (Material 3)

### 2.1 Theme and inheritance
- `MaterialTheme(colorScheme, typography, shapes) { … }`. "When you customize
  these values, your changes are automatically reflected in the M3 components."
  `Shapes` has five sizes, `extraSmall` to `extraLarge`, for example
  `RoundedCornerShape(4.dp)` to `RoundedCornerShape(24.dp)`. Elevation is
  mainly tonal; `Surface` has both `tonalElevation` and `shadowElevation`.
  **[primary]** https://developer.android.com/develop/ui/compose/designsystems/material3
- `CompositionLocal` creates "tree-scoped named objects". `MaterialTheme`
  provides `LocalColorScheme`, `LocalShapes` and `LocalTypography`.
  `CompositionLocalProvider(Local… provides value)` overrides a value for a
  subtree, and `current` returns the closest ancestor's value.
  `compositionLocalOf` invalidates only the readers when the value changes.
  `staticCompositionLocalOf` recomposes the whole provided content and suits
  values that rarely change. The guidance: a local "should have a good default
  value", and locals should be used only for tree-scoped concepts. **[primary]**
  https://developer.android.com/develop/ui/compose/compositionlocal
- `LocalContentColor` is the canonical example of an inherited contextual
  value: `Surface` provides a content color that contrasts with its own
  background. **[primary]** (same URL)

### 2.2 Custom design systems
- There are three documented approaches: extend `MaterialTheme` (for example
  an `ExtendedColors` held in a `staticCompositionLocalOf`), replace one
  subsystem, or build a fully custom design system. A custom system uses
  `@Immutable` data classes, its own locals and an `object CustomTheme` with
  `@Composable` getters. **[primary]**
  https://developer.android.com/develop/ui/compose/designsystems/custom
- A fully custom theme must also supply `TextSelectionColors` and the
  indication (ripple); otherwise Material components use their own values.
  **[primary]** (same URL)

### 2.3 Component defaults and per-state colors
- Customization goes through `XxxDefaults` factories that return `XxxColors`
  or `XxxElevation` objects, for example `CardDefaults.cardColors(...)` with
  disabled variants and `CardDefaults.cardElevation(defaultElevation,
  pressedElevation, focusedElevation)`. **[primary]**
  https://developer.android.com/develop/ui/compose/designsystems/material3
- `Slider(colors = SliderDefaults.colors(thumbColor, activeTrackColor,
  inactiveTrackColor, …))`. `Slider` also accepts `thumb` and `track`
  composables for full replacement. **[primary]**
  https://developer.android.com/develop/ui/compose/components/slider
- The Compose component API guidelines **[primary]**
  https://android.googlesource.com/platform/frameworks/support/+/androidx-main/compose/docs/compose-component-api-guidelines.md
  - "All component default expressions should either be inline or live in the
    top level object called `ComponentDefaults`". Defaults are public so
    wrappers can reuse them.
  - `ComponentColors`/`ComponentElevation` classes "allow for more granular
    control, where the user can specify the enabled and disabled
    colors/elevation separately". State is resolved by methods such as
    `backgroundColor(enabled)`.
  - "Prefer explicit inputs … such as function parameters" over implicit
    `CompositionLocal` inputs. Theme locals are read in default expressions,
    not in the body, so a caller can override them.
  - "Avoid grab-bag style parameters and classes, akin to `ComponentStyle` or
    `ComponentConfiguration`." Prefer separately named variants
    (`Button`/`OutlinedButton`/`TextButton`).
  - "Every component that emits UI should have a modifier parameter", applied
    first in the root's chain.
  - Slots (`@Composable` lambdas) let a caller pass "any component with any
    styling".
  - "Avoid making parameters nullable to utilize `null` as a 'use default in
    the implementation' signal." This is the opposite of Flutter's
    null-means-default.

### 2.4 Modifier ordering
- "The order of modifier functions is significant." `clickable` then `padding`
  makes the padding clickable; the reverse does not. The docs present this as
  the replacement for the box model: there is no margin, only ordered padding.
  `padding(start/end)` follows layout direction; `absolutePadding` does not.
  **[primary]** https://developer.android.com/develop/ui/compose/modifiers

### 2.5 Shadows
- `Modifier.shadow(elevation, shape)` gives a Material elevation shadow.
  `Modifier.dropShadow(shape, Shadow(radius, spread, color, offset, brush,
  alpha))` and `Modifier.innerShadow(...)` give full control. They can be
  stacked, take a `Brush` for gradient shadows, and animate through
  `updateTransition`. `innerShadow` draws over content, so it must come after
  `background`. **[primary]**
  https://developer.android.com/develop/ui/compose/graphics/draw/shadows

### 2.6 Indication (pressed and hover feedback)
- `IndicationNodeFactory` replaces `Indication#rememberUpdatedInstance`.
  `RippleConfiguration` and `LocalRippleConfiguration` configure ripples within
  a subtree. Providing `null` disables them: `CompositionLocalProvider(
  LocalRippleConfiguration provides null)`. Global changes now go through a
  design system's own ripple built on `createRippleModifierNode()`.
  **[primary]**
  https://developer.android.com/develop/ui/compose/touch-input/user-interactions/migrate-indication-ripple
- From `material3` 1.5.0-beta01, `LocalRippleThemeConfiguration` configures
  the ripple theme for all Material components. **[primary]**
  https://developer.android.com/develop/ui/compose/designsystems/custom

### 2.7 Motion at theme level
- `MotionScheme` "provides all the FiniteAnimationSpecs for a MaterialTheme".
  It has `standard()` and `expressive()`, and the functions
  `default/fast/slow` × `Spatial/Effects` `Spec<T>()`. Spatial specs are for
  bounds and shape changes; effects specs are for color and alpha. "You can
  customize the motion scheme for all components in the MaterialTheme."
  **[primary]** https://developer.android.com/reference/kotlin/androidx/compose/material3/MotionScheme
- Animation specs (see §9) are values, so a theme can carry them.

---

## 3. Material 3 dynamic color, HCT and OKLab

### 3.1 material-color-utilities
- The library ships in C++, Dart, Java, Swift, TypeScript and Kotlin. Its
  modules are HCT, quantize, score, scheme, dynamiccolor, contrast, blend and
  palettes. Schemes can be created "from a single color or a core palette".
  Dynamic colors "adjust based on UI state (dark theme, style, preferences,
  contrast requirements, etc.)". **[primary]**
  https://github.com/material-foundation/material-color-utilities
- HCT uses "CAM16 hue and chroma, and L* from L*a*b*". "A difference of 40 in
  HCT tone guarantees a contrast ratio >= 3.0, and a difference of 50
  guarantees a contrast ratio >= 4.5." **[primary]**
  https://github.com/material-foundation/material-color-utilities/blob/main/typescript/hct/hct.ts
- Variants (`enum Variant`): MONOCHROME, NEUTRAL, TONAL_SPOT, VIBRANT,
  EXPRESSIVE, FIDELITY, CONTENT, RAINBOW, FRUIT_SALAD, CMF. **[primary]**
  https://github.com/material-foundation/material-color-utilities/blob/main/typescript/dynamiccolor/variant.ts
  - TonalSpot: "low to medium colorfulness and a Tertiary TonalPalette with a
    hue related to the source color. The default Material You theme on
    Android 12 and 13."
  - Vibrant: "maxes out colorfulness at each position in the Primary Tonal
    Palette."
  - Expressive: "intentionally detached from the source color."
  - Fidelity and Content: the source color goes in `primaryContainer`,
    "adjusted for color relativity", and the tertiary container is its
    complement.
  - Monochrome: "grayscale". Neutral: "near grayscale".
  **[primary]** https://github.com/material-foundation/material-color-utilities/tree/main/typescript/scheme
- `DynamicScheme(sourceColorHct(s), variant, contrastLevel, isDark, platform,
  specVersion)`. `contrastLevel` runs "from -1 to 1. -1 represents minimum
  contrast, 0 represents standard …, and 1 represents maximum contrast". The
  scheme holds primary, secondary, tertiary, neutral and neutralVariant tonal
  palettes. **[primary]**
  https://github.com/material-foundation/material-color-utilities/blob/main/typescript/dynamiccolor/dynamic_scheme.ts
- How a seed becomes a scheme. Each role is a `DynamicColor` defined as a
  palette, a tone function, an optional `background` role, a `contrastCurve`
  and a `toneDeltaPair`. A `ContrastCurve` holds four values for contrast
  levels -1, 0, 0.5 and 1, and gives the role's required contrast against its
  background. Roles are constraints solved per scheme, not stored hex values.
  **[primary]**
  https://github.com/material-foundation/material-color-utilities/blob/main/typescript/dynamiccolor/dynamic_color.ts
  and `contrast_curve.ts` in the same directory.
- Compose guidance: each key color "relate[s] to a tonal palette of 13 tones".
  Pair `on-X` with `X` for accessible contrast. **[primary]**
  https://developer.android.com/develop/ui/compose/designsystems/material3
  (The m3.material.io page is JavaScript-rendered and could not be read.)

### 3.2 OKLab and OKLCH
- Björn Ottosson's OKLab (23 Dec 2020) is "designed to be simple to use, while
  doing a good job at predicting perceived lightness, chroma and hue". L, C and
  h "should be perceived as orthogonal". CIELAB predicts blue hues badly. LCh
  is its polar form. The 2025 intro notes adoption in CSS Color 4/5,
  Photoshop, Unity and Godot. **[primary for the definition; author's site]**
  https://bottosson.github.io/posts/oklab/
- CSS Color 4 defines `oklab()` and `oklch()`, and shows OkLCh lightness and
  hue "correctly reflecting" perceived differences where HSL fails.
  **[primary]** https://www.w3.org/TR/css-color-4/#ok-lab
- libadwaita derives standalone accent colors in OKLab: `min(l, 0.5) a b` in
  light and `max(l, 0.85) a b` in dark (see §4). **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html
- The DTCG color module accepts `oklab` and `oklch` color spaces (see §7).

---

## 4. GTK 4 and libadwaita

### 4.1 CSS nodes and selectors
- Each widget has CSS nodes with a name, state and style classes. Example:
  `GtkScale` has `scale` > `marks.top|bottom` > `mark`, and `trough` >
  `slider`, `highlight`, `fill`. Selectors support type/class/ID,
  `:hover :active :focus :disabled :checked :selected :backdrop`, `:dir(rtl)`,
  `:not()` and structural pseudo-classes. GTK differs from the web here:
  `:focus-within` and `:focus-visible` are set on all ancestors of the focus
  widget. **[primary]** https://docs.gtk.org/gtk4/css-overview.html
- Provider priorities: `GTK_STYLE_PROVIDER_PRIORITY_USER` (800) is for
  `$XDG_CONFIG_HOME/gtk-4.0/gtk.css`. "You should not use priorities higher
  than this, to give the user the last word." **[primary]**
  https://docs.gtk.org/gtk4/const.STYLE_PROVIDER_PRIORITY_USER.html

### 4.2 CSS properties, variables and media queries
- "Starting with 4.16, GTK supports custom properties" (`--x` and `var(--x,
  fallback)`). `@define-color`, `shade()`, `alpha()` and `mix()` are
  deprecated in favor of standard CSS. Standard color syntax is supported:
  `oklab()`, `oklch()`, `color-mix()`, relative colors and `calc()`.
  `box-shadow`, per-corner `border-radius`, `filter`, `transform`,
  `transition-*` and `animation-*` are supported. `backdrop-filter` is not
  listed. **[primary]** https://docs.gtk.org/gtk4/css-properties.html
- "Since GTK 4.20 CSS Media Queries are supported":
  `prefers-color-scheme`, `prefers-contrast` (`more`/`less`) and
  `prefers-reduced-motion`. **[primary]** (same URL)

### 4.3 libadwaita variables, classes and style manager
- CSS variables come in bg/fg pairs plus a standalone color: `--accent-bg-color`,
  `--accent-fg-color` and `--accent-color`, and the same for destructive,
  success, warning and error. There are surface groups for window, view,
  headerbar, sidebar, card, dialog, popover, thumbnail and overview. Helpers
  `--border-opacity`, `--dim-opacity`, `--disabled-opacity`, `--border-color`
  (from `currentColor`) and `--window-radius` are "not meant to be
  overridden". The GNOME palette is exposed as `--blue-1…5` and so on. Apps
  override with `:root { --accent-bg-color: #e01b24; }`. **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html
- Standalone colors "are automatically derived from the background color". If
  you override a background for one widget, you must also override its
  standalone color. **[primary]** (same URL)
- Style classes: `.suggested-action`, `.destructive-action`, `.flat`, `.raised`,
  `.circular`, `.pill`, `.card`, `.boxed-list`, `.linked`, `.toolbar`, `.osd`,
  `.accent/.success/.warning/.error`, `.title-1…4`, `.heading`, `.caption`,
  `.monospace` and `.numeric`. `.opaque` is deprecated since 1.6 (use
  `.suggested-action` with overridden accent). `.dim-label` became `.dimmed`
  in 1.7. **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/style-classes.html
- Apps load `style.css`, `style-dark.css`, `style-hc.css` and
  `style-hc-dark.css` resources, or use media queries. `AdwStyleManager:
  color-scheme` takes PREFER_DARK, FORCE_DARK or FORCE_LIGHT. High contrast:
  "Libadwaita applications automatically support it; applications cannot
  disable it." To override the accent, set `--accent-bg-color` on `:root`, and
  for a single widget also `--accent-color`. "When accent color is overridden,
  AdwStyleManager API will still return the system color." **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/styles-and-appearance.html
- `AdwStyleManager` has `accent-color`, `accent-color-rgba` and
  `system-supports-accent-colors` (since 1.6), and `document-font-name` and
  `monospace-font-name` (since 1.7). **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.StyleManager.html
- `AdwAccentColor` (since 1.6) is a fixed set of nine system accents (blue
  `#3584e4` by default, plus teal, green, yellow, orange, red, pink, purple and
  slate) with `to_standalone_rgba`. **[primary]**
  https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/enum.AccentColor.html
- The freedesktop settings portal carries the system preferences:
  `org.freedesktop.appearance` keys `color-scheme` (0 none, 1 dark, 2 light),
  `accent-color` (sRGB `(ddd)` in [0,1]), `contrast` (0 normal, 1 high) and
  `reduced-motion` (0 none, 1 reduce). **[primary]**
  https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html

### 4.4 libadwaita's stance on themes
- libadwaita 1.0 (31 Dec 2021) exposed its colors ("31 as of the moment of
  writing") as named colors that "will be treated as a proper API", so apps
  could recolor at runtime instead of forking SCSS. **[secondary: maintainer
  blog]** https://blogs.gnome.org/alicem/2021/12/31/libadwaita-1-0/
- Maintainers say third-party themes still technically work (`GTK_THEME`,
  `~/.config/gtk-4.0/gtk.css`) but are not supported. The supported knobs are
  recoloring, accent, dark and high contrast. **[secondary: GNOME Discourse
  and blogs]** https://discourse.gnome.org/t/why-and-how-libadwaita-prevents-theming/17079 ,
  https://discourse.gnome.org/t/why-gtk-theme-env-breaks-adwaita-applications/16016 ,
  https://blogs.gnome.org/alatiera/2021/09/18/the-truth-they-are-not-telling-you-about-themes/

---

## 5. Qt

### 5.1 QStyle, QProxyStyle and QStyleOption
- `QStyle` is an "abstract base class that encapsulates the look and feel of a
  GUI". Drawing is split into primitive elements (`drawPrimitive`), control
  elements (`drawControl`), complex controls (`drawComplexControl`) and their
  sub-controls (`SC_…`). Metrics come from `pixelMetric` and behavior from
  `styleHint`. Widgets pass a `QStyleOption` with geometry, palette and state
  flags (`State_Enabled`, `State_MouseOver`, `State_Sunken`, `State_HasFocus`,
  `State_On`, `State_Horizontal`…). **[primary]**
  https://doc.qt.io/qt-6/qstyle.html
- `QProxyStyle` wraps a base style to override selected elements, for example
  one `styleHint`. Caveat: "no guarantee that QStyle::proxy() will be called
  for user defined or system controlled styles." **[primary]**
  https://doc.qt.io/qt-6/qproxystyle.html

### 5.2 QPalette
- There are three color groups (Active, Inactive, Disabled) and roles: Window,
  WindowText, Base, AlternateBase, ToolTipBase/Text, PlaceholderText, Text,
  Button, ButtonText, BrightText, Light/Midlight/Mid/Dark/Shadow (derived from
  Window), Highlight, HighlightedText, Link, LinkVisited and Accent. Accent was
  added in 6.6 and defaults to Highlight. "Some styles do not use the palette
  for all drawing." **[primary]** https://doc.qt.io/qt-6/qpalette.html

### 5.3 Qt Style Sheets
- The syntax is "almost identical to those of HTML CSS". Selectors include
  type, `.Class` (exact class), property (`[flat="false"]`), `#objectName`,
  descendant and child. `::sub-control` and `:pseudo-state` support negation
  (`:!hover`), AND-chaining (`:hover:checked`) and comma OR. Specificity
  follows CSS2, but "all Type Selectors have the same specificity". A widget's
  own sheet beats ancestor and application sheets "irrespective of the
  specificity". There is no `!important`. Font and color are not inherited by
  default; `Qt::AA_UseStyleSheetPropagationInWidgetStyles` enables it.
  **[primary]** https://doc.qt.io/qt-6/stylesheet-syntax.html
- QSlider example: `QSlider::groove:horizontal { border: 1px solid #999999;
  height: 8px; … }`, `QSlider::handle:horizontal { … margin: -2px 0; }`, and
  `QSlider::add-page:vertical` / `::sub-page:vertical` for the two sides of
  the track. **[primary]** https://doc.qt.io/qt-6/stylesheet-examples.html

### 5.4 Qt Quick Controls
- Styles: Basic, Fusion, Imagine, macOS, iOS, Material, Universal, Windows and
  FluentWinUI3. A style is selected at compile time (import) or at run time
  (`QQuickStyle::setStyle` > `-style` > `QT_QUICK_CONTROLS_STYLE` >
  `qtquickcontrols2.conf`). There is a run-time fallback style. **[primary]**
  https://doc.qt.io/qt-6/qtquickcontrols-styles.html
- A control is a tree of replaceable delegates (`background`, `contentItem`,
  `handle`, `indicator`). The Slider example replaces `handle` with a
  `Rectangle` positioned by `control.visualPosition` and colored by
  `control.pressed`. A custom style is a directory of QML files whose roots
  come from `QtQuick.Templates`; missing controls "fall back" to Basic. The
  macOS and Windows styles "are not suitable for customizing". **[primary]**
  https://doc.qt.io/qt-6/qtquickcontrols-customize.html
- **Qt Labs StyleKit** (technology preview, Qt 6.11) is the closest prior art
  to what BarrierKit wants. **[primary]**
  https://doc.qt.io/qt-6.12/qtlabsstylekit-overview-features.html ,
  https://doc.qt.io/qt-6.11/qtlabsstylekit-index.html ,
  https://doc.qt.io/qt-6.12/qml-qt-labs-stylekit-style.html
  - A `Style` object "describes the visual appearance of all Qt Quick
    Controls". It has one group per control (`button`, `slider`, `checkBox`)
    and per-part properties (`background`, `handle`, `indicator`, `text`) with
    `color`, `radius`, `border`, `shadow` and `gradient`.
  - Fallback chain: specific control > `abstractButton` > `control` >
    `Style.fallbackStyle` (a full Basic-like style).
  - State-prefixed properties: `hovered.background.color: "lightskyblue"`.
    Nested states win: "hovered.checked … takes precedence over both hovered
    and checked."
  - `transition` on a control animates state changes (`StyleAnimation`,
    `ColorAnimation`).
  - `light` and `dark` `Theme`s plus `CustomTheme { name; theme }`, selected
    by `Style.themeName`. Unset theme properties fall back to the style.
  - Type variations restyle all controls of a type inside a container (for
    example, buttons inside any `frame`). Instance variations are named and
    attached with `StyleVariation.variations`, and apply to that control and
    its descendants. `variations: []` opts out.
  - `QStyleKitStyle` lets Qt Widgets load the same Style file.
    https://doc.qt.io/qt-6/qstylekitstyle.html

---

## 6. Web platform (CSS)

- **Cascade (CSS Cascade 5, CR 2022-01-13).** The criteria apply in this
  order: origin and importance; context; element-attached styles; layers;
  specificity; order of appearance. Within `@layer`, a layer declared later
  wins for normal declarations, and unlayered styles form an implicit final
  layer that beats all layers. For `!important` the order reverses.
  **[primary]** https://www.w3.org/TR/css-cascade-5/
- **Custom properties.** They are inherited ("Inherited: yes"). `var(--x,
  fallback)` takes a fallback, which may itself use `var()`. The initial value
  is the guaranteed-invalid value, and substituting it makes the property
  "invalid at computed-value time". Reference cycles invalidate every property
  in the cycle. **[primary]** https://www.w3.org/TR/css-variables-1/
- `@property` registers a typed custom property. `syntax` and `inherits` are
  required, and `initial-value` is required unless the syntax is `*`. A typed
  property can be animated (the MDN example animates a `<percentage>`).
  **[primary]** https://developer.mozilla.org/en-US/docs/Web/CSS/@property
- **States.** `:focus-visible` matches "only if the user needs to be informed
  where the focus currently is", unlike `:focus`. **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/:focus-visible
- **User preference media features (Media Queries 5).**
  `prefers-reduced-transparency: reduce` is "to minimize the amount of
  transparent or translucent layer effects" (not Baseline yet). **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-transparency
  See also `prefers-reduced-motion` and `prefers-contrast`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion ,
  https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-contrast
- **Forced colors.** Under `forced-colors: active`, author `color`,
  `background-color`, `border-color`, `outline-color` and others are replaced
  by system colors (`Canvas`, `CanvasText`, `ButtonText`…). `box-shadow` and
  `text-shadow` are forced to `none`, and non-url `background-image` to
  `none`. `forced-color-adjust: none` opts out. This is a user override that
  sits above the author's theme. **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors
- **Backdrop blur.** `backdrop-filter` applies "effects such as blurring …
  to the area behind an element", which needs a (semi-)transparent background.
  It has been Baseline since September 2024. **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/backdrop-filter
- **Corners.** `border-*-radius` sets one quarter ellipse per corner, and
  overlapping curves are scaled down proportionally. CSS Borders 4 (WD
  2026-10-15) adds `corner-*-shape`. **[primary]**
  https://drafts.csswg.org/css-borders-4/
  `corner-shape` keywords map to `superellipse(K)`: square = ∞, squircle = 2,
  round = 1 (initial), bevel = 0, scoop = −1, notch = −∞. It has "no effect"
  if the radius is 0. "Borders, outlines, shadows, and background effects …
  will follow the defined corner shape", including `backdrop-filter`. It is
  experimental and not Baseline. **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/corner-shape
- **Logical properties.** "Direction-relative equivalents" of physical
  properties use block/inline and start/end. Examples: `margin-inline-start`
  and `border-start-start-radius`. The physical side depends on `writing-mode`
  and `direction`. **[primary]**
  https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_logical_properties_and_values

---

## 7. W3C Design Tokens Community Group (DTCG)

- **Status.** The Format Module 2025.10 and the Color Module 2025.10 are Final
  Community Group Reports dated 28 Oct 2025; the color module says it "is
  considered stable". A 2026 draft exists with a "do not implement" warning.
  **[primary]** https://www.designtokens.org/tr/2025.10/format/ ,
  https://www.designtokens.org/tr/2025.10/color/ ,
  https://www.designtokens.org/tr/drafts/format/
- **Files.** Token files are JSON with media type
  `application/design-tokens+json` and extensions `.tokens` or `.tokens.json`.
  **[primary]** (format)
- **Tokens.** An object with `$value` is a token. Optional fields are `$type`
  (inherited from the closest group, never guessed from the value),
  `$description`, `$extensions` (tools must preserve unknown data) and
  `$deprecated`. Names cannot contain `{`, `}` or `.`. **[primary]** (format)
- **Groups.** A group is an object without `$value`. `$root` gives a group a
  base token. `$extends` inherits another group by deep merge, "where local
  properties override inherited properties at the same path". Circular
  extension is invalid. **[primary]** (drafts/format; also in 2025.10)
- **References.** `{group.token}` "always resolves to the $value property of
  the target token". JSON Pointer `$ref`
  (`#/colors/blue/$value/components/0`) reaches sub-values. Alias chains are
  allowed and cycles are errors. **[primary]** (format)
- **Types.** `color` takes `colorSpace` (srgb, srgb-linear, hsl, hwb, lab,
  lch, oklab, oklch, display-p3, a98-rgb, prophoto-rgb, rec2020, xyz-d65,
  xyz-d50), `components` (numbers or `none`), `alpha` and a `hex` fallback.
  `dimension` is `{value, unit: px|rem}`. Others: `fontFamily`, `fontWeight`
  (1–1000 or names), `duration` `{value, unit: ms|s}`, `cubicBezier` (4
  numbers, x in [0,1]) and `number`. Composites: `strokeStyle`, `border`
  `{color,width,style}`, `transition` `{duration, delay, timingFunction}`,
  `shadow` (one or an array of `{color, offsetX, offsetY, blur, spread,
  inset}`), `gradient` (stops `{color, position∈[0,1]}`) and `typography`
  `{fontFamily, fontSize, fontWeight, letterSpacing, lineHeight (multiplier)}`.
  **[primary]** https://www.designtokens.org/tr/2025.10/format/ ,
  https://www.designtokens.org/tr/2025.10/color/
- **Resolver Module (themes and modes).** It addresses the "combinatorial
  explosion" of light/dark × size × brand × a11y with three parts:
  - **Sets**: sources merged in array order.
  - **Modifiers**: "similar to a set, but allows for conditional inclusion via
    the contexts map", with a `default`.
  - **`resolutionOrder`**: "tokens later in the array overriding any tokens
    that came before them".

  An input picks one context per modifier, for example `{"theme": "light",
  "size": "large"}`. Resolution runs validation, ordering, alias resolution
  and output. **[primary, draft]**
  https://www.designtokens.org/tr/drafts/resolver/
- **Style Dictionary.** It deep-merges the `include` and `source` globs, and
  "The tokens found using the 'source' attribute will overwrite tokens found
  using include". This is the base + override pattern. It then runs
  transforms, resolves aliases such as `{size.font.base}`, and emits one
  format per platform. **[primary]**
  https://styledictionary.com/reference/config/ ,
  https://styledictionary.com/info/architecture/
- **Tokens Studio.** "A Theme is a combination of Token Sets", and "Multiple
  Themes can be applied at the same time to create a matrix", which they call
  multi-dimensional theming. **[primary, vendor docs]**
  https://docs.tokens.studio/manage-themes/themes-overview

---

## 8. AwesomeWM `beautiful` and `gears.shape`

- `beautiful` is a "Key+value based theme library". `beautiful.init(path|table)`
  loads a theme table and `beautiful.get()` returns it. Variables include
  `font`, `bg_normal`, `bg_focus`, `fg_*` and `wallpaper`. Helpers:
  `beautiful.xresources.apply_dpi` and `beautiful.gtk.get_theme_variables` (a
  GTK theme bridge). **[primary]**
  https://awesomewm.org/apidoc/theme_related_libraries/beautiful.html
- `wibox.widget.slider` properties `bar_shape`, `bar_height`, `bar_color`,
  `bar_active_color`, `bar_margins`, `bar_border_*`, `handle_shape`,
  `handle_color`, `handle_width`, `handle_margins`, `handle_border_*` and
  `handle_cursor` each fall back to a `beautiful.slider_<name>` variable when
  nil ("Fallback to the current value of `beautiful.slider_handle_color`").
  The model is two levels, theme variable < instance property, with no subtree
  scope. **[primary]** https://awesomewm.org/apidoc/widgets/wibox.widget.slider.html
- `gears.shape` functions take `(cr, width, height, …)` and only build a
  cairo path; the caller then fills, clips or strokes it. Shapes include
  `rounded_rect`, `partially_rounded_rect`, `rounded_bar`, `circle`,
  `squircle`, `losange`, `octogon`, `arrow`, `powerline`, `star`, `hexagon`
  and `pie`. Extra arguments are bound with a closure, and
  `gears.shape.transform` composes transforms. A shape is a function value
  stored in the theme. **[primary]**
  https://awesomewm.org/apidoc/theme_related_libraries/gears.shape.html

---

## 9. Animation customization

- **Flutter.** A `Curve` "must map t=0.0 to 0.0 and t=1.0 to 1.0". `flipped`
  gives the reversed inversion. `Curves` has presets, for example
  `easeInOutCubicEmphasized`, a `ThreePointCubic`. **[primary]**
  https://api.flutter.dev/flutter/animation/Curve-class.html ,
  https://api.flutter.dev/flutter/animation/Curves-class.html
  `AnimationStyle` lets a widget or app replace curves and durations or turn
  them off (§1.3).
- **Compose.** `AnimationSpec` is a value: `spring(dampingRatio, stiffness)`
  (the default for `animate*AsState`; it "guarantees the continuity of
  velocity when target value changes"), `tween(durationMillis, delayMillis,
  easing)`, `keyframes`, `keyframesWithSpline`, `repeatable`,
  `infiniteRepeatable` and `snap`. A custom easing is `Easing { fraction -> … }`,
  and `CubicBezierEasing` is built in. Custom types animate through a
  `TwoWayConverter` to an `AnimationVector`. **[primary]**
  https://developer.android.com/develop/ui/compose/animation/customize
  Theme-level specs come from `MotionScheme` (§2.7).
- **CSS.** CSS Easing 2 (FPWD 2024-08-29) defines `linear()` with many points
  ("A typical use of linear() is to provide many points to create the
  illusion of a curve", for example a bounce), `cubic-bezier()` with x in
  [0,1], and `steps()`. **[primary]** https://www.w3.org/TR/css-easing-2/
- **GTK and Qt StyleKit.** GTK CSS supports `transition-*` and `animation-*`
  (§4.2). StyleKit attaches a `transition` to a control style (§5.4).
- **Turning motion off.** The platform signals are Flutter
  `MediaQueryData.disableAnimations`, CSS and GTK `prefers-reduced-motion`,
  and the portal key `reduced-motion` (§1.7, §4.2, §4.3). WCAG technique C39
  uses `prefers-reduced-motion` (§10).

---

## 10. Accessibility constraints a theme system must keep

- **1.4.3 Contrast (Minimum), AA.** Text needs 4.5:1, or 3:1 for large text.
  Text in inactive components and logotypes is exempt. **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
- **1.4.11 Non-text Contrast, AA.** Components need 3:1 against adjacent colors
  for the "visual information required to identify user interface components
  and states", including focus indicators. Inactive components are exempt.
  2.999:1 fails. **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html
- **2.4.7 Focus Visible, AA.** A focus indicator must exist. **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html
- **2.4.11 Focus Not Obscured (Minimum), AA.** The focused component "is not
  entirely hidden due to author-created content". **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html
- **2.4.13 Focus Appearance, AAA.** The indicator must be at least the area of
  a 2 CSS px perimeter and have 3:1 between the same pixels focused and
  unfocused. **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance.html
- **2.3.3 Animation from Interactions, AAA.** "Motion animation triggered by
  interaction can be disabled" unless essential. Technique C39 uses
  `prefers-reduced-motion`. **[primary]**
  https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html
- **How toolkits enforce or clamp themes:**
  - libadwaita: apps "cannot disable" high contrast. Helper opacities are not
    meant to be overridden. Standalone accent colors are derived and clamped in
    OKLab (L ≤ 0.5 light, L ≥ 0.85 dark). (§4.3) **[primary]**
  - GTK: the user provider priority (800) is the ceiling: "give the user the
    last word". (§4.1) **[primary]**
  - CSS forced colors: the browser replaces author colors at paint time and
    removes shadows. (§6) **[primary]**
  - Material dynamic color: roles carry a `contrastCurve` against a declared
    background, so contrast is solved, not hoped for. A tone delta of 50
    guarantees ≥ 4.5:1. (§3.1) **[primary]**
  - Flutter: dedicated `highContrastTheme` slots (§1.1). **[primary]**
  - Apple: `NSWorkspace.accessibilityDisplayShouldReduceTransparency` exists
    for Reduce Transparency. The page is JavaScript-rendered and its text was
    not verified.
    https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducetransparency

---

## Lessons for BarrierKit

Current context, checked in the repo: Tengen JSON
(`crates/ctdesign/tengen/{tokens,components}.json`) holds themes plus
per-component metrics with aliases such as `"radius": "radius.button"` and
variants (`button.large/small/menu`). `build.rs` emits typed constants.
`Palette::current()` is already contextual (a `Backdrop` provides it). The
`Resolved` appearance carries `reduced_motion`, `reduce_transparency` and
`contrast`, and the design system has `contrast_ratio` with
`ACCENT_MIN_CONTRAST_*`.

1. **One precedence order, written down and enforced.** Use built-in default <
   theme (Tengen) < theme variant (dark/hc) < subtree override < instance
   override < accessibility clamps. User preferences sit on top, as with GTK's
   USER priority, CSS forced colors and libadwaita's mandatory high contrast.
   Resolve per field, as Flutter does with `style` > `themeStyleOf` >
   `defaultStyleOf`, not per whole object.
2. **Resolve properties per state.** Model control properties as
   `StateValue<T>`: a base value plus optional overrides for `hovered`,
   `pressed`, `focused`, `disabled` and `selected`. Combined states (for
   example `hovered.checked`) beat single states, as in StyleKit. The
   resolver takes a `State` bitset, like Flutter's `WidgetStateProperty` and
   Qt's `State_*` flags. Keep the "no override for this state, use the
   default" path explicit. Compose warns against overloading `null` for this;
   use a named `Default`/`Inherit` variant.
3. **One `XxxDefaults` per control, generated from Tengen.** A typed struct
   (`SliderStyle`, `ButtonStyle`) holds every visual knob: colors per state,
   sizes, radii, shadow and shape. Its public `default()` constructor reads the
   current theme, so wrappers can reuse it (Compose `ComponentDefaults`).
   Generate the struct and its defaults from `components.json` so the JSON
   remains the single source.
4. **Use an inherited context with a narrow scope.** Extend the
   `Palette::current()` mechanism into a `Theme::current()` context that
   carries palette, metrics and motion, overridable for a subtree. This is
   Flutter `Theme`/`SliderTheme`, Compose `CompositionLocalProvider` and
   StyleKit type variations. Follow Compose's rules: every context value has a
   good default, and only tree-scoped concepts go in context. Read context in
   default expressions so explicit parameters always win.
5. **Make shapes typed values, not just radii.** Add a `Shape` trait (or
   closed enum) with `outer_path(rect, direction)`, `inner_path`,
   `insets()` and `lerp(&other, t)`. Ship `Rect`, `RoundedRect(per-corner)`,
   `Continuous`/superellipse (already `continuous_corner`), `Capsule` and
   `Circle`. Shadows, borders, clips and backdrop blur must all use the same
   path, as CSS `corner-shape` requires. AwesomeWM `gears.shape`, Flutter
   `ShapeBorder` and CSS `superellipse(K)` show the range. Mapping K
   (∞ square, 2 squircle, 1 round, 0 bevel, −1 scoop) gives a single numeric
   token for corner style.
6. **Part painters for complex controls.** For Slider, Switch and Stepper,
   expose replaceable parts (`track`, `thumb`, `tick`, `value_indicator`):
   Flutter `*Shape` classes, Compose `thumb`/`track` slots, Qt `::groove`
   `::handle` `::add-page` `::sub-page` and Qt Quick `background`/`handle`.
   Each part painter receives the resolved style, state, the animation
   progress of activation and enable (Flutter passes `activationAnimation` and
   `enableAnimation`), the value and the text direction.
7. **Theme extensions with `lerp`.** Let apps add their own token groups
   (`impl ThemeExtension for MyTokens { fn lerp(&self, other, t) }`, keyed by
   `TypeId`). Require `lerp` on every theme struct, including `Shape`,
   `Shadow` and `Color` in OKLab, so light/dark and accent changes can animate
   as Flutter's `AnimatedTheme` does. Honor `reduced_motion` by snapping, like
   `AnimationStyle.noAnimation`.
8. **Aliases as first-class data, resolved at build time.** Align Tengen with
   DTCG: `{group.token}` references, `$type` inherited from groups,
   `$extends` for group inheritance, `$extensions` for kit-specific data,
   composite `shadow`/`border`/`transition`/`typography`, and colors as
   `{colorSpace: "oklch", components, alpha}`. Detect cycles in `build.rs`.
   This keeps the existing `"radius.button"` alias pattern but makes it
   standard and lintable.
9. **Build alternative themes as resolver modifiers, not copied files.** Keep
   one base set plus modifiers (`mode: light|dark`,
   `contrast: normal|high`, `density: regular|compact`, `brand: …`) and an
   ordered `resolutionOrder` where later wins. This follows the DTCG Resolver,
   Style Dictionary `include` < `source` and Tokens Studio theme matrices. A
   complete new theme is then a small override file, not a fork. Generate one
   Rust constant set per combination at build time, or resolve at run time
   from a typed table.
10. **Derive color roles with contrast constraints.** Define roles relative to
    a background with a minimum ratio (Material `DynamicColor` with
    `background` and `contrastCurve`), parameterized by a `contrast_level`
    from −1 to 1 that the portal's `contrast` key maps onto. Clamp derived
    accents in OKLab L as libadwaita does for standalone colors. Generalize
    the existing `ACCENT_MIN_CONTRAST_*` and `contrast_ratio` checks into
    build-time assertions over every fg/bg pair: 4.5:1 for text, 3:1 for
    non-text and focus rings.
11. **Pair roles as bg/fg/standalone.** Follow libadwaita
    (`--accent-bg-color`, `--accent-fg-color`, `--accent-color`) and Material
    `X`/`on-X`. A per-instance accent override then changes the whole trio
    consistently. libadwaita warns that overriding only the background breaks
    the standalone color.
12. **Motion as theme tokens.** Store `Motion { spatial: {fast, default,
    slow}, effects: {fast, default, slow} }` of a typed `AnimationSpec`
    (`Spring{damping, stiffness}`, `Tween{duration, easing}`, `Linear(points)`
    for CSS-`linear()`-style curves). This is Compose `MotionScheme`. The
    distinction matters: spatial specs can be springs, while effects specs
    (color and opacity) must not overshoot. Controls ask for a role, never a
    literal duration. `reduced_motion` replaces spatial specs with
    `snap`/crossfade while keeping effects short (WCAG 2.3.3).
13. **Translucency and blur as overridable tokens with a solid fallback.** Each
    material (sidebar, popover, menu, HUD) carries `{tint, opacity,
    blur_radius, saturation, solid_fallback}`. `reduce_transparency` and
    high contrast force `solid_fallback`, matching CSS
    `prefers-reduced-transparency` and forced colors removing shadows. Group
    backdrops that share a source, as Flutter's `BackdropGroup` merges blurs
    for cost.
14. **Layout direction and orientation as inherited context.** Use
    `start`/`end` in insets, per-corner radii (`start_start`…) and part
    order. Resolve to physical sides from a `LayoutDirection` context, as CSS
    logical properties, Compose `padding(start)` versus `absolutePadding`, and
    GTK `:dir(rtl)` do. Orientation (horizontal/vertical slider) is a
    property that part painters receive, not a separate theme.
15. **Prefer named variants to grab-bag style objects at the call site.** The
    public surface stays SwiftUI-like (`.button_style(.bordered)`, `.tint()`,
    `.control_size(.small)`), mapping onto the Tengen variants that already
    exist (`button.large/small`). This follows Compose's "prefer multiple
    components over style classes" and libadwaita's style classes. The full
    `XxxStyle` struct stays available for the rare deep override, as
    StyleKit's instance variations.
