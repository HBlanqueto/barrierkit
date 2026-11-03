# 8. Vistas

**Objetivo:** conocer los contenedores de la capa 2 que organizan el contenido.
**Lector:** quien arma pantallas completas.

## Formularios: `Form`, `Section`, `Row`

Es la pieza de los ajustes: filas agrupadas en tarjetas.

```rust
Form::new()
    .section(
        Section::titled("Ventanas")
            .row(Row::new("Ampliación").control(Switch::new("Ampliación").on(true)))
            .row(Row::value("Estado", "Normal · 87 %"))
            .row(Row::navigation("Avanzado", |_| {})),
    )
    .highlighted(Some("Estado"))
```

![Formulario con fila de control, de valor y de navegación](assets/componentes/form-light.png)

| Fila | Para |
|---|---|
| `Row::new(título).control(..)` | Etiqueta y control |
| `Row::value(título, valor)` | Información de solo lectura |
| `Row::navigation(título, al_pulsar)` | Ir a una subpágina (chevron) |
| `Row::destructive(título, al_pulsar)` | Acción destructiva, en rojo |

- `.detail(texto)` añade una explicación bajo el título; los textos largos
  pasan a varias líneas sin cortar el control.
- `.keywords(&[..])` alimenta la búsqueda de `PreferencesWindow`.
- `Section::new()` va sin título; `.footer(texto)` añade un pie.
- `.highlighted(Some(título))` resalta la fila que eligió el buscador.

![Formulario con textos largos](assets/componentes/form-long-text-light.png)

## Tabla: `Table`

```rust
Table::new("Archivos", vec!["Nombre", "Tamaño"])
    .rows(filas)
    .sort(Some(SortOrder { column: 0, ascending: true }))
    .selected(Some(1))
    .on_sort(move |orden| criterio.set(Some(orden)))
    .on_select(move |fila| elegida.set(fila))
```

![Tabla ordenada por nombre con una fila elegida](assets/componentes/table-light.png)

Pulsar una cabecera pide un orden nuevo (`next_sort` alterna ascendente y
descendente); la app ordena sus filas.

## Rejilla: `CollectionView`

```rust
CollectionView::new("Documentos")
    .item(icono(), "Malaysia")
    .item(icono(), "Facturas")
    .selected(vec![1])
    .on_select(move |elegidos: Vec<usize>| seleccion.set(elegidos))
```

![Rejilla de iconos con uno elegido](assets/componentes/collection-light.png)

Admite selección múltiple con modificador.

## Pestañas: `TabView`

```rust
TabView::new("Preferencias")
    .tab(Symbol::MagnifyingGlass, "Buscar", pagina_buscar)
    .tab(Symbol::SpeakerWave, "Sonido", pagina_sonido)
    .selected(0)
```

![Pestañas Buscar y Sonido](assets/componentes/tab-view-light.png)

## Estados, hojas y alertas

```rust
empty_state(Symbol::MagnifyingGlass, "Sin resultados")
sheet(contenido)
Alert::new("¿Olvidar esta red?", "Tendrás que volver a escribir la contraseña.")
    .confirm("Olvidar", |_| {})
    .cancel("Cancelar", |_| {})
```

| Vacío | Alerta |
|---|---|
| ![Estado vacío](assets/componentes/empty-state-light.png) | ![Alerta con botón de confirmar](assets/componentes/alert-light.png) |

`sheet` muestra el contenido como diálogo centrado sobre un velo
(`Palette::scrim`) que cubre la ventana. `Alert` confirma con Intro y cancela
con Esc.

## Navegación: `History`

```rust
let historial = use_state(|| History::new(Seccion::Inicio));
historial.write().push(Seccion::Ajustes);
navigation_buttons(historial)
```

`History<T>` guarda la pila atrás/adelante. `navigation_buttons` dibuja ‹ y ›
y responde a las acciones `GO_BACK` (Alt+←) y `GO_FORWARD` (Alt+→).

## Búsqueda: `SearchIndex`

```rust
let mut indice = SearchIndex::new();
indice.add("wifi", "Wi-Fi", &["red", "inalámbrica"]);
let resultados = indice.search("inalam");
```

Busca sin distinguir mayúsculas ni tildes. `PreferencesWindow` lo construye
solo con los títulos y las `keywords` de cada `Row`.

## Diseño adaptable

```rust
let adaptable = use_breakpoint();
match adaptable.current() {
    Breakpoint::Wide => tres_columnas(),
    Breakpoint::Medium => dos_columnas(),
    Breakpoint::Compact => una_columna(),
}
```

Los cortes son `BREAKPOINT.medium` (620) y `BREAKPOINT.wide` (900).
`adaptable.attach(rect())` mide el elemento al que se engancha. Ondina lo usa
para pasar de tres columnas a una (capítulo [14](14-apps-de-prueba.md)).

Siguiente: [9. Patrones](09-patrones.md).
