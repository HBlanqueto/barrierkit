# 16. Frequently asked questions

**Goal:** answer the most common questions about the kit quickly.
**Reader:** anyone. Each answer links to the chapter that expands on it.

## What do we have today?

| Layer | Pieces |
|---|---|
| 0 · Platform | Windows and popups, focus and Tab, per-component cursor, fractional scaling, text clipboard, opaque region, AT-SPI, Vulkan |
| 1 · Primitives | `Palette`, `text`, `Symbol`, `use_interaction`, `Button`, `IconButton`, `Link`, `Switch`, `Checkbox`, `RadioGroup`, `SegmentedControl`, `PopUpButton`, `ColorWell`, `ChoiceCards`, `ToggleGroup`, `Stepper`, `Slider`, `LevelSlider`, `TextField`, `SearchField`, `SecureField`, `ProgressBar`, `ProgressRing`, `Spinner`, `badge`, `divider`, `avatar`, `use_tooltip`, `context_menu`, `ScrollView`, `MenuView`, `Titlebar`, `TrafficLights`, `WindowChrome`, `Sidebar`, `Backdrop`, `media`, `video`, `use_breakpoint`, `Spring` |
| 2 · Views | `SplitView`, `Form`/`Section`/`Row`, `Table`, `CollectionView`, `TabView`, `History` + `navigation_buttons`, `SearchIndex`, `empty_state`, `sheet`, `Alert` |
| 3 · Patterns | `PreferencesWindow`, `about_window` |
| App | `Action` + `Shortcut`, `AppMenu`, `launch`, `load_defaults`/`save_defaults`/`use_defaults`, restoration, `use_undo`, `copy`/`paste` |

## What is missing?

The pending work is in [ADR 0047](../decisiones/0047-barrierkit-en-capas.en.md),
«Pendiente» ("Pending"). In short:

- **Blocked by Freya 0.4.3:** partial damage and primary selection.
- **Waiting for the first app that asks for it:** drag and drop between apps,
  a native `Image`, search and overflow in the titlebar, `OutlineView`,
  `ColumnBrowser`, opening files and URLs, typed clipboard.
- **Depends on the compositor:** restoring the window position.

## Does it use the GPU or the CPU?

Drawing goes through Vulkan, normally on the GPU; layout runs on the CPU. A
software Vulkan driver (lavapipe) also works. Video uses VA-API if available
and, if not, the CPU. See [1](01-que-es.en.md#does-it-use-the-gpu-or-the-cpu)
and [13](13-fondos-y-medios.en.md#video-video-feature).

## Does it work outside MalaysiaDreams?

Yes, as a normal Wayland client, if the compositor draws the first frame (see
[14](14-plataforma.en.md#known-limitation-the-first-frame)). Outside
MalaysiaDreams there are no compositor borders, shadow or blur, and the global
menu only exists on D-Bus.

## Can I use Freya directly?

Yes, `rect`, `label` and the `freya-core` hooks are available. Paint with the
`Palette` and the tokens. Do not use the `freya-components` components with
their default style: the kit already has its own version with the Tengen look.

## How do I set my own colors?

With variants: `Tint`, `Accent` and `ColorWell`. A new value for everyone goes
into Tengen. See [4](04-apariencia-y-tokens.en.md#custom-colors-in-an-app).

## How do I make the sidebar non-resizable or give it other limits?

`SplitView::resizable(false)` and `SplitView::sidebar_range(mín, máx)`; in
`PreferencesWindow`, `sidebar_resizable` and `sidebar_range`. See
[8](08-barra-lateral.en.md).

## Where does an app store its data?

Preferences in `~/.config/malaysia/<app_id>.domain`; window state in
`~/.local/share/malaysia/<app_id>.state`. See
[11](11-app.en.md#preferences-and-restoration).

## How do I add a component to the kit?

Follow [5, "Creating a new control"](05-controles.en.md#creating-a-new-control).
It comes with its page in Tengen App Show, its render test in light and dark
and, if it is interactive, its keyboard test.

## How do I check a change?

```sh
nix develop
./dev.sh check
./dev.sh app tengen-app-show
```

`./dev.sh check` runs `fmt`, `clippy`, the token and dependency scripts and
the tests. If a visual change is intentional, regenerate the reference images
with `UPDATE_GOLDEN=1 cargo test -p barrier-kit --test kit`.
