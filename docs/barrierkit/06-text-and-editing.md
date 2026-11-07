# 6. Text and editing

**Goal:** add an editable text area, style its text, check its spelling and print it.
**Scope:** multi-line editing, rich text, the font panel, spelling, tag fields, printing and
the rule editor. One-line fields (`TextField`, `SearchField`, `SecureField`, `ComboBox`) are in
[5](05-controls.md#text-fields); menus and popovers in general are in
[7](07-menus-and-popovers.md); the portals in general are in [13](13-app.md).
**Reader:** anyone who builds an editor, a notes app or a filter.
**Input:** a window with state ([2](02-first-app.md)). **Output:** an editor with bold, italic,
fonts, links, spell checking and printing.

## Steps

1. Keep the text and its styles in the app:

   ```rust
   let notes = use_state(String::new);
   let styles = use_state(Vec::<StyleRun>::new);
   ```

2. Add the editor. It writes every change into `notes`:

   ```rust
   TextEditor::new(notes, "Notes").styles(styles).height(Size::px(200.0))
   ```

3. Style the text. Select some and press Ctrl+B or Ctrl+I, or Ctrl+T for the
   [font panel](#font-panel). The editor writes the runs into `styles`
   ([Rich text](#rich-text)).
4. Check spelling. Install a Hunspell dictionary for the app language (on NixOS, for example
   `hunspellDicts.en_US`). Misspelled words get red dots; a secondary click on one offers
   guesses ([Spelling](#spelling)).
5. Print it. Ask the portal for a page setup and hand it a PDF:

   ```rust
   Button::new("Print…").on_press(move |_| {
       let text = notes.peek().clone();
       spawn(async move { portal::print("Notes", move |setup| text_pdf(&text, setup)).await });
   })
   ```

## Edit menu

A secondary click on `TextField`, `SearchField`, `ComboBox`, `TokenField` or `TextEditor` opens
the edit menu: Undo, Redo, Cut, Copy, Paste and Select All. Each item focuses the field and
sends it its shortcut (Ctrl+Z, Y, X, C, V, A), so it acts on the field's own selection and
history.

| Field | Cut and Copy | Undo and Redo |
|---|---|---|
| `TextEditor` | Disabled without a selection | Follow the editor's history |
| One-line fields | Always enabled: Freya's `Input` does not expose its selection | Always enabled |

With the focus in a field, those shortcuts belong to the field before the app's menu.
In `TextField` and `TextEditor`, a click on a misspelled word adds the spelling items on top
([Spelling](#spelling)).

## Text editor

`text_editor::TextEditor` edits several lines, like `NSTextView`. It uses Freya's editing
engine (`freya-edit`) and the look of `TextField`.

| Method | Default | Effect |
|---|---|---|
| `TextEditor::new(value, placeholder)` | — | `value: State<String>` receives every edit |
| `.height(Size)` | fill | The text scrolls inside this height |
| `.styles(State<Vec<StyleRun>>)` | plain text | Turns on [rich text](#rich-text) |

| Key | Effect |
|---|---|
| Enter | New line |
| Escape | Drops the focus |
| Ctrl+Z | Undo |
| Ctrl+Y or Shift+Ctrl+Z | Redo |
| Ctrl+F | Opens the [find bar](#find-bar) |
| Ctrl+B, Ctrl+I | Toggle bold or italic on the selection (with `styles`) |
| Ctrl+T | Opens the font panel for the selection (with `styles`) |

A focused editor has an accent ring. When the app replaces `value`, the editor takes the new
text, clears the selection and starts a new undo history.

### Find bar

Ctrl+F opens a bar above the text, like SwiftUI's `findNavigator`: a `SearchField`, the counter
("2 of 3", or "Not Found"), previous and next buttons and Done. ↑, ↓ and Enter in the field
move between matches. The current match is selected, and the text scrolls to it
(`ScrollController::reveal_y`, [5](05-controls.md#scrolling)).

`find_matches(text, query)` returns the matches as `(start, end)` character indices. It ignores
case, does not overlap and returns nothing for an empty query.

## Rich text

Module `rich_text`. The text stays in the `State<String>`; the styles live beside it as runs
in character indices.

| Type | Fields |
|---|---|
| `TextStyle` | `bold`, `italic`, `link: Option<String>`, `family: Option<String>`, `size: Option<f32>`; `Default` is plain |
| `StyleRun` | `range: Range<usize>`, `style: TextStyle` |
| `FontChoice` | `family`, `size`, `bold`, `italic` |
| `Emphasis` | `Bold`, `Italic` |

```rust
let styles = use_state(|| vec![StyleRun {
    range: 0..5,
    style: TextStyle { bold: true, ..TextStyle::default() },
}]);
```

How the editor draws and edits runs:

| Behavior | Rule |
|---|---|
| Typing | Runs move with the text. Typed text inherits the bold, italic and font of the previous character, not its link |
| Ctrl+B, Ctrl+I | If the whole selection has the emphasis, it is removed; otherwise it is added |
| Links | Drawn in the accent and underlined. The pointer becomes a hand; a click opens the link through the `OpenURI` portal instead of moving the caret |
| Fonts | `family` and `size` replace `BODY` for their run |

The functions behind it are pure and work on any runs:

| Function | Returns |
|---|---|
| `char_styles(runs, length)` | One `TextStyle` per character |
| `runs_from(styles)` | Runs that merge equal neighbors and skip plain text |
| `restyle_after_edit(runs, old, new)` | Runs moved after an edit from `old` to `new` |
| `toggle(runs, length, range, Emphasis)` | Runs with the emphasis toggled on `range` |
| `font_at(runs, length, index)` | The `FontChoice` at a character; `BODY` when unset |
| `apply_font(runs, length, range, &FontChoice)` | Runs with that font on `range` |
| `link_at(runs, index)` | The link at a character, if any |

## Font panel

Module `font_panel`, after `NSFontPanel`. The panel opens in a
[popover](07-menus-and-popovers.md) with a preview and four columns:

| Column | Content |
|---|---|
| Collection | All Fonts, Recently Used (chosen in this session, newest first), Fixed Width |
| Family | The system fonts and those added with `register_font` ([4](04-appearance-and-tokens.md)) |
| Typeface | Regular, plus Italic, Bold and Bold Italic when the family has them |
| Size | 6, 8, 9, 10, 11, 12, 13, 14, 16, 20, 24, 36, 48, 72 (`FONT_PANEL.sizes`, GTK's list), 72 wide; a field above takes any positive size |

`FontPicker::new(FontChoice)` is a button with the family and size that opens the panel.
`.on_change(|font: FontChoice| ..)` reports each change. Without `on_change` the button does
nothing. It shows the `FontChoice` it gets, so store the change.

```rust
FontPicker::new(font.read().clone()).on_change(move |picked| font.set(picked))
```

| Helper | Returns |
|---|---|
| `faces(&[FontStyle])` | The `(bold, italic)` pairs to list; Regular always |
| `parse_size(typed)` | `Some(size)` for a finite number above 0 |
| `remember(&mut recent, family)` | Moves the family to the front without duplicates |

## Spelling

`TextField` and `TextEditor` check spelling with
[spellbook](https://github.com/helix-editor/spellbook), which reads Hunspell dictionaries
(`.aff` and `.dic`).

1. The kit picks the dictionary from the app languages (`i18n::languages`,
   [17](17-languages.md)): first the exact tag (`en_US`), then the first dictionary of the same
   language in sorted order (`en_AU` before `en_GB`).
2. It searches like Hunspell: `DICPATH`, then `hunspell/` and `myspell/` under
   `XDG_DATA_HOME` and `XDG_DATA_DIRS`.
3. The first field loads it on a separate thread. Without a dictionary nothing is marked.
4. Misspelled words get a row of red dots (`SPELLING.dot`, 2). Words with digits are skipped.
5. A secondary click on one puts its guesses at the top of the edit menu ("No Guesses Found"
   when there are none), then Ignore Spelling and Learn Spelling.
6. Ignore accepts the word until the app quits. Learn also appends it to Enchant's personal
   list, `$XDG_CONFIG_HOME/enchant/<dictionary>.dic`, shared with GTK and GNOME apps
   ([enchant(5)](https://man.archlinux.org/man/enchant.5.en)).

| Function | Returns |
|---|---|
| `words(text)` | Character ranges of the words; an apostrophe between letters joins them |
| `misspelled(text, known)` | The word ranges for which `known(word)` is false |
| `word_at(text, index)` | The word range that contains or touches `index` |
| `replace_chars(text, range, with)` | The text with that character range replaced |
| `dictionary_name(names, tag)` | The dictionary name for a language tag, as in step 1 |
| `personal_words(list)` | The words of a personal list, without blanks or `#` lines |

## Token field

`token_field::TokenField::new(State<Vec<String>>, placeholder)` keeps tags as capsules before a
text input, like `NSTokenField`. `.width(Size)` sets its width (default fill).

| Input | Effect |
|---|---|
| Enter or `,` | Turns the typed text into a token |
| Backspace on empty input | Selects the last token; a second press removes it |
| ← → on empty input | Walk the tokens |
| Click on a token | Selects it |
| Backspace or Delete with a token selected | Removes it |

Tokens are 18 tall capsules with the accent at 20 %; a selected one takes the full accent.

| Item | What it is |
|---|---|
| `commit_token(&mut tokens, draft)` | Trims spaces and a trailing comma and pushes the token; `false` if empty |
| `TokenKey` | `Backspace`, `Delete`, `Left`, `Right` |
| `token_selection(chosen, count, key)` | `(next selection, removed index)` for a key |

## Printing

Printing goes through the `Print` portal. The kit lays out plain text in `BODY`; styles are not
printed.

1. `portal::print(title, |setup| pdf).await` calls `PreparePrint`, which opens the system print
   dialog. If the user cancels, it stops.
2. The closure gets a `PageSetup` and returns the document as PDF bytes.
3. The kit writes the PDF to a `0600` file in `$XDG_RUNTIME_DIR` and calls `Print` with it,
   reusing the dialog's token so the dialog does not show again.

`print::PageSetup` is in millimetres, with the orientation already applied:

| Field | Default |
|---|---|
| `width`, `height` | 210 × 297 (A4) |
| `margins` | `[top, right, bottom, left]`, all 0 |

`size_in_pixels()` returns the page size at `PRINT.dpi` (300).

| Function | Returns |
|---|---|
| `text_pdf(text, &setup)` | A PDF 1.7 with real, selectable and searchable text |
| `text_pages(text, &setup)` | The same pages as images, for a preview |
| `page_starts(lines, page_height)` | The top of each page, never splitting a line |
| `page_of(starts, top)` | The page that holds a line starting at `top` |

`text_pdf` and `text_pages` use margins of at least 72 points on the sides and 90 at the top
and bottom (`PRINT.margin_x`, `PRINT.margin_y`, after `NSPrintInfo`). Freya's Skia has no PDF
backend, so `text_pdf` walks the paragraph glyphs and writes them with
[pdf-writer](https://docs.rs/pdf-writer/0.15.0). Each font is embedded with only the glyphs in
use ([subsetter](https://docs.rs/subsetter/0.2.6)), with a `ToUnicode` map for copying. Glyphs
without outlines, such as color emoji, are painted as images at 300 dpi.

`PrintPreview::new(setup, &pages)` shows the images of `text_pages` in a scrolling column, each
with "Page 1 of 2" below. Put it in a sheet before printing ([8](08-windows.md)).

```rust
let setup = PageSetup::default();
let pages = text_pages(&notes.read(), &setup);
PrintPreview::new(setup, &pages)
```

## Rule editor

`rule_editor::RuleEditor` edits a filter as rows of field, operator and value, like
`NSPredicateEditor`.

```rust
const FIELDS: [RuleField; 2] = [
    RuleField { label: "Name", kind: RuleKind::Text },
    RuleField { label: "Kind", kind: RuleKind::Choice(&["File", "Folder"]) },
];
let predicate = use_state(|| Predicate { all: true, rules: vec![] });
RuleEditor::new("Filter", FIELDS.to_vec(), predicate)
```

`RuleEditor::new(label, fields, State<Predicate>)` writes every change into the state.

| Part | Behavior |
|---|---|
| Header | "Match All / Any of the following are true" |
| Rule row | Field and operator `PopUpButton`s, then a `TextField` or, for `Choice`, a `PopUpButton`; − and + at the end |
| + | Adds a rule after this row; with Alt held it adds a compound rule (a group with its own All/Any) |
| − | Removes the row; an empty group goes too. The last top-level rule cannot be removed |

| Type | What it is |
|---|---|
| `RuleField { label, kind }` | A field the user can filter on |
| `RuleKind` | `Text`, `Number` or `Choice(&[..])` |
| `Operator` | `Contains`, `BeginsWith`, `EndsWith`, `Is`, `IsNot`, `Greater`, `Less` |
| `operators(kind)` | Text: the first five; Number: `Is`, `IsNot`, `Greater`, `Less`; Choice: `Is`, `IsNot` |
| `Rule { id, field, operator, value }` | One condition. `Rule::new(id, fields, field)` uses the first operator and choice; `matches(kind, actual)` compares numbers as numbers and text without case |
| `Criterion` | `Rule(Rule)` or `Group { id, predicate }` |
| `Predicate { all, rules }` | All or any of its criteria. `evaluate(fields, value_of)` checks an item; an empty "any" matches. Also `rule(id)`, `remove(id)`, `insert_after(id, fields, compound)` |

```rust
let shown = predicate.read().evaluate(&FIELDS, &|field| match field {
    0 => file.name.clone(),
    _ => file.kind.clone(),
});
```

Next: [7. Menus and popovers](07-menus-and-popovers.md).
