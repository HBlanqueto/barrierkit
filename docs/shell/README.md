# Shell kit documentation

**Goal:** find the chapter that answers a question about the desktop shell's
crates.
**Scope:** `bk-shell`, `ctshell` and `bk`'s `shell` feature, from the first
panel to running under a compositor. Apps never depend on them; the app kit is
in [the BarrierKit documentation](../barrierkit/README.md), and the decisions
are in [`docs/decisions.md`](../decisions.md).
**Reader:** whoever writes Malaysia's shell, compositor, portal or System
Settings against this repository. It assumes the app guide's chapters 1 and 2.
**Input:** a question. **Output:** the chapter, or the glossary row, that
answers it.

## Steps

1. If you know the name of an item, or its SwiftUI or AppKit counterpart, look
   it up in the [shell glossary](glossary.md); each row links to its chapter.
2. Otherwise, pick the chapter in the table below.
3. New to the shell kit: read chapters 1 and 2 in order, then the chapter of
   the part you build.
4. Before you push, check the change as [chapter 10](10-run-and-debug.md)
   says.

## Chapters

| # | Chapter | Answers |
|---|---|---|
| 1 | [What the shell kit is](01-what-is-the-shell-kit.md) | Which crates are there, who depends on what, and how is a shell component not an app? |
| 2 | [Your first component](02-first-component.md) | How do I put a panel with a text along the top of the screen? |
| 3 | [Layer surfaces](03-layer-surfaces.md) | How do I place, size and resize a layer surface, and where does it take the pointer? |
| 4 | [Desktop windows](04-desktop-windows.md) | How do I list, activate, minimize, capture and tile other apps' windows, and read their menus? |
| 5 | [Lock screen](05-lock-screen.md) | How do I lock the session and unlock it? |
| 6 | [Status items](06-status-items.md) | How do I host other apps' tray icons and menus? |
| 7 | [Glass and blur](07-glass-and-blur.md) | How does a shell surface get its blur, its glass and its theme? |
| 8 | [Desktop settings](08-desktop-settings.md) | Where are the Dock, menu bar, notification and wallpaper settings, and how do I read and write them? |
| 9 | [Shell tokens](09-shell-tokens.md) | Which sizes, colors and timings does `ctshell::design` hold? |
| 10 | [Run and debug](10-run-and-debug.md) | How do I check a change, run a component nested and read its log, and what does not run on macOS? |
| — | [Glossary](glossary.md) | What is the name of a shell item, and which chapter documents it? |

## Other questions

| Question | Where |
|---|---|
| How does `bk-shell` hook into the kit's runner? | [1, How it joins the runner](01-what-is-the-shell-kit.md#how-it-joins-the-runner) |
| Which globals must the compositor offer? | [10, Globals](10-run-and-debug.md#globals): only `zwlr_layer_shell_v1` or `ext_session_lock_manager_v1` |
| Does it build on macOS? | [10](10-run-and-debug.md#what-cannot-run-on-macos): `ctshell` does, `bk-shell` does not |
| How do I write the view itself? | As in an app: [the BarrierKit documentation](../barrierkit/README.md) |

## Single sources

- Names and their SwiftUI and AppKit equivalents: the
  [shell glossary](glossary.md).
- Token values: `crates/ctdesign/tengen/*.json`, turned into
  `ctshell::design` by `crates/ctshell/build.rs`.
- The program of chapter 2: `crates/bk-shell/examples/hello_panel.rs`.
