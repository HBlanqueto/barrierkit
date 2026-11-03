# 0047 — BarrierKit en capas: un toolkit para construir apps completas

Estado: **aceptado** (2026-10-28). Lo hecho y lo pendiente están en «Estado» (2026-11-02).

## Contexto

BarrierKit (ADR 0034) nació para dibujar la shell. Hoy tiene:

- un runner Wayland propio;
- la titlebar CSD, los semáforos, la barra lateral y el menú;
- unos pocos controles propios;
- re-exportaciones directas de `freya-components` (`Button`, `Switch`, `Slider`, `Input`…) que se ven con el estilo de Freya y no con el de Tengen.

La auditoría P4 lo confirmó: Ajustes del sistema reimplementa la barra lateral, el buscador, los radios, el desplegable, las muestras de acento y las filas en tarjeta. Cada app nueva tendría que volver a hacerlo.

El objetivo es que BarrierKit sirva para construir **cualquier** app del escritorio, como AppKit o libadwaita, no una sola forma de app:

- Ajustes del sistema;
- un Finder propio;
- apps al nivel de libadwaita;
- una app de muestra con todos los controles (Master Tengen App Show).

Para eso necesita una arquitectura en capas, un catálogo de piezas y reglas que impidan volver a duplicar.

Decisiones del usuario ya tomadas (2026-10-28):

- **API declarativa**, como SwiftUI y como Freya hoy. Nada de controladores al estilo del AppKit clásico.
- **Personalización por variantes de Tengen.** Una app elige variantes del sistema (tamaño, primario o destructivo, isla o plana), no colores ni medidas sueltas.
- **Búsqueda como en macOS.** Encuentra secciones y también opciones dentro del contenido.
- **Navegación atrás/adelante** con historial.
- **Barra lateral completa:** grupos con título, grupos plegables, fila de cuenta e insignias.
- **SplitView:** ancho ajustable, se puede ocultar, se pliega sola en ventanas estrechas y admite un inspector derecho.
- **Infraestructura de app «como macOS»:** acciones unificadas, preferencias por app, instancia única y restauración de estado.
- **Orden capa por capa.** Se termina una capa antes de empezar la siguiente, y Ajustes se rehace al final.

## Decisión

### 1. Cuatro capas y una columna transversal

```
┌─────────────────────────────────────────────────────────────┐
│ Capa 3 · Patrones      PreferencesWindow, BrowserWindow,    │
│                        AboutWindow…                         │
├─────────────────────────────────────────────────────────────┤
│ Capa 2 · Contenedores  SplitView, SourceList, Toolbar,      │  App (transversal)
│          y vistas      NavigationStack, Form, Table,        │  Application, Action,
│                        CollectionView, TabView, Sheet…      │  Defaults, Restoration,
├─────────────────────────────────────────────────────────────┤  UndoManager, Pasteboard
│ Capa 1 · Primitivas    Estilo (paleta semántica, texto,     │
│                        símbolos) + controles                │
├─────────────────────────────────────────────────────────────┤
│ Capa 0 · Plataforma    Runner Wayland, ventanas, popups,    │
│                        entrada, foco, cursor, escala, a11y  │
└─────────────────────────────────────────────────────────────┘
```

**Reglas de dependencia**

1. Una capa solo usa capas inferiores. La capa 1 no conoce `SplitView`, y la 0 no conoce los controles.
2. **La capa 3 solo compone la API pública de la capa 2.** Si un patrón necesita algo que la capa 2 no ofrece, ese algo se añade a la capa 2 y queda disponible para cualquier app. Ningún patrón tiene comportamiento privado.
3. Una app puede usar cualquier capa directamente. Ajustes usa la capa 3, un Finder compone la capa 2 y una vista muy especial puede dibujar con primitivas de Freya (`rect`, `label`) usando la paleta semántica.
4. La columna **App** la usan las capas 2 y 3 (la barra de herramientas y los menús consumen `Action`), pero no depende de ellas.

### 2. Reglas para todo componente del kit

- **Fuente única de diseño.** Todo valor visual sale de `vow-design` (Tengen, ADR 0011/0032/0046). Si falta un token, se añade a `tengen/components.json` y se genera; nunca va un literal en el componente.
- **Variantes, no estilos libres.** Cada componente expone variantes del sistema mediante enums como `ButtonStyle::{Primary, Secondary, Plain, Destructive}` o `ControlSize::{Small, Regular, Large}`. No hay `.background(color)` en los componentes del kit.
- **Controlado y declarativo.** El estado lo tiene la app (`State<T>`). El componente recibe `value` y avisa con `on_change`. Nada de estado oculto que la app no pueda leer.
- **Interacción común.** Todo control interactivo usa el mismo hook `use_interaction()`, que da hover, pressed, foco de teclado y disabled. El anillo de foco (`focus-ring`) se dibuja solo al navegar con teclado.
- **Accesibilidad obligatoria.** Cada componente asigna rol, etiqueta y estado (checked, expanded, selected), y es navegable por teclado. Un control sin etiqueta accesible no compila: la etiqueta es argumento de `new`.
- **Reactivo a la apariencia.** Los componentes leen la apariencia del contexto que provee el runner. Un cambio de claro u oscuro, de acento o de tinte vuelve a pintar sin reiniciar la app.
- **No reinventar Freya.** Cuando `freya-components` 0.4.3 ya tiene la lógica, el kit la envuelve con el aspecto de Tengen en lugar de reescribirla. Casos: `ResizableContainer`, `Table`, `VirtualScrollView`, `DragZone`/`DropZone`, `Tooltip`, `Attached`, `Checkbox`, `RadioItem`, `Select`, `ProgressBar`, `ContextMenu`, `Accordion`. Si el envoltorio no es posible, el porqué se documenta con `// shortcut:` o en el ADR del componente.
- **Nombres de AppKit** cuando existe equivalente (`SplitView`, `SourceList`, `PopUpButton`, `ColorWell`, `Sheet`), en inglés en el código.
- **Cada componente nuevo** llega con:
  - una página en la app de catálogo (ver §6);
  - un test de render en claro, oscuro y tintado;
  - un test de teclado si es interactivo.

### 3. Catálogo por capa

Estado: **existe** = ya está en el kit, aunque haya que arreglarlo · **parcial** · **nuevo**.

**Capa 0 · Plataforma** (`platform/`)

| Pieza | Estado | Nota |
|---|---|---|
| Runner, ventanas, popups, layer-shell, lock | existe | Se parte en módulos después de los lotes L1 (P4a-08) |
| Foco de teclado y Tab | **nuevo** | L1b (F-01) |
| Cursor por componente | **nuevo** | L1b (F-02) |
| Escala fraccional (wp_fractional_scale + viewporter) | **nuevo** | L1c |
| Portapapeles y selección primaria | **nuevo** | `wl_data_device`, `zwp_primary_selection` |
| Arrastrar y soltar entre apps | nuevo | `wl_data_device` + `DragZone` |
| AT-SPI (accesskit_unix) | nuevo | Junto al Bloque 12b |
| Damage y región opaca | nuevo | P4a S-07 |

**Capa 1 · Primitivas**

| Pieza | Estado | Nota |
|---|---|---|
| `Palette` semántica | **nuevo** | `label`, `secondary_label`, `tertiary_label`, `separator`, `window_background`, `under_page_background`, `control_background`, `control_accent`, `selected_content_background`, `destructive`, `success`, `warning`, sacados de Tengen y del tinte (VowAppearance). Reemplaza el `ColorsSheet` aplanado de `theme.rs` |
| Tema de Freya generado desde la paleta | parcial | Para los componentes de Freya que se usen tal cual; reactivo |
| `TextStyle` | parcial | Los 9 estilos de Tengen (Display…Caption, Mono). `Text::new(..).style(Body).secondary()` |
| `Symbol` / registro de iconos | **nuevo** | Iconos por nombre, tamaño ligado al estilo de texto, caché por (nombre, tamaño, color). Sustituye las 6 copias de `svg_viewer` |
| `use_interaction` | **nuevo** | hover, pressed, foco de teclado, disabled |
| `Button`, `IconButton` | **nuevo** | Variantes primary/secondary/plain/destructive y tres tamaños; botón por defecto (Enter) y de cancelar (Esc) |
| `Switch`, `Checkbox`, `Radio` + `RadioGroup` | **nuevo** | `Radio` sale de Ajustes |
| `Slider`, `Stepper` | nuevo | |
| `SegmentedControl` | **nuevo** | Token `segmented` |
| `TextField`, `SecureField`, `SearchField` | parcial | Con anillo de foco, placeholder, botón de borrar y rol de contraseña |
| `PopUpButton`, `ComboBox` | **nuevo** | Sobre `MenuView`; el desplegable de semáforos de Ajustes es un `PopUpButton` |
| `ColorWell` | **nuevo** | Muestras de color con nombre; sale de Ajustes |
| `ProgressBar`, `ProgressRing`, `Spinner` | nuevo | Token `progress` |
| `Badge`, `Label`, `Link`, `Divider`, `Avatar`, `Image` | nuevo | |
| `ToolGroup`, `ToolButton` | existe | Pasan a usar `Action` (§4) |
| Semáforos, titlebar | existe | Se arreglan en L1d |

**Capa 2 · Contenedores y vistas**

| Pieza | Estado | Nota |
|---|---|---|
| `SplitView` | **nuevo** | Barra lateral más detalle, con inspector derecho opcional. Ancho ajustable con mínimo y máximo, guardado en la restauración. Se oculta con un botón de la barra de herramientas y un atajo. Por debajo de un ancho de ventana (token) se pliega sola y se abre como panel superpuesto. Isla o plana (ADR 0041). Lleva dentro los semáforos cuando la barra lateral está visible. Sobre `ResizableContainer` |
| `SourceList` | parcial | La barra lateral: grupos con título, grupos plegables, fila de cuenta, ítems con símbolo o icono de app, insignias y contadores, selección y teclado. Hoy son `Sidebar`/`SidebarItem`/`SidebarSection` |
| `Toolbar` | parcial | Zonas leading, center y trailing; botones de navegación; título y subtítulo; hueco para búsqueda; overflow en menú. Vive en la titlebar CSD (ADR 0035) |
| `NavigationStack` | **nuevo** | Historial atrás/adelante, subpáginas (`push`), título actual para la barra de herramientas, acciones `go-back` y `go-forward` con atajos |
| `Searchable` | **nuevo** | Índice de búsqueda: cada sección y cada fila declaran título y palabras clave; el kit busca y devuelve resultados; elegir uno navega y resalta la fila. Lo usa `SearchField` en la barra lateral o en la barra de herramientas |
| `Form`, `Section`, `Row` | **nuevo** | Token `list-group`. Variantes de fila: etiqueta + control, de navegación (chevron, `push`), de información y destructiva. Encabezado y pie por sección |
| `ScrollView`, `VirtualList` | existe | Freya |
| `Table`, `OutlineView`, `ColumnBrowser` | nuevo | Columnas ordenables, virtualizadas. Base para un Finder |
| `CollectionView` | nuevo | Rejilla de iconos con selección múltiple, selección por arrastre, renombrar y arrastrar |
| `TabView` | nuevo | Pestañas de documento en la ventana |
| `Sheet`, `Alert`, `Popover`, `Tooltip`, `ContextMenu` | parcial | `MenuView` ya existe. Sheet baja de la titlebar. Alert con botones por defecto y cancelar |
| `EmptyState` | nuevo | Token `settings.empty` generalizado |

**Capa 3 · Patrones**

| Pieza | Estado | Nota |
|---|---|---|
| `PreferencesWindow` | **nuevo** | SplitView + SourceList (cuenta, grupos) + SearchField con `Searchable` del contenido + NavigationStack + Form. Opción de ancho fijo (solo cambia de alto) con el semáforo verde «+» |
| `BrowserWindow` | descartado | Se borró sin usuarios (`0c93856`); un Finder compone la capa 2 directamente |
| `AboutWindow` | nuevo | Ventana «Acerca de» estándar |

**App (transversal, `app/`)**

| Pieza | Estado | Nota |
|---|---|---|
| `Application` | **nuevo** | `app_id`, arranque, instancia única (`org.freedesktop.Application`, `DBusActivatable`), abrir archivos y URLs, política de qué pasa al cerrar la última ventana |
| `Action` | **nuevo** | id, título, `Shortcut` tipado, estado activa/marcada y handler **siempre en el hilo de UI**. Una acción alimenta el menú global (dbusmenu), la barra de herramientas, los menús contextuales y el teclado. Cadena de respuesta: ventana enfocada → app. Sustituye las cadenas de `MenuItem` y `shortcut_matches` (P4b-10/14) |
| `Defaults` | **nuevo** | Preferencias por app en `~/.config/malaysia/<app_id>.domain` (formato `.domain`, ADR 0021), tipadas con serde, con aviso al cambiar (el mismo watcher que `AppearanceWatch`) |
| `Restoration` | **nuevo** | Tamaño y posición, sección, ancho de barra lateral, pila de navegación y desplazamiento por ventana. Se conectará con `xdg-session-management` cuando el compositor lo tenga |
| `UndoManager` | nuevo | Deshacer y rehacer conectados a las acciones estándar de Edición |
| `Pasteboard` | nuevo | API sobre el portapapeles de la capa 0 |

### 4. Estructura del crate

Un solo crate `barrier-kit`. Las capas 0, 2, 3, la columna App y la shell tienen carpeta; la capa 1 son módulos sueltos en la raíz, uno por familia de controles:

```
barrier-kit/src/
  platform/   capa 0  (runtime, window, popup, surface, input, focus, clipboard, vulkan, timer…)
  *.rs        capa 1  (style, theme, symbol, interaction, controls, compact, choices, level,
                       progress, adornments, popup_button, tooltip, context_menu, backdrop,
                       adaptive, titlebar, traffic, sidebar, menu/, app_menu/)
  views/      capa 2  (split, navigation, search, form, table, collection, tabs, feedback)
  patterns/   capa 3  (preferences, search_popover, about)
  app/        transversal (action, application, defaults, pasteboard, undo)
  shell/      solo para la shell, detrás de la feature `shell` (ver abajo)
```

No hay `prelude.rs`: cada app importa de su módulo (`barrier_kit::views::form::Form`).

**La parte de la shell va separada, como en macOS.** El Dock, la barra de menús y la pantalla de bloqueo de macOS no usan la API pública de AppKit para lo que hacen distinto: usan frameworks privados (SkyLight y compañía). Aquí se reproduce esa separación dentro del mismo crate:

- Todo lo que solo usa `malaysia-shell` vive en su propia carpeta `src/shell/`, en archivos propios y nunca mezclado con el resto:

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

- Los submódulos de `shell/` son privados (`mod activation;`, `mod infinity;`…); la API pública sale únicamente de `mod.rs` mediante `pub use`. Las vistas de la shell importan `barrier_kit::shell::{LayerSpec, run_layer, …}`, no los archivos internos.

- La carpeta solo se compila con la feature `shell` (`#[cfg(feature = "shell")] pub mod shell;`). `malaysia-shell` la activa en su `Cargo.toml`; Ajustes, Master Tengen App Show y cualquier app nueva no. Para ellas `barrier_kit::shell` no existe: ni lo compilan ni pueden llamarlo por error.
- **Dirección de las dependencias:** `shell/` usa `platform/`, pero `platform/` no sabe nada de `shell/`. El runner expone puntos de extensión (ver «Puntos de extensión» más abajo), y `shell/` se engancha a ellos. Ningún `#[cfg(feature = "shell")]` suelto dentro de `platform/`, salvo la línea que registra el módulo.
- Las vistas de la shell (menubar, dock, lock, launcher) siguen en `malaysia-shell` y se construyen con las capas 1–3 como cualquier app. `src/shell/` solo aporta los protocolos y tipos de plataforma que esas vistas necesitan.

Publicar el menú propio de una app (`app_menu_export.rs`, `AppMenuAddress`) es de la plataforma, porque lo usan todas las apps.

#### Puntos de extensión

- `Runtime` tiene una única ranura `extension: Option<Extension>`; la `Extension` guarda el estado de la shell como `Box<dyn Any>` y cuatro ganchos `fn`: `provide_contexts`, `new_seat`, `new_output` y `poll`.
- Son `fn` (no closures ni traits) para que el núcleo los llame pasando `&mut Runtime`; la extensión nunca sale de `Runtime`: el código de la shell usa préstamos cortos (`extension` / `extension_mut`) y clona las asas que necesita.
- `SurfaceRole::Extension(Box<dyn ExtensionRole>)` sustituye a `Layer` y `Lock`; `ExtensionRole::parent_popup` permite que una superficie layer sea padre de un `xdg_popup`.
- `seat_handle` es de la plataforma (mover, redimensionar, grab de popups).
- `run_window` no instala extensión; `run_layer` y `run_lock` instalan `ShellState`.
- En el lote 1B-2 se completó: los archivos de la shell se movieron a `src/shell/` bajo la feature `shell`, los submódulos pasaron a ser privados detrás de una fachada de re-exports, `SurfaceSpec` se eliminó y `run_layer` recibe ahora `shell::LayerSpec` (las variantes `Lock` y `Window` desaparecieron porque `run_lock` y `run_window` tienen sus propias firmas), y la app del componente se instala en `ShellState::install` en lugar de `Runtime::app`.

Los únicos consumidores son `malaysia-shell` y `malaysia-settings`, así que la API puede romperse entre fases sin capa de compatibilidad.

### 5. Orden: capa por capa

| Fase | Contenido | Sale cuando |
|---|---|---|
| 0 | Lotes de bugs L1a–L1d (auditoría P4) | Los informes de L1 revisados y con push |
| 1 | **Capa 0**: escala fraccional, portapapeles, damage y región opaca, partir `platform/`, mover lo de la shell a `src/shell/` con la feature `shell` | Prueba visible en anidado y en el ThinkPad |
| 2 | **Capa 1**: paleta semántica y tema reactivo, `TextStyle`, `Symbol`, `use_interaction`, todos los controles de la tabla | Cada control con su página en el catálogo y sus tests |
| 3 | **App**: `Action`, `Application`, `Defaults` (la capa 2 los necesita) | El menú global y los atajos van por `Action` |
| 4 | **Capa 2**: SplitView, SourceList, Toolbar, NavigationStack, Searchable, Form, Sheet/Alert/Popover. Después Table, CollectionView y TabView | Catálogo completo |
| 5 | **Capa 3**: PreferencesWindow; `Restoration` | — |
| 6 | **Ajustes del sistema rehecho** solo con la capa 3; se borran sus copias locales | Ajustes no define ningún control ni color propio |

La app **Master Tengen App Show** se crea en la fase 2 con sus primeras páginas y crece con cada fase.

Cada fase se divide en lotes pequeños para opencode, con prueba visible en la sesión anidada.

### 6. App de catálogo: Master Tengen App Show

**Master Tengen App Show** (crate y binario `tengen-app-show`, `app_id` `…tengen-app-show`) es la widget-factory del kit, equivalente a gtk4-widget-factory. Usa `PreferencesWindow`, con una sección por capa y una página por componente que muestra todas sus variantes y estados. Es a la vez:

- la documentación viva del kit;
- el sitio de las pruebas visibles de cada lote;
- la fuente de los tests de render (cada página se renderiza en claro, oscuro y tintado).

## Estado (2026-11-02)

Hecho en `main`:

- **Capa 0**: puente AT-SPI con `accesskit_unix` (árbol, foco y foco de ventana, en cada ventana), escala fraccional, separación de la shell, foco y modo teclado (Tab activa la navegación por teclado; las teclas van al nodo enfocado), portapapeles sobre `wl_data_device` y región opaca de las ventanas con fondo opaco.
- **Capa 1**: `style` (paleta semántica y caras de texto), `symbol`, `interaction` (hover, pulsado, foco y anillo), `Button`, `IconButton`, `Switch`, `Checkbox`, `RadioGroup`, `ColorWell`, `ChoiceCards`, `SegmentedControl`, `ToggleGroup`, `Stepper`, `Slider` por pasos, `LevelSlider` (continuo y grueso), `TextField`, `SearchField`, `PopUpButton`, `ProgressBar`, `ProgressRing`, `Spinner`, `badge`, `divider`, `avatar`, `Link` y `use_tooltip` (popup sin grab bajo el puntero).
- **App**: `Action` con `Shortcut` tipado, `load_defaults`/`save_defaults` con aviso de cambios, `launch` (instancia única; `Activate` trae la ventana al frente por xdg-activation y `ActivateAction` lanza la acción del mismo id que el menú), un campo de texto con el foco se queda Ctrl+A, C, V, X, Y y Z antes que el menú, restauración del panel, `use_undo` (`UndoManager` con las acciones estándar `UNDO`, Ctrl+Z, y `REDO`, Ctrl+Mayús+Z), que apaga y enciende sus entradas del menú global según la pila y portapapeles de texto (`copy`/`paste`).
- **Capa 2**: `Form`/`Section`/`Row`, `SplitView` (sustituye a `SidebarLayout`), `History` y `navigation_buttons` (acciones `GO_BACK` y `GO_FORWARD` con Alt+← y Alt+→; `PreferencesWindow` pone el título del panel en la barra), `SearchIndex`, `empty_state`, `sheet`, `Alert`, `TabView`, `Table`, `CollectionView`, `SidebarAccount` y `context_menu` (`MenuView` en el puntero con el clic secundario).
- **Capa 3**: `PreferencesWindow` (buscador con sugerencias y recientes que se restauran, resultados con el icono de su panel y recorrido con ↑, ↓ y Enter) y `about_window`.
- **Fondos tintados**: `Backdrop` pinta tres capas en 127°, 217° y 336° con dithering, desde una imagen (k-medias en OKLab, `vow_appearance::backdrop`) o tres colores, y funde entre fondos. Dentro, `Palette::current()` da su tinta y su acento (`Accent::System`, `Own` o `FromBackdrop`), igual en claro que en oscuro; los controles con relleno aceptan `.tint(Tint::Accent | Color | Backdrop)`. El contenido tiene que ser un componente hijo del `Backdrop`: las funciones evaluadas en el padre leen la paleta del sistema. La `Sidebar` deja ver el fondo o, en isla, la tiñe la app con `SidebarBackdrop::Tinted` (ADR 0057).
- **Diseño adaptable**: `use_breakpoint()` da `Compact`, `Medium` o `Wide` según `BREAKPOINT`.
- **Master Tengen App Show** (`tengen-app-show`) **Ondina** (`ondina`), el reproductor que usa los fondos, el acento propio, los tres tamaños y la navegación del kit, y **Rótulo** (`rotulo`), que prueba las titlebars verticales; son las tres apps de prueba del repositorio.
- **Fase 6**: Ajustes del sistema es un `PreferencesWindow` con un `Form`; ya no tiene controles, barra lateral ni buscador propios.

Pendiente:

- Capa 0: selección primaria, arrastrar y soltar entre apps, las acciones de AT-SPI (un lector de pantalla ya lee el árbol y el foco, pero aún no puede pulsar ni enfocar) y damage parcial. El damage espera al renderizado incremental de Freya: `RenderPipeline::render` repinta el árbol entero en 0.4.3 (`freya-core/src/render_pipeline.rs`, «TODO: Use incremental rendering»).
- Capa 1:
  - `Image` propio.
- Capa 2:
  - `Toolbar` con zonas, título, búsqueda y desbordamiento; hoy solo hay pestañas de barra (`views/tabs.rs`) y `ToolGroup`.
  - `OutlineView` y `ColumnBrowser`.
- App:
  - `Application` no abre archivos ni URLs (`Open` llega por D-Bus y se ignora) ni tiene política para el cierre de la última ventana: ninguna app abre documentos ni tiene varias ventanas todavía.
  - `Restoration` guarda el panel, las búsquedas y el tamaño de la ventana. La posición depende de `xdg-session-management` en el compositor; la pila de navegación y el desplazamiento, de una app que los necesite.
  - `Action`: el menú guarda el atajo como texto y `find_menu_shortcut` lo vuelve a parsear en cada tecla.
  - El portapapeles solo lleva texto.

## Consecuencias

- Ajustes del sistema deja de definir controles: `controls.rs`, `sidebar.rs`, `groups.rs` y sus `svg_viewer`/`rgba` locales desaparecen en la fase 6.
- `barrier_kit::controls` deja de re-exportar componentes de Freya con su aspecto por defecto.
- El menú global, los atajos y la barra de herramientas comparten un único modelo de acciones. Las cadenas de atajo dejan de parsearse en cada tecla.
- Se añaden muchos tokens a `tengen/components.json` (controles, split view, toolbar, tabla…) y el generador de `vow-design` crece.
- El trabajo es largo. Rehacer Ajustes queda al final de la fase 6.

## Referencias

- [`0011`](0011-valores-de-diseno-como-constantes-rust.md), [`0032`](0032-neptunia-como-fuente-de-diseno.md), [`0046`](0046-tengen-lenguaje-visual.md): tokens y Tengen.
- [`0021`](0021-superficie-de-configuracion-y-control.md): dominios `.domain` y D-Bus.
- [`0034`](0034-celestialkit-sobre-freya.md): BarrierKit sobre Freya.
- [`0035`](0035-identificacion-de-apps-propias-y-csd.md): CSD de las apps propias.
- [`0040`](0040-menu-global-real.md): menú global.
- [`0041`](0041-barra-lateral-en-isla.md): barra lateral en isla.
- [`0042`](0042-ajustes-del-sistema.md): Ajustes del sistema.

## Anexo A · Cómo se escribe una app

Es la API de `main` a 2026-11-02. Las apps del repositorio son la referencia completa: `tengen-app-show` usa la capa 3, `ondina` compone la capa 2 a mano (`SplitView`, `Titlebar`, `Backdrop`, `use_breakpoint`) y `rotulo` prueba las titlebars verticales.

### Una app de preferencias con la capa 3

```rust
use barrier_kit::app::{Action, Instance, Launch, Shortcut, ShortcutKey, launch};
use barrier_kit::app::{REDO, UNDO, UndoManager, use_menu_action, use_undo};
use barrier_kit::app_menu::{AppMenu, Menu};
use barrier_kit::choices::ColorWell;
use barrier_kit::patterns::preferences::{Pane, PaneGroup, PreferencesWindow};
use barrier_kit::platform::{Decorations, WindowOptions, run_window};
use barrier_kit::symbol::Symbol;
use barrier_kit::views::form::{Form, Row, Section};
use freya_core::integration::AppComponent;
use freya_core::prelude::*;
use vow_design::{ACCENT_SWATCHES, WINDOW_MIN_SIZE};

const RESET: Action =
    Action::new("reset", "Restablecer").shortcut(Shortcut::ctrl(ShortcutKey::Char('r')));

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
            .decorations(Decorations::Client)
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

- `launch` da la instancia única: una segunda ejecución reenvía su token de activación y sale.
- Una `Action` se declara una vez y alimenta el menú global, su atajo y `use_menu_action`. Los atajos solo llegan si la acción está en el menú de la ventana.
- `use_undo` envuelve un `State` de la app. Cada `set` guarda el valor anterior, y `UNDO` y `REDO` lo recorren desde el menú, el atajo o los métodos `undo` y `redo`.
- `PreferencesWindow` construye la barra lateral, el buscador y la restauración del panel. Las `keywords` de cada `Row` entran en la búsqueda.
- Los controles son controlados: `selected`, `value`, `on` o `checked` mandan cuando la app los cambia, y el control conserva lo que cambia el usuario mientras tanto. Por eso el `ColorWell` muestra lo que deshace la app.
- Los colores y medidas salen de `vow_design`. Una app no escribe literales de diseño.
