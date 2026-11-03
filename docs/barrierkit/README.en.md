# BarrierKit documentation

BarrierKit is the kit for building the apps and the shell of MalaysiaDreams.
This documentation covers it from the first window to the Wayland runner.

- **Reader:** anyone who writes an app with the kit or works inside the kit.
- **Requirements:** basic Rust and knowing what a Wayland compositor is.
- **Source:** the code on `main` as of 2026-11-03. The images are real
  screenshots of the code shown (see [`assets/README.en.md`](assets/README.en.md)).

Español: [README.md](README.md).

## Walkthrough

Each chapter builds on the previous ones.

| # | Chapter | Answers |
|---|---|---|
| 1 | [What BarrierKit is](01-que-es.en.md) | What is it, which crates does it have, does it use the GPU or the CPU? |
| 2 | [Your first app](02-primera-app.en.md) | How do I open a window with a button? |
| 3 | [Layers](03-capas.en.md) | How is the kit organized and what can I use? |
| 4 | [Appearance and tokens](04-apariencia-y-tokens.en.md) | Where do colors and sizes come from? How do I set my own? |
| 5 | [Controls](05-controles.en.md) | Which interactive elements exist and how do they look? |
| 6 | [Windows and titlebars](06-ventanas-y-titlebars.en.md) | Which titlebar and traffic light types exist? |
| 7 | [Sidebar and SplitView](07-barra-lateral.en.md) | How is it configured, resized and collapsed? |
| 8 | [Views](08-vistas.en.md) | Forms, tables, grids, tabs, alerts, navigation and search |
| 9 | [Patterns](09-patrones.en.md) | Preferences window and "About" |
| 10 | [App infrastructure](10-app.en.md) | Actions, shortcuts, global menu, single instance, preferences, undo |
| 11 | [Animations](11-animaciones.en.md) | What animates, with which curve, and how is motion reduced? |
| 12 | [Tinted backdrops and media](12-fondos-y-medios.en.md) | Backdrops from an image, custom accent, video |
| 13 | [Platform](13-plataforma.en.md) | Runner, Vulkan, Wayland protocols, the shell part |
| 14 | [Test apps](14-apps-de-prueba.en.md) | Tengen App Show, Motion Show, Ondina, Rótulo and Marea |
| 15 | [Frequently asked questions](15-preguntas.en.md) | What do we have and what is missing? |

## Single sources

This documentation links and does not copy. When a fact changes, it changes
there:

- Decisions: [`docs/decisiones/`](../decisiones/), especially
  [ADR 0047](../decisiones/0047-barrierkit-en-capas.md) (layers and status).
- Tokens: `crates/vow-design/tengen/tokens.json` and `components.json`.
- Appearance options: `crates/vow-appearance/src/lib.rs`
  (`AppearanceConfig`).
