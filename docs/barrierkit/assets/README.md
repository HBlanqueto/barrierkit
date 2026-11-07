# Imágenes de la documentación

**Objetivo:** saber de dónde sale cada imagen y cómo regenerarla.
**Lector:** quien actualiza la documentación tras un cambio visual.

| Carpeta | Contenido | Origen |
|---|---|---|
| `componentes/` | Un control o vista suelto, en claro y oscuro | Copia de `crates/barrier-kit/tests/golden/` |
| `estados/` | La titlebar y la barra lateral en cada estado de ventana (capítulo [7](../07-titlebars.md)) | Escenas del kit renderizadas sin pantalla por `platform/doc_shots.rs` |
| `marca/` | Icono y banner de BarrierKit | Logo en SVG y su versión PNG, en español e inglés |

Todas son renders reales del código. Ninguna está retocada.

## Regenerar `componentes/`

1. Entra en el dev shell: `nix develop`.
2. Regenera las referencias: `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
3. Cópialas: `cp crates/barrier-kit/tests/golden/*.png docs/barrierkit/assets/componentes/`.

## Regenerar `estados/`

1. Entra en el dev shell: `nix develop`.
2. Renderiza las escenas:
   `BARRIER_KIT_SHOTS=$PWD/docs/barrierkit/assets/estados cargo test -p barrier-kit --lib doc_shots`.

Cada escena fija el estado de la ventana (foco, maximizada, pantalla
completa), la barra del compositor, `FullscreenBar` y el diseño de titlebar.
La franja negra de arriba en las de modo foco es la barra del compositor, que
en una ventana real dibuja Malaysia.
