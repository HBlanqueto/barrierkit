# 3. Capas

**Objetivo:** saber dónde vive cada pieza y qué puede usar cada una.
**Lector:** quien elige con qué nivel del kit construir su app.
La decisión completa está en el [ADR 0047](../decisiones/0047-barrierkit-en-capas.md).

## El mapa

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

## Dónde está cada capa en el código

| Capa | Carpeta o módulos de `crates/barrier-kit/src/` |
|---|---|
| 0 · Plataforma | `platform/`, `i18n`, `portal`, `drag`, `notifications`, `secrets` |
| 1 · Primitivas | `style`, `theme`, `symbol`, `interaction`, `animation`, `controls`, `buttons`, `compact`, `choices`, `level`, `progress`, `indicator`, `adornments`, `popup_button`, `combo_box`, `popover`, `tooltip`, `context_menu`, `password`, `text_editor`, `rich_text`, `spelling`, `token_field`, `path_control`, `date`, `date_picker`, `color_picker`, `image_well`, `rule_editor`, `print`, `charts/`, `canvas`, `glass_container`, `backdrop`, `media`, `video`, `adaptive`, `titlebar`, `traffic`, `chrome`, `sidebar`, `menu/`, `app_menu/`, `status_item` |
| 2 · Vistas | `views/` (`form`, `group`, `split`, `split_panes`, `floating_pane`, `table`, `tree`, `collection`, `tabs`, `window_tabs`, `text_view`, `feedback`, `navigation`, `search`) y `font_panel` |
| 3 · Patrones | `patterns/` (`preferences`, `search_popover`, `about`, `language`) |
| App | `app/` (`action`, `application`, `defaults`, `undo`) |
| Shell | `shell/`, solo con la feature `shell` |

No hay `prelude`: cada pieza se importa de su módulo, por ejemplo
`barrier_kit::views::form::Form`.

## Reglas

1. Una capa solo usa las de debajo. La capa 1 no conoce `SplitView`.
   Excepción: `platform::app` monta la raíz de cada ventana
   (`chrome::window_root`), pinta su fondo con `theme` y sirve el menú global
   y los iconos de estado (`app_menu`, `status_item`). Los tests
   (`#[cfg(test)]`) montan ventanas enteras y usan cualquier capa.
2. La capa 3 solo compone la API pública de la capa 2. Lo que un patrón
   necesita se añade a la capa 2 y queda para todas las apps.
3. Una app puede usar cualquier capa. Ajustes usa la 3; Ondina compone la 2 a
   mano; una vista muy especial dibuja con `rect` y `label` de Freya y la
   `Palette`.
4. La shell (`menubar`, `dock`, `lock`, `launcher`) usa `shell/`. Las demás apps
   no activan la feature y ni siquiera ven el módulo.

## Qué capa elegir

| Si quieres… | Usa |
|---|---|
| Una app de ajustes con barra lateral, buscador y paneles | Capa 3: `PreferencesWindow` |
| Una app con navegación propia (reproductor, navegador de archivos) | Capa 2: `SplitView` + `Sidebar` + `Titlebar` |
| Una ventana pequeña o un diálogo | Capa 1: controles dentro de un `rect` |
| Un componente que no existe | Capa 1 + `use_interaction` y la `Palette`; si lo usarán otras apps, va al kit |

## Las reglas que cumple todo componente

- **Valores de Tengen.** Ningún literal de color o medida; todo sale de
  `vow_design`.
- **Variantes, no estilos libres.** `ButtonStyle::Primary`, `Tint::Backdrop`,
  `SidebarBackdrop::Tinted`… nunca `.background(color)` en un control del kit.
- **Controlado.** La app tiene el estado y el control avisa.
- **Interacción común.** Hover, pulsado, foco de teclado y anillo de foco salen
  de `use_interaction()`.
- **Accesible.** La etiqueta accesible es argumento de `new`: un control sin
  etiqueta no compila.
- **Reactivo.** Un cambio de modo, acento o tinte repinta sin reiniciar.

Siguiente: [4. Apariencia y tokens](04-apariencia-y-tokens.md).
