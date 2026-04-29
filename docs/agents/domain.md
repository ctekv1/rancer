# Domain docs

This repo uses a **single-context** layout.

## Layout

- `CONTEXT.md` at the repo root — domain language, concepts, and project overview
- `docs/adr/` at the repo root — architectural decision records

## Consumer rules

Skills that read domain docs (`improve-codebase-architecture`, `diagnose`, `tdd`) should:

1. Read `CONTEXT.md` at the repo root to understand domain language
2. Read relevant ADRs from `docs/adr/` to understand past decisions
3. If `CONTEXT.md` does not exist, proceed without it (the skill will still work)
