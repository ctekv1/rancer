# PRD: Architecture Deepening — Phase 8

## Problem Statement

After implementing the core rendering pipeline (SDL2 + OpenGL texture rendering, dirty rect optimization, version tracking), the codebase accumulated significant technical debt that made it harder to work with:

1. **Drawing performance dips** during active strokes due to inefficient code paths obscured by dead code and tangled dependencies
2. **Two competing undo systems** causing confusion about which one to use
3. **Brush settings duplicated** between `BrushTool` and `UiState`, requiring manual synchronization that was fragile and hard to extend
4. **Dead `Canvas::Selection` type** with non-functional methods alongside the working `PixelSelection` type
5. **`LayerContent` enum** with only one variant, adding unnecessary indirection throughout the codebase
6. **Public struct fields** on `Canvas` that were accessed directly from tools and tests, making it impossible to change internal representation without breaking everything

## Solution

Systematically clean up the architecture through five targeted deepening passes using TDD. Each pass removed dead code, simplified data flow, or deepened module boundaries — resulting in a codebase where modules are easier to reason about and modify independently.

## User Stories

1. As a developer, I want to modify canvas internals without updating dozens of direct field accesses, so that I can iterate faster on rendering and data structures
2. As a developer, I want a single, clearly-owned undo system, so that I don't waste time figuring out which undo mechanism is active
3. As a developer, I want brush settings accessible through a single trait interface, so that UI and tools stay synchronized without manual `downcast_mut` logic
4. As a developer, I want only the selection type that actually works, so that I don't get confused by dead selection methods on Canvas
5. As a developer, I want `Layer.content` to be a concrete type instead of a single-variant enum, so that I don't write meaningless pattern matches everywhere
6. As a developer, I want zero compiler warnings in the codebase, so that real issues stand out immediately
7. As a user, I want the drawing to feel smooth without performance dips, so that the editor is pleasant to use
8. As a user, I want no screen flashing when opening the app or drawing, so that the experience is safe and professional
9. As a user, I want to see UI panels for tools and layers, so that I can actually use the features beyond drawing

## Implementation Decisions

### 1. Dead Undo System Removal
- Removed `Canvas.undo_stack`, `Canvas.undo()`, `Canvas.redo()`, `Canvas.can_undo()`, `Canvas.can_redo()` — these were never populated and duplicated `AppState.history`
- `GlRenderFrame` now carries `can_undo`/`can_redo` booleans from `AppState` instead of calling Canvas methods
- The `undo` crate's `Record<CanvasCommand>` remains the sole undo system

### 2. LayerContent Enum → Direct RasterLayer
- `Layer.content` changed from `LayerContent` enum to direct `RasterLayer` type
- Removed ~30 pattern matches across `canvas.rs`, `brush_tool.rs`, `selection.rs`, `opengl_renderer.rs`, and test files
- Removed dead vector stroke rendering code in `GlRenderer` that referenced a non-existent `LayerContent::Vector` variant
- Removed `Layer.is_raster()` (always returned `true`) and `LayerContent` type entirely

### 3. Dead Canvas::Selection Removal
- Removed `Selection` struct (~30 lines) and 9 Canvas methods (`begin_selection`, `move_selection`, `copy_selection`, `commit_selection`, `commit_selection_to_raster`, `clear_selection`, `selection()`, `has_selection()`)
- `PixelSelection` in `src/selection.rs` remains the sole working selection type, used by `SelectionTool`

### 4. Duplicated Brush State Consolidation
- Created `BrushSettings` struct (`size`, `opacity`, `color`, `brush_type`) in `tools/mod.rs`
- `Tool` trait gained default methods: `brush_settings()`, `set_brush_size()`, `set_brush_opacity()`, `set_brush_color()`, `set_brush_type()`
- `BrushTool` stores settings as a single `BrushSettings` field instead of 4 separate fields
- `UiState` no longer stores brush settings — it only handles tool selection, layer operations, and panel visibility
- Setting brush values now goes through the trait: `app.active_tool_mut().set_brush_size(25)` instead of manual `downcast_mut` + setter calls
- `ToolType` moved from `ui/state.rs` to `tools/mod.rs` to break the `tools → ui` dependency

### 5. Canvas Encapsulation
- Changed Canvas struct fields from `pub` to `pub(crate)`: `width`, `height`, `background_color`, `layers`, `active_layer`, `version`
- Added getter methods: `width()`, `height()`, `active_layer_index()`, `layers_mut()`
- All external access now goes through methods, enabling future internal changes without breaking public API
- Fixed borrow checker issues in `BrushTool.stamp_at()` and `PixelSelection` methods by hoisting immutable borrows before mutable ones

### 6. Warning Cleanup
- Removed 8 unused imports across 6 files
- Removed 2 dead constants (`MIN_EXPORT_SIZE`, `MAX_EXPORT_SIZE`) from `export.rs`
- Removed 2 unused struct fields (`width`, `height`) from `Sdl2App`
- Fixed 8 unused variables with underscore prefix or removal
- Added `let _ =` for 2 unused `Result` values in `AppState`
- Fixed 1 missing import in test code (`RoundDab` in `brush/engine.rs` tests)

## Testing Decisions

### Good Tests (Principle)
Tests verify behavior through public APIs, not implementation details. They should survive internal refactors because they test what the system does, not how it does it.

### Tests Modified
| Test File | Changes |
|-----------|---------|
| `brush_tool_tests.rs` | Updated to use `canvas.layers()[idx]` instead of direct field access |
| `selection_tests.rs` | Rewrote to use `canvas.layers_mut()[idx]` and `canvas.active_layer_index()` |
| `undo_tests.rs` | Updated to use `canvas.layer_count()` and `canvas.layers()[idx]` |
| `render_optimization_tests.rs` | Simplified pixel mutation to `layer.content.image.set_pixel()` |
| `ui_tests.rs` | Added `brush_settings_read_through_tool_trait` and `brush_settings_write_through_tool_trait` tests; removed old brush-setting tests that tested `UiState` brush fields (now removed) |

### Test Results
- 138 tests pass
- 0 tests fail
- 0 compiler warnings

## Out of Scope

1. **egui integration** — UI panel rendering remains blocked on egui + SDL2 + glow context compatibility. This is a separate initiative.
2. **Vector layers** — The `LayerContent` enum removal was based on the fact that vector support never existed. Adding it would be a future feature requiring a different design.
3. **Drawing performance optimization** — Architecture fixes simplified the code paths, but dedicated performance profiling of `BrushEngine::stamp_dab`, composite cost, and render loop overhead is a separate effort.
4. **Complete UI visibility** — The SDL2 window + OpenGL render loop works, but UI panels are not yet visible to the user.

## Further Notes

- This work followed TDD's red-green-refactor loop: each change was driven by a test that verified observable behavior
- The `Tool` trait is now a deeper module: a simple interface (`on_press`, `on_drag`, `on_release`, `brush_settings()`) backed by substantial implementation
- `Canvas` encapsulation enables future optimizations (e.g., chunked layer storage, GPU-side compositing) without changing external call sites
- The codebase went from 22 compiler warnings to 0, making real issues immediately visible
