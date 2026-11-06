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
| `StreamChart` | flujo o `.polar(true)` (reloj de flujo) | Cómo cambia el reparto con el tiempo |
| `Chart3D` | `surface`, `mesh` o `points` | Superficies z = f(x, y), modelos 3D y nubes de puntos |

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
5. Para ver cómo cambia un reparto con el tiempo, usa `StreamChart`. Su
   línea base «wiggle» (Byron y Wattenberg) minimiza la inclinación de las
   capas, y las series que alcanzan antes su pico van al centro. Con
   `.polar(true)` el tiempo da la vuelta a un reloj que empieza a las 12 y
   deja un paso abierto donde se unen principio y final.
6. Para 3D, pasa una rejilla de alturas a `Chart3D::surface`, vértices y
   triángulos a `Chart3D::mesh` (el modelo se ajusta a la caja) o puntos
   `[x, y, z]` a `Chart3D::points`. El usuario la gira arrastrando o elige un
   plano: libre, frente, arriba, lado o isométrica. La malla se dibuja en la
   GPU en una sola llamada (ADR 0060).

Como en Swift Charts, el eje Y va a la derecha y la leyenda debajo. Al pasar
el puntero sobre un `Chart`, una regla marca la posición y la leyenda muestra
el valor de cada serie en ese punto. Sin puntero, muestra el último valor.

## Valores

Todos viven en `components.chart` de `vow-design` (`CHART`):

| Token | Valor | Uso |
|---|---|---|
| `height` | 180 | Alto por defecto del área de datos |
| `line-width` | 2 | Grosor de las líneas |
| `point-size` | 7 | Diámetro de los puntos |
| `bar-gap` / `bar-radius` | 0,25 / 0 | Hueco entre grupos de barras; sin radio, como `BarMark` |
| `area-opacity` | 0,35 | Opacidad del relleno de las áreas |
| `grid-width` / `rule-width` | 0,5 / 1 | Rejilla y reglas |
| `ticks` | 4 | Marcas deseadas en el eje Y (pasos de 1, 2 o 5 × 10ⁿ) |
| `sector-inner` / `sector-inset` / `sector-radius` | 0,618 / 1,5 / 5 | Hueco del donut, separación y esquina de los sectores, como la muestra de la WWDC23 |
| `stream-hole` | 0,32 | Hueco central del reloj de flujo |
| `azimuth` / `inclination` | −30° / 25° | Pose inicial de `Chart3D` |
| `drag-degrees` | 0,5 | Grados por punto al arrastrar |

Los colores de las series siguen `series` (azul, verde, naranja, morado,
rojo, amarillo, rosa), que apunta a los colores de acento del sistema
(`CHART_SERIES`).

## Prueba

`./dev.sh app trazo` abre Trazo: una pestaña con cada marca 2D, otra con un
monitor del sistema que lee `/proc` cada segundo y otra con las gráficas 3D
(capítulo [15](15-apps-de-prueba.md#trazo-trazo)).
