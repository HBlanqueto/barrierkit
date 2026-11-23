# Rich text: what is left

**Scope:** `TextEditor.styles` and `rich_text` against SwiftUI `AttributedString` with
`TextEditor` and AppKit `NSTextView` with `NSAttributedString`. **Reader:** whoever extends
`RunStyle`.

## Today

Per-character styles exist. The text stays in a `State<String>`; `State<Vec<StyleRun>>` holds
runs of `RunStyle { bold, italic, underline, strikethrough, link, family, size }` over character ranges.
`styled_spans` draws one Freya `Span` per run. Runs follow edits (`restyle_after_edit`), Ctrl+B,
Ctrl+I and Ctrl+U toggle emphasis, Ctrl+T sets the font, and a link opens through the portal.
`rich_text::markdown` reads inline Markdown into the same runs and `styled_spans` draws them, for `Text::markdown`
and `Text + Text` too (ADR 0111).

## Gap

| Attribute (AppKit, SwiftUI) | Status | Blocker |
|---|---|---|
| `foregroundColor`, `backgroundColor` | absent | Colour picker wiring (M). Span `color` exists; a span background does not in Freya 0.5.3 |
| `strikethroughStyle` | partial | `RunStyle::strikethrough` is drawn and comes from Markdown `~~x~~`; no shortcut in the editor. A Freya `Span` has one `TextDecoration`, so it does not combine with underline or a link |
| underline style, colour, baseline, kerning, shadow | absent | Freya wires no decoration style or colour, baseline shift or word spacing |
| `NSParagraphStyle` (alignment, indent, spacing, tabs), lists, tables | absent | Freya wires no `ParagraphStyle` options |
| Import and export (RTF, HTML) | absent | Format decision needed; no RTF writer in the tree |

## Plan

1. Done (ADR 0111): `strikethrough: bool` beside `underline`, drawn when no link or underline shares the
   run. Left: an editor shortcut. Size S.
2. Add `color: Option<Rgba>` and a `ColorPicker` shortcut in the editor. Size M.
3. Wait for Freya paragraph options before paragraph styles and lists. Size L.
