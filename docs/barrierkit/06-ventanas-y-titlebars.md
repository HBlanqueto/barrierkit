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
| `.min_size(..)`, `.max_size(..)` | Límites. Si son iguales, la ventana es de tamaño fijo y el botón verde la maximiza solo en vertical |
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

## Maximizada, pantalla completa y modo foco

El estado llega en cada configure del compositor y se lee con
`WindowControls::get().status.read()`: `maximized`, `fullscreen` y `activated`.

| Estado | Cómo se entra | Qué hace el kit |
|---|---|---|
| Normal | — | Titlebar en su sitio; al cambiar de tamaño, el tamaño se guarda |
| Maximizada | Botón verde con `green_button = "zoom"`, doble clic con `zoom` o `fill`, o `toggle_maximize()` | Nada visual: la titlebar sigue igual. En MalaysiaDreams el compositor conserva el marco (esquinas y sombra). El tamaño no se guarda |
| Pantalla completa y modo foco | Botón verde con `focus` (por defecto) o `fullscreen`, o `toggle_fullscreen()` | La titlebar sale del diseño y el contenido ocupa toda la ventana. Los semáforos se ocultan. El tamaño no se guarda |

En MalaysiaDreams, el modo foco da a la ventana el estado `fullscreen` y una
barra negra que baja al acercar el puntero arriba
([ADR 0051 y 0054 de MalaysiaDreams](https://github.com/HBlanqueto/malaysia/tree/main/docs/decisiones)).
El kit pide `set_overlay` por `infinity_focus_v1` v2. Mientras la barra está
bajada, `titlebar_overlaid()` es verdadero y cada titlebar se dibuja encima del
contenido, a la altura de la barra, con cristal progresivo y apareciendo con
ella:

| Diseño | En pantalla completa, con la barra bajada |
|---|---|
| `Top` | La `Titlebar` baja sobre el contenido. Los semáforos de la barra lateral bajan lo mismo |
| `Rail`, `Actions` | El rail se dibuja como cristal y baja con la barra |
| `Unified` | Igual que `Top` |
| `Pill` | La píldora no se ve hasta que baja la barra; entonces aparece como cristal |

En pantalla completa, `reveal` es siempre `Always`. Sin el protocolo (otro
compositor), la titlebar se queda en el diseño y solo se ocultan los semáforos.

### Qué hace la app

1. Si usa `Titlebar`, `WindowChrome`, `Sidebar` o `PreferencesWindow`, no tiene
   que hacer nada: los tres estados ya funcionan.
2. Deja hueco para la barra en el contenido que va bajo la titlebar
   superpuesta, como Ondina:

   ```rust
   let arriba = try_consume_root_context::<WindowControls>()
       .map_or(0.0, |controls| controls.fullscreen_content_top());
   rect().padding(Gaps::new(arriba, 0.0, 0.0, 0.0)).child(contenido)
   ```

3. Para un comportamiento propio (un modo inmersivo que esconde paneles),
   lee el estado y decide:

   ```rust
   let controls = WindowControls::get();
   let inmersivo = controls.status.read().fullscreen;
   rect().maybe_child((!inmersivo).then(panel_de_herramientas))
   ```

4. Para ofrecerlo en un menú, llama a `toggle_maximize()` o
   `toggle_fullscreen()` desde una `Action` (capítulo [10](10-app.md)).

### El botón verde y el doble clic

Los decide el usuario en `window.domain`. Con una ventana de tamaño fijo
(mínimo igual a máximo) cambian:

| `green_button` | Ventana normal | Tamaño fijo |
|---|---|---|
| `focus` (por defecto) | Pantalla completa | Maximiza en vertical |
| `zoom` | Maximiza | Maximiza en vertical, aunque el botón se dibuja gris y sin glifo |
| `fullscreen` | Pantalla completa | Maximiza en vertical |
| `none` | Nada | Maximiza en vertical |

| `double_click_titlebar` | Ventana normal | Tamaño fijo |
|---|---|---|
| `zoom` (por defecto), `fill` | Maximiza | Nada |
| `minimize` | Minimiza | Minimiza |
| `none` | Nada | Nada |

## Popups

Los menús, desplegables y ayudas usan `xdg_popup`. Para uno propio:

```rust
let id = Popups::get().open(spec, AppComponent::new(MiPopup));
Popups::get().close(id);
```

`PopupSpec` fija el rectángulo de anclaje, el ancla, la gravedad, el tamaño, el
ajuste si no cabe y si toma el grab del puntero.

Siguiente: [7. Barra lateral y SplitView](07-barra-lateral.md).
