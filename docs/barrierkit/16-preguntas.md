# 16. Preguntas frecuentes

**Objetivo:** responder rápido a lo que más se pregunta del kit.
**Lector:** cualquiera. Cada respuesta enlaza al capítulo que la amplía.

## ¿Qué tenemos hoy?

| Capa | Piezas |
|---|---|
| 0 · Plataforma | Ventanas y popups, foco y Tab, cursor por componente, escala fraccional, portapapeles de texto, región opaca, AT-SPI, Vulkan |
| 1 · Primitivas | `Palette`, `text`, `Symbol`, `use_interaction`, `Button`, `IconButton`, `Link`, `Switch`, `Checkbox`, `RadioGroup`, `SegmentedControl`, `PopUpButton`, `ColorWell`, `ChoiceCards`, `ToggleGroup`, `Stepper`, `Slider`, `LevelSlider`, `TextField`, `SearchField`, `SecureField`, `ProgressBar`, `ProgressRing`, `Spinner`, `badge`, `divider`, `avatar`, `use_tooltip`, `context_menu`, `ScrollView`, `MenuView`, `Titlebar`, `TrafficLights`, `WindowChrome`, `Sidebar`, `Backdrop`, `media`, `video`, `use_breakpoint`, `Spring` |
| 2 · Vistas | `SplitView`, `Form`/`Section`/`Row`, `Table`, `CollectionView`, `TabView`, `History` + `navigation_buttons`, `SearchIndex`, `empty_state`, `sheet`, `Alert` |
| 3 · Patrones | `PreferencesWindow`, `about_window` |
| App | `Action` + `Shortcut`, `AppMenu`, `launch`, `load_defaults`/`save_defaults`/`use_defaults`, restauración, `use_undo` |

## ¿Qué falta?

Lo pendiente está en el [ADR 0047](../decisiones/0047-barrierkit-en-capas.md),
«Pendiente». En resumen:

- **Bloqueado por Freya 0.4.3:** damage parcial.
- **Espera a la primera app que lo pida:** `Image` propio, búsqueda y
  desbordamiento en la titlebar, qué hacer al cerrar la última ventana y
  restaurar la navegación y el desplazamiento.
- **Depende del compositor:** restaurar la posición de la ventana.

## ¿Usa GPU o CPU?

El dibujo va por Vulkan, normalmente en la GPU; el diseño, en la CPU. Un driver
Vulkan por software (lavapipe) también sirve. El vídeo usa VA-API si existe y,
si no, la CPU. Ver [1](01-que-es.md#usa-gpu-o-cpu) y [13](13-fondos-y-medios.md#vídeo-feature-video).

## ¿Funciona fuera de Malaysia?

Sí, como cliente Wayland normal (ver
[14](14-plataforma.md#el-primer-frame)). Fuera de
Malaysia no hay bordes, sombra ni blur del compositor, y el menú global
solo existe en D-Bus.

## ¿Puedo usar Freya directamente?

Sí, `rect`, `label` y los hooks de `freya-core` están disponibles. Pinta con la
`Palette` y los tokens. No uses los componentes de `freya-components` con su
estilo por defecto: el kit ya tiene su versión con aspecto Tengen.

## ¿Cómo pongo mis propios colores?

Con variantes: `Tint`, `Accent` y `ColorWell`. Un valor nuevo para todos va a
Tengen. Ver [4](04-apariencia-y-tokens.md#colores-propios-en-una-app).

## ¿Cómo hago que la barra lateral no se redimensione o tenga otros límites?

`SplitView::resizable(false)` y `SplitView::sidebar_range(mín, máx)`; en
`PreferencesWindow`, `sidebar_resizable` y `sidebar_range`. Ver
[8](08-barra-lateral.md).

## ¿Dónde guarda una app sus datos?

Preferencias en `~/.config/malaysia/<app_id>.domain`; estado de ventana en
`~/.local/share/malaysia/<app_id>.state`. Ver [11](11-app.md#preferencias-y-restauración).

## ¿Cómo añado un componente al kit?

Sigue [5, «Crear un control nuevo»](05-controles.md#crear-un-control-nuevo).
Lleva su test de render en claro y oscuro y, si es interactivo, su test de
teclado.

## ¿Cómo compruebo un cambio?

```sh
nix develop
./dev.sh check
```

`./dev.sh check` pasa `fmt`, `clippy`, los scripts de tokens y de dependencias
y los tests. Si un cambio visual es intencionado, regenera las imágenes de
referencia con `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
