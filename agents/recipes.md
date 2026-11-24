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
   kit, `cargo run -p bk --example <name>` opens it.
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

Use it when the titlebar needs a sidebar or toolbar items. `WindowStyle::Plain`
or `HiddenTitleBar` in `.options(..)` leaves out the kit `Titlebar`, so you draw
your own ([ADR 0115](../docs/decisions.md#0115--standard-menus-by-default-edit-items-reach-the-focused-text-field-macos-menus-are-appkits)).

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
3. `WindowStyle::Plain` leaves out the kit `Titlebar`, because the window
   draws its own with buttons ([ADR 0115](../docs/decisions.md#0115--standard-menus-by-default-edit-items-reach-the-focused-text-field-macos-menus-are-appkits)). Every kit window is
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

1. The `Action` is a constant. Its item goes in a `Commands` menu, after the
   standard ones, and the shortcut works only through an enabled menu item
   ([chapter 13](../docs/barrierkit/13-app.md#actions-and-shortcuts)).
2. Every handler of the window receives every id, so the handler compares the
   id with `ADD.id`.
3. A button takes its own shortcut: `Button::keyboard_shortcut(Shortcut)`, with no menu
   item ([`recipes/keyboard_shortcut.rs`](recipes/keyboard_shortcut.rs)). A menu
   item with the same key wins.
4. `.keyboard_shortcut(Shortcut)` on a view gives the shortcut to every button
   inside it that has none; `Shortcut::DEFAULT_ACTION` and `CANCEL_ACTION` are
   Enter and Escape. `.role(ButtonRole::Destructive)` makes a button red.

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
5. `.list_row_background`, `.list_row_separator` and `.list_row_insets` take a
   closure of the row index ([chapter 11](../docs/barrierkit/11-views.md#list-row-modifiers)).

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
   gives the compact bar with a centered title, and `Expanded` a title row over the toolbar row.
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
2. `NavigationLink::new(label, route)` is a button that pushes `route`. `use_navigation_path::<Route>()`
   gives any page below the stack the path, to pop (`path.write().pop()`) or push from code.
3. A push moves the new page in from the right edge and the old one 30 % of the
   width to the left, dimmed, on the spring measured on Mac Catalyst
   ([UIKit animations](../docs/research/uikit-animations.md#2-motion-table)). Reduced motion
   shows the page at once. Wrap the change in `with_animation` to use another
   animation ([14](../docs/barrierkit/14-animations.md#implicit-animation)).

## 33. A tap, a long press and a drag on one view

SwiftUI: `Rectangle().gesture(LongPressGesture().exclusively(before: TapGesture())).simultaneousGesture(DragGesture().updating($offset) { value, state, _ in state = value.translation.width })`.

The program is [`recipes/gestures.rs`](recipes/gestures.rs).

1. `TapGesture`, `LongPressGesture`, `DragGesture`, `MagnifyGesture` and `RotateGesture` take `.on_changed` and
   `.on_ended`; `.updating(state, |value, state| ..)` is `@GestureState`: it changes while the
   gesture does and goes back when it ends ([`use_gesture_state`]).
2. `.gesture(g)` attaches one gesture, `.simultaneous_gesture(g)` one that never fails the others and
   `.high_priority_gesture(g)` one that beats those of the views below. A node keeps one handler per
   event: put several gestures on a view with `a.exclusively(b)` (`a` first: while it can still begin,
   `b` waits) or `a.simultaneously(b)`, not with two `.gesture` calls.
3. The deepest view wins a pointer. A tap that has moved 10 points is no tap; a long press fires after
   500 ms in place. Two fingers (two `finger_id`s) drive `MagnifyGesture` and `RotateGesture`; a trackpad
   pinch needs the runner to map it.

## 34. An alert and a popover driven by state

SwiftUI: `.alert("Delete this file?", isPresented: $deleting) { Button("Delete", role: .destructive) { .. } }` and `Button("Info") { info = true }.popover(isPresented: $info) { Text("Updated just now").padding() }`.

The program is [`recipes/presentations.rs`](recipes/presentations.rs).

1. `.alert(state, || Alert::new(..))` on a container shows the alert while the `State<bool>` is `true`;
   any button, Enter or Escape writes `false` after the callback. `.sheet(state, || content)` works the same,
   but only the app closes a sheet.
2. `.popover(state, size, || content)` opens a popover that points at the view it sits on, so put it on a
   `rect()` around the button, not on the container. A click outside writes `false`. It uses a hook: call it on every render.

## 35. A toolbar by placement

SwiftUI: `.toolbar { ToolbarItem(placement: .navigation) { .. }; ToolbarItem(placement: .principal) { Text(page) }; ToolbarSpacer(.flexible); ToolbarItem(placement: .primaryAction) { Button("New") { .. } } }`.

The program is [`recipes/toolbar_items.rs`](recipes/toolbar_items.rs).

1. `Titlebar::toolbar(content)` takes one `ToolbarItem::new(placement, view)` or one `ToolbarSpacer`
   per call and lays it by placement: `Navigation` after the sidebar toggle, `Principal` and `Status` in the
   title's place, `PrimaryAction` and `Automatic` on the right.
2. A `ToolbarSpacer::flexible()` sends the `Automatic` items before it to the navigation side;
   `ToolbarSpacer::fixed()` adds one gap. The bar does not measure its items: there is no overflow menu.

## 36. Three columns

SwiftUI: `NavigationSplitView { sidebar } content: { notes } detail: { note }`.

The program is [`recipes/three_columns.rs`](recipes/three_columns.rs).

1. `SplitView::content(view)` adds the middle column. It has the sidebar's grip and limits, and only the
   sidebar hides.
2. The `Titlebar` goes in the detail. The content column has no bar: leave `sidebar_bar_height()` free at
   its top.

## 37. Scroll to a position, to an id and to an edge

SwiftUI: `ScrollView { ForEach(rows) { Text($0.name).id($0.id) } }.scrollPosition($position)` with `position = .init(edge: .top)`, `position.scrollTo(id: "row-40", anchor: .center)` and `.onScrollGeometryChange(for: CGFloat.self, of: \.contentOffset.y) { offset = $1 }`.

The program is [`recipes/scroll_position.rs`](recipes/scroll_position.rs).

1. `.id("row-40")` names a view inside the `ScrollView`; the id is a `String`.
2. `ScrollView::scroll_position(state)` takes a `State<ScrollPosition>`. Set it to `ScrollPosition::edge(..)`,
   `::at(x, y)` or `::id(id, anchor)` and the view scrolls; wrap the set in `with_animation(..)` to animate.
   When the user scrolls, the state becomes `ScrollPosition::at(..)` and `is_positioned_by_user()` is true.
3. `ScrollController::scroll_to(position, animation)` does the same from a button, with the `Animation` as
   an argument. `.on_scroll_geometry_change(..)` and `.on_scroll_phase_change(..)` report the offset and the phase.

## 38. Shapes, fills, borders and clips

SwiftUI: `Circle().fill(.tint)`, `RoundedRectangle(cornerRadius: 18).strokeBorder(.tint, lineWidth: 4)` and `LinearGradient(..).clipShape(.capsule)`.

The program is [`recipes/shapes.rs`](recipes/shapes.rs).

1. `Rectangle`, `RoundedRectangle::new(radius)`, `Capsule`, `Circle`, `Ellipse` and `Path` fill their
   frame: size them with a parent `rect()` or `.width(..)` and `.height(..)`. `.fill(ShapeStyle)` and
   `.stroke_border(ShapeStyle, width)` draw; a `Path` has `.stroke(..)` instead, and `move_to`, `add_line`,
   `add_curve` and `close_subpath` build it.
2. `.clip_shape(Shape::Circle)` (also `Capsule` and `RoundedRectangle { .. }`) clips a view and its
   children. An ellipse or a path cannot clip.

## 39. Full screen, with each window style

SwiftUI: the green light, `View > Enter Full Screen`, and `NSWindow.didEnterFullScreenNotification`.

The program is [`recipes/full_screen.rs`](recipes/full_screen.rs). It takes `title-bar`, `hidden-title-bar` or `plain`,
then `unified`, `compact` or `expanded` for its `WindowToolbarStyle`: SwiftUI's `.toolbar { }` with `.windowToolbarStyle(..)`.

1. `WindowControls::get().status.read().fullscreen` is true while the window is in full screen, whichever way it
   got there: the native green light of a `TitleBar` or `HiddenTitleBar` window, the kit's own light of a `Plain`
   one, or `toggle_fullscreen()`. Read it to change the layout; do not keep a copy.
2. `scripts/macos-fullscreen.sh` runs the three styles beside the same SwiftUI windows and fails when the kit
   does not enter, return or tell the app. A SwiftUI `.plain` window does not enter full screen; the kit's does.

## 40. A view that animates in and out

SwiftUI: `if shown { Text("Hello").transition(.move(edge: .leading).combined(with: .opacity)) }` with `withAnimation { shown.toggle() }`, and `Text("\(count)").contentTransition(.opacity)`.

The program is [`recipes/transitions.rs`](recipes/transitions.rs).

1. Put the view that comes and goes in `Presence::new(condition.then(|| view))` and give it `.transition(..)`:
   `Transition::OPACITY`, `SCALE`, `scale(s)`, `offset(x, y)` and `move_edge(Edge)`, joined with `.combined(..)` or
   `Transition::asymmetric(insertion, removal)`.
2. Change the state inside `with_animation(..)`. Without it the view comes and goes at once. The removed view
   stays mounted until its animation ends.
3. `.content_transition(ContentTransition::Opacity)` on any view fades the old one out and the new one in when it
   changes, also inside `with_animation(..)`. `matchedGeometryEffect` and `numericText` are not built.

## 41. Glass behind any view

SwiftUI: `Text("Capsule").padding().glassEffect(.regular, in: .capsule)`.

The program is [`recipes/glass_effect.rs`](recipes/glass_effect.rs).

1. `.glass_effect(Glass::REGULAR, Shape::Capsule)` puts glass behind the view; `Glass::CLEAR` is the clear
   variant, and `Shape::RoundedRectangle { .. }` and `Shape::Circle` work too.
2. On macOS 26 and later the glass is the system's. Elsewhere it is the kit's blur, opaque under Reduce Transparency.
3. A window draws at most 8 such glasses on macOS (3 on Linux). Use `GlassContainer` for glasses that merge.

## 42. Text with runs, Markdown and selection

SwiftUI: `Text("Total: ") + Text("42 items").bold()`, `Text("**Bold**, _italic_ and a [link](https://example.org)")` and `Text("Copy me").textSelection(.enabled)`.

The program is [`recipes/text_runs.rs`](recipes/text_runs.rs).

1. `Text + Text` is one paragraph; each operand keeps its own `bold()`, `italic()`, `underline()`, `strikethrough()`,
   `font(..)` and `foreground_style(..)`. A modifier on the sum reaches what the operands did not set.
2. `Text::markdown(..)` reads inline Markdown; `Text::new(..)` shows its string verbatim (SwiftUI reads a literal as Markdown,
   a `String` variable as plain: the kit asks for it by name). A link opens in the browser.
3. `.text_selection(true)` lets the reader select and copy. A link of a selectable text does not open.

## 43. Copy, paste and drag typed data

SwiftUI: `NSPasteboard.general.setString(_, forType: .string)`, `.draggable(transferable)` and a `Transferable` type with `DataRepresentation(contentType: .png)`.

The program is [`recipes/pasteboard.rs`](recipes/pasteboard.rs).

1. `DragPayload` is the typed value: `Text`, `Html { text, html }`, `Image(png bytes)` and `Files(paths)`.
2. `Pasteboard::general().copy(&value)` replaces the clipboard with any `Transferable` (`String`, `Vec<PathBuf>`, `DragPayload`, or a
   type of the app) and returns false when the platform cannot hold it (images and files on Linux).
   `paste::<T>()` reads the richest item as a `T`.
3. `use_drag_source().attach(element, || value.payload())` drags the same payload to another app. On macOS try it
   against TextEdit (rich text), Preview or Finder (an image, files).

## 44. Layers, a flow layout and a view that fits

SwiftUI: `ZStack(alignment: .bottomLeading) { card; Text("Caption") }`, `Image(systemName: "bell").overlay(alignment: .topTrailing) { badge }`, a custom `Layout`, and `ViewThatFits(in: .horizontal) { full; compact }`.

The program is [`recipes/layout_layers.rs`](recipes/layout_layers.rs).

1. `ZStack::new().alignment(..).child(..)` draws its children on top of one another;
   `.overlay(alignment, content)` and `.background_view(alignment, content)` do the same
   around one view, which alone sets the size ([chapter 11](../docs/barrierkit/11-views.md#zstack-overlay-and-background_view)).
2. A `Layout` returns a `Placement` (the size and one origin per subview) from the space offered and
   the size of each subview; `LayoutContainer::with_layout(layout)` is its view.
3. `ViewThatFits::new(Axis::Horizontal).candidate(extent, view)` shows the first view whose
   `extent` fits the width offered, the last when none does. The extent is yours: nothing is measured.
4. Put the largest child first: the parent sizes the container by its first child.

## 45. A right-to-left interface

SwiftUI: `ContentView().environment(\.layoutDirection, .rightToLeft)`.

The program is [`recipes/right_to_left.rs`](recipes/right_to_left.rs).

1. `.layout_direction(LayoutDirection::RightToLeft)` mirrors the stacks, frames, grids, `ZStack`s, texts and
   `Form` rows below it, and sets the layout direction of the six control styles
   ([chapter 11](../docs/barrierkit/11-views.md#right-to-left)).
2. A view built inside a component under it reads the direction as it is built; one built before it is
   mirrored once by the modifier.
3. Padding is physical: swap `left` and `right` yourself. `List`, `Table` and the window chrome do not mirror yet.

## 46. Accessibility labels, hints and groups

SwiftUI: `Button("+") { }.accessibilityLabel("Add a task").accessibilityHint("Adds one to the list")`.

The program is [`recipes/accessibility.rs`](recipes/accessibility.rs).

1. `AccessibilityModifiers` (in the prelude) gives every view and control `.accessibility_label`, `.accessibility_hint`,
   `.accessibility_value`, `.accessibility_hidden`, `.accessibility_identifier`,
   `.accessibility_element(AccessibilityChildBehavior::Combine)` and `.accessibility_add_traits(AccessibilityTraits::IS_HEADER)`
   ([chapter 5](../docs/barrierkit/05-controls.md#accessibility-modifiers)).
2. On macOS the window publishes the tree to VoiceOver and the AX API; read it with `axdump <pid>`. The label is AXTitle.
3. A `TextField` is named by its label, else its placeholder; its typed text is the value.

## 47. Choose files in a sheet of the window

SwiftUI: `.fileImporter(isPresented: $importing, allowedContentTypes: [.png], allowsMultipleSelection: true) { .. }`.

The program is [`recipes/file_dialogs.rs`](recipes/file_dialogs.rs).

1. `.file_importer(State<bool>, vec![ContentType::extension("png")], multiple, |paths| ..)` and
   `.file_exporter(State<bool>, "Name.png", types, |path| ..)` open `NSOpenPanel` and `NSSavePanel` as a sheet of the window.
   The closure runs only when the user chose; the app writes the file.
2. `.file_dialog_default_directory(path)` wraps a view that holds a component using the modifiers: the environment
   is read by the components below it ([chapter 8](../docs/barrierkit/08-windows.md#documents-and-file-dialogs)).
3. On Linux the portal ignores the parent, the filters and the start folder.

## 48. A document window that asks before it closes

SwiftUI: `DocumentGroup(newDocument: Notes()) { .. }` and `.onOpenURL { url in .. }`.

The program is [`recipes/documents.rs`](recipes/documents.rs).

1. `use_document(|path| write(path))` makes the window a document; call `document.set_edited(..)` when the content differs
   from the file. An edited window shows the dot and, on the native close, asks "Save changes?"
   with Save, Cancel and Don’t Save. Cmd-Q is meant to ask for the first edited window (not run).
2. `document.save().await` is the Save command: it asks for a file when `document.url()` is `None`, then calls your closure.
3. `.on_open_url(|url: OpenedUrl| ..)` receives the files that Finder and `open -a` hand to the app. Declare
   `CFBundleDocumentTypes` in the app's `Info.plist`.
4. macOS only: on Linux nothing is vetoed and no open event arrives ([ADR 0112](../docs/decisions.md)).

## 49. An inspector and a background extension

SwiftUI: `NavigationSplitView { .. } detail: { photo.backgroundExtensionEffect().inspector(isPresented: $shown) { .. } }`.

The program is [`recipes/inspector.rs`](recipes/inspector.rs).

1. `SplitView::inspector(view, presented)` adds the right pane; toggle `presented` to slide it in and out.
2. `SplitView::background_extension(view)` takes the view that lies behind the detail's leading edge, laid
   out as wide as the detail: it continues under the sidebar and, on macOS, under the inspector, through
   their glass ([10](../docs/barrierkit/10-sidebar.md#background-extension)).

## 50. Every SwiftUI window design

SwiftUI: `WindowGroup` with `.windowStyle(.titleBar)` or `.hiddenTitleBar`, and `.toolbar { }` with
`.windowToolbarStyle(.unified)`, `.unified(showsTitle: false)`, `.unifiedCompact`,
`.unifiedCompact(showsTitle: false)` or `.expanded`.

The program is [`recipes/window_designs.rs`](recipes/window_designs.rs). It takes `title-bar`,
`hidden-title-bar`, `unified`, `unified-no-title`, `compact`, `compact-no-title` or `expanded`; the
`NavigationSplitView` design is [`notes_sidebar`](recipes/notes_sidebar.rs).

1. `WindowStyle::TitleBar` is AppKit's own bar; every other design is a `HiddenTitleBar` window with
   native lights and a `Titlebar` drawn by the kit. A `HiddenTitleBar` window with no `Titlebar` keeps
   SwiftUI's bare strip, its lights at (8, 8).
2. `Titlebar::toolbar_style(..)` picks the bar and `.shows_title(false)` drops the title, so the items
   follow the lights. Windowed, in full screen and at the top edge in full screen the bar matches the
   SwiftUI window ([9](../docs/barrierkit/09-titlebars.md)).
