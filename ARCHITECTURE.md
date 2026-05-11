# Rancer Architecture

## Overview

Rancer is a high-performance digital art application built in Rust with a raster-based layered canvas. The current stack uses SDL2 for window management, glow (OpenGL) for rendering, and egui for the UI.

## Module Overview

```
src/
├── lib.rs             # Library entry point, re-exports
├── main.rs            # Binary entry point
├── app.rs             # AppState — owns canvas, active tool, undo history
├── canvas.rs          # Core data model (layers, raster images, version)
├── commands.rs        # Command pattern for undo/redo (AddLayer, RemoveLayer, etc.)
├── compositor.rs      # Pixel compositing engine (composite_all, composite_rect)
├── renderer.rs        # OpenGL rendering (CanvasRenderer: shaders, VAO, texture, draw)
├── events.rs          # AppEvent enum (Press, Drag, Release, Key, Quit)
├── export_ui.rs       # Native file dialogs for save/export
├── logger.rs          # File + console logging
├── preferences.rs     # User settings persistence (TOML)
│
├── brush/             # Brush engine
│   ├── mod.rs
│   ├── dab.rs         # DabMask — pixel-level brush stamp data
│   ├── round.rs       # RoundDab — anti-aliased round dab generation
│   └── engine.rs      # BrushEngine — line rasterization, stamp placement
│
├── tools/             # Tool trait and implementations
│   ├── mod.rs         # Tool trait, BrushSettings, ToolType enum
│   └── brush_tool.rs  # BrushTool — paint and eraser modes
│
├── ui/                # egui-based UI
│   ├── mod.rs         # Re-exports
│   ├── state.rs       # UiState — panel visibility, tool/color selection, theme
│   ├── egui_impl.rs   # show_ui(), IconCache, color picker
│   ├── icons.rs       # SVG icon loading and caching
│   └── egui_integration.rs  # egui_sdl2 painter + context setup
│
├── window/            # Platform window management
│   ├── mod.rs
│   └── sdl2.rs        # Sdl2App — event loop, render lifecycle, egui pass
```

## Core Data Flow

### Frame Render

```
AppState (canvas, tool, history)
    │
    ▼
Compositor::render(&mut canvas)
    ├── Checks version — skip if unchanged
    ├── Composite all/partial layers → CompositeResult
    └── Returns (CompositeResult, x, y) dirty rect position
    │
    ▼
CanvasRenderer::upload(gl, &composite, x, y)
    └── tex_image_2d / tex_sub_image_2d onto OpenGL texture
    │
    ▼
CanvasRenderer::draw(gl, clear_r, g, b)
    └── Clear viewport → bind texture → draw fullscreen quad
    │
    ▼
egui::run_and_render(ctx, |ctx| show_ui(...))
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
    ├── Press → tool.on_press(canvas, x, y)
    ├── Drag  → tool.on_drag(canvas, x, y)
    ├── Release → tool.on_release(canvas) + commit to undo record
    └── Key   → undo/redo, tool shortcuts
```

## Core Design Patterns

### Canvas Version

`canvas.version()` is a monotonically-increasing `u64` that increments on any modification (pixel change, layer add/remove, toggle visibility, set opacity). The `Compositor` checks version to skip re-compositing when nothing changed.

### Compositor Dirty-Rect

The `Compositor` struct tracks dirty rectangles across frames. Only the changed region is re-composited and uploaded to the GPU (via `glTexSubImage2D`), avoiding full-frame re-renders.

### Upload vs. Draw Separation

OpenGL rendering is split into two phases:

1. **Upload** (`CanvasRenderer::upload`) — only when compositor produces new data. Handles full vs. partial texture upload.
2. **Draw** (`CanvasRenderer::draw`) — every frame unconditionally. Clears viewport, binds texture, draws fullscreen quad.

This separation was critical: conditional upload prevents unnecessary GPU transfers, while unconditional draw prevents screen flashing on idle frames.

### Command Pattern for Undo/Redo

All canvas mutations go through the `undo` crate's `Record<CanvasCommand>`. Each command implements `edit()`, `undo()`, and `redo()`. Supported commands:

- `AddLayer` — inserts a new layer, records insertion index for correct undo
- `RemoveLayer` — removes layer at index, stores layer data for restore
- `ToggleVisibility` — flips layer visibility
- `SetOpacity` — changes layer opacity, stores previous value

### Tool Trait

Tools implement a common trait with methods for `on_press`, `on_drag`, `on_release`, plus `brush_settings()` / `set_brush_color()`. No downcasting is needed — the trait exposes `set_eraser_mode()` / `is_eraser()` as default methods.

## Module Responsibilities

| Module | Lines | Purpose |
|--------|-------|---------|
| canvas.rs | 581 | Data model: layers, RasterImage, version, dirty rect |
| ui_tests.rs | 513 | UI integration tests |
| ui/egui_impl.rs | 371 | egui UI rendering, color picker, IconCache |
| tools/brush_tool.rs | 361 | BrushTool with paint + eraser modes |
| preferences.rs | 336 | User settings (TOML save/load) |
| commands.rs | 216 | Undo/redo command implementations |
| window/sdl2.rs | 208 | Window, event loop, frame lifecycle |
| renderer.rs | 196 | OpenGL shaders, texture, quad rendering |
| brush/engine.rs | 181 | BrushEngine: line rasterization |
| compositor.rs | 184 | Pixel compositing engine, CompositeResult |
| app.rs | 141 | AppState: owns canvas, tool, undo |
| ui/icons.rs | 140 | SVG icon caching |
| ui/state.rs | 103 | UiState: panels, tool, color, theme |

## Dependencies

- **sdl2** — Window creation, OpenGL context, input events
- **glow** — OpenGL function loading and safe Rust bindings
- **egui** / **egui_sdl2** — Immediate-mode GUI
- **undo** — Generic command pattern for undo/redo
- **image** — PNG encoding and SVG loading
- **toml** / **serde** — Preferences serialization
- **resvg** / **usvg** — SVG rasterization for UI icons
