# Remake-Redo: tracking

**Goal:** record the measurements that decide each step of Remake-Redo, the
cleanup of build times, features, dead code and documentation.
**Scope:** from the baseline of phase 0 to the before/after table of phase 6.
This file is deleted when the project ends.
**Reader:** whoever continues the project.
**Input:** the workspace at `ed53f46`. **Output:** one table per measurement,
with the command that produced it.

## Steps

1. Enter `nix develop`. Every build number below comes from inside it.
2. Run the command of each row and write its result in the table.
3. Mark a row `not run` with the reason when the command cannot run.
4. Repeat the same commands in phase 6 and add an `After` column.

## Baseline

| Measurement | Command | Before |
|---|---|---|
| Clean build of the workspace | `cargo build --timings` after `cargo clean` | not run: dev shell blocked |
| Incremental build of `bk` | `touch crates/bk/src/controls.rs && cargo build --timings` | not run: dev shell blocked |
| Clippy of the workspace | `cargo clippy --workspace --all-targets` | not run: dev shell blocked |
| Generic bloat of `bk` | `cargo llvm-lines -p bk` | not run: dev shell blocked |
| Crates in the tree of `bk`, default features | `cargo tree -p bk -e normal,build --prefix none \| sort -u` | 271 |
| Crates in the tree of `bk`, `shell,tracy,video` | same, with `--features shell,tracy,video` | 303 |
| Unused dependencies | `cargo shear`, `cargo machete` | 0 real; 6 false positives (see below) |
| Advisories | `cargo deny --all-features check advisories` | 6 unmaintained, 0 vulnerabilities |
| Duplicate crates | `cargo deny --all-features check bans` | 16 crates with 2 or 3 versions |

The dev shell needs the Skia source from `github.com/rust-skia/skia`; on
2026-11-07 the cloud session could not download it.

### Unused dependencies

Both tools read source text, so they miss uses inside macros and renamed
crates:

| Report | Why it is used |
|---|---|
| `infinity-protocol`: `bitflags`, `wayland-backend`, `wayland-client`, `wayland-server` | The code that `wayland_scanner::generate_*!` expands names them |
| `bk`: `freya-skia-safe` | Its library is `skia_safe`, used in `platform/`, `charts/` and `video.rs` |
| `skia-prebuild`: `freya-engine`, `freya-skia-safe` | The crate exists only to build Skia |

### Advisories

| Crate | Pulled by |
|---|---|
| `zvariant`, `zvariant_derive`, `zbus_names` (merged into `zbus`) | `zbus` 5 |
| `ttf-parser` | `bk` (printing) and `freya-components` |
| `paste` | `freya-core` |
| `basic-toml` | `i18n-embed` |

### Duplicates that the workspace can remove

| Crate | Versions | The workspace's version comes from |
|---|---|---|
| `toml` | 0.5, 0.8, 1.1 | `toml = "0.8"` in `[workspace.dependencies]`; Skia's build uses 1.1 |
| `toml_edit` | 0.22, 0.25 | `toml_edit = "0.22"`; `proc-macro-crate` uses 0.25 |

The other duplicates (`syn` 2 and 3, `thiserror` 1 and 2, `hashbrown`,
`itertools`, `event-listener`, `objc2*`, `windows-sys`) come from upstream
crates.

### What each dependency of `bk` adds

Crates that leave the tree of `bk` (all features) when the group goes:

| Group | Crates removed |
|---|---|
| `i18n-embed`, `i18n-embed-fl`, `rust-embed` | 44 |
| `gstreamer*` (`video`) | 29 |
| `accesskit_unix` | 15 |
| `pdf-writer`, `subsetter`, `ttf-parser`, `flate2` | 11 |
| `tracing-tracy` (`tracy`) | 3 |
| `spellbook` | 1 |
| `zbus` | 0 |

`zbus` and `flate2` stay whatever `bk` does: `freya-core` 0.4.3 depends on
`mundy` with the `async-io` feature, which pulls `zbus` on Linux, and
`freya-skia-bindings` needs `flate2` to build.
