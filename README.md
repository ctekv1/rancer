# Rancer

A digital art application built in Rust with cross-platform support.

**Version:** 0.0.7  
**License:** [GNU GPL-3.0](LICENSE)

## Features

- **GPU-accelerated rendering** — WGPU 28.0 with MSAA support, OpenGL for Linux
- **Layer system** — Multiple layers with visibility toggle, opacity, lock, reorder (up to 20 layers)
- **Zoom & Pan** — Mouse wheel zoom toward cursor, space+drag panning, zoom UI buttons
- **HSV color picker** — Three sliders with click-and-drag, custom saved colors palette (FIFO, max 10)
- **Brush types** — Square, Round (soft-edged), Spray (scattered dots), Calligraphy (45° broad-nib)
- **Brush tools** — Adjustable size (3/5/10/25/50px), opacity presets (25%/50%/75%/100%), eraser toggle
- **Selection tool** — Rectangular selection with move/copy, marching ants animation, whole-stroke capture
- **Undo/Redo** — Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y
- **Native export** — File save dialog, OS notifications, stroke bounding box export (up to 4096×4096)
- **Cross-platform** — winit/WGPU for Windows, GTK4/OpenGL for Linux
- **Auto-saving preferences** — TOML config with platform-specific storage
- **Frame rate limiter** — User-configurable FPS cap (default 60, 0=unlimited, Windows only)
- **Performance optimizations** — Committed stroke vertex caching, UI vertex caching
- **Raster layers** — Bitmap layer support (in progress)

## Build & Run

```bash
# Linux (GTK4)
sudo apt install libgtk-4-dev
cargo build
cargo run

# Windows
cargo build
cargo run
```

## Usage

| Action | Control |
|--------|---------|
| Draw | Left click and drag |
| Eraser | Right-click (hold), or press E to toggle |
| Pan | Space + drag |
| Zoom | Mouse wheel (toward cursor), or +/− buttons |
| Brush size | Click size boxes, or +/− keys |
| Brush type | Click type buttons (square, round, spray, calligraphy) |
| Selection tool | Click tool button, drag on canvas to select |
| Move selection | Click and drag inside selection |
| Copy selection | Ctrl+drag inside selection |
| Commit selection | Delete key |
| Clear selection | Escape or Ctrl+D |
| Undo | Ctrl+Z |
| Redo | Ctrl+Y or Ctrl+Shift+Z |
| Clear canvas | Ctrl+Delete |
| Export | Click export button, or press S |
| Navigate colors | Arrow Up/Down |

## Architecture

```
src/
├── canvas.rs          — Core data model: Stroke, Layer, Canvas, ActiveStroke, BrushType
│                     RasterImage, RasterLayer, LayerContent (raster layer support)
├── renderer.rs        — WGPU rendering (Windows): uses RenderFrame pattern
│                     raster_texture_cache, raster_pipeline (infrastructure)
├── opengl_renderer.rs — OpenGL rendering (Linux): uses GlRenderFrame pattern
├── geometry/          — Vertex generation for strokes and UI elements
│   ├── mod.rs         — Shared utilities (hex_to_color, generate_rect, hsv_to_rgb)
│   ├── stroke.rs      — Stroke vertex generation (Square, Round, Spray, Calligraphy)
│   └── ui_elements.rs — UI element vertex generation (sliders, buttons, layer panel)
├── ui.rs              — Shared hit-testing logic across backends
├── window_winit.rs    — Windows backend: winit event loop, input handling
├── window_gtk4.rs     — Linux backend: GTK4 + OpenGL
├── window_backend.rs  — Shared trait for window backends
├── export.rs          — PNG export with bounding box computation
│                     render_selection_region (raster selection support)
├── export_ui.rs       — Export dialog helpers, OS notifications
├── preferences.rs     — TOML-based config with platform-specific paths
├── gl_loader.rs       — OpenGL function loader for Linux
├── logger.rs          — Logging to file and stdout
└── main.rs            — Entry point, platform detection
```

## Tech Stack

- Rust 1.94+
- WGPU 28.0 (GPU rendering — Windows)
- winit 0.30 (window management — Windows)
- GTK4 0.9 (window/UI — Linux)
- OpenGL/glow 0.14 (GPU rendering — Linux)
- image 0.24 (PNG export)
- rfd 0.15 (native file dialogs)
- chrono 0.4 (timestamps)
- dirs 5.0 (platform-specific config directories)
- serde + toml (serialization)

## Configuration

- **Windows:** `%APPDATA%\rancer\config.toml`
- **Linux:** `~/.config/rancer/config.toml`

### Adjustable Settings

```toml
[renderer]
max_fps = 60        # Frame rate limit (0 = unlimited)
msaa_samples = 4    # Anti-aliasing samples
```

- `max_fps`: Limits render rate to save power on laptops. Set to 0 for unlimited. (Currently Windows only)
- `msaa_samples`: MSAA level (1, 2, 4, 8, 16). Higher = smoother lines but more GPU usage.

## Status

- [x] Cross-platform window backends (winit + GTK4)
- [x] GPU-accelerated rendering (WGPU + OpenGL)
- [x] User preferences system with auto-save
- [x] HSV color picker with custom colors
- [x] Brush opacity control
- [x] Undo/Redo system
- [x] Eraser tool
- [x] Canvas clear
- [x] Export with native file dialog
- [x] Zoom & Pan
- [x] Layer system (visibility, opacity, lock, reorder)
- [x] MSAA (WGPU backend)
- [x] Export captures full canvas content (bounding box)
- [x] Brush types (square, round, spray, calligraphy) with preference persistence
- [x] Selection tool (rectangular selection with move/copy, marching ants animation)
- [x] Frame rate limiter (configurable via config)
- [ ] Transform tools (scale, rotate, flip)
- [x] Raster pixel-edge selection (COMPLETE - Phases 1-3)
  - Data structures: RasterImage, RasterLayer, LayerContent enum
  - Selection stores bitmap, extracted on begin_selection
  - commit_selection_to_raster() method
  - Raster layer infrastructure ready (placeholder render)

## License

This project is licensed under the GNU General Public License v3.0 — see the [LICENSE](LICENSE) file for details.
