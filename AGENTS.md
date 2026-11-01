# AGENTS.md — barrierkit

## Ramas y push (permanente)

- Todo cambio va **directo a `main`**. No crees ramas, ni locales ni remotas,
  ni hagas push a otra rama. Vale para cualquier agente (opencode, Claude Code
  en local o en la nube, DeepSeek, Kimi, etc.).
- Hay varias sesiones trabajando a la vez: `git pull --ff-only` antes de
  empezar y antes de cada push. Nunca `push --force`, `reset --hard` sobre
  commits subidos ni rebase de historia publicada.
- malaysia fija barrierkit por rev (`Cargo.toml`, `flake.nix`, `flake.lock`).
  Ese rev debe estar en `main`; un commit que solo vive en una rama puede
  desaparecer de GitHub y romper el build de malaysia.
- `for-malaysia` se unió a `main` el 2026-10-31; su historia vive en el tag
  `archivo/for-malaysia`.

## Código mínimo (reglas duras, estilo ponytail)

Se cumplen al pie de la letra. Ganan el diff más corto y la respuesta más corta.

1. **Reutiliza antes de escribir.** Busca con `grep` si ya existe la función, el
   token o el módulo. Cada valor y cada regla viven en un solo sitio; nunca copies.
2. **Nada «por si acaso».** Prohibido añadir parámetros, features, traits,
   genéricos, opciones o ramas que hoy nadie usa. El código muerto se borra: no
   se comenta ni se tapa con `#[allow(dead_code)]`.
3. **Lo más directo.** Nada de wrappers ni abstracciones de un solo uso. Se
   extrae a la tercera repetición, no antes.
4. **Una función, una tarea; un módulo, un motivo de cambio.** Si un bloque
   necesita un comentario para entenderse, extráelo a una función con ese nombre.
5. **Los nombres sustituyen al comentario:** `dias_transcurridos`, no `d`.
   Booleanos con forma de pregunta: `focused`, `zoom_supported`.
6. **Comentarios: ninguno por defecto.** Solo se permiten:
   - `// SAFETY:` en cada `unsafe`;
   - una línea con el *porqué* de un invariante cuya rotura desestabiliza y que
     el código no puede expresar: orden de protocolo, rodeo de un bug externo
     (con enlace) o seguridad. Cita el ADR si existe.

   Prohibido: explicar qué hace el código, contar historia («antes…», «ahora…»),
   `TODO` sin issue, código comentado, cabeceras decorativas y `///` que repiten
   el nombre. Si borras docs en un crate con `#![warn(missing_docs)]`, quita el
   lint en el mismo commit.
7. **Diff mínimo.** Toca solo lo que pide el cambio: no reformatees ni
   renombres de paso. El formato y los lints los ponen las herramientas.
8. **Tests solo para lógica pura con riesgo**, y que el nombre sea la
   especificación. Si borras código, borra también sus tests.
9. **Ahorra tokens.** Lee rangos, no ficheros enteros. No pegues en el chat el
   código que ya está en el diff. Cierra con 1–3 líneas: qué cambió y qué queda.

### Así se aplica en barrierkit

```rust
// ✗ literal y geometría recalculada
let x = 8.0 + i as f64 * (12.0 + 8.0);
// ✓ token y función que ya existen (vow-design, vow-appearance::traffic)
let (x, y, d) = vow_appearance::traffic::button_rect(i);
```

```rust
// ✗ doc que repite el nombre
/// Whether the window currently has focus.
pub fn focused(mut self, focused: bool) -> Self { … }
// ✓ sin comentario: el nombre basta
pub fn focused(mut self, focused: bool) -> Self { … }
```

```rust
// ✓ comentario que se queda (platform/egl.rs)
// SAFETY: called once on the calloop thread; libEGL is present because …
```

- Los colores, radios, duraciones y métricas nuevos van a `vow-design`, nunca
  a `barrier-kit`.
- Un control nuevo compone primitivas del kit; no reimplementa la barra
  lateral, los menús ni los semáforos (ADR 0047).
