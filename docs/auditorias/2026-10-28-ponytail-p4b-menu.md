# Auditoría Ponytail ultra P4b — Menús de BarrierKit

- **Commit:** `cae5759`
- **Alcance:**
  - `crates/barrier-kit/src/menu.rs` (3,413 líneas)
  - `crates/barrier-kit/src/app_menu.rs` (505 líneas)
  - `crates/barrier-kit/src/app_menu/layout.rs` (346 líneas)
  - `crates/barrier-kit/src/app_menu/server.rs` (515 líneas)
  - `crates/malaysia-shell/src/menubar/menu.rs` (328 líneas)
  - `crates/malaysia-shell/src/menubar/dbusmenu_map.rs` (619 líneas)
  - `crates/malaysia-shell/src/menubar/remote.rs` (544 líneas)
  - `crates/malaysia-shell/src/menubar/focused.rs` (996 líneas)
  - Tests: `crates/barrier-kit/tests/menu.rs` (539 líneas)
- **No leído entero:** `crates/malaysia-shell/src/menubar/dbusmenu.rs` (1,469 líneas); solo sus primeras 300 líneas y las firmas públicas.
- **Fecha:** 2026-10-28

## Qué hace este repo

MalaysiaDreams es un compositor Wayland en Rust. El alcance P4b cubre el subsistema de menús de la shell y de las apps propias: la exportación D-Bus `com.canonical.dbusmenu` desde las apps (`barrier-kit/src/app_menu`), el control visual de menús emergentes (`barrier-kit/src/menu.rs`), y el cliente/mapeador del menú global en la barra (`malaysia-shell/src/menubar`).

**Carga asumida:** una sesión de escritorio con un puñado de apps con menú global (por ejemplo, Kate, Firefox, apps propias) y un solo proceso de shell (`malaysia-shell`) que atiende todos los títulos. No es un servicio multiusuario ni de alta concurrencia, pero sí debe sobrevivir a cierres y reaperturas de ventanas sin perder menús ni bloquear el hilo de UI.

---

## Hallazgos

### Must fix

#### P4b-01 · Las rutas D-Bus `/MenuBar/N` se reutilizan al cerrar una ventana

- **Tipo:** bug
- **Dónde:** `crates/barrier-kit/src/app_menu/server.rs:64-68`
- **Evidencia:**
  ```rust
  let index = {
      let menus = server.menus.lock().expect("menu registry poisoned");
      menus.len() as u32
  };
  let path = format!("{PATH_PREFIX}{index}");
  ```
- **Por qué importa:** Si se registran las ventanas 0 y 1, cierras la 0 y abres una nueva, `menus.len()` vale 1, la nueva ventana recibe `/MenuBar/1` que aún está registrado en zbus, `connection.object_server().at(...)` falla y el menú de la nueva ventana desaparece silenciosamente.
- **Arreglo mínimo:** Usar un contador monotónico `AtomicU32` en lugar de `menus.len()`.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-20

#### P4b-02 · Los separadores dentro de un mismo menú comparten id numérico

- **Tipo:** bug
- **Dónde:** `crates/barrier-kit/src/app_menu/layout.rs:138` y `crates/barrier-kit/src/app_menu.rs:349`
- **Evidencia:**
  ```rust
  let id = id_map.get("__separator__");
  ```
- **Por qué importa:** `IdMap::get` devuelve el mismo id para la misma clave. Un menú con `[A, Separador, B, Separador, C]` asigna el mismo id a ambos separadores; el segundo `snapshot.nodes.insert` sobrescribe al primero, faltan nodos en el layout D-Bus y `GetLayout` puede devolver un árbol con menos separadores de los esperados.
- **Arreglo mínimo:** Usar una clave única por separador, por ejemplo `format!("__separator__:{index}")`, o que `IdMap` reserve un rango de ids fijos para separadores por posición.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta

#### P4b-03 · `GetLayout(0)` reconstruye los ids con un `IdMap` nuevo que puede divergir del snapshot

- **Tipo:** bug
- **Dónde:** `crates/barrier-kit/src/app_menu/server.rs:297-302`
- **Evidencia:**
  ```rust
  let (menu, _) = self.registered.handle.snapshot();
  let mut id_map = IdMap::default();
  let (_snapshot, layout) = build_layout(&menu, &mut id_map, revision);
  return (revision, layout);
  ```
- **Por qué importa:** `event(id)` usa `action_id_for_numeric`, que también reconstruye ids desde cero. Si el menú cambia entre `GetLayout` y `Event` (por ejemplo, se añade un ítem antes del objetivo), el id numérico del `Event` apunta a una acción distinta. Esto permite activar el menú equivocado por una carrera de actualización.
- **Arreglo mínimo:** Servir `GetLayout(0)` desde `last_layout` (el snapshot ya construido), no reconstruir. Asegurar que `event()` use el `IdMap` del snapshot correspondiente.
- **Tamaño:** M
- **Líneas que se borrarían:** ~10
- **Confianza:** media
- **Semilla:** S-22

#### P4b-04 · `Server::register` bloquea el hilo de UI esperando la respuesta de D-Bus

- **Tipo:** bug / riesgo
- **Dónde:** `crates/barrier-kit/src/app_menu/server.rs:77-82`
- **Evidencia:**
  ```rust
  let (reply_tx, reply_rx) = async_channel::bounded(1);
  let _ = server.commands.send_blocking(Command::Register { ... });
  async_io::block_on(reply_rx.recv()).ok().flatten()
  ```
- **Por qué importa:** Se llama desde `platform/mod.rs:2333` durante la creación de una ventana. Si el hilo D-Bus está ocupado o el bus de sesión tarda en responder, el hilo de UI se congela y el compositor deja de procesar eventes.
- **Arreglo mínimo:** Devolver inmediatamente un `AppMenuAddress` basado en el nombre único conocido y procesar el registro real en segundo plano; notificar errores por log, no bloqueando.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-23

#### P4b-05 · El callback `on_action` de D-Bus se ejecuta en el hilo de D-Bus, no en el de UI

- **Tipo:** riesgo
- **Dónde:** `crates/barrier-kit/src/app_menu/server.rs:347-354` y `crates/barrier-kit/src/platform/mod.rs:2327-2333`
- **Evidencia:**
  ```rust
  fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
      if event_id != "clicked" { return; }
      if let Some(action_id) = action_id_for_numeric(&self.registered, id) {
          let on_action = self.registered.on_action.clone();
          (on_action)(&action_id);
      }
  }
  ```
- **Por qué importa:** `WindowOptions::on_menu_action` es un cierre arbitrario del usuario. Aunque su tipo exige `Send + Sync`, el usuario espera que corra en el hilo de UI de Freya. Ejecutarlo en el hilo D-Bus puede romper estado de componentes o provocar condiciones de carrera.
- **Arreglo mínimo:** Enviar el `action_id` de vuelta al hilo de UI a través del `WakeHandle` del `MenuHandle` y ejecutar el callback allí.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-21

#### P4b-06 · `truncate_middle` es cuadrático porque re-mide en cada iteración

- **Tipo:** escala
- **Dónde:** `crates/barrier-kit/src/menu.rs:189-215`
- **Evidencia:**
  ```rust
  while left > 0 || right > 0 {
      let body: String = chars.iter().take(left).chain(chars.iter().skip(chars.len() - right)).collect();
      if measure_text(&body, font_size) <= max_body { ... }
      ...
  }
  ```
- **Por qué importa:** `measure_text` construye un `ParagraphBuilder` de Skia en cada llamada. Para etiquetas muy largas (hasta el ancho máximo permitido) el bucle hace O(n) mediciones y cada una es O(n) de trabajo, llegando a O(n²) por fila. Se dispara en cada `preferred_size` y en cada render de fila con etiqueta larga.
- **Arreglo mínimo:** Medir una vez por posición de corte usando un buffer de anchos acumulados, o usar búsqueda binaria sobre los índices de caracteres.
- **Tamaño:** M
- **Líneas que se borrarían:** ~5
- **Confianza:** alta

#### P4b-07 · `decode_icon` usa `unsafe` sin comentario `// SAFETY:`

- **Tipo:** riesgo
- **Dónde:** `crates/barrier-kit/src/menu.rs:2505-2506`
- **Evidencia:**
  ```rust
  fn decode_icon(bytes: &[u8], size: f32) -> Option<ImageHandle> {
      let data = unsafe { SkData::new_bytes(bytes) };
      ...
  }
  ```
- **Por qué importa:** El proyecto audita `unsafe`. Sin explicar por qué `bytes` sigue vivo mientras `SkData` lo referencia, un futuro cambio podría pasar un buffer temporal y causar use-after-free.
- **Arreglo mínimo:** Añadir `// SAFETY: bytes is borrowed for the duration of the call and SkData does not outlive the borrow.` u homologar con `Bytes` antes de llamar a Skia.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta

### Should fix

#### P4b-08 · Al pasar el ratón por una fila de submenú se abre inmediatamente si no había otro abierto

- **Tipo:** bug / UX
- **Dónde:** `crates/barrier-kit/src/menu.rs:1004-1028`
- **Evidencia:**
  ```rust
  Rc::new(move |index: usize, pos: CursorPoint| {
      ...
      if let Some(child_index) = open_child_index.get() && child_index != index ... {
          if point_in_triangle(pos, from, to_top, to_bottom) {
              start_delay(index, from, to_top, to_bottom);
              return;
          }
      }
      execute_target(index);
  })
  ```
- **Por qué importa:** Solo se retrasa cuando ya hay un submenú abierto y el cursor cruza el triángulo de seguridad. La primera apertura de submenú es instantánea, lo que provoca aperturas accidentales al cruzar verticalmente el menú.
- **Arreglo mínimo:** Aplicar `SUBMENU_INTENT_DELAY` también cuando no hay submenú abierto, o al menos cuando el movimiento no es claramente hacia la derecha.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media
- **Semilla:** S-24

#### P4b-09 · `PartialEq` de `MenuView` siempre devuelve `true` sin documentar por qué

- **Tipo:** atajo
- **Dónde:** `crates/barrier-kit/src/menu.rs:738-742`
- **Evidencia:**
  ```rust
  impl PartialEq for MenuView {
      fn eq(&self, _other: &Self) -> bool {
          true
      }
  }
  ```
- **Por qué importa:** Freya usa `PartialEq` para decidir si re-renderizar. Siempre devolver `true` fuerza reconciliación conservadora, pero sin un comentario que lo justifique parece un olvido. Si en el futuro alguien añade un `should_update`, el comportamiento será impredecible.
- **Arreglo mínimo:** Añadir `// shortcut: MenuView is re-rendered every frame by design because items_source/searchable may change externally; revisit if Freya gets explicit update gates.`
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media
- **Semilla:** S-25

#### P4b-10 · Los atajos se parsean en cada pulsación de tecla

- **Tipo:** escala
- **Dónde:** `crates/barrier-kit/src/app_menu.rs:261-264`
- **Evidencia:**
  ```rust
  pub fn shortcut_matches(text: &str, modifiers: Modifiers, key: &Key) -> bool {
      let parsed = parse_shortcut(text);
      parsed.modifiers == modifiers && parsed.key == normalised_key(key) && !parsed.key.is_empty()
  }
  ```
- **Por qué importa:** `shortcut_matches` se llama desde `platform/mod.rs:3503` por cada `KeyDown` para cada atajo del menú. Parsear `"Ctrl+Shift+S"` en una cadena, normalizar y comparar es trabajo repetido que crece con el número de atajos.
- **Arreglo mínimo:** Almacenar `ParsedShortcut` dentro de `MenuItem::Action` (o construirlo una vez por menú) y comparar `Modifiers`/`Key` directamente.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** alta
- **Semilla:** S-26

#### P4b-11 · `measure_text` crea un párrafo nuevo de Skia en cada llamada

- **Tipo:** escala
- **Dónde:** `crates/barrier-kit/src/menu.rs:169-184`
- **Evidencia:**
  ```rust
  pub fn measure_text(text: &str, size: f32) -> f32 {
      ...
      let mut builder = ParagraphBuilder::new(&paragraph_style, crate::fonts::collection());
      builder.add_text(text);
      let mut paragraph = builder.build();
      paragraph.layout(f32::MAX);
      paragraph.max_intrinsic_width()
  }
  ```
- **Por qué importa:** Se llama una vez por fila en `preferred_size` y varias veces en `truncate_middle`. Cada llamada reconstruye el shaper y la colección de fuentes. En un menú grande esto es el cuello de botella de medición.
- **Arreglo mínimo:** Cachear un `ParagraphBuilder`/`Paragraph` en un `thread_local!` y reutilizarlo con `set_text`.
- **Tamaño:** M
- **Líneas que se borrarían:** 0
- **Confianza:** alta

#### P4b-12 · `IMAGE_CACHE` nunca se limpia y puede crecer indefinidamente

- **Tipo:** escala / riesgo
- **Dónde:** `crates/barrier-kit/src/menu.rs:2477-2503`
- **Evidencia:**
  ```rust
  thread_local! {
      static IMAGE_CACHE: RefCell<HashMap<u64, Option<ImageHandle>>> = RefCell::new(HashMap::new());
  }
  ```
- **Por qué importa:** Cada icono distinto (por hash de bytes y tamaño) se queda en memoria para siempre. Una app que muestre muchos iconos de menú dinámicos (por ejemplo, historial reciente con previsualizaciones) aumenta el RSS sin límite.
- **Arreglo mínimo:** Añadir un límite LRU (por ejemplo, 64 entradas) o limpiar al cerrar el popup.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4b-13 · La normalización de búsqueda está duplicada entre kit y shell

- **Tipo:** sobra
- **Dónde:** `crates/barrier-kit/src/menu.rs:1479-1493` y `crates/malaysia-shell/src/menubar/focused.rs:987-995`
- **Evidencia:**
  ```rust
  fn normalize_query(text: &str) -> String {
      text.to_lowercase()
          .replace('á', "a")
          .replace('é', "e")
          ...
  }
  ```
- **Por qué importa:** Las dos funciones son idénticas salvo que una incluye mayúsculas. Cualquier cambio en la estrategia de búsqueda (por ejemplo, añadir `ç` o `å`) hay que hacerlo en dos sitios.
- **Arreglo mínimo:** Mover `normalize_query` a `barrier-kit` y reexportarla para `malaysia-shell`.
- **Tamaño:** S
- **Líneas que se borrarían:** ~14
- **Confianza:** alta

#### P4b-14 · El parser de atajos para dbusmenu y el formateador visual parsean la misma cadena por separado

- **Tipo:** sobra / reinvención
- **Dónde:** `crates/barrier-kit/src/app_menu.rs:172-196` y `crates/barrier-kit/src/menu.rs:131-166`
- **Evidencia:**
  ```rust
  // app_menu.rs
  "ctrl" | "control" => "Control",
  // menu.rs
  "ctrl" | "control" => ctrl = true,
  ```
- **Por qué importa:** La misma gramática de atajo (`Ctrl+Shift+S`, `Alt+Inicio`) se parsea dos veces con lógica ligeramente diferente. `app_menu.rs` produce `Vec<String>` para D-Bus; `menu.rs` produce glifos macOS. Mantenerlos sincronizados es propenso a errores.
- **Arreglo mínimo:** Tener un único parser que devuelva una estructura intermedia (modificadores + tecla) y dos funciones de renderizado.
- **Tamaño:** M
- **Líneas que se borrarían:** ~20
- **Confianza:** media

#### P4b-15 · `to_dbusmenu` en `app_menu.rs` duplica la lógica de `layout.rs`

- **Tipo:** sobra
- **Dónde:** `crates/barrier-kit/src/app_menu.rs:316-423` vs `crates/barrier-kit/src/app_menu/layout.rs:62-235`
- **Evidencia:** Ambas funciones recorren `AppMenu`/`Menu`/`MenuItem` generando ids con `IdMap` y propiedades dbusmenu (`label`, `enabled`, `shortcut`, `toggle-type`, etc.).
- **Por qué importa:** Cualquier cambio en el mapeo (por ejemplo, el bug de separadores de P4b-02) debe aplicarse en ambos sitios. Además `to_dbusmenu` repite el mismo error de `__separator__` compartido.
- **Arreglo mínimo:** Hacer que `to_dbusmenu` llame a `build_layout` y convierta el `LayoutItem` resultante a `RawItem`.
- **Tamaño:** M
- **Líneas que se borrarían:** ~80
- **Confianza:** alta

#### P4b-16 · `on_pointer_leave` no cancela el temporizador de apertura pendiente

- **Tipo:** bug
- **Dónde:** `crates/barrier-kit/src/menu.rs:1127-1133`
- **Evidencia:**
  ```rust
  root = root.on_pointer_leave(move |event: Event<PointerEventData>| {
      let pos = event.global_location();
      let mut hs = hover_state_leave.borrow_mut();
      hs.last = Some((Instant::now(), pos));
      hs.pending = None;
  });
  ```
- **Por qué importa:** Se limpia `pending`, pero el `timers.after(SUBMENU_INTENT_DELAY, ...)` ya fue programado. Si el cursor sale del menú, el temporizador puede ejecutarse más tarde y abrir un submenú en un popup que ya no tiene foco.
- **Arreglo mínimo:** Guardar el `generation` actual y comprobarlo dentro del callback del timer (ya se hace parcialmente), pero también cerrar cualquier popup hijo al salir del menú raíz.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

### Nice to have

#### P4b-17 · El parámetro `_menu` de `Server::register` no se usa

- **Tipo:** sobra
- **Dónde:** `crates/barrier-kit/src/app_menu/server.rs:58-62`
- **Evidencia:**
  ```rust
  fn register(
      _menu: AppMenu,
      handle: MenuHandle,
      on_action: Arc<dyn Fn(&str) + Send + Sync>,
  ) -> Option<crate::platform::AppMenuAddress> { ... }
  ```
- **Por qué importa:** El menú ya está dentro del `MenuHandle`. Pasarlo de nuevo confunde la API y sugiere que podría usarse para algo.
- **Arreglo mínimo:** Eliminar el parámetro `_menu`.
- **Tamaño:** S
- **Líneas que se borrarían:** 1
- **Confianza:** alta

#### P4b-18 · `render_item` tiene 23 argumentos y varios no se usan

- **Tipo:** abstracción
- **Dónde:** `crates/barrier-kit/src/menu.rs:1929-1955`
- **Evidencia:** Los parámetros `_palette_source`, `_popup`, `_items`, `_focused`, `_open_child_id`, `_open_child_index`, `_on_open_submenu`, `_submenu_source` llevan prefijo `_`.
- **Por qué importa:** Es una función enorme que recibe más estado del que necesita. Esto dificulta las refactorizaciones y aumenta el coste de copiar closures.
- **Arreglo mínimo:** Agrupar los parámetros usados en una estructura `RenderContext` y eliminar los que no se usan.
- **Tamaño:** M
- **Líneas que se borrarían:** ~8
- **Confianza:** media

#### P4b-19 · `visible_items` y `search_result_lists` reconstruyen listas en cada render aunque la consulta no cambie

- **Tipo:** escala
- **Dónde:** `crates/barrier-kit/src/menu.rs:1552-1567` y `1576-1661`
- **Evidencia:** Se llaman desde `render()` con `query_text` actual; no hay caché de resultados para la misma consulta.
- **Por qué importa:** Con muchas entradas buscables, filtrar, normalizar y ordenar en cada frame desperdicia CPU. El cambio de `query_text` ya dispara `use_state`.
- **Arreglo mínimo:** Cachear el último `(query, menu_results, help_results)` y reutilizarlos si la consulta no cambia.
- **Tamaño:** S
- **Líneas que se borrarían:** 0
- **Confianza:** media

#### P4b-20 · El mapeo de dbusmenu en `malaysia-shell` reinventa el parsing de propiedades

- **Tipo:** reinvención
- **Dónde:** `crates/malaysia-shell/src/menubar/dbusmenu_map.rs:36-80`
- **Evidencia:** Extrae manualmente `label`, `enabled`, `visible`, `type`, `children-display`, `toggle-type`, `toggle-state`, `shortcut`, `icon-name`, `icon-data`.
- **Por qué importa:** `barrier-kit` ya tiene `to_dbusmenu`/`build_layout` para la dirección inversa y el `RawItem` es casi idéntico al `LayoutItem` de kit. El mapeo podría compartir más lógica (por ejemplo, `mnemonic` es el inverso de `dbusmenu_label` pero vive solo en shell).
- **Arreglo mínimo:** Mover `mnemonic`/`dbusmenu_label` a `barrier-kit` como funciones inversas públicas y compartir la lista de propiedades conocidas.
- **Tamaño:** M
- **Líneas que se borrarían:** ~10
- **Confianza:** baja

---

## Brecha AppKit

Qué falta en el alcance para que una app de BarrierKit no tenga que reinventar nada:

| Qué falta | Token en Neptunia | Quién lo reimplementa hoy |
|-----------|-------------------|---------------------------|
| Menú contextual (right-click) genérico | `MENU_*` en `tokens.json` | Cada app construye su propio `MenuView` |
| Barra de menús integrada en ventanas no SSD | `MENUBAR_*` | No existe; solo layer-shell |
| Atajos globales registrables | `SHORTCUT_*` (no existe) | `app_menu::find_menu_shortcut` solo en foco |
| Menú con imágenes arbitrarias (no solo PNG) | `MENU_ICON_*` (no existe) | `MenuIcon::Image` limitado a PNG |
| Accesibilidad: roles ARIA para menús | `A11Y_*` (no existe) | Solo `a11y_builder` con `probe` |

---

## Preguntas

1. **¿El callback `on_menu_action` debe correr en el hilo de UI o está documentado como thread-safe?** Si se mantiene en D-Bus, hay que documentarlo; si no, hay que moverlo al hilo de UI.
2. **¿Es intencional que `MenuView::PartialEq` siempre sea `true`?** Si es un atajo deliberado, ¿debería llevar `// shortcut:`?
3. **¿Se soportarán múltiples separadores con ids distintos en dbusmenu, o se prefiere colapsarlos siempre?** Esto afecta el arreglo de P4b-02.
4. **¿El registro D-Bus debe ser síncrono porque el `appmenu` object necesita la dirección inmediatamente, o puede devolverse una dirección provisional?** Esto decide el arreglo de P4b-04.

---

## Revisado y correcto

| Semilla | Estado | Motivo |
|---------|--------|--------|
| S-21 | **Correcto** | Los atajos locales se ejecutan en el hilo de UI (`platform/mod.rs:3495-3514`) y las acciones D-Bus en el hilo de D-Bus. Es una separación de responsabilidades, no un bug, aunque el callback `on_menu_action` debería documentar su hilo. |

---

## Verdict

Los menús funcionan bien en el caso feliz, pero tienen tres grietas en el registro D-Bus (reuso de rutas, divergencia de ids y bloqueo del hilo de UI) y dos problemas de rendimiento (`truncate_middle` cuadrático y `measure_text` sin caché) que se notarán en menús grandes. Corregir primero P4b-01, P4b-02, P4b-03 y P4b-04. Después P4b-06, P4b-11 y P4b-12.

## Lean

- **Líneas que se borrarían:** ~120 (duplicados `to_dbusmenu`, `normalize_query`, parser de atajos, parámetros no usados).
- **Dependencias que se borrarían:** 0.

## Not checked

- `crates/malaysia-shell/src/menubar/dbusmenu.rs` más allá de sus primeras 300 líneas.
- `crates/malaysia-shell/src/menubar/bar.rs` y `appmenu.rs` (no están en el alcance).
- El comportamiento real con `UPDATE_GOLDEN` y versiones distintas de Skia.
- La ruta de fallback 14c cuando no hay bus de sesión D-Bus.
