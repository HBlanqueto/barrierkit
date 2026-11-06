# 5. Controles

**Objetivo:** conocer cada elemento interactivo de la capa 1, su código y su
aspecto.
**Lector:** quien construye la interfaz de una app.

Cada imagen es la referencia de render del kit (`crates/barrier-kit/tests/golden/`),
generada por el fragmento que la acompaña en `tests/kit/controls.rs`. Todas
existen en claro y en oscuro; aquí se muestra el claro y el oscuro va al lado
cuando aporta. En vivo, todos están en la app Tengen App Show (capítulo [15](15-apps-de-prueba.md)).

## Reglas comunes

- El primer argumento de `new` es la **etiqueta accesible**.
- Son **controlados**: la app pasa el valor y recibe el cambio.
- Todos responden al teclado: Tab mueve el foco y dibuja el anillo de foco;
  Espacio o Intro activan; las flechas mueven deslizadores y segmentos.
- Los que tienen relleno aceptan `.tint(Tint::Accent | Tint::Color(c) | Tint::Backdrop)`.

![Anillo de foco al navegar con Tab](assets/componentes/focus-ring-light.png)

## Botones

Módulos `controls` y `compact`.

```rust
Button::new("Cancelar")
Button::new("Guardar").style(ButtonStyle::Primary)
Button::new("Borrar").disabled(true)
```

| Claro | Oscuro |
|---|---|
| ![Botones Cancelar, Guardar y Borrar](assets/componentes/button-light.png) | ![Botones en oscuro](assets/componentes/button-dark.png) |

- `ButtonStyle`: `Secondary` (por defecto) y `Primary`.
- Tamaño normal: 24 de alto, radio 6 y 12 de margen lateral (`button`), el `regular` de AppKit en macOS 26.
- `.small(true)`: 20 de alto, 11 de margen lateral y texto de 11 (`button.small`), como el de «Acerca de este PC».
- `.large(true)`: 28 de alto en cápsula (`button.large`), el `large` de AppKit; lo usa `Alert`.
- `.width(Size)` y `.on_press(|_| ..)`. Al pulsar, el botón se escala a
  `BUTTON.pressed_scale` (0,97).

```rust
IconButton::new(Symbol::Plus, "Añadir")
Stepper::new("Núcleos", 1, 16).value(4)
ToggleGroup::new("Formato")
    .item("B", Face::Bold, "Negrita", true)
    .item("I", Face::Italic, "Cursiva", false)
    .item("U", Face::Underline, "Subrayado", false)
```

![Botón de icono, stepper y grupo conmutable](assets/componentes/compact-light.png)

`IconButton` mide 30 (`icon-button`). Dentro de un `ToolGroup` o de una
barra unificada (52) pasa a una cápsula de 36, la altura de `TOOLGROUP`,
como los elementos de toolbar de macOS 26.

`Stepper` muestra el valor y, a 8, las flechas de AppKit: 20 × 26, radio 6,
subir arriba y bajar abajo (`stepper`). Las flechas del teclado también
cambian el valor.

`Link::new("Más información…")` es un enlace de texto con `on_press`.

## Interruptor y casilla

```rust
Switch::new("Apagado")
Switch::new("Encendido").on(true)
Switch::new("Inactivo").on(true).disabled(true)
Checkbox::new("Recordarme").checked(true)
```

| Interruptores | Casilla |
|---|---|
| ![Tres interruptores](assets/componentes/switch-light.png) | ![Casilla marcada](assets/componentes/checkbox-light.png) |

`Switch` avisa con `.on_toggle(|on: bool| ..)`; `Checkbox` igual. El pomo del
interruptor se anima en `SWITCH.transition_ms`.

## Elección

```rust
RadioGroup::new("Aplicar el tinte a")
    .option("Solo las ventanas")
    .option("Todo el escritorio")

SegmentedControl::new("Periodo")
    .segment("Día").segment("Semana").segment("Mes")
    .selected(1)

PopUpButton::new("Semáforos")
    .item("Automático").item("Clásico").item("Acento")
    .selected(1)

ColorWell::new("Color de acento", &ACCENT_SWATCHES).selected(Some(0))
```

| Radio | Segmentado |
|---|---|
| ![Grupo de radio activo e inactivo](assets/componentes/radio-group-light.png) | ![Control segmentado con Semana elegido](assets/componentes/segmented-control-light.png) |

| Desplegable | Desplegable con icono | Muestras de color |
|---|---|---|
| ![PopUpButton](assets/componentes/popup-button-light.png) | ![PopUpButton con icono SVG](assets/componentes/popup-button-icon-light.png) | ![ColorWell con ocho acentos](assets/componentes/color-well-light.png) |

- `PopUpButton` abre un `MenuView` como popup; `.icon_item(texto, MenuIcon::svg(..))`
  añade un icono, y `MenuIcon::Swatch(color)`, un círculo de color.
- En un menú, la marca tiene una columna para todo el menú, solo si alguna
  fila la usa. El icono solo desplaza el texto de su fila, como en macOS; una
  fila marcada conserva su icono.
- `ChoiceCards::new(..).option(rótulo, vista_previa)` muestra tarjetas grandes
  (como «Claro, Oscuro, Acento» en Tengen App Show).

## Deslizadores

```rust
Slider::stepped(5).value(2).labels(&["A", "B", "C", "D", "E"])
LevelSlider::new("Brillo").value(0.3)
LevelSlider::new("Volumen").value(0.6).style(LevelStyle::Fat(Symbol::SpeakerWave))
```

| Por pasos | Continuo y grueso |
|---|---|
| ![Slider por pasos](assets/componentes/slider-light.png) | ![LevelSlider normal y grueso](assets/componentes/level-slider-light.png) |

`Slider` trabaja con índices (`usize`) y `LevelSlider` con `f32` entre 0 y 1.

## Campos de texto

```rust
let nombre = use_state(|| "hblanqueto".to_string());
TextField::new(nombre, "Nombre de usuario")
SearchField::new(consulta, "Buscar").auto_focus(true)
let clave = use_state(PasswordField::default);
SecureField::new(clave).on_submit(move |valor| entrar(valor))
ComboBox::new(shell, "Intérprete").item("zsh").item("fish")
TextEditor::new(notas, "Notas").height(Size::px(72.0))
```

| Texto y búsqueda | Contraseña |
|---|---|
| ![TextField y SearchField](assets/componentes/text-field-light.png) | ![SecureField](assets/componentes/secure-field-light.png) |

`TextField` y `SearchField` escriben directamente en un `State<String>`. Con
el foco en un campo, Ctrl+A, C, V, X, Y y Z son del campo antes que del menú.
`SearchField::auto_focus(true)` toma el foco al montarse.

`SecureField` guarda la contraseña en un `State<PasswordField>`
(`barrier_kit::password`) que crea la app. Enter llama a `on_submit` con el
valor en un `Zeroizing<String>`. Quien dibuja su propio campo pasa cada tecla a
`PasswordField::handle_key(&key, modifiers)`: devuelve el valor al pulsar Enter.

`ComboBox` (`barrier_kit::combo_box`) es un campo de texto con un botón de
flecha, como `NSComboBox`: se escribe libremente o se elige de la lista, que
abre debajo con el menú del kit. Mide lo que el render de AppKit en macOS 26
([native-theme](https://github.com/tiborgats/native-theme)): botón de 24 × 20
a 4 del borde, esquina `radius.xs` y 6 entre el texto y el botón
(`components.combo-box`).

`TextEditor` (`barrier_kit::text_editor`) edita varias líneas, como
`NSTextView`. Enter añade una línea, Escape quita el foco y el texto se
desplaza dentro del alto que le da `.height(..)` (por defecto, todo el
contenedor). Usa el motor de edición de Freya (`freya-edit`) y el aspecto de
`TextField`.

`TokenField` (`barrier_kit::token_field`), como `NSTokenField`, guarda
etiquetas en un `State<Vec<String>>`: Enter o una coma convierten el texto en
una cápsula y Retroceso con el campo vacío borra la última. `PathControl`
(`barrier_kit::path_control`), como `NSPathControl`, muestra las partes de
una ruta separadas por chevrones en una fila de 22 y avisa con `on_select` de
la parte pulsada. La altura de 22 sale de xib de Xcode; la cápsula (18 de
alto, 7 de margen y el acento al 20 %) es una estimación: no hay un probe de
macOS 26 para estos dos controles.

## Progreso

```rust
ProgressBar::determinate("Copiando sistema", 0.62)
ProgressBar::indeterminate("Buscando redes")
ProgressRing::new("Descargando", 0.4)
Spinner::new("Cargando")
```

![Barra, barra indeterminada, anillo e indicador](assets/componentes/progress-light.png)

```rust
CapacityIndicator::new("Disco", 0.82).thresholds(0.75, 0.9)
RatingIndicator::new("Valoración", 3).on_change(move |v| estrellas.set(v))
```

`barrier_kit::indicator` sigue a `NSLevelIndicator`. `CapacityIndicator` es
una barra de `LEVEL_INDICATOR.height` (16) que pasa de verde a amarillo en
el umbral de aviso y a rojo en el crítico (`LEVEL_INDICATOR_COLORS`).
`RatingIndicator` muestra 5 estrellas de 12; un clic en una fija ese valor.
Las medidas salen de xib de Xcode y de
[macos_ui](https://pub.dev/packages/macos_ui); no hay un probe de macOS 26.

## Adornos

```rust
badge("2")
divider()
avatar("HB")
Link::new("Más información…")
```

![Insignia, separador, avatar y enlace](assets/componentes/adornments-light.png)

```rust
GroupBox::new(Some("Red")).child(Checkbox::new("Wi-Fi"))
DisclosureGroup::new("Avanzado").expanded(true).child(Checkbox::new("Wi-Fi"))
```

`barrier_kit::views::group` tiene `GroupBox` (`NSBox`, `GroupBox` de
SwiftUI) y `DisclosureGroup`. La caja tiene margen interior 5 (el
`contentViewMargins` por defecto de `NSBox`) y esquina 11, la que Qt usa para
la caja de macOS 26 ([qmacstyle_mac.mm](https://github.com/qt/qtbase/blob/22c02af96a9c0bdcfac956231bf706d6189fab07/src/plugins/styles/mac/qmacstyle_mac.mm#L139)).
El grupo desplegable sangra el contenido 13, el `indentationPerLevel` de
`NSOutlineView` en macOS 26.6.2 medido por
[native-theme](https://github.com/tiborgats/native-theme), con una
cabecera de 24 (su `rowHeight`). Un clic o Espacio en la cabecera lo abre o
lo cierra.

## Popover

```rust
show_popover(area_del_boton, (240.0, 32.0), || text("Hola", TEXT, color).into_element());
```

`show_popover` (`barrier_kit::popover`) abre un popup bajo el área que se le
pasa, con el tamaño del contenido más `POPOVER.padding` y la banda de la flecha.
La app pide con `infinity_window_style_v1.set_popover` (versión 2) que Malaysia
le dé la forma de `NSPopover` en macOS 26: esquina continua de 20 y una flecha
de 25.42 × 13 con punta redonda, unida al cuerpo con un redondeo, que apunta
al centro del área. El compositor recorta, bordea y sombrea esa forma
([ADR 0059](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0059-radio-por-barra-y-esquina-continua.md));
la app solo pinta el fondo. Las medidas son las que el paquete Flutter
[`popover`](https://pub.dev/packages/popover) sacó de AppKit en macOS 26.7.
Un clic fuera lo cierra.

## Ayuda emergente y menú contextual

```rust
let ayuda = use_tooltip("Añadir una carpeta");
ayuda.attach(IconButton::new(Symbol::Plus, "Añadir"))

context_menu(fila, elementos, |id: String| { /* acción */ })
```

- `use_tooltip` abre un popup sin grab bajo el puntero.
- `context_menu` abre un `MenuView` en el puntero con el clic secundario.
  `elementos` es un `Vec<MenuItem>`; `Action::item(true)` crea uno desde una
  acción (capítulo [11](11-app.md)).
- `MenuItem::action(id, texto)`, `::submenu(texto, elementos)`,
  `::header(texto)` y `::placeholder(texto)` crean los demás; `.with_shortcut`,
  `.with_enabled`, `.with_check` y `.with_icon` los ajustan.
- `menu::normalize_query(texto)` prepara una búsqueda como lo hace el menú, y
  `menu::submenu_contents(cargando, elementos)` muestra «Cargando…» en un
  submenú que aún llega vacío.

## Desplazamiento

```rust
ScrollView::new().height(Size::px(120.0)).child(lista)
```

![ScrollView con tres elementos](assets/componentes/scroll-view-light.png)

`ScrollView` admite `.direction(Direction::Horizontal)`, `.scroll_bar(false)`
y un `use_scroll_controller()` para desplazar desde el código. La barra se
desvanece en `DURATION_FAST_MS` salvo con `scroll_bars = "always"`, que la
muestra con carril de 11 px. `.top_line(true)` dibuja el divisor encima del
contenido, que pasa por debajo al desplazarse.

## Símbolos

`Symbol` es el registro de iconos del kit (31 hoy): `MagnifyingGlass`,
`XMark`, `Plus`, `Minus`, `Chevron*`, `SidebarLeft`, `SpeakerWave`, `Play`,
`Pause`, `House`, `Star`, `Clock`, `Checkmark`… `symbol(Symbol::Plus, 16.0, color)`
lo dibuja suelto; los controles reciben el `Symbol` directamente.

## Crear un control nuevo

1. Busca si ya existe (regla 1 de `AGENTS.md`); un control nuevo compone
   primitivas del kit.
2. Usa `use_interaction()` y `interaction.attach(rect(), habilitado)` para
   hover, pulsado y foco; `interaction.focus_ring(..)` para el anillo.
3. Pinta solo con `Palette::current()` y tokens de `vow_design`.
4. Añade su caso a `tests/kit/controls.rs` (claro y oscuro) y su página en
   Tengen App Show ([ADR 0047](../decisiones/0047-barrierkit-en-capas.md) §2).

Siguiente: [6. Ventanas](06-ventanas.md).
