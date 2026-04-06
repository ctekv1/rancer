# Rancer Roadmap

## Completed (v0.0.7)

- [x] Brush Types - Round, square, spray, calligraphy (all phases complete)
- [x] Selection Tool - Rectangular selection with move/copy, marching ants animation

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

- [ ] Raster Pixel-Edge Selection (refinement for Selection Tool)
    - [ ] Replace whole-stroke selection with true pixel-level raster selection
    - [ ] Render strokes to offscreen texture, extract pixels within rect
    - [ ] Selected pixels become movable bitmap overlay
    - [ ] On deselect: convert bitmap back to strokes or keep as layer
    - [ ] Handles partial strokes at boundary with pixel precision
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

- [x] Split `geometry.rs` into 3 files: `mod.rs`, `stroke.rs`, `ui_elements.rs`
- [x] Refactored `renderer.rs`: `RenderFrame` pattern, eliminated duplicated state
- [x] Refactored `opengl_renderer.rs`: `GlRenderFrame` pattern, batched UI rendering
- [x] Refactored `window_gtk4.rs`: Consolidated `GlRenderState`, debounced saves
- [x] Refactored `window_winit.rs`: Extracted handler methods, consolidated state
- [x] Fixed duplicate `#[test]` attribute in `canvas.rs`
