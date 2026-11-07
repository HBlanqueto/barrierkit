# BarrierKit documentation

**Goal:** find the chapter that answers a question about the kit.
**Scope:** BarrierKit from the first window to the Wayland runner. Decisions
are in [`docs/decisions/`](../decisions/); the Tengen visual language is in
[`docs/architecture/tengen.md`](../architecture/tengen.md).
**Reader:** anyone who writes an app with the kit or works inside it. It
assumes basic Rust and knowing what a Wayland compositor is.
**Input:** a question. **Output:** the chapter, or the glossary row, that
answers it.

## Steps

1. If you know the SwiftUI or AppKit name of what you need, look it up in the
   [glossary](20-glossary.md); each row links to its chapter.
2. Otherwise, pick the chapter in the table below.
3. New to the kit: read chapters 1 to 3 in order.
4. An AI agent starts from [`agents/prompt.md`](../../agents/prompt.md).

## Chapters

| # | Chapter | Answers |
|---|---|---|
| 1 | [What BarrierKit is](01-what-is-barrierkit.md) | What is it, which crates does it have, does it use the GPU or the CPU? |
| 2 | [Your first app](02-first-app.md) | How do I open a window with text and state? |
| 3 | [Layers](03-layers.md) | How is the kit organized and what can I use? |
| 4 | [Appearance and tokens](04-appearance-and-tokens.md) | Where do colors, fonts, symbols and sizes come from? |
| 5 | [Controls](05-controls.md) | Which controls exist and how do I wire their value? |
| 6 | [Text and editing](06-text-and-editing.md) | How do I edit, style, spell-check and print text? |
| 7 | [Menus and popovers](07-menus-and-popovers.md) | How do I show a context menu, a popover or a tooltip? |
| 8 | [Windows](08-windows.md) | How do I set a window's size and state, and open popups? |
| 9 | [Title bars](09-titlebars.md) | Which title bars and traffic lights exist and how do they look? |
| 10 | [Sidebar and split view](10-sidebar.md) | How is a sidebar built, resized and collapsed? |
| 11 | [Views](11-views.md) | Forms, tables, trees, grids, tabs, alerts, navigation and search |
| 12 | [Patterns](12-patterns.md) | Preferences window, About window, language picker |
| 13 | [App services](13-app.md) | Actions, shortcuts, menu bar menu, status item, settings, undo, single instance |
| 14 | [Animations](14-animations.md) | What animates, with which curve, and how is motion reduced? |
| 15 | [Backdrops and media](15-backdrops-and-media.md) | Tinted backdrops, glass groups, artwork and video |
| 16 | [Charts](16-charts.md) | How do I draw series, shares, heat maps and 3D charts? |
| 17 | [Languages](17-languages.md) | How do I translate an app and how is the language chosen? |
| 18 | [Platform](18-platform.md) | Runner, Vulkan, Wayland protocols, the shell feature |
| 19 | [Profiling](19-profiling.md) | Where does each frame's time go? |
| 20 | [Glossary](20-glossary.md) | What is the BarrierKit name of a SwiftUI or AppKit concept? |

## Other questions

| Question | Where |
|---|---|
| What is missing? | The «Status» of [ADR 0047](../decisions/0047-barrierkit-in-layers.md#status-2026-11-02) |
| Does it work outside Malaysia? | [18](18-platform.md#wayland-protocols): every protocol but `wl_compositor` and `xdg_wm_base` is optional |
| Can I use Freya directly? | [3, Rules](03-layers.md#rules) and [2, The prelude](02-first-app.md#the-prelude) |
| How do I change how a control looks? | [`agents/styles.md`](../../agents/styles.md) |
| Where does an app store its data? | [13](13-app.md) |
| How do I add a component to the kit? | [3, What every component does](03-layers.md#what-every-component-does) and [`AGENTS.md`](../../AGENTS.md) |
| How do I check a change? | `./dev.sh check` inside `nix develop`; its steps are `check()` in [`dev.sh`](../../dev.sh) |

## Single sources

This documentation links and does not copy. When a fact changes, it changes
in its source:

- Decisions: [ADR 0047](../decisions/0047-barrierkit-in-layers.md) (layers and
  status) and [ADR 0061](../decisions/0061-swiftui-style-entry-api.md) (entry
  API and names).
- Names and their SwiftUI and AppKit equivalents: the
  [glossary](20-glossary.md).
- Token values: `crates/ctdesign/tengen/tokens.json` and `components.json`.
- Appearance options: `crates/ctappearance/src/config.rs`
  (`AppearanceConfig`).
- Screenshots: [`assets/README.md`](assets/README.md).
