# Imágenes de la documentación

**Objetivo:** saber de dónde sale cada imagen y cómo regenerarla.
**Lector:** quien actualiza la documentación tras un cambio visual.

| Carpeta | Contenido | Origen |
|---|---|---|
| `componentes/` | Un control o vista suelto, en claro y oscuro | Copia de `crates/barrier-kit/tests/golden/` |
| `apps/` | Ventanas completas de las apps de prueba | Capturas de las apps en ejecución |
| `ejemplos/` | Los ejemplos de `crates/barrier-kit/examples/` | Capturas de los ejemplos en ejecución |

Todas son renders reales del código. Ninguna está retocada.

## Regenerar `componentes/`

1. Entra en el dev shell: `nix develop`.
2. Regenera las referencias: `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
3. Cópialas: `cp crates/barrier-kit/tests/golden/*.png docs/barrierkit/assets/componentes/`.

## Regenerar `apps/` y `ejemplos/`

Se capturaron el 2026-11-03 en sway sin pantalla, con lavapipe como driver
Vulkan y la fuente Geist instalada:

1. Arranca sway sin pantalla con ventanas flotantes y CSD:

   ```sh
   WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 sway -c sway.cfg
   ```

   ```
   output HEADLESS-1 mode 1400x900 bg #3a4256 solid_color
   default_border none
   for_window [app_id=".*"] floating enable, border csd
   ```

2. Prepara la apariencia y el panel en carpetas propias:
   - `MALAYSIA_CONFIG_DIR/appearance.domain` con `color_scheme` (y
     `sidebar_style` para la barra plana);
   - `MALAYSIA_DATA_DIR/io.github.hblanqueto.malaysia.<app>.state` con
     `pane = "<id>"` para abrir un panel;
   - `MALAYSIA_CONFIG_DIR/tengen-app-show.domain` con `layout = "Top"`, `"Rail"`…
3. Lanza la app con `VK_ICD_FILENAMES` apuntando a lavapipe y espera unos
   segundos.
4. Recorta la ventana con `grim -g "x,y ancho×alto"`, tomando la geometría de
   `swaymsg -t get_tree`.

Sway no respeta la petición de CSD en el primer configure. Para capturar, el
binario se compiló con un cambio local que no está en `main`, en
`platform/window.rs`: tratar el configure como `DecorationMode::Client`.

Ver [14, «El primer frame»](../14-plataforma.md#el-primer-frame).
