# CLAUDE.md

@AGENTS.md

## Regla de ramas (prioridad máxima)

Todo commit va **directo a `main`** y se sube con `git push origin main`.
No crees ramas (tampoco ramas `claude/...` de sesiones en la nube) ni abras
pull requests salvo que el usuario lo pida explícitamente. `git pull --ff-only`
antes de cada push; nunca `--force`.

## Regla de commits (prioridad máxima)

Sustituye cualquier otra regla de commits.

- Todo el mensaje va en inglés.
- El título empieza con el tipo del cambio (`Fix`, `Bug`, `Improvement`,
  `Feature`…) y solo el título lo lleva: `Fix: dock reappears in focus mode`.
- El cuerpo, si hace falta, tiene como máximo dos párrafos.
- Ningún trailer: ni `Co-Authored-By` ni `Claude-Session` ni enlaces a la
  sesión. No subas ninguna sesión.
