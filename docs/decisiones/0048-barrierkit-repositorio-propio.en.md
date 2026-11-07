# 0048 — BarrierKit in its own repository

Status: **accepted** (2026-10-28).

## Context

BarrierKit lives inside the Malaysia workspace (`crates/barrier-kit`), together with the compositor and the shell. ADR 0047 turns it into a complete toolkit for building any app, in the manner of AppKit or GTK + libadwaita. A toolkit like that has its own life cycle, its own showcase app and its own decisions, and should not share a repository with the compositor.

GNOME and macOS separate these pieces in the same way:

| GNOME | macOS | Malaysia |
|---|---|---|
| GTK + libadwaita (with Adwaita and `AdwStyleManager`) | AppKit (with Aqua and `NSAppearance`) | **barrierkit**: barrier-kit + Tengen + vow-appearance |
| gtk4-widget-factory (in the GTK repo) | — | Master Tengen App Show (in barrierkit) |
| Mutter | WindowServer / SkyLight | malaysia-compositor |
| gnome-shell | Dock, menu bar | malaysia-shell |
| gnome-control-center | System Settings | malaysia-settings |
| wayland-protocols | — | Infinity protocol |

In both cases, **the toolkit takes its style and the reading of the system appearance with it**. If `vow-design` and `vow-appearance` stayed in malaysia, barrierkit would depend on malaysia and malaysia on barrierkit: a cycle.

## Decision

### 1. Split of repositories

**`github.com/HBlanqueto/barrierkit`** (MIT) is the library:

```
barrierkit/
  Cargo.toml                  own workspace, same pinned versions (freya-* =0.4.3, SCTK…)
  crates/barrier-kit/         the toolkit (ADR 0034, 0047), with the `shell` feature
  crates/vow-design/          Tengen: tengen/tokens.json, components.json → Rust constants
  crates/vow-appearance/      appearance.domain + backdrop → resolved palette; watcher
  crates/infinity-protocol/   protocols/infinity-*.xml + generated code,
                              features `client` (kit) and `server` (compositor)
  crates/tengen-app-show/     Master Tengen App Show
  crates/marea/, ondina/      test apps: video and artwork; tinted backdrop
  docs/                       kit decisions and documentation
  flake.nix, nix/skia*        prebuilt Skia for freya-skia-bindings 0.98.0 + its guard
  scripts/                    check-client-only-deps, check-design-tokens
  dev.sh                      app, check and tracy
```

**`github.com/HBlanqueto/malaysia`** is the desktop:

```
malaysia/
  crates/malaysia-compositor, malaysia-scene, grade-graphics
  crates/malaysia-shell        (barrier-kit with features = ["shell"])
  crates/malaysia-settings     (System Settings, stays here)
  crates/xdg-desktop-portal-malaysia
  protocols/gtk-shell.xml      (only the compositor uses it)
  flake.nix, nix/malaysia.nix, dev.sh, share/, desktop docs/
```

### 2. How malaysia consumes the kit

Like any Rust library from another repository, the same as Smithay (ADR 0028):

```toml
# malaysia/Cargo.toml, [workspace.dependencies]
barrier-kit       = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<hash>" }
vow-design        = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<hash>" }
vow-appearance    = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<hash>" }
infinity-protocol = { git = "https://github.com/HBlanqueto/barrierkit", rev = "<hash>" }
```

- All four carry **the same `rev`**. Updating the kit in the desktop is a malaysia commit that changes that hash: `chore(deps): barrierkit <hash>`.
- **Joint development.** With `~/src/barrierkit` next to `~/src/malaysia`, `./dev.sh kit-local` writes an **unversioned** `.cargo/config.toml` with `[patch."https://github.com/HBlanqueto/barrierkit"]` pointing to the local paths, and `./dev.sh kit-remote` deletes it. This way a kit change is tested in the nested session before it is published.
- **Nix.** barrierkit is an input of the malaysia flake. crane vendors the git dependency with its `outputHashes`, the same as Smithay. Prebuilt Skia becomes an output of barrierkit (`packages.skia-binaries`) and malaysia reuses it: there is a single version guard, in barrierkit.

### 3. What does not change

- The compositor still does not link Freya or Skia. It only uses `vow-design`, `vow-appearance` and `infinity-protocol` (feature `server`), none of which depends on Freya. Cargo compiles only the crates that are requested even if they share a repo. `malaysia/scripts/check-client-only-deps.sh` keeps checking it.
- The names of crates, modules and types stay. Only where they come from changes.
- A single `main` branch in each repo.

### 4. History and documentation

- `crates/barrier-kit`, `crates/vow-design` and `crates/vow-appearance` move with **their history** (`git filter-repo`). In malaysia they are deleted with a commit that refers to barrierkit.
- **A single ADR numbering for both repos.** A new decision takes the next free number wherever it is, so that "ADR 0047" means the same in both. The kit ADRs (0011, 0032, 0033, 0034, 0035, 0039, 0041, 0046, 0047) move to `barrierkit/docs/decisiones/`. In malaysia, a one-line file remains for each one, referring to the new location. This ADR (0048) exists in both repos.
- The audits and the triage of P4 (BarrierKit) move to barrierkit.

### 5. Order

1. Close and push batch L1b in malaysia, as it is.
2. Separation batch, in a single step:
   - create barrierkit with history;
   - create `infinity-protocol`;
   - switch malaysia to the git dependency;
   - adjust the flakes and `dev.sh`.
3. L1c, L1d and all the phases of ADR 0047 are done in barrierkit. Each batch the desktop needs ends with a malaysia commit that bumps the `rev`.

## Consequences

- Changing the kit and using it in the desktop are two commits in two repos, although `dev.sh kit-local` lets you test them together first.
- barrierkit can be built, tested and shown (Master Tengen App Show) without the compositor. Its CI does not need Smithay.
- opencode reports that touch both repos must cite the commit of each one.

## References

- [`0028`](https://github.com/HBlanqueto/malaysia/blob/main/docs/decisiones/0028-smithay-por-revision.en.md): git dependency with a fixed rev.
- [`0034`](0034-celestialkit-sobre-freya.en.md): BarrierKit on Freya.
- [`0047`](0047-barrierkit-en-capas.en.md): BarrierKit in layers.
