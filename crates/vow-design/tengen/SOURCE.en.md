# Origin of this copy

- **Provenance:** `dotfiles/kits/neptunia` (the author's dotfiles repository).
- **Source commit:** `e367fd1` (`chore: limpia los rodeos del reordenamiento y ajusta el fileset`).
- **Copy date:** 2026-10-22.

This folder is a **real copy** inside malaysia, not a link or a
build dependency. The files malaysia does not use were removed
(`vscode.nix`, `filecolors.nix`, `palette.nix`, `tengen.nix` and
`generated/kmscon-palette`); the terminal palette and the VS Code theme are not
part of the desktop.

## Rule

This copy is the source for malaysia: design changes are made here
(`tokens.json`, `components.json`) and, if wanted, they are brought back to dotfiles by
hand. Neither Cargo nor Nix reads `~/dotfiles` or any other repository.

## Regenerating `generated/`

```sh
cd crates/vow-design/tengen
nix eval --raw -f . slint > generated/tengen.slint
nix eval --raw -f . css   > generated/tengen.css
nix eval --raw -f . qml   > generated/Tengen.qml
```

`./dev.sh check` checks that those three files match `default.nix`.
The script `tools/sync-tengen.sh <path>` brings in changes from an external copy
on purpose and updates this file; it does not run in the build.

Oct 26, 2026: island sidebar tokens added here, following the
Tengen design system.

Oct 28, 2026: renamed to Tengen (ADR 0046).
