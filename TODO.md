# Rancer Roadmap

## Completed (v0.0.7)

- [x] Zoom & Pan - Mouse wheel zoom toward cursor, space+drag panning, zoom buttons

## Completed (v0.0.6)

- [x] Custom Color Picker - HSV picker with sliders and custom saved colors (FIFO palette)
- [x] Brush Opacity Control - Slider or presets (25%, 50%, 75%, 100%)
- [x] Keyboard Shortcuts - Eraser toggle (E), brush size (+/-)
- [x] Undo/Redo UI - Visual buttons or status indicator
- [x] Canvas Clear - Button or shortcut to clear canvas
- [x] Export Canvas - PNG export button in UI + keyboard shortcut (S)
- [x] Dead code cleanup - Removed ColorPalette, CanvasExport, unused functions (~200 lines)
- [x] Dependency audit - Removed unused `futures`, trimmed `tokio` to `rt-multi-thread`
- [x] CI pipeline - Test count corrected, Linux + Windows workflows

## Tier 2 - Professional Features (v0.0.7+)

- [x] Zoom & Pan - Mouse wheel zoom, space+drag pan, zoom buttons
  - [x] Phase 1: Shader Changes
    - [x] Add zoom/pan uniforms to WGPU shader (render.wgsl)
    - [x] Add zoom/pan uniforms to OpenGL shader (opengl_renderer.rs)
    - [x] Update vertex transform to apply zoom/pan
    - [x] Adjust line width by zoom factor (cancelled - not needed for MVP)
  - [x] Phase 2: Renderer State
    - [x] Add zoom/pan fields to WGPU Renderer struct
    - [x] Add zoom/pan fields to OpenGL Renderer struct
    - [x] Add set_zoom/set_pan public methods to both renderers
  - [x] Phase 3: Mouse Coordinate Transform
    - [x] Transform mouse coords in window_winit.rs (drawing)
    - [x] Transform mouse coords in window_gtk4.rs (drawing)
    - [x] Transform canvas hit test coords in ui.rs (cancelled - not needed for Approach A: UI stays fixed)
  - [x] Phase 4: Zoom/Pan Input Handlers
    - [x] Handle mouse wheel zoom in window_winit.rs
    - [x] Handle mouse wheel zoom in window_gtk4.rs
    - [x] Handle space+drag pan in window_winit.rs
    - [x] Handle space+drag pan in window_gtk4.rs
  - [x] Zoom toward mouse cursor position
  - [x] Zoom in/out UI buttons
- [ ] Brush Types - Round, square, spray, calligraphy
- [ ] Layer System - Multiple layers, reorder, visibility toggle
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
  - [ ] Known Issues
    - [ ] Layer rendering order inverted — bottom of list renders on top, needs `.rev()` in `all_strokes()`
- [ ] Selection Tool - Rectangular selection with move/copy
- [ ] Transform Tools - Scale, rotate, flip canvas/strokes

## Tier 3 - File Management (v0.0.8+)

- [ ] Project Format - Save/load .rancer files (JSON/bincode)
- [ ] Image Import - Open PNG/JPG as background layer
- [ ] Multiple Export - Different formats (JPEG, WebP, SVG)
- [ ] Auto-save - Periodic backup to prevent data loss

## Tier 4 - Advanced Features (v0.1.0+)

- [ ] Pressure Sensitivity - Tablet support for size/opacity
- [ ] Smoothing Algorithm - Better stroke interpolation
- [ ] Text Tool - Add text to canvas
- [ ] Filters/Effects - Blur, sharpen, color adjustments
- [ ] Symmetry Drawing - Mirror/kaleidoscope modes

## Technical Debt & Known Issues

- [ ] **MSAA not functional** - `sample_count` hardcoded to 1 in both renderers despite config being 4
  - Windows: `Renderer::new()` ignores `config.msaa_samples`
  - Linux: GLArea not configured for multisampling, no FBO setup in GlRenderer
- [ ] **Windows high-DPI resize** - Black space/content shift on window resize (upstream wgpu issue)
  - Workaround attempted: triple `request_redraw()` + `force_window_repaint()` in `window_winit.rs`
  - See `docs/window-resize-issue.md` for full investigation
- [ ] **Export UX** - Saves silently to Pictures directory with no file picker or confirmation toast
  - Consider using `rfd` crate for native file dialog
