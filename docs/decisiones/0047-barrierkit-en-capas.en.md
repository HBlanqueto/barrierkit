# 0047 — Layered BarrierKit: a toolkit for building complete apps

Status: **accepted** (2026-10-28). What is done and what is pending is in «Status» (2026-11-02).

## Context

BarrierKit (ADR 0034) was born to draw the shell. Today it has:

- its own Wayland runner;
- the CSD titlebar, the traffic lights, the sidebar and the menu;
- a few controls of its own;
- direct re-exports of `freya-components` (`Button`, `Switch`, `Slider`, `Input`…) that look like Freya and not like Tengen.

The P4 audit confirmed it: System Settings reimplements the sidebar, the search field, the radios, the drop-down, the accent swatches and the card rows. Every new app would have to do it again.

The goal is for BarrierKit to serve for building **any** desktop app, like AppKit or libadwaita, not a single kind of app:

- System Settings;
- a Finder of our own;
- apps on par with libadwaita;
- a sample app with every control (Master Tengen App Show).

For that it needs a layered architecture, a catalogue of pieces and rules that prevent duplication from coming back.

User decisions already taken (2026-10-28):

- **Declarative API**, like SwiftUI and like Freya today. No controllers in the style of classic AppKit.
- **Customisation through Tengen variants.** An app picks system variants (size, primary or destructive, island or flat), not loose colours or measurements.
- **Search like on macOS.** It finds sections and also options inside the content.
- **Back/forward navigation** with history.
- **Full sidebar:** titled groups, collapsible groups, account row and badges.
- **SplitView:** adjustable width, it can be hidden, it collapses by itself in narrow windows and it supports a right inspector.
- **App infrastructure "like macOS":** unified actions, per-app preferences, single instance and state restoration.
- **Layer-by-layer order.** One layer is finished before the next one starts, and Settings is rebuilt at the end.

## Decision

### 1. Four layers and a cross-cutting column

```
┌─────────────────────────────────────────────────────────────┐
│ Capa 3 · Patrones      PreferencesWindow, BrowserWindow,    │
│                        AboutWindow…                         │
├─────────────────────────────────────────────────────────────┤
│ Capa 2 · Contenedores  SplitView, SourceList, Toolbar,      │  App (transversal)
│          y vistas      NavigationStack, Form, Table,        │  Application, Action,
│                        CollectionView, TabView, Sheet…      │  Defaults, Restoration,
├─────────────────────────────────────────────────────────────┤  UndoManager
│ Capa 1 · Primitivas    Estilo (paleta semántica, texto,     │
│                        símbolos) + controles                │
├─────────────────────────────────────────────────────────────┤
│ Capa 0 · Plataforma    Runner Wayland, ventanas, popups,    │
│                        entrada, foco, cursor, escala, a11y  │
└─────────────────────────────────────────────────────────────┘
```

**Dependency rules**

1. A layer only uses lower layers. Layer 1 does not know `SplitView`, and layer 0 does not know the controls.
2. **Layer 3 only composes the public API of layer 2.** If a pattern needs something layer 2 does not offer, that something is added to layer 2 and becomes available to any app. No pattern has private behaviour.
3. An app can use any layer directly. Settings uses layer 3, a Finder composes layer 2 and a very special view can draw with Freya primitives (`rect`, `label`) using the semantic palette.
4. Layers 2 and 3 use the **App** column (the toolbar and the menus consume `Action`), but it does not depend on them.

### 2. Rules for every kit component

- **Single source of design.** Every visual value comes from `vow-design` (Tengen, ADR 0011/0032/0046). If a token is missing, it is added to `tengen/components.json` and generated; a literal never goes in the component.
- **Variants, not free styles.** Each component exposes system variants through enums such as `ButtonStyle::{Primary, Secondary, Plain, Destructive}` or `ControlSize::{Small, Regular, Large}`. There is no `.background(color)` in the kit components.
- **Controlled and declarative.** The app holds the state (`State<T>`). The component receives `value` and reports with `on_change`. No hidden state the app cannot read.
- **Common interaction.** Every interactive control uses the same hook `use_interaction()`, which gives hover, pressed, keyboard focus and disabled. The focus ring (`focus-ring`) is drawn only when navigating with the keyboard.
- **Mandatory accessibility.** Each component assigns role, label and state (checked, expanded, selected), and is keyboard navigable. A control without an accessible label does not compile: the label is an argument of `new`.
- **Reactive to appearance.** Components read the appearance from the context the runner provides. A change of light or dark, accent or tint repaints without restarting the app.
- **Do not reinvent Freya.** When `freya-components` 0.4.3 already has the logic, the kit wraps it with the Tengen look instead of rewriting it. Cases: `ResizableContainer`, `Table`, `VirtualScrollView`, `DragZone`/`DropZone`, `Tooltip`, `Attached`, `Checkbox`, `RadioItem`, `Select`, `ProgressBar`, `ContextMenu`, `Accordion`. If the wrapper is not possible, the reason is documented with `// shortcut:` or in the component's ADR.
- **AppKit names** when an equivalent exists (`SplitView`, `SourceList`, `PopUpButton`, `ColorWell`, `Sheet`), in English in the code.
- **Each new component** arrives with:
  - a page in the catalogue app (see §6);
  - a render test in light, dark and tinted;
  - a keyboard test if it is interactive.

### 3. Catalogue by layer

Status: **exists** = already in the kit, even if it needs fixing · **partial** · **new**.

**Layer 0 · Platform** (`platform/`)

| Piece | Status | Note |
|---|---|---|
| Runner, windows, popups, layer-shell, lock | exists | Split into modules after the L1 batches (P4a-08) |
| Keyboard focus and Tab | **new** | L1b (F-01) |
| Per-component cursor | **new** | L1b (F-02) |
| Fractional scale (wp_fractional_scale + viewporter) | **new** | L1c |
| Clipboard and primary selection | **new** | `wl_data_device`, `zwp_primary_selection` |
| Drag and drop between apps | new | `wl_data_device` + `DragZone` |
| AT-SPI (accesskit_unix) | new | Together with Block 12b |
| Damage and opaque region | new | P4a S-07 |

**Layer 1 · Primitives**

| Piece | Status | Note |
|---|---|---|
| Semantic `Palette` | **new** | `label`, `secondary_label`, `tertiary_label`, `separator`, `window_background`, `under_page_background`, `control_background`, `control_accent`, `selected_content_background`, `destructive`, `success`, `warning`, taken from Tengen and the tint (VowAppearance). Replaces the flattened `ColorsSheet` of `theme.rs` |
| Freya theme generated from the palette | partial | For the Freya components used as they are; reactive |
| `TextStyle` | partial | The 9 Tengen styles (Display…Caption, Mono). `Text::new(..).style(Body).secondary()` |
| `Symbol` / icon registry | **new** | Icons by name, size tied to the text style, cache by (name, size, colour). Replaces the 6 copies of `svg_viewer` |
| `use_interaction` | **new** | hover, pressed, keyboard focus, disabled |
| `Button`, `IconButton` | **new** | Variants primary/secondary/plain/destructive and three sizes; default button (Enter) and cancel button (Esc) |
| `Switch`, `Checkbox`, `Radio` + `RadioGroup` | **new** | `Radio` comes from Settings |
| `Slider`, `Stepper` | new | |
| `SegmentedControl` | **new** | Token `segmented` |
| `TextField`, `SecureField`, `SearchField` | partial | With focus ring, placeholder, clear button and password role |
| `PopUpButton`, `ComboBox` | **new** | On top of `MenuView`; the traffic-light drop-down in Settings is a `PopUpButton` |
| `ColorWell` | **new** | Named colour swatches; comes from Settings |
| `ProgressBar`, `ProgressRing`, `Spinner` | new | Token `progress` |
| `Badge`, `Label`, `Link`, `Divider`, `Avatar`, `Image` | new | |
| `ToolGroup` | exists | Groups `IconButton`s; `ToolButton` is gone |
| Traffic lights, titlebar | exists | Fixed in L1d |

**Layer 2 · Containers and views**

| Piece | Status | Note |
|---|---|---|
| `SplitView` | **new** | Sidebar plus detail, with optional right inspector. Adjustable width with minimum and maximum, saved in the restoration. It hides with a toolbar button and a shortcut. Below a window width (token) it collapses by itself and opens as an overlay panel. Island or flat (ADR 0041). It holds the traffic lights when the sidebar is visible. On top of `ResizableContainer` |
| `SourceList` | partial | The sidebar: titled groups, collapsible groups, account row, items with symbol or app icon, badges and counters, selection and keyboard. Today they are `Sidebar`/`SidebarItem`/`SidebarSection` |
| `Toolbar` | partial | Leading, center and trailing zones; navigation buttons; title and subtitle; slot for search; overflow into a menu. It lives in the CSD titlebar (ADR 0035) |
| `NavigationStack` | **new** | Back/forward history, subpages (`push`), current title for the toolbar, actions `go-back` and `go-forward` with shortcuts |
| `Searchable` | **new** | Search index: each section and each row declare title and keywords; the kit searches and returns results; picking one navigates and highlights the row. `SearchField` uses it in the sidebar or in the toolbar |
| `Form`, `Section`, `Row` | **new** | Token `list-group`. Row variants: label + control, navigation (chevron, `push`), information and destructive. Header and footer per section |
| `ScrollView`, `VirtualList` | exists | Freya |
| `Table`, `OutlineView`, `ColumnBrowser` | new | Sortable, virtualised columns. Base for a Finder |
| `CollectionView` | new | Icon grid with multiple selection, drag selection, rename and drag |
| `TabView` | new | Document tabs in the window |
| `Sheet`, `Alert`, `Popover`, `Tooltip`, `ContextMenu` | partial | `MenuView` already exists. Sheet drops from the titlebar. Alert with default and cancel buttons |
| `EmptyState` | new | Token `settings.empty` generalised |

**Layer 3 · Patterns**

| Piece | Status | Note |
|---|---|---|
| `PreferencesWindow` | **new** | SplitView + SourceList (account, groups) + SearchField with `Searchable` of the content + NavigationStack + Form. Fixed-width option (only the height changes) with the green «+» traffic light |
| `BrowserWindow` | rejected | Deleted with no users (`0c93856`); a Finder composes layer 2 directly |
| `AboutWindow` | new | Standard «Acerca de» ("About") window |

**App (cross-cutting, `app/`)**

| Piece | Status | Note |
|---|---|---|
| `Application` | **new** | `app_id`, startup, single instance (`org.freedesktop.Application`, `DBusActivatable`), opening files and URLs, policy for what happens when the last window closes |
| `Action` | **new** | id, title, typed `Shortcut`, enabled/checked state and handler **always on the UI thread**. One action feeds the global menu (dbusmenu), the toolbar, the context menus and the keyboard. Responder chain: focused window → app. Replaces the `MenuItem` strings and `shortcut_matches` (P4b-10/14) |
| `Defaults` | **new** | Per-app preferences in `~/.config/malaysia/<app_id>.domain` (`.domain` format, ADR 0021), typed with serde, with change notification (the same watcher as `AppearanceWatch`) |
| `Restoration` | **new** | Size and position, section, sidebar width, navigation stack and scroll per window. It will connect to `xdg-session-management` when the compositor has it |
| `UndoManager` | new | Undo and redo wired to the standard Edición ("Edit") actions |

### 4. Crate structure

A single crate, `barrier-kit`. Layers 0, 2, 3, the App column and the shell have a folder; layer 1 is loose modules at the root, one per control family:

```
barrier-kit/src/
  platform/   capa 0  (runtime, window, popup, surface, input, focus, clipboard, vulkan, timer…)
  *.rs        capa 1  (style, theme, symbol, interaction, controls, compact, choices, level,
                       progress, adornments, popup_button, tooltip, context_menu, backdrop,
                       adaptive, titlebar, traffic, sidebar, menu/, app_menu/)
  views/      capa 2  (split, navigation, search, form, table, collection, tabs, feedback)
  patterns/   capa 3  (preferences, search_popover, about)
  app/        transversal (action, application, defaults, undo)
  shell/      solo para la shell, detrás de la feature `shell` (ver abajo)
```

There is no `prelude.rs`: each app imports from its module (`barrier_kit::views::form::Form`).

**The shell part goes separately, as on macOS.** The macOS Dock, menu bar and lock screen do not use the public AppKit API for what they do differently: they use private frameworks (SkyLight and friends). Here that separation is reproduced inside the same crate:

- Everything only `malaysia-shell` uses lives in its own folder `src/shell/`, in its own files and never mixed with the rest:

  ```
  shell/
    mod.rs             punto de entrada, solo existe con la feature; fachada pública
    layer.rs           superficies layer-shell (run_layer, Anchors, Layer, LayerSpec…)
    lock.rs            ext-session-lock (run_lock, SessionUnlock)
    toplevels.rs       foreign-toplevel: lista y control de ventanas para el Dock
    infinity.rs        protocolo privado de colocación de ventanas
    menus.rs           direcciones de los menús globales de otras apps (GTK y dbusmenu) para la barra de menús
    activation.rs      tokens xdg-activation para lanzar apps desde el Dock
  ```

- The `shell/` submodules are private (`mod activation;`, `mod infinity;`…); the public API comes only from `mod.rs` through `pub use`. The shell views import `barrier_kit::shell::{LayerSpec, run_layer, …}`, not the internal files.

- The folder only compiles with the `shell` feature (`#[cfg(feature = "shell")] pub mod shell;`). `malaysia-shell` enables it in its `Cargo.toml`; Settings, Master Tengen App Show and any new app do not. For them `barrier_kit::shell` does not exist: they neither compile it nor can call it by mistake.
- **Direction of dependencies:** `shell/` uses `platform/`, but `platform/` knows nothing about `shell/`. The runner exposes extension points (see «Extension points» below), and `shell/` hooks into them. No stray `#[cfg(feature = "shell")]` inside `platform/`, except the line that registers the module.
- The shell views (menubar, dock, lock, launcher) stay in `malaysia-shell` and are built with layers 1–3 like any app. `src/shell/` only provides the protocols and platform types those views need.

Publishing an app's own menu (`app_menu_export.rs`, `AppMenuAddress`) belongs to the platform, because every app uses it.

#### Extension points

- `Runtime` has a single slot `extension: Option<Extension>`; the `Extension` holds the shell state as `Box<dyn Any>` and four `fn` hooks: `provide_contexts`, `new_seat`, `new_output` and `poll`.
- They are `fn` (not closures or traits) so the core calls them passing `&mut Runtime`; the extension never leaves `Runtime`: the shell code uses short borrows (`extension` / `extension_mut`) and clones the handles it needs.
- `SurfaceRole::Extension(Box<dyn ExtensionRole>)` replaces `Layer` and `Lock`; `ExtensionRole::parent_popup` lets a layer surface be the parent of an `xdg_popup`.
- `seat_handle` belongs to the platform (move, resize, popup grab).
- `run_window` installs no extension; `run_layer` and `run_lock` install `ShellState`.
- Batch 1B-2 completed it: the shell files moved to `src/shell/` under the `shell` feature, the submodules became private behind a facade of re-exports, `SurfaceSpec` was removed and `run_layer` now receives `shell::LayerSpec` (the `Lock` and `Window` variants disappeared because `run_lock` and `run_window` have their own signatures), and the component's app is installed in `ShellState::install` instead of `Runtime::app`.

The only consumers are `malaysia-shell` and `malaysia-settings`, so the API can break between phases without a compatibility layer.

### 5. Order: layer by layer

| Phase | Content | Done when |
|---|---|---|
| 0 | Bug batches L1a–L1d (P4 audit) | The L1 reports reviewed and pushed |
| 1 | **Layer 0**: fractional scale, clipboard, damage and opaque region, split `platform/`, move the shell parts to `src/shell/` with the `shell` feature | Visible test in nested mode and on the ThinkPad |
| 2 | **Layer 1**: semantic palette and reactive theme, `TextStyle`, `Symbol`, `use_interaction`, every control in the table | Each control with its catalogue page and its tests |
| 3 | **App**: `Action`, `Application`, `Defaults` (layer 2 needs them) | The global menu and the shortcuts go through `Action` |
| 4 | **Layer 2**: SplitView, SourceList, Toolbar, NavigationStack, Searchable, Form, Sheet/Alert/Popover. Then Table, CollectionView and TabView | Complete catalogue |
| 5 | **Layer 3**: PreferencesWindow; `Restoration` | — |
| 6 | **System Settings rebuilt** with layer 3 only; its local copies are deleted | Settings defines no control or colour of its own |

The **Master Tengen App Show** app is created in phase 2 with its first pages and grows with each phase.

Each phase is split into small batches for opencode, with a visible test in the nested session.

### 6. Catalogue app: Master Tengen App Show

**Master Tengen App Show** (crate and binary `tengen-app-show`, `app_id` `…tengen-app-show`) is the kit's widget-factory, the equivalent of gtk4-widget-factory. It uses `PreferencesWindow`, with one section per layer and one page per component that shows all its variants and states. It is at the same time:

- the living documentation of the kit;
- the place for the visible tests of each batch;
- the source of the render tests (each page renders in light, dark and tinted).

Its Animación ("Animation") group shows motion: the Freya curves, the Grade spring (`Spring` is an `AnimatedValue` and is used with `use_animation`), the Tengen durations, the animatable properties (opacity, scale, rotation, position, colour, radius and width), the kit controls that already animate, the entry and exit of elements, the transitions between slides and tabs, the response to events (hover, press, error, notification and alert) and reduced motion. It is a mockup: what later goes into the kit comes from here. Its Ventana ("Window") group changes the titlebar (`TitlebarLayout`, rail, appearance), the fullscreen bar (`FullscreenBar`, `fullscreen_push`) and the sidebar of its own window.

**Marea** (`marea`) tests the kit with media: a featured item with background video, gradients that blend it into the window, a glass island on top, horizontal shelves with posters and video preview, genre chips and a player. For it the kit gains `video` (feature `video`: GStreamer decodes with VA-API and each frame arrives as a dmabuf imported into the kit's `VkDevice`, with no copies; without dmabuf it is downloaded once and without VA-API the CPU decodes), `media` (`scrim`, `material`, `MediaButton`, `RoundButton`, `use_artwork`), `register_font`, `ScrollView` with direction, controller and optional bar, and the tokens `MEDIA` and `SCROLL_BAR`. `crates/marea/media/generate.mjs` generates the test videos and frames from the mockup scenes.

## Status (2026-11-02)

Done in `main`:

- **Layer 0**: AT-SPI bridge with `accesskit_unix` (tree, focus and window focus in each window; the screen reader's Focus and Click focus and press), fractional scale, shell separation, focus and keyboard mode (Tab enables keyboard navigation; keys go to the focused node), clipboard on top of `wl_data_device` and opaque region of windows with an opaque background.
- **Layer 1**: `style` (semantic palette and text faces), `symbol`, `interaction` (hover, pressed, focus and ring), `Button`, `IconButton`, `Switch`, `Checkbox`, `RadioGroup`, `ColorWell`, `ChoiceCards`, `SegmentedControl`, `ToggleGroup`, `Stepper`, stepped `Slider`, `LevelSlider` (continuous and thick), `TextField`, `SearchField`, `PopUpButton`, `ProgressBar`, `ProgressRing`, `Spinner`, `badge`, `divider`, `avatar`, `Link` and `use_tooltip` (popup without grab under the pointer).
- **App**: `Action` with typed `Shortcut`, `load_defaults`/`save_defaults` with change notification, `launch` (single instance; `Activate` brings the window to the front through xdg-activation and `ActivateAction` fires the action with the same id as the menu), a focused text field keeps Ctrl+A, C, V, X, Y and Z before the menu, panel restoration, `use_undo` (`UndoManager` with the standard actions `UNDO`, Ctrl+Z, and `REDO`, Ctrl+Shift+Z), which disables and enables its global menu entries according to the stack.
- **Layer 2**: `Form`/`Section`/`Row`, `SplitView` (replaces `SidebarLayout`), `History` and `navigation_buttons` (actions `GO_BACK` and `GO_FORWARD` with Alt+← and Alt+→; `PreferencesWindow` puts the panel title in the bar), `SearchIndex`, `empty_state`, `sheet`, `Alert`, `TabView`, `Table`, `CollectionView`, `SidebarAccount` and `context_menu` (`MenuView` at the pointer on secondary click).
- **Layer 3**: `PreferencesWindow` (search field with suggestions and recents that are restored, results with the icon of their panel and traversal with ↑, ↓ and Enter) and `about_window`.
- **Tinted backdrops**: `Backdrop` paints three layers at 127°, 217° and 336° with dithering, from an image (k-means in OKLab, `vow_appearance::backdrop`) or three colours, and cross-fades between backdrops. Inside, `Palette::current()` gives its ink and its accent (`Accent::System`, `Own` or `FromBackdrop`), the same in light as in dark; filled controls accept `.tint(Tint::Accent | Color | Backdrop)`. The content has to be a child component of the `Backdrop`: functions evaluated in the parent read the system palette. The `Sidebar` lets the backdrop show through or, as an island, the app tints it with `SidebarBackdrop::Tinted` (ADR 0057).
- **Adaptive layout**: `use_breakpoint()` gives `Compact`, `Medium` or `Wide` according to `BREAKPOINT`.
- **Master Tengen App Show** (`tengen-app-show`), which gathers components, animations and titlebars, **Ondina** (`ondina`), the player that uses the backdrops, its own accent, the three sizes and the kit navigation, and **Marea** (`marea`), with media; they are the test apps of the repository.
- **Phase 6**: System Settings is a `PreferencesWindow` with a `Form`; it no longer has its own controls, sidebar or search field.

Pending. None of this is built until its reason goes away (rule 2 of AGENTS.md: nothing «por si acaso» ("just in case")):

- **Blocked by Freya 0.4.3:**
  - Partial damage: `RenderPipeline::render` repaints the whole tree (`freya-core/src/render_pipeline.rs`, «TODO: Use incremental rendering»).
  - Primary selection: the `Input` of `freya-components` exposes neither the selection nor the middle click, so there is nothing to publish and nowhere to paste.
- **Waits for the first app that uses it:**
  - Layer 0: drag and drop between apps (a Finder or a file manager).
  - Layer 1: an `Image` of our own; today Freya's `ImageViewer` inside the controls is enough.
  - Layer 2: search slot and overflow in the `Titlebar`, which already has `left` and `right` zones and a title; `OutlineView` and `ColumnBrowser`.
  - App: `Open` for files and URLs, policy when the last window closes, restoration of the navigation stack and of the scroll, clipboard with types besides text.
- **Depends on the compositor:** the window position, with `xdg-session-management`.

Decided: menu shortcuts stay stored as text (`MenuItem::Action::shortcut`). That way the shell menu bar receives the GTK and dbusmenu accelerators of other apps, and `find_menu_shortcut` only walks the menu of the focused window, which is small. The kit's own actions use typed `Shortcut`.

## Consequences

- System Settings stops defining controls: `controls.rs`, `sidebar.rs`, `groups.rs` and their local `svg_viewer`/`rgba` disappear in phase 6.
- `barrier_kit::controls` stops re-exporting Freya components with their default look.
- The global menu, the shortcuts and the toolbar share a single action model.
- Many tokens are added to `tengen/components.json` (controls, split view, toolbar, table…) and the `vow-design` generator grows.
- The work is long. Rebuilding Settings comes at the end of phase 6.

## References

- [`0011`](0011-valores-de-diseno-como-constantes-rust.en.md), [`0032`](0032-neptunia-como-fuente-de-diseno.en.md), [`0046`](0046-tengen-lenguaje-visual.en.md): tokens and Tengen.
- [`0021`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0021-superficie-de-configuracion-y-control.md): `.domain` domains and D-Bus.
- [`0034`](0034-celestialkit-sobre-freya.en.md): BarrierKit on top of Freya.
- [`0035`](0035-identificacion-de-apps-propias-y-csd.en.md): CSD of our own apps.
- [`0040`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0040-menu-global-real.md): global menu.
- [`0041`](0041-barra-lateral-en-isla.en.md): island sidebar.
- [`0042`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0042-ajustes-del-sistema.md): System Settings.

## Annex A · How an app is written

This is the API of `main` as of 2026-11-03. The repository apps are the complete reference: `tengen-app-show` uses layer 3 and tests both `TitlebarLayout`s with `WindowChrome`, and `ondina` composes layer 2 by hand (`SplitView`, `Titlebar`, `Backdrop`, `use_breakpoint`).

### A preferences app with layer 3

```rust
use barrier_kit::app::{Action, Instance, Launch, Shortcut, ShortcutKey, launch};
use barrier_kit::app::{REDO, UNDO, UndoManager, use_menu_action, use_undo};
use barrier_kit::app_menu::{AppMenu, Menu};
use barrier_kit::choices::ColorWell;
use barrier_kit::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use barrier_kit::platform::{WindowOptions, run_window};
use barrier_kit::symbol::Symbol;
use barrier_kit::views::form::{Form, Row, Section};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use vow_design::{ACCENT_SWATCHES, WINDOW_MIN_SIZE};

const RESET: Action =
    Action::new("reset", || tr!("reset")).shortcut(Shortcut::ctrl(ShortcutKey::Char('r')));

struct Notas {
    app_id: &'static str,
    instance: Option<Instance>,
}

impl App for Notas {
    fn render(&self) -> impl IntoElement {
        let instance = self.instance.clone();
        use_hook(move || {
            if let Some(instance) = &instance {
                instance.listen();
            }
        });
        let color = use_state(|| 0);
        let history = use_undo(color);
        use_menu_action(move |id| {
            if id == RESET.id {
                history.set(0);
            }
        });

        PreferencesWindow::new()
            .restoration(self.app_id)
            .group(PaneGroup::new(None).pane(Pane::new(
                "general",
                "General",
                Symbol::CircleHalf,
                ACCENT_SWATCHES[0].1,
                general(history, *color.read()),
            )))
    }
}

fn general(history: UndoManager<usize>, selected: usize) -> Form {
    Form::new().section(
        Section::titled("Etiqueta").row(
            Row::new("Color").keywords(&["color", "etiqueta"]).control(
                ColorWell::new("Color", &ACCENT_SWATCHES)
                    .selected(Some(selected))
                    .on_change(move |index| history.set(index)),
            ),
        ),
    )
}

fn main() {
    let app_id = barrier_kit::titlebar::app_id("notas").leak();
    let instance = match launch(app_id) {
        Ok(Launch::Forwarded) => return,
        Ok(Launch::Primary(instance)) => Some(instance),
        Err(_) => None,
    };
    run_window(
        AppComponent::new(Notas { app_id, instance }),
        WindowOptions::new("Notas", "notas")
            .size((760, 520))
            .min_size(Some(WINDOW_MIN_SIZE))
            .menu(AppMenu::new(vec![Menu::new(
                "Edición",
                vec![UNDO.item(true), REDO.item(true), RESET.item(true)],
            )])),
    )
    .expect("notas");
}
```

- `launch` gives the single instance: a second run forwards its activation token and exits.
- An `Action` is declared once and feeds the global menu, its shortcut and `use_menu_action`. Shortcuts only arrive if the action is in the window's menu.
- `use_undo` wraps a `State` of the app. Each `set` saves the previous value, and `UNDO` and `REDO` walk it from the menu, the shortcut or the `undo` and `redo` methods.
- `PreferencesWindow` builds the sidebar, the search field and the panel restoration. The `keywords` of each `Row` go into the search.
- Controls are controlled: `selected`, `value`, `on` or `checked` rule when the app changes them, and the control keeps what the user changes in the meantime. That is why the `ColorWell` shows what the app undoes.
- Colours and measurements come from `vow_design`. An app does not write design literals.
