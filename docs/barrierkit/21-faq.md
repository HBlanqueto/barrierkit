# 21. Frequently asked questions

**Goal:** answer the common questions in one or two sentences.
**Scope:** short answers only; each one links to the chapter that explains it.
**Reader:** anyone.
**Input:** a question. **Output:** the answer and where to read more.

## Steps

1. Find the question below.
2. Follow its link for the details.
3. If it is not here, look the name up in the [glossary](20-glossary.md).

## What is in the kit?

Every public item, with its SwiftUI and AppKit equivalent, is in the
[glossary](20-glossary.md). The layers are in [3](03-layers.md).

## What is missing?

The pending work is in the "Status" section of
[ADR 0047](../decisions/0047-barrierkit-in-layers.md#status-2026-11-02). In
short:

- **Blocked by Freya 0.4.3:** partial damage; every frame repaints the whole
  tree.
- **Waiting for the first app that asks for it:** a native `Image`, search and
  overflow in the title bar, a policy for when the last window closes, and
  restoring the navigation stack and the scroll.
- **Depends on the compositor:** restoring the window position.

## Does it use the GPU or the CPU?

Drawing goes through Vulkan, normally on the GPU; layout runs on the CPU. A
software Vulkan driver (lavapipe) also works. See [1](01-what-is-barrierkit.md)
and [18](18-platform.md).

## Does it work outside Malaysia?

Yes, as a normal Wayland client. Without Malaysia there are no
compositor-drawn borders, shadow or blur, and the global menu only exists on
D-Bus. See [18](18-platform.md).

## Can I use Freya directly?

Yes. The prelude brings Freya's `rect()`, `label()` and hooks
([2](02-first-app.md#the-prelude)). Paint with the `Palette` and the tokens
([4](04-appearance-and-tokens.md)). Don't use the `freya-components` widgets
with their default style: the kit has its own, with the Tengen look.

## How do I set my own colors?

Use the variants the kit offers (`Tint`, `Accent`, `ColorWell`). A new value
for every app goes into Tengen's tokens. See [4](04-appearance-and-tokens.md).

## How do I make the sidebar fixed or change its limits?

`SplitView::resizable(false)` and `SplitView::sidebar_range(min, max)`; in
`SettingsWindow`, `sidebar_resizable` and `sidebar_range`. See
[10](10-sidebar.md) and [12](12-patterns.md).

## Where does an app store its data?

Preferences in `~/.config/malaysia/<app_id>.domain` and window state in
`~/.local/share/malaysia/<app_id>.state`. `MALAYSIA_CONFIG_DIR` and
`MALAYSIA_DATA_DIR` override both folders. See [13](13-app.md).

## How do I add a component to the kit?

Follow the rules in [`AGENTS.md`](../../AGENTS.md) and build it from kit
primitives ([5](05-controls.md)). Give it a render test in light and dark and,
if it is interactive, a keyboard test. Add its row to the
[glossary](20-glossary.md) in the same commit.

## How do I check a change?

1. `nix develop`
2. `./dev.sh check`: `cargo fmt --check`, `cargo clippy` with `-D warnings`
   (library alone, then the workspace with the `shell` and `tracy` features),
   the token and client-dependency scripts, and the tests.
3. If a visual change is intended, regenerate the reference images with
   `UPDATE_GOLDEN=1 cargo test -p bk --test kit` and review them.
