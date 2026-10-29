# Auditoría Ponytail ultra P4c — Controles, titlebar y tráfico de BarrierKit

- **Commit:** `cae5759`
- **Alcance:**
  - `crates/barrier-kit/src/controls.rs` (423 líneas)
  - `crates/barrier-kit/src/titlebar.rs` (660 líneas)
  - `crates/barrier-kit/src/traffic.rs` (353 líneas)
  - `crates/barrier-kit/src/sidebar.rs` (662 líneas, solo widgets reutilizados)
  - `crates/barrier-kit/src/theme.rs` (751 líneas)
  - `crates/barrier-kit/src/animation.rs` (85 líneas)
  - `crates/barrier-kit/src/password.rs` (94 líneas)
  - `crates/barrier-kit/tests/controls.rs` (207 líneas)
  - Comparación: `crates/malaysia-settings/src/{main,controls,cards,groups,sidebar,toolbar}.rs`
  - Uso real: `crates/malaysia-shell/src/{dock,lock,launcher}.rs`
- **No leído entero:** `crates/malaysia-settings/src/main.rs` más allá de la inicialización de tema y la ventana de previsualización.
- **Fecha:** 2026-10-28

## Qué hace este repo

MalaysiaDreams es un compositor Wayland en Rust. El alcance P4c cubre los controles visuales reutilizables que las apps propias (`malaysia-settings`, `malaysia-shell`) consumen a través de `barrier-kit`: botones, campos de texto, conmutadores, sliders, grupos de herramientas, la barra de título client-side y los semáforos. Estos componentes deben hablar el lenguaje de diseño Tengen y usar los tokens de `vow-design`, no los valores por defecto de Freya.

**Carga asumida:** pocas apps propias, cientos de controles por ventana, interacción directa con ratón/teclado. No es de alta concurrencia, pero sí debe mantenerse reactivo a cambios de tema y preferencias del usuario.

---

## Hallazgos

### Must fix

#### P4c-01 · Los controles exportados son widgets de Freya sin envoltorio de tema

- **Tipo:** bug / diseño
- **Dónde:** `crates/barrier-kit/src/controls.rs:5-12`
- **Evidencia:**
  ```rust
  pub use freya_components::button::Button;
  pub use freya_components::input::Input as TextField;
  pub use freya_components::menu::Menu;
  pub use freya_components::popup::Popup as Popover;
  pub use freya_components::scrollviews::ScrollView;
  pub use freya_components::segmented_button::SegmentedButton as SegmentedControl;
  pub use freya_components::slider::Slider;
  pub use freya_components::switch::Switch;
  ```
- **Por qué importa:** Las apps propias usan `barrier_kit::controls::Button` esperando que se vea como macOS/Tengen, pero obtienen el estilo por defecto de Freya. El `theme::shell()` no se aplica automáticamente a estos re-exports, así que el color de acento, los radios y las tipografías pueden diferir.
- **Arreglo mínimo:** Crear wrappers propios (o un builder `Themed`) que apliquen `use_init_theme(theme::shell)` y los tokens de `vow-design` antes de delegar en Freya.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-40

#### P4c-02 · `ToolButton` usa la altura del separador como tamaño de icono

- **Tipo:** bug
- **Dónde:** `crates/barrier-kit/src/controls.rs:372`
- **Evidencia:**
  ```rust
  .child(svg_viewer(self.icon, fg, TOOLGROUP.separator_h));
  ```
- **Por qué importa:** El tamaño del icono de un botón de herramienta está atado al token de altura de separador. Si se ajusta el grosor del separador para cambiar la percepción del grupo, los iconos cambian de tamaño sin relación.
- **Arreglo mínimo:** Usar un token propio, por ejemplo `TOOLGROUP.icon_size`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-41

#### P4c-03 · `ToolGroup::new()` resuelve el tema en el constructor

- **Tipo:** riesgo / escala
- **Dónde:** `crates/barrier-kit/src/controls.rs:175-178`
- **Evidencia:**
  ```rust
  pub fn new() -> Self {
      let style = match crate::theme::resolved().sidebar.style { ... };
  ```
- **Por qué importa:** `theme::resolved()` cae a lectura de disco cuando no hay contexto de runner (ver P4c-15). Construir muchos `ToolGroup` fuera de un árbol Freya (por ejemplo, en tests o en pre-construcción de UI) hace I/O sincrónica repetida.
- **Arreglo mínimo:** No leer el tema en el constructor; aplazar a `render()` o aceptar un `SidebarStyle` explícito.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media
- **Semilla:** S-42

#### P4c-04 · `SecureField` expone la longitud de la contraseña al árbol de accesibilidad

- **Tipo:** bug / accesibilidad
- **Dónde:** `crates/barrier-kit/src/controls.rs:123-145`
- **Evidencia:**
  ```rust
  .child(label().text(masked))
  ```
  Sin `.a11y_role(AccessibilityRole::TextInput)` ni ninguna marca de campo protegido.
- **Por qué importa:** Aunque el texto mostrado son balas, un lector de pantalla ve una etiqueta de texto plano con tantos caracteres como la contraseña. No identifica el campo como entrada de contraseña y revela la longitud del secreto.
- **Arreglo mínimo:** Si Freya lo soporta, usar `Input` con una propiedad `secure`/`password`; de lo contrario, añadir al nodo el rol `TextInput`, el estado `protected` y ocultar el texto del lector.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-49

#### P4c-05 · `theme::for_scheme()` ignora accent, tint, reduce_transparency y wallpaper

- **Tipo:** bug / fidelidad
- **Dónde:** `crates/barrier-kit/src/theme.rs:117-139`
- **Evidencia:**
  ```rust
  let appearance = vow_appearance::AppearanceConfig {
      color_scheme: if dark { ColorScheme::Dark } else { ColorScheme::Light },
      ..Default::default()
  };
  ```
- **Por qué importa:** Se usa en tests y en la previsualización de `malaysia-settings`. Una preview que fuerza `Dark` pierde el acento del usuario, el tinte del wallpaper y las preferencias de accesibilidad, por lo que el resultado no representa el escritorio real.
- **Arreglo mínimo:** Aceptar un `AppearanceConfig` completo o partir de la configuración real cargada desde disco y solo cambiar el `color_scheme`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-47

### Should fix

#### P4c-06 · `theme::sheet()` colapsa todos los colores semánticos

- **Tipo:** diseño / deuda
- **Dónde:** `crates/barrier-kit/src/theme.rs:57-86`
- **Evidencia:**
  ```rust
  success: color(accent),
  warning: color(accent),
  error: color(accent),
  info: color(accent),
  text_secondary: color(text),
  text_placeholder: color(text),
  ```
- **Por qué importa:** Freya usa `success`/`warning`/`error`/`info` para estados de controles. Si todos valen el acento, un interruptor de error o un indicador de advertencia serán indistinguibles del acento normal. Los placeholders y textos secundarios tampoco se diferencian.
- **Arreglo mínimo:** Mapear estos colores desde tokens de `vow_appearance` o `vow-design` en lugar de reutilizar `accent`/`text`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta

#### P4c-07 · El helper `svg_viewer` está duplicado en kit, settings y shell

- **Tipo:** sobra
- **Dónde:** `crates/barrier-kit/src/controls.rs:387-396`, `crates/barrier-kit/src/sidebar.rs:39-46`, `crates/malaysia-settings/src/controls.rs:395-404`, `crates/malaysia-settings/src/sidebar.rs:308-317`
- **Evidencia:** Cuatro funciones `svg_viewer(template, color, size)` prácticamente idénticas (mismo reemplazo de `currentColor`, mismo `SvgViewer::new`).
- **Por qué importa:** Cualquier cambio en el manejo de SVG (caché de clave, color con alpha, loader) debe replicarse en cuatro sitios.
- **Arreglo mínimo:** Exportar `barrier_kit::svg_viewer` y reemplazar las copias locales.
- **Tamaño:** S
- **Líneas que se borrarían:** ~40
- **Confianza:** alta

#### P4c-08 · `malaysia-settings` implementa `radio` y `accent_swatch` desde cero con literales

- **Tipo:** reinvención
- **Dónde:** `crates/malaysia-settings/src/controls.rs:20-146`
- **Evidencia:**
  ```rust
  .fill(rgba(Rgba::rgba8(0, 0, 0, 0.15)))
  .background(rgba(Rgba::rgb8(255, 255, 255)))
  ```
- **Por qué importa:** Son controles genéricos que deberían vivir en `barrier-kit` para que cualquier app propia los reutilice. Los colores fijos rompen con temas oscuros o de alto contraste.
- **Arreglo mínimo:** Mover `radio`, `accent_swatch` y `traffic_dropdown` a `barrier-kit::controls` y parametrizar todos los colores.
- **Tamaño:** M
- **Líneas que se borrarían:** 0 (en kit; ~200 en settings)
- **Confianza:** alta

#### P4c-09 · `accent_swatch` usa siempre el color de texto claro para el anillo de selección

- **Tipo:** bug visual
- **Dónde:** `crates/malaysia-settings/src/controls.rs:31`
- **Evidencia:**
  ```rust
  let ring_color = vow_design::LIGHT.text.focused;
  ```
- **Por qué importa:** En modo oscuro el anillo de selección es un color claro que puede fundirse con el fondo claro de la tarjeta, haciendo invisible el estado seleccionado.
- **Arreglo mínimo:** Usar el color de texto fuerte de la paleta actual (`scheme_palette.ink_strong` o similar) en lugar de `LIGHT`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta

#### P4c-10 · `malaysia-settings` inicializa el tema de dos formas inconsistentes

- **Tipo:** bug / inconsistencia
- **Dónde:** `crates/malaysia-settings/src/main.rs:46` y `crates/malaysia-settings/src/main.rs:175`
- **Evidencia:**
  ```rust
  use_init_theme(theme::shell);
  // ...
  use_init_theme(|| barrier_kit::theme::for_scheme(dark));
  ```
- **Por qué importa:** La ventana principal resuelve la apariencia real del usuario; la previsualización fuerza un esquema plano sin acento ni tinte. El resultado de la preview no coincide con el escritorio que se está configurando.
- **Arreglo mínimo:** Usar `theme::shell()` como base y, si se quiere forzar claro/oscuro, mutar solo `color_scheme` de la configuración real.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-50

#### P4c-11 · `_traffic_input` es código muerto

- **Tipo:** sobra
- **Dónde:** `crates/barrier-kit/src/titlebar.rs:506-514`
- **Evidencia:**
  ```rust
  #[allow(dead_code)]
  fn _traffic_input(status: WindowStatus, fixed: bool) -> TrafficInput { ... }
  ```
- **Por qué importa:** No se llama desde ningún sitio (el prefijo `_` y el `#[allow(dead_code)]` lo confirman). Confunde a quien lee el módulo y sugiere una API no terminada.
- **Arreglo mínimo:** Borrarlo o exponerlo como `pub fn traffic_input(...)` si se necesita para tests/previews.
- **Tamaño:** XS
- **Líneas que se borrarían:** 9
- **Confianza:** alta
- **Semilla:** S-51

#### P4c-12 · `window_behavior()` no reacciona a cambios de `window.domain`

- **Tipo:** comportamiento
- **Dónde:** `crates/barrier-kit/src/titlebar.rs:429-438`
- **Evidencia:**
  ```rust
  static BEHAVIOR: OnceLock<WindowBehavior> = OnceLock::new();
  BEHAVIOR.get_or_init(|| { ... lee window.domain una vez ... });
  ```
- **Por qué importa:** Si el usuario cambia `double_click_titlebar` o `green_button` en Ajustes, las ventanas abiertas siguen usando el comportamiento antiguo hasta reiniciarlas. El compositor también lee `window.domain`; conviene que ambos tengan el mismo ciclo de vida.
- **Arreglo mínimo:** Documentar que el cambio requiere reinicio, o leer el archivo en cada render si el rendimiento lo permite.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4c-13 · `traffic.rs` reconstruye el SVG de cada glifo en cada render

- **Tipo:** escala
- **Dónde:** `crates/barrier-kit/src/traffic.rs:264-325`
- **Evidencia:** `glyph_svg` genera `format!(...)` con todos los segmentos cada vez que se pinta un semáforo.
- **Por qué importa:** Aunque los semáforos son pequeños, se renderizan en cada frame de hover/animación. Formatear strings SVG repetidamente es trabajo innecesario.
- **Arreglo mínimo:** Cachear por `(index, color_key, fixed_width)` en un `LazyCell` o `thread_local`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4c-14 · `TrafficLights` fuerza `zoom_supported: true` cuando `fixed_width` es true

- **Tipo:** inconsistencia
- **Dónde:** `crates/barrier-kit/src/traffic.rs:99-106`
- **Evidencia:**
  ```rust
  let visual_input = if self.fixed_width {
      TrafficInput { zoom_supported: true, ..input }
  } else { input };
  ```
- **Por qué importa:** Más abajo `clickable = zoom_supported || fixed_width || kind != Maximize`, por lo que en ventanas de ancho fijo la luz verde siempre es clickeable. Pero `green_admitted` en `titlebar.rs:460` dice que `Zoom` no se admite cuando `fixed` es true. El resultado es que la luz verde ejecuta `toggle_vertical_maximize` (línea 341) solo porque `fixed_width` lo permite, no porque `zoom_supported` sea coherente.
- **Arreglo mínimo:** No mutar `zoom_supported`; dejar que `clickable` ya incluya `fixed_width` para `Maximize` si esa es la intención, y eliminar la línea `zoom_supported: true`.
- **Tamaño:** XS
- **Líneas que se borrarían:** 1
- **Confianza:** media

### Nice to have

#### P4c-15 · `theme::resolved()` cae a lectura de disco fuera de runner

- **Tipo:** riesgo / escala
- **Dónde:** `crates/barrier-kit/src/theme.rs:94-103`
- **Evidencia:**
  ```rust
  if let Some(state) = try_consume_root_context::<State<Resolved>>() { ... }
  else { resolve(&load_appearance(), load_dominant()) }
  ```
- **Por qué importa:** Cualquier componente que llame a `resolved()` en su constructor o en un helper fuera del árbol Freya (como `ToolGroup::new`, `Sidebar::new`) hace I/O sincrónica. En listas largas esto se acumula.
- **Arreglo mínimo:** Documentar claramente qué funciones son seguras fuera de runner, o cachear la configuración en memoria la primera vez.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4c-16 · Los tests de controles solo cubren golden images y `SecureField`

- **Tipo:** cobertura
- **Dónde:** `crates/barrier-kit/tests/controls.rs` y `crates/barrier-kit/src/controls.rs:403-423`
- **Evidencia:** Ningún test verifica que `Button`, `Switch`, `Slider`, `SegmentedControl`, `ToolGroup` o `ToolButton` cambien de estado ante interacciones.
- **Por qué importa:** Los golden images detectan cambios visuales, pero no regressiones de comportamiento (por ejemplo, un `Switch` que deja de notificar, un `ToolButton` que no respeta `disabled`).
- **Arreglo mínimo:** Añadir tests unitarios para los callbacks y estados de los controles custom.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta

#### P4c-17 · `Titlebar` reserva 80 px fijos para los slots laterales

- **Tipo:** UX / diseño
- **Dónde:** `crates/barrier-kit/src/titlebar.rs:261-283`
- **Evidencia:**
  ```rust
  .width(Size::px(80.0))
  ```
- **Por qué importa:** En ventanas estrechas (por ejemplo, 300 px) los 80 px de cada lado más el ancho del título y los semáforos pueden superponerse o truncar el título de forma brusca.
- **Arreglo mínimo:** Usar anchos mínimos (`Size::min_width`) o flex para que el centro ocupe primero el espacio sobrante.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4c-18 · No hay `Checkbox` ni `Radio` exportados desde `barrier-kit::controls`

- **Tipo:** AppKit / brecha
- **Dónde:** `crates/barrier-kit/src/controls.rs:1-23`
- **Evidencia:** Solo se re-exportan `Button`, `TextField`, `Menu`, `Popover`, `ScrollView`, `SegmentedControl`, `Slider`, `Switch`.
- **Por qué importa:** `malaysia-settings` ya implementa `radio` a mano. Cada app propia acabará reinventando `Checkbox` y `Radio`. ADR 0034 apostaba por un kit unificado.
- **Arreglo mínimo:** Añadir wrappers para `FormCheckbox`/`FormRadio` de Freya (o implementaciones propias) que usen `appearance.accent`.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta

---

## Brecha AppKit

Qué controles o comportamientos estándar de macOS faltan en `barrier-kit` y se reinventan en cada app:

| Qué falta | Uso típico | Quién lo reimplementa hoy |
|-----------|------------|---------------------------|
| `Checkbox` | Ajustes, forms | `malaysia-settings` no lo usa aún; lo hará a mano |
| `Radio` | Selección mutuamente excluyente | `malaysia-settings/src/controls.rs:83-146` |
| `SpinButton` / `Stepper` | Números con flechas | No existe |
| `ComboBox` / `PopupButton` | Lista desplegable de valores | `traffic_dropdown` a mano en settings |
| `SecureField` accesible | Diálogos de contraseña | `barrier-kit::controls::SecureField` sin rol (P4c-04) |
| `ProgressIndicator` | Instalación, carga | No existe |
| `ColorWell` | Selector de color | `accent_swatch` a mano en settings |
| `SearchField` | Búsqueda con icono de lupa | No existe |
| `Focus` y `Fullscreen` del semáforo verde | Acciones de ventana | `titlebar.rs:477-482` solo imprimen `eprintln!` |

---

## Preguntas

1. **¿Es intencional que `barrier-kit::controls` sea principalmente un `pub use` de Freya?** Si sí, convendría documentarlo; si no, hay que envolver cada control.
2. **¿Por qué `theme::for_scheme()` no parte de la configuración real del usuario?** Los tests y previews deberían reflejar el acento y el wallpaper.
3. **¿`SecureField` es un campo de contraseña temporal o el definitivo?** Si es definitivo, necesita rol de accesibilidad y ocultar la longitud.
4. **¿Se quiere que `window.domain` se recargue en caliente en la titlebar?** Hoy requiere reiniciar la app.
5. **¿`Focus` y `Fullscreen` del botón verde están planificados?** Ahora solo hay `eprintln!`.

---

## Revisado y correcto

| Semilla | Estado | Motivo |
|---------|--------|--------|
| S-43 | **Confirmado (brecha)** | No hay `SpinButton` ni `Stepper` en `barrier-kit` ni en `malaysia-settings`. Es una ausencia, no un bug. |
| S-48 | **Correcto con matices** | `Switch` es un re-export de Freya; el tema de Freya *sí* se inicializa con `use_init_theme`, pero el color de acento no se mapea explícitamente a `appearance.accent`. Esto se cubre en P4c-01. |

---

## Veredicto

Los controles custom (`ToolGroup`, `ToolButton`, `SecureField`, titlebar, tráfico) están bien encaminados y usan tokens, pero el kit sigue dependiendo demasiado de re-exports de Freya. Eso rompe la promesa de un lenguaje de diseño unificado (ADR 0034). Antes de seguir añadiendo apps propias, conviene envolver los controles de Freya con el tema Tengen (P4c-01), mover los controles genéricos de `malaysia-settings` a `barrier-kit` (P4c-08) y arreglar `SecureField` (P4c-04).

Prioridad de arreglos: P4c-01, P4c-04, P4c-05, P4c-08, P4c-09, P4c-10.

## Lean

- **Líneas que se borrarían:** ~50 (helper `svg_viewer` duplicado, `_traffic_input`, colores literales movidos a tokens).
- **Dependencias que se borrarían:** 0.

## Not checked

- La versión exacta de Freya 0.4.3 y qué props de `Input` soporta para campos seguros.
- El renderizado real de golden images bajo distintas versiones de Skia.
- `crates/malaysia-settings/src/main.rs` completo más allá de la inicialización de tema.
- Controles de apps de terceros; este alcance solo cubre apps propias de MalaysiaDreams.
