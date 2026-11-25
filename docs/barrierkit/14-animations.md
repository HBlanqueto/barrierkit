# 14. Animations

**Goal:** animate a value of your own with the kit's durations and spring, and
respect reduced motion.
**Scope:** starts with a component that changes a value and ends with that
change animated. It covers `bk::animation` and how the kit uses
`freya-animation`. The duration tokens are defined in
[4. Appearance and tokens](04-appearance-and-tokens.md#motion); the controls that
animate on their own are in their chapters.
**Reader:** anyone who adds motion to an app or a control.
**Input:** a `State` whose change should animate. **Output:** a value that
moves from the old state to the new one, or jumps when the user asked for
reduced motion.

## Steps

1. Add `freya-animation` to the app's `Cargo.toml`. The kit builds on it and
   does not re-export it.
2. Read the reduced motion preference once per component:

   ```rust
   let reduced_motion = use_reduced_motion();
   ```

3. Animate the state with `use_animation_transition`. Give it a duration token
   through `motion_ms`, so reduced motion turns it into 0:

   ```rust
   let target = use_state(|| 0.0f32);
   let motion = use_animation_transition(target, move |from, to| {
       Spring::new(
           from,
           to,
           Duration::from_millis(motion_ms(reduced_motion, DURATION_BASE)),
           0.0,
       )
   });
   ```

4. Read the current value while rendering: `motion.get().current()` for a
   `Spring`, `motion.get().value()` for an `AnimNum`.
5. Change the state (`target.set(1.0)`). The animation runs from the old value
   to the new one.

## Durations

The kit times every animation with these tokens of `ctdesign` (the springs are in [Spring](#spring)):

| Token | Use |
|---|---|
| `DURATION_FAST_MS` | Quick feedback: pressed scales, `Slider` clicks |
| `DURATION_BASE` | Showing, hiding, collapsing, morphing |
| `DURATION_SLOW_MS` | Fading between backdrops |
| `DURATION_LIST_ROW_MS` | Rows of a `List` or `Table` inserted, removed or moved inside `with_animation` (SwiftUI on macOS) |
| `DURATION_MENU_CLOSE_MS`, `DURATION_POPOVER_IN_MS`, `DURATION_POPOVER_OUT_MS` | Measured on Mac Catalyst; no kit control uses them yet |

A Freya animation takes them as milliseconds: `AnimNum::new(from, to)
.time(DURATION_BASE).ease(Ease::Out).function(Function::Cubic)`. A component
token with its own duration (`SWITCH.transition_ms`,
`PROGRESS.indeterminate_ms`) wins over the general ones.

✗ A literal in the app: `.time(250)`.
✓ A token: `.time(motion_ms(reduced_motion, DURATION_BASE))`.

## What animates in the kit

| Element | Duration | Curve |
|---|---|---|
| `Switch` knob | `SWITCH.transition_ms` | `Spring`, bounce `SWITCH.transition_bounce` (ω0 15, ζ 0.75, [measured](../research/macos-motion-controls.md#2-animated)) |
| `DisclosureGroup` and `SidebarSection`, expanding and collapsing: the rows unroll and the chevron turns 90 degrees | `DURATION_DISCLOSURE_MS` | `ease_default`, Core Animation's default Bézier (0.25, 0.1, 0.25, 1); a click animates, the child rows stay mounted until the end (measured on SwiftUI's `DisclosureGroup`, `List(children:)` and `Section(isExpanded:)`) |
| `OutlineView` expanding a row | `DURATION_OUTLINE_EXPAND_MS` | `ease_out_appkit`, Core Animation's `easeOut` (0, 0, 0.58, 1); collapsing and the chevron are instant, as `NSOutlineView` (`OutlineViewStyle.animation`) |
| Showing or hiding the sidebar by the toggle (button, shortcut, menu) or inside `with_animation`; the inspector always | `DURATION_BASE` | `ease_in_ease_out`, Core Animation's `easeInEaseOut` (measured on the sidebar of a SwiftUI `NavigationSplitView`) |
| Sidebar width while dragging, by keys or by a double click on the divider | none | One frame, as AppKit |
| `Slider` and `LevelSlider` knob and fill, on a click on the track | `SLIDER.click_ms` | `Spring`, bounce `SLIDER.click_bounce` (ω0 18, ζ 0.95, [measured](../research/macos-motion-controls.md#2-animated)); a drag, a key and a value set in code are instant |
| `Slider` and `LevelSlider` knob growing and shrinking | `SLIDER.press_ms` | `ctdesign::slider_press`, Bézier (0, 0.13, 0.53, 1.11) |
| [Rows of a `List` or `Table`](11-views.md#rows-inserted-removed-and-moved) inserted, removed or moved inside `with_animation` | `DURATION_LIST_ROW_MS` | `ease_default`, whatever the transaction's animation (`TableStyle.animation`); instant without `with_animation` |
| [`NavigationStack`](11-views.md#navigationstack) push and pop | `SPRING_PUSH_MS` | `Spring`, no bounce |
| Pill of a [`TabView`](11-views.md#tabview) strip | `SPRING_SHEET_MS` | `Spring`, no bounce |
| Revealing hidden titlebars, the focus bar | `DURATION_BASE` | Ease out, cubic |
| Scroll bar fade in, out | `SCROLL_BAR.fade_in_ms`, `SCROLL_BAR.fade_out_ms`; hidden `SCROLL_BAR.hide_delay_ms` after the last scroll | `ease_in_ease_out`, measured on `flashScrollers()`; `ScrollViewStyle.fade_in` and `fade_out` |
| `scroll_to` and `.scroll_position(..)` set inside `with_animation` | The app's `Animation` | Instant without `with_animation`; SwiftUI's `scrollTo` and `NSClipView.animator()` are too |
| Determinate `ProgressView` fill, on every value change | `PROGRESS.fill_ms` | Bézier `PROGRESS.fill_curve`; ignores `with_animation` |
| Indeterminate `ProgressView` | `PROGRESS.indeterminate_ms` | Linear, a segment with a fading tail crosses the bar and starts again |
| Open `Gauge` marker | The animation of `with_animation` | Only inside `with_animation`; `ProgressRing` and the capacity ring jump |
| `Spinner` | `PROGRESS.spinner_ms` | Linear, `PROGRESS.spinner_frames` steps a turn, looping |
| Fade between [`Backdrop`](15-backdrops-and-media.md#backdrop) colors | `DURATION_SLOW_MS` | In-out, cubic |

Pressing a button changes its fill to the pressed color; `BUTTON.pressed_scale` is 1, so it does not scale, as in AppKit.

## Spring

`Spring` is the kit's spring, SwiftUI's `Spring(duration:bounce:)`. It
implements Freya's `AnimatedValue`, so it works in `use_animation` and
`use_animation_transition` like `AnimNum`. A physical spring matches the
measured curves and carries its velocity, which a timed curve cannot.

| Item | What it does |
|---|---|
| `Spring::new(from, to, duration, bounce)` | `duration` is the period `2π / ω0`; `bounce` is `1 − ζ`, clamped to 0–0.95 |
| `.with_velocity(v)` | The velocity at the start, in units per second |
| `.settling_duration()` | When the value is within 0.001 of the travel; from then on it is `to` exactly |
| `.value(elapsed)`, `.velocity(elapsed)` | The value and its rate after `elapsed` |
| `.current()` | The value at the animation's current time |

- With `bounce` 0 the spring is critically damped: no overshoot. Every spring
  measured is this one: periods of 344 ms (sheets, tab strip pill), 274 ms
  (alerts, Mac Catalyst) and 503 ms (push and pop, SwiftUI on macOS); they are
  the `SPRING_*_MS` tokens
  ([UIKit animations](../research/uikit-animations.md#2-motion-table),
  [containers](../research/macos-motion-containers.md#2-motion-table)).
- With more bounce it is underdamped and overshoots.
- A spring started from the value and velocity of another goes on exactly as
  the first would have. That is how a change interrupted midway keeps moving
  without a jump.
- `ctdesign::ease_out` and `ctdesign::ease_in_out` are the kit's cubic timed
  curves: the kit, the shell and the compositor call the same two functions.
  `ctdesign::ease_in_ease_out` is Core Animation's `easeInEaseOut` Bézier
  (0.42, 0, 0.58, 1), which AppKit's sidebar follows. `ctdesign::cubic_bezier`
  evaluates any such curve.
- In reverse it runs from `to` to `from`, from rest.

## Implicit animation

SwiftUI animates what changes inside `withAnimation`. The kit does the same
with two functions:

| Item | What it does |
|---|---|
| `with_animation(animation, body)` | Runs `body` and tags the changes that follow it with `animation`. `Animation::None` is `withAnimation(nil)` |
| `use_animated(target, animation)` | Returns a value that follows `target`. When `target` changes within 100 ms of a `with_animation`, it uses that animation; otherwise `animation` |

```rust
let progress = use_animated(f32::from(u8::from(*expanded.read())), Animation::None);
Button::new("Show").on_press(move |_| {
    with_animation(Animation::Spring { duration_ms: SPRING_SHEET_MS, bounce: 0.0 }, || expanded.toggle())
})
```

- Reduced motion turns every animation into `Animation::None`.
- A target that changes while the value moves starts a new animation from the
  value and the velocity shown. A spring keeps the velocity; `Ease` and
  `Custom` start from rest.
- `use_animated` reads the time since the change, so a 250 ms animation takes
  250 ms at any refresh rate. Freya's `AnimNum` and the kit's `Spring` in an
  app's own `use_animation` add whole milliseconds per frame instead: at
  180 Hz they run about 10 % long.
- On macOS every animation steps at the start of a display refresh, before the
  frame is drawn.
- The outermost `with_animation` of a turn decides. SwiftUI scopes a transaction
  to the writes in its closure, so an inner `withAnimation` animates its own
  writes; the kit ignores the inner call, because a Freya `State` has one write
  path the kit cannot tag.

## Transitions

A view that comes and goes animates when an app wraps it in `Presence` and
changes the state inside `with_animation`. It reuses `use_animated` and the
`with_animation` transaction, so a transition moves as any other change does:

```rust
Presence::new((*shown.read()).then(|| Text::new("Hello")))
    .transition(Transition::move_edge(Edge::Leading).combined(Transition::OPACITY))
```

| Item | What it does |
|---|---|
| `Presence::new(Option<view>)` | `Some` shows the view, `None` removes it; the removed view stays mounted until its animation ends. Without `with_animation` it comes and goes at once. `.key(..)` as every view |
| `.transition(t)` | Default `Transition::OPACITY` |
| `Transition::OPACITY`, `SCALE`, `IDENTITY` | The look of a view that is gone: transparent, scaled to 0, or unchanged |
| `Transition::scale(s)`, `offset(x, y)`, `move_edge(edge)` | Scaled to `s`, shifted by `x, y`, or one of its own widths or heights beyond `edge` (left to right) |
| `.combined(other)` | Both at once: opacity and scale multiply, offsets add |
| `Transition::asymmetric(insertion, removal)` | One on the way in, the other on the way out |
| `Image::system(..)` | A symbol that changes inside `with_animation` crossfades without any modifier (SwiftUI's `Image(systemName:)` swap: t50 140 ms, settle 290 ms, the transaction's spring); outside `with_animation` it swaps at once. `.content_transition(ContentTransition::Identity)` on the `Image` turns it off |
| `.content_transition(ContentTransition::Opacity)` | Any view: when it changes, the old one fades out over the new one, which fades in. `ContentTransition::Identity` swaps it at once |

- Progress is `use_animated`'s, so a spring may overshoot a scale and a toggle
  in midway goes on from the value and velocity shown.
- A leaving view takes no pointer events.
- An absent `Presence` takes no room in a stack, spacing included.
- Not built: `matchedGeometryEffect` (it needs a frame registry shared by two
  places of the tree and a first frame at the old place, before any layout),
  `ContentTransition::numericText` and `interpolate` (per-glyph), `slide`,
  `push`, `blurReplace`, and a transition that runs on a view the app removes
  without `Presence`.
- Not built for rows: `CollectionView` tiles inserted, removed or moved, and
  the height of a `List` section that gains or loses rows (it jumps; the rows
  inside it animate). `Table` and `List` copy SwiftUI's list, not AppKit's
  `insertRows` (ease-out, 200 ms) and `moveRow` (ease-out, 400 ms for 4 rows),
  which only an app that sets `TableStyle.animation` gets.
- Not built for symbols: `.symbolEffect(.replace)` (a 78 ms blur and scale of the glyph's layers, independent of the transaction), `variableValue`, `.bounce` and the other `symbolEffect`s. The kit draws a symbol as one SVG, with no per-layer or per-wave rendering and no blur, so a copy would be a fake. A pulse is an opacity loop of 2000 ms; it is not built because nothing in the kit loops a symbol yet.
- Glass buttons add no animation of their own. On macOS 26 and later `ButtonStyle::glass` and `glass_prominent` are one native `NSGlassEffectView`, interactive on macOS 27 (`effectIsInteractive`), so macOS plays the press and release lift. Without native glass (before macOS 26, Reduce Transparency) the button changes its fill only, as a bordered button does.

## Reduced motion

`reduced_motion = true` in `appearance.domain` asks for no motion, as does the
system's Reduce Motion on macOS or the `reduced-motion` key of the settings
portal on Linux. Every kit
animation then takes 0 ms and the loops (indeterminate progress, spinner) stay
still. An app's own animation does the same with two functions:

| Item | What it does |
|---|---|
| `use_reduced_motion()` | Returns a `State<bool>` that follows the preference live |
| `motion_ms(reduced_motion, ms)` | Returns 0 when reduced motion is on, `ms` otherwise |

Freya builds an animation's closure once, so the closure must read the
preference when it runs, not when it is built. `motion_ms` takes the `State`
for that reason: a change applies to the next animation.

✗ `let ms = if *use_reduced_motion().read() { 0 } else { DURATION_BASE };`
captured in the closure.
✓ `motion_ms(reduced_motion, DURATION_BASE)` inside the closure.

## Helpers

| Item | What it does |
|---|---|
| `use_switch_progress(on, animation)` | Returns 0–1 that follows `on: State<bool>` with the duration and curve of an [`Animation`](../../.claude/skills/barrierkit-app/styles.md#value-types); it jumps with `Animation::None` and with reduced motion. The full-screen bar uses it with `FullscreenBarStyle::animation` |
| `use_alive()` | Returns an `Rc<Cell<bool>>` that is `true` while the component is mounted and `false` after; a background task checks it before writing state |

```rust
let expanded = use_state(|| false);
let progress = use_switch_progress(expanded, Animation::Ease { duration_ms: DURATION_BASE });
rect().height(Size::px(progress * content_height))
```

Next: [15. Backdrops and media](15-backdrops-and-media.md).
