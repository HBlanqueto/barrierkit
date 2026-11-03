# 15. Test apps

**Goal:** know which app in the repository tests each part of the kit and how
to launch it.
**Reader:** anyone who wants to see the kit working or check a change.

## Launching them

1. Enter the dev shell: `nix develop`.
2. Launch the app:

   ```sh
   ./dev.sh app tengen-app-show
   ./dev.sh app tengen-motion-show
   ./dev.sh app ondina
   ./dev.sh app marea
   cargo run -p rotulo
   ```

They run on any compositor that draws the first frame (chapter
[14](14-plataforma.en.md)). On MalaysiaDreams they also get borders, shadow,
blur and the global menu.

## Master Tengen App Show (`tengen-app-show`)

The kit's live catalog, like `gtk4-widget-factory`. It is a
`PreferencesWindow` with one group per layer and one pane per component.

![«Botones» ("Buttons") pane of Tengen App Show](assets/apps/tengen-botones-light.png)

| Group | Panes |
|---|---|
| Capa 1 · Primitivas (layer 1, primitives) | Botones, Interruptores, Elección, Deslizadores, Campos, Progreso, Adornos (buttons, switches, choice, sliders, fields, progress, adornments) |
| Capa 2 · Vistas (layer 2, views) | Formularios, Tabla y rejilla, Pestañas, Fondo tintado, Estados y alertas, Ventana y herramientas, Diseño adaptable (forms, table and grid, tabs, tinted backdrop, states and alerts, window and tools, adaptive layout) |
| Capa 3 · Patrones (layer 3, patterns) | Acerca de, Apariencia (about, appearance; it forces light, dark or tinted) |
| App | Deshacer, Barra lateral (undo, sidebar: resizing and ranges) |

| Interruptores | Elección | Deslizadores |
|---|---|---|
| ![Switches](assets/apps/tengen-interruptores-light.png) | ![Choice](assets/apps/tengen-eleccion-light.png) | ![Sliders](assets/apps/tengen-deslizadores-light.png) |

| Campos | Progreso | Adornos |
|---|---|---|
| ![Fields](assets/apps/tengen-campos-light.png) | ![Progress](assets/apps/tengen-progreso-light.png) | ![Adornments](assets/apps/tengen-adornos-light.png) |

| Tabla y rejilla | Pestañas | Estados y alertas |
|---|---|---|
| ![Table and grid](assets/apps/tengen-datos-light.png) | ![Tabs](assets/apps/tengen-pestanas-light.png) | ![States and alerts](assets/apps/tengen-estados-light.png) |

| Diseño adaptable | Acerca de | Apariencia |
|---|---|---|
| ![Adaptive layout](assets/apps/tengen-adaptable-light.png) | ![About](assets/apps/tengen-acerca-light.png) | ![Appearance](assets/apps/tengen-apariencia-light.png) |

Forms in dark:

![Forms in dark](assets/apps/tengen-formularios-dark.png)

Each new kit component arrives with its pane here.

## Master Tengen Motion Show (`tengen-motion-show`)

The motion catalog, also a `PreferencesWindow`: Freya curves, Grade springs,
animatable properties, controls in motion, appearance, transitions between
slides, response to events and reduced motion. Its screenshots are in chapter
[12](12-animaciones.en.md).

## Ondina (`ondina`)

A music player that composes layer 2 by hand: `SplitView`, `Sidebar`,
`Titlebar`, `TabView` and `Backdrop`.

| Light | Dark |
|---|---|
| ![Ondina in light](assets/apps/ondina-light.png) | ![Ondina in dark](assets/apps/ondina-dark.png) |

- **Tinted backdrop** from the album cover, a palette or none.
- **Accent** from the system, from the backdrop or custom, chosen in its
  «Apariencia» ("Appearance") pane.
- **Three sizes** with `use_breakpoint`. Wide shows the sidebar, the player
  and the queue. Medium opens the queue with a titlebar button. Compact hides
  the sidebar and turns the sections into tabs under the titlebar:

  ![Compact Ondina, 600 px](assets/apps/ondina-compact.png)

- **Custom actions** with shortcuts: play or pause, next (Ctrl+→), previous
  (Ctrl+←) and sections (Ctrl+1 to Ctrl+4), in its global menus «Controles»
  ("Controls") and «Ver» ("View").

## Rótulo (`rotulo`)

It tests the two `TitlebarLayout` values, the normal or unified rail, its width, its items at the top and the bottom, the round traffic lights, the five reveal modes, the three resting edges and
the island or flat sidebar. It saves the choice with `save_defaults("rotulo")`.
Its screenshots are in chapter [7](07-titlebars.en.md).

## Marea (`marea`)

A movie and TV app that tests the kit with media (`video` feature):

![Marea: featured item with video, sidebar with account, chips and shelves](assets/apps/marea-dark.png)

- A featured item with background video and a scrim (`scrim`) that blends it
  into the window.
- Glass (`material`) over the content.
- Horizontal shelves (horizontal `ScrollView`) with cover art and video
  previews.
- Genre chips, media buttons and a player.

The test videos and frames are generated with
`crates/marea/media/generate.mjs`. The screenshot was made without a GPU: the
video was decoded on the CPU (`Decoding::Cpu`).

Next: [16. Frequently asked questions](16-preguntas.en.md).
