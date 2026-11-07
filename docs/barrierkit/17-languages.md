# 17. Languages

**Goal:** an app shows its visible text in the user's language, with no text
literal in the code.
**Scope:** from adding the translation files to reading a text with `tr!`. It
covers the `i18n` module, the `localize!` and `tr!` macros and how the
language is chosen. The language picker widgets are in
[12. Patterns](12-patterns.md). Date and number formats are not covered.
**Reader:** whoever adds or changes visible text in a BarrierKit app.
**Input:** an app that starts with `Application::new(name)`
([2. First app](02-first-app.md)). **Output:** every visible text comes from
`i18n/<language>/<package>.ftl`; English fills in any missing message.

## Steps

1. Add the three translation crates to the app's `Cargo.toml`, with the
   versions BarrierKit uses:

   ```toml
   i18n-embed = { version = "0.16", features = ["fluent-system"] }
   i18n-embed-fl = "0.10"
   rust-embed = "8"
   ```

2. Create `i18n.toml` next to `Cargo.toml`:

   ```toml
   fallback_language = "en"

   [fluent]
   assets_dir = "i18n"
   ```

3. Create `i18n/en/<package>.ftl` and `i18n/es/<package>.ftl`. `<package>` is
   the package name in `Cargo.toml`. Both files have the same ids:

   ```
   play = Play
   minutes-left = { $minutes } min left
   ```

4. Create `src/localize.rs` with one line, and declare it at the crate root
   (`mod localize;` in `main.rs`):

   ```rust
   bk::localize!();
   ```

5. Choose the language on the first line of `main`, before any `tr!`. Use the
   same name as `Application::new` and `load_defaults`:

   ```rust
   fn main() -> Result<(), RunError> {
       bk::i18n::use_app_language("notes");
       Application::new("notes").window("Notes", content).run()
   }
   ```

6. Put `app_language_row("notes")` in the app's preferences
   ([12. Patterns](12-patterns.md)).
7. Read every visible text through `tr!`:

   ```rust
   use crate::localize::tr;

   // ✗
   Button::new("Play")
   // ✓
   Button::new(tr!("play"))
   tr!("minutes-left", minutes = left)
   ```

8. Build. `tr!` fails to compile when the id or a variable is missing from the
   English file.

## How the language is chosen

The kit asks `ctappearance::language` once per process, the first time a
text is read. The first source that gives a value wins:

| Order | Source | Written by |
|---|---|---|
| 1 | `language` in `<name>.domain` in the config dir | The app's language picker |
| 2 | `language` in `language.domain` in the config dir | The system language picker (Settings) |
| 3 | `LANGUAGE`, then the first of `LC_ALL`, `LC_MESSAGES`, `LANG` | The session |
| 4 | `fallback_language` of `i18n.toml` (`en`) | The app |

- The config dir is `$MALAYSIA_CONFIG_DIR`, or `$XDG_CONFIG_HOME/malaysia`
  (`~/.config/malaysia`).
- Step 3 follows gettext: `LANGUAGE` is a `:` list and goes before the locale.
  With the `C` or `POSIX` locale, step 3 requests nothing.
- A locale becomes a tag: `es_MX.UTF-8` → `es-MX`. i18n-embed negotiates
  `es-MX` down to the `es` files.
- An empty `language` value counts as not set.
- The kit's own texts (menus, pickers, dialogs) follow the same choice.

A change shows the next time the app opens: `tr!("id")` keeps the first text
it reads for the life of the process.

## `localize!`

`bk::localize!()` expands in the app crate, so it must live in the
module `crate::localize`. It creates:

| Item | What it is |
|---|---|
| `struct Localizations` | The `i18n/` folder, embedded with `rust-embed` |
| `pub static LOADER` | A `LazyLock<FluentLanguageLoader>` built with `i18n::load` on first use |
| `macro tr!` (`pub(crate)`) | `bk::tr!` bound to `crate::localize::LOADER` |

## `tr!`

The app calls the `tr!` that `localize!` creates. `bk::tr!` itself
takes the loader first: `tr!(LOADER, "id", ..)`.

| Form | Returns | Notes |
|---|---|---|
| `tr!("id")` | `&'static str` | Read once, then cached and leaked for the process |
| `tr!("id", name = value, ..)` | `String` | Read on every call; `value` is any Fluent argument |

Both forms call `i18n_embed_fl::fl!`, which checks the id and the argument
names against the fallback file at compile time. A message missing in the
chosen language shows in English.

## `i18n::use_app_language`

`use_app_language(domain: &str)` reads `language` from `<domain>.domain` and
keeps it for the process. Only the first call counts. Without a call, the
app starts at step 2 of the order above.

The `domain` is the app's name: `Application::new(name)`, `load_defaults(name)`
([13. App](13-app.md#settings-and-restoration)) and `use_app_language(name)` take the same string.
Otherwise the picker writes one file and the app reads another.

## `i18n::load`

`load(loader, assets) -> FluentLanguageLoader` selects the requested languages
in `loader` from `assets` and returns it. It turns off Unicode isolation marks
around arguments. When the selection fails, it logs a warning and returns the
loader as it is. `localize!` calls it; an app only calls it to build a
second loader by hand.

## The language pickers

`app_language_row(domain)` and `system_language_row()` show a `PopUpButton`
with three entries. Each choice writes the file at once and keeps the other
keys of the file.

| Entry | Writes |
|---|---|
| Automatic | Removes the `language` key |
| Español | `language = "es"` |
| English | `language = "en"` |

| Row | File | Detail text |
|---|---|---|
| `app_language_row(domain)` | `<domain>.domain` | Applies the next time the app opens |
| `system_language_row()` | `language.domain` | Applies to the apps you open afterwards |

## Adding a language

1. Copy `i18n/en/<package>.ftl` to `i18n/<language>/<package>.ftl` in each
   crate, BarrierKit's `crates/bk/i18n/` included.
2. Translate the values. The ids stay the same.
3. To offer it in the pickers, add it to `LANGUAGES` and to the entries in
   `crates/bk/src/patterns/language.rs`.

Next: [18. Platform](18-platform.md).
