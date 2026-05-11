# ADR 002: Extract GL rendering into a `CanvasRenderer` module

## Status

Accepted (2026-05-10)

## Context

`Sdl2App` in `src/window/sdl2.rs` was a single struct handling window creation, event loops, SDL2 lifecycle, egui integration, and all OpenGL rendering (shader compilation, VAO creation, texture management, draw calls). At 379 lines it violated the single-responsibility principle and made the GL code hard to test independently.

The rendering path had two distinct concerns:
1. **Texture upload** — uploading composite pixel data to an OpenGL texture (conditional: only when compositor produces new data)
2. **Frame draw** — clearing the viewport and drawing the textured fullscreen quad (unconditional: every frame)

These had different lifetimes and error-handling needs, but were coupled in a single `render()` method.

## Decision

Extract all GL rendering into a new `src/renderer.rs` module with:

- `CanvasRenderer::new(gl, width, height) -> Result<Self>` — compiles shaders, creates texture and VAO
- `CanvasRenderer::upload(gl, composite, x, y)` — uploads pixel data to texture (conditional per frame)
- `CanvasRenderer::draw(gl, clear_r, g, b)` — clears viewport, draws textured quad (every frame)
- `CanvasRenderer::resize(width, height)` — updates internal dimensions
- Private helpers: `create_shader_program`, `create_quad_vao`

Shader string constants (`VERTEX_SHADER`, `FRAGMENT_SHADER`) moved with the module.

## Consequences

- `Sdl2App` shrank from 379 to 206 lines — window/event lifecycle only
- The upload vs. draw split fixed a regression where idle frames skipped all rendering, causing screen flashing
- All GL calls are encapsulated — replacing glow with a different GL backend only touches one module
- `CanvasRenderer::draw` takes `&self` since GL state is external — thread-safety invariant is clear
- Shader tests in `window_tests` now import from `renderer::` instead of `sdl2::`
