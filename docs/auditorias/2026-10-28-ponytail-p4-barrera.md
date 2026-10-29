# Ponytail ultra audit P4 — `crates/barrier-kit`

- **Commit:** `cae5759`.
- **Scope:** `crates/barrier-kit/src/**/*.rs`, `crates/barrier-kit/examples/*.rs`, `crates/barrier-kit/tests/*.rs`, `crates/barrier-kit/Cargo.toml`.
- **Files read:** 23 files, ~11 kLOC.
- **Auditor:** OpenCode / ponytail-ultra.
- **Date:** 2026-10-28.

## TL;DR

BarrierKit is a small, well-tested toolkit, but its three risky areas are all live at runtime: **Freya hook rules**, **D-Bus menu registration**, and **theme loading**. None of the findings are catastrophic, but several will break menus or leak resources in a multi-window app. Three dependencies are dead weight and can be removed today. No new `unsafe` was introduced in this crate.

---

## Findings

### B4-01. `use_hook` is called conditionally inside `menu_popup.rs` selftest (Freya rule violation)

- **Where:** `crates/barrier-kit/examples/menu_popup.rs:42`.
- **What:** `use_hook(move || { ... })` is inside `if std::env::var_os("MENU_POPUP_SELFTEST").is_some()`.
- **Why it matters:** `freya-core-0.4.3/src/lifecycle/base.rs` requires hooks to be called in the same order on every render; a conditional branch that is sometimes absent violates the rule. In debug builds this may already panic; in release builds it corrupts hook state.
- **Fix:** Move the hook outside the `if` and make the body no-op when the env var is absent, or render a different root component for the selftest.
- **If skipped:** Future Freya upgrade or debug assertions turn the example into a crash.

### B4-02. `MenuPopup`/`MenuView::new` accepts no close/dismiss callback

- **Where:** `crates/barrier-kit/src/menu.rs:508`.
- **What:** `MenuView::new(items, on_select, on_close)` exists, but the higher-level `MenuPopup` component in `menu_popup.rs` wires `on_close` to `|| {}` in several places (`menu_popup.rs:402`, `:535`, `:813`) and stores the only real dismissal path in `PopupHandle`.
- **Why it matters:** A caller that embeds `MenuView` directly loses the keyboard-escape or parent-dismiss path unless it also owns the `PopupHandle`. The API is split between the view and the platform popup.
- **Fix:** Add an `on_dismiss` callback to the `MenuPopup` public API and document that `MenuView` alone does not dismiss itself.
- **If skipped:** Reusable menu components in future shell work will re-implement dismissal or leak popups.

### B4-03. App-menu D-Bus paths are reused after a window closes (confirmed from PX-08)

- **Where:** `crates/barrier-kit/src/app_menu/server.rs:64-68`.
- **What:** `let index = menus.len() as u32;` then `format!("{PATH_PREFIX}{index}")`.
- **Why it matters:** Closing window 0 leaves one entry; the next window also gets path `/MenuBar/1`, object registration fails, menus silently disappear.
- **Fix:** Use a monotonic `AtomicU32` counter instead of `len()`.
- **If skipped:** Multi-window native apps lose menus after any window closes.

### B4-04. `Server::register` sends an `Arc<RegisteredMenu>` across threads without checking the receiver is alive

- **Where:** `crates/barrier-kit/src/app_menu/server.rs:77-82`.
- **What:** `send_blocking(Command::Register { ..., registered, reply })` then `block_on(reply_rx.recv()).ok().flatten()`.
- **Why it matters:** If the D-Bus thread panics, `reply_rx.recv()` returns `None`, the function returns `None`, but the menu is still inserted into the local `menus` map on line 75.
- **Fix:** Insert into `menus` only after receiving a successful reply, or roll back on failure.
- **If skipped:** A dead D-Bus thread leaves orphaned registry entries and leaking `Arc`s.

### B4-05. `theme::resolved()` reads a JSON file synchronously on every render

- **Where:** `crates/barrier-kit/src/theme.rs:133-160` (static inside `RESOLVED`).
- **What:** The cached `OnceLock` is fine after first call, but the initial read uses `std::fs::read_to_string` with no error path that warns the user; a parse error causes an unconditional `panic!`.
- **Why it matters:** Same finding as `apariencia.md` H2; it is still present. A malformed user theme crashes the shell process on startup.
- **Fix:** Return a `Result` and fall back to the compiled-in `tengen` tokens; log the path that failed.
- **If skipped:** One bad theme file prevents the session from starting.

### B4-06. `theme::resolved()` cache is never invalidated when the user changes theme

- **Where:** `crates/barrier-kit/src/theme.rs:126-132`.
- **What:** `RESOLVED` is a `OnceLock<ResolvedTheme>`.
- **Why it matters:** There is no `reload()` or file watcher. The settings app will need a full shell restart to apply a theme change.
- **Fix:** Expose a `reload()` that replaces the cached value and emit `AppearanceChanged` after the settings domain is updated.
- **If skipped:** Theme changes feel broken compared to macOS / GNOME.

### B4-07. Five `fn rgba` helpers duplicate color parsing logic

- **Where:** `crates/barrier-kit/src/theme.rs:371`, `:380`, `:389`, `:398`, `:407`.
- **What:** Identical or near-identical `fn rgba(hex: &str) -> Color` implementations (confirmed by `rg 'fn rgba'` returning 5 matches).
- **Why it matters:** P5-14 already found the same duplication in the shell; now it is also in the crate that owns the tokens. A color format change requires editing five places.
- **Fix:** One `pub(crate) fn rgba` in `theme.rs`; delete the duplicates.
- **If skipped:** Theme maintenance cost doubles.

### B4-08. `fn svg_viewer` and `fn svg_with_color` are duplicated between crates

- **Where:** `crates/barrier-kit/src/controls.rs` and `crates/barrier-kit/src/sidebar.rs`.
- **What:** `rg 'fn svg_viewer'` returns 2 matches; `rg 'fn svg_with_color'` returns 1, but the pattern (load SVG → render with color override) appears in both files.
- **Why it matters:** Same duplication pattern as P0 baseline findings. Fixes to SVG caching or color handling must be applied twice.
- **Fix:** Move both helpers to a shared `svg` module inside `barrier-kit`.
- **If skipped:** Sidebar and titlebar controls diverge in icon rendering.

### B4-09. `_traffic_input` is dead code kept with `#[allow(dead_code)]`

- **Where:** `crates/barrier-kit/src/titlebar.rs:506-514`.
- **What:** A private helper that converts `WindowStatus` to `TrafficInput` is never called.
- **Why it matters:** Dead from PX-15. It is not exported, not tested, and not used by the live `traffic.rs` path.
- **Fix:** Delete it; if the conversion is needed later, derive it from the live component.
- **If skipped:** Rotten helper that misleads the next developer touching traffic-light input.

### B4-10. `MenuItem` labels are stored as `String` with no max-length guard

- **Where:** `crates/barrier-kit/src/menu.rs:17-47` (enum definition).
- **What:** `label: String` for actions, submenus, and separators.
- **Why it matters:** A malicious or buggy app can send a megabyte-long label. `measure_text` and `MenuView::preferred_size` will allocate and render it.
- **Fix:** Add a `const MAX_LABEL_LEN: usize` and truncate or reject labels in `MenuView::new`.
- **If skipped:** Easy DoS vector for any app that exports a D-Bus menu.

### B4-11. `measure_text` allocates a fresh paragraph for every call

- **Where:** `crates/barrier-kit/src/menu.rs:194-225`.
- **What:** Creates a `cosmic_text::Buffer` and shapes the text every time a menu size is requested.
- **Why it matters:** Menu sizing happens during layout and hover; repeated allocation adds up when opening nested submenus.
- **Fix:** Cache the paragraph keyed by `(text, font_size)` in `MenuView`, or use `Buffer::set_text` + `SwashCache`.
- **If skipped:** Menu popups will be measurably slower on low-end hardware.

### B4-12. `MenuView` keyboard navigation is not in tests

- **Where:** `crates/barrier-kit/tests/menu.rs`.
- **What:** Tests cover rendering, disabled items, checkmarks, and separators, but no test sends arrow keys, Enter, or Escape.
- **Why it matters:** The keyboard path is the primary accessibility path; it can break without CI noticing.
- **Fix:** Add three tests: arrow-down skips disabled, Enter fires `on_select`, Escape calls `on_close`.
- **If skipped:** Regressions in keyboard menu navigation go unnoticed.

### B4-13. `password.rs` zeroizes on drop but not on every edit

- **Where:** `crates/barrier-kit/src/password.rs:42-58`.
- **What:** `zeroize::ZeroizeOnDrop` is derived; the buffer is cleared when the component is destroyed.
- **Why it matters:** Edits replace the `String` contents, but the old allocation is not zeroized and may remain in freed memory.
- **Fix:** Use `Zeroizing<String>` or manually `zeroize()` before every reassignment.
- **If skipped:** Lock-screen password fragments stay in memory longer than necessary.

### B4-14. `traffic.rs` tests assert exact snapshot bytes without tolerating Skia version drift

- **Where:** `crates/barrier-kit/tests/traffic.rs:200-246`.
- **What:** Tests compare PNG bytes against checked-in snapshots.
- **Why it matters:** Skia produces slightly different rasterization across versions; the first dependency bump will fail CI for non-bugs.
- **Fix:** Compare dominant colors / bounding boxes instead of raw bytes, or regenerate snapshots in CI.
- **If skipped:** Future `freya-skia-bindings` update produces red herring failures.

### B4-15. `animation.rs` has no tests

- **Where:** `crates/barrier-kit/src/animation.rs` (85 lines).
- **What:** Spring easing and `animate` helper are untested.
- **Why it matters:** Animation bugs are subtle (overshoot, NaN on zero stiffness, duration mismatch).
- **Fix:** Add two tests: spring converges to target, and zero duration jumps immediately.
- **If skipped:** Animated UI glitches are caught only visually.

### B4-16. `controls.rs` zoom button uses a hard-coded size instead of a token

- **Where:** `crates/barrier-kit/src/controls.rs:190-210` (approximate, verified by read).
- **What:** The zoom button rect uses literal `12.0`/`16.0` sizes.
- **Why it matters:** ADR 0020/ADR 0011 require all design values to come from `vow-design`.
- **Fix:** Replace literals with `vow_design::TRAFFIC_LIGHT_SIZE` or add a new token.
- **If skipped:** Tengen theme change does not propagate to zoom button.

### B4-17. `sidebar.rs` duplicates expand/collapse animation constants

- **Where:** `crates/barrier-kit/src/sidebar.rs:120-150`.
- **What:** Spring stiffness and damping literals for the sidebar collapse animation.
- **Why it matters:** Same values may be reused elsewhere (dock, menubar). Centralizing them in `vow-design` keeps motion consistent.
- **Fix:** Add `SIDEBAR_SPRING_STIFFNESS`/`DAMPING` tokens, or at minimum crate constants.
- **If skipped:** Different shell components animate at different speeds.

### B4-18. `Cargo.toml` declares `wayland-backend` but never uses it directly

- **Where:** `crates/barrier-kit/Cargo.toml:32`.
- **What:** `wayland-backend = "0.3"` is listed.
- **Why it matters:** No direct import found in `src/`, `examples/`, or `tests/`.
- **Fix:** Remove the line and verify `cargo check -p barrier-kit` still passes.
- **If skipped:** Unused dependency increases closure size and Nix build time.

### B4-19. `Cargo.toml` declares `futures-channel` and `futures-util` separately; `futures` would be cleaner

- **Where:** `crates/barrier-kit/Cargo.toml:36-37`.
- **What:** Both `futures-util` and `futures-channel` are listed.
- **Why it matters:** The crate already uses `async-channel` and `async-io`; several `futures-*` usages are only for `mpsc` and `waker`, which could be served by `futures` with a smaller surface.
- **Fix:** Evaluate replacing the two crates with `futures = "0.3"` and the needed features.
- **If skipped:** Low-priority dependency clutter.

### B4-20. `app_menu/layout.rs` uses `Vec::remove` during diff without a transaction

- **Where:** `crates/barrier-kit/src/app_menu/layout.rs:200-260` (approximate).
- **What:** `build_layout` / `diff_layout` mutate a `Vec` of children while emitting D-Bus update signals.
- **Why it matters:** A partial failure mid-diff can leave the layout and the D-Bus view out of sync.
- **Fix:** Build the new layout in a temporary vector, swap it in, then emit `LayoutUpdated` once.
- **If skipped:** Rare menu corruption after dynamic menu changes.

### B4-21. `platform/mod.rs` is 4,000+ lines and mixes five responsibilities

- **Where:** `crates/barrier-kit/src/platform/mod.rs`.
- **What:** One file contains Wayland connection, layer-shell/popup management, EGL context, event dispatch, timers, and Infinity protocol.
- **Why it matters:** Navigation cost is high; changes to popups risk breaking EGL teardown.
- **Fix:** Split into `platform/wayland.rs`, `platform/egl.rs`, `platform/popup.rs`, `platform/timer.rs`, keeping `mod.rs` as a thin facade.
- **If skipped:** Each new protocol (Bloque 11/12) makes the file harder to maintain.

### B4-22. `platform/mod.rs` `Drop` for `EglContext` does not wait for GPU idle

- **Where:** `crates/barrier-kit/src/platform/mod.rs:1020-1050` (approximate, verified by read).
- **What:** EGL surfaces are destroyed, but no `glFinish` or fence is inserted.
- **Why it matters:** On Mesa/Intel, tearing down the display while GPU commands are in flight can hang or corrupt the next process that reuses the card.
- **Fix:** Call `glFinish()` before `eglMakeCurrent(dpy, EGL_NO_SURFACE, ...)` in the drop path.
- **If skipped:** Intermittent GPU hangs when shell components restart.

### B4-23. `MenuView::preferred_size` does not consider icon sizes

- **Where:** `crates/barrier-kit/src/menu.rs:340-420`.
- **What:** Width is computed from label + shortcut + padding, but icons are rendered at a fixed size without being measured.
- **Why it matters:** A wide custom icon will overlap the label or be clipped.
- **Fix:** Include `icon.width()` in the row width calculation.
- **If skipped:** Custom app icons in menus look broken.

### B4-24. Examples compile `menu_popup.rs` with `type UseState<T> = State<T>` alias and then never use it

- **Where:** `crates/barrier-kit/examples/menu_popup.rs:19`.
- **What:** `type UseState<T> = State<T>;` is defined but the example uses `State` directly from `freya_core::prelude::*`.
- **Why it matters:** Dead alias; confusing for readers copying the example.
- **Fix:** Remove the alias.
- **If skipped:** Minor example hygiene issue.

### B4-25. `app_menu.rs` exposes `MenuHandle::set_path` but no getter

- **Where:** `crates/barrier-kit/src/app_menu.rs:90-100` (approximate).
- **What:** The handle is mutated by `server.rs` after creation.
- **Why it matters:** The split ownership makes it unclear who owns the canonical path; a getter would help tests assert the assigned path.
- **Fix:** Add `pub fn path(&self) -> Option<&str>`.
- **If skipped:** Tests for B4-03 cannot verify the path without reaching into internals.

### B4-26. No documented public re-export list in `lib.rs`

- **Where:** `crates/barrier-kit/src/lib.rs:1-59`.
- **What:** The module list is present but there is no doc comment describing the crate’s public surface.
- **Why it matters:** New shell/app developers have to grep the source to know what is public API.
- **Fix:** Add a module-level `#![doc = include_str!("../README.md")]` or write a short `//!` block.
- **If skipped:** Onboarding friction.

### B4-27. `controls.rs` and `titlebar.rs` duplicate window-button hit-test math

- **Where:** `crates/barrier-kit/src/controls.rs:150-200` and `crates/barrier-kit/src/titlebar.rs:300-360`.
- **What:** Both compute close/minimize/zoom button rectangles and pointer proximity.
- **Why it matters:** Same layout logic in two places; a Tengen token change must be edited twice.
- **Fix:** Extract a `traffic_geometry` helper used by both modules.
- **If skipped:** Button positions drift between CSD and SSD titlebars.

### B4-28. `sidebar.rs` `Sidebar` has no `min_width`/`max_width` props

- **Where:** `crates/barrier-kit/src/sidebar.rs:60-90`.
- **What:** The component accepts `width` but clamps nothing.
- **Why it matters:** macOS Finder sidebar has a minimum width; without it the user can collapse the sidebar to zero by accident.
- **Fix:** Add optional `min_width` and `max_width` props with sensible defaults.
- **If skipped:** Shell sidebar UX regresses.

### B4-29. `examples/window_csd.rs` and `window_ssd.rs` are identical except for one flag

- **Where:** `crates/barrier-kit/examples/window_csd.rs` and `window_ssd.rs`.
- **What:** Both files are 69 lines and differ only in the `WindowChrome` variant.
- **Why it matters:** Example duplication means fixes to one are forgotten in the other.
- **Fix:** Merge into one `window.rs` example that takes `--csd`/`--ssd` arguments.
- **If skipped:** Examples slowly diverge.

### B4-30. `menu.rs` separator rendering uses a fixed `1.0` stroke

- **Where:** `crates/barrier-kit/src/menu.rs:720-740` (approximate).
- **What:** Menu separator height is hard-coded.
- **Why it matters:** ADR 0020 requires separators to come from `vow-design` tokens.
- **Fix:** Replace with `vow_design::MENU_SEPARATOR_HEIGHT`.
- **If skipped:** Theme scaling does not affect separators.

---

## AppKit gaps

1. **No rich-text labels in menus.** `MenuItem::label` is plain `String`; macOS supports attributed titles and custom views.
2. **No toolbar API.** BarrierKit has titlebar, sidebar, and menus, but no `Toolbar` component for app windows.
3. **No drag-and-drop primitives.** No `on_drag`/`on_drop` hooks on `WindowChrome` or sidebar items.
4. **No accessibility tree wiring.** `accesskit` is a dependency but no `NodeBuilder` integration was found in the components.
5. **No window state restoration.** Geometry and sidebar collapse state are not persisted.

---

## Questions

1. Is the `wayland-backend` dependency kept for a future `infinity` protocol binding, or is it safe to remove?
2. Should `theme::resolved()` watch the theme file, or should the settings app call an explicit `reload()`?
3. Are the PNG snapshots in `tests/traffic.rs` regenerated automatically in CI, or are they pinned to the Nix-built Skia version?
4. Does `platform/mod.rs` intentionally keep all Wayland/EGL/timer code in one file for build-time reasons, or is a split welcome?

---

## Seeds from previous audits verified/refuted

| Seed | Status | Evidence |
|------|--------|----------|
| PX-08 app-menu path reuse | **Confirmed** | `app_menu/server.rs:64-68` still uses `menus.len()` as index. |
| PX-15 dead `_traffic_input` | **Confirmed** | `titlebar.rs:506-514` still marked `#[allow(dead_code)]`. |
| H2 `theme.rs` disk read on startup | **Confirmed** | `theme.rs:133-160` reads JSON synchronously and panics on error. |
| P5-14 duplicate `rgba` helpers | **Confirmed in barrier-kit** | 5 `fn rgba` copies in `theme.rs`. |
| P0 duplicate SVG viewers | **Confirmed in barrier-kit** | `controls.rs` and `sidebar.rs` both have `svg_viewer`. |

---

## Verdict

Fix first: **B4-03** (menu path reuse), **B4-01** (conditional hook), **B4-05** (theme panic), **B4-13** (password zeroize), **B4-10** (menu label length). Then delete the dead code/helpers (B4-09, B4-07, B4-08, B4-18) and split `platform/mod.rs` when the next protocol lands.

Lean: remove `wayland-backend`, deduplicate `rgba`, `svg_viewer`, and `_traffic_input`, and merge the two window examples. Net reduction: ~150 lines and one dependency.
