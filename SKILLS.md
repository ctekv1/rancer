# SKILLS.md

This file provides guidance to agents when working with code in this repository.

## Commands

```bash
# Build
cargo build --release          # Production build
cargo build                    # Debug build
build-windows.bat              # Windows build script (checks Rust, runs release build)

# Test & Lint
cargo test
cargo clippy -- -D warnings

# Linux (requires GTK4 dev libraries)
sudo apt-get install libgtk-4-dev
cargo build --release
```

Domain context lives in `CONTEXT.md` at the repo root.

## Agent skills

### Issue tracker

Issues tracked on GitHub via `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default labels: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout (one CONTEXT.md + docs/adr/ at repo root). See `docs/agents/domain.md`.
