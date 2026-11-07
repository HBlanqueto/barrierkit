# 0061 — A SwiftUI-shaped entry API and a glossary of names

Status: **accepted** (2026-11-07). Implemented in `barrier_kit::prelude`,
`barrier_kit::app::{Application, Scene}` and
`barrier_kit::elements::{Text, VStack, HStack}`.

## Context

A window with one line of text needed 8 imports from 4 crates and about 35
lines: the app built its own background, `Titlebar` and centered box, and
`text()` asked for font and color every time. SwiftUI does the same in 12
lines. The audit of 2026-11-07 also found 34 places that rebuilt Freya's
`expanded()` by hand, and font tokens (`TEXT`, `DISPLAY`, `TITLE_1`) whose names
did not match the SwiftUI styles they copy.

Apple's guidance for this is progressive disclosure: the common case needs no
configuration, and the call grows only with the use case
([WWDC22, *The craft of SwiftUI API design*](https://developer.apple.com/videos/play/wwdc2022/10059/)).

## Decision

1. **One import:** `barrier_kit::prelude::*` re-exports the kit's entry points,
   elements and controls, Freya's prelude, Torin's `Alignment`, `Direction`
   and `Size`, and the type and spacing tokens. An app depends only on
   `barrier-kit`.
2. **`Application::new(name).window(title, content).run()`:** `Scene::run`
   draws the window background and a `Titlebar` with the title, centers the
   content and calls `run_window`. `Scene::options` reaches every
   `WindowOptions` setting.
3. **`run_window` takes `impl Into<AppComponent>`**, so a closure works as the
   root, as Freya already allows.
4. **`Text`, `VStack`, `HStack`:** thin types over Freya's `label()` and
   `rect()` with the defaults of SwiftUI: `Text` uses `BODY` and the palette's
   label color and wraps; the stacks center their children with `SPACE_3`
   between them. They forward Freya's extension traits, so every layout and
   event method still works.
5. **SwiftUI names:** the font tokens become `LARGE_TITLE`, `TITLE`, `TITLE_2`,
   `HEADLINE`, `BODY`, `CALLOUT`, `CAPTION`, `MONO`. New names follow the rules
   of the [glossary](../barrierkit/15-glossary.md): SwiftUI, then AppKit, then
   plain English. The kit's app is `Application` because Freya owns `App`.
6. **A guide for AI agents:** [`docs/ai/prompt.md`](../ai/prompt.md) is a
   single entry prompt that links the glossary, recipes that compile, and a
   checklist.

## Consequences

- An app shows text in a window with one import and one expression.
- `run_window` and `rect()` stay for windows that draw their own titlebar.
- malaysia and curseterm rename `TEXT`, `DISPLAY` and `TITLE_1` when they move
  their pinned `rev` past this change.
- Every public item that is added or renamed goes into the glossary in the same
  commit (`AGENTS.md`).
