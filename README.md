# BarrierKit

![BarrierKit, el toolkit de aplicaciones de Malaysia Compositor](docs/barrierkit/assets/marca/barrierkit-banner.png)

BarrierKit es el toolkit de aplicaciones de [Malaysia](https://github.com/HBlanqueto/malaysia): lo que GTK + libadwaita son para GNOME, o AppKit para macOS. Da las capas, los controles, los contenedores y los patrones con los que se construyen las apps propias de Malaysia y su shell.

English: [`README.en.md`](README.en.md).

## Crates

- `barrier-kit` — el toolkit: runner de la plataforma, primitivas de estilo, controles, vistas y patrones de ventana.
- `vow-design` — Tengen, el sistema de tokens de diseño. `tokens.json` y `components.json` pasan a constantes Rust tipadas con `build.rs`.
- `vow-appearance` — resuelve `appearance.domain` y el fondo del escritorio en una paleta concreta, y vigila el fichero.
- `infinity-protocol` — protocolo Wayland privado compartido con el compositor de Malaysia. Features `client` (kit) y `server` (compositor).

## Compilar

El repositorio se compila dentro del dev shell de Nix (ADR 0007):

```sh
nix develop
./dev.sh check
```

`./dev.sh check` pasa fmt, clippy, los checks del kit y los tests; su lista está en [`dev.sh`](dev.sh).

El binario precompilado de Skia para `freya-skia-bindings` 0.98.0 está en `nix/skia.nix`.

## Probar en cualquier escritorio

`barrier-kit` puede correr como cliente Wayland en cualquier compositor, no solo en Malaysia. Una app arranca con el protocolo básico (`wl_compositor`, `xdg_shell`, `wl_seat`) y sigue adelante si faltan los globals opcionales.

Si faltan `infinity_window_manager_v1`, `ext_foreign_toplevel_list_v1` / `zwlr_foreign_toplevel_manager_v1`, `wp_cursor_shape_manager_v1` u `org_kde_kwin_appmenu_manager`, el runner lo registra con `tracing::info!` y continúa.

## Documentación

[`docs/barrierkit/`](docs/barrierkit/README.md) explica el kit desde la primera ventana hasta el runner Wayland, con capturas reales.

## Decisiones

- ADR 0047 — [BarrierKit en capas](docs/decisiones/0047-barrierkit-en-capas.md)
- ADR 0048 — [BarrierKit en su propio repositorio](docs/decisiones/0048-barrierkit-repositorio-propio.md)
- ADR 0057 — [El cristal de pantalla completa cubre toda la ventana; la barra lateral sobre un fondo tintado](docs/decisiones/0057-cristal-y-barra-lateral-sobre-fondo.md)

Licencia: MIT
