# Tengen: de los JSON a las constantes de vow-design

Tengen es el sistema de diseño del escritorio (ADR 0032, renombrado en el
ADR 0046). Este documento cubre cómo un valor de diseño pasa de los JSON de
Tengen a las constantes Rust de `vow-design`, y cómo `vow-appearance` las
aplica. Empieza al editar un JSON y acaba cuando el kit, el compositor, la
shell y el portal leen la constante.

**Lector:** quien cambia un color, una medida o un umbral de diseño.

## Entradas y salidas

- **Entradas:** `crates/vow-design/tengen/`.
  - `tokens.json`: colores por tema, radios, espacios, tamaños, tipografía y
    sombras.
  - `components.json`: medidas y estados de los componentes, la shell, la
    titlebar y los factores de apariencia.
- **Salida:** `$OUT_DIR/tengen.rs`, que `crates/vow-design/src/lib.rs` incluye
  con `include!`. Son constantes de compilación (ADR 0011).
- **Consumidores:** `barrier-kit`, el compositor, la shell y el portal leen
  las constantes. `vow-appearance` las combina con `appearance.domain` y
  `wallpaper.state` para dar la apariencia resuelta.

Ningún otro formato sale de estos JSON. Nada fuera de `build.rs` los lee.

## Pasos para cambiar un valor

1. Edita `tokens.json` o `components.json`. Un valor de un componente puede
   ser un número o una referencia `grupo.clave` a `tokens.json`.
2. Compila. `crates/vow-design/build.rs`, con un módulo por familia en
   `build/`, lee los dos JSON con `serde_json` (solo en
   `build-dependencies`) y escribe `tengen.rs`. `cargo:rerun-if-changed`
   vigila los dos ficheros.
3. Si falta un campo que `vow-design` necesita, el build falla y nombra el
   campo y el fichero.
4. Ejecuta `./dev.sh check`.

No edites `crates/vow-design/src/` para cambiar un valor: allí viven los
tipos (`Rgba`, `Palette`, `Titlebar`, `ShellPalette`…) y la API.

## Valores que manda Tengen

- **Temas:** `light`, `dark` y `dark-tint`, con `LIGHT`, `DARK` y `DARK_TINT`.
- **Semáforos:** `traffic-close/min/zoom` son `#ff5f57`, `#febc2e` y `#28c840`
  en los tres temas. `traffic-inactive`, `traffic-rim`,
  `traffic-rim-inactive` y `traffic-glyph` están en `tokens.json`.
- **Titlebar:** `titlebar-bg`/`titlebar-bg-inactive`,
  `titlebar-title`/`titlebar-title-inactive` y `titlebar-line`.
- **Alturas:** `size.titlebar-h = 44` en las apps propias y
  `size.titlebar-ssd-h = 32` en la barra que dibuja el compositor.
- **Tipografía:** `font.families.sans` empieza por Geist; `titlebar-title` es
  13/17/600.
- **Sombras:** `shadow.layers.{light,dark}` guardan las capas ambient y
  contact; `shadow.unfocused` fija los factores de la ventana inactiva.
- **Shell:** `components.shell` define menubar, dock y lock.

## Apariencia y wallpaper

- `ShellPalette` (`SHELL_LIGHT`, `SHELL_DARK`, `SHELL_DARK_TINT`) se elige por
  `appearance.color_scheme`; `auto` resuelve a `light` con
  `ColorScheme::resolved()`.
- `window_tint_scope = "desktop"` mezcla barra y dock con el color dominante
  del wallpaper, que se lee de `wallpaper.state`.
- `malaysia-wallpaper` (ADR 0049) decodifica la imagen, calcula el dominante
  y escribe `~/.local/share/malaysia/wallpaper.state` (`dominant = [r, g, b]`).
- El compositor, el kit y el portal leen ese fichero y lo vigilan junto con
  `appearance.domain`.

## Comprobaciones

- `scripts/check-design-tokens.sh`, en `./dev.sh check` y en el check
  `design-tokens` del flake: ningún literal de color ni de medida de UI fuera
  de `vow-design` en `crates/barrier-kit/src`.

## Relación

- [`interfaz.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/interfaz.md): el mapa visual completo.
- [`ventanas.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/ventanas.md): titlebar y semáforos.
- [`wallpaper.md`](https://github.com/HBlanqueto/malaysia/blob/main/docs/arquitectura/wallpaper.md): el componente del fondo y su estado.
- ADR 0011: los valores son constantes Rust.
- ADR 0032 y ADR 0046: Tengen como fuente de diseño.
