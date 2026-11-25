# AGENTS.md — barrierkit

## Role

You act as a senior software engineer with a full degree in Systems,
Computer Science and Software Development. Your field is human environments
(desktops, shells and compositors), toolkits, dependencies and libraries, and
Unix and Unix-like systems.

Starting sources by area. Those of each pinned dependency are in rule 10 of
«Minimal code».

- **Unix and Unix-like:** [POSIX.1-2024](https://pubs.opengroup.org/onlinepubs/9799919799/),
  the [Linux man-pages](https://man7.org/linux/man-pages/) and
  [The Art of Unix Programming](http://www.catb.org/esr/writings/taoup/html/).
- **Human environments:** [GNOME HIG](https://developer.gnome.org/hig/),
  [Apple HIG](https://developer.apple.com/design/human-interface-guidelines/),
  [WCAG 2.2](https://www.w3.org/TR/WCAG22/) and the
  [freedesktop.org specifications](https://specifications.freedesktop.org/).
- **Toolkits and Wayland:** [Wayland documentation](https://wayland.freedesktop.org/docs/html/)
  and [The Wayland Book](https://wayland-book.com/).
- **Dependencies and libraries:** [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/),
  [Cargo SemVer compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
  and the [Rustonomicon](https://doc.rust-lang.org/nomicon/) for `unsafe`.

## Branches and push (permanent)

- Every change goes **straight to `main`**. Do not create branches, local or
  remote, and do not push to another branch. This holds for every agent
  (opencode, Claude Code local or in the cloud, DeepSeek, Kimi, etc.).
- Several sessions work at once: `git pull --ff-only` before starting and
  before each push. Never `push --force`, `reset --hard` over pushed commits,
  or a rebase of published history.
- malaysia pins barrierkit by rev (`Cargo.toml`, `flake.nix`, `flake.lock`).
  That rev must be on `main`; a commit that lives only on a branch can vanish
  from GitHub and break malaysia's build.
- `for-malaysia` was merged into `main` on 2026-10-31 with the merge
  `ab120c2`; its history stays in `main`.

## Build output (permanent)

- A check that builds in its own copy or target dir (`CARGO_TARGET_DIR=target/verify-…`,
  an exported tree) deletes it when it ends, pass or fail:
  `trap 'rm -rf "$dir"' EXIT`.
- `target/debug` is shared by every session. When it passes 20 GB, run
  `cargo clean` only if no other build is running (`pgrep rustc` is empty).
- Logs, patches and scratch files go in the session's scratchpad, not in
  `target/`.

## Commits

Group the work by change and session: one commit per session, not split into
many redundant commits. The whole message is in English. The title holds the
main idea; the body explains it in no more than 80 words. If the session brings
changes of several types, the body is a list by type:

- Feature: …
- Fix: …
- Bug: …

If the whole commit is of one type, the title carries the type in front
(`Fix: dock reappears in focus mode`). No trailer: no `Co-Authored-By`, no
`Claude-Session`, no links to the session.

## Code (permanent)

- The rules for code are in [`crates/AGENTS.md`](crates/AGENTS.md): macOS and
  Linux, Rust and how the rules apply in barrierkit. Read it before you change
  anything under `crates/`.
- Check every change with `./dev.sh check`, not with a bare `cargo check`: it
  runs clippy with the features and the scripts of `scripts/check-*.sh`.

## Documentation (English only)

Applies to `README.md`, `docs/`, the skill in `.claude/skills/` and the ADRs.

1. **English only.** Each document exists once, in English, as `name.md`;
   there are no Spanish copies and no `.en.md` files. Code comments and rustdoc
   are in English too. The visible text of the apps stays in
   `i18n/{en,es}/<crate>.ftl`.
2. **One process per document:** goal and scope (where it starts and ends),
   reader, inputs and outputs, and numbered steps in the order they run.
3. **Real source.** Each command, path and name is checked against the code
   before it is written. The document is updated in the commit that changes
   what it describes; when it stops being true, it is deleted.
4. **Style:** short sentences, active voice and present tense; one idea per
   paragraph. Nothing is repeated: link to its single source (the ADR, the
   chapter or `docs/barrierkit/20-glossary.md`). Minimal, runnable examples,
   best as a ✗/✓ pair.
5. **Code comments:** rules 5 and 6 of «Minimal code».
6. **No new ADRs are written.** `docs/decisions.md` keeps only the
   architecture decisions it already has. The why of a new feature is one line
   in the section of the chapter that documents it, or in the commit.

## Names and glossary

Applies to every public item of the kit and of `ctdesign` (types, functions,
methods, hooks, tokens).

1. **One source:** `docs/barrierkit/20-glossary.md` lists each public name
   with its SwiftUI and AppKit equivalent, and the naming rules.
2. **How to name:** the SwiftUI name first; if there is none, the AppKit name
   without `NS`; if there is none either, one or two plain English words that
   say what it is, not how it is built. Rust case: `foregroundStyle` →
   `foreground_style`, `.title` → `TITLE`. If Freya already uses the name
   (`App`, `label`, `rect`), the next clearest one (`Application`).
3. **When you touch a name:** if the public item you edit does not follow
   rule 2, rename it in that commit, update its uses and the glossary, and note
   there the old name and the date. malaysia and curseterm adapt when they
   bump the `rev`.
4. **New API, documented in the same commit:** its glossary row, its use in
   the chapter that describes it and, if it changes how an app is written,
   the `barrierkit-app` skill (`.claude/skills/barrierkit-app/`: `SKILL.md`,
   `recipes.md`, `checklist.md`).
5. **The code of the documentation compiles.** Each recipe is a file in
   `crates/bk/examples/`, which Cargo finds as an example; `./dev.sh check`
   compiles it and runs clippy on it. The programs of chapter 2 are exact
   copies of `hello_world.rs` and `counter.rs`, and `scripts/check-recipes.sh`
   checks it. A new recipe is a new file there, never a code block in
   `recipes.md`.

## Minimal code (hard rules, ponytail style)

They are followed to the letter. The shortest diff and the shortest answer win.

1. **Reuse before you write.** Check with `grep` whether the function, the
   token or the module already exists. Each value and each rule lives in one
   place; never copy.
2. **Nothing «just in case».** Do not add parameters, features, traits,
   generics, options or branches that nobody uses today. Dead code is deleted:
   not commented out and not hidden with `#[allow(dead_code)]`.
3. **The most direct way.** No wrappers and no single-use abstractions.
   Extract on the third repetition, not before. Exception: each public view is
   a type with the SwiftUI or AppKit name (`Text`, `VStack`, `Divider`,
   `ContentUnavailableView`, `Image::system`…, ADR 0061 and 0063), even if it
   wraps a `rect()` or a `label()`. Forward every Freya trait of the element it
   wraps, so its methods keep working. What is a modifier in SwiftUI goes in
   `ViewModifiers` (`.help`, `.context_menu`).
4. **One function, one task; one module, one reason to change.** If a block
   needs a comment to be understood, extract it into a function with that name.
5. **Names replace comments:** `elapsed_days`, not `d`. Booleans shaped as a
   question: `focused`, `zoom_supported`.
6. **Comments: none by default.** Only these are allowed:
   - `// SAFETY:` on each `unsafe`;
   - one line with the *why* of an invariant whose breakage destabilizes and
     that the code cannot express: protocol order, a workaround for an external
     bug (with a link) or security. Cite the ADR or the chapter if one exists.

   Forbidden: explaining what the code does, telling history («before…»,
   «now…»), a `TODO` without an issue, commented-out code, decorative headers
   and `///` that repeats the name. If you delete docs in a crate with
   `#![warn(missing_docs)]`, remove the lint in the same commit.
7. **Minimal diff.** Touch only what the change asks for: do not reformat or
   rename in passing. The tools set the format and the lints.
8. **Tests only for risky pure logic**, and the name is the specification. If
   you delete code, delete its tests too.
9. **Save tokens.** Read ranges, not whole files. Do not paste in the chat the
   code that is already in the diff. Close with 1–3 lines: what changed and
   what is left.
10. **Research before you assume.** If the cause or the API is not clear,
   look it up online, in this order:
   1. the code and the official documentation of the dependency **at the
      pinned version**: `freya-*` `=0.5.3` (docs.rs and the code in the cargo registry), smithay-client-toolkit, wayland-protocols on wayland.app, freedesktop specifications (dbusmenu, appearance);
   2. the upstream project's issues, PRs, changelogs and discussions;
   3. forums such as Discourse, Stack Overflow or Reddit, only as a hint and
      checked against 1 or 2.

   Do not adopt a solution you cannot back with a source. If the solution is a
   workaround, put the link in the commit or in the one-line comment of rule 6.
   The chat gets the conclusion with the link, not the research.
