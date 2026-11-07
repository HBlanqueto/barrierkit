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
        .size((760, 520))
        .min_size(Some(WINDOW_MIN_SIZE))
        .menu(menu),
)
```

| Opción | Efecto |
|---|---|
| `new(título, nombre)` | Título visible y nombre corto. El `app_id` es `io.github.hblanqueto.malaysia.<nombre>` salvo que pases `.app_id(..)` |
| `.size((ancho, alto))` | Tamaño inicial (640 × 420 por defecto). Si hay uno guardado, se restaura |
| `.min_size(..)`, `.max_size(..)` | Límites. Si son iguales, la ventana es de tamaño fijo |
| `.menu(AppMenu)` | Menú global de la ventana (capítulo [11](11-app.md)) |
| `.fullscreen_bar(FullscreenBar)` | Cómo se dibuja la titlebar superpuesta en pantalla completa; `Solid` por defecto (capítulo [7](07-titlebars.md#la-barra-en-pantalla-completa-fullscreenbar)) |
| `.fullscreen_push(bool)` | En pantalla completa, la barra empuja todo el contenido en vez de taparlo; apagado por defecto (capítulo [7](07-titlebars.md#empujar-el-contenido-fullscreen_push)) |

Dentro de la app, `WindowControls::get()` da el estado (`activated`,
`maximized`, `fullscreen`, `decoration`, `fixed`) y las órdenes: `close()`,
`minimize()`, `toggle_maximize()`, `toggle_fullscreen()`, `move_window()`,
`resize(WindowEdge)`, `set_title(título)`, que cambia el título que ven el
compositor y la barra de menús, `set_background_alpha(alfa)`, que multiplica
el alfa del fondo con que el runner limpia la ventana (1 es opaco), y
`open_window(app, opciones)`, que abre otra ventana en el mismo proceso; el
proceso termina al cerrarse la última.

Con `zwp_text_input_v3`, el texto en composición del método de entrada llega a
`WindowControls::get().ime_preedit` (un `State<String>`, vacío si no hay) y el
texto confirmado llega como una tecla `Key::Character`.
`set_ime_cursor_area(x, y, ancho, alto)` dice al método de entrada dónde está
el cursor, en píxeles lógicos de la ventana.

`primary_selection()` da un `ClipboardProvider` para la selección primaria
(`zwp_primary_selection`), la que pega el botón central: `set_contents` la
publica y `get_contents` lee la de otra app. `copy_rich(texto, html)` copia al
portapapeles el texto plano y además `text/html`, para que las apps que lo
entienden peguen con formato.

## Ventanas auxiliares

```rust
open_utility_window(AppComponent::new(Inspector), WindowOptions::new("Inspector", "Vista").size((260, 360)));
```

Una app abre paneles flotantes, como `UtilityWindow` de SwiftUI o el inspector
de Vista Previa, con `barrier_kit::platform::open_utility_window`:

1. Se llama desde un componente de la ventana que la abre.
2. El runtime crea otra `xdg_toplevel` con `set_parent` hacia esa ventana, así
   que el compositor la mantiene encima y la agrupa con ella.
3. Usa el `app_id` de la ventana madre y no guarda ni restaura su tamaño, para
   no pisar el de la principal.
4. Cerrarla solo la quita a ella; la app sigue mientras quede otra ventana.
5. Antes del primer commit se declara panel (`set_panel`, versión 3 de
   `infinity_window_style_v1`) con su `WindowOptions::panel`. Por defecto,
   `PanelBehavior::UTILITY`, lo mismo que `UtilityWindow` de SwiftUI: se
   oculta mientras la app no está activa
   ([`hidesOnDeactivate`](https://developer.apple.com/documentation/appkit/nswindow/hidesondeactivate)),
   flota sobre las ventanas de su app y solo toma el teclado cuando hace falta
   ([`becomesKeyOnlyIfNeeded`](https://developer.apple.com/documentation/appkit/nspanel/becomeskeyonlyifneeded)).
   Un panel nunca es la ventana principal: mientras tiene el teclado, la
   ventana de la que salió sigue activa. Lo aplica el compositor; sin la
   versión 3, la ventana se comporta como una normal.

`WindowOptions::hud(true)` le da el estilo HUD de `NSPanel`
(`hudWindow`): un panel «más oscuro y translúcido» que la HIG reserva para
apps de fotos, vídeo o presentaciones
([Panels](https://developer.apple.com/design/human-interface-guidelines/panels)):

1. La ventana se borra con `HUD_BACKGROUND`, que es translúcido, así que no
   declara región opaca y el compositor desenfoca lo que hay detrás.
2. Todo su contenido usa el esquema oscuro (`use_appearance`), sea cual sea el
   de la app.
3. Su `Titlebar` pasa a `close_only`: barra fina, fondo transparente y solo
   cerrar activo, porque la HIG pide no ofrecer minimizar en un panel.
4. El contenido no debe pintar fondo propio para que se vea el material.
   Apple no publica el color del material; `#1e1e1e` al 85 % es una
   aproximación.

## Icono en la barra de menús

```rust
let icono = StatusItem::new("correo", "Correo", "mail-unread")
    .items(vec![MenuItem::action("nuevo", "Nuevo mensaje")]);
run_window(app, WindowOptions::new("Correo", "Correo").status_item(icono))?;
use_menu_action(|id| if id == STATUS_ITEM_ACTIVATE { /* mostrar la ventana */ });
```

`WindowOptions::status_item` publica un icono de estado, el `NSStatusItem`
de AppKit, con el protocolo
[StatusNotifierItem](https://specifications.freedesktop.org/status-notifier-item/latest/):

1. El kit exporta `org.kde.StatusNotifierItem` en `/StatusNotifierItem`, con
   el nombre `org.kde.StatusNotifierItem-<pid>-<n>`, título, `IconName` del
   tema de iconos y su menú como dbusmenu.
2. Se registra en `org.kde.StatusNotifierWatcher`. Si el watcher aparece
   después (la shell arranca más tarde o se reinicia), vuelve a registrarse.
3. Cada elemento del menú llega con su `id` a `use_menu_action`, igual que el
   menú de la app. `Activate` llega como `STATUS_ITEM_ACTIVATE`.
4. El menú y el objeto comparten conexión, porque la especificación busca el
   `Menu` en el mismo nombre de bus que el icono.

La especificación escribe las interfaces como `org.freedesktop.*`, pero KDE,
libayatana-appindicator y waybar usan `org.kde.*`, y el kit usa esos nombres.

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
| `focus` (por defecto) | Pantalla completa | Maximiza |
| `zoom` | Maximiza | Maximiza |
| `fullscreen` | Pantalla completa | Maximiza |
| `none` | Nada; el botón se dibuja gris y sin glifo | Nada; el botón se dibuja gris y sin glifo |

| `double_click_titlebar` | Ventana normal | Tamaño fijo |
|---|---|---|
| `zoom` (por defecto), `fill` | Maximiza | Nada |
| `minimize` | Minimiza | Minimiza |
| `none` | Nada | Nada |

Las dos tablas viven en `vow_appearance::window`: `read_window_behavior()` lee
el dominio y `green_action` y `double_click_action` dan la acción. El chrome del
compositor usa las mismas funciones.

## Popups

Los menús, desplegables y ayudas usan `xdg_popup`. Para uno propio:

```rust
let id = Popups::get().open(spec, AppComponent::new(MiPopup));
Popups::get().close(id);
```

`PopupSpec` fija el rectángulo de anclaje, el ancla, la gravedad, el tamaño, el
ajuste si no cabe y si toma el grab del puntero.

Siguiente: [7. Titlebars](07-titlebars.md).
