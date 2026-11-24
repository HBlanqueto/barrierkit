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
| `@AppStorage("bold") var bold = false` | `let bold = use_app_storage("bold", false)`; a `State` saved in the app's settings | [`app_storage.rs`](../../agents/recipes/app_storage.rs) |
| `@Binding`, `$text` on a text field | pass the `State<String>`; the field edits it | [`binding.rs`](../../agents/recipes/binding.rs) |
| `@Binding`, `$on` on any other control | pass the `State`: `Switch::new("Wi-Fi", wifi)`; a list or table takes `.selection(state)` | [`toggle.rs`](../../agents/recipes/toggle.rs) |
| `@Environment`, `.environment(_:_:)` | `.environment(value)` above, `use_environment::<T>()` below | [`environment.rs`](../../agents/recipes/environment.rs) |
| `.disabled(_:)`, `\.isEnabled` | `.disabled(bool)` on any view, `use_is_enabled()` | [`disabled.rs`](../../agents/recipes/disabled.rs) |
| `@Observable` | a `Copy` struct of `State`s, given as a context | [`observable.rs`](../../agents/recipes/observable.rs) |
| `@FocusState`, `.focused($f, equals:)`, `.defaultFocus` | `use_focus_state::<T>()`, `.focused(state, value)`, `state.set(Some(value))`, `.default_focus(state, value)` | [`focus_state.rs`](../../agents/recipes/focus_state.rs) |
| `ForEach(items, id: \.id)` | `.children(iter.map(..))` and `.key(id)` | [`for_each.rs`](../../agents/recipes/for_each.rs), [`stopwatch.rs`](../../agents/recipes/stopwatch.rs) |
| `List { ForEach(mail) { MailRow($0) } }`, `TableColumn("On") { Toggle(..) }` | `List::new(label, names).row(\|index\| view)`, `Table::cell(column, \|row\| view)` | [`list_rows.rs`](../../agents/recipes/list_rows.rs) |
| `if` and `else` in a view builder | a Rust `if`, `.maybe_child(opt)`, `.into_element()` | [`if_else.rs`](../../agents/recipes/if_else.rs) |
| `let onSave: () -> Void` in a view | a `Callback<A, R>` field, built from any closure | [`callback.rs`](../../agents/recipes/callback.rs) |
| `.commands`, `.keyboardShortcut` on a menu item | `Action`, `Application::commands(Commands::new().menu(..))`, `use_menu_action` | [`actions.rs`](../../agents/recipes/actions.rs) |
| `.keyboardShortcut` on a button or any view, `Button(role:)` | `Button::keyboard_shortcut(Shortcut)`, `.keyboard_shortcut(Shortcut)` of `EnvironmentModifiers`, `Button::role(ButtonRole)` | [`keyboard_shortcut.rs`](../../agents/recipes/keyboard_shortcut.rs) |
| `WindowGroup`, `Window`, `Settings`, `@Environment(\.openWindow)`, `\.dismissWindow` | `Application::scene(..)`, `use_open_window()`, `use_dismiss_window()` | [`scenes.rs`](../../agents/recipes/scenes.rs) |
| `.windowStyle(.hiddenTitleBar)` with a `.toolbar` (no title, items after the lights, no bar fill) | a `Titlebar` with `.shows_title(false)`; SwiftUI draws two adjacent items on one glass, the kit on one `ToolGroup` holding both | [`window_designs.rs`](../../agents/recipes/window_designs.rs) |
| `TabView { … }` in a `Window` | `TabView::new(..).style(\|_\| TabViewStyle::automatic())` in a `HiddenTitleBar` window | none |
| `.inspector(isPresented:content:)` | `.inspector(view, presented)` on any view, or on a `SplitView` | [`inspector.rs`](../../agents/recipes/inspector.rs) |
| `.windowStyle(.hiddenTitleBar)`, `.toolbar { ToolbarItem(placement:) }`, `.windowToolbarStyle(_:)`, `.windowToolbarFullScreenVisibility(.onHover)` | `WindowStyle::HiddenTitleBar` in `.options(..)`, a `Titlebar` with `.toolbar(ToolbarItem::new(placement, view))`, `.toolbar_style(..)`, `.shows_title(..)`, `WindowToolbarFullScreenVisibility::OnHover` ([9](09-titlebars.md#window-designs)) | [`window_designs.rs`](../../agents/recipes/window_designs.rs) |
| `Spacer()` | `Spacer::new()`, `.min_length(f32)` | [`layout_grid.rs`](../../agents/recipes/layout_grid.rs) |
| `.frame(width:height:alignment:)`, `.frame(maxWidth: .infinity)` | `.frame()`, then `.width(..)`, `.height(..)`, `.alignment(..)`; `.width(Size::fill())` for `maxWidth: .infinity` in a column | [`layout_grid.rs`](../../agents/recipes/layout_grid.rs) |
| `Grid { GridRow { .. } }`, `.gridCellColumns(_:)`, `.gridColumnAlignment(_:)` | `Grid::new().row(GridRow::new().child(..))`, `.grid_cell_columns(n)`, `.grid_column_alignment(..)` | [`layout_grid.rs`](../../agents/recipes/layout_grid.rs) |
| `LazyVGrid(columns: [GridItem(.adaptive(minimum:))])` | `LazyVGrid::new(GridItem::adaptive(min))` | none |
| `ZStack(alignment:)`, `.overlay(alignment:content:)`, `.background(alignment:content:)` | `ZStack::new().alignment(..).child(..)`, `.overlay(alignment, content)`, `.background_view(alignment, content)` | [`layout_layers.rs`](../../agents/recipes/layout_layers.rs) |
| `Layout`, `ViewThatFits(in:)` | `Layout::place_subviews` with `LayoutContainer::with_layout(..)`; `ViewThatFits::new(axis).candidate(extent, view)` | [`layout_layers.rs`](../../agents/recipes/layout_layers.rs) |
| `.sheet(isPresented:)`, `.alert(_:isPresented:)`, `.popover(isPresented:)` | `.sheet(state, ..)`, `.alert(state, ..)`, `.popover(state, size, ..)` with a `State<bool>` | [`presentations.rs`](../../agents/recipes/presentations.rs) |
| `.onHover`, `.onTapGesture(count:)` | `.on_hover(\|inside\| ..)`, `.on_tap_gesture(count, \|_\| ..)`; both use the pointer events a gesture uses, so put one on a view without another | [`gestures.rs`](../../agents/recipes/gestures.rs) |
| `.onAppear`, `.onDisappear`, `.onChange`, `.task` | [`use_hook`, `use_drop`, `use_side_effect_with_deps`, `use_future`](#effects) | none |

## State and bindings

`use_state` returns a `State<T>`. It is `Copy`, so a closure takes it by
`move`. `read()` subscribes the view to the value; `peek()` does not. `write()`
and `set` need `let mut`.

A hook runs in the same order on every render. Never call one inside an `if`,
a loop or an iterator closure; a wrong order panics at run time. `.help` is a
hook ([chapter 7](07-menus-and-popovers.md#view-modifiers)).

The kit has no `Binding` type: every control takes the `State` it edits, as a
SwiftUI control takes a binding (`Checkbox::new("Bold", bold)`,
`Slider::new("Volume", volume)`), so a child view receives that `State` as a
field. A list, table, outline or tab view takes it with `.selection(state)`, and
a sidebar row with `.tag(selection, value)`. A part of a `State`, like
`$items[row]`, is not a `State`: keep one `State` per row.

```swift
// ✗ Toggle("Wi-Fi", isOn: $wifi)
```

✓ [`toggle.rs`](../../agents/recipes/toggle.rs): `Switch::new("Wi-Fi", wifi)`.

## Environment

`.environment(value)` puts a value in the environment of a view, and `use_environment::<T>()` reads the nearest one below it. The type is the key, so an app wraps a `bool` or a `String` in its own type. `T` is `Clone + PartialEq + Default`: the default is what a reader gets when no view above provides one.

The scope keeps a `State` and writes the new value in its render, so a view that read the value renders again when it changes: Freya 0.5.3 has no other way to make a context change visible. Put the modifier after the children, as in SwiftUI.

✓ [`environment.rs`](../../agents/recipes/environment.rs).

A context by type is the level below. `use_provide_context` keeps one value for the life of the view, and `use_consume` returns the same value on every later render; it panics when no ancestor provides the type. A value that changes needs a `State` inside it ([observation](#observation)).

A popup, a menu, a tooltip and a sheet are other window runners: they read nothing from the view that opened them.

The built-in keys have their own readers:

| SwiftUI | Kit |
|---|---|
| `\.colorScheme`, contrast, accent | `Palette::current()`, `theme::resolved()`; `use_appearance(..)` overrides them for a subtree ([4](04-appearance-and-tokens.md)) |
| `\.controlSize` | `.control_size(ControlSize)` on a view |
| `\.layoutDirection` | `.layout_direction(LayoutDirection)` on a view; `use_environment::<LayoutDirection>()` reads it ([11](11-views.md#right-to-left)) |
| `\.accessibilityReduceMotion` | `use_reduced_motion()` |
| `\.isEnabled` | `use_is_enabled()`; `.disabled(bool)` on a view ([disabled](#disabled)) |
| `\.font`, `\.foregroundStyle` | set on each element; a `Text` keeps its own color ([`styles.md`](../../agents/styles.md)) |

## Disabled

`.disabled(bool)` takes any view. A control inside it is disabled when its own `.disabled` or any view above it says so, and `.disabled(false)` enables nothing that an outer view disabled. The scope stays in the tree when the flag changes, so what is inside keeps its state.

A disabled control draws and behaves as the one with its own flag: no callbacks, not focusable, announced as disabled. `MenuButton`, `ComboButton`, `HelpLink`, `Link` and the items of `ToggleGroup` stop reacting and are announced as disabled, but are not dimmed: their disabled look is not measured. These views do not follow it yet: the rows of `List`, `Table`, `OutlineView` and `CollectionView`, tabs, sidebar items, the media controls, the indicators, `ImageWell`, `TokenField` and `TextEditor`.

```swift
// ✗ TextField(..).disabled(saving); Button("Save").disabled(saving)   // one call per control
```

✓ [`disabled.rs`](../../agents/recipes/disabled.rs): one `.disabled(saving)` on the stack.

## Observation

`@Observable` tracks each property. The kit does the same by construction: a
view subscribes to the `State` it reads, so a view that reads `count` and not
`step` ignores a change to `step`. Put the states in one `Copy` struct, build
it in the render (its fields are hooks), and give it to `use_provide_context`.

✓ [`observable.rs`](../../agents/recipes/observable.rs).

## Focus

`use_focus_state::<T>()` returns a `FocusState<T>`. It is `Copy`; `T` is your enum of fields (`Clone + PartialEq`). `.focused(state, value)` on a view registers the controls inside it under `value`, so the app moves the focus by its own name, not by a raw id: the text fields (`TextField`, `SecureField`, `SearchField`, `TextEditor`, `ComboBox`, `TokenField`) and every other control the kit draws with `use_interaction`.

- `state.get()` returns the value of the focused control, or `None`, and subscribes the view.
- `state.set(Some(value))` moves the focus to the first control registered under `value`; `state.set(None)` takes it off.
- `.default_focus(state, value)` on the view that holds the fields focuses `value` once, after the view has appeared.

`get` follows the window's focus, so after `set` it changes in the next pass, not at once. The `value` of `.focused` is read when the view mounts: a control whose value changes needs a new `.key(..)`.

✓ [`focus_state.rs`](../../agents/recipes/focus_state.rs).

Below this level, `use_a11y()` returns an id that the view owns. A control that takes the id becomes the focus target, `id.request_focus()` moves the focus to it, and `use_focus(id)` reads its focus state. Only `SearchField` takes an id, with `.a11y_id(id)`.

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

An `Action` is a constant with an id, a title and an optional shortcut. A
`Menu` in `Application::commands` lists its items, and `use_menu_action`
receives its id ([chapter 13](13-app.md#actions-and-shortcuts)). Every handler
of the window receives every id: compare it with the action's `.id`.

```swift
// ✗ .commands { CommandMenu("Counter") { Button("Add one") { count += 1 }.keyboardShortcut("k") } }
```

✓ [`actions.rs`](../../agents/recipes/actions.rs).

A button takes its own shortcut: `Button::keyboard_shortcut(Shortcut)` calls `on_press` when the key is pressed in the window, while the button is enabled. `.keyboard_shortcut(Shortcut)` on any view gives that shortcut to every `Button` inside it that has none of its own, as SwiftUI's environment does; `Shortcut::DEFAULT_ACTION` and `CANCEL_ACTION` are Enter and Escape. `Button::role(ButtonRole::Destructive)` draws the destructive look. A key that a menu item of the window carries goes to the menu and never reaches the button, and a shortcut without a modifier also fires while a text field has the focus.

✓ [`keyboard_shortcut.rs`](../../agents/recipes/keyboard_shortcut.rs).

## Effects

| SwiftUI | Kit | When it runs |
|---|---|---|
| `.onAppear` | `use_hook(\|\| ..)` | in the first render of the view |
| `.onDisappear`, `deinit` | `use_drop(\|\| ..)` | when the view leaves the tree |
| `.onChange(of:)` | `use_side_effect_with_deps(&value, ..)`; the old value from `use_previous_and_current` | when the value changes, and once at creation |
| `.task`, `.task(id:)` | `use_future(..)`; a `State` read outside the async block restarts it | cancelled when the view leaves the tree |
| `TimelineView` | `use_timeline(active, every)` | every `every` while `active` ([`stopwatch.rs`](../../agents/recipes/stopwatch.rs)) |
| `NavigationStack(path:)`, `NavigationLink(value:)` | `NavigationStack::new(path, root, destination)` and `use_navigation_path::<R>()` | pages that push and pop ([`navigation_stack.rs`](../../agents/recipes/navigation_stack.rs)) |
| `withAnimation(.spring(duration:bounce:)) { … }` | `with_animation(Animation::Spring { duration_ms, bounce }, \|\| …)` and `use_animated` | the changes that follow use the animation ([`with_animation.rs`](../../agents/recipes/with_animation.rs)) |

The hooks are in `freya-core` 0.5.3, `src/lifecycle/` and `src/hooks/`.

## What the kit does not do

[ADR 0085](../decisions.md) fixes these as non-goals until an app asks for one; the kit builds the close veto, the quit and the open-URL event for documents ([Documents and file dialogs](08-windows.md#documents-and-file-dialogs)).

| SwiftUI or AppKit | In the kit |
|---|---|
| a `Binding` type, `$model.field` | none; pass the `State` ([State and bindings](#state-and-bindings)) |
| a command sent to the focused view (the responder chain) | none; every handler of the window gets the id |
| `windowShouldClose` | `use_document` vetoes the close of an edited window and asks "Save changes?" ([8](08-windows.md#documents-and-file-dialogs)); any other close request closes the window at once, except a sheet's |
| `applicationShouldTerminate`, `NSApp.terminate` | none to call; the last window closing ends the process. The Quit command (`QUIT`) closes every window ([13](13-app.md#quit)); Cmd-Q on macOS asks for the first edited document first |
| `.onOpenURL`, `application(_:open:)` | `.on_open_url(\|OpenedUrl\| ..)` on macOS; none on Linux |

The full audit of Swift syntax against the kit, with probes, is in
[Swift syntax in Rust](../research/swift-syntax-in-rust.md).
