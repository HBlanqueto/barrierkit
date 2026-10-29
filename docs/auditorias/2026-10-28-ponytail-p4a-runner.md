# Ponytail ultra audit P4a — runner de BarrierKit

- **Commit:** `cae5759`.
- **Scope:** `crates/barrier-kit/src/platform/mod.rs`, `src/platform/input.rs`, `src/platform/infinity.rs`, `src/fonts.rs`.
- **Files read:**
  - `platform/mod.rs`: 4224 líneas (completo).
  - `platform/input.rs`: 208 líneas (completo).
  - `platform/infinity.rs`: 25 líneas (completo).
  - `fonts.rs`: 37 líneas (completo).
- **No se pudo leer entero:** nada.
- **Semillas del alcance:** B4-21 y B4-22 del informe P4 anterior (ambas en `platform/mod.rs`); S-21 de P4b (atajos de menú en el hilo de UI, `platform/mod.rs:3495-3514`).

## TL;DR

El runner funciona, pero tiene errores reales de comportamiento: las ventanas hijas abiertas desde código pierden la relación `parent`, la activación por clic no obtiene token, el touch en superficies secundarias va a la primera superficie, y la escala puede quedarse obsoleta en layer surfaces antes del primer configure. También quedan dos semillas anteriores sin cerrar (B4-21 y B4-22). Nada es catastrófico, pero varios afectan la experiencia de usuario y la corrección del protocolo Wayland.

---

## Findings

### Must fix

#### P4a-01 · `WindowControls::open_window` ignora `options.parent`

- **Tipo:** bug.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:2525-2527` y `:606-608`.
- **Evidencia:**
  ```rust
  /// Opens another toplevel in the same process, attached to this window as
  /// its parent when `options.parent` is `Some(surface_id())`.
  pub fn open_window(&self, app: AppComponent, options: WindowOptions) { ... }

  // En process_window_commands:
  WindowCommand::Open(app, options) => {
      self.create_window(app, options, None);  // <-- siempre None
      continue;
  }
  ```
- **Por qué importa:** El documento promete que pasando `options.parent = Some(surface_id())` la nueva ventana queda anclada a la actual. Como el código pasa `None`, todos los diálogos/modales se crean sin padre. El compositor no las trata como modales y el usuario puede perderlas detrás de la ventana principal.
- **Arreglo mínimo:** Cambiar `None` por `options.parent` (o, si se prefiere un default, `options.parent.or(Some(context.controls.surface_id()))`).
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-02 · `GraphicsShared::new` tiene un bloque `unsafe` sin comentario `// SAFETY:`

- **Tipo:** riesgo / calidad de `unsafe`.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:683-684`.
- **Evidencia:**
  ```rust
  let egl = unsafe { DynamicInstance::<egl::EGL1_5>::load_required() }
      .map_err(|_| RunError::Egl("libEGL could not be loaded"))?;
  ```
  No hay `// SAFETY:` antes ni después del bloque. El bloque de `get_platform_display` sí lo tiene en `:685-692`.
- **Por qué importa:** La política del repositorio explica que todo `unsafe` debe llevar `// SAFETY:`. Además, `load_required()` carga símbolos de una biblioteca dinámica; si la versión mínima de EGL no se cumple, el comportamiento depende del driver. El comentario obliga a documentar la precondición (`EGL 1.5` y hilo único).
- **Arreglo mínimo:** Añadir `// SAFETY: called once on the calloop thread; libEGL is present because the compositor already initialized EGL.` encima del bloque.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-03 · `poll_activation` solo acepta serial de teclado, falla en lanzamientos por clic

- **Tipo:** bug.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:2828-2849`.
- **Evidencia:**
  ```rust
  if self.last_key_serial == 0 {
      callback(None);
      continue;
  }
  ...
  token.set_serial(self.last_key_serial, seat);
  ```
  `last_input_serial` se actualiza tanto en puntero (`:3354`) como en teclado (`:3434`).
- **Por qué importa:** Si el usuario abre una app desde el dock con el ratón antes de haber pulsado una tecla, `last_key_serial` es `0` y el token es `None`. El compositor no activará la nueva ventana. `xdg-activation-v1` acepta cualquier serial de evento reciente, no solo teclado.
- **Arreglo mínimo:** Usar `self.last_input_serial.get()` en vez de `self.last_key_serial`, o al menos el máximo de ambos.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-04 · `scale_factor_changed` no actualiza la escala cuando `logical_size` es cero

- **Tipo:** bug.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:2919-2935`.
- **Evidencia:**
  ```rust
  let scale = scale_factor.max(1) as f64;
  let window = &mut self.surfaces[index];
  if window.scale != scale && window.logical_size.0 > 0 {
      let size = window.logical_size;
      window.configure(size, scale);
  }
  ```
- **Por qué importa:** Si el compositor envía `wl_surface.preferred_buffer_scale` antes del primer `configure` (por ejemplo, un layer surface creado sin tamaño fijo), `window.scale` no se actualiza. `LayerShellHandler::configure` usa `self.surfaces[index].scale` (`:3157`), así que el primer buffer se renderiza con escala 1.0 aunque el monitor sea HiDPI. Queda borroso hasta el próximo evento de escala.
- **Arreglo mínimo:** Asignar siempre `window.scale = scale;` y reconfigurar solo si `logical_size.0 > 0`.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-05 · Touch `up`/`motion` siempre envía el evento a `surfaces.first()`

- **Tipo:** bug.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:3598-3638`.
- **Evidencia:**
  ```rust
  fn up(..., id: i32, ...) {
      let Some(surface) = self.surfaces.first().map(|s| s.surface.clone()) else {
          return;
      };
      self.touch_event(qh, surface, id, (0.0, 0.0), TouchPhase::Ended);
  }

  fn motion(..., id: i32, position: (f64, f64)) {
      let Some(surface) = self.surfaces.first().map(|s| s.surface.clone()) else {
          return;
      };
      self.touch_event(qh, surface, id, position, TouchPhase::Moved);
  }
  ```
  SCTK 0.19.2 no proporciona la superficie en `up` ni `motion` (`src/seat/touch.rs:101-122`).
- **Por qué importa:** Si hay más de una superficie (ventana + popup o lock + overlay), el `up` y `motion` de un contacto que empezó en un popup se envían al toplevel. Rompe menús táctiles, popups y cualquier interacción multitáctil real.
- **Arreglo mínimo:** Mantener un `HashMap<i32, WlSurface>` (o `ObjectId`) en `Shell`, insertar en `down` y consultar/eliminar en `up`/`motion`.
- **Tamaño:** M.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

### Should fix

#### P4a-06 · `input::code` deja la mayoría de las teclas como `Code::Unidentified`

- **Tipo:** brecha / calidad de input.
- **Dónde:** `crates/barrier-kit/src/platform/input.rs:69-97`.
- **Evidencia:**
  ```rust
  fn code(keysym: Keysym) -> Code {
      match keysym {
          Keysym::Return | Keysym::KP_Enter => Code::Enter,
          ...
          Keysym::space => Code::Space,
          _ => Code::Unidentified,
      }
  }
  ```
- **Por qué importa:** Letras, dígitos y teclas del pad numérico se reportan a Freya como `Code::Unidentified`. Cualquier componente que use `Code` para atajos físicos (juegos, edición de texto, atajos de teclado robustos) no funcionará. `freya-winit` mapea muchas más teclas.
- **Arreglo mínimo:** Añadir matches para `Keysym::a..=z`, `Keysym::_0..=_9`, `KP_*` y las teclas de puntuación más comunes, mapeando a `Code::KeyA`..`KeyZ`, `Digit0`..`Digit9`, etc.
- **Tamaño:** M.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** —

#### P4a-07 · `GraphicsShared::Drop` no espera a que la GPU termine antes de destruir el contexto (B4-22)

- **Tipo:** riesgo.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:754-760`.
- **Evidencia:**
  ```rust
  impl Drop for GraphicsShared {
      fn drop(&mut self) {
          let _ = self.egl.make_current(self.display, None, None, None);
          self.gr.borrow_mut().abandon();
          let _ = self.egl.destroy_context(self.display, self.context);
      }
  }
  ```
- **Por qué importa:** `abandon()` y `destroy_context()` pueden ejecutarse mientras comandos GL del último frame todavía están en vuelo. En algunos drivers esto puede colgar el proceso siguiente que reutilice la tarjeta o dejar un framebuffer corrupto. B4-22 ya lo señaló y sigue vigente.
- **Arreglo mínimo:** Insertar `glFinish()` (o un fence) entre `make_current` y `abandon`. Si no hay acceso directo a `glFinish`, cargar la función con `egl.get_proc_address`.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** B4-22

#### P4a-08 · `platform/mod.rs` es un monolito de 4224 líneas que mezcla cinco responsabilidades (B4-21)

- **Tipo:** abstracción / mantenibilidad.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs` entero.
- **Evidencia:** En un solo archivo viven: inicialización EGL/Wayland (`GraphicsShared`, `Graphics`), dispatch de protocolo (`Shell` y todos sus `impl Dispatch`), gestión de popups (`open_popup`, `close_popup`, `reposition_popup`), timers/repeat/activación, y toplevels/Infinity.
- **Por qué importa:** Cualquier cambio en popups obliga a leer código de EGL o D-Bus en el mismo archivo. El coste de navegación crece con cada protocolo nuevo (Bloque 11/12).
- **Arreglo mínimo:** Partir en `platform/egl.rs`, `platform/wayland.rs`, `platform/popup.rs`, `platform/timer.rs` y dejar `mod.rs` como fachada delgada. No cambiar lógica, solo mover.
- **Tamaño:** L.
- **Líneas que se borrarían:** 0 (reorganización).
- **Confianza:** alta.
- **Semilla:** B4-21

#### P4a-09 · `activation_callbacks` crece indefinidamente y nunca se compacta

- **Tipo:** riesgo / crecimiento de memoria.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:2843-2844` y `:2858-2863`.
- **Evidencia:**
  ```rust
  let id = self.activation_callbacks.len();
  self.activation_callbacks.push(Some(callback));
  ...
  if let Some(callback) = state.activation_callbacks.get_mut(*id).and_then(Option::take) {
      callback(Some(token));
  }
  ```
- **Por qué importa:** Cada token consumido deja un `None` en el vector. Si la app pide muchos tokens (por ejemplo, un lanzador que abre apps frecuentemente), la memoria del vector solo crece. No hay fuga real grave porque los callbacks se liberan, pero es un acumulador sin límite.
- **Arreglo mínimo:** Reutilizar huecos `None` al hacer `push`, o compactar periódicamente.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-10 · `apply_pending_window_configures` intenta dibujar aunque `configure()` haya fallado

- **Tipo:** bug leve.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:3082, :3132`.
- **Evidencia:**
  ```rust
  self.surfaces[index].configure(size, scale);
  ...
  if self.surfaces[index].draw() && self.surfaces[index].needs_redraw {
      self.surfaces[index].request_redraw(qh);
  }
  ```
  `SurfaceWindow::configure` loguea el error y devuelve, pero `apply_pending_window_configures` no lo consulta.
- **Por qué importa:** Si EGL falla al recrear la superficie, el siguiente `draw()` trabaja sobre un `graphics.skia` que puede ser `None` o un EGL surface inválido. En la práctica `draw()` devuelve `false`, pero se hace trabajo innecesario y se generan errores confusos.
- **Arreglo mínimo:** Guardar el resultado de `configure` y saltar el `draw()` si falló.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** —

#### P4a-11 · Bloque `session_unlock` muerto en `run_layer`

- **Tipo:** sobra.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:1891-1899`.
- **Evidencia:**
  ```rust
  if let Some(lock) = shell.active_lock.clone() {
      shell.session_unlock = Some(SessionUnlock { ... });
  }
  ```
  Dentro de `run_layer`, `active_lock` siempre es `None` porque los locks se crean en `run_lock` y se activan en `SessionLockHandler::locked` (`:3164`).
- **Por qué importa:** Código que nunca se ejecuta. Confunde al lector sobre si un layer surface puede desbloquear la sesión.
- **Arreglo mínimo:** Borrar el bloque.
- **Tamaño:** S.
- **Líneas que se borrarían:** ~9.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-12 · `request_close` de la última ventana no libera el appmenu antes de salir

- **Tipo:** riesgo / fuga.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:2988-3022`.
- **Evidencia:**
  ```rust
  if window_count <= 1 {
      self.needs_exit = true;
  }
  ...
  if !self.needs_exit {
      if let Some(index) = self.surfaces.iter().position(...) {
          if let Some(address) = self.surfaces[index].appmenu_address.take() {
              crate::app_menu::server::unregister(&address.path);
          }
          if let Some(appmenu) = self.surfaces[index].appmenu.take() {
              appmenu.release();
          }
          ...
      }
  }
  ```
- **Por qué importa:** Si la ventana que cierra es la única, el proceso sale sin desregistrar la ruta D-Bus del menú. Aunque el proceso muere, el bus de sesión puede mantener el nombre fantasma unos segundos, y una siguiente ventana con el mismo índice de path fallaría (además del problema B4-03 ya conocido).
- **Arreglo mínimo:** Mover `unregister`/`release` fuera del `if !self.needs_exit`, o al menos ejecutarlos antes de `needs_exit = true`.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** —

#### P4a-13 · `SurfaceWindow::configure` en error de EGL reconfigura la app a `(0, 0)`

- **Tipo:** bug leve.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:1534-1539`.
- **Evidencia:**
  ```rust
  if let Err(error) = graphics.configure(&self.surface, buffer_size, scale) {
      tracing::error!("BarrierKit: {error}");
      self.app.resize(self.buffer_size, self.scale);
      self.needs_redraw = true;
      return;
  }
  ```
- **Por qué importa:** En el primer configure, `self.buffer_size` es `(0, 0)`. Si EGL falla, se redimensiona la app a `(0, 0)`, lo que puede provocar un panic en Torin al medir un árbol vacío, enmascarando el verdadero error de EGL.
- **Arreglo mínimo:** No llamar `app.resize` con `(0, 0)`; usar el `size` lógico recibido o simplemente loguear y no tocar el tamaño de la app.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** —

### Nice to have

#### P4a-14 · `wayland-backend` se declara pero no se usa directamente

- **Tipo:** sobra / dependencia.
- **Dónde:** `crates/barrier-kit/Cargo.toml:32`.
- **Evidencia:** `rg 'wayland_backend|wayland-backend' crates/barrier-kit/src crates/barrier-kit/examples crates/barrier-kit/tests` solo encuentra `Cargo.toml`. El único uso de `backend` es `sctk::reexports::client::backend::ObjectId` (`platform/mod.rs:133`), que viene de `wayland-client`, no de `wayland-backend`.
- **Por qué importa:** Aumenta el cierre de dependencias y el tiempo de build de Nix sin aportar nada. B4-18 ya lo señaló y sigue vigente.
- **Arreglo mínimo:** Borrar la línea de `Cargo.toml` y verificar `cargo check -p barrier-kit`.
- **Tamaño:** S.
- **Líneas que se borrarían:** 1.
- **Confianza:** alta.
- **Semilla:** B4-18

#### P4a-15 · `run_layer`, `run_window` y `run_lock` duplican el boilerplate del event loop

- **Tipo:** duplicación.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:1882-1909`, `:1928-1946`, `:1961-1989`.
- **Evidencia:** Los tres crean `EventLoop`, llaman a `insert_ping`, insertan `WaylandSource`, guardan `loop_signal` y ejecutan `event_loop.run` con el mismo closure de idle.
- **Por qué importa:** Cualquier cambio en el setup del loop (por ejemplo, añadir una fuente más) hay que hacerlo tres veces. Es fuente de olvidos.
- **Arreglo mínimo:** Extraer una función `fn run_event_loop(shell, conn, event_queue) -> Result<(), RunError>`.
- **Tamaño:** M.
- **Líneas que se borrarían:** ~30.
- **Confianza:** alta.
- **Semilla:** —

#### P4a-16 · Tamaños por defecto de `PopupSpec` y `WindowOptions` usan literales

- **Tipo:** atajo / diseño.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:264-276` y `:473-486`.
- **Evidencia:**
  ```rust
  size: (100, 100),   // PopupSpec::default
  size: (640, 420),   // WindowOptions::new
  ```
- **Por qué importa:** ADR 0011/0020 dicen que todos los valores de diseño deben venir de `vow-design`. Hoy un cambio de tema no puede alterar el tamaño inicial de ventanas o popups.
- **Arreglo mínimo:** Añadir tokens `window.default_width/height` y `popup.default_size` a `vow-design/tengen/components.json` y usarlos aquí.
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** media.
- **Semilla:** —

#### P4a-17 · `request_frame_callback` descarta el `WlCallback` recién creado

- **Tipo:** atajo documentable.
- **Dónde:** `crates/barrier-kit/src/platform/mod.rs:1506-1512`.
- **Evidencia:**
  ```rust
  let _callback: WlCallback = self.surface.frame(qh, self.surface.clone());
  self.surface.commit();
  ```
- **Por qué importa:** El callback es manejado por SCTK a través de `CompositorHandler::frame`. Descartarlo está bien porque el compositor lo destruye al llamarlo, pero no queda claro si es intencional. No hay bug funcional.
- **Arreglo mínimo:** Añadir un comentario `// The compositor calls and destroys the callback; we only need CompositorHandler::frame.`
- **Tamaño:** S.
- **Líneas que se borrarían:** 0.
- **Confianza:** alta.
- **Semilla:** —

---

## Brecha AppKit (no son hallazgos)

Lo que falta en el runner para que una app de BarrierKit no tenga que reinventar nada:

1. **Portapapeles propio por `wl_data_device` y `zwp_primary_selection`.** No hay token; hoy el kit entrega un portapapeles vacío (`platform/mod.rs:1014` pone `None::<Box<dyn ClipboardProvider>>`). Ninguna app lo reimplementa, pero tampoco funciona copiar/pegar nativo.
2. **Arrastrar y soltar.** No hay token; no está implementado.
3. **`pointer-constraints` + `relative-pointer`.** No hay token; no está implementado. Necesario para juegos o apps que capturen el cursor.
4. **`zwp_idle_inhibitor_v1`.** No hay token; no está implementado. Necesario para reproducción de vídeo.
5. **`wp_presentation` para timing de frames.** No hay token; no está implementado. Útil para video sincronizado.
6. **Soporte real multi-seat.** No hay token; `Shell` solo guarda un puntero, un teclado y un touch (`:1634-1638`).

---

## Preguntas

1. **¿`poll_activation` debe seguir usando solo serial de teclado?**
   - A: Sí, el requisito de `xdg-activation` es un serial de teclado reciente y no queremos activar por clic.
   - B: No, debe usar `last_input_serial` para permitir lanzamientos desde el dock/ratón.

2. **¿Se acepta el fallback de touch a `surfaces.first()` por ahora?**
   - A: Sí, las apps actuales del shell usan una sola superficie y no hay prioridad.
   - B: No, hay que implementar el mapa `touch_id -> surface` antes de que las apps táctiles crezcan.

3. **¿Se divide `platform/mod.rs` ahora o se espera al siguiente protocolo?**
   - A: Ahora, antes de que crezca más.
   - B: Después, cuando se añada el siguiente protocolo y se vea dónde encaja.

4. **¿`GraphicsShared::Drop` necesita `glFinish()`?**
   - A: No, el proceso está saliendo y los drivers lo gestionan.
   - B: Sí, para evitar hangs/corrupción cuando otro proceso reutilice la GPU.

---

## Revisado y correcto

- **S-21 (P4b-menu):** Los atajos de menú local se ejecutan en el hilo de UI (`platform/mod.rs:3495-3514`); las acciones D-Bus se ejecutan en el hilo del servidor D-Bus (`app_menu/server.rs`). Verificado en HEAD. Es una separación de responsabilidades, no un bug. El callback `on_menu_action` podría documentar en qué hilo corre, pero eso es una mejora, no un hallazgo.

---

## Verdict

Arreglar primero: **P4a-01** (parent perdido), **P4a-03** (activación por ratón), **P4a-05** (touch en múltiples superficies), **P4a-04** (escala obsoleta) y **P4a-02** (`// SAFETY:`). Después abordar P4a-07 (GPU idle) y planificar P4a-08 (dividir `mod.rs`).

**Lean:** `-1` dependencia (`wayland-backend`), `~10` líneas borrables (bloque muerto en `run_layer`), y `~30` líneas de boilerplate duplicado si se unifica el loop. Neto estimado: **-1 dep, -~20-40 líneas**.

**Not checked:** `app_menu/server.rs`, `menu.rs`, `controls.rs`, `titlebar.rs`, `theme.rs`, `animation.rs`, `password.rs`, `examples/`, tests fuera de `platform/mod.rs`, y el grafo transitivo de dependencias de `Cargo.lock`.
