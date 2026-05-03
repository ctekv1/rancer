# Domain Docs — Consumer Rules

This repo uses a **single-context** layout.

## Layout

- `CONTEXT.md` at the repo root — domain language, concepts, and project overview
- `docs/adr/` at the repo root — architectural decision records
- `docs/agents/` — agent-specific config (this folder)

## Consumer Rules

Skills that read domain docs (`improve-codebase-architecture`, `diagnose`, `tdd`) should:

1. Read `CONTEXT.md` at the repo root to understand domain language
2. Read relevant ADRs from `docs/adr/` to understand past decisions
3. If `CONTEXT.md` does not exist, proceed without it (the skill will still work)

## Current Status

- [ ] `CONTEXT.md` — not yet created (planned)
- [ ] `docs/adr/` — not yet created (planned)
- [x] `docs/agents/domain.md` — exists (this file)
- [x] `docs/agents/issue-tracker.md` — created
- [x] `docs/agents/triage-labels.md` — created
