# AGENTS.md — barrierkit

## Ramas y push (permanente)

- Todo cambio va **directo a `main`**. No crees ramas, ni locales ni remotas,
  ni hagas push a otra rama. Vale para cualquier agente (opencode, Claude Code
  en local o en la nube, DeepSeek, Kimi, etc.).
- Hay varias sesiones trabajando a la vez: `git pull --ff-only` antes de
  empezar y antes de cada push. Nunca `push --force`, `reset --hard` sobre
  commits subidos ni rebase de historia publicada.
- malaysia fija barrierkit por rev (`Cargo.toml`, `flake.nix`, `flake.lock`).
  Ese rev debe estar en `main`; un commit que solo vive en una rama puede
  desaparecer de GitHub y romper el build de malaysia.
- `for-malaysia` se unió a `main` el 2026-10-31; su historia vive en el tag
  `archivo/for-malaysia`.
