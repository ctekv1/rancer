# Technical Debt Fixes — v0.0.6 / v0.0.7

This document covers the technical debt fixes applied across v0.0.6 and v0.0.7.

---

## v0.0.6 Fixes

### 1. Dead Code Removal
Removed `RancerApp` and `AppConfig` structs from `lib.rs` — never used.

### 2. Canvas Resize Coordinate Mapping
Added `canvas.resize()` in `WindowEvent::Resized` handler.

### 3. Shared UI Hit Detection
Extracted 449 lines of duplicated UI hit detection into `src/ui.rs`.

### 4. Combined Vertex Buffer with Stroke Separation
Combined all stroke vertices into a single GPU buffer, but draw each stroke separately.

### 5. WGPU Error Handling on Windows
Made the Cairo fallback Linux-only via `#[cfg]` attributes.

### 6. Zero-Size Window Guard
Added guard for `(0, 0)` window size during `resumed` phase.

---

## v0.0.7 Structural Refactoring

### 1. Split `geometry.rs` (2095 → 3 files)
- **Before:** Single 2095-line monolith mixing stroke geometry, UI geometry, and shared utilities
- **After:** `geometry/mod.rs` (shared utilities + re-exports), `geometry/stroke.rs` (stroke vertices), `geometry/ui_elements.rs` (UI vertices)
- **Impact:** Zero changes to consumers — `pub use` re-exports maintain backward compatibility

### 2. Refactored `renderer.rs` (1129 → 477 lines)
- **Before:** 15 fields (9 app state + 6 WGPU), 12 setter methods, 12 proxy vertex methods
- **After:** 11 fields (all WGPU internals), 0 setters, 0 proxies
- **New pattern:** `RenderFrame` — single source of truth for render data, passed to `render(&mut self, frame: &RenderFrame)`
- **Impact:** Eliminated all state sync bugs between `WindowApp` and `Renderer`

### 3. Refactored `opengl_renderer.rs` (444 → 276 lines)
- **Before:** `render_hsv` took 12 parameters, 12 proxy vertex methods
- **After:** `render(&self, frame: &GlRenderFrame)` — 1 parameter, batched UI rendering
- **Performance:** 12 GPU uploads per frame → 2 (strokes + batched UI)

### 4. Refactored `window_gtk4.rs` (1222 → ~1030 lines)
- **Before:** ~20 individual `Rc<RefCell<...>>` variables, 17-parameter `setup_mouse_events`
- **After:** Single `GlRenderState` struct, 4-parameter `setup_mouse_events`
- **Performance:** Debounced preference saves (save on close only, not per-event)

### 5. Refactored `window_winit.rs` (~1180 → ~1035 lines)
- **Before:** 16 scattered state fields, deeply nested event handlers
- **After:** `WinitRenderState` struct, extracted `handle_ui_click()`, `handle_keyboard()`, `handle_cursor_moved()` methods
- **New:** `request_redraw()` helper consolidates triple redraw + Windows repaint workaround

### 6. MSAA Fix
- **Before:** `sample_count` hardcoded to 1 despite config specifying 4
- **After:** Multisampled texture created when `sample_count > 1`, resolves to swapchain

### 7. Export Fixes
- **Bounding box:** Export now computes stroke bounding box instead of using window dimensions
- **File dialog:** Replaced auto-save with `rfd` native save dialog
- **OS notifications:** `notify-send` on Linux, console print on Windows
- **Size limits:** Min 100×100, max 4096×4096

### 8. Bug Fixes
- Layer rendering order inverted → fixed with `.enumerate().rev()`
- Active stroke renders on top of all layers → fixed by inserting at active layer position
- Slider drag blocked by drawing state check → fixed by checking for active stroke before returning
- Duplicate `#[test]` attribute on `test_active_stroke_with_opacity`

---

## Test Summary

| Module | Tests (v0.0.7) |
|--------|----------------|
| `canvas::tests` | 38 |
| `geometry::tests` | 32 |
| `export::tests` | 10 |
| `preferences::tests` | 13 |
| `renderer::tests` | 5 |
| `ui::tests` | 16 |
| `logger::tests` | 2 |
| `window_winit::tests` | 8 |
| **Total** | **168** |

All 168 tests pass with no regressions.
