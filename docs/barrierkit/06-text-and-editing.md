# 6. Text and editing

**Goal:** add an editable text area, style its text and check its spelling.
**Scope:** multi-line editing, rich text, the font panel, spelling, tag fields and
the rule editor. One-line fields (`TextField`, `SearchField`, `SecureField`, `ComboBox`) are in
[5](05-controls.md#text-fields); menus and popovers in general are in
[7](07-menus-and-popovers.md); the portals in general are in [13](13-app.md).
**Reader:** anyone who builds an editor, a notes app or a filter.
**Input:** a window with state ([2](02-first-app.md)). **Output:** an editor with bold, italic,
fonts, links and spell checking.

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

3. Style the text. Select some and press Ctrl+B, Ctrl+I or Ctrl+U, or Ctrl+T for the
   [font panel](#font-panel). The editor writes the runs into `styles`
   ([Rich text](#rich-text)).
4. Check spelling. Install a Hunspell dictionary for the app language (on NixOS, for example
   `hunspellDicts.en_US`). Misspelled words get red dots; a secondary click on one offers
   guesses ([Spelling](#spelling)).

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
engine (`freya-edit`). It looks like an `NSTextView` in a bordered scroll view: a square
field of the window background (`textBackgroundColor`) with a 1 border inside, the text 13 on 16
lines starting 5.5 from the left and flush with the top (`TEXT_EDITOR`), as SwiftUI's `TextEditor` on macOS 27.2. `NSTextView` and SwiftUI's
`TextEditor` draw 12 pt Helvetica until the app sets a font; the kit always draws the 13 pt
system face.

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
| Ctrl+B, Ctrl+I, Ctrl+U | Toggle bold, italic or underline on the selection (with `styles`) |
| Ctrl+T | Opens the font panel for the selection (with `styles`) |

A focused editor with a border has an accent ring; one with `border: None` draws none, as an `NSTextView` outside a bordered scroll view. When the app replaces `value`, the editor takes the new
text, clears the selection and starts a new undo history.

### Find bar

Ctrl+F opens a bar above the text, like SwiftUI's `findNavigator`: a `SearchField`, the counter
("2 of 3", or "Not Found"), previous and next buttons and Done. ↑, ↓ and Enter in the field
move between matches. The current match is selected, and the text scrolls to it
(`ScrollController::reveal_y`, [5](05-controls.md#scrolling)).

`find_matches(text, query)` returns the matches as `(start, end)` character indices. It ignores
case, does not overlap and returns nothing for an empty query.

### Style

`TextEditor` draws from a `TextEditorStyle`: `.style(|s| ..)` for one editor,
`.control_style(|s: TextEditorStyle| ..)` for the editors inside a view. The
controls of the find bar take their own styles. The fields are in
[Styles and themes](../../crates/bk/src/styles/text_editor.rs).

## Rich text

Module `rich_text`. The text stays in the `State<String>`; the styles live beside it as runs
in character indices.

| Type | Fields |
|---|---|
| `RunStyle` | `bold`, `italic`, `underline`, `strikethrough`, `link: Option<String>`, `family: Option<String>`, `size: Option<f32>`, `foreground_style: Option<ShapeStyle>`; `Default` is plain |
| `StyleRun` | `range: Range<usize>`, `style: RunStyle` |
| `FontChoice` | `family`, `size`, `bold`, `italic` |
| `Emphasis` | `Bold`, `Italic`, `Underline` |

```rust
let styles = use_state(|| vec![StyleRun {
    range: 0..5,
    style: RunStyle { bold: true, ..RunStyle::default() },
}]);
```

How the editor draws and edits runs:

| Behavior | Rule |
|---|---|
| Typing | Runs move with the text. Typed text inherits the style of the previous character, not its link |
| Ctrl+B, Ctrl+I, Ctrl+U | If the whole selection has the emphasis, it is removed; otherwise it is added |
| Links | Drawn in the accent and underlined. The pointer becomes a hand; a click opens the link through the `OpenURI` portal instead of moving the caret |
| Fonts | `family` and `size` replace `BODY` for their run |
| Colour | `foreground_style` paints its run, like `AttributedString.foregroundColor`. A `ShapeStyle::Role` follows light and dark; a link keeps the accent |

The functions behind it are pure and work on any runs:

| Function | Returns |
|---|---|
| `char_styles(runs, length)` | One `RunStyle` per character |
| `runs_from(styles)` | Runs that merge equal neighbors and skip plain text |
| `restyle_after_edit(runs, old, new)` | Runs moved after an edit from `old` to `new` |
| `toggle(runs, length, range, Emphasis)` | Runs with the emphasis toggled on `range` |
| `font_at(runs, length, index)` | The `FontChoice` at a character; `BODY` when unset |
| `apply_font(runs, length, range, &FontChoice)` | Runs with that font on `range` |
| `link_at(runs, index)` | The link at a character, if any |
| `markdown(source)` | The plain text and the runs of inline Markdown ([Text runs](#text-runs)) |

## Text runs

`Text` (recipe [42](../../.claude/skills/barrierkit-app/recipes.md#42-text-with-runs-markdown-and-selection)) takes
SwiftUI's inline forms. Each is a `Text` again, so it keeps the layout and style modifiers.

| Form | Result |
|---|---|
| `Text::new("a") + Text::new("b").bold(true)` | One paragraph of two runs; each keeps what its author set (`bold`, `italic`, `underline`, `strikethrough`, `font`, `foreground_style`). What neither set, such as a `foreground_style` on the sum, reaches both |
| ``Text::markdown("**b** *i* ~~s~~ `c` [t](https://x.org)")`` | Bold, italic, struck, monospaced and a link in the accent; a click on the link opens it through the `OpenURI` portal. A mark without its mate (`2 * 3`), `snake_case` and `\*` stay literal. `Text::new` never reads Markdown, so a file name with an asterisk stays as written |
| `.monospaced_digit()` | Every digit takes the same width (the font's `tnum` feature), so a clock or a counter does not shift as it changes |
| `.text_selection(true)` | The reader selects with the pointer and copies with Command+C or Ctrl+C |

Limits:

1. Markdown is inline only: no headings, lists, images or line breaks of its own.
2. A link of a selectable text does not open, and its selection draws the Freya highlight.
3. A sum takes the line height and the paragraph options (`line_limit`, alignment) of the plain
   `Text::new("")` it starts from; a layout modifier of an operand is dropped, as in SwiftUI.
4. A span has one decoration, so a link or an underline hides a strikethrough on the same run.
5. SwiftUI deprecated `Text + Text` in macOS 26 in favor of string interpolation; Rust has no
   interpolation of views, so the kit keeps `+`.

## Font panel

Module `font_panel`, after `NSFontPanel`. The panel opens in a
[popover](07-menus-and-popovers.md) with a preview and four columns:

| Column | Content |
|---|---|
| Collection | All Fonts, Recently Used (chosen in this session, newest first), Fixed Width |
| Family | The system fonts and those added with `register_font` ([4](04-appearance-and-tokens.md)) |
| Typeface | Regular, plus Italic, Bold and Bold Italic when the family has them |
| Size | 6, 8, 9, 10, 11, 12, 13, 14, 16, 20, 24, 36, 48, 72 (`FONT_PANEL.sizes`, GTK's list), 72 wide; a field above takes any positive size |

`FontPicker::new(State<FontChoice>)` is a button with the family and size that opens the panel.
Each change in the panel writes the `State`.

```rust
FontPicker::new(font)
```

| Helper | Returns |
|---|---|
| `faces(&[FontStyle])` | The `(bold, italic)` pairs to list; Regular always |
| `parse_size(typed)` | `Some(size)` for a finite number above 0 |
| `remember(&mut recent, family)` | Moves the family to the front without duplicates |

### Style

`FontPicker` is a `Button`: its `.style(|s| ..)` takes a
[`ButtonStyle`](../../crates/bk/src/styles/button.rs). The panel has no style.

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
   ([enchant(5)](https://man.archlinux.org/man/enchant.5.en)). The list is read once, with the
   dictionary, and again when its modification time changes, so words other apps learn show up.
   A word learned before the list is open waits and is saved when it opens. A list that cannot
   be read (for example, not UTF-8) is never written to: Learn then works until the app quits.

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

Tokens are 15 tall rounded rectangles (radius 2) in a light blue fill (`TOKEN_FILL_*`); a
selected one takes the accent.

| Item | What it is |
|---|---|
| `commit_token(&mut tokens, draft)` | Trims spaces and a trailing comma and pushes the token; `false` if empty |
| `TokenKey` | `Backspace`, `Delete`, `Left`, `Right` |
| `token_selection(chosen, count, key)` | `(next selection, removed index)` for a key |

### Style

`.style(|s| ..)` takes a [`TokenFieldStyle`](../../crates/bk/src/styles/token_field.rs): the field,
the tokens and the selected token. The tokens of a `SearchField` draw from the
`TokenFieldStyle` in context.

## Rule editor

`rule_editor::RuleEditor` edits a filter as rows of field, operator and value, like
`NSPredicateEditor`.

```rust
let fields = vec![
    RuleField { label: "Name".into(), kind: RuleKind::Text },
    RuleField { label: "Kind".into(), kind: RuleKind::Choice(vec!["File".into(), "Folder".into()]) },
];
let predicate = use_state(|| Predicate { all: true, rules: vec![] });
RuleEditor::new("Filter", fields, predicate)
```

`RuleKind::Choice` owns its choices as `Cow<'static, str>`, so a runtime list needs no leak.
`RuleKind` is `Clone`, not `Copy`: `operators` and `Rule::matches` take `&RuleKind`.

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
| `RuleKind` | `Text`, `Number` or `Choice(Vec<Cow<'static, str>>)` |
| `Operator` | `Contains`, `BeginsWith`, `EndsWith`, `Is`, `IsNot`, `Greater`, `Less` |
| `operators(&kind)` | Text: the first five; Number: `Is`, `IsNot`, `Greater`, `Less`; Choice: `Is`, `IsNot` |
| `Rule { id, field, operator, value }` | One condition. `Rule::new(id, fields, field)` uses the first operator and choice; `matches(&kind, actual)` compares numbers as numbers and text without case |
| `Criterion` | `Rule(Rule)` or `Group { id, predicate }` |
| `Predicate { all, rules }` | All or any of its criteria. `evaluate(fields, value_of)` checks an item; an empty "any" matches. Also `rule(id)`, `remove(id)`, `insert_after(id, fields, compound)` |

```rust
let shown = predicate.read().evaluate(&FIELDS, &|field| match field {
    0 => file.name.clone(),
    _ => file.kind.clone(),
});
```

### Style

`RuleEditor` draws its gaps, indent and header text from a `RuleEditorStyle`
([fields](../../crates/bk/src/styles/rule_editor.rs)). Its pop-up buttons, text fields
and buttons take their own styles, so `.control_style(|s: PopUpButtonStyle| ..)`
on the editor reaches them.

Next: [7. Menus and popovers](07-menus-and-popovers.md).
