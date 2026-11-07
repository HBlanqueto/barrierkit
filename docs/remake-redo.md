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
| Clean build of the workspace | `cargo clean && cargo build --timings` | 369.8 s |
| Incremental build, `touch` | `touch crates/bk/src/controls.rs && cargo build` | 2.8 s |
| Incremental build, real edit | change `.max_lines(1)` in `controls.rs`, `cargo build` | 3.2 s |
| Incremental test build, real edit | the same edit, `cargo test --workspace --features bk/shell,bk/tracy --no-run` | 4.5 s |
| Clippy after a build | `cargo clippy --workspace --all-targets` | 49.1 s |
| Clippy, `touch` | `touch crates/bk/src/controls.rs && cargo clippy --workspace --all-targets` | 3.1 s |
| LLVM IR of `bk` | `cargo llvm-lines -p bk --lib` | 1 347 165 lines, 45 191 functions |
| Crates in the tree of `bk`, default features | `cargo tree -p bk -e normal,build --prefix none \| sort -u` | 271 |
| Crates in the tree of `bk`, `shell,tracy,video` | same, with `--features shell,tracy,video` | 303 |
| Unused dependencies | `cargo shear`, `cargo machete` | 0 real; 6 false positives (see below) |
| Advisories | `cargo deny --all-features check advisories` | 6 unmaintained, 0 vulnerabilities |
| Duplicate crates | `cargo deny --all-features check bans` | 16 crates with 2 or 3 versions |

The machine has 4 cores. A `touch` does not change the content, so rustc's
incremental cache reuses almost everything; the real edit is the number to
beat.

### Slowest crates of the clean build

Seconds of compilation per crate, summed over its units, from
`target/cargo-timings/cargo-timing.html`:

| Crate | s | Pulled by |
|---|---|---|
| `syn` (2.0 and 3.0, three units) | 134.6 | proc macros |
| `write-fonts` | 67.4 | `subsetter` (printing) |
| `zbus` | 61.5 | `bk`, `mundy` (`freya-core`), `accesskit_unix` |
| `bindgen` | 44.7 | `freya-skia-bindings` |
| `atspi-proxies` | 36.7 | `accesskit_unix` |
| `thiserror-impl` | 33.1 | proc macro |
| `freya-components` | 32.9 | `bk` |
| `accesskit_unix` | 32.1 | `bk` |
| `zvariant_utils`, `zbus_macros` | 61.6 | `zbus` |
| `read-fonts` | 28.9 | `subsetter` (printing) |
| `bk` | 23.0 | — |

`[profile.dev.package."*"]` outranks `build-override`, so proc macros and
build scripts are also built at `opt-level = 2`
([Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)).

### LLVM IR of `bk` by crate

| Crate of the function | Share |
|---|---|
| `bk` | 18.6 % |
| `core`, `alloc`, `std` | 34.2 % |
| `freya_core` | 14.7 % |
| `zbus`, `zvariant` | 12.8 % |
| `hashbrown` | 3.0 % |
| `spellbook` | 1.8 % |

No single generic stands out: the largest function is `MenuView::render`,
2 407 lines (0.2 %).

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

## Phase 1: profiles and toolchain

Each row is one run of the same steps: `cargo clean && cargo build`, then two
real edits of `controls.rs` followed by `cargo build` (lib) and
`cargo test --workspace --features bk/shell,bk/tracy --no-run` (tests).

| Change | Clean build | Edit, lib | Edit, tests | Adopted |
|---|---|---|---|---|
| Baseline (`mold`, deps at `opt-level = 2`) | 362.0 s | 3.6 s | 4.7 s | — |
| Default linker | 364.0 s | 3.4 s | 7.9 s | no |
| `wild` 0.11 | 360.5 s | 3.2 s | 4.6 s | no: within the noise of `mold` |
| Deps at `opt-level = 0`, except Freya, Skia, Torin and Ragnarok | 120.2 s | 3.6 s | 4.8 s | yes |
| The same, plus `spellbook` and the PNG stack at 2 | 119.8 s | 3.6 s | 4.9 s | yes |
| `ash` without `debug` | 114.3 s | 3.5 s | 4.7 s | yes |

Runtime checks for the `opt-level` change:

| Check | At `opt-level = 0` | At `opt-level = 2` |
|---|---|---|
| `spellbook::Dictionary::new` with `es_ES` | 171 ms | 21 ms |
| `cargo test -p bk --test kit` (PNG encode and decode) | 10.5 s | 9.7 s |

Not changed, with the reason:

- `zbus` defaults: `bk` uses `zbus::blocking` in `app/application.rs`,
  `notifications.rs` and `secrets.rs`.
- `wayland-protocols`: `bk` uses both `staging` and `unstable` protocols.
- `toml` 0.8 → 1.x and `toml_edit` 0.22 → 0.25: the other versions build for
  the host (proc macros and build scripts), so they are separate units anyway.

## Phase 2: opt-in features of `bk`

Clean build of `bk` alone (`cargo clean && cargo build -p bk …`), after
phase 1:

| Features | Clean build | Edit of `controls.rs` | Crates |
|---|---|---|---|
| `a11y`, `charts`, `print` (the defaults until `default = ["a11y"]`) | 117.1 s | 3.2 s | 271 |
| `--no-default-features` | 101.1 s | 3.0 s | 244 |
| only `a11y` | 106.8 s | — | 260 |
| only `charts` | 103.5 s | — | 244 |
| only `print` | 113.5 s | — | 255 |

| Feature | Modules | Dependencies it adds | Used by malaysia | Used by curseterm |
|---|---|---|---|---|
| `a11y` | `platform/atspi.rs` | `accesskit_unix` (+16 crates) | every app window | every window |
| `charts` | `charts`, `styles::ChartStyle` | none | no | no |
| `print` | `print`, `portal::print`, `styles::PrintPreviewStyle` | `pdf-writer`, `subsetter`, `ttf-parser`, `flate2` (+11 crates) | no | no |
| `shell` | `shell` | none | `malaysia-shell` | no |
| `video` | `video` | `gstreamer*` (+29 crates) | no | no |
| `tracy` | `init_tracing` to Tracy | `tracing-tracy` | no | no |

Candidates left out, with the reason:

| Candidate | Why not |
|---|---|
| `spelling` | `spellbook` costs 7 s of CPU, and `TextField`, `TextEditor` and the context menu call it |
| `dbus` | `freya-core` pulls `zbus` through `mundy` anyway, and `portal` serves the clipboard, drag and drop, `ImageWell`, `TextEditor` and buttons |
| i18n | Every visible string of the kit goes through `tr!` |

`cargo hack clippy --feature-powerset --no-dev-deps -p bk -- -D warnings`
passes for the 64 combinations (691 s).
