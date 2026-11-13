# Decisions

**Goal:** find the decision behind a part of the kit.
**Scope:** the architecture decision records (ADR) of BarrierKit. The ones
inherited from Malaysia keep only what still holds here and link to their full
record.
**Reader:** anyone who changes the kit or asks why it is built this way.
**Input:** a question about a design choice. **Output:** the ADR that answers
it.

## Steps

1. Find the subject in the table below.
2. Read the ADR; its status says whether a later one changes it.
3. A new decision takes the next free number and adds its row here in the same
   commit.

| ADR | Decision | Status |
|---|---|---|
| [0011](0011-design-values-as-rust-constants.md) | Design values are Rust constants | accepted (from Malaysia) |
| [0032](0032-neptunia-as-design-source.md) | A JSON design source | accepted (from Malaysia) |
| [0033](0033-skia-in-clients-and-celestialkit-from-otto-kit.md) | Skia in clients and a kit from otto-kit | superseded by 0034 |
| [0034](0034-celestialkit-on-freya.md) | BarrierKit on Freya | accepted (from Malaysia) |
| [0035](0035-own-app-identification-and-csd.md) | Identifying own apps and CSD | accepted (from Malaysia) |
| [0039](0039-infinity-v1-window-management-for-the-shell.md) | Infinity v1: window management for the shell | accepted (from Malaysia) |
| [0041](0041-island-sidebar.md) | Island sidebar | accepted (from Malaysia) |
| [0046](0046-tengen-visual-language.md) | Tengen: the name of the visual language | accepted (from Malaysia) |
| [0047](0047-barrierkit-in-layers.md) | Layered BarrierKit: a toolkit for building complete apps | accepted |
| [0048](0048-barrierkit-own-repository.md) | BarrierKit in its own repository | accepted |
| [0056](0056-barrierkit-on-vulkan.md) | BarrierKit draws with Vulkan | accepted |
| [0057](0057-glass-and-sidebar-over-backdrop.md) | The full-screen glass covers the whole window; the sidebar on a tinted backdrop | accepted |
| [0060](0060-charts-in-barrierkit.md) | Charts in barrierkit | accepted |
| [0061](0061-swiftui-style-entry-api.md) | A SwiftUI-shaped entry API and a glossary of names | accepted |
| [0062](0062-short-crate-names.md) | Short crate names: `bk`, `ctdesign` and `ctappearance` | accepted |
| [0063](0063-views-as-types-and-swift-names.md) | Views are types, modifiers are methods, Apple names | accepted |
| [0064](0064-styles-and-themes.md) | Styles and themes: every visual property, at three levels | accepted |
| [0065](0065-sidebar-glass.md) | The sidebar is a glass panel | accepted |
