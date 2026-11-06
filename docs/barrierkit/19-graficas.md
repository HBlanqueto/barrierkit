# 19. Gráficas

**Objetivo:** dibujar datos con las gráficas del kit, de 2D a 3D.
**Lector:** quien muestra series, repartos o mediciones, como un monitor del
sistema.

## Qué hay

El módulo `barrier_kit::charts` sigue a Swift Charts: una gráfica recibe
series y las dibuja con una marca. Todo se pinta con Skia en un solo elemento;
ejes, leyenda y lectura son texto del kit.

| Componente | Marcas | Para qué |
|---|---|---|
| `Chart` | `MarkKind::Line`, `Area`, `Bar`, `Point` | Series sobre un eje: historiales, comparaciones |
| `SectorChart` | pastel o `.donut(true)` | Repartos de un total |
| `HeatMap` | rejilla de celdas | Intensidad por fila y columna |
| `Chart3D` | `surface` o `points` | Superficies z = f(x, y) y nubes de puntos |

## Pasos

1. Crea la gráfica con su etiqueta de accesibilidad y su marca:

   ```rust
   Chart::new("Uso de CPU", MarkKind::Area)
       .stacked(true)
       .y_domain(0.0, 100.0)
       .unit("%")
       .series(Series::new("Usuario", usuario))
       .series(Series::new("Sistema", sistema))
   ```

2. Elige cómo se unen los puntos con `.interpolation(..)`: `Linear` (por
   defecto), `Monotone` (curva sin sobrepasar los datos) o `Step`.
3. Añade etiquetas del eje X con `.x_labels(..)` y líneas de referencia con
   `.rule(valor)`. Sin `.y_domain`, el eje Y va de 0 al máximo redondeado.
4. Para un reparto, usa `SectorChart::new(..).slice(nombre, valor)`; la
   leyenda muestra cada valor y su porcentaje, y el donut el total.
5. Para 3D, pasa una rejilla de alturas a `Chart3D::surface` o puntos
   `[x, y, z]` a `Chart3D::points`. El usuario la gira arrastrando.

Al pasar el puntero sobre un `Chart`, una regla marca la posición y la
leyenda muestra el valor de cada serie en ese punto. Sin puntero, muestra el
último valor.

## Valores

Todos viven en `components.chart` de `vow-design` (`CHART`):

| Token | Valor | Uso |
|---|---|---|
| `height` | 180 | Alto por defecto del área de datos |
| `line-width` | 2 | Grosor de las líneas |
| `point-size` | 7 | Diámetro de los puntos |
| `bar-gap` / `bar-radius` | 0,25 / 2 | Hueco entre grupos de barras y su radio |
| `area-opacity` | 0,35 | Opacidad del relleno de las áreas |
| `grid-width` / `rule-width` | 0,5 / 1 | Rejilla y reglas |
| `ticks` | 4 | Marcas deseadas en el eje Y (pasos de 1, 2 o 5 × 10ⁿ) |
| `sector-inner` / `sector-inset` | 0,62 / 1,5° | Hueco del donut y separación entre sectores |
| `azimuth` / `inclination` | −30° / 25° | Pose inicial de `Chart3D` |
| `drag-degrees` | 0,5 | Grados por punto al arrastrar |

Los colores de las series siguen `series` (azul, verde, naranja, morado,
rojo, amarillo, rosa), que apunta a los colores de acento del sistema
(`CHART_SERIES`).

## Prueba

`./dev.sh app trazo` abre Trazo: una pestaña con cada marca 2D, otra con un
monitor del sistema que lee `/proc` cada segundo y otra con las gráficas 3D
(capítulo [15](15-apps-de-prueba.md#trazo-trazo)).
