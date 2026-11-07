# 3. Layers

**Goal:** know where each piece lives and what each one can use.
**Reader:** anyone choosing which level of the kit to build their app with.
The full decision is in [ADR 0047](../decisiones/0047-barrierkit-en-capas.en.md).

## The map

```
┌───────────────────────────────────────────────────────┐
│ Capa 3 · Patrones     PreferencesWindow, about_window │
├───────────────────────────────────────────────────────┤  App (transversal)
│ Capa 2 · Vistas       SplitView, Form, Table,         │  Action, launch,
│                       CollectionView, TabView,        │  Defaults, Restoration,
│                       History, SearchIndex, Alert…    │  UndoManager, copy/paste
├───────────────────────────────────────────────────────┤
│ Capa 1 · Primitivas   Palette, text, Symbol,          │
│                       use_interaction, controles,     │
│                       Titlebar, Sidebar, MenuView…    │
├───────────────────────────────────────────────────────┤
│ Capa 0 · Plataforma   run_window, popups, entrada,    │
│                       foco, cursor, escala, Vulkan    │
└───────────────────────────────────────────────────────┘
        shell/ (feature `shell`): layer-shell, lock, Dock
```

## Where each layer lives in the code

| Layer | Folder or modules in `crates/barrier-kit/src/` |
|---|---|
| 0 · Platform | `platform/`, `i18n`, `portal`, `drag`, `notifications`, `secrets` |
| 1 · Primitives | `style`, `theme`, `symbol`, `interaction`, `animation`, `controls`, `buttons`, `compact`, `choices`, `level`, `progress`, `indicator`, `adornments`, `popup_button`, `combo_box`, `popover`, `tooltip`, `context_menu`, `password`, `text_editor`, `rich_text`, `spelling`, `token_field`, `path_control`, `date`, `date_picker`, `color_picker`, `image_well`, `rule_editor`, `print`, `charts/`, `canvas`, `glass_container`, `backdrop`, `media`, `video`, `adaptive`, `titlebar`, `traffic`, `chrome`, `sidebar`, `menu/`, `app_menu/`, `status_item` |
| 2 · Views | `views/` (`form`, `group`, `split`, `split_panes`, `floating_pane`, `table`, `tree`, `collection`, `tabs`, `window_tabs`, `text_view`, `feedback`, `navigation`, `search`) and `font_panel` |
| 3 · Patterns | `patterns/` (`preferences`, `search_popover`, `about`, `language`) |
| App | `app/` (`action`, `application`, `defaults`, `undo`) |
| Shell | `shell/`, only with the `shell` feature |

There is no `prelude`: each piece is imported from its module, for example
`barrier_kit::views::form::Form`.

## Rules

1. A layer only uses the ones below it. Layer 1 does not know `SplitView`.
   Exception: `platform::app` mounts the root of each window
   (`chrome::window_root`), paints its background with `theme` and serves the
   global menu and the status icons (`app_menu`, `status_item`). Tests
   (`#[cfg(test)]`) mount whole windows and use any layer.
2. Layer 3 only composes the public API of layer 2. Whatever a pattern needs is
   added to layer 2 and becomes available to every app.
3. An app can use any layer. Settings uses layer 3; Ondina composes layer 2 by
   hand; a very special view draws with Freya's `rect` and `label` and the
   `Palette`.
4. The shell (`menubar`, `dock`, `lock`, `launcher`) uses `shell/`. The other
   apps do not enable the feature and do not even see the module.

## Which layer to choose

| If you want… | Use |
|---|---|
| A settings app with a sidebar, search field and panes | Layer 3: `PreferencesWindow` |
| An app with its own navigation (media player, file browser) | Layer 2: `SplitView` + `Sidebar` + `Titlebar` |
| A small window or a dialog | Layer 1: controls inside a `rect` |
| A component that does not exist | Layer 1 + `use_interaction` and the `Palette`; if other apps will use it, it goes into the kit |

## The rules every component follows

- **Tengen values.** No color or size literals; everything comes from
  `vow_design`.
- **Variants, not free styles.** `ButtonStyle::Primary`, `Tint::Backdrop`,
  `SidebarBackdrop::Tinted`… never `.background(color)` on a kit control.
- **Controlled.** The app holds the state and the control reports changes.
- **Shared interaction.** Hover, pressed, keyboard focus and focus ring come
  from `use_interaction()`.
- **Accessible.** The accessible label is an argument of `new`: a control
  without a label does not compile.
- **Reactive.** A change of mode, accent or tint repaints without restarting.

Next: [4. Appearance and tokens](04-apariencia-y-tokens.en.md).
