# 18. Idiomas

**Objetivo:** que una app muestre su texto en el idioma del usuario sin mezclar
texto y código.
**Alcance:** desde un texto nuevo en la interfaz hasta verlo en español o
inglés. No cubre formatos de fecha ni números.
**Lector:** quien añade o cambia texto visible en una app del kit.

**Entradas:** los ficheros `i18n/<idioma>/<crate>.ftl` de la app.
**Salida:** `tr!("id")` devuelve el texto en el idioma elegido; si falta,
en inglés.

## Cómo se elige el idioma

`vow_appearance::language` decide al arrancar, en este orden:

1. `language` en el dominio de la app (`~/.config/malaysia/<app>.domain`), que
   escribe la fila «Idioma» de sus preferencias.
2. `language` en `~/.config/malaysia/language.domain`, que escribe Ajustes.
3. `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` y `LANG`, en el orden de gettext. Con el
   locale `C` o `POSIX` no se pide ninguno.
4. El inglés, idioma de respaldo de cada crate (`i18n.toml`).

`es_MX`, `es_AR` o `es_ES` caen en `es`. El cambio se ve al volver a abrir la
app.

## Pasos para traducir una app

1. Añade a su `Cargo.toml` `i18n-embed`, `i18n-embed-fl` y `rust-embed` (las
   tres `.workspace = true`).
2. Crea `i18n.toml` junto al `Cargo.toml`:
   ```toml
   fallback_language = "en"

   [fluent]
   assets_dir = "i18n"
   ```
3. Crea `i18n/en/<crate>.ftl` e `i18n/es/<crate>.ftl`, con los mismos ids en
   el mismo orden:
   ```
   play = Reproducir
   minutes-left = Quedan { $minutes } min
   ```
4. Crea `src/localize.rs` con una sola línea, `barrier_kit::localize!();`, y
   declara `mod localize;` en `main.rs`. La macro crea el `LOADER` de la app y
   su `tr!`.
5. Llama a `barrier_kit::i18n::use_app_language("<app>")` en la primera línea
   de `main`, con el mismo nombre que usa `load_defaults`.
6. Pon `app_language_row("<app>")` en sus preferencias.
7. Escribe el texto en el código solo con `tr!`:
   ```rust
   // ✗
   Button::new("Reproducir")
   // ✓
   Button::new(tr!("play"))
   tr!("minutes-left", minutes = left)
   ```
   `tr!("id")` da un `&'static str`; con variables da un `String`. Una
   `Action` toma el título como función: `Action::new("next", || tr!("next"))`.

`fl!` comprueba al compilar que el id y sus variables existen en el fichero
en inglés.

## Añadir un idioma

1. Copia `i18n/en/<crate>.ftl` a `i18n/<idioma>/<crate>.ftl` en cada crate.
2. Traduce los valores; los ids no cambian.

Un mensaje que falte sale en inglés.
