# ADR 001: Extract compositing into its own module

## Status

Accepted (2026-05-10)

## Context

`Canvas` originally owned both data (layers, version tracking, dirty rects) and compositing logic (`composite_all()`, `composite_rect()`). The compositing functions were the only consumers of `Layer` and `RasterImage` internals. This mixed responsibility made `Canvas` harder to test and reasoned about.

The `Compositor` was also gaining stateful behavior — dirty-rect tracking and version-checking for incremental updates — which didn't belong in the data model.

## Decision

Extract all compositing logic into a new `src/compositor.rs` module containing:

- A stateless `compositor::composite_all(&Canvas) -> CompositeResult` function
- A stateless `compositor::composite_rect(&Canvas, x, y, w, h) -> CompositeResult` function
- A private `compositor::blend_pixel()` shared by both
- A stateful `Compositor` struct that owns version tracking and dirty-rect lifecycle

The `Canvas` struct retained only its data (layers, version counter, dirty rect as state that `Compositor` consumes).

## Consequences

- `Canvas` is now a pure data model — easier to test, refactor, and serialize
- Compositing has a single source of truth (`blend_pixel`) instead of being duplicated across `Canvas` and drawer code
- `Sdl2App::render_frame` shrank from 45 lines of conditionals to one call: `compositor.render(&mut canvas)`
- The seam is clean enough to swap compositing strategy (e.g., SIMD, compute shader) without touching canvas or window code
