# BarrierKit documentation

BarrierKit is the kit for building the apps and the shell of Malaysia.
This documentation covers it from the first window to the Wayland runner.

- **Reader:** anyone who writes an app with the kit or works inside the kit.
- **Requirements:** basic Rust and knowing what a Wayland compositor is.
- **Source:** the code on `main` as of 2026-11-07. The images are real
  screenshots of the code shown (see [`assets/README.md`](assets/README.md)).

## Walkthrough

Each chapter builds on the previous ones.

| # | Chapter | Answers |
|---|---|---|
| 1 | [What BarrierKit is](01-what-is-barrierkit.md) | What is it, which crates does it have, does it use the GPU or the CPU? |
| 2 | [Your first app](02-first-app.md) | How do I open a window with text, state and a button? |
| 3 | [Layers](03-layers.md) | How is the kit organized and what can I use? |
| 4 | [Appearance and tokens](04-appearance-and-tokens.md) | Where do colors and sizes come from? How do I set my own? |
| 5 | [Controls](05-controls.md) | Which interactive elements exist and how do they look? |
| 6 | [Windows](06-windows.md) | How do I open a window, resize it and read its state? |
| 7 | [Titlebars](07-titlebars.md) | Which titlebars and traffic lights exist and how do they look in each state? |
| 8 | [Sidebar and SplitView](08-sidebar.md) | How is it configured, resized and collapsed? |
| 9 | [Views](09-views.md) | Forms, tables, grids, tabs, alerts, navigation and search |
| 10 | [Patterns](10-patterns.md) | Preferences window and "About" |
| 11 | [App infrastructure](11-app.md) | Actions, shortcuts, global menu, single instance, preferences, undo |
| 12 | [Animations](12-animations.md) | What animates, with which curve, and how is motion reduced? |
| 13 | [Tinted backdrops and media](13-backdrops-and-media.md) | Backdrops from an image, custom accent, video |
| 14 | [Platform](14-platform.md) | Runner, Vulkan, Wayland protocols, the shell part |
| 15 | [Glossary](15-glossary.md) | What is the BarrierKit name for a SwiftUI or AppKit concept? How are names chosen? |
| 16 | [Frequently asked questions](16-faq.md) | What do we have and what is missing? |
| 17 | [Profiling with Tracy](17-profiling.md) | Where does each frame's time go? |
| 18 | [Languages](18-languages.md) | How do I translate an app and how is the language chosen? |
| 19 | [Charts](19-charts.md) | How do I draw series, shares and 3D charts? |

## For AI agents

[`docs/ai/prompt.md`](../ai/prompt.md) is the single prompt for an agent that
writes apps with the kit. It links the glossary, recipes that compile and a
checklist.

## Single sources

This documentation links and does not copy. When a fact changes, it changes
there:

- Decisions: [`docs/decisions/`](../decisions/), especially
  [ADR 0047](../decisions/0047-barrierkit-in-layers.md) (layers and status)
  and [ADR 0061](../decisions/0061-swiftui-style-entry-api.md) (entry
  API and names).
- Tokens: `crates/vow-design/tengen/tokens.json` and `components.json`.
- Appearance options: `crates/vow-appearance/src/config.rs`
  (`AppearanceConfig`).
