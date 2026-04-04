# Rancer Roadmap

## Completed (v0.0.6)

- [x] Zoom & Pan - Mouse wheel zoom toward cursor, space+drag panning, zoom buttons
- [x] Layer System - Multiple layers, reorder, visibility toggle, opacity, lock
- [x] MSAA - Multisampled rendering with resolve texture (WGPU backend)
- [x] Export UX - Native file save dialog, OS notifications, bounding box export
- [x] Structural refactoring - geometry.rs split, RenderFrame pattern, consolidated state
- [x] Custom Color Picker - HSV picker with sliders and custom saved colors (FIFO palette)
- [x] Brush Opacity Control - Slider or presets (25%, 50%, 75%, 100%)
- [x] Keyboard Shortcuts - Eraser toggle (E), brush size (+/-)
- [x] Undo/Redo UI - Visual buttons or status indicator
- [x] Canvas Clear - Button or shortcut to clear canvas
- [x] Export Canvas - PNG export button in UI + keyboard shortcut (S)
- [x] Dead code cleanup - Removed ColorPalette, CanvasExport, unused functions (~200 lines)
- [x] Dependency audit - Removed unused `futures`, trimmed `tokio` to `rt-multi-thread`
- [x] CI pipeline - Test count corrected, Linux + Windows workflows

## Tier 2 - Professional Features (v0.0.7)

- [ ] Brush Types - Round, square, spray, calligraphy
  - [x] Phase 1: Data Model (canvas.rs)
    - [x] Add BrushType enum (Square, Round, Spray, Calligraphy)
    - [x] Add brush_type field to ActiveStroke and Stroke
    - [x] Update ActiveStroke::new() to accept brush_type
    - [x] Update Canvas::begin_stroke() to accept brush_type
  - [x] Phase 2: Stroke Vertex Generation (geometry/stroke.rs)
    - [x] Refactor generate_stroke_vertex_strip into BrushType dispatcher
    - [x] Add StrokeMesh struct with DrawMode (TriangleStrip / Triangles)
    - [x] Square — keep existing logic unchanged
    - [x] Round — soft feathered edge via 4-vertex ribbon (inner/outer alpha gradient)
    - [x] Spray — scattered dots, density tied to brush size, deterministic seed
    - [x] Calligraphy — 45° broad-nib effect, width varies with stroke angle
  - [x] Phase 3: UI (geometry/ui_elements.rs)
    - [x] Add generate_brush_type_vertices(selected_type: BrushType) at y=225
    - [x] 4 icon buttons: square, round, spray, calligraphy
    - [x] Blue selection border on active type
  - [x] Phase 4: Hit Testing (ui.rs)
    - [x] Add BrushType(BrushType) variant to UiElement
    - [x] Add hit-test region y=225-255 for brush type buttons
  - [x] Phase 5: Preferences (preferences.rs)
    - [x] Add default_type: String to BrushPreferences
    - [x] Update Default impl to use "Square"
  - [x] Phase 6: Backend Integration (window_winit.rs, window_gtk4.rs)
    - [x] Add brush_type: BrushType to app state
    - [x] Handle UiElement::BrushType clicks
    - [x] Pass brush_type through begin_stroke() calls
    - [x] Persist to preferences on click (saves but not loaded on startup)
  - [ ] Phase 7: Testing
    - [x] Each brush type generates non-empty vertices
    - [x] Spray produces scattered vertices (Triangles mode)
    - [x] Calligraphy width varies with stroke angle
    - [x] Hit-test for each brush type button
    - [ ] BrushType serialization round-trip
    - [ ] Brush type preference loaded on startup
- [ ] Selection Tool - Rectangular selection with move/copy
- [ ] Transform Tools - Scale, rotate, flip canvas/strokes

## Tier 3 - File Management (v0.0.9)

- [ ] Project Format - Save/load .rancer files (JSON/bincode)
- [ ] Image Import - Open PNG/JPG as background layer
- [ ] Multiple Export - Different formats (JPEG, WebP, SVG)
- [ ] Auto-save - Periodic backup to prevent data loss

## Tier 4 - Advanced Features (v0.0.10+)

- [ ] Pressure Sensitivity - Tablet support for size/opacity
- [ ] Smoothing Algorithm - Better stroke interpolation
- [ ] Text Tool - Add text to canvas
- [ ] Filters/Effects - Blur, sharpen, color adjustments
- [ ] Symmetry Drawing - Mirror/kaleidoscope modes

## Technical Debt & Known Issues

- [x] ~~**MSAA not functional**~~ — Fixed v0.0.7: MSAA now uses a resolve texture. `config.msaa_samples` (default 4) is respected.
  - Linux: GLArea/OpenGL renderer still lacks multisampling (separate issue)
- [ ] **Windows high-DPI resize** - Black space/content shift on window resize (upstream wgpu issue)
  - Workaround attempted: triple `request_redraw()` + `force_window_repaint()` in `window_winit.rs`
  - See `docs/window-resize-issue.md` for full investigation
  - May require switching graphics backend (SDL2 or raw Win32)
- [x] ~~**Export UX**~~ — Fixed v0.0.7: native file save dialog via `rfd`, OS notifications
- [x] ~~**Export captures only window area**~~ — Fixed v0.0.7: stroke bounding box, max 4096x4096
- [ ] **Round brush rendering** — Soft-edged ribbon draws differently at slow vs fast speeds; needs refinement

## Structural Refactoring (v0.0.7)

- [x] Split `geometry.rs` (2095 lines) into 3 files: `mod.rs`, `stroke.rs`, `ui_elements.rs`
- [x] Refactored `renderer.rs` (1129 → 477 lines): `RenderFrame` pattern, eliminated duplicated state
- [x] Refactored `opengl_renderer.rs` (444 → 276 lines): `GlRenderFrame` pattern, batched UI rendering
- [x] Refactored `window_gtk4.rs` (1222 → ~1030 lines): Consolidated `GlRenderState`, debounced saves
- [x] Refactored `window_winit.rs` (~1180 → ~1035 lines): Extracted handler methods, consolidated state
- [x] Fixed duplicate `#[test]` attribute in `canvas.rs`
