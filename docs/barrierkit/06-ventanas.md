# 6. Ventanas

**Objetivo:** abrir ventanas con las opciones correctas y controlar su tamaño,
su estado y sus popups.
**Lector:** quien define la ventana principal de una app.
**Ver también:** el aspecto de la titlebar y de los semáforos está en el
capítulo [7](07-titlebars.md).

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
| `.menu(AppMenu)` | Menú global de la ventana (capítulo [11](11-app.md)) |
| `.fullscreen_bar(FullscreenBar)` | Cómo se dibuja la titlebar superpuesta en pantalla completa; `Solid` por defecto (capítulo [7](07-titlebars.md#la-barra-en-pantalla-completa-fullscreenbar)) |

Dentro de la app, `WindowControls::get()` da el estado (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) y las órdenes: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`.

## Estado de la ventana

El estado llega en cada configure del compositor y se lee con
`WindowControls::get().status.read()`: `activated`, `maximized`, `fullscreen`,
`decoration` y `fixed`. Cómo se ve la titlebar en cada estado está en el
capítulo [7](07-titlebars.md#maximizada-pantalla-completa-y-modo-foco).

## Redimensionar por los bordes

Con decoración `Client`, el runner reserva dentro de la ventana una banda de
`WINDOW_RESIZE.edge` (5 px) en cada lado y de `WINDOW_RESIZE.corner` (16 px) a
lo largo de cada esquina:

1. Al pasar por la banda, el cursor cambia a la flecha del borde o la esquina y
   la app deja de recibir el movimiento.
2. Al pulsar con el botón izquierdo, el compositor empieza a redimensionar
   (`xdg_toplevel.resize`).
3. No hay banda si la ventana está maximizada, en pantalla completa, es de
   tamaño fijo o usa decoración del servidor, ni mientras se arrastra algo de
   la app.

## El botón verde y el doble clic

Los decide el usuario en `window.domain`. Con una ventana de tamaño fijo
(mínimo igual a máximo) cambian:

| `green_button` | Ventana normal | Tamaño fijo |
|---|---|---|
| `focus` (por defecto) | Pantalla completa | Maximiza en vertical |
| `zoom` | Maximiza | Maximiza en vertical |
| `fullscreen` | Pantalla completa | Maximiza en vertical |
| `none` | Nada; el botón se dibuja gris y sin glifo | Nada; el botón se dibuja gris y sin glifo |

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

Siguiente: [7. Titlebars](07-titlebars.md).
