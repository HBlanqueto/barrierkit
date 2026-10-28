# BarrierKit

BarrierKit is the application toolkit for [MalaysiaDreams](https://github.com/HBlanqueto/malaysia): the GTK + libadwaita to its GNOME, or the AppKit to its macOS. It provides the layers, controls, containers and patterns used to build Malaysia's own apps and its shell.

## Crates

- `barrier-kit` — the toolkit itself: platform runner, style primitives, controls, views and window patterns.
- `vow-design` — Tengen, the design-token system. `tokens.json` and `components.json` become typed Rust constants through `build.rs`.
- `vow-appearance` — resolves `appearance.domain` and the desktop wallpaper into a concrete palette, plus a file watcher.
- `infinity-protocol` — private Wayland protocol shared with the MalaysiaDreams compositor. Features `client` (kit) and `server` (compositor).

## Build

The repository must be built inside the Nix dev shell (ADR 0007):

```sh
nix develop
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

The Skia prebuild for `freya-skia-bindings` 0.98.0 lives in `nix/skia.nix`.

## Probar en cualquier escritorio

`barrier-kit` puede correr como cliente Wayland en cualquier compositor, no solo en MalaysiaDreams. Los ejemplos de `crates/barrier-kit/examples` arrancan con el protocolo básico (`wl_compositor`, `xdg_shell`, `wl_seat`) y siguen adelante si faltan los globals opcionales.

```sh
nix develop
./dev.sh examples          # lista los ejemplos
./dev.sh example window_csd # abre una ventana contra el WAYLAND_DISPLAY actual
```

Qué se espera en GNOME (Mutter) y qué requiere Malaysia:

| Ejemplo | GNOME / Mutter | MalaysiaDreams |
|---|---|---|
| `window_csd` | Ventana con titlebar propia (CSD). Bordes redondeados, sombra y menú global **no** los pone GNOME. | Titlebar Tengen, bordes, sombra, blur y menú global integrados. |
| `window_ssd` | Pide SSD pero Mutter no anuncia `zxdg_decoration_manager_v1`; el kit cae a CSD y dibuja su titlebar. | SSD con titlebar del kit y semáforos; compositor pinta el chrome. |
| `dialog` | Abre ventana principal + diálogo modal como cliente normal. | Igual; Malaysia gestiona el diálogo como toplevel secundario. |
| `contador` | App con menú exportado vía D-Bus; sin `org_kde_kwin_appmenu_manager` el menú vive dentro de la ventana. | Menú global en la top bar del shell. |
| `menu_popup` | **No abre** en GNOME: necesita `zwlr_layer_shell_v1`, que Mutter no tiene. Sale con mensaje informativo. | Barra/menú popup en la capa `Top` del escritorio. |

Si faltan `infinity_window_manager_v1`, `ext_foreign_toplevel_list_v1` / `zwlr_foreign_toplevel_manager_v1`, `wp_cursor_shape_manager_v1` u `org_kde_kwin_appmenu_manager`, el runner lo registra con `tracing::info!` y continúa.

## Decisions

- ADR 0047 — [BarrierKit en capas](docs/decisiones/0047-barrierkit-en-capas.md)
- ADR 0048 — [BarrierKit en su propio repositorio](docs/decisiones/0048-barrierkit-repositorio-propio.md)

License: MIT
