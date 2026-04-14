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

- [x] Raster Pixel-Edge Selection (refinement for Selection Tool) - Phases 1-3
    - [x] Data structures: RasterImage, RasterLayer, LayerContent enum
    - [x] Layer uses LayerContent (vector or raster)
    - [x] Selection stores bitmap field
    - [x] render_selection_region in export.rs
    - [x] WGPU/OpenGL infrastructure ready
    - [x] Selection bitmap extraction on begin_selection
    - [x] commit_selection_to_raster() method
    - [x] Raster layer infrastructure ready (full texture render pending)
    - NOTE: Dead code fields marked with #[allow(dead_code)] - DO NOT REMOVE
      These are infrastructure for Phase 3+ full texture rendering:
      - Renderer::raster_texture_cache / raster_bind_group_cache / raster_sampler / raster_pipeline
      - Renderer::create_raster_texture() method needed for queue.write_texture
      - Update shader.wgsl with vs_textured/fs_textured entry points
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

## Performance

- [x] **Committed stroke vertex cache** — CPU+GPU caching for committed strokes (both WGPU and OpenGL backends). Strokes are immutable after commit, so their vertex data only needs to be computed once.
    - [x] Add `version: u64` to `Canvas` struct (`src/canvas.rs`), incremented by a private `invalidate()` helper on every mutating method
    - [x] Add committed stroke CPU cache to `WgpuRenderer` (`src/renderer.rs`): per-layer `LayerStrokeCache` with `strip_strokes` and `tri_strokes` vectors
    - [x] Add GPU buffer caching to `WgpuRenderer`: persistent `RefCell<Option<wgpu::Buffer>>` buffers reused across frames
    - [x] Add committed stroke CPU cache to `GlRenderer` (`src/opengl_renderer.rs`): same per-layer caching pattern
    - [x] In render loops: skip committed stroke regeneration when `canvas.version() == self.canvas_version_cached`; only the active stroke regenerates every frame
    - Expected impact: eliminates ~138 MB/sec of wasted CPU vertex generation at 60 FPS with 10 round-brush strokes
- [x] **UI vertex caching** — Cache UI element vertices (palette, sliders, buttons). Both WGPU and OpenGL renderers now cache UI vertices and only regenerate when state changes.
- [x] **Profiling instrumentation** — Added `Timer` in logger.rs. Added to renderers: full frame, stroke cache update. Logs timing on Drop.

## Future Performance (v0.0.8+)

Note: Unlike professional tools like Krita (which use CPU for brush rendering), Rancer uses GPU-accelerated rendering. This is ideal for vector-style drawing but may need different strategies for complex procedural brushes.

- [ ] **Level of Detail (LOD) rendering** — When zoomed out significantly, reduce stroke detail to improve performance. Store multiple detail levels per stroke for fast LOD switching.
- [ ] **Multithreaded stroke generation** — Parallelize geometry generation for complex brush types using Rayon or similar. Useful if future brush types become more CPU-intensive.
- [ ] **Stroke LOD cache** — Store multiple detail levels per stroke for fast LOD switching without regeneration.
- [x] **Frame rate limiter option** — Added `max_fps` to preferences (0=unlimited, default 60). Windows uses winit's wait_timeout for rate limiting. (Linux TBD)
- [ ] **Brush type architecture** — If adding complex brushes (particle effects, procedural textures), consider separating GPU-friendly brushes from CPU-parallelized brushes.

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

## Code Quality & Testing (v0.0.8)

- [x] Added tarpaulin coverage config (`.tarpaulin.toml`)
- [x] Added Makefile with common dev commands
- [ ] **Integration Testing** - Platform-specific code requires manual/integration tests:
    - window_gtk4.rs: GTK4 event handling, display rendering
    - window_winit.rs: winit window management
    - opengl_renderer.rs, renderer.rs: GPU rendering paths
    - export_ui.rs: Native file dialogs
  These are excluded from unit test coverage (require display/GPU).
  Use INTEGRATION_TESTS.md checklist for manual verification.
