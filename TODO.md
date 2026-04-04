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
    - [x] Add brush_type field to ActiveStroke
    - [x] Update ActiveStroke::new() to accept brush_type
    - [x] Update Canvas::begin_stroke() to accept brush_type
  - [ ] Phase 2: Stroke Vertex Generation (geometry/stroke.rs)
    - [ ] Refactor generate_stroke_vertex_strip into BrushType dispatcher
    - [ ] Square — keep existing logic unchanged
    - [ ] Round — soft feathered edge via alpha gradient at stroke edges
    - [ ] Spray — scattered dots, density tied to brush size, deterministic seed
    - [ ] Calligraphy — 45° broad-nib effect, width varies with stroke angle
  - [ ] Phase 3: UI (geometry/ui_elements.rs)
    - [ ] Add generate_brush_type_vertices(selected_type: BrushType) at y=225
    - [ ] 4 icon buttons: square, round, spray, calligraphy
    - [ ] Blue selection border on active type
  - [ ] Phase 4: Hit Testing (ui.rs)
    - [ ] Add BrushType(BrushType) variant to UiElement
    - [ ] Add hit-test region y=225-255 for brush type buttons
  - [ ] Phase 5: Preferences (preferences.rs)
    - [ ] Add default_type: String to BrushPreferences
    - [ ] Update Default impl to use "square"
  - [ ] Phase 6: Backend Integration (window_winit.rs, window_gtk4.rs)
    - [ ] Add brush_type: BrushType to app state
    - [ ] Handle UiElement::BrushType clicks
    - [ ] Pass brush_type through begin_stroke() calls
    - [ ] Persist to preferences
  - [ ] Phase 7: Testing
    - [ ] BrushType serialization round-trip
    - [ ] Each brush type generates non-empty vertices
    - [ ] Spray produces scattered vertices (not a solid strip)
    - [ ] Calligraphy width varies with stroke angle
    - [ ] Hit-test for each brush type button
- [x] Layer System - Multiple layers, reorder, visibility toggle
  - [x] Phase 1: Data Model (canvas.rs)
    - [x] Add Layer struct (name, strokes, visible, opacity, locked)
    - [x] Update Canvas struct with layers Vec and active_layer index
    - [x] Add layer methods: add_layer, remove_layer, set_active_layer, move_layer
    - [x] Add layer utility: toggle_visibility, set_opacity, clear_layer
    - [x] Update commit_stroke to commit to active layer
    - [x] Update undo/redo to work per-layer
    - [x] Add MAX_LAYERS constant (20)
  - [x] Phase 2: Renderer Updates
    - [x] Update WGPU renderer to iterate layers bottom-to-top
    - [x] Skip invisible layers in render loop
    - [x] Apply layer opacity as alpha multiplier
    - [x] Render active stroke on active layer
    - [x] Update OpenGL renderer with same logic
  - [x] Phase 3: Backend Integration (window_winit.rs)
    - [x] Add active_layer_index to WindowApp state
    - [x] Update stroke commit flow to use active layer
    - [x] Add layer state persistence in preferences
  - [x] Phase 4: Backend Integration (window_gtk4.rs)
    - [x] Add active_layer_index to WindowApp state
    - [x] Update stroke commit flow to use active layer
  - [x] Phase 5: Layer Panel UI (geometry.rs)
    - [x] Add generate_layer_panel_vertices function
    - [x] Add generate_layer_row_vertices function
    - [x] Add generate_add_layer_button_vertices
    - [x] Add generate_delete_layer_button_vertices
    - [x] Add generate_layer_visibility_toggle_vertices
  - [x] Phase 6: Layer Panel UI (ui.rs)
    - [x] Add LayerPanel, LayerRow, AddLayer, DeleteLayer, LayerVisibilityToggle variants
    - [x] Add hit_test for layer panel elements
  - [x] Phase 7: Testing
    - [x] Add unit tests for layer CRUD operations
    - [x] Test undo/redo with multiple layers
    - [x] Test layer visibility toggle
  - [x] Known Issues
    - [x] ~~Layer rendering order inverted~~ (fixed v0.0.7)
    - [x] ~~Active stroke renders on top of all layers~~ (fixed v0.0.7)
- [ ] Selection Tool - Rectangular selection with move/copy
- [ ] Transform Tools - Scale, rotate, flip canvas/strokes

## Tier 3 - File Management (v0.0.8)

- [ ] Project Format - Save/load .rancer files (JSON/bincode)
- [ ] Image Import - Open PNG/JPG as background layer
- [ ] Multiple Export - Different formats (JPEG, WebP, SVG)
- [ ] Auto-save - Periodic backup to prevent data loss

## Tier 4 - Advanced Features (v0.0.9+)

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

## Structural Refactoring (v0.0.7)

- [x] Split `geometry.rs` (2095 lines) into 3 files: `mod.rs`, `stroke.rs`, `ui_elements.rs`
- [x] Refactored `renderer.rs` (1129 → 477 lines): `RenderFrame` pattern, eliminated duplicated state
- [x] Refactored `opengl_renderer.rs` (444 → 276 lines): `GlRenderFrame` pattern, batched UI rendering
- [x] Refactored `window_gtk4.rs` (1222 → ~1030 lines): Consolidated `GlRenderState`, debounced saves
- [x] Refactored `window_winit.rs` (~1180 → ~1035 lines): Extracted handler methods, consolidated state
- [x] Fixed duplicate `#[test]` attribute in `canvas.rs`
