# How SwiftUI and AppKit customize control appearance

**Goal:** show, from Apple's own documentation, how styles work at system, container and instance level, with a source for every claim.
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

Research for BarrierKit. Question: how do Apple's frameworks let an app change
the look of controls at three levels, so that BarrierKit can copy the design?
The three levels are:

- **System or theme:** appearance, accent, contrast and accessibility.
- **Container:** settings that descendants inherit.
- **Single instance:** one view.

Sources: Apple documentation pages (read through their DocC JSON at
`https://developer.apple.com/tutorials/data/documentation/<fw>/<path>.json`),
Human Interface Guidelines pages, WWDC session transcripts, and Swift Evolution.
Fetched 2026-11-07. Every URL below is a primary source. This file uses no blog
or forum sources.

Abbreviations: `SUI` = `https://developer.apple.com/documentation/swiftui/`,
`AK` = `https://developer.apple.com/documentation/appkit/`,
`HIG` = `https://developer.apple.com/design/human-interface-guidelines/`,
`WWDC` = `https://developer.apple.com/videos/play/`.

---

## 0. The model in one paragraph

SwiftUI splits a control into three parts:

- **Semantics and state:** a `Button` has an action, a role and a label. A
  `Toggle` has an `isOn` binding.
- **A style:** a value that turns a *configuration* (the state and the label
  view) into a *body*.
- **An environment:** a typed key-value map that flows from parent to child.

A modifier like `.buttonStyle(.bordered)` does not change the button. It writes
the style into the environment of a subtree. Every button below that point reads
the style and calls `makeBody(configuration:)`. Colours, sizes, materials,
animations and accessibility settings travel the same way, as environment values
that the style reads. AppKit reaches a similar result with older tools:

- **`NSAppearance`:** inherited from the app to its windows and then to views.
- **Dynamic colours:** resolved against the current drawing appearance.
- **Enums on controls:** `controlSize`, `bezelStyle` and so on.
- **Subclassing:** `draw(_:)` and `updateLayer()` give full control.

---

## 1. Style protocols (configuration → body)

### 1.1 ButtonStyle / PrimitiveButtonStyle

- `ButtonStyle`: "A type that applies standard interaction behavior and a custom appearance to all buttons within a view hierarchy." The style controls the look only. The system keeps the press, hover and keyboard behaviour.
  SUI`buttonstyle`
- `PrimitiveButtonStyle`: "A type that applies custom interaction behavior and a custom appearance to all buttons within a view hierarchy." Built-in values: `automatic`, `accessoryBar`, `accessoryBarAction`, `bordered`, `borderedProminent`, `borderless`, `card`, `glass`, `glassProminent`, `glass(_:)`, `link`, `plain`.
  SUI`primitivebuttonstyle`
- Requirement: `func makeBody(configuration: Self.Configuration) -> Self.Body`. "The system calls this method for each Button instance in a view hierarchy where this style is the current button style."
  SUI`buttonstyle/makebody(configuration:)`
- `ButtonStyleConfiguration` has three properties:
  - `label`: a type-erased view.
  - `isPressed: Bool`: "whether the user is currently pressing the button".
  - `role: ButtonRole?`: an "optional semantic role that describes the button's purpose". A style is meant to change its appearance based on the role. Apple's own example draws `.cancel` in bold and `.destructive` in red.
  SUI`buttonstyleconfiguration`, SUI`buttonstyleconfiguration/role`, SUI`buttonstyleconfiguration/ispressed`
- `PrimitiveButtonStyleConfiguration` has `label`, `role` and `trigger()`, which "Performs the button's action". A primitive style owns the gesture and calls `trigger()` itself.
  SUI`primitivebuttonstyleconfiguration`, SUI`primitivebuttonstyleconfiguration/trigger()`
- `ButtonRole`: "A value that describes the purpose of a button". Values: `cancel`, `destructive`, `close`, `confirm`.
  SUI`buttonrole`

### 1.2 Other style protocols (same pattern)

| Protocol | Configuration contents | Built-in values | URL |
|---|---|---|---|
| `ToggleStyle` | `label`, `isOn`/`$isOn` (Binding), `isMixed` | `automatic`, `button`, `checkbox`, `switch` | SUI`togglestyle`, SUI`togglestyleconfiguration` |
| `LabelStyle` | `icon`, `title` | `automatic`, `iconOnly`, `titleAndIcon`, `titleOnly` | SUI`labelstyle`, SUI`labelstyleconfiguration` |
| `GaugeStyle` | (configuration) | `automatic`, `circular`, `accessoryCircular`, `accessoryCircularCapacity`, `linear`, `linearCapacity`, `accessoryLinear`, `accessoryLinearCapacity` | SUI`gaugestyle` |
| `ProgressViewStyle` | (configuration) | `automatic`, `circular`, `linear` | SUI`progressviewstyle` |
| `MenuStyle` | (configuration) | `automatic`, `button` | SUI`menustyle` |
| `DatePickerStyle` | `DatePickerStyleConfiguration` | `automatic`, `compact`, `field`, `graphical`, `stepperField`, `wheel` | SUI`datepickerstyle` |
| `PickerStyle` | **no public `makeBody`** (closed set) | `automatic`, `inline`, `menu`, `navigationLink`, `palette`, `radioGroup`, `segmented`, `tabs`, `wheel` | SUI`pickerstyle` |
| `TextFieldStyle` | **no public `makeBody`** (closed set) | `automatic`, `bordered`, `plain` (`roundedBorder`/`squareBorder` deprecated) | SUI`textfieldstyle` |

Some styles are open and some are closed. `PickerStyle` and `TextFieldStyle`
list only "Getting built-in … styles" and "Supporting types" topics. They have
no "Creating custom … styles" section. In other words, Apple exposes a custom
`makeBody` only where it can describe the control as a small configuration.
Pickers and text fields have complex internal state, so they stay closed.
SUI`pickerstyle`, SUI`textfieldstyle`

The "View styles" overview gives the general rule: "SwiftUI … automatically
selects the appropriate style for a particular presentation context … You can
override the automatic style by using one of the style view modifiers. These
modifiers typically propagate throughout a container view … Any of the style
protocols that define a `makeBody(configuration:)` method … also enable you to
define custom styles." The same page lists the other style families:
`ListStyle`, `TableStyle`, `DisclosureGroupStyle`, `ControlGroupStyle`,
`FormStyle`, `GroupBoxStyle`, `LabeledContentStyle`, `TextEditorStyle`,
`TabViewStyle` and `NavigationSplitViewStyle`.
SUI`view-styles`

### 1.3 Label: icon and title order

- `titleAndIcon`: "In most cases, labels show both their title and icon by default. However, some containers might apply a different default label style to their content, such as only showing icons within toolbars … The relative layout of the title and icon is dependent on the context … In most cases … arranged horizontally with the icon leading." SUI`labelstyle/titleandicon`
- `iconOnly`: "The title of the label is still used for non-visual descriptions, such as VoiceOver." SUI`labelstyle/icononly`
- To put the icon after the title, write a custom `LabelStyle` whose `makeBody` returns `HStack { configuration.title; configuration.icon }`. SwiftUI has no built-in "trailing icon" style. SUI`labelstyle/makebody(configuration:)`
- `labelIconToTitleSpacing(_:)` (26.0) sets the icon-to-title spacing. It is also an environment value. SUI`view/labelicontotitlespacing(_:)`
- `labelsVisibility(_:)` hides the labels of `LabeledContent`-based controls in a subtree. "Always provide a label … SwiftUI uses labels for other purposes, including accessibility." SUI`view/labelsvisibility(_:)`

### 1.4 Propagation through the environment

`.buttonStyle(_:)` "Sets the style for buttons within this view". It also
reaches controls that build themselves from a button. In Apple's example, a
`Menu` gets `.menuStyle(.button)` and a `Toggle` gets `.toggleStyle(.button)`.
A `.buttonStyle(.bordered)` higher in the tree then applies to all of them:
"The button style modifier then causes not only the explicit Sign In Button, but
also the menu and toggles with button styling, to render with the bordered
button style."
SUI`view/buttonstyle(_:)-7qx1`

### 1.5 Replacing the body versus adjusting the current style

- **Full replacement:** `makeBody` returns a new view tree built from `configuration.label` and the state. `ChecklistToggleStyle` builds a toggle from a `Button` and an `Image`. "By using a customized Button to compose the toggle's body, SwiftUI automatically provides the behaviors that users expect." SUI`togglestyle/makebody(configuration:)`
- **Decorating the current style:** inside `makeBody`, call the control's configuration initializer, which renders the *inherited* style. Then add modifiers:
  `Toggle(configuration).padding().border(.red)` (SUI`toggle/init(_:)`).
  `Button(configuration).border(Color.red)` for a `PrimitiveButtonStyle` (SUI`button/init(_:)`).
- **Ordering rule:** "Apply the custom style closer to the toggle than the modified style because SwiftUI evaluates style view modifiers in order from outermost to innermost. If you apply the styles in the other order, the red border style doesn't have an effect, because the built-in styles override it completely." SUI`togglestyle/makebody(configuration:)`
- **Leading-dot sugar:** `extension ToggleStyle where Self == ChecklistToggleStyle { static var checklist: … }` makes `.toggleStyle(.checklist)` possible. SUI`togglestyle`. The language feature behind it is SE-0299, "Extending Static Member Lookup in Generic Contexts" (Swift 5.5). Its motivating example is SwiftUI's `toggleStyle`. https://github.com/swiftlang/swift-evolution/blob/main/proposals/0299-extend-generic-static-member-lookup.md
- **Reusable instance modifiers:** the `ViewModifier` protocol, with `body(content:)` and `.modifier(_:)`, is "a reusable modifier that you can apply to any view". It is often wrapped in a `View` extension. SUI`viewmodifier`

---

## 2. Environment

- `EnvironmentValues`: "A collection of environment values propagated through a view hierarchy." The system sets some values from the device and user settings, such as `locale` and `pixelLength`. Others have defaults. "The value that you set affects the environment for the view that you modify — including its descendants in the view hierarchy — but only up to the point where you apply a different environment modifier." Apple says to prefer dedicated modifiers such as `.lineLimit(2)` over `.environment(\.lineLimit, 2)`. SUI`environmentvalues`
- `@Environment(\.keyPath)`: "A property wrapper that reads a value from a view's environment". It can read but not set a value. "If the value changes, SwiftUI updates any parts of your view that depend on the value." It can also read `@Observable` objects by type. SUI`environment`. The language feature is SE-0258, Property Wrappers (Swift 5.1). https://github.com/swiftlang/swift-evolution/blob/main/proposals/0258-property-wrappers.md
- `.environment(_:_:)`: "This modifier affects the given view, as well as that view's descendant views. It has no effect outside the view hierarchy on which you call it." "Prefer the dedicated modifier when available, and offer your own when defining custom environment values." SUI`view/environment(_:_:)`
- Custom keys, older form: `EnvironmentKey` with `static let defaultValue`, plus a computed property on `EnvironmentValues` that uses `self[Key.self]`. Apple recommends wrapping it in a convenience modifier. SUI`environmentkey`
- Custom keys, current form: `extension EnvironmentValues { @Entry var myCustomValue: String = "Default value" }`. The same macro creates `Transaction`, `ContainerValues` and `FocusedValues` entries. SUI`entry()`
- **Upward override:** `preferredColorScheme(_:)` "propagates up through the view hierarchy to the enclosing presentation". Setting `\.colorScheme` directly changes only the subtree. SUI`view/preferredcolorscheme(_:)`, SUI`environmentvalues/colorscheme`
- **Monotonic values:** `.disabled(true)` on a container "override[s] any `.disabled(_:)` modifiers you set on views within the container". A child cannot enable itself again. SUI`view/disabled(_:)`

Environment values that matter for styling:

| Value | Type / meaning | URL |
|---|---|---|
| `controlSize` | `ControlSize`; default `.regular`; set by `.controlSize(_:)` | SUI`environmentvalues/controlsize`, SUI`view/controlsize(_:)` |
| `colorScheme` | `.light` / `.dark` | SUI`environmentvalues/colorscheme`, SUI`colorscheme` |
| `colorSchemeContrast` | `.standard` / `.increased`; read-only: "depends entirely on user settings, and you can't change it" | SUI`environmentvalues/colorschemecontrast` |
| `accessibilityReduceMotion` | "UI should avoid large animations, especially those that simulate the third dimension" (read-only) | SUI`environmentvalues/accessibilityreducemotion` |
| `accessibilityReduceTransparency` | "UI (mainly window) backgrounds should not be semi-transparent; they should be opaque" | SUI`environmentvalues/accessibilityreducetransparency` |
| `accessibilityDifferentiateWithoutColor` | "should not convey information using color alone and instead should use shapes or glyphs" | SUI`environmentvalues/accessibilitydifferentiatewithoutcolor` |
| `accessibilityPrefersCrossFadeTransitions` (26.4) | Reduce Motion + Prefer Cross-Fade: "avoid Slide animations and prefer Cross-Fade" | SUI`environmentvalues/accessibilityPrefersCrossFadeTransitions` |
| `isEnabled` | default `true` | SUI`environmentvalues/isenabled` |
| `layoutDirection` | `.leftToRight` / `.rightToLeft` | SUI`environmentvalues/layoutdirection` |
| `backgroundMaterial` | the material under the view; "If you set a material, the standard content styles enable their vibrant rendering modes" | SUI`environmentvalues/backgroundmaterial` |
| `backgroundStyle`, `labelIconToTitleSpacing`, `buttonSizing`, `symbolRenderingMode`, `font`, `dynamicTypeSize`, `legibilityWeight` … | full list under topic groups "Accessibility", "Controls and input", "Display characteristics", "Text styles", "View attributes" | SUI`environmentvalues` |

---

## 3. ShapeStyle (paint as a value)

- `ShapeStyle`: "A color or pattern to use when rendering a shape." A custom style implements `resolve(in environment: EnvironmentValues) -> some ShapeStyle`. Apple's example returns a different blend mode depending on `environment.colorScheme`. You can use a style with `fill`, `stroke`, `foregroundStyle`, `background` and others. Topic groups:
  - System colours.
  - Linear, radial, angular, conic and elliptical gradients.
  - Materials (`ultraThinMaterial` … `ultraThickMaterial`, `bar`).
  - Hierarchical styles (`primary` … `quinary`).
  - Semantic styles (`foreground`, `background`, `selection`, `separator`, `tint`, `placeholder`, `link`, `fill`, `windowBackground`).
  - Modifiers (`blendMode(_:)`, `opacity(_:)`, `shadow(_:)`).
  SUI`shapestyle`, SUI`shapestyle/resolve(in:)`
- **Hierarchy:** `HierarchicalShapeStyle` "maps to one of the numbered content styles": `primary`, `secondary`, `tertiary`, `quaternary` and `quinary`. `.secondary` "maps to the second level of the current foreground style, or … the default foreground style if you haven't set" one. If you set the foreground to red, `.secondary` gives a weaker red. SUI`hierarchicalshapestyle`, SUI`shapestyle/secondary-swift.type.property`
- `foregroundStyle(_:)` accepts any `ShapeStyle`, including gradients, and applies it to text, shapes and template images in a subtree. `foregroundStyle(_:_:)` sets the primary and secondary levels together. SUI`view/foregroundstyle(_:)`, SUI`view/foregroundstyle(_:_:)`
- `tint(_:)`: "override the default accent color for this view. Unlike an app's accent color, which can be overridden by user preference, the tint color is always respected and should be used as a way to provide additional meaning to the control … Some controls adapt to the tint color differently based on their style … in macOS, a button with the bordered style doesn't tint its background, but one with the borderedProminent style does." SUI`view/tint(_:)-23xyq`. `.tint` as a `ShapeStyle` (`TintShapeStyle`): "If no explicit tint is set, the tint is derived from the app's accent color." SUI`shapestyle/tint`, SUI`tintshapestyle`
- `Material`: "A background material type" (`ultraThin`, `thin`, `regular`, `thick`, `ultraThick`, `bar`). "A material isn't a view … The blurring effect provided by the material isn't simple opacity. Instead, it uses a platform-specific blending." SUI`material`
- Gradients: `LinearGradient`, `RadialGradient` and `AngularGradient` ("also known as a 'conic' gradient") map unit-space points into the bounds of the filled shape. `MeshGradient` (iOS 18, macOS 15) is "A two-dimensional gradient defined by a 2D grid of positioned colors" with Bézier control points and optional `smoothsColors`. `Color.gradient` "Returns the standard gradient for the color" (`AnyGradient`). SUI`lineargradient`, SUI`radialgradient`, SUI`angulargradient`, SUI`meshgradient`, SUI`color/gradient`
- `ShapeStyle.opacity(_:)`: "a new style based on `self` that multiplies by the specified opacity". SUI`shapestyle/opacity(_:)-swift.method`
- `ShapeStyle.shadow(_:)` takes a `ShadowStyle`, as in `Rectangle().fill(.red.shadow(.drop(radius: 2, y: 3)))`. SUI`shapestyle/shadow(_:)-swift.method`
- `View.blendMode(_:)`. SUI`view/blendmode(_:)`
- `backgroundStyle(_:)` sets the `backgroundStyle` environment value used by `.background(in: Shape)`. You can reset it with `.environment(\.backgroundStyle, nil)`. SUI`view/backgroundstyle(_:)`
- `background(_:in:fillStyle:)` fills an `InsettableShape` with a style behind the view. SUI`view/background(_:in:fillstyle:)`
- `AnyShapeStyle`: "A type-erased ShapeStyle value". You need it when branches return different style types. SUI`anyshapestyle`

---

## 4. Shapes

- `Shape`: "A 2D shape that you can use when drawing a view." It conforms to `Sendable`, `Animatable` and `View`. Requirement: `path(in:)`, with an optional `sizeThatFits(_:)`. "Shapes without an explicit fill or stroke get a default fill based on the foreground color." Static members include:
  - `.rect`, `.rect(cornerRadius:style:)` and `.rect(topLeadingRadius:…:style:)`.
  - `.capsule`, `.circle`, `.ellipse` and `.containerRelative`.
  - `.buttonBorder` and `.textInputBorder`.
  Shapes also support boolean operations (`union`, `intersection`, `subtracting`…) and a `layoutDirectionBehavior`. SUI`shape`
- `RoundedRectangle(cornerRadius:style:)`, where `RoundedCornerStyle` is `.circular` or `.continuous` ("Continuous curvature rounded rect corners"). SUI`roundedrectangle`, SUI`roundedcornerstyle`, SUI`roundedcornerstyle/continuous`
- `Capsule` is "equivalent to a rounded rectangle where the corner radius is chosen as half the length of the rectangle's smallest edge". SUI`capsule`
- `UnevenRoundedRectangle(topLeadingRadius:bottomLeadingRadius:bottomTrailingRadius:topTrailingRadius:style:)` sets a radius per corner, using leading and trailing (so it follows RTL). SUI`unevenroundedrectangle`
- `ContainerRelativeShape`: "dimensions the system calculates from an inset version of the current container shape". It falls back to a rectangle. SUI`containerrelativeshape`
- New in 26: `containerShape(_:)` sets the container shape for descendants. `ConcentricRectangle` makes corners "squared, rounded, or concentric relative to a container shape's corners … automatically calculates each corner's radius relative to the container shape … without hard-coded values." SUI`view/containershape(_:)`, SUI`concentricrectangle`. The AppKit session calls this "concentricity. Each element is designed with a curvature that sits neatly within the corner radius of its container." WWDC`wwdc2025/310`
- `InsettableShape.strokeBorder(_:lineWidth:antialiased:)` draws the stroke inside the bounds. `inset(by:)`. SUI`insettableshape`
- `clipShape(_:style:)` clips a view. `contentShape(_:eoFill:)` "Defines the content shape for hit testing". SUI`view/clipshape(_:style:)`, SUI`view/contentshape(_:eofill:)`
- `buttonBorderShape(_:)` with `ButtonBorderShape` values `.automatic`, `.capsule`, `.circle`, `.roundedRectangle` and `.roundedRectangle(radius:)`. "The border shape is used to draw the platter for a bordered button … affects buttons of the bordered and borderedProminent styles." This is an environment-propagated setting that the style reads. SUI`buttonbordershape`, SUI`view/buttonbordershape(_:)`
- `border(_:width:)` draws inside the view's bounds. "To place a border around the outside … apply padding of the same width before adding the border." Modifier order matters. SUI`view/border(_:width:)`

---

## 5. Size and layout

- `ControlSize`: `.mini`, `.small`, `.regular`, `.large`, `.extraLarge`. Set it with `.controlSize(_:)` "to override the system default size for controls in this view". SUI`controlsize`, SUI`view/controlsize(_:)`
- `buttonSizing(_:)` (26), with `ButtonSizing` set to `.flexible` and similar: "Many built-in controls that display as a button adapt … Your own views and styles can adapt … by reading the buttonSizing environment value." SUI`view/buttonsizing(_:)`
- `frame(width:height:alignment:)`: "If you only specify one of the dimensions, the resulting view assumes this view's sizing behavior in the other dimension." SUI`view/frame(width:height:alignment:)`
- `padding(_:_:)`: "If you omit the `length`, SwiftUI uses a default amount of padding … appropriate for the platform and the presentation context." "The order in which you apply modifiers matters." SUI`view/padding(_:_:)`
- `layoutPriority(_:)`: the default is 0. "A parent layout offers the child views with the highest layout priority all the space offered to the parent minus the minimum space required for all its lower-priority children." SUI`view/layoutpriority(_:)`
- `HStack(alignment:spacing:content:)`: `spacing: CGFloat? = nil` means system-adaptive spacing. SUI`hstack/init(alignment:spacing:content:)`
- `LayoutDirection`: "SwiftUI horizontally flips the x position of each view within its parent, so layout calculations automatically produce the desired effect for both modes without any changes." You can override it per subtree through `.environment(\.layoutDirection, .rightToLeft)`. SUI`layoutdirection`
- `flipsForRightToLeftLayoutDirection(_:)` mirrors the content itself, for example a directional glyph. SUI`view/flipsforrighttoleftlayoutdirection(_:)`
- `rotationEffect(_:anchor:)` is a render-only transform: "It has no effect on the view's frame." SUI`view/rotationeffect(_:anchor:)`
- `Layout` protocol (iOS 16, macOS 13) for custom containers:
  - Required: `sizeThatFits(proposal:subviews:cache:)` and `placeSubviews(in:proposal:subviews:cache:)`.
  - Optional: `spacing(subviews:cache:)`, `explicitAlignment(of:…)`, `makeCache` and `updateCache`.
  - Conforms to `Animatable`.
  SUI`layout`. Session: "Compose custom layouts with SwiftUI" (the `EqualWidthHStack` example). WWDC`wwdc2022/10056`

---

## 6. Shadows and effects

- `shadow(color:radius:x:y:)`: the default colour is `Color(.sRGBLinear, white: 0, opacity: 0.33)`. "Use a radius of zero to create a sharp shadow." SUI`view/shadow(color:radius:x:y:)`
- `ShadowStyle` has `.drop(color:radius:x:y:)` and `.inner(color:radius:x:y:)`. It is used *inside* a `ShapeStyle`, as in `.fill(.red.shadow(.inner(radius: 4)))`. "Inner shadows draw on top of the source content by blurring, tinting, inverting and offsetting its per-pixel alpha values." SUI`shadowstyle`, SUI`shadowstyle/inner(color:radius:x:y:)`
- `blur(radius:opaque:)`: a Gaussian blur. SUI`view/blur(radius:opaque:)`
- `opacity(_:)`: nested opacities multiply. SUI`view/opacity(_:)`
- `visualEffect(_:)` (iOS 17, macOS 14) applies effects with access to a `GeometryProxy`, without changing layout. SUI`view/visualeffect(_:)`. Session: "Create custom visual effects with SwiftUI". WWDC`wwdc2024/10151`
- **Liquid Glass (26):**
  - `glassEffect(_ glass: Glass = .regular, in shape: some Shape = DefaultGlassEffectShape())`. "SwiftUI uses the regular variant by default along with a Capsule shape." SUI`view/glasseffect(_:in:)`
  - `Glass` is "the configuration of the Liquid Glass material": `.regular`, `.clear` and `.identity`, with `tint(_:)` and `interactive(_:)`. SUI`glass`
  - `GlassEffectContainer(spacing:)` "combines multiple Liquid Glass shapes into a single shape that can morph … improving rendering performance … The higher the spacing, the sooner blending begins." SUI`glasseffectcontainer`
  - `glassEffectID(_:in:)` drives morphing between glass shapes. SUI`view/glasseffectid(_:in:)`
  - Button styles `.glass` and `.glassProminent`. SUI`primitivebuttonstyle`
  - Article: "Applying Liquid Glass to custom views". SUI`applying-liquid-glass-to-custom-views`
- AppKit glass (26): `NSGlassEffectView` ("embeds its content view in a dynamic glass effect") with `contentView`, `cornerRadius`, `style`, `tintColor` and `effectIsInteractive`. `NSGlassEffectContainerView` with `spacing` "merges descendant glass effect views". AK`nsglasseffectview`, AK`nsglasseffectcontainerview`. The session says to put content *inside* `contentView`: "avoid placing the NSGlassEffectView behind your content as a sibling view." WWDC`wwdc2025/310`
- `NSVisualEffectView`, the classic material on AppKit:
  - `material` (`NSVisualEffectView.Material`): `.titlebar`, `.selection`, `.menu`, `.popover`, `.sidebar`, `.headerView`, `.sheet`, `.windowBackground`, `.hudWindow`, `.fullScreenUI`, `.toolTip`, `.contentBackground`, `.underWindowBackground`, `.underPageBackground`. The colour-named ones (`.light`, `.dark`…) are deprecated.
  - `blendingMode`: `.behindWindow` or `.withinWindow`.
  - `state`: `.followsWindowActiveState`, `.active` or `.inactive`.
  - Also `isEmphasized` and `maskImage`.
  - "always pick a material based on its intended use … Don't select materials based on the apparent colors."
  - Vibrancy needs `allowsVibrancy = true` on custom leaf views. "Once enabled in a parent view, a subview cannot turn off vibrancy."
  - "Do not override draw(_:) or updateLayer()" in subclasses.
  AK`nsvisualeffectview`, AK`nsvisualeffectview/material-swift.enum`, AK`nsvisualeffectview/blendingmode-swift.enum`, AK`nsvisualeffectview/state-swift.enum`, AK`nsview/allowsvibrancy`

---

## 7. Animation

- `Animation`: "The way a view changes over time". Families:
  - `default`.
  - `linear`, `easeIn`, `easeOut`, `easeInOut` (each also with `duration:`).
  - Spring presets `smooth`, `snappy`, `bouncy`, each tunable with `(duration:extraBounce:)`.
  - `spring(duration:bounce:blendDuration:)`, `interactiveSpring`, `interpolatingSpring`.
  - Custom: `timingCurve` and `init(_ base: some CustomAnimation)`.
  - Higher-order: `delay`, `repeatCount`, `repeatForever`, `speed`.
  SUI`animation`
- Presets: `smooth` is "no bounce", `snappy` is "small amount of bounce", `bouncy` is "higher amount of bounce". SUI`animation/smooth`, SUI`animation/snappy`, SUI`animation/bouncy`. `spring(duration:bounce:…)` is "persistent … each animation will be replaced by their successor, preserving velocity". SUI`animation/spring(duration:bounce:blendduration:)`
- `Spring` (iOS 17) converts between duration/bounce and mass/stiffness/damping. It can also be evaluated on its own with `value(target:…time:)`. SUI`spring`. "Animate with springs": Apple configures springs with "just 2 parameters: duration and bounce … we're adopting these universally across Apple's design and engineering efforts". It also says springs are now the default animation. WWDC`wwdc2023/10158`
- `CustomAnimation` protocol: `animate(value:time:context:) -> V?` returns `nil` when the animation is done. Optional: `velocity` and `shouldMerge`. `AnimationContext` provides `state` and `environment`, so an animation can read environment values. SUI`customanimation`, SUI`animationcontext`
- `withAnimation(_:_:)` "sets the given Animation as the animation property of the thread's current Transaction". SUI`withanimation(_:_:)`
- `.animation(_:value:)` is scoped to changes of one value. `.animation(_:body:)` applies the animation only to the modifiers inside the closure. SUI`view/animation(_:value:)`, SUI`view/animation(_:body:)`
- `Transaction`, "The context of the current state-processing update", carries `animation`, `disablesAnimations`, `isContinuous`, `tracksVelocity` and custom `@Entry` keys. `disablesAnimations` "is `true` during the initial phase of a two-part transition update". Use `.transaction(_:)` and `withTransaction` to modify it for a subtree. SUI`transaction`, SUI`transaction/disablesanimations`, SUI`view/transaction(_:)`
- `Animatable` / `animatableData` (`VectorArithmetic`), with the `@Animatable` macro. "SwiftUI reads the old and new animatableData values, then interpolates between them." SUI`animatable`
- `Transition` protocol (iOS 17): `body(content:phase:)` with `TransitionPhase` (`.willAppear`, `.identity`, `.didDisappear`). Built-in transitions: `opacity`, `scale`, `slide`, `move(edge:)`, `push(from:)`, `blurReplace` and others. Transitions "should not use any identity-affecting changes like `.id`, `if`, and `switch`". SUI`transition`
- `PhaseAnimator` cycles through discrete phases. `KeyframeAnimator` uses timed tracks (`LinearKeyframe`, `SpringKeyframe`, `CubicKeyframe`, `MoveKeyframe`). SUI`phaseanimator`, SUI`keyframeanimator`, SUI`animations`. Session: "Wind your way through advanced animations in SwiftUI". WWDC`wwdc2023/10157`
- `matchedGeometryEffect(id:in:properties:anchor:isSource:)` interpolates frames "in window space" between a removed view and an inserted view. "If the number of currently-inserted views in the group with `isSource = true` is not exactly one results are undefined." SUI`view/matchedgeometryeffect(id:in:properties:anchor:issource:)`
- How it works, from "Explore SwiftUI animation":
  - Covers "how SwiftUI … determines what to animate using Animatable, interpolates values over time using Animation, and propagates context for the current update using Transaction".
  - Built-in animatable attributes like `scaleEffect` animate "off the main thread and without calling into any of your view" code.
  - A bare `withAnimation` has used a smooth spring by default since iOS 17.
  WWDC`wwdc2023/10156`
- Reduce Motion is the app's job:
  - SwiftUI: `accessibilityReduceMotion` (read-only) and, from 26.4, `accessibilityPrefersCrossFadeTransitions`.
  - AppKit: `NSWorkspace.shared.accessibilityDisplayShouldReduceMotion`, plus `accessibilityDisplayOptionsDidChangeNotification`.
  - HIG: "When this setting is active, ensure your app or game responds by reducing automatic and repetitive animations, including zooming, scaling, and peripheral motion" and "Make motion optional."
  SUI`environmentvalues/accessibilityreducemotion`, AK`nsworkspace/accessibilitydisplayshouldreducemotion`, HIG`accessibility`, HIG`motion`
- AppKit `NSAnimationContext` is "analogous to CATransaction … Each thread maintains its own stack of nestable NSAnimationContext instances, with each new instance initialized as a copy of the instance below (so, inheriting its current properties)." It has `duration`, `timingFunction`, `allowsImplicitAnimation` and `completionHandler`. Use `runAnimationGroup(_:completionHandler:)` with `view.animator()`. This is the same "inherited context" idea as `Transaction`. AK`nsanimationcontext`, AK`nsanimationcontext/runanimationgroup(_:completionhandler:)`, AK`nsanimationcontext/allowsimplicitanimation`

---

## 8. AppKit theming

- `NSAppearance` "determine[s] which colors and images AppKit uses when drawing windows, views, and controls … Apps inherit the default system appearance, windows inherit their app's appearance, and views inherit the appearance of their nearest ancestor … When AppKit draws a control, it automatically sets the current appearance on the current thread to the control's appearance." AK`nsappearance`
- `NSAppearance.Name`: `aqua`, `darkAqua`, `vibrantLight`, `vibrantDark`, `accessibilityHighContrastAqua`, `accessibilityHighContrastDarkAqua`, `accessibilityHighContrastVibrantLight`, `accessibilityHighContrastVibrantDark`. Increased contrast is a separate appearance, not a flag. AK`nsappearance/name-swift.struct`
- `bestMatch(from:)` picks the closest supported name. `performAsCurrentDrawingAppearance(_:)` and `currentDrawing()` (macOS 11) set or read the appearance used for colour resolution. `allowsVibrancy`. AK`nsappearance/bestmatch(from:)`, AK`nsappearance/performascurrentdrawingappearance(_:)`, AK`nsappearance/currentdrawing()`, AK`nsappearance/allowsvibrancy`
- `NSAppearanceCustomization` has `appearance`, which you set to override, and `effectiveAppearance`. The effective value "is provided by the nearest ancestor of the receiver that has set an appearance". NSApplication, NSWindow and NSView adopt it. `NSView.viewDidChangeEffectiveAppearance()` is the change hook. AK`nsappearancecustomization`, AK`nsappearancecustomization/effectiveappearance`, AK`nsview/viewdidchangeeffectiveappearance()`, AK`choosing-a-specific-appearance-for-your-macos-app`
- Dynamic colour: `NSColor(name:dynamicProvider: (NSAppearance) -> NSColor)`: "When methods on a color need color component values, AppKit calls the provider with currentDrawingAppearance." AK`nscolor/init(name:dynamicprovider:)`. Colours from asset catalogs and semantic colours "adapt automatically … color objects you create using fixed component values do not adapt." https://developer.apple.com/documentation/uikit/supporting-dark-mode-in-your-interface
- Semantic colours ("UI element colors"):
  - Label: `labelColor` through `quaternaryLabelColor` (also `quinaryLabelColor`).
  - Text: `textColor`, `placeholderTextColor`, `selectedTextBackgroundColor`, `unemphasizedSelected…`.
  - Content: `linkColor`, `separatorColor`, `selectedContentBackgroundColor`.
  - Control: `controlAccentColor`, `controlColor`, `controlBackgroundColor`, `controlTextColor`, `disabledControlTextColor`.
  - Window: `windowBackgroundColor`, `underPageBackgroundColor`.
  - Fill: `systemFill` through `quinarySystemFill`.
  AK`ui-element-colors`
  - `controlAccentColor`: "The user's current accent color preference … Don't make assumptions about the color space." AK`nscolor/controlaccentcolor`
  - Adaptable system colours: `systemBlue`, `systemRed`… AK`standard-colors`
- `NSControl.ControlSize`: `.mini`, `.small`, `.regular`, `.large`, `.extraLarge`, set on `NSControl.controlSize`. AK`nscontrol/controlsize-swift.enum`, AK`nscontrol/controlsize-swift.property`. `NSView.prefersCompactControlSizeMetrics` (26) is "inherited down the view hierarchy". It restores pre-26 metrics for all controls in the subtree. AK`nsview/preferscompactcontrolsizemetrics`, WWDC`wwdc2025/310`
- `NSButton.BezelStyle` sets the button's look:
  - `.automatic`.
  - Push: `.push`, `.flexiblePush`.
  - Disclosure: `.disclosure`, `.pushDisclosure`.
  - Toolbar: `.toolbar`, `.accessoryBar`, `.accessoryBarAction`.
  - Informational: `.helpButton`, `.badge`, `.circular`.
  - `.smallSquare` and `.glass`.
  "if the button is not bordered, the bezel style is ignored." AK`nsbutton/bezelstyle-swift.enum`, AK`nsbutton/bezelstyle-swift.property`
- `NSButton.bezelColor` is "The color of the button's bezel, in appearances that support it". `contentTintColor` is a tint for template image and text, "only applicable to borderless buttons … used in combination with other theme-appropriate effects". AK`nsbutton/bezelcolor`, AK`nsbutton/contenttintcolor`
- Custom drawing:
  - **Cell:** `NSCell` is "A mechanism for displaying text or images in a view object without the overhead of a full NSView subclass", with `draw(withFrame:in:)`, `drawInterior(withFrame:in:)` and `highlight(_:withFrame:in:)`. `NSControl.cellClass` swaps the cell. AK`nscell`, AK`nscontrol/cellclass`
  - **View:** override `NSView.draw(_:)`, or set `wantsUpdateLayer = true` and override `updateLayer()`, which is "a much more efficient way". AK`nsview/draw(_:)`, AK`nsview/updatelayer()`, AK`nsview/wantsupdatelayer`
  - Either way, the dynamic colours resolve against the view's appearance during drawing.
- Layout direction: `NSView.userInterfaceLayoutDirection`. "It is the responsibility of the view to respect this value." AK`nsview/userinterfacelayoutdirection`

---

## 9. Colour algorithms Apple exposes

- **Accent colour:**
  - SwiftUI `Color.accentColor`: "In macOS, SwiftUI applies customization of the accent color only if the user chooses Multicolor." SUI`color/accentcolor`
  - HIG: if the user picks any accent other than multicolor, "the system applies their chosen color … replacing your accent color". A fixed-colour sidebar icon is the exception. HIG`color`
  - `tint(_:)` is the per-control override that is "always respected". SUI`view/tint(_:)-23xyq`
- **Derived shades from one base colour:**
  - **Hierarchical levels:** `.secondary` … `.quaternary` derive from the current foreground style. "They also automatically change their behavior as you set a color on them, setting versions of the color for each level. And the same support works for … even things like gradients." WWDC`wwdc2021/10021`, SUI`hierarchicalshapestyle`
  - **State shades:** `NSColor.withSystemEffect(_:)` with `NSColor.SystemEffect` `.none`, `.pressed`, `.deepPressed`, `.disabled` and `.rollover`. "Instead of defining separate colors for user interactions … This method blends the current color with an appropriate modifier … takes the current appearance into account." AK`nscolor/withsystemeffect(_:)`, AK`nscolor/systemeffect`
  - **Blends:** `NSColor.highlight(withLevel:)`, `shadow(withLevel:)` and `blended(withFraction:of:)`. AK`nscolor/highlight(withlevel:)`, AK`nscolor/blended(withfraction:of:)`. SwiftUI `Color.mix(with:by:in:)` (iOS 18 / macOS 15) defaults to the `.perceptual` colour space. SUI`color/mix(with:by:in:)`
  - **Gradient:** `Color.gradient` gives "the standard gradient for the color". SUI`color/gradient`
  - **Resolution:** `Color.resolve(in: EnvironmentValues) -> Color.Resolved` evaluates a semantic colour in a given environment. SUI`color/resolve(in:)`
- **Increased contrast:** HIG says to supply "light and dark variants, and an increased contrast option for each variant". HIG`color`. Contrast targets: "no lower than 4.5:1 … strive for a contrast ratio of 7:1, especially in small text". HIG`dark-mode`. Read the setting through:
  - SwiftUI: `colorSchemeContrast`.
  - AppKit: the `accessibilityHighContrast*` appearances and `NSWorkspace.accessibilityDisplayShouldIncreaseContrast` ("use a less subtle color palette or bolder lines"). AK`nsworkspace/accessibilitydisplayshouldincreasecontrast`
- **Base versus elevated backgrounds:** "In Dark Mode, the system uses two sets of background colors — called base and elevated … the background color automatically changes from base to elevated when an interface is in the foreground, such as a popover or modal sheet." HIG`dark-mode`
- **Desktop tinting:** with the graphite accent, "window backgrounds … pick up color from the current desktop picture". Custom component backgrounds should include some transparency in their neutral state. HIG`dark-mode`
- **Vibrancy:**
  - "a subtle blending of foreground and background colors to increase the contrast". It works best with greyscale content, so use `labelColor` … `quaternaryLabelColor`. AK`nsvisualeffectview`
  - In SwiftUI, "there's no extra API for this effect. It happens automatically when you use the new Secondary through Quaternary styles in a material context." WWDC`wwdc2021/10021`. See also SUI`environmentvalues/backgroundmaterial`.
  - HIG: "use vibrant colors on top of materials" and "Choose materials … based on semantic meaning … Avoid selecting a material … based on the apparent color." HIG`materials`
- **Liquid Glass colour:** it "has no inherent color, and instead takes on colors from the content directly behind it". Small elements switch between light and dark based on what is underneath. Prominent buttons put the accent on the glass background, not on the label. HIG`color`. Reduce Transparency "makes Liquid Glass frostier", and accessibility settings "act as modifiers on the material". WWDC`wwdc2025/219`. Toolbar items with a prominent style tint the glass with the accent colour, and `backgroundTintColor` overrides it. WWDC`wwdc2025/310`
- **Reduce Transparency and Differentiate Without Color on AppKit:** `NSWorkspace.accessibilityDisplayShouldReduceTransparency` and `…ShouldDifferentiateWithoutColor`. AK`nsworkspace/accessibilitydisplayshouldreducetransparency`, AK`nsworkspace/accessibilitydisplayshoulddifferentiatewithoutcolor`
- HIG on system colours: "Avoid hard-coding system color values … Each dynamic color is semantically defined by its purpose, rather than its appearance … Avoid redefining the semantic meanings." HIG`color`

---

## 10. WWDC sessions on the design philosophy

| Session | Key idea (paraphrase or quote) | URL |
|---|---|---|
| The craft of SwiftUI API design: Progressive disclosure (WWDC22) | "designing APIs so that the complexity of the call site grows with the complexity of the use case"; four strategies: consider common use cases, "provide intelligent defaults", "optimize the call site", "compose pieces rather than enumerating possibilities" ("Compose, don't enumerate", e.g. `Spacer` instead of stack spacing options) | WWDC`wwdc2022/10059` |
| Demystify SwiftUI (WWDC21) | SwiftUI sees "identity, lifetime, and dependencies"; dependencies (incl. environment) decide what re-renders | WWDC`wwdc2021/10022` |
| SwiftUI Essentials (WWDC19) | views as values, modifiers, environment | WWDC`wwdc2019/216` |
| Add rich graphics to your SwiftUI app (WWDC21) | materials ultraThin…ultraThick; hierarchical foreground styles; automatic vibrancy in material context | WWDC`wwdc2021/10021` |
| What's new in SwiftUI (WWDC22) | ShadowStyle, `Color.gradient`, Layout | WWDC`wwdc2022/10052` |
| Compose custom layouts with SwiftUI (WWDC22) | `Layout` protocol, `ViewThatFits`, `AnyLayout` | WWDC`wwdc2022/10056` |
| Explore SwiftUI animation (WWDC23) | Animatable → Animation → Transaction; attribute graph; CustomAnimation (`animate`, `shouldMerge`, `velocity`) | WWDC`wwdc2023/10156` |
| Animate with springs (WWDC23) | duration + bounce everywhere; smooth/snappy/bouncy; springs default | WWDC`wwdc2023/10158` |
| Wind your way through advanced animations in SwiftUI (WWDC23) | phaseAnimator, keyframeAnimator | WWDC`wwdc2023/10157` |
| Create custom visual effects with SwiftUI (WWDC24) | `visualEffect`, scroll transitions, shaders | WWDC`wwdc2024/10151` |
| Enhance your UI animations and transitions (WWDC24) | zoom/navigation transitions | WWDC`wwdc2024/10145` |
| SwiftUI Accessibility: Beyond the basics (WWDC21) | accessibility modifiers and children | WWDC`wwdc2021/10119` |
| Meet Liquid Glass (WWDC25) | lensing, adaptivity, accessibility settings modify the material | WWDC`wwdc2025/219` |
| Build a SwiftUI app with the new design (WWDC25) | adopting glass in SwiftUI | WWDC`wwdc2025/323` |
| Build an AppKit app with the new design (WWDC25) | concentricity, `extraLarge` control size, `prefersCompactControlSizeMetrics` inherited, `NSGlassEffectView` contentView, prominent tinted toolbar items | WWDC`wwdc2025/310` |

Swift Evolution items behind the API shape:

- SE-0258 Property Wrappers (`@Environment`, `@State`): https://github.com/swiftlang/swift-evolution/blob/main/proposals/0258-property-wrappers.md
- SE-0289 Result Builders (`@ViewBuilder`, `@ContentBuilder` in `makeBody`): https://github.com/swiftlang/swift-evolution/blob/main/proposals/0289-result-builders.md
- SE-0299 Static member lookup in generic contexts (`.toggleStyle(.switch)`): https://github.com/swiftlang/swift-evolution/blob/main/proposals/0299-extend-generic-static-member-lookup.md

---

## Lessons for BarrierKit

1. **Separate the control from its look.** Each control (`Button`, `Toggle`, `Label`, `ProgressView`, `Gauge`, `Menu`) keeps semantics and state. A `…Style` trait has one method, `make_body(&self, configuration) -> impl View`. The configuration holds only what the look needs:
   - `label`, `is_pressed` and `role` for a button.
   - `is_on` (binding) and `is_mixed` for a toggle.
   - `icon` and `title` for a label.
   `ButtonStyle` keeps the system interaction. A `PrimitiveButtonStyle`-like variant owns the gesture and calls `configuration.trigger()`.
2. **Styles are environment values, not instance fields.** `.button_style(s)` writes into the environment of a subtree. Every button below reads it. Controls that are built from a button (a toggle in button style, a menu in button style) pick it up too. The instance level is the same modifier on a single view. Per §1.4, one mechanism covers both the container level and the instance level.
3. **Offer "decorate the current style" next to "replace".** Expose `Toggle::from(configuration)` and `Button::from(configuration)`, which render the *inherited* style, so a custom style can wrap it with padding or a border. Write down Apple's ordering rule: the outer style modifier wins, and the inner one is evaluated first (§1.5).
4. **Keep open and closed styles apart on purpose.** Open `make_body` only for controls with a small, stable configuration. Complex controls (text field, picker) get a closed enum of built-in styles, as Apple does with `PickerStyle` and `TextFieldStyle`.
5. **Use a typed, inherited environment with defaults.** Each key has one default (`@Entry var x: T = default`). Overrides apply "to the view and its descendants … up to the point where you apply a different environment modifier". Each key gets a dedicated modifier (`.control_size()`, `.tint()`), and raw `.environment(key, v)` is a fallback. Some keys need special merge rules:
   - `disabled`: a parent can force it and a child cannot undo it.
   - `preferredColorScheme`: propagates up to the window.
6. **Store user settings as read-only environment values set by the platform:** `color_scheme`, `color_scheme_contrast`, `reduce_motion`, `reduce_transparency`, `differentiate_without_color`, `layout_direction`, `control_size` and `is_enabled`. In BarrierKit they come from `org.freedesktop.appearance` and the portal. Styles *read* them, and apps never poll the system.
7. **Paint is a `ShapeStyle` value that resolves late.** Colours, gradients, materials, `tint`, `.secondary` and `.separator` are all values with `resolve(&Environment) -> Resolved`. Resolution happens at draw time against the current environment, as with `Color.resolve(in:)` and AppKit's `NSColor(name:dynamicProvider:)`. Never bake RGB into a widget. Light, dark and increased contrast then need no code in the widget.
8. **Use semantic roles over raw colours, with a hierarchy derived from one base:**
   - Role tokens: `label`, `secondary_label`…, `separator`, `control_background`, `window_background`, `accent`, `selection`.
   - `primary`…`quaternary` (and `quinary`) derive from the current foreground style. Setting the foreground to red gives four reds.
   - State shades come from one function, like `NSColor.withSystemEffect(.pressed/.rollover/.disabled)`, not one token per state.
   - Increased contrast is a separate variant per token, not an alpha tweak.
   All of this lives in `ctdesign`.
9. **Accent and tint are two layers.** The user's accent is the system level. App code cannot beat it when the user picked a non-multicolor accent, and it is available as the `.tint` style. A per-control `tint` is the instance level. It is "always respected" and it carries meaning (green to answer, red to decline). Each style decides *where* tint applies: Apple tints the background of a prominent button and the label of a borderless one.
10. **Materials are styles and depend on context.** `Material` (ultra_thin…ultra_thick, bar) and Liquid Glass (`regular`, `clear`, `identity`, `.tint()`, `.interactive()`) are shape styles or effects. Their rendering is the compositor's or the platform's job. They set `background_material` in the environment so that `.secondary`… render with vibrancy automatically. Choose them by purpose (`sidebar`, `menu`, `popover`), never by apparent colour. Reduce Transparency changes the material itself, making it frostier or opaque, without any app code. Group nearby glass shapes in a container (`GlassEffectContainer` / `NSGlassEffectContainerView`) so they render and morph together.
11. **Shapes are values with container-relative geometry.**
    - Offer `RoundedRectangle{radius, style: Continuous|Circular}`, `Capsule`, `Circle`, `UnevenRoundedRectangle` (leading/trailing, so it follows RTL) and `ContainerRelativeShape`/`ConcentricRectangle`.
    - Offer a `container_shape()` modifier so that inner corners are computed from the container's radius minus the inset, never hard-coded.
    - `button_border_shape()` is an environment value that bordered styles read.
    - `content_shape` (hit-testing) stays separate from `clip_shape` (rendering).
12. **Size is a semantic scale:** `ControlSize::{Mini, Small, Regular, Large, ExtraLarge}` in the environment. Styles map it to metrics from `ctdesign`. Add a compact-metrics switch that is inherited down the tree, like `prefersCompactControlSizeMetrics`, for dense inspectors. Use `None` for "system default" spacing and padding, as `HStack(spacing: nil)` and `.padding()` do.
13. **Animations are values carried by a transaction.**
    - `Animation` is a plain value: smooth, snappy, bouncy, `spring(duration, bounce)` and ease curves, plus `delay`, `speed` and `repeat`. Express springs only as duration and bounce, as Apple does, and convert to stiffness and damping internally.
    - A `Transaction` (the `NSAnimationContext` stack in AppKit) flows down with the update. It holds `animation`, `disables_animations` and custom keys.
    - `with_animation(a, || …)` sets it for a state change. `.animation(a, value)` scopes it to one value.
    - Make the default for a bare `with_animation` a smooth spring.
    - Allow custom curves through one trait, `CustomAnimation::animate(value, t, ctx) -> Option<V>`, where the context can read the environment.
14. **Accessibility overrides sit at the bottom and styles honour them.** Built-in styles and transitions read `reduce_motion` (replace slides and scales with cross-fades), `reduce_transparency` (opaque materials), `increased_contrast` (contrast variants and bolder strokes) and `differentiate_without_color` (add a glyph or shape). Apple leaves custom animation to the app ("UI should avoid large animations"). BarrierKit can go further: built-in `Transition`s and spring presets can degrade on their own when `reduce_motion` is set.
15. **Disclose complexity in steps.** Use defaults first (`Button("Save")`). Next come built-in style values (`.button_style(BORDERED_PROMINENT)`). Then environment knobs (`tint`, `control_size`, `button_border_shape`). A custom `ButtonStyle` comes last. Full custom drawing (AppKit's `draw(_:)`/`updateLayer()`, or BarrierKit's own canvas) is the final escape hatch, and dynamic colours still resolve against the current appearance there. Compose instead of enumerating options. Per Apple's progressive-disclosure session: "Compose, don't enumerate."
