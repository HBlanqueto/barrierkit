# 7. Titlebars

**Objetivo:** elegir la titlebar de una app y saber cómo se ve en cada estado:
normal, sin foco, maximizada, en pantalla completa y en el modo foco de
Malaysia.
**Lector:** quien diseña la ventana de una app.

## Qué hay

| Pieza | Para | Por defecto |
|---|---|---|
| `Titlebar` | La titlebar horizontal CSD: semáforos, zonas izquierda y derecha, título | Con barra lateral si se pasa `.with_sidebar(true)` |
| `TrafficLights` | Cerrar, minimizar y maximizar | Cuadrados, colores de `traffic_lights` |
| `WindowChrome` + `ChromeStyle` | Dos diseños: `Top` y `Rail` (normal o unificado); los dos admiten elementos propios | `Top` |
| `FullscreenBar` | Cómo se dibuja la titlebar superpuesta en pantalla completa | `Solid` («A · Lisa») |

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
- Con `.with_sidebar(true)` se alinea con la barra lateral y el título va al
  inicio, junto a `left`, como en macOS. Los semáforos se
  quedan fijos en su sitio aunque la barra se pliegue.
- Arrastrarla mueve la ventana; el doble clic hace lo que diga `window.domain`.
- `.close_only(true)` deja solo el botón de cerrar, para diálogos: la barra es
  transparente y mide lo que la SSD (`TITLEBAR.thickness`), con los semáforos en
  el mismo sitio.
- `.hairline(false)` quita el divisor de abajo; la ventana de ajustes lo pone en
  su `ScrollView` (`.top_line(true)`).

El panel «Ventana y herramientas» de Tengen App Show muestra los semáforos
sueltos, una `Titlebar` con título y otra con `.close_only(true)`:

![Semáforos, barra de título y «solo cerrar»](assets/apps/tengen-ventana-light.png)

### Variantes en una ventana normal

La misma `Titlebar` cambia sola según la barra lateral, su estilo y el foco.
Ninguna de estas variantes la elige la app: salen de `with_sidebar`, de
`sidebar_style` en `appearance.domain` y de si la ventana está activa.

| Variante | Cómo se ve |
|---|---|
| Con barra lateral en isla (por defecto): los semáforos van en la barra lateral; la barra lleva la línea translúcida abajo | ![Titlebar con barra lateral en isla](assets/estados/titlebar-isla.png) |
| Con barra lateral en isla, ventana sin foco: semáforos y título en gris | ![Titlebar con barra lateral en isla sin foco](assets/estados/titlebar-isla-inactiva.png) |
| Con barra lateral plana: barra de `TITLEBAR_CLIENT_HEIGHT` con línea abajo | ![Titlebar con barra lateral plana](assets/estados/titlebar-plana.png) |
| Sin barra lateral: semáforos en la barra y título centrado entre dos zonas simétricas | ![Titlebar sin barra lateral](assets/estados/titlebar-sin-barra.png) |
| Sin barra lateral, ventana sin foco: fondo `titlebar_inactive` y semáforos grises | ![Titlebar sin barra lateral sin foco](assets/estados/titlebar-sin-barra-inactiva.png) |

### Radio de la esquina

La `Titlebar` dice al compositor qué barra dibuja con
`infinity_window_style_v1.set_bar`, y Malaysia redondea la ventana con el
radio de esa barra ([ADR 0059 de Malaysia](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0059-radio-por-barra-y-esquina-continua.md)).
La app no hace nada:

| `Titlebar` | Barra (`set_bar`) | Alto | Radio (`WINDOW_CORNER`) |
|---|---|---|---|
| `.close_only(true)` | `titlebar` | `TITLEBAR.thickness` (32) | `titlebar` (16) |
| `with_sidebar(true)` y barra lateral en isla | `unified` | `SETTINGS.toolbar_height` (52) | `unified` (26) |
| El resto | `compact` | `TITLEBAR_CLIENT_HEIGHT` (40) | `compact` (20) |

Si cambia `sidebar_style`, la barra vuelve a enviarse. `utility` (15) queda
para paneles. Una app que dibuja su propia barra sin `Titlebar` (CurseTerm)
la declara con `WindowControls::get().set_bar(Bar::Titlebar)`; si no, Malaysia
usa `unified`. El usuario puede fijar un radio para todas las ventanas con
`corner_radius` en `window.domain`.

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

## Dos tipos de titlebar: `WindowChrome`

`chrome::WindowChrome::new(estilo, contenido)` envuelve la ventana y coloca los
semáforos, el título y los elementos del rail según un `ChromeStyle`:

```rust
let estilo = ChromeStyle {
    layout: TitlebarLayout::Rail,
    rail_width: TITLEBAR_RAIL.width,
    round_lights: false,
    rail_style: RailStyle::Normal,
    reveal: Reveal::Always,
    resting_edge: RestingEdge::None,
};
WindowChrome::new(estilo, SplitView::new(barra, detalle))
    .rail_top(arriba)
    .rail_bottom(abajo)
```

| `TitlebarLayout` | Qué es | Captura (Tengen App Show) |
|---|---|---|
| `Top` (por defecto) | Titlebar horizontal arriba, con elementos a la izquierda y a la derecha (`Titlebar::left` y `right`) | ![Diseño Top](assets/apps/tengen-titlebar-top.png) |
| `Rail` | Titlebar vertical a la izquierda, con los semáforos en columna, el título girado 90° y elementos arriba y abajo (`rail_top` y `rail_bottom`) | ![Diseño Rail](assets/apps/tengen-titlebar-rail.png) |

El `Rail` tiene dos estilos:

| `RailStyle` | Qué es | Captura (Tengen App Show) |
|---|---|---|
| `Normal` (por defecto) | Una franja propia a la izquierda de la ventana | ![Rail normal](assets/apps/tengen-titlebar-rail.png) |
| `Unified` | Sin franja: la columna va dentro de la barra lateral. Sin barra lateral visible se dibuja como `Normal` | ![Rail unificado](assets/apps/tengen-titlebar-unified.png) |

### Elementos del rail

Igual que la `Titlebar` de `Top` lleva botones u otros elementos a izquierda y
derecha, el `Rail` los lleva en vertical:

| Método | Dónde va |
|---|---|
| `WindowChrome::rail_top(elementos)` | Bajo los semáforos |
| `WindowChrome::rail_bottom(elementos)` | Al pie del rail |

Con algún elemento, el rail se ensancha a `TITLEBAR_RAIL.actions_width` (34).
En `Unified` los elementos van en la columna de la barra lateral, en los mismos
sitios.

| `Normal` con elementos | `Unified` con elementos |
|---|---|
| ![Rail normal con elementos arriba y abajo](assets/estados/rail-elementos.png) | ![Rail unificado con elementos arriba y abajo](assets/estados/rail-unificado-elementos.png) |

### La barra lateral con el `Rail`

Con el `Rail`, los semáforos salen de la barra lateral y dejan libre su parte de
arriba. La `Sidebar` sube entonces su primera fila (en `PreferencesWindow`, el
buscador) hasta la fila de la titlebar, centrada con sus botones. La medida sale
de `sidebar_bar_metrics()` y de `TEXT_FIELD.height`, así que vale en isla y en
plana.

Al cambiar de `Top` a `Rail`, o al revés, la fila se desliza con
`DURATION_BASE` y una curva cúbica de salida. Con movimiento reducido, salta.
En pantalla completa no baja con la barra del compositor, como el resto de la
ventana con el `Rail`.

| `Top`: el buscador, bajo los semáforos | `Rail`: el buscador, en la fila de la titlebar |
|---|---|
| ![Barra lateral con Top](assets/estados/barra-lateral-top.png) | ![Barra lateral con Rail](assets/estados/barra-lateral-rail.png) |

### Campos de `ChromeStyle`

| Campo | Valores | Efecto |
|---|---|---|
| `rail_width` | `TITLEBAR_RAIL.width` o `.width_wide` | Ancho del rail sin elementos |
| `round_lights` | `bool` | Semáforos redondos o cuadrados (por defecto) |
| `rail_style` | `Normal`, `Unified` | Franja propia o columna dentro de la barra lateral |
| `reveal` | `Always`, `Drawer`, `Push`, `Corner`, `Popup` | Siempre visible, o aparece al acercar el puntero: como cajón encima, empujando el contenido, solo en la esquina o como píldora emergente con los semáforos |
| `resting_edge` | `None`, `Line`, `Accent` | Qué queda visible cuando está oculta: nada, una línea o una línea de acento |

Algunas combinaciones no tienen sentido y se corrigen solas
(`ChromeStyle::effective`): el `Rail` unificado con `Drawer`, `Push` o `Popup`
pasa a `Corner`; en pantalla completa, `reveal` es siempre `Always`.

Los paneles Titlebar, Pantalla completa y Barra lateral de Tengen App Show
(capítulo [15](15-apps-de-prueba.md)) prueban todas las combinaciones en vivo.

## Maximizada, pantalla completa y modo foco

El estado se lee como explica el capítulo [6](06-ventanas.md#estado-de-la-ventana).
Las capturas de esta sección son una ventana de prueba (barra lateral en isla,
diseño `Top` salvo donde se indica) renderizada por el kit en cada estado.

### Los cuatro estados con el diseño por defecto (`Top`)

| Estado | Cómo se entra | Qué hace el kit | Cómo se ve |
|---|---|---|---|
| Normal | — | Titlebar en su sitio, semáforos en la barra lateral. Los bordes de la ventana la redimensionan. Al cerrar se guarda el tamaño | ![Ventana normal](assets/estados/estado-normal.png) |
| Maximizada | Botón verde con `green_button = "zoom"`, doble clic con `zoom` o `fill`, o `toggle_maximize()` | Lo mismo que en normal, sin bordes de redimensionado. En Malaysia el compositor conserva el marco (esquinas y sombra). Al cerrar se guarda que estaba maximizada y vuelve a abrirse así | ![Ventana maximizada: igual que la normal](assets/estados/estado-maximizada.png) |
| Pantalla completa sin el protocolo de foco (otro compositor) | Botón verde con `focus` (por defecto) o `fullscreen`, o `toggle_fullscreen()` | La titlebar se queda en su sitio y se ocultan los semáforos | ![Pantalla completa sin protocolo: sin semáforos](assets/estados/estado-completa-sin-protocolo.png) |
| Modo foco de Malaysia, barra arriba | Igual; el compositor da el estado `fullscreen` | La titlebar sale del diseño y el contenido ocupa toda la ventana | ![Modo foco con la barra arriba: solo contenido](assets/estados/estado-foco-barra-arriba.png) |

En Malaysia, el modo foco
([ADR 0051 y 0054 de Malaysia](https://github.com/HBlanqueto/malaysia/tree/main/docs/decisiones))
baja una barra negra al acercar el puntero arriba. El kit pide `set_overlay`
por `infinity_focus_v1` v2, así que la ventana no se mueve: mientras la barra
está bajada, `titlebar_overlaid()` es verdadero y la titlebar se dibuja encima
del contenido, justo debajo de la barra, sobre el fondo que elige la app
(`FullscreenBar`, abajo). Aparece y desaparece con la barra en
`DURATION_BASE`. En pantalla completa, `reveal` es siempre `Always`.

### La barra en pantalla completa: `FullscreenBar`

Cada ventana elige cómo se dibuja su titlebar superpuesta. La opción va en
`WindowOptions` y vale para toda la titlebar superpuesta:

```rust
WindowOptions::new("Ondina", "ondina").fullscreen_bar(FullscreenBar::ProgressiveGlass)
```

**Por defecto, todas las apps usan `Solid` («A · Lisa»).** Solo Ondina pide
`ProgressiveGlass`. Las tres salen de la propuesta «Ondina a pantalla
completa»; «C · Degradado» se descartó. En las capturas, la franja negra de arriba
es la barra del compositor, añadida a la imagen.

| `FullscreenBar` | Qué dibuja | Dónde empieza el contenido (`fullscreen_content_top`) | Cómo se ve |
|---|---|---|---|
| `Solid` («A · Lisa», **por defecto**) | Una banda opaca del color de la ventana, del alto de la titlebar. Corta lo que pasa por debajo con un borde duro | Barra + `FULLSCREEN_BAR.content_top` (44) | ![A · Lisa](assets/estados/barra-lisa.png) |
| `Glass` («B1 · Vidrio en banda») | Una banda del alto de la titlebar, de borde a borde de la ventana y sobre la barra lateral, que difumina lo de debajo (`FULLSCREEN_BAND`), con tinte y una línea abajo | Barra + `FULLSCREEN_BAND.content_top` (44) | ![B1 · Vidrio en banda](assets/estados/barra-vidrio-banda.png) |
| `ProgressiveGlass` («B2 · Vidrio progresivo», Ondina) | Difuminado máximo arriba que se desvanece en 96 px, sin línea, de borde a borde de la ventana y sobre la barra lateral ([ADR 0057](../decisiones/0057-cristal-y-barra-lateral-sobre-fondo.md)) | Barra + `FULLSCREEN_GLASS.content_top` (74) | ![B2 · Vidrio progresivo](assets/estados/barra-vidrio-progresivo.png) |

Solo `Glass` y `ProgressiveGlass` difuminan.

#### Empujar el contenido: `fullscreen_push`

«E · Empuja» no es una barra más, sino una opción que se combina con cualquiera
de las tres. Por defecto está apagada:

```rust
WindowOptions::new("Ondina", "ondina")
    .fullscreen_bar(FullscreenBar::ProgressiveGlass)
    .fullscreen_push(true)
```

Las dos son el valor inicial. La app las cambia en vivo con
`WindowControls::get().fullscreen_bar` y `.fullscreen_push`, dos `State`.

Con ella, la barra no tapa nada: todo el contenido de la ventana, barra lateral
incluida, baja lo que miden la barra del compositor y la titlebar
(`WindowControls::push_offset`). Los semáforos bajan con la titlebar y
`fullscreen_content_top` es 0.

| Combinación | Cómo se ve |
|---|---|
| `Solid` + `fullscreen_push(true)` | ![A · Lisa empujando el contenido](assets/estados/barra-empuja.png) |
| `ProgressiveGlass` + `fullscreen_push(true)` | ![B2 · Vidrio progresivo empujando el contenido](assets/estados/barra-empuja-vidrio-progresivo.png) |
| Barra arriba: el contenido llega al borde superior | ![Empuja con la barra arriba](assets/estados/barra-empuja-arriba.png) |

#### Con el `Rail`

Con el `Rail`, la titlebar superior no se retrae ni se superpone: se queda
en su sitio, como en la ventana normal, y nada baja con la barra del
compositor. Solo el rail se oculta con la barra arriba y, al bajar, entra desde
la izquierda con la misma barra, en vertical, con sus semáforos bajo la barra:

| `FullscreenBar` | El rail | Cómo se ve |
|---|---|---|
| `Solid` («A · Lisa», **por defecto**) | Franja opaca del ancho del rail, encima del contenido | ![Rail con A · Lisa](assets/estados/rail-barra-lisa.png) |
| `Glass` («B1 · Vidrio en banda») | Franja del ancho del rail que difumina lo de debajo, con una línea en el borde interior | ![Rail con B1 · Vidrio en banda](assets/estados/rail-barra-vidrio-banda.png) |
| `ProgressiveGlass` («B2 · Vidrio progresivo») | Difuminado máximo en el borde de la ventana que se desvanece hacia dentro en 96 px | ![Rail con B2 · Vidrio progresivo](assets/estados/rail-barra-vidrio-progresivo.png) |
| Cualquiera + `fullscreen_push(true)` | El rail vuelve al diseño y empuja el contenido de lado lo que mide | ![Rail empujando el contenido](assets/estados/rail-barra-empuja.png) |

### Modo foco con la barra bajada, por diseño

**El diseño se mantiene al maximizar y en pantalla completa**, también con la
barra lateral en isla: el `Rail` sigue a la izquierda.

| `Rail` con isla, maximizada | `Rail` con isla, modo foco |
|---|---|
| ![Rail con isla y la ventana maximizada](assets/estados/isla-rail-maximizada.png) | ![Rail con isla en modo foco](assets/estados/isla-rail-foco.png) |

Las capturas de esta tabla usan la barra lateral plana (salvo `Top`) y la barra por
defecto (`Solid`). La franja negra de arriba (30 px) es la barra del
compositor, añadida a la imagen: la app no la dibuja.

| Diseño | Qué baja | Cómo se ve |
|---|---|---|
| `Top` (por defecto) | La `Titlebar` sobre el contenido; los semáforos de la barra lateral bajan lo mismo | ![Top con la barra bajada](assets/estados/foco-top.png) |
| `Rail` unificado | La columna de la barra lateral; los semáforos aparecen con la barra, debajo de ella | ![Rail unificado con la barra bajada](assets/estados/foco-unified.png) |
| `Rail` | El rail entra desde la izquierda con la barra elegida (arriba, «Con el `Rail`»); sus semáforos quedan bajo la barra del compositor | ![Rail con la barra bajada](assets/estados/rail-barra-lisa.png) |
| `Rail` con elementos | Igual, con sus elementos | ![Rail con elementos y la barra bajada](assets/estados/foco-rail-elementos.png) |

### Qué hace la app

1. Si usa `Titlebar`, `WindowChrome`, `Sidebar` o `PreferencesWindow`, no tiene
   que hacer nada: todos los estados de las tablas ya funcionan.
2. Deja hueco para la barra en el contenido que va bajo la titlebar
   superpuesta, como Ondina:

   ```rust
   let arriba = try_consume_root_context::<WindowControls>()
       .map_or(0.0, |controls| controls.fullscreen_content_top());
   rect().padding(Gaps::new(arriba, 0.0, 0.0, 0.0)).child(contenido)
   ```

3. Si una vista tapa la titlebar (un reproductor de vídeo a toda la ventana),
   pone sus propios semáforos para que se pueda maximizar o salir de pantalla
   completa sin ella, como el reproductor de Marea:

   ```rust
   window_traffic_lights(WindowControls::get().status.read().activated, false)
   ```

4. Para un comportamiento propio (un modo inmersivo que esconde paneles),
   lee el estado y decide:

   ```rust
   let controls = WindowControls::get();
   let inmersivo = controls.status.read().fullscreen;
   rect().maybe_child((!inmersivo).then(panel_de_herramientas))
   ```

5. Para ofrecerlo en un menú, llama a `toggle_maximize()` o
   `toggle_fullscreen()` desde una `Action` (capítulo [11](11-app.md)).

Siguiente: [8. Barra lateral y SplitView](08-barra-lateral.md).
