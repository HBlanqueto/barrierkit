# Shell documentation

**Goal:** find the page that answers a question about the desktop's own
crates, `bk-shell` and `ctshell`.
**Scope:** the two crates of this repository that only Malaysia's desktop
uses. Apps never depend on them; the app kit is in
[the BarrierKit documentation](../barrierkit/README.md).
**Reader:** whoever writes Malaysia's shell, compositor, portal or System
Settings against this repository.
**Input:** a question. **Output:** the page, or the glossary row, that
answers it.

## Steps

1. Pick the crate in the table below.
2. Read its chapter; each starts with the steps of its task.
3. Look a name up in the [shell glossary](glossary.md).

## The crates

| Crate | What it is | Who depends on it | Chapter |
|---|---|---|---|
| `bk-shell` (`bk_shell`) | The desktop shell's components on the kit: layer surfaces, the lock screen, the windows of every app, activation tokens and the status item host. Linux only | Malaysia's shell | [bk-shell](bk-shell.md) |
| `ctshell` | The desktop's settings and tokens, without Freya: the wallpaper, menu bar, Dock and notification settings, the glass of the desktop surfaces and `ctshell::design` | Malaysia's compositor, portal, System Settings and shell | [ctshell](ctshell.md) |

`bk-shell` builds on `bk`, `ctappearance`, `ctdesign` and `infinity-protocol`;
`ctshell` on `ctappearance` and `ctdesign`
([1. What BarrierKit is](../barrierkit/01-what-is-barrierkit.md#the-crates)).
`scripts/check-client-only-deps.sh` fails if `ctshell` pulls Freya or Skia.
