# Recipes

**Goal:** start a program from a whole `src/main.rs` that already compiles.
**Scope:** one program per common task. Each item's reference lives in its
chapter; the names are in the [glossary](../docs/barrierkit/20-glossary.md).
**Reader:** an AI agent sent here by [`prompt.md`](prompt.md), or a person.
**Input:** a task. **Output:** the closest program, ready to change.

## Steps

1. Find the recipe closest to the task in the list below.
2. Copy its file from [`recipes/`](recipes/) into `src/main.rs`. Each file is
   an example of `bk`, so `./dev.sh check` builds and lints it; inside the
   kit, `cargo run -p bk --example <name>` opens it. There `.cargo/config.toml`
   forces `MALAYSIA_CONFIG_DIR` to `crates/bk/tests/config`, so it opens in
   Spanish, without your appearance, and saves its settings in that folder.
3. Change only what the task needs.
4. Build it inside `nix develop` and go through the [checklist](checklist.md).

## 1. Text in a window

SwiftUI: `WindowGroup { Text("Hello, world").font(.title).padding(40) }`

The program is [`recipes/hello_world.rs`](recipes/hello_world.rs); [chapter 2](../docs/barrierkit/02-first-app.md#hello-world) explains it line by line.

## 2. State and a button

SwiftUI: `@State var count = 0` with `VStack { Text("\(count)"); Button("Add one") { count += 1 } }`

The program is [`recipes/counter.rs`](recipes/counter.rs); [chapter 2](../docs/barrierkit/02-first-app.md#adding-state) explains it line by line.

## 3. Rows and columns

SwiftUI: `VStack(alignment: .leading, spacing: 4) { HStack { … } }`

The program is [`recipes/rows_and_columns.rs`](recipes/rows_and_columns.rs).

## 4. A toggle bound to state

SwiftUI: `Toggle("Wi-Fi", isOn: $wifi)`

The program is [`recipes/toggle.rs`](recipes/toggle.rs).

## 5. A settings form

SwiftUI: `Form { Section("General") { Toggle("Sounds", isOn: $sounds) } }`

The program is [`recipes/settings_form.rs`](recipes/settings_form.rs).

For a real preferences window use `SettingsWindow` (chapter
[12](../docs/barrierkit/12-patterns.md)).

## 6. A reusable view

SwiftUI: `struct Greeting: View { let name: String; var body: some View { … } }`

The program is [`recipes/reusable_view.rs`](recipes/reusable_view.rs).

A `Component` must be `PartialEq`: the kit redraws it only when its fields
change. A handler field is a `Callback`
([recipe 22](#22-a-view-that-takes-a-handler)), which never compares equal, so a
component that holds one redraws with its parent every time; a bare closure
field does not compile under `#[derive(PartialEq)]`.

## 7. Drawing the whole window yourself

Use it when the titlebar needs a sidebar or toolbar items. `run_window` takes
the same closure but adds nothing: you draw the background and the titlebar.

The program is [`recipes/custom_window.rs`](recipes/custom_window.rs).

## 8. Translated text

Follow the steps in chapter [17](../docs/barrierkit/17-languages.md); then every
literal above becomes `tr!("id")`, as in `Text::new(tr!("hello"))`.

## 9. A stopwatch

SwiftUI: a `Text` fed by `TimelineView(.periodic(from:by:))` with Start/Stop and Lap/Reset buttons.

The program is [`recipes/stopwatch.rs`](recipes/stopwatch.rs).

`use_timeline(active, every)` renders the window again every `every` while
`active` is true ([8](../docs/barrierkit/08-windows.md)), so `Instant::now()`
in the render is the time of each tick.

## 10. Buttons in the title bar over a gradient

SwiftUI: `.toolbar { ToolbarItemGroup { … } }` over a `LinearGradient` background.

The program is [`recipes/titlebar_over_gradient.rs`](recipes/titlebar_over_gradient.rs).

1. The gradient is a [`Backdrop`](../docs/barrierkit/15-backdrops-and-media.md#backdrop),
   not a hand-written `LinearGradient`: it gives the content a legible palette
   and makes the `Titlebar` transparent. Its colors come from a
   `BACKDROP_SCENES` preset or from an image (`BackdropSource::Image`).
2. The content is a `Component`, so it reads the backdrop's palette.
3. `run_window` is used because the window draws its own title bar with
   buttons ([8](../docs/barrierkit/08-windows.md)). Every kit window is
   client-side decorated.

## 11. Style one control

SwiftUI: a custom `ButtonStyle` given with `.buttonStyle(_:)` to one view, a stack or the whole app.

The program is [`recipes/styled_control.rs`](recipes/styled_control.rs).

1. The theme makes every slider square and every button glass
   (`ButtonStyle::glass` is a preset: a function of the style, like SwiftUI's
   `.glass`); the stack turns its two sliders
   vertical; `.style` paints one fill red, grey while pressed, and gives the
   `LevelSlider` the `fat` preset, the Control Center bar. Each override names only what
   changes and ends with `..s`. `Theme::style` and `.control_style` take any
   control's style: the parameter's type (`|s: SliderStyle|`) picks it.
2. `Role` colors follow light, dark and contrast; the kit still applies
   reduce motion, reduce transparency and the focus ring.
3. The levels, fields and value types are in [`styles.md`](styles.md).

## 12. Controls at every size

SwiftUI: `.controlSize(.mini)`, `.small`, `.regular`, `.large` or `.extraLarge` on a button, a toggle or a slider.

The program is [`recipes/control_sizes.rs`](recipes/control_sizes.rs).

`.control_size(ControlSize)` on a view sizes every control inside it, like SwiftUI's
`.controlSize(_:)`; `.size(ControlSize)` on one control wins over it. Both take one of five
sizes on `Button`, `PopUpButton`, `SegmentedControl`, `Checkbox`, `RadioGroup`, `Switch`,
`Slider`, `Stepper`, `TextField`, `SearchField`, `ComboBox` and `DatePicker`; the sizes are in
[chapter 5](../docs/barrierkit/05-controls.md#control-size).

## 13. Modifiers on controls

SwiftUI: `Button("Save") { }.padding(8).help("Save the file")` inside an `HStack`.

The program is [`recipes/control_modifiers.rs`](recipes/control_modifiers.rs).

1. Every control takes `.padding`, `.margin`, `.width`, `.background`,
   `.corner_radius`, `.opacity`, `.help` and `.context_menu`, in any order
   ([chapter 5](../docs/barrierkit/05-controls.md#modifiers-on-controls)).
2. A modifier sets a property on that one node; it does not wrap it. Outer
   space is `.margin`: a second `.padding` replaces the first.
3. `.help` uses a hook: call it on every render, never inside a condition or
   a loop.

## 14. A button with a view label

SwiftUI: `Button(action: add) { Label("Add", systemImage: "plus") }`.

The program is [`recipes/button_label.rs`](recipes/button_label.rs).

1. `.label(|| view)` takes a closure that returns the view; it replaces the
   drawn text. The string given to `Button::new` stays the accessible name
   ([chapter 5](../docs/barrierkit/05-controls.md#buttons)).
2. Inside the closure, `Text`, `Label` and `label()` take the button's font and
   the ink of its current state, so the symbol and the title turn white on the
   accent fill of `bordered_prominent`.
3. Use `Label` for a symbol with a title; a bare `Image::system` keeps its own
   color.

## 15. State passed to a child view

SwiftUI: `struct NameField: View { @Binding var name: String … }` used as `NameField(name: $name)`.

The program is [`recipes/binding.rs`](recipes/binding.rs).

1. A text field takes the `State<String>` it edits. The child holds that
   `State` as a field, and the parent reads the same value.
2. Any other control takes the value and a callback, as in
   [recipe 4](#4-a-toggle-bound-to-state). The kit has no `Binding`
   ([ADR 0085](../docs/decisions.md)).

## 16. A value from the environment

SwiftUI: `.environment(\.account, account)` above and `@Environment(\.account) var account` below.

The program is [`recipes/environment.rs`](recipes/environment.rs).

1. The value is a type that is `Clone + PartialEq + Default`; the type is the
   key. `.environment(value)` on a view gives it to everything inside, and
   `use_environment::<T>()` below returns the nearest one, or `T::default()`.
2. A reader renders again when the value changes. A model with several fields
   holds a `State` for each ([recipe 17](#17-observable-state)).

## 17. Observable state

SwiftUI: `@Observable class Counter { var count = 0; var step = 1 }` shared through the environment.

The program is [`recipes/observable.rs`](recipes/observable.rs).

1. The model is a `Copy` struct of `State`s. Build it in the render, then give
   it to `use_provide_context`.
2. A view re-renders for the `State`s it reads: `CountLabel` reads `count`, and
   a change to `step` does not touch it.

## 18. Moving the focus

SwiftUI: `@FocusState var field: Field?`, `.focused($field, equals: .email)` and `field = .email`.

The program is [`recipes/focus_state.rs`](recipes/focus_state.rs).

1. `use_focus_state::<Field>()` returns a `FocusState`; `.focused(focus, value)`
   registers the controls inside a view under a value of your enum.
2. `focus.set(Some(value))` moves the focus, `focus.set(None)` takes it off and
   `focus.get()` is the focused value. `.default_focus(focus, value)` on the
   view that holds the fields focuses one of them once.

## 19. A list from data

SwiftUI: `ForEach(fruits, id: \.id) { Text($0.name) }`

The program is [`recipes/for_each.rs`](recipes/for_each.rs).

1. `.children(iter)` takes any iterator; `.key(id)` gives each child its
   identity, so its state follows the item when the list changes.
2. A duplicate key among siblings panics in debug builds.

## 20. Views that depend on a condition

SwiftUI: `if signedIn { Text("Welcome back") } else { Button("Sign in") { … } }`

The program is [`recipes/if_else.rs`](recipes/if_else.rs).

1. Two arms of different types need `.into_element()`.
2. `.maybe_child(opt)` adds a child only when the option is `Some`.

## 21. A command with a shortcut

SwiftUI: `.commands { CommandMenu("Counter") { Button("Add one") { … }.keyboardShortcut("k") } }`

The program is [`recipes/actions.rs`](recipes/actions.rs).

1. The `Action` is a constant. Its item goes in the `MainMenu`, and the
   shortcut works only through an enabled menu item
   ([chapter 13](../docs/barrierkit/13-app.md#actions-and-shortcuts)).
2. Every handler of the window receives every id, so the handler compares the
   id with `ADD.id`.
3. A button takes its own shortcut: `Button::keyboard_shortcut(Shortcut)`, with no menu
   item ([`recipes/keyboard_shortcut.rs`](recipes/keyboard_shortcut.rs)). A menu
   item with the same key wins.

## 22. A view that takes a handler

SwiftUI: `struct SaveBar: View { let onSave: () -> Void; var body: some View { Button("Save", action: onSave) } }`

The program is [`recipes/callback.rs`](recipes/callback.rs).

1. The field is a `Callback<A, R>`. It is `Clone`, and a constructor that takes
   `impl Into<Callback<A, R>>` accepts any closure.
2. `call(arg)` runs it. It never compares equal, so `SaveBar` redraws with its
   parent ([recipe 6](#6-a-reusable-view)).
3. A closure that writes a `State` takes it by `move`, and the `State` needs
   `let mut`.

## 23. Views in list rows and table cells

SwiftUI: `List { ForEach(mail) { MailRow($0) } }` and `TableColumn("Enabled") { Toggle(..) }`.

The program is [`recipes/list_rows.rs`](recipes/list_rows.rs).

1. `List::new(label, names)` keeps the row count and the accessible names;
   `.row(|index| view)` draws each row as a view. `Table::cell(column,
   |row| view)` does it for one column; the other columns draw their strings
   ([chapter 11](../docs/barrierkit/11-views.md#views-in-rows-and-cells)).
2. Inside the closure, `Text` takes the font and the ink of the row, so the
   title turns white on a selected row. `Avatar` and `Badge` keep their own
   colors.
3. A control stops the press so the row does not select: wrap it in a `rect()`
   with `.on_press(|event: Event<PressEventData>| event.stop_propagation())`.
4. Rows have a fixed height: set `TableStyle::row_height` for a taller row. The
   closure never compares equal, so the list redraws with its parent.

## 24. Disabling a group of controls

SwiftUI: `VStack { TextField(..); Button("Save") { } }.disabled(saving)`.

The program is [`recipes/disabled.rs`](recipes/disabled.rs).

1. `.disabled(bool)` goes after the children and takes the whole view; every
   control inside is disabled when its own flag or a view above it is.
2. The flag can change on every render: what is inside keeps its state.

## 25. Several windows

SwiftUI: `WindowGroup { }`, `Window("Inspector", id: "inspector") { }`, `Settings { }` and `@Environment(\.openWindow) var openWindow`.

The program is [`recipes/scenes.rs`](recipes/scenes.rs).

1. `.window(..)` is the window that opens at launch. `.scene(..)` adds a
   `Window`, a `WindowGroup` or `Settings`
   ([chapter 8](../docs/barrierkit/08-windows.md#scenes)).
2. `use_open_window()` goes at the top of a component of a scene window. Its
   `open(id)` opens the scene with that id, or fronts it when it is a `Window`
   that is already open; `open_value(7)` opens the `WindowGroup::for_value`
   window with that value.
3. `use_dismiss_window()` closes this window (`dismiss()`) or every window of an
   id (`dismiss_id(id)`).
4. `.commands(Commands::new())` adds the standard menus, with a Window menu
   that lists the open windows and a Settings… item
   ([chapter 13](../docs/barrierkit/13-app.md#standard-menus)).

## 26. A grid of labeled rows and a toolbar

SwiftUI: `Grid(alignment: .leading) { GridRow { Text("Name").gridColumnAlignment(.trailing); Text(name) } }`, then `HStack { Button("Reset") { }; Spacer(); Button("Save") { } }`.

The program is [`recipes/layout_grid.rs`](recipes/layout_grid.rs).

1. A `GridRow` holds the cells of one row. Every column is as wide as its widest
   cell and every row as tall as its tallest ([chapter 11](../docs/barrierkit/11-views.md#layout)).
2. `.grid_column_alignment(..)` on one cell aligns its whole column;
   `Grid::alignment` places the other cells.
3. `Spacer::new()` takes the space the other children leave, so the two buttons
   sit at the ends of the row.
4. `.frame()` then `.width(..)` and `.alignment(..)` is
   `.frame(maxWidth: .infinity, alignment: .leading)`.

## 27. Glass over scrolling content

SwiftUI: `ZStack(alignment: .top) { List { … }; Color.clear.frame(height: 52).background(.regularMaterial) }`, with `.scrollEdgeEffectStyle(.soft, for: .top)`.

The program is [`recipes/glass_window.rs`](recipes/glass_window.rs).

1. `TopGlass::new()` is the first child of the element that holds the title
   bar, and the title bar is its second child: that element is drawn over the
   glass. `Material::new(12.0)` is the first child of the card it sits behind.
2. A compositor that offers `infinity_window_style_v1` 10 blurs what is under
   them and the kit draws no blur; any other makes the kit draw an opaque solid
   ([chapter 15](../docs/barrierkit/15-backdrops-and-media.md#who-draws-the-blur)).
3. Do not make the window's only top-level element hold the glass: it needs
   content beside it to blur.
4. To see the compositor's blur, run it under Malaysia as
   [chapter 18](../docs/barrierkit/18-platform.md#glass-over-the-windows-content)
   says.

## 28. A window with its own frame on macOS

SwiftUI: `WindowGroup { NavigationSplitView { List { … } } detail: { … } }` with `.windowStyle(.plain)` and a `.toolbar`.

The program is [`recipes/notes_sidebar.rs`](recipes/notes_sidebar.rs).

1. `WindowOptions::window_style(WindowStyle::Plain)` makes the window borderless
   on macOS, so the `Titlebar` and its traffic lights are the kit's. Linux
   ignores it; the compositor decides there.
2. `Sidebar::traffic_lights(true)` holds the lights while the sidebar shows, and
   `Titlebar::with_sidebar(true)` puts the title at the start of the detail.
   [`custom_window`](recipes/custom_window.rs) and
   [`titlebar_over_gradient`](recipes/titlebar_over_gradient.rs) use the same
   option.

## 29. A toolbar window without a sidebar

SwiftUI: `NavigationStack { List { Label(..) } .navigationTitle("Tasks").toolbar { ToolbarItemGroup { Button(..) } } }`.

The program is [`recipes/tasks_list.rs`](recipes/tasks_list.rs).

1. A `Titlebar` with items is the unified toolbar: 52 high, the lights at their
   place and the title at the start. `.toolbar_style(WindowToolbarStyle::UnifiedCompact)`
   gives the compact bar with a centered title.
2. A `ToolGroup` of two items is a 76 wide capsule and one item is a 38 circle;
   no divider, as in SwiftUI.

## 30. A change that animates only when you say so

SwiftUI: `Button("Show") { withAnimation(.smooth) { expanded.toggle() } }` and a `frame(height:)` that follows `expanded`.

The program is [`recipes/with_animation.rs`](recipes/with_animation.rs).

1. `use_animated(target, Animation::None)` returns the value that follows
   `target`. With `Animation::None` as its default, it moves only inside a
   transaction.
2. `with_animation(animation, || expanded.toggle())` tags the change with
   `animation`: here the spring of a sheet (`SPRING_SHEET_MS`, `bounce` 0).
   Every `use_animated` whose target changes right after it uses that
   animation, and reduced motion turns it into 0 ms.
3. A change made again while the value moves goes on from the value and the
   velocity it shows.

## 31. Children that fade as one, and a shader that moves

SwiftUI: `ZStack { Circle(); Circle() }.compositingGroup().opacity(0.5)` and `.layerEffect(ShaderLibrary.pulse(.float(time)), maxSampleOffset: .zero)` inside a `TimelineView`.

The program is [`recipes/compositing_group.rs`](recipes/compositing_group.rs).

1. `CompositingGroup::new(effect).child(..)` draws its children into one layer and
   applies `effect` to the layer: the two discs at `GroupEffect::Opacity(0.5)` show
   no darker seam where they overlap.
2. `GroupEffect::ScrollEdge { depth, progress, dark }` is the pocket under a
   toolbar where content scrolls: a blur and a tone over the first `depth`
   points, shown by `progress`. The button moves `progress` with a spring.
3. `GroupEffect::Shader` runs an SkSL image filter. `uniform shader content;`
   receives the layer. A uniform named `time` is the seconds since the group
   appeared, and the group redraws every frame while the shader has it.

## 32. Pages that push and pop

SwiftUI: `NavigationStack(path: $path) { Home() .navigationDestination(for: Route.self) { Detail($0) } }` with `NavigationLink(value:)`.

The program is [`recipes/navigation_stack.rs`](recipes/navigation_stack.rs).

1. The app owns the route type and a `State<NavigationPath<Route>>`.
   `NavigationStack::new(path, root, |route| page)` draws the root and one page per route.
2. `use_navigation_path::<Route>()` gives any page below the stack the path, so a
   button pushes (`path.write().push(..)`) or pops without a callback chain.
3. A push moves the new page in from the right edge and the old one 30 % of the
   width to the left, dimmed, on the spring measured on Mac Catalyst
   ([UIKit animations](../docs/research/uikit-animations.md#2-motion-table)). Reduced motion
   shows the page at once. Wrap the change in `with_animation` to use another
   animation ([14](../docs/barrierkit/14-animations.md#implicit-animation)).
