# Rancer Architecture

## Overview

Rancer is a high-performance digital art application built in Rust with GPU-accelerated rendering. It supports cross-platform drawing with a layered canvas, multiple brush types, and PNG export.

The application runs on:
- **Windows**: winit for window management + WGPU for rendering
- **Linux**: GTK4 for window management + OpenGL for rendering

## Module Overview

```
src/lib.rs (main library exports)
├── canvas.rs          # Core data model (layers, strokes, undo/redo)
├── export.rs          # PNG export with automatic bounding box
├── geometry/          # Pure math for vertex generation
│   ├── mod.rs        # Shared utilities
│   ├── stroke.rs    # Stroke mesh generation
│   └── ui_elements.rs # UI button/panel vertices
├── renderer.rs        # WGPU rendering pipeline (Windows)
├── opengl_renderer.rs # OpenGL rendering (Linux)
├── ui.rs             # UI hit testing (platform-shared)
├── preferences.rs    # User settings persistence
├── logger.rs         # Logging utilities
├── window_backend.rs  # Window trait definition
├── window_winit.rs  # Windows window handling
├── window_gtk4.rs   # Linux window handling  
├── export_ui.rs     # Native file dialogs
└── gl_loader.rs    # OpenGL function loader
```

## Core Modules

### canvas.rs (2300 lines)

The central data model containing:

- **Layers**: Stack of vector layers, each with strokes, visibility, opacity, lock state
- **Strokes**: Points with color, size, opacity, brush type
- **ActiveStroke**: In-progress stroke being drawn
- **Selection**: Rectangle selection with move/copy support
- **Undo/Redo**: History stacks for all canvas operations
- **Version tracking**: Auto-incrementing version for cache invalidation

Key types:
```rust
pub struct Canvas { /* layers, history, version */ }
pub struct Layer { name, visible, opacity, locked, content: LayerContent }
pub struct Stroke { points, color, size, opacity, brush_type }
pub enum BrushType { Square, Round, Spray, Calligraphy }
```

### geometry/ (1400 lines across 3 files)

Pure mathematical functions that generate vertex data for rendering. Contains no GPU or window dependencies.

- **stroke.rs**: ConvertsStroke points to triangle mesh vertices
- **ui_elements.rs**: Generates vertices for all UI buttons/panels
- **mod.rs**: Shared utilities (rect generation, color conversion)

Each function takes raw data and returns `Vec<f32>` vertices in format `[x, y, r, g, b, a]`.

### export.rs (575 lines)

PNG export with automatic bounding box calculation:
1. Compute bounding box from all strokes (with padding)
2. Cap at maximum dimensions (4096x4096)
3. Render strokes to in-memory image
4. Save as PNG via `image` crate

### renderer.rs (1400 lines)

WGPU-based rendering for Windows. Key patterns:

- **RenderFrame**: Stateless - all data passed via struct
- **Stroke cache**: GPU vertex buffer cached by canvas version
- **UI cache**: Vertex cache keyed by UI state (hue, brush size, etc.)

The renderer owns no application state - it only owns GPU resources.

### ui.rs (530 lines)

Shared UI hit testing for both platforms:
- Maps screen coordinates to UI elements
- Returns `UiElement` enum for action handling
- Used by both winit and GTK4 backends

## Platform-Specific Components

### Windows (winit + WGPU)

- **window_winit.rs** (1285 lines): Window creation, event loop, input handling
- **renderer.rs**: WGPU initialization and rendering

Data flow:
```
winit events → WindowApp → Canvas → RenderFrame → WGPU → present()
```

### Linux (GTK4 + OpenGL)

- **window_gtk4.rs** (1300+ lines): GTK4 application, widget, event handling
- **opengl_renderer.rs**: OpenGL rendering pipeline

Data flow:
```
GTK4 signals → WindowApp → Canvas → GlRenderFrame → OpenGL → flush()
```

### Window Backend Trait

`window_backend.rs` defines a common interface:
```rust
pub trait WindowBackend {
    fn init(&mut self) -> Result<()>;
    fn run(&self);
    fn canvas(&self) -> &Rc<RefCell<Canvas>>;
    fn mouse_position(&self) -> Point;
    fn mouse_state(&self) -> MouseState;
}
```

Currently the trait is thin (42 lines) - the two backends have significant platform-specific code.

## Key Design Patterns

### RenderFrame Pattern

The rendering is stateless. Each frame receives all data via `RenderFrame`:

```rust
pub struct RenderFrame<'a> {
    pub canvas: &'a Canvas,
    pub active_stroke: Option<&'a ActiveStroke>,
    pub ui: UiRenderState<'a>,
    pub viewport: ViewportState,
}
```

Benefits:
- Renderer owns no application state
- Easy to recreate pipelines on resize
- UI cache keyed by render state

### Canvas Version

`canvas.version()` auto-increments on any modification:
- Add/remove/modify strokes
- Layer operations
- Selection changes
- Clear/undo/redo

Used to invalidate GPU caches efficiently.

### ActiveStroke

In-progress stroke separate from committed strokes:
- Added to canvas during mouse drag
- Rendered in real-time with fresh vertices
- Only commits to layer (and increments version) on mouse release

### Layer Content

Layers support two content types (for future raster layers):
```rust
pub enum LayerContent {
    Vector(Vec<Stroke>),    // Current: stroke list
    Raster(RasterImage),    // Future: pixel data
}
```

## Adding New Features

### New Brush Type

1. Add variant to `canvas::BrushType` enum
2. Implement stroke generation in `geometry/stroke.rs::stroke_to_mesh_*`
3. Add test in `geometry/stroke::tests`
4. Add UI button in `geometry/ui_elements.rs::generate_brush_type_vertices`

### New UI Element

1. Define hit test in `ui.rs::hit_test()`
2. Handle action in window backends
3. Add vertex generation in `geometry/ui_elements.rs`
4. Add test in `geometry/ui_elements::tests`

### New Export Format

Add to `export.rs`. The current export is PNG-specific, but the pattern supports other formats via the `image` crate.

## File Statistics

| File | Lines | Purpose |
|------|-------|---------|
| canvas.rs | 2300 | Data model |
| window_gtk4.rs | 1300 | Linux window |
| window_winit.rs | 1285 | Windows window |
| renderer.rs | 1400 | WGPU rendering |
| geometry/ui_elements.rs | 1809 | UI vertices |
| geometry/stroke.rs | 700+ | Stroke mesh |
| export.rs | 575 | PNG export |
| ui.rs | 530 | Hit testing |

## Dependencies

Core dependencies (non-platform):
- winit (0.30) - Window creation
- wgpu (28.0) - GPU rendering
- tokio - Async runtime
- image (0.24) - PNG encoding
- toml - Preferences

Linux-only:
- gtk4 (0.9) - GTK4 bindings
- glow (0.14) - OpenGL wrapper
- rfd (0.15) - Native dialogs

Windows-only:
- rfd (0.15) - Native dialogs