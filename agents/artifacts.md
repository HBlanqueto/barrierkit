# Artifacts: prompt for AI agents

**Goal:** let an AI agent (for example Claude, in a claude.ai artifact) draw
an HTML page that looks like a BarrierKit app.
**Scope:** from a description of a window to one HTML file built only from
BarrierKit tokens. Writing the real app in Rust is in [`prompt.md`](prompt.md);
measuring the page against the kit is in [`compare.md`](compare.md).
**Reader:** an AI agent. Read it whole before drawing.
**Input:** a task, a SwiftUI sketch or an existing app. **Output:** one HTML
file whose controls carry `data-bk` and whose values all come from
[`tengen.css`](tengen.css).

---

You are drawing a BarrierKit window in HTML. The page is a mock-up of a real
app, so every color, size and font comes from the kit, and every control has
the kit's name.

## Steps

1. Read [`prompt.md`](prompt.md) and the
   [glossary](../docs/barrierkit/20-glossary.md): the page uses the same views
   and names as the Rust app.
2. Put [`tengen.css`](tengen.css) in the page: a `<link>` inside the repo, or
   its whole text in a `<style>` in an artifact, which cannot load local files.
   Never edit its values; it is generated from `crates/ctdesign/tengen`.
3. Load Geist from Google Fonts, the family that `--bk-font-families-sans`
   names first.
4. Build each control as one element with the class `bk-<name>` and the
   attribute `data-bk="<View> <label>"`, for example
   `<button class="bk-button" data-bk="Button Guardar">`. The attribute pairs
   it with the kit when you measure.
5. Take every value from a variable. Numbers in `tengen.css` have no unit:
   write `calc(var(--bk-button-height) * 1px)`.
6. Paint with the roles below, never with a color of your own.
7. To check the page against the kit, follow [`compare.md`](compare.md).

## Roles

A control's style names a `Role`; its color is one variable. Source:
`Palette::system` in `crates/bk/src/style.rs`.

| `Role` | Variable |
|---|---|
| `Label` | `--bk-ink-strong` |
| `SecondaryLabel` | `--bk-ink-muted` |
| `TertiaryLabel` | `--bk-ink-faint` |
| `Separator` | `--bk-line` |
| `ControlBorder` | `--bk-line-strong` |
| `WindowBackground` | `--bk-bg-window` |
| `ContentBackground` | `--bk-bg-surface` |
| `ControlBackground` | `--bk-bg-control` |
| `ControlHover` | `--bk-bg-hover` |
| `Accent` (`ShapeStyle::Accent`) | `--bk-accent` |
| `AccentInactive` | `--bk-accent-inactive` |
| `OnAccent` | `--bk-on-accent` |
| `Destructive` | `--bk-danger` |
| `RaisedBackground` | `--bk-bg-raised` |
| `SunkenBackground` | `--bk-bg-sunken` |
| `Scrim` | `--bk-scrim` |
| `SelectedControl` | `--bk-bg-selected-control` |
| `CardBorder` | `--bk-card-border` |

With *Increase Contrast*, a custom accent or a tinted window, the kit derives
some of these colors at run time; the variables hold the untinted values.

## Variables

| Prefix | Comes from | Example |
|---|---|---|
| `--bk-<color>` | `tokens.json` › `themes`; light by default, dark under `prefers-color-scheme: dark` or `data-theme="dark"` | `--bk-accent` |
| `--bk-radius-*`, `--bk-space-*`, `--bk-size-*`, `--bk-duration-*` | `tokens.json` | `--bk-space-3` |
| `--bk-font-styles-<style>-{size,line-height,weight}` | `tokens.json` › `font.styles` (`body`, `headline`, `title-1`…) | `--bk-font-styles-body-size` |
| `--bk-<component>-*` | `components.json` › `components` | `--bk-button-padding-x` |

## Example

[`twins/button.html`](twins/button.html) draws the `button` scene of
`crates/bk/tests/kit/controls.rs`: `Button`, a `bordered_prominent` button
and a disabled one.

## Rules

1. One file. Only `tengen.css` and Google Fonts come from outside.
2. No literal colors, sizes or fonts. The one exception is the white page of a
   twin of a golden image ([`compare.md`](compare.md#steps)).
3. Every control has `class="bk-<name>"` and `data-bk="<View> <label>"`.
4. The text is the app's text, in the app's language.
5. Effects the browser cannot draw the same way (Skia's superellipse corners,
   the compositor's blur) are approximated, and the page says so.
