# Rancer GitHub Issues Plan - Phase 4 Raster Rendering

This document contains draft GitHub issues for implementing Phase 4 raster rendering and related infrastructure improvements. Copy each issue block below and create a new GitHub issue with the provided title, description, and labels.

---

## Issue 1: Switch Graphics Backend from WGPU to OpenGL

**Title:**  
`Switch graphics backend from wgpu to OpenGL`

**Description:**

## Summary
Replace the WGPU-based renderer with OpenGL (using the glow library) to resolve window resize rendering issues and unify the graphics pipeline across platforms.

## Motivation
- **Resize issue**: WGPU has known unfixed issues with window resize on Windows causing black space and content shifting (see docs/window-resize-issue.md)
- **Unified architecture**: Krita and other professional drawing apps use OpenGL successfully
- **Lower maintenance**: Single graphics backend instead of wgpu + OpenGL split

## Current State
- Windows: `winit` + `wgpu` (has resize bug, raster layers never rendered due to logic in renderer.rs)
- Linux: `GTK4` + `OpenGL/glow` (working implementation exists)

## Implementation Steps

### 1. Add Dependencies (Cargo.toml)
```toml
[dependencies]
glow = "0.14"

[target.'cfg(target_os = "windows")'.dependencies]
glow = "0.14"  # Add for Windows
```

### 2. Create Unified OpenGL Renderer
- Use existing `gl_loader.rs` pattern for symbol loading
- Convert existing WGSL shaders to GLSL (vertex/fragment)
- Maintain compatibility with existing `RenderFrame`, `ViewportState`, `UiRenderState` interfaces
- Implement texture upload via `gl.tex_image_2d` / `gl.tex_sub_image_2d`

### 3. Window Integration (src/window_winit.rs)
- Replace WGPU surface creation with OpenGL context creation via winit
- Keep existing event handling and input processing
- Update render loop to call OpenGL renderer instead of WGPU

### 4. Remove WGPU
- Remove WGPU dependency from Cargo.toml
- Keep winit for windowing (no change needed)

### 5. Test Checklist
- [ ] Application launches without errors on both Windows and Linux
- [ ] Drawing works (all brush types)
- [ ] Window resize works correctly (no black space or content shifting)
- [ ] Pan/zoom works
- [ ] Export to PNG works
- [ ] Undo/redo works
- [ ] Performance is acceptable (no significant FPS drop)

## Files Affected
- `Cargo.toml` — Remove wgpu, add glow for Windows
- `src/lib.rs` — Update module exports (remove wgpu renderer, add OpenGL renderer)
- `src/window_winit.rs` — Replace WGPU initialization/rendering with OpenGL
- `src/renderer.rs` — Remove or repurpose (may keep as interface)
- New file: `src/gl_renderer.rs` (or reuse/adapt existing opengl_renderer.rs)

## Time Estimate
Medium effort — can use Linux opengl_renderer.rs as template (~1-2 sessions)

## Related Issues
- Fixes: Window resize rendering issue (docs/window-resize-issue.md)
- Enables: Per-layer version tracking (next issue)

**Labels:** `graphics`, `backend`, `infrastructure`  
**Milestone:** v0.0.8

---

## Issue 2: Implement Per-Layer Version Tracking for Efficient Cache Invalidation

**Title:**  
`Implement per-layer version tracking for efficient cache invalidation`

**Description:**

## Summary
Replace the single global canvas version counter with per-layer version tracking to avoid unnecessary full vector stroke cache rebuilds when only raster layers change.

## Motivation
- **Current problem**: Single global `version: u64` increments on any change (even raster paint), forcing full vector stroke cache rebuild every frame
- **Performance impact**: Raster painting becomes sluggish as vector cache rebuilds unnecessarily
- **Solution**: Track versions per layer so only changed layers trigger cache updates

## Current State
In `src/canvas.rs`:
- Line 339: `version: u64` - single global counter
- Lines 816-827 in `renderer.rs`: Raster layers collected but never used due to continue statement
- Phase 4 docs identify this as the first structural issue to fix

## Implementation Steps

### 1. Update Canvas Structure
Add `layer_versions: Vec<u64>` field parallel to `layers: Vec<Layer>`

### 2. Initialize and Maintain Layer Versions
- Initialize in `Default::default()` as `vec![0]` matching initial layer
- Update `add_layer()`, `remove_layer()`, `move_layer()` to keep `layer_versions` in sync
- Add `pub fn layer_version(&self, idx: usize) -> u64` getter
- Add private `fn invalidate_layer(&mut self, idx: usize)` that increments both `version` and `layer_versions[idx]`

### 3. Update Undo System
Change `undo_stack: Vec<(usize, Stroke)>` → `undo_stack: Vec<UndoEntry>` with enum:
```rust
pub enum UndoEntry {
    VectorStroke(usize, Stroke),
    RasterSnapshot { layer_idx: usize, data: Vec<u8> },
}
```

### 4. Add Raster Layer Helpers
- `pub fn raster_layer_mut(&mut self, idx: usize) -> Option<&mut RasterLayer>` — calls `invalidate_layer`
- `pub fn create_raster_layer(&mut self, name: String) -> Result<usize, String>` — inserts transparent RGBA8 layer

### 5. Update Renderer
In `src/renderer.rs`:
- Add `raster_texture_versions: Vec<u64>` to track last uploaded version per layer
- Implement `fn ensure_raster_textures_valid(&mut self, device, queue, canvas: &Canvas)`:
  - Iterates raster layers, calls upload_raster_layer only when version mismatch
  - Resizes caches when layer count changes
- Implement helper functions:
  - `fn build_textured_uniforms(canvas, viewport, raster: &RasterLayer, opacity: f32) -> [f32; 10]`
  - `fn build_raster_quad_vertices(raster: &RasterLayer) -> [[f32; 4]; 6]`
- Refactor `render_wgpu()` to use per-layer ordered dispatch:
  - Call `ensure_raster_textures_valid()` before render pass
  - Iterate layers bottom-to-top:
    * Vector layer: set vector pipeline, draw strip/tri ranges
    * Raster layer: set raster pipeline, bind texture, draw textured quad

## Files Affected
- `src/canvas.rs` — Core data model changes
- `src/renderer.rs` — Update to use per-layer validation
- Tests: Update any tests that depend on undo/version behavior

## Time Estimate
Low-Medium effort — mostly data model updates with clear specifications

## Test Checklist
- [ ] Unit tests pass for canvas operations
- [ ] Undo/redo still works correctly for vector strokes
- [ ] Creating/removing layers maintains correct version counts
- [ ] Raster layer mutations invalidate only that layer's cache
- [ ] Vector layer changes still invalidate vector cache appropriately

## Files Affected
- `src/canvas.rs`
- `src/renderer.rs`

## Time Estimate
Low-Medium effort — focused data model changes

## Related Issues
- Prerequisite for: Raster display (Issue 3)
- Enabled by: OpenGL backend switch (Issue 1)

**Labels:** `canvas`, `performance`, `infrastructure`  
**Milestone:** v0.0.8

---

## Issue 3: Display Raster Layers on Canvas (Phase 4A)

**Title:**  
`Display raster layers on canvas`

**Description:**

## Summary
Make raster layers visible on screen by rendering them as textured quads in the correct layer order with proper opacity and positioning.

## Motivation
- Currently, raster layers created via `commit_selection_to_raster()` are silently dropped in the render loop
- Need to implement the textured quad rendering path for raster layers
- This is the first deliverable of Phase 4 raster rendering

## Current State
- Raster infrastructure exists in `canvas.rs` (RasterImage, RasterLayer, LayerContent::Raster)
- `renderer.rs` has stub fields for raster rendering (`raster_texture_cache`, etc.) marked with `#[allow(dead_code)]`
- In `renderer.rs` lines 817-827: raster layers are collected into `raster_layers_data` but then skipped with `continue`
- No actual raster rendering occurs in the WGPU render pass

## Implementation Steps

### 1. Update Shaders (src/shaders/render.wgsl)
- Add `opacity: f32` to `TexturedUniforms` struct between `zoom` and `pan_offset`
- Update `fs_textured`: `return vec4(tex_color.rgb, tex_color.a * textured_uniforms.opacity);`

### 2. Enhance Renderer (src/renderer.rs for WGPU, src/gl_renderer.rs for OpenGL)
- Add raster texture tracking: `raster_texture_versions: Vec<u64>`
- Implement `fn upload_raster_layer(&mut self, device, queue, layer_idx, raster: &RasterLayer, canvas_layer_version: u64)`:
  1. Create wgpu::Texture (format `Rgba8Unorm`, usage `TEXTURE_BINDING | COPY_DST`)
  2. Call `queue.write_texture` with `raster.image.data`
  3. Create TextureView and BindGroup (uniform buffer + sampler + texture)
  4. Store in cache and set version
- Implement `fn ensure_raster_textures_valid(&mut self, device, queue, canvas: &Canvas)`:
  - Iterates raster layers, calls upload_raster_layer only when version mismatch
  - Resizes caches when layer count changes
- Implement helper functions:
  - `fn build_textured_uniforms(canvas, viewport, raster: &RasterLayer, opacity: f32) -> [f32; 10]`
  - `fn build_raster_quad_vertices(raster: &RasterLayer) -> [[f32; 4]; 6]`
- Refactor `render_wgpu()` to use per-layer ordered dispatch:
  - Call `ensure_raster_textures_valid()` before render pass
  - Iterate layers bottom-to-top:
    * Vector layer: set vector pipeline, draw strip/tri ranges
    * Raster layer: set raster pipeline, bind texture, draw textured quad

### 3. Update Canvas (src/canvas.rs)
- Add `layer_versions: Vec<u64>` field (from Issue 2)
- Add `pub fn raster_layer_mut(&mut self, idx: usize) -> Option<&mut RasterLayer>` that calls `invalidate_layer`

## Files Affected
- `src/shaders/render.wgsl` — Add opacity to textured uniforms
- `src/renderer.rs` — WGPU rendering enhancements
- `src/gl_renderer.rs` — OpenGL rendering enhancements (Linux)
- `src/canvas.rs` — Add raster layer helper method
- `src/window_winit.rs` — No changes needed (uses renderer interface)

## Time Estimate
Medium effort — focused rendering implementation

## Test Checklist
- [ ] Raster layers created via commit_selection_to_raster() appear on canvas
- [ ] Raster layers respect opacity settings
- [ ] Raster layers composite correctly in layer order (over/under vector layers)
- [ ] Pan/zoom works correctly with raster layers
- [ ] Toggling layer visibility works
- [ ] Reordering layers changes composite order correctly
- [ ] Performance impact is minimal (no significant FPS drop when raster layers present)

## Files Affected
- `src/shaders/render.wgsl`
- `src/renderer.rs`
- `src/gl_renderer.rs`
- `src/canvas.rs`

## Time Estimate
Medium effort — rendering implementation

## Related Issues
- Requires: Per-layer version tracking (Issue 2)
- Enables: Raster painting tool (Issue 4)

**Labels:** `phase-4a`, `raster`, `rendering`  
**Milestone:** v0.0.8

---

## Issue 4: Add Raster Painting Tool (Phase 4B)

**Title:**  
`Add raster painting tool`

**Description:**

## Summary
Implement direct pixel painting on raster layers, allowing users to paint brush strokes directly onto raster layers similar to Krita/Procreate.

## Motivation
- Currently users can only commit selections to raster layers or paint on vector layers
- Need a true raster painting tool that modifies pixel data directly
- This is the second deliverable of Phase 4 raster rendering

## Current State
- Raster layers exist but can only be modified via selection commit
- No direct pixel painting interface exists
- Canvas has infrastructure for raster layers but lacks painting methods

## Implementation Steps

### 1. Enhance Canvas (src/canvas.rs)
Add `pub fn paint_raster_circle(&mut self, layer_idx: usize, cx: f32, cy: f32, radius: f32, color: Color, brush_opacity: f32) -> bool`:
- Convert canvas-space `(cx, cy)` to pixel coords accounting for `RasterLayer.offset`
- Iterate bounding box, paint pixels within radius using standard alpha-over compositing
- Call `invalidate_layer(layer_idx)` if any pixel changed
- Return whether any pixels were modified

### 2. Update Window Backend (src/window_winit.rs)
Add to app state:
- `raster_painting: bool`
- `last_raster_point: Option<(f32, f32)>`

On mouse-button-down when active layer is raster:
1. Push `UndoEntry::RasterSnapshot` before any pixels change
2. Set `raster_painting = true`
3. Call `canvas.paint_raster_circle()`
4. Set `last_raster_point`

On cursor-moved when `raster_painting`:
- Interpolate between `last_raster_point` and current (prevents gaps at fast mouse speed)
- Call `paint_raster_circle()` for each step
- Update `last_raster_point`

On mouse-button-up:
- Clear `raster_painting` and `last_raster_point`

**Important**: Do NOT use `ActiveStroke` — pixels commit directly on every mouse move

### 3. Update Linux Backend (src/window_gtk4.rs)
Mirror the exact same raster painting changes from window_winit.rs

### 4. Update Canvas Data Model
Ensure `UndoEntry::RasterSnapshot` is properly handled in undo/redo logic (from Issue 2)

## Files Affected
- `src/canvas.rs` — Add paint_raster_circle method
- `src/window_winit.rs` — Add raster painting state and handlers
- `src/window_gtk4.rs` — Mirror raster painting implementation
- `src/renderer.rs` / `src/gl_renderer.rs` — No changes needed (uses updated canvas data)

## Time Estimate
Medium effort — input handling and canvas method implementation

## Test Checklist
- [ ] Create raster layer, make active, drag mouse → pixels appear
- [ ] Paint with opacity < 1.0 → correct alpha blending
- [ ] Brush size control changes radius
- [ ] Undo restores pre-stroke state
- [ ] Paint across canvas boundary → no panic
- [ ] Vector layer above raster paints on top (correct layer ordering)
- [ ] Performance is acceptable during painting (no significant lag)

## Files Affected
- `src/canvas.rs`
- `src/window_winit.rs`
- `src/window_gtk4.rs`

## Time Estimate
Medium effort — input handling implementation

## Related Issues
- Requires: Raster display (Issue 3)
- Enables: Future features like pressure sensitivity, blending modes

**Labels:** `phase-4b`, `raster`, `painting`  
**Milestone:** v0.0.8

---

## Issue 5: Unified OpenGL Backend (Phase 4C)

**Title:**  
`Unified OpenGL backend for Windows and Linux`

**Description:**

## Summary
Ensure the OpenGL rendering backend has feature parity between Windows and Linux platforms, completing Phase 4 raster rendering implementation.

## Motivation
- Currently Linux has a working OpenGL implementation via GTK4/GLArea
- Windows needs equivalent OpenGL implementation via winit
- Need to unify both backends to have identical raster rendering capabilities
- This completes the Phase 4 raster rendering work

## Current State
- Linux: `GTK4` + `OpenGL/glow` (working raster display and painting)
- Windows: `winit` + `OpenGL/glow` (to be implemented in Issues 1-4)
- Need to verify both platforms behave identically for raster operations

## Implementation Steps

### 1. Create Common OpenGL Renderer Interface
- Extract common traits or abstract base class for OpenGL operations
- Ensure both platforms use the same shader programs and uniform layouts
- Standardize texture upload and binding group creation

### 2. Platform-Specific Adaptations
**Windows (src/window_winit.rs):**
- OpenGL context creation via winit
- Handle Windows-specific OpenGL quirks (if any)
- Use same render loop as Linux after context creation

**Linux (src/window_gtk4.rs):**
- Keep existing GTK4 GLArea implementation
- Ensure it uses the same OpenGL renderer code path as Windows

### 3. Verify Feature Parity
Confirm both platforms support:
- Raster layer display with correct positioning
- Raster layer opacity
- Proper layer ordering (raster/vector interleaving)
- Raster painting with undo/redo
- Brush size and opacity controls
- Export to PNG
- Zoom/pan functionality

### 4. Handle Platform Differences
Address any platform-specific OpenGL behavior:
- Context creation differences
- Extension availability
- Performance characteristics
- Error handling

## Files Affected
- `src/window_winit.rs` — Windows OpenGL context and rendering
- `src/window_gtk4.rs` — Linux OpenGL rendering (ensure consistency)
- `src/gl_renderer.rs` — Common OpenGL renderer implementation
- `src/gl_loader.rs` — Symbol loading (if needed for Windows)
- `src/shaders/render.wgsl` / `src/shaders/render.glsl` — Shader consistency

## Time Estimate
Low effort — mostly verification and minor adjustments

## Test Checklist
- [ ] Application launches and draws correctly on both Windows and Linux
- [ ] Raster layer display works identically on both platforms
- [ ] Raster painting works identically on both platforms
- [ ] Undo/redo works identically on both platforms
- [ ] Export to PNG produces identical results
- [ ] Performance is comparable between platforms
- [ ] No platform-specific crashes or graphical glitches

## Files Affected
- `src/window_winit.rs`
- `src/window_gtk4.rs`
- `src/gl_renderer.rs`
- `src/gl_loader.rs`

## Time Estimate
Low effort — verification and parity testing

## Related Issues
- Depends on: All previous Issues (1-4)
- Completes: Phase 4 raster rendering

**Labels:** `phase-4c`, `opengl`, `cross-platform`  
**Milestone:** v0.0.8

---

## Implementation Order & Dependencies

### Recommended Sequence
1. **Issue 1**: Switch Graphics Backend from WGPU to OpenGL
   - Enables all subsequent work by fixing resize issue and unifying graphics

2. **Issue 2**: Implement Per-Layer Version Tracking
   - Required for efficient raster display and painting

3. **Issue 3**: Display Raster Layers on Canvas (Phase 4A)
   - First visible raster feature

4. **Issue 4**: Add Raster Painting Tool (Phase 4B)
   - Main user-facing raster feature

5. **Issue 5**: Unified OpenGL Backend (Phase 4C)
   - Final quality assurance and platform parity

### Dependencies Graph
```
Issue 1 → Issue 2 → Issue 3 → Issue 4
                              ↘
                               → Issue 5
```

### Milestone Target
All issues target **v0.0.8** milestone, representing a significant enhancement over v0.0.7 with:
- Fixed window resize rendering
- Visible and paintable raster layers
- Cross-platform OpenGL backend
- Foundation for future features (blending modes, image import, etc.)

---

## Notes for Implementation

### Testing Strategy
- **Unit tests**: Continue existing practice for pure logic (canvas.rs, geometry, etc.)
- **Integration testing**: Use manual verification via INTEGRATION_TESTS.md for:
  - Window management and input
  - Rendering correctness
  - Platform-specific behavior
  - Performance characteristics

### Risk Mitigation
- **Issue 1**: Keep wgpu dependency in Cargo.toml until OpenGL implementation is verified working
- **Issue 2**: Maintain backward compatibility in undo system during transition
- **Issue 3-4**: Implement incrementally with testable checkpoints
- **Issue 5**: Use existing Linux OpenGL implementation as reference

### Future Work (Post-Phase 4)
Once these issues are complete, consider:
- Blending modes (Phase 5 per TODO.md)
- Image import (PNG/JPG as raster layers)
- Pressure sensitivity
- Advanced selection tools
- AI Coach infrastructure (separate from core painting features)

---

*End of document. Copy each issue block above to create corresponding GitHub issues.*