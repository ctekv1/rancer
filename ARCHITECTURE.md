# Rancer Architecture

## Overview

Rancer is a high-performance digital art application built in Rust with a raster-based layered canvas. Uses SDL2 for window management, glow (OpenGL) for rendering, and egui for the UI.

## Module Overview

```
src/
├── lib.rs             # Library entry point, re-exports
├── main.rs            # Binary entry point
├── app.rs             # AppState — owns canvas, active tool, undo history
├── events.rs          # AppEvent enum (Press, Drag, Release, Key, Quit)
├── canvas.rs          # Core data model (Canvas, Layer, RasterImage, Color, DirtyRect)
├── commands.rs        # Command pattern for undo/redo (CanvasCommand enum)
├── compositor.rs      # Pixel compositing engine (Compositor, CompositeResult)
├── renderer.rs        # OpenGL rendering (CanvasRenderer: shaders, VAO, texture, draw)
├── export_ui.rs       # Native file dialogs for save/export
├── logger.rs          # File + console logging
├── preferences.rs     # User settings persistence (TOML)
├── gl_loader.rs       # OpenGL function loader (legacy, unused)
│
├── brush/             # CPU dab-based brush engine
│   ├── mod.rs         # BrushType enum, re-exports
│   ├── dab.rs         # DabMask — pixel-level brush stamp data
│   ├── round.rs       # RoundDab — anti-aliased round dab generation
│   └── engine.rs      # BrushEngine — stamp placement, alpha compositing
│
├── tools/             # Tool trait and implementations
│   ├── mod.rs         # Tool trait, BrushConfig trait, BrushSettings, ToolType
│   └── brush_tool.rs  # BrushTool — paint and eraser modes
│
├── ui/                # egui-based UI
│   ├── mod.rs         # Re-exports
│   ├── state.rs       # UiState — panel visibility, tool/color selection, theme
│   ├── egui_impl.rs   # show_ui(), IconCache, color picker, layer panel
│   ├── icons.rs       # SVG icon loading and caching
│   └── egui_integration.rs  # egui_sdl2 painter + context setup
│
├── window/            # Platform window management
│   ├── mod.rs
│   └── sdl2.rs        # Sdl2App — event loop, render lifecycle, egui pass
│
└── tests (cfg(test) only):
    ├── app_tests.rs              # AppState integration tests
    ├── undo_tests.rs             # Undo/redo command tests
    ├── ui_tests.rs               # UI state and layer operations
    ├── window_tests.rs           # Shader compilation tests
    ├── sdl2_event_tests.rs       # SDL2 → AppEvent mapping
    ├── raster_render_tests.rs    # Render module tests
    └── render_optimization_tests.rs  # Compositor dirty-rect and version tests
```

## Core Data Flow

### Frame Render

```
AppState (canvas, tool, history)
    │
    ▼
Compositor::render(&mut canvas)
    ├── Checks version — skip if unchanged
    ├── Composite all (full) or composite_rect (dirty) → CompositeResult
    └── Returns (CompositeResult, x, y) + consumes dirty rect
    │
    ▼
CanvasRenderer::upload(gl, &composite, x, y)
    ├── Full frame (x=0,y=0,size matches) → glTexImage2D
    └── Partial dirty rect → glTexSubImage2D
    │
    ▼
CanvasRenderer::draw(gl, clear_r, g, b)
    └── Clear viewport → bind texture → draw fullscreen TRIANGLE_FAN quad
    │
    ▼
EguiIntegration::run_and_render(ctx, |ctx| show_ui(...))
    └── egui draws UI overlay on top of canvas
    │
    ▼
gl_swap_window() — present to screen
```

### Input → Canvas

```
SDL2 Event → sdl_event_to_app_event() → AppEvent
    │
    ▼
AppState::handle_event(event)
    ├── egui.handle_event() — let egui consume first (e.g. color picker)
    ├── Press → tool.on_press(canvas, x, y)
    ├── Drag  → tool.on_drag(canvas, x, y)
    ├── Release → tool.on_release(canvas, x, y)
    └── Key   → undo (z), redo (y), tool.on_key(code)
```

## Core Design Patterns

### Canvas Version & Dirty Rect

`canvas.version()` is a monotonically-increasing `u64` incremented on any modification (pixel change, layer add/remove, toggle visibility, set opacity). The `Compositor::render()` checks the version — if unchanged it returns `None` and skips compositing entirely.

`DirtyRect` tracks the bounding box of all changes since last composit. The compositor chooses between `composite_all` (full frame) and `composite_rect` (dirty region only) based on whether the dirty area is less than half the canvas. This avoids full-frame re-composites for small brush strokes.

### Upload vs. Draw Separation

OpenGL rendering is split into two phases:

1. **Upload** (`CanvasRenderer::upload`) — only when compositor produces new data. Uses `glTexImage2D` for full frames or `glTexSubImage2D` for partial dirty-rect uploads.
2. **Draw** (`CanvasRenderer::draw`) — every frame unconditionally. Clears viewport to canvas background color, disables blending (pre-composited pixels), binds texture, draws fullscreen quad via `TRIANGLE_FAN`.

This separation prevents unnecessary GPU transfers on idle frames while avoiding screen flashing.

### Command Pattern for Undo/Redo

All canvas mutations go through the `undo` crate's `Record<CanvasCommand>`. Each `CanvasCommand` variant wraps a struct implementing `undo::Edit` with `edit(target)` / `undo(target)` methods. `redo()` delegates to `edit()` automatically.

Current commands:

| Command | Target | Undo Strategy |
|---------|--------|---------------|
| `AddLayer` | Canvas | Stores insertion index and layer clone; removes on undo |
| `RemoveLayer` | Canvas | Stores removed layer data; inserts back at original index on undo |
| `ToggleVisibility` | Canvas | Stores `was_visible` before toggle; restores on undo |
| `SetOpacity` | Canvas | Stores `old_opacity` before change; restores on undo |

Undo depth is capped at 30 levels (configurable via preferences). The `undo` crate supports branching history.

### Tool Trait

Tools implement `Tool` with `on_press(x, y, &mut Canvas)`, `on_drag(x, y, &mut Canvas)`, `on_release(x, y, &mut Canvas)`, and `on_key(code)`. No downcasting needed — `brush_settings()` and `as_brush_config()` provide optional access to brush-specific configuration.

`BrushConfig` is a separate trait for brush-like tools exposing `set_brush_size`, `set_brush_opacity`, `set_brush_color`, `set_eraser_mode`, `is_eraser`.

### Compositing Model

The compositor uses premultiplied-alpha blend: each visible layer is composited bottom-to-top using `blend_pixel()` which implements the standard over operator with destination-alpha pre-multiplication. Layer opacity is applied as a multiplier on source alpha during blending:

```
out_a = src_a + dst_a * (1 - src_a)
out_rgb = (src_rgb * src_a + dst_rgb * dst_a * (1 - src_a)) / out_a
```

The background color is used as the initial framebuffer. The compositor works in RGBA byte order throughout; conversion to GPU format is handled by `glTexImage2D` with `glow::RGBA` format.

### Brush Engine

CPU-based dab stamping. Each brush type generates a `DabMask` (alpha mask) that is stamped into a `RasterImage` buffer via `BrushEngine::stamp_dab()` with alpha compositing. Currently supports `Round` (antialiased circle via `RoundDab`) and `Square` (filled square) brush types. Dab spacing is controlled by brush size to ensure continuous strokes.

## Module Responsibilities

| Module | Lines | Purpose |
|--------|-------|---------|
| canvas.rs | 581 | Core data model: Canvas, Layer, RasterImage, Color, DirtyRect |
| ui_tests.rs | 556 | UI integration tests |
| ui/egui_impl.rs | 394 | show_ui(), IconCache, color picker, layer panel |
| tools/brush_tool.rs | 361 | BrushTool with paint + eraser modes |
| render_optimization_tests.rs | 361 | Compositor dirty-rect and version caching tests |
| preferences.rs | 336 | User settings (TOML save/load) |
| window/sdl2.rs | 213 | Sdl2App event loop, render lifecycle, egui pass |
| commands.rs | 204 | CanvasCommand enum with AddLayer/RemoveLayer/ToggleVisibility/SetOpacity |
| compositor.rs | 184 | Compositor: version check, full/dirty compositing, blend_pixel |
| renderer.rs | 196 | CanvasRenderer: OpenGL shader compilation, VAO, texture upload, draw |
| logger.rs | 165 | File + console logging with timer instrumentation |
| sdl2_event_tests.rs | 164 | SDL2 → AppEvent mapping tests |
| brush/engine.rs | 157 | BrushEngine: stamp_dab with compositing |
| undo_tests.rs | 150 | Undo/redo command tests |
| app.rs | 141 | AppState: owns canvas, tool, undo history, handle_event |
| ui/icons.rs | 140 | SVG icon loading and caching via resvg |
| app_tests.rs | 127 | AppState integration tests |
| ui/state.rs | 107 | UiState: panel visibility, tool/color selection, theme |
| brush/dab.rs | 91 | DabMask type for brush stamp alpha masks |
| brush/round.rs | 70 | RoundDab: anti-aliased circle dab generation |
| export_ui.rs | 56 | Native file dialogs (via rfd) for save/export |
| ui/egui_integration.rs | 55 | EguiIntegration: SDL2 event forwarding + glow painter |
| tools/mod.rs | 49 | Tool trait, BrushConfig trait, BrushSettings, ToolType |
| lib.rs | 42 | Public module declarations, glow re-export |
| raster_render_tests.rs | 31 | Render module tests |
| window_tests.rs | 26 | Shader source validation tests |
| events.rs | 19 | AppEvent enum (Press, Drag, Release, Key, Quit) |
| brush/mod.rs | 17 | BrushType enum, module re-exports |
| main.rs | 16 | Binary entry point, preferences load, run_app |
| ui/mod.rs | 14 | UI re-exports |
| window/mod.rs | 6 | Window module declaration |

## Dependencies

- **sdl2** (0.38) — Window creation, OpenGL context, input events (`bundled` feature)
- **glow** (0.16) — OpenGL function loading and safe Rust bindings
- **egui-sdl2** (0.2, glow-backend) — egui integration with SDL2 + glow
- **egui_extras** (0.33, svg) — SVG rendering support for egui
- **undo** (0.52) — Generic command pattern for undo/redo
- **image** (0.24) — PNG encoding and SVG loading
- **toml** (0.8) / **serde** (1.0) — Preferences serialization
- **resvg** (0.45) — SVG rasterization for UI icons
- **rfd** (0.15) — Native file dialogs (Linux + Windows)
- **chrono** (0.4) — Timestamps for log entries
- **log** (0.4) / **env_logger** (0.11) — Logging infrastructure
- **dirs** (5.0) — Platform-standard config/data directories
