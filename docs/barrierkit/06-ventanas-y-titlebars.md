# 6. Ventanas y titlebars

**Objetivo:** abrir ventanas con las opciones correctas y elegir el tipo de
titlebar y de semáforos.
**Lector:** quien define la ventana principal de una app.

## Abrir una ventana

```rust
run_window(
    AppComponent::new(MiApp),
    WindowOptions::new("Notas", "notas")
        .decorations(Decorations::Client)
        .size((760, 520))
        .min_size(Some(WINDOW_MIN_SIZE))
        .menu(menu),
)
```

| Opción | Efecto |
|---|---|
| `new(título, nombre)` | Título visible y nombre corto. El `app_id` es `io.github.hblanqueto.malaysia.<nombre>` salvo que pases `.app_id(..)` |
| `.decorations(..)` | `Client` (por defecto): el kit dibuja la titlebar. `Server`: la pide al compositor; si no hay `zxdg_decoration_manager_v1`, vuelve a `Client` |
| `.size((ancho, alto))` | Tamaño inicial (640 × 420 por defecto). Si hay uno guardado, se restaura |
| `.min_size(..)`, `.max_size(..)` | Límites. Si son iguales, la ventana es de tamaño fijo y el botón verde en modo `zoom` maximiza solo en vertical |
| `.parent(..)` | Ventana hija (diálogo) |
| `.menu(AppMenu)` | Menú global de la ventana (capítulo [10](10-app.md)) |

Dentro de la app, `WindowControls::get()` da el estado (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) y las órdenes: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`.

## La titlebar horizontal: `Titlebar`

```rust
Titlebar::new()
    .title("Documento")
    .with_sidebar(true)
    .left(SidebarToggle)
    .right(IconButton::new(Symbol::Plus, "Nuevo"))
```

- Tiene tres zonas: `left`, título centrado y `right`.
- Sin barra lateral, las zonas son simétricas para que el título quede
  centrado.
- Con `.with_sidebar(true)` se alinea con la barra lateral. Los semáforos se
  quedan fijos en su sitio aunque la barra se pliegue.
- Arrastrarla mueve la ventana; el doble clic hace lo que diga `window.domain`.
- Con `Decorations::Server` concedida, mide 0 y no dibuja nada.
- `.close_only(true)` deja solo el botón de cerrar, para diálogos.

El panel «Ventana y herramientas» de Tengen App Show muestra los semáforos
sueltos, una `Titlebar` con título y otra con `.close_only(true)`:

![Semáforos, barra de título y «solo cerrar»](assets/apps/tengen-ventana-light.png)

## Los semáforos: `TrafficLights`

Cerrar, minimizar y maximizar. Normalmente no se crean a mano:
`Titlebar` y `Sidebar::traffic_lights(true)` los colocan con
`titlebar::window_traffic_lights(enfocada, fija)`. Si se necesitan sueltos:

```rust
TrafficLights::new()
    .focused(true)
    .zoom_supported(true)
    .vertical(false)
    .round(false)
    .on_action(|boton: ButtonKind| { /* Close, Minimize, Maximize */ })
```

| Ajuste | Dónde | Valores |
|---|---|---|
| Color | `appearance.domain` → `traffic_lights` | `auto`, `standard` (o `macos`), `accent`, `wallpaper` |
| Botón verde | `window.domain` → `green_button` | `focus` (por defecto: pantalla completa), `zoom`, `fullscreen`, `none` |
| Doble clic en la titlebar | `window.domain` → `double_click_titlebar` | `zoom` (por defecto), `fill`, `minimize`, `none` |

En una ventana inactiva los semáforos se vuelven grises.

## Cinco tipos de titlebar: `WindowChrome`

`chrome::WindowChrome::new(estilo, contenido)` envuelve la ventana y coloca los
semáforos y el título según un `ChromeStyle`:

```rust
let estilo = ChromeStyle {
    layout: TitlebarLayout::Rail,
    side: Side::Left,
    rail_width: TITLEBAR_RAIL.width,
    round_lights: false,
    reveal: Reveal::Always,
    resting_edge: RestingEdge::None,
};
WindowChrome::new(estilo, SplitView::new(barra, detalle)).footer(pie)
```

| `TitlebarLayout` | Qué es | Captura (Rótulo) |
|---|---|---|
| `Top` (por defecto) | Titlebar horizontal arriba | ![Diseño Top](assets/apps/rotulo-top.png) |
| `Rail` | Franja vertical a un lado con los semáforos en columna y el título girado 90° | ![Diseño Rail](assets/apps/rotulo-rail.png) |
| `Unified` | Los semáforos en columna dentro de la barra lateral, sin franja propia. Sin barra lateral visible pasa a `Rail` | ![Diseño Unified](assets/apps/rotulo-unified.png) |
| `Pill` | Una píldora flotante con los semáforos. En pantalla completa es cristal | ![Diseño Pill](assets/apps/rotulo-pill.png) |
| `Actions` | Un `Rail` más ancho con un pie de acciones (`.footer(..)`) | ![Diseño Actions](assets/apps/rotulo-actions.png) |

| Campo | Valores | Efecto |
|---|---|---|
| `side` | `Left`, `Right` | Lado del rail |
| `rail_width` | `TITLEBAR_RAIL.width` o `.width_wide` | Ancho del rail |
| `round_lights` | `bool` | Semáforos redondos o cuadrados (por defecto) |
| `reveal` | `Always`, `Drawer`, `Push`, `Corner`, `Popup` | Siempre visible, o aparece al acercar el puntero: como cajón encima, empujando el contenido, solo en la esquina o como píldora emergente |
| `resting_edge` | `None`, `Line`, `Accent` | Qué queda visible cuando está oculta: nada, una línea o una línea de acento |

Algunas combinaciones no tienen sentido y se corrigen solas
(`ChromeStyle::effective`): `Unified` con `Drawer`, `Push` o `Popup` pasa a
`Corner`; `Pill` con `Drawer` o `Push` pasa a `Popup`; en pantalla completa,
`reveal` es siempre `Always`.

Rail a la derecha, semáforos redondos, barra lateral plana y modo oscuro:

![Rail derecho con semáforos redondos](assets/apps/rotulo-rail-flat-round.png)

La app Rótulo (capítulo [14](14-apps-de-prueba.md)) prueba todas las
combinaciones en vivo.

## Popups

Los menús, desplegables y ayudas usan `xdg_popup`. Para uno propio:

```rust
let id = Popups::get().open(spec, AppComponent::new(MiPopup));
Popups::get().close(id);
```

`PopupSpec` fija el rectángulo de anclaje, el ancla, la gravedad, el tamaño, el
ajuste si no cabe y si toma el grab del puntero.

Siguiente: [7. Barra lateral y SplitView](07-barra-lateral.md).
