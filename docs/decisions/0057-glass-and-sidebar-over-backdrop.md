# 0057 — The full-screen glass covers the whole window; the sidebar on a tinted backdrop

Status: **accepted** (2026-11-03). Rule 1 applies to windows with
`FullscreenBar::Glass` or `ProgressiveGlass` (Ondina); by default the bar is `Solid`
(chapter 7 of [`docs/barrierkit`](../barrierkit/09-titlebars.md#the-bar-in-full-screen-fullscreenbar)).

## Context

Ondina switched to the kit's `SplitView`, `Sidebar` and `Titlebar` (`6689f78`). With that, two bugs appeared:

- In full screen with the overlaid bar (ADR 0054), the `Titlebar` drew the glass with the width of its column. The sidebar and the inspector stayed outside: Ondina's queue looked sharp under the buttons while the center blurred.
- The `Sidebar` always painted the system sidebar background. On a `Backdrop`, an opaque gray island appeared on top of the gradient, even though the `Titlebar` already turned transparent with `backdrop::tinted()`.

## Decision

### 1. Full-screen glass rule

**In full screen with the overlaid titlebar, the progressive glass covers the whole top strip of the window, from edge to edge, no matter what is underneath:** island or flat sidebar, tinted or not, detail, inspector or any other composition of the app. No column limits or clips it, and it looks the same as in the accepted mockup (Main and tinted Island):

- strong blur at the top edge that fades out downwards (`FULLSCREEN_GLASS`);
- above the glass there are only the titlebar, the traffic lights and the overlaid layers (menus, popups, vertical rails);
- all the content that passes underneath, in any column, blurs the same way.

Implementation: the `Titlebar` places the glass with `Position::new_global()` and `Size::window_percent(100.0)`, in `TITLEBAR_OVERLAY_LAYER` and before the bar. The bar keeps the width of its column; the glass does not. It stays inside the app tree to read the `Backdrop` tint (`glass_tint`).

The glass never goes inside an opacity layer: Skia's blur (`backdrop`) reads the layer it is drawn in, and inside an opacity layer that layer is empty, so it blurs nothing. The glass itself applies the fade-in with the bar to its mask, its tint and its line (`fullscreen_backdrop(.., opacity)`).

### 2. The sidebar on a tinted backdrop

Island or flat is still the user's preference (`sidebar_style`). The app only chooses a variant for when the bar is inside a `Backdrop`:

```rust
pub enum SidebarBackdrop { Clear, Tinted }   // Clear by default
Sidebar::new().backdrop(SidebarBackdrop::Tinted)
```

| | Without `Backdrop` | `Clear` | `Tinted` |
|---|---|---|---|
| Flat | system background | transparent, line with `Palette::separator` | same as `Clear` |
| Island | system background, border and inner border | no fill or borders | `Backdrop::average()` with alpha `BACKDROP.island` and border `Palette::separator` |

Inside a `Backdrop`, the rows take the ink and the accent from `Palette::current()` (`sidebar_palette`), not the system sidebar palette.

## Consequences

- A new app does not have to do anything for the glass to cover the whole window; it cannot limit it to one column either.
- If an app puts the `Titlebar` inside a container with `Overflow::Clip`, that container clips the glass: the `Titlebar` must not go inside one.
- If an app wraps the `Titlebar` in `.opacity(..)`, the glass stops blurring for the same reason as above.
- Ondina offers *Transparente / Tintada* ("Transparent / Tinted") in Apariencia ("Appearance"); Master Tengen App Show shows both in «Fondo tintado» ("Tinted backdrop").

## References

- [`0041`](0041-island-sidebar.md): island sidebar.
- [`0047`](0047-barrierkit-in-layers.md): layers and variants.
- malaysia ADR 0054: overlaid titlebar in focus.
