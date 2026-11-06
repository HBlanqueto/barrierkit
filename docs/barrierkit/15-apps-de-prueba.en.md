# 15. Test apps

**Goal:** know which app in the repository tests each part of the kit and how
to launch it.
**Reader:** anyone who wants to see the kit working or check a change.

## Launching them

1. Enter the dev shell: `nix develop`.
2. Launch the app:

   ```sh
   ./dev.sh app tengen-app-show
   ./dev.sh app ondina
   ./dev.sh app marea
   ./dev.sh app trazo
   ```

They run on any Wayland compositor (chapter
[14](14-plataforma.en.md#the-first-frame)). On Malaysia they also get borders, shadow,
blur and the global menu.

`dev.sh` sets `MALAYSIA_CONFIG_DIR` and `MALAYSIA_DATA_DIR` under
`$XDG_RUNTIME_DIR/barrierkit` unless they are already set. The apps do not
touch your real configuration.

## Master Tengen App Show (`tengen-app-show`)

The test app for the kit's components, like `gtk4-widget-factory`. It is a
`PreferencesWindow` inside a `WindowChrome`: one group per layer and one pane
per component, plus the Ventana ("Window") and Animación ("Animation") groups.

![«Botones» ("Buttons") pane of Tengen App Show](assets/apps/tengen-botones-light.png)

| Group | Panes |
|---|---|
| Capa 1 · Primitivas (layer 1, primitives) | Botones, Interruptores, Elección, Deslizadores, Campos, Progreso, Adornos (buttons, switches, choice, sliders, fields, progress, adornments) |
| Capa 2 · Vistas (layer 2, views) | Formularios, Tabla y rejilla, Pestañas, Fondo tintado, Estados y alertas, Ventana y herramientas, Diseño adaptable (forms, table and grid, tabs, tinted backdrop, states and alerts, window and tools, adaptive layout) |
| Capa 3 · Patrones (layer 3, patterns) | Acerca de, Apariencia (about, appearance; it forces light, dark or tinted) |
| Ventana (window) | Titlebar, Pantalla completa, Barra lateral (titlebar, full screen, sidebar) |
| Animación (animation) | Curvas, Propiedades, Kit en movimiento, Aparición, Diapositivas, Puntero y respuesta, Movimiento (curves, properties, kit in motion, appearance, slides, pointer and response, motion) |
| App | Deshacer (undo) |

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

### Window

The panes of the Ventana group change the app's own window. They save the
choice with `save_defaults("tengen-app-show")` in
`~/.config/malaysia/tengen-app-show.domain`.

| Pane | What it tests |
|---|---|
| Titlebar | The two `TitlebarLayout` values, the rail width, the round traffic lights, the normal or unified rail, its items at the top and the bottom, the five reveal modes and the three resting edges |
| Pantalla completa (full screen) | The `FullscreenBar` and `fullscreen_push`, live |
| Barra lateral (sidebar) | Island or flat, show or hide, resizing and ranges |

| Titlebar | Full screen | Sidebar |
|---|---|---|
| ![Titlebar pane](assets/apps/tengen-titlebar-top.png) | ![Full screen pane](assets/apps/tengen-pantalla-completa-light.png) | ![Sidebar pane](assets/apps/tengen-barra-lateral-light.png) |

The screenshots of each layout are in chapter [7](07-titlebars.en.md).

### Animation

The panes of the Animación group show Freya curves, Grade springs, animatable
properties, controls in motion, appearance, transitions between slides,
response to events and reduced motion. Their screenshots are in chapter
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

## Marea (`marea`)

A movie and TV app that tests the kit with media (`video` feature):

![Marea: featured item with video, sidebar with account, chips and shelves](assets/apps/marea-dark.png)

- A featured item with background video and a scrim (`scrim`) that blends it
  into the window.
- Glass (`material`) over the content.
- Horizontal shelves (horizontal `ScrollView`) with cover art and video
  previews.
- Genre chips, media buttons and a player.
- A test featured item, "Sprite Fright", that shows the poster and three
  images with a crossfade. With the pointer still over it for 2 s, it plays
  the start of the short film from YouTube (resolved by `yt-dlp`). The
  "Películas abiertas" ("Open movies") shelf opens the same video.

The test videos and frames are generated with
`node crates/marea/media/generate.mjs`. It needs `ffmpeg` with libx264 and Node
with Playwright and Chromium, which the dev shell does not ship. Left out are
the `sprite-fright-*` files in
`media/stills/`: frames from "Sprite Fright" (Blender Studio, CC BY 4.0) and
a title drawn with Creepster (see `NOTICE`). The screenshot was made without a GPU: the
video was decoded on the CPU (`Decoding::Cpu`).

## Trazo (`trazo`)

Tests the charts (chapter [19](19-graficas.en.md)) in three tabs:

| Tab | Content |
|---|---|
| 2D charts | Monotone lines, stacked areas, grouped bars with a rule, stacked bars, points, steps, pie, donut, heat map, stream and flow clock |
| Monitor | User and system CPU, usage per core, memory, network, load average, core clock, memory by process (horizontal bars) and processes, read from `/proc` every second |
| 3D | Blender's Suzanne (3936 triangles), sin(r)/r surface and point cloud; drag to rotate or pick a plane |

`cargo test -p trazo` checks that every tab draws its charts; with
`TRAZO_SHOT=<folder>` it saves one screenshot per tab.

Next: [16. Frequently asked questions](16-preguntas.en.md).
