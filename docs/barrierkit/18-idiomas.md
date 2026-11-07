# 18. Languages

**Goal:** an app shows its text in the user's language without mixing text and
code.
**Scope:** from a new piece of interface text to seeing it in Spanish or
English. It does not cover date or number formats.
**Reader:** whoever adds or changes visible text in a kit app.

**Inputs:** the app's `i18n/<language>/<crate>.ftl` files.
**Output:** `tr!("id")` returns the text in the chosen language; if it is
missing, in English.

## How the language is chosen

`vow_appearance::language` decides at startup, in this order:

1. `language` in the app's domain (`~/.config/malaysia/<app>.domain`), written
   by the "Language" row of its preferences.
2. `language` in `~/.config/malaysia/language.domain`, written by Settings.
3. `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` and `LANG`, in gettext's order. With the
   `C` or `POSIX` locale nothing is requested.
4. English, the fallback language of each crate (`i18n.toml`).

`es_MX`, `es_AR` or `es_ES` fall into `es`. The change shows the next time the
app opens.

## Steps to translate an app

1. Add `i18n-embed`, `i18n-embed-fl` and `rust-embed` to its `Cargo.toml` (all
   three `.workspace = true`).
2. Create `i18n.toml` next to `Cargo.toml`:
   ```toml
   fallback_language = "en"

   [fluent]
   assets_dir = "i18n"
   ```
3. Create `i18n/en/<crate>.ftl` and `i18n/es/<crate>.ftl`, with the same ids in
   the same order:
   ```
   play = Play
   minutes-left = { $minutes } min left
   ```
4. Create `src/localize.rs` with a single line, `barrier_kit::localize!();`,
   and declare `mod localize;` in `main.rs`. The macro creates the app's
   `LOADER` and its `tr!`.
5. Call `barrier_kit::i18n::use_app_language("<app>")` on the first line of
   `main`, with the same name `load_defaults` uses.
6. Put `app_language_row("<app>")` in its preferences.
7. Write text in the code only through `tr!`:
   ```rust
   // ✗
   Button::new("Play")
   // ✓
   Button::new(tr!("play"))
   tr!("minutes-left", minutes = left)
   ```
   `tr!("id")` gives a `&'static str`; with variables it gives a `String`. An
   `Action` takes its title as a function: `Action::new("next", || tr!("next"))`.

`fl!` checks at compile time that the id and its variables exist in the
English file.

## Adding a language

1. Copy `i18n/en/<crate>.ftl` to `i18n/<language>/<crate>.ftl` in each crate.
2. Translate the values; the ids do not change.

A missing message shows in English.
