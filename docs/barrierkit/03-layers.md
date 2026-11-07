# 3. Layers

**Goal:** find the module of any piece of the kit and know which layer a new
piece belongs to.
**Scope:** the modules of `crates/bk/src/`, the prelude and the rules
between layers. Which layer to start an app from is in
[1](01-what-is-barrierkit.md#which-level-to-use); the reasons are in
[ADR 0047](../decisions/0047-barrierkit-in-layers.md).
**Reader:** anyone who imports from the kit or adds a component to it.
**Input:** a name from the kit or a component to add. **Output:** its module,
its import and its layer.

## Steps

1. Try the prelude first: `use bk::prelude::*;` brings the names in
   [The prelude](#the-prelude).
2. For any other name, find its module in [Modules by layer](#modules-by-layer)
   and import it by path, for example `use bk::views::form::Form;`.
   The [glossary](20-glossary.md) lists every public name with its module.
3. To add a component, put it in the lowest layer that has everything it needs.
   It may use only the layers below it ([Rules](#rules)).
4. Build it as every component is built ([What every component does](#what-every-component-does)).

## The map

```text
┌───────────────────────────────────────────────────────┐
│ Layer 3 · Patterns    SettingsWindow, AboutView,       │
│                       LanguagePicker                   │
├───────────────────────────────────────────────────────┤  App (cross-cutting)
│ Layer 2 · Views       SplitView, Form, Table,          │  Application, Scene,
│                       CollectionView, TabView, Alert…  │  Action, launch,
├───────────────────────────────────────────────────────┤  defaults, restoration,
│ Layer 1 · Primitives  Text, VStack, Palette, Symbol,   │  UndoManager
│                       controls, Titlebar, Sidebar…     │
├───────────────────────────────────────────────────────┤
│ Layer 0 · Platform    run_window, popups, input,       │
│                       timers, portal, Vulkan           │
└───────────────────────────────────────────────────────┘
        shell (feature `shell`): layer-shell, lock, toplevels, tray
```

## Modules by layer

Each row lists the public modules of `crates/bk/src/lib.rs`. Internal
modules (not `pub`) are in the last column; an app cannot import them.

| Layer | Public modules | Internal |
|---|---|---|
| 0 · Platform | `platform`, `i18n`, `portal`, `drag`, `notifications`, `secrets` | `session_bus`, `fonts`, `logging` |
| 1 · Primitives | `elements`, `style`, `theme`, `symbol`, `interaction`, `animation`, `adaptive`, `controls`, `buttons`, `compact`, `choices`, `level`, `progress`, `indicator`, `adornments`, `popup_button`, `combo_box`, `popover`, `menu`, `password`, `text_editor`, `rich_text`, `spelling`, `token_field`, `path_control`, `date`, `date_picker`, `color_picker`, `image_well`, `rule_editor`, `print` (feature `print`), `charts` (feature `charts`), `canvas`, `glass_container`, `backdrop`, `media`, `video` (feature `video`), `modifiers`, `titlebar`, `traffic`, `chrome`, `sidebar`, `main_menu`, `status_item` | `scroll`, `glass`, `controlled`, `context_menu`, `tooltip` |
| 2 · Views | `views` (`collection`, `feedback`, `floating_pane`, `form`, `group`, `navigation`, `search`, `split`, `split_panes`, `table`, `tabs`, `text_view`, `tree`, `window_tabs`) and `font_panel` | — |
| 3 · Patterns | `patterns` (`about`, `language`, `settings`) | `patterns::search_popover` |
| App | `app` and `prelude` | — |
| Shell | `shell`, only with the `shell` feature | — |

Three items live at the crate root: `register_font`, `init_tracing` and
`ProbeExt` ([4](04-appearance-and-tokens.md#helpers)).

## The prelude

The prelude re-exports the entry points, the controls, Freya's prelude and the
common tokens. Its exact contents are in [2](02-first-app.md#the-prelude). Any
other token comes from `ctdesign`, which the app then adds as a dependency.

## Rules

1. A layer uses only the layers below it. Layer 1 does not know `SplitView`.
2. `app` is cross-cutting: every layer may use it (actions, defaults,
   restoration).
3. Layer 0 has one exception: `platform` mounts the root of each window
   (`chrome::window_root`), paints its background with `theme`, and serves the
   menu bar menu and the status item (`main_menu`, `status_item`).
4. Tests (`#[cfg(test)]`) mount whole windows and may use any layer.
5. Layer 3 composes only the public API of layer 2. What a pattern needs is
   added to layer 2, so every app gets it.
6. An app may use any layer. A settings app uses layer 3; a player composes
   layer 2; a special view draws with Freya's `rect` and `label` and the
   `Palette`.
7. Only `malaysia-shell` enables the `shell` feature. Other apps do not see the
   module.

## What every component does

| Rule | ✗ | ✓ |
|---|---|---|
| Values come from Tengen ([4](04-appearance-and-tokens.md#tokens-ctdesign)) | `.corner_radius(6.)` | `.corner_radius(BUTTON.radius)` |
| Colors come from the `Palette` ([4](04-appearance-and-tokens.md#the-palette)) | `Color::from_rgb(0, 0, 0)` | `color(Palette::current().label)` |
| Variants and styles, not paint on top ([ADR 0064](../decisions/0064-styles-and-themes.md)) | `.background(red)` on a kit control | `.style(ButtonStyle::bordered_prominent)`, `.tint(Tint::Color(..))`, `Slider::….style(\|s\| ..)` |
| The app holds the state | The control keeps the value | The control takes the value and reports changes |
| Shared interaction | Own hover and focus code | `use_interaction()` for hover, press, focus and focus ring |
| Accessible | A control without a name | The accessible label is an argument of `new`: `Button::new("Save")` |
| Reactive | Read the appearance once | A change of mode, accent or tint repaints without a restart |

Next: [4. Appearance and tokens](04-appearance-and-tokens.md).
