# Rancer
A digital art application built in Rust, targeting desktop (Linux + Windows) with a future web/WASM build.

## Project goals
- High performance canvas with brush/drawing tools
- Cross-platform: Linux primary, Windows secondary
- AI coaching features planned for later (not now)

## Stack
- Rust (stable)
- Tauri v2 (UI shell — to be added later)
- wgpu (GPU rendering — to be added later)

## Current focus
Building the core canvas and drawing engine. No AI or UI framework yet.

## Build commands
- `cargo build` — build the project
- `cargo test` — run tests
- `cargo clippy` — lint

## Conventions
- Write tests for all public functions
- Keep crates small and focused
- Prefer explicit error handling over unwrap