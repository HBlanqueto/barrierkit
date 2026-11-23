# 14. Animations

**Goal:** animate a value of your own with the kit's durations and spring, and
respect reduced motion.
**Scope:** starts with a component that changes a value and ends with that
change animated. It covers `bk::animation` and how the kit uses
`freya-animation`. The duration tokens are defined in
[4. Appearance and tokens](04-appearance-and-tokens.md); the controls that
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
| `DURATION_FAST_MS` | Following the pointer: scroll bars, the sidebar width while dragging |
| `DURATION_BASE` | Showing, hiding, collapsing, morphing |
| `DURATION_SLOW_MS` | Fading between backdrops |
| `DURATION_LIST_ROW_MS`, `DURATION_MENU_CLOSE_MS`, `DURATION_POPOVER_IN_MS`, `DURATION_POPOVER_OUT_MS` | Measured on Mac Catalyst; no kit control uses them yet |

A Freya animation takes them as milliseconds: `AnimNum::new(from, to)
.time(DURATION_BASE).ease(Ease::Out).function(Function::Cubic)`. A component
token with its own duration (`SWITCH.transition_ms`,
`PROGRESS.indeterminate_ms`) wins over the general ones.

✗ A literal in the app: `.time(250)`.
✓ A token: `.time(motion_ms(reduced_motion, DURATION_BASE))`.

## What animates in the kit

| Element | Duration | Curve |
|---|---|---|
| `Switch` knob | `SWITCH.transition_ms` | Ease out, cubic |
| Collapsing a sidebar section | `SPRING_SHEET_MS` | `Spring`, no bounce |
| Showing or hiding the sidebar and the inspector | `DURATION_BASE` | `ease_in_ease_out`, Core Animation's `easeInEaseOut` (measured on the sidebar of a SwiftUI `NavigationSplitView`) |
| Sidebar width while dragging | `DURATION_FAST_MS` | `Spring`, no bounce |
| Moving the sidebar into the rail, revealing hidden titlebars, the focus bar | `DURATION_BASE` | Ease out, cubic |
| Scroll bar | `DURATION_FAST_MS` | Ease out, cubic |
| Indeterminate `ProgressView` | `PROGRESS.indeterminate_ms` | In-out, cubic, back and forth |
| `Spinner` | `PROGRESS.spinner_ms` | Linear, one spoke at a time, looping |
| Fade between [`Backdrop`](15-backdrops-and-media.md#backdrop) colors | `DURATION_SLOW_MS` | In-out, cubic |
| [`GlassContainer`](15-backdrops-and-media.md#glass-that-merges) morph | `DURATION_BASE` | `Spring`, no bounce |

Pressing a button changes its fill to the pressed color; `BUTTON.pressed_scale` is 1, so it does not scale, as in AppKit.

## Spring

`Spring` is the kit's spring, SwiftUI's `Spring(duration:bounce:)`. It
implements Freya's `AnimatedValue`, so it works in `use_animation` and
`use_animation_transition` like `AnimNum`.

| Item | What it does |
|---|---|
| `Spring::new(from, to, duration, bounce)` | `duration` is the period `2π / ω0`; `bounce` is `1 − ζ`, clamped to 0–0.95 |
| `.with_velocity(v)` | The velocity at the start, in units per second |
| `.settling_duration()` | When the value is within 0.001 of the travel; from then on it is `to` exactly |
| `.value(elapsed)`, `.velocity(elapsed)` | The value and its rate after `elapsed` |
| `.current()` | The value at the animation's current time |

- With `bounce` 0 the spring is critically damped: no overshoot. Every spring
  measured on Mac Catalyst is this one, with periods of 344 ms (sheets), 274 ms
  (alerts) and about 251 ms (push); they are the `SPRING_*_MS` tokens
  ([UIKit animations](../research/uikit-animations.md#2-motion-table)).
- With more bounce it is underdamped and overshoots.
- A spring started from the value and velocity of another goes on exactly as
  the first would have. That is how a change interrupted midway keeps moving
  without a jump.
- `ctdesign::ease_out` and `ctdesign::ease_in_out` are the kit's cubic timed
  curves: the kit, the shell and the compositor call the same two functions.
  `ctdesign::ease_in_ease_out` is Core Animation's `easeInEaseOut` Bézier
  (0.42, 0, 0.58, 1), which AppKit's sidebar follows.
- In reverse it runs from `to` to `from`, from rest.

## Implicit animation

SwiftUI animates what changes inside `withAnimation`. The kit does the same
with two functions ([ADR 0102](../decisions.md)):

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
changes the state inside `with_animation` ([ADR 0113](../decisions.md)):

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
| `.content_transition(ContentTransition::Opacity)` | Any view: when it changes, the old one fades out over the new one, which fades in |

- Progress is `use_animated`'s, so a spring may overshoot a scale and a toggle
  in midway goes on from the value and velocity shown.
- A leaving view takes no pointer events.
- An absent `Presence` takes no room in a stack, spacing included.
- Not built: `matchedGeometryEffect` (it needs a frame registry shared by two
  places of the tree and a first frame at the old place, before any layout),
  `ContentTransition::numericText` and `interpolate` (per-glyph), `slide`,
  `push`, `blurReplace`, and a transition that runs on a view the app removes
  without `Presence`.

## Reduced motion

`reduced_motion = true` in `appearance.domain` asks for no motion. Every kit
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
| `use_switch_progress(on, animation)` | Returns 0–1 that follows `on: State<bool>` with the duration and curve of an [`Animation`](../../agents/styles.md#value-types); it jumps with `Animation::None` and with reduced motion. The full-screen bar uses it with `FullscreenBarStyle::animation` |
| `use_alive()` | Returns an `Rc<Cell<bool>>` that is `true` while the component is mounted and `false` after; a background task checks it before writing state |

```rust
let expanded = use_state(|| false);
let progress = use_switch_progress(expanded, Animation::Ease { duration_ms: DURATION_BASE });
rect().height(Size::px(progress * content_height))
```

Next: [15. Backdrops and media](15-backdrops-and-media.md).
