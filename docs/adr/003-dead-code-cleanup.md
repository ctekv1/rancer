# ADR 003: Remove dead code and relocate CompositeResult

## Status

Accepted (2026-05-11)

## Context

After the compositor and renderer extractions (ADR 001, ADR 002), several dead-code remnants remained:

- `src/viewport.rs` — `Viewport` struct (zoom/pan/coordinate transforms) was defined but never instantiated or imported anywhere in application code. No struct field, no `use crate::viewport` existed. Only the module declaration in `lib.rs` kept it compiled.
- `CompositeResult` was defined in `src/canvas.rs` but only used by `compositor.rs` and `renderer.rs`. It is semantically the output type of the compositor — its natural home is with the compositing logic.
- `src/ui/state.rs` imported `Tool` from `crate::tools` but the compiler flagged it unused — the trait methods called (`as_brush_config`) resolved through other imports.

## Decision

1. **Delete** `src/viewport.rs` and remove `pub mod viewport` from `lib.rs`. The Viewport struct may be re-added when zoom/pan is actually implemented.
2. **Move** `CompositeResult` struct definition from `canvas.rs` to `compositor.rs`. Update imports: `renderer.rs` now imports from `crate::compositor::CompositeResult` instead of `crate::canvas::CompositeResult`.
3. **Remove** the unused `Tool` import in `ui/state.rs`.

## Consequences

- -72 lines of dead code eliminated
- `CompositeResult` lives next to the module that produces it — better cohesion
- Zero warnings, 117 tests pass
- When zoom/pan needs implementing, `Viewport` can be recreated with current understanding
