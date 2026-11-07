# 10. Patterns

**Goal:** build a complete window with a single piece.
**Reader:** anyone who makes a settings app or needs an "About" window.

## Preferences window: `PreferencesWindow`

It joins `SplitView`, `Sidebar`, search field, history and `Form`. The app only
declares the panes:

```rust
PreferencesWindow::new()
    .restoration(app_id)
    .suggestions(&["general"])
    .account("HB", "Héctor Blanqueto", "Cuenta local")
    .group(
        PaneGroup::new(Some("General")).pane(Pane::new(
            "general",
            "General",
            Symbol::CircleHalf,
            ACCENT_SWATCHES[0].1,
            Form::new().section(
                Section::titled("Etiqueta").row(
                    Row::new("Color")
                        .keywords(&["color", "etiqueta"])
                        .control(ColorWell::new("Color", &ACCENT_SWATCHES)),
                ),
            ),
        )),
    )
```

![PreferencesWindow with an account and two panes](assets/components/preferences-light.png)

| Option | Effect |
|---|---|
| `Pane::new(id, título, símbolo, tinte, form)` | A pane: a sidebar item and its form |
| `PaneGroup::new(Some(título))` | Group with a title in the sidebar; `None` without a title |
| `.account(..)` | Account row at the top of the sidebar |
| `.restoration(app_id)` | Saves and restores the open pane, the recent searches and the width and visibility of the sidebar |
| `.suggestions(&[ids])` | Suggested panes when the search field is empty |
| `.sidebar_resizable(bool)`, `.sidebar_range(Option<(mín, máx)>)`, `.narrow_sidebar(NarrowSidebar)` | Passed on to `SplitView` (chapter [8](08-sidebar.md)) |
| `.sidebar_toggle(false)` | Removes the `SidebarToggle` button from the bar |

### The search field

- A line separates it from the pane list.
- It searches the pane titles and the titles and `keywords` of each row.
- It shows each result with the icon of its pane. ↑, ↓ and Enter walk through
  them.
- Choosing a result opens the pane and highlights the row.
- The pane title appears in the bar, with the back and forward buttons.

System Settings is built with this pattern.

## "About": `about_window`

```rust
about_window(avatar("C"), "Contador", Some("Versión 1.0"), "™ y © 2026 Malaysia")
```

With `None` as the version, the window omits that line.

![About window](assets/components/about-light.png)

Next: [11. App infrastructure](11-app.md).
