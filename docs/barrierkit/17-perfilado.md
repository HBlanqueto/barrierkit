# 17. Perfilar con Tracy

**Objetivo:** medir cuánto tarda cada frame de una app del kit y en qué parte se
va el tiempo. Empieza al lanzar la app con Tracy y acaba al leer las zonas en
el visor.
**Lector:** quien busca por qué una app va lenta o comprueba que un cambio la
acelera.
**Entradas:** una app que llame a `barrier_kit::init_tracing()` y la feature
`barrier-kit/tracy`.
**Salidas:** una traza en el visor de [Tracy](https://github.com/wolfpld/tracy)
con un frame por cada `draw` y sus zonas.

English: [17-perfilado.en.md](17-perfilado.en.md).

## Pasos

1. Entra en el dev shell: `nix develop`. Trae el visor `tracy`.
2. Lanza la app en release con la capa de Tracy:

   ```sh
   ./dev.sh tracy marea
   ```

3. Abre el visor en otra terminal del dev shell y conéctate a la app que
   aparece en la lista:

   ```sh
   tracy
   ```

4. Usa la app. El cliente solo guarda datos mientras el visor está conectado.
5. Lee las zonas de cada frame:

   | Zona | Mide | Código |
   |---|---|---|
   | `draw` | El frame completo de una superficie | `platform/surface.rs` |
   | `acquire` | La espera de la imagen del swapchain | `platform/surface.rs` |
   | `layout` | El layout y las reacciones a `on_sized` | `platform/surface.rs` |
   | `render` | La grabación de los comandos de Skia | `platform/surface.rs` |
   | `glass` | El cristal progresivo, una pasada por capa | `glass.rs` |
   | `material` | El material de medios desenfocado | `glass.rs` |
   | `present` | El envío a la GPU y la presentación | `platform/surface.rs` |

## Cómo leerlo

- `render` mide CPU. El trabajo de la GPU aparece como espera en `acquire` y
  `present`.
- Freya 0.4.3 repinta el árbol entero en cada frame. Una ventana quieta no debe
  tener frames nuevos.
- Cada popup tiene su propio `draw`, así que añade frames a la traza.

## Una app nueva

1. Llama a `barrier_kit::init_tracing()` al principio de `main`. Respeta
   `RUST_LOG` como `tracing_subscriber::fmt::init()`.
2. Compila con `--features barrier-kit/tracy`. Sin la feature, las zonas son de
   nivel `trace` y no cuestan nada con el filtro por defecto.

## Versión de Tracy

El visor y el cliente deben hablar el mismo protocolo. El flake trae Tracy
0.13.1, y `Cargo.lock` fija `tracy-client` 0.18.4 y `tracy-client-sys` 0.28.0,
que son su pareja. Si subes uno, sube el otro en el mismo commit.
