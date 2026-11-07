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

The kit times every animation with three tokens of `ctdesign`:

| Token | Use |
|---|---|
| `DURATION_FAST_MS` | Following the pointer: scroll bars, the sidebar width while dragging |
| `DURATION_BASE` | Showing, hiding, collapsing, morphing, chart data |
| `DURATION_SLOW_MS` | Fading between backdrops |

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
| Collapsing a sidebar section | `DURATION_BASE` | Ease out, cubic |
| Showing or hiding the sidebar and the inspector | `DURATION_BASE` | Ease out, cubic |
| Sidebar width while dragging | `DURATION_FAST_MS` | `Spring`, no bounce |
| Moving the sidebar into the rail, revealing hidden titlebars, the focus bar | `DURATION_BASE` | Ease out, cubic |
| Scroll bar | `DURATION_FAST_MS` | Ease out, cubic |
| Indeterminate `ProgressView` | `PROGRESS.indeterminate_ms` | In-out, cubic, back and forth |
| `Spinner` | `PROGRESS.spinner_ms` | Linear, one spoke at a time, looping |
| Fade between [`Backdrop`](15-backdrops-and-media.md#backdrop) colors | `DURATION_SLOW_MS` | In-out, cubic |
| [`GlassContainer`](15-backdrops-and-media.md#glass-that-merges) morph | `DURATION_BASE` | `Spring`, no bounce |
| [`Chart`](16-charts.md#chart) data change | `DURATION_BASE` | Ease out, cubic |

Pressing a button scales it to `BUTTON.pressed_scale` with no animation.

## Spring

`Spring` is the kit's spring curve (Grade). It implements Freya's
`AnimatedValue`, so it works in `use_animation` and
`use_animation_transition` like `AnimNum`.

| Item | What it does |
|---|---|
| `Spring::new(from, to, duration, bounce)` | `bounce` is clamped to 0–1 |
| `.current()` | The value at the animation's current time |
| `.value(elapsed)` | The value after `elapsed`; `to` from `duration` on |
| `.value_at(progress)` | The value at `progress` 0–1, clamped |

- With `bounce` 0 the curve is a cubic ease-out, with no overshoot.
- With more bounce it is a damped oscillation,
  `1 − e^(−SPRING_DAMPING·t)·cos(π(1 + bounce)·t)`, that overshoots and
  settles. `SPRING_DAMPING` is a `ctdesign` token.
- It always ends exactly on `to` at `duration`. In reverse it runs from `to`
  to `from`.

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
| `use_switch_progress(on, ms)` | Returns 0–1 that follows `on: State<bool>` with a cubic ease-out over `ms`; jumps with reduced motion. The kit's show and hide animations use it |
| `use_alive()` | Returns an `Rc<Cell<bool>>` that is `true` while the component is mounted and `false` after; a background task checks it before writing state |

```rust
let expanded = use_state(|| false);
let progress = use_switch_progress(expanded, DURATION_BASE);
rect().height(Size::px(progress * content_height))
```

Next: [15. Backdrops and media](15-backdrops-and-media.md).
