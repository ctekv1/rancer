# Rancer
A digital art application built in Rust, targeting desktop (Linux + Windows) with a future web/WASM build.

## Current state
Core canvas data model is complete and tested:
- Canvas struct with stroke management and undo/redo
- ActiveStroke system (begin, add points, commit)
- ColorPalette with 10 default colors and custom color support
- 13 passing tests

## Next task
Add a basic window using winit that opens on launch and 
accepts mouse input to draw strokes on the canvas.

## Stack
- Rust stable
- winit (window management) — adding now
- wgpu (GPU rendering) — next after window
- Tauri v2 (UI shell) — later

## Build commands
- `cargo build`
- `cargo test`
- `cargo clippy`

## Conventions
- Write tests for all public functions
- Prefer explicit error handling over unwrap
- Keep crates small and focused