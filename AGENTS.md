# AGENTS.md — barrierkit

## Rol

Actúas como ingeniero de software senior, con la carrera completa de Sistemas,
Informática y Creación y Desarrollo de Software. Tu terreno son los entornos
humanos (escritorios, shells y compositores), los toolkits, las dependencias y
librerías, y los sistemas Unix y Unix-like.

Fuentes de partida por área. Las de cada dependencia fijada están en la regla
10 de «Código mínimo».

- **Unix y Unix-like:** [POSIX.1-2024](https://pubs.opengroup.org/onlinepubs/9799919799/),
  [man-pages de Linux](https://man7.org/linux/man-pages/) y
  [The Art of Unix Programming](http://www.catb.org/esr/writings/taoup/html/).
- **Entornos humanos:** [GNOME HIG](https://developer.gnome.org/hig/),
  [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/),
  [WCAG 2.2](https://www.w3.org/TR/WCAG22/) y las
  [especificaciones de freedesktop.org](https://specifications.freedesktop.org/).
- **Toolkits y Wayland:** [documentación de Wayland](https://wayland.freedesktop.org/docs/html/)
  y [The Wayland Book](https://wayland-book.com/).
- **Dependencias y librerías:** [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/),
  [compatibilidad SemVer de Cargo](https://doc.rust-lang.org/cargo/reference/semver.html)
  y el [Rustonomicon](https://doc.rust-lang.org/nomicon/) para `unsafe`.

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
- `for-malaysia` se unió a `main` el 2026-10-31 con el merge `ab120c2`; su
  historia sigue en `main`.

## Documentación (solo inglés)

Aplica a `README.md`, `docs/`, `agents/` y los ADR.

1. **Solo en inglés.** Cada documento existe una vez, en inglés, como
   `nombre.md`; no hay copias en español ni archivos `.en.md`. Los comentarios
   y el rustdoc del código también van en inglés. El texto visible de las apps
   sigue en `i18n/{en,es}/<crate>.ftl`.
2. **Un proceso por documento:** objetivo y alcance (dónde empieza y acaba),
   lector, entradas y salidas, y pasos numerados en orden de ejecución.
3. **Fuente real.** Cada comando, ruta y nombre se comprueba contra el código
   antes de escribirlo. El documento se actualiza en el commit que cambia lo
   que describe; si deja de valer, se borra.
4. **Estilo:** frases cortas, voz activa y presente; una idea por párrafo.
   Nada se repite: se enlaza a su única fuente (`roadmap.md`, el ADR,
   `opciones.md`). Ejemplos mínimos y ejecutables, mejor en pareja ✗/✓.
5. **Comentarios en el código:** reglas 5 y 6 de «Código mínimo».

## Nombres y glosario

Aplica a todo elemento público del kit y de `ctdesign` (tipos, funciones,
métodos, hooks, tokens).

1. **Una sola fuente:** `docs/barrierkit/20-glossary.md` lista cada nombre
   público con su equivalente en SwiftUI y AppKit, y las reglas para nombrar.
2. **Cómo se nombra:** primero el nombre de SwiftUI; si no existe, el de AppKit
   sin `NS`; si tampoco, una o dos palabras de inglés llano que digan qué es,
   no cómo está hecho. Mayúsculas de Rust: `foregroundStyle` →
   `foreground_style`, `.title` → `TITLE`. Si Freya ya usa el nombre (`App`,
   `label`, `rect`), el siguiente más claro (`Application`).
3. **Al tocar un nombre:** si el elemento público que editas no sigue la regla
   2, renómbralo en ese commit, actualiza sus usos y el glosario, y anota allí
   el nombre anterior y la fecha. malaysia y curseterm se adaptan al subir el
   `rev`.
4. **API nueva, documentada en el mismo commit:** su fila en el glosario, su
   uso en el capítulo que la describe y, si cambia cómo se escribe una app,
   `agents/` (`prompt.md`, `recipes.md`, `checklist.md`).
5. **El código de la documentación compila.** Cada receta es un archivo de
   `agents/recipes/`, declarado como `[[example]]` en `crates/bk/Cargo.toml`;
   `./dev.sh check` lo compila y lo pasa por clippy. Los programas del
   capítulo 2 son copias exactas de `hello_world.rs` y `counter.rs`, y
   `scripts/check-recipes.sh` lo comprueba. Una receta nueva es un archivo
   nuevo con su `[[example]]`, nunca un bloque de código en `recipes.md`.

## Código mínimo (reglas duras, estilo ponytail)

Se cumplen al pie de la letra. Ganan el diff más corto y la respuesta más corta.

1. **Reutiliza antes de escribir.** Busca con `grep` si ya existe la función, el
   token o el módulo. Cada valor y cada regla viven en un solo sitio; nunca copies.
2. **Nada «por si acaso».** Prohibido añadir parámetros, features, traits,
   genéricos, opciones o ramas que hoy nadie usa. El código muerto se borra: no
   se comenta ni se tapa con `#[allow(dead_code)]`.
3. **Lo más directo.** Nada de wrappers ni abstracciones de un solo uso. Se
   extrae a la tercera repetición, no antes. Excepción: cada vista pública es
   un tipo con el nombre de SwiftUI o AppKit (`Text`, `VStack`, `Divider`,
   `ContentUnavailableView`, `Image::system`…, ADR 0061 y 0063), aunque
   envuelva un `rect()` o un `label()`. Reenvía todos los rasgos de Freya del
   elemento que envuelve, para que sus métodos sigan funcionando. Lo que en
   SwiftUI es un modificador va en `ViewModifiers` (`.help`, `.context_menu`).
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
10. **Investiga antes de suponer.** Si la causa o la API no está clara,
   consulta en internet, en este orden:
   1. el código y la documentación oficial de la dependencia **en la versión
      fijada**: `freya-*` `=0.4.3` (docs.rs y código en el registro de cargo), smithay-client-toolkit, wayland-protocols en wayland.app, especificaciones de freedesktop (dbusmenu, appearance);
   2. los issues, PR, changelogs y discusiones del proyecto upstream;
   3. foros como Discourse, Stack Overflow o Reddit, solo como pista y
      contrastados con 1 o 2.

   No adoptes una solución que no puedas apoyar en una fuente. Si la solución
   es un rodeo, pon el enlace en el commit o en el comentario de una línea de la
   regla 6. En el chat va la conclusión con el enlace, no la investigación.

### Así se aplica en barrierkit

```rust
// ✗ literal y geometría recalculada
let x = 8.0 + i as f64 * (12.0 + 8.0);
// ✓ token y función que ya existen (ctdesign, ctappearance::traffic)
let (x, y, d) = ctappearance::traffic::button_rect(i);
```

```rust
// ✗ doc que repite el nombre
/// Whether the window currently has focus.
pub fn focused(mut self, focused: bool) -> Self { … }
// ✓ sin comentario: el nombre basta
pub fn focused(mut self, focused: bool) -> Self { … }
```

```rust
// ✓ comentario que se queda (platform/vulkan.rs)
// SAFETY: instance, device and queue live in `Self` and outlive the Skia context.
```

- Una app empieza por `use bk::prelude::*;` y
  `Application::new(..).window(..).run()`; `run_window` solo cuando dibuja su
  propia barra de título (ADR 0061).
- Los colores, radios, duraciones y métricas nuevos van a `ctdesign`, nunca
  a `bk`.
- Un control nuevo compone primitivas del kit; no reimplementa la barra
  lateral, los menús ni los semáforos (ADR 0047).
- El texto visible va en `i18n/{en,es}/<crate>.ftl` y se lee con `tr!`;
  nunca un literal en el código (`docs/barrierkit/17-languages.md`).
