# 16. SwiftUI to the kit

**Goal:** write the kit's form of SwiftUI code that is not a name: state,
bindings, environment, observation, focus, loops, conditions, actions and
effects.
**Scope:** the SwiftUI constructs whose kit form is a pattern, not a type.
Names are in the [glossary](20-glossary.md); each control's methods are in its
chapter. AppKit idioms are not covered.
**Reader:** an app author, or an AI agent, who has the SwiftUI code and does not
know the kit's shape.
**Input:** SwiftUI code. **Output:** the kit idiom and the recipe to copy.

## Steps

1. Find the Swift construct in the [table](#constructs).
2. Copy the recipe in its last column and change only what the task needs.
3. Before you build a substitute for a construct that has no row, read
   [What the kit does not do](#what-the-kit-does-not-do).

## Constructs

| Swift | Kit | Recipe |
|---|---|---|
| `@State var count = 0` | `let mut count = use_state(\|\| 0)`; `count.read()`, `count.set(v)`, `*count.write() += 1` | [`counter.rs`](../../agents/recipes/counter.rs) |
| `@Binding`, `$text` on a text field | pass the `State<String>`; the field edits it | [`binding.rs`](../../agents/recipes/binding.rs) |
| `@Binding`, `$on` on any other control | pass the value and handle the change | [`toggle.rs`](../../agents/recipes/toggle.rs) |
| `@Environment`, `.environment(_:_:)` | `use_provide_context(\|\| value)` above, `use_consume::<T>()` below | [`environment.rs`](../../agents/recipes/environment.rs) |
| `@Observable` | a `Copy` struct of `State`s, given as a context | [`observable.rs`](../../agents/recipes/observable.rs) |
| `@FocusState`, `.focused($f)` | `use_a11y()`, `.a11y_id(id)`, `id.request_focus()` | [`focus_state.rs`](../../agents/recipes/focus_state.rs) |
| `ForEach(items, id: \.id)` | `.children(iter.map(..))` and `.key(id)` | [`for_each.rs`](../../agents/recipes/for_each.rs), [`stopwatch.rs`](../../agents/recipes/stopwatch.rs) |
| `if` and `else` in a view builder | a Rust `if`, `.maybe_child(opt)`, `.into_element()` | [`if_else.rs`](../../agents/recipes/if_else.rs) |
| `let onSave: () -> Void` in a view | a `Callback<A, R>` field, built from any closure | [`callback.rs`](../../agents/recipes/callback.rs) |
| `.commands`, `.keyboardShortcut` | `Action`, `MainMenu`, `use_menu_action` | [`actions.rs`](../../agents/recipes/actions.rs) |
| `.onAppear`, `.onDisappear`, `.onChange`, `.task` | [`use_hook`, `use_drop`, `use_side_effect_with_deps`, `use_future`](#effects) | none |

## State and bindings

`use_state` returns a `State<T>`. It is `Copy`, so a closure takes it by
`move`. `read()` subscribes the view to the value; `peek()` does not. `write()`
and `set` need `let mut`.

A hook runs in the same order on every render. Never call one inside an `if`,
a loop or an iterator closure; a wrong order panics at run time. `.help` is a
hook ([chapter 7](07-menus-and-popovers.md#view-modifiers)).

The kit has no `Binding`. The text controls (`TextField`, `SearchField`,
`SecureField`, `TextEditor`, `ComboBox`, `TokenField`) and `ImageWell` take the
`State` they edit, so a child view receives that `State` as a field. Every
other control takes its value and a callback
([the controlled pattern](05-controls.md#the-controlled-pattern)).

```swift
// ✗ Toggle("Wi-Fi", isOn: $wifi)
```

✓ [`toggle.rs`](../../agents/recipes/toggle.rs): `Switch::new("Wi-Fi").on(*wifi.read()).on_toggle(..)`.

## Environment

A context is found by its type, in the view or the nearest ancestor that
provides it. `use_provide_context` keeps one value for the life of the view,
and `use_consume` returns the same value on every later render; it panics when
no ancestor provides the type. A value that changes needs a `State` inside it.

The built-in keys have their own readers:

| SwiftUI | Kit |
|---|---|
| `\.colorScheme`, contrast, accent | `Palette::current()`, `theme::resolved()`; `use_appearance(..)` overrides them for a subtree ([4](04-appearance-and-tokens.md)) |
| `\.controlSize` | `.control_size(ControlSize)` on a view |
| `\.accessibilityReduceMotion` | `use_reduced_motion()` |
| `\.isEnabled`, `\.font`, `\.foregroundStyle` | set on each element; a `Text` keeps its own color ([`styles.md`](../../agents/styles.md)) |

## Observation

`@Observable` tracks each property. The kit does the same by construction: a
view subscribes to the `State` it reads, so a view that reads `count` and not
`step` ignores a change to `step`. Put the states in one `Copy` struct, build
it in the render (its fields are hooks), and give it to `use_provide_context`.

✓ [`observable.rs`](../../agents/recipes/observable.rs).

## Focus

`use_a11y()` returns an id that the view owns. A control that takes the id
becomes the focus target, `id.request_focus()` moves the focus to it, and
`use_focus(id)` reads its focus state. Only `SearchField` takes an id;
`TextField` and `SecureField` have `.auto_focus(bool)`. There is no enum of
fields.

✓ [`focus_state.rs`](../../agents/recipes/focus_state.rs).

## Loops and conditions

A stack takes its children one at a time: `.child`, `.children(iter)` and
`.maybe_child(opt)`. There is no closure that returns several views.

`.key(id)` gives a child an identity, so its state follows its item when the
list changes. Children without a key match by position. A duplicate key among
siblings panics in debug builds.

```swift
// ✗ ForEach(fruits, id: \.id) { Text($0.name) }
```

✓ [`for_each.rs`](../../agents/recipes/for_each.rs).

Two arms of an `if` that return different types do not compile. Convert each
arm with `.into_element()`, or leave the arm out with `.maybe_child`.

```swift
// ✗ if signedIn { Text("Welcome back") } else { Button("Sign in") { } }
```

✓ [`if_else.rs`](../../agents/recipes/if_else.rs).

A string given to `.child("x")` becomes Freya's bare label, with no theme. Use
`Text`.

## Actions

An `Action` is a constant with an id, a title and an optional shortcut.
`MainMenu` lists its items, and `use_menu_action` receives its id
([chapter 13](13-app.md#actions-and-shortcuts)). Every handler of the window
receives every id: compare it with the action's `.id`.

```swift
// ✗ .commands { CommandMenu("Counter") { Button("Add one") { count += 1 }.keyboardShortcut("k") } }
```

✓ [`actions.rs`](../../agents/recipes/actions.rs).

## Effects

| SwiftUI | Kit | When it runs |
|---|---|---|
| `.onAppear` | `use_hook(\|\| ..)` | in the first render of the view |
| `.onDisappear`, `deinit` | `use_drop(\|\| ..)` | when the view leaves the tree |
| `.onChange(of:)` | `use_side_effect_with_deps(&value, ..)`; the old value from `use_previous_and_current` | when the value changes, and once at creation |
| `.task`, `.task(id:)` | `use_future(..)`; a `State` read outside the async block restarts it | cancelled when the view leaves the tree |
| `TimelineView` | `use_timeline(active, every)` | every `every` while `active` ([`stopwatch.rs`](../../agents/recipes/stopwatch.rs)) |

The hooks are in `freya-core` 0.5.3, `src/lifecycle/` and `src/hooks/`.

## What the kit does not do

[ADR 0085](../decisions.md) fixes these as non-goals until an app asks for one.

| SwiftUI or AppKit | In the kit |
|---|---|
| a `Binding` type, `$model.field` | none; use the `State` or the value and callback above |
| a command sent to the focused view (the responder chain) | none; every handler of the window gets the id |
| `windowShouldClose` | none; a close request closes the window at once, except a sheet's |
| `applicationShouldTerminate`, `NSApp.terminate` | none; the last window closing ends the process |
| `.onOpenURL`, `application(_:open:)` | none |

The full audit of Swift syntax against the kit, with probes, is in
[Swift syntax in Rust](../research/swift-syntax-in-rust.md).
