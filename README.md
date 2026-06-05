# Rancer

A digital art application built in Rust with cross-platform support.

**Version:** 0.0.7  
**License:** [GNU GPL-3.0](LICENSE)

## Features

- **GPU-accelerated rendering** — OpenGL via glow with SDL2 windowing
- **Layer system** — Multiple layers with visibility toggle, opacity, lock, reorder (up to 20 layers)
- **HSV color picker** — Three sliders with click-and-drag, custom saved colors palette (FIFO, max 10)
- **Brush types** — Round (soft-edged anti-aliased), Square (filled)
- **Brush tools** — Adjustable size, opacity presets (25%/50%/75%/100%), eraser toggle, separate paint/eraser settings
- **Undo/Redo** — Z/Y keyboard shortcuts, UI buttons, command pattern via `undo` crate
- **Native export** — File save dialog via `rfd`, OS notifications
- **Single backend** — SDL2 on both Linux and Windows (no platform `#[cfg]` branching)
- **Auto-saving preferences** — TOML config with platform-specific storage
- **Performance optimizations** — Dirty-rect compositing, version-based caching, partial texture uploads
- **egui UI** — Immediate-mode GUI with SVG icons, theme toggle, color picker, layer panel
- **Raster canvas** — CPU dab-based brush engine stamping into `RasterImage` pixel buffers

## Build & Run

```bash
# Linux
cargo build
cargo run

# Windows
cargo build
cargo run
```

SDL2 is statically compiled via the `bundled` feature — no system library needed.

## Usage

| Action | Control |
|--------|---------|
| Draw | Left click and drag |
| Eraser | Press E to toggle, or set via UI |
| Brush size | Click size buttons, or +/− keys |
| Brush opacity | Click opacity presets (25/50/75/100%) |
| Brush type | Click type buttons (round, square) |
| Undo | Z key (or UI button) |
| Redo | Y key (or UI button) |
| Add layer | Click + button in layer panel |
| Remove layer | Click − button in layer panel |
| Toggle visibility | Click eye icon in layer panel |
| Export | Click export button, or press S |

## Architecture

```
src/
├── app.rs             — AppState: canvas, active tool, undo history
├── canvas.rs          — Core data model: Canvas, Layer, RasterImage, Color, DirtyRect
├── commands.rs        — Command pattern: AddLayer, RemoveLayer, ToggleVisibility, SetOpacity
├── compositor.rs      — Pixel compositing: composite_all, composite_rect, blend_pixel
├── renderer.rs        — CanvasRenderer: OpenGL shaders, VAO, texture upload, draw
├── events.rs          — AppEvent enum (Press, Drag, Release, Key, Quit)
├── export_ui.rs       — Native file dialogs for save/export
├── logger.rs          — File + console logging
├── preferences.rs     — User settings persistence (TOML)
│
├── brush/             — CPU dab-based brush engine
│   ├── mod.rs         — BrushType enum (Round, Square)
│   ├── dab.rs         — DabMask — pixel-level brush stamp data
│   ├── round.rs       — RoundDab — anti-aliased round dab generation
│   └── engine.rs      — BrushEngine — stamp placement, alpha compositing
│
├── tools/             — Tool trait and implementations
│   ├── mod.rs         — Tool trait, BrushConfig trait, BrushSettings
│   └── brush_tool.rs  — BrushTool — paint and eraser modes
│
├── ui/                — egui-based UI
│   ├── mod.rs
│   ├── state.rs       — UiState — panel visibility, tool selection, theme
│   ├── egui_impl.rs   — show_ui(), IconCache, color picker, layer panel
│   ├── icons.rs       — SVG icon loading and caching
│   └── egui_integration.rs  — egui-sdl2 glow-backed integration
│
└── window/
    ├── mod.rs
    └── sdl2.rs        — Sdl2App — event loop, render lifecycle, egui pass
```

See `ARCHITECTURE.md` for detailed data flow and design patterns.

## Tech Stack

- Rust 1.94+
- SDL2 0.38 (window management, GL context, input events)
- glow 0.16 (OpenGL function loading and safe Rust bindings)
- egui-sdl2 0.2 (egui integration with SDL2 + glow backend)
- undo 0.52 (command pattern for undo/redo)
- image 0.24 (PNG encoding, SVG loading)
- resvg 0.45 (SVG rasterization for UI icons)
- rfd 0.15 (native file dialogs)
- serde + toml (serialization)
- chrono 0.4 (timestamps)
- dirs 5.0 (platform-specific config directories)

## Configuration

- **Linux:** `~/.config/rancer/config.toml`
- **Windows:** `%APPDATA%\rancer\config.toml`

## Status

- [x] SDL2 + OpenGL windowing (single cross-platform backend)
- [x] Raster layer canvas (in-memory `RasterImage` layers)
- [x] CPU dab-based brush engine (Round, Square)
- [x] Brush tool with paint + eraser modes
- [x] Undo/Redo via command pattern (AddLayer, RemoveLayer, ToggleVisibility, SetOpacity)
- [x] egui UI with SVG icons, theme toggle, color picker, layer panel
- [x] User preferences system with auto-save
- [x] Export with native file dialog
- [x] Dirty-rect compositing with version caching
- [x] 119 unit/integration tests

## License

This project is licensed under the GNU General Public License v3.0 — see the [LICENSE](LICENSE) file for details.
