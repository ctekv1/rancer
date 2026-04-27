# Phase 4: Raster Rendering

## Context

Phases 1–3 delivered complete raster data infrastructure: `RasterImage`, `RasterLayer`, `LayerContent::Raster`, selection-to-raster commit, and WGSL textured quad shaders (`vs_textured`/`fs_textured`). However, raster layers are silently dropped in every render loop with `continue`. Phase 4 makes them appear on screen and adds a raster painting tool.

### Two structural issues to fix first

1. **Single global `version: u64` in Canvas** — every raster paint event bumps this and forces a full vector stroke cache rebuild. Per-layer version tracking is needed.
2. **Flat-accumulation render loop** — `render_wgpu()` collects all vector vertices from all layers first, then draws everything. This makes correct interleaving of raster and vector layers (e.g. raster layer 2 over vector layer 1 over raster layer 0) impossible. Must switch to a per-layer ordered dispatch pattern.

### Krita reference

- **CPU paint → GPU display**: Tools write pixels to CPU `Vec<u8>`; GPU textures are re-uploaded per frame only for dirty layers. Same pattern we use.
- **Per-layer dirty tracking**: Whole-layer re-upload is correct at Rancer's canvas sizes (≤4K × 4K, 3.5 MB at 1280×720). Krita's 64×64 tile system is for infinite/sparse canvases — overkill here.
- **Direct painting first**: Krita's `KisIndirectPaintingSupport` (wet-layer accumulation) is Phase 5. Phase 4 writes pixels directly to `RasterImage.data` on each mouse move — this is Krita's "direct" mode.
- **Per-layer GPU textures, not CPU pre-composite**: Raster and vector layers composite in correct stack order inside a single render pass. CPU pre-flattening would destroy interleaved layer ordering.

---

## Architecture Decisions

### Render loop: per-layer ordered dispatch

**Current** (`render_wgpu`, line ~816): Collects all vector vertices into flat buffers, draws them all at once. Raster layers go into `raster_layers_data` but are never drawn.

**New**: Single render pass iterates layers bottom-to-top, switches pipeline per layer type:
- Vector layer → set vector pipeline, draw strip/tri ranges for that layer
- Raster layer → set raster pipeline, bind texture, draw textured quad

WGPU allows `render_pass.set_pipeline()` multiple times within one pass. This is the correct approach and matches how Krita composites layers in its OpenGL display.

### Opacity

`fs_textured` currently returns `tex_color` with no opacity. Add `opacity: f32` to `TexturedUniforms` and multiply in the shader. The Rust side passes `layer.opacity * raster.opacity` combined.

### WGSL struct alignment

`TexturedUniforms` has an implicit 4-byte pad between `zoom: f32` (offset 8) and `pan_offset: vec2<f32>` (offset 16, align 8). Adding `opacity: f32` fills that pad slot — no struct size change.

Rust layout: `[canvas_w, canvas_h, zoom, opacity, pan_x, pan_y, img_w, img_h, img_off_x, img_off_y]` — 10 floats, 40 bytes.

### Undo for raster

`undo_stack: Vec<(usize, Stroke)>` is vector-only. Extend to an enum:

```rust
pub enum UndoEntry {
    VectorStroke(usize, Stroke),
    RasterSnapshot { layer_idx: usize, data: Vec<u8> },
}
```

Push a `RasterSnapshot` on mouse-down before any pixels are painted. On undo, restore saved `data`. A 1280×720 RGBA8 snapshot is 3.5 MB — acceptable for a ≤50 entry stack.

---

## Phase 4A — GPU Display of Existing Raster Layers

**Goal**: A raster layer created via `commit_selection_to_raster()` appears on screen, correctly positioned, with opacity, composited in correct layer order with vector layers.

### `src/canvas.rs`
- Add `layer_versions: Vec<u64>` field to `Canvas` (parallel to `layers: Vec<Layer>`)
- Initialize to `vec![0]` in `Default::default()`
- Add `pub fn layer_version(&self, idx: usize) -> u64`
- Add private `fn invalidate_layer(&mut self, idx: usize)` — increments both `version` and `layer_versions[idx]`
- Update `add_layer()`, `remove_layer()`, `move_layer()` to keep `layer_versions` in sync
- Change `undo_stack: Vec<(usize, Stroke)>` → `undo_stack: Vec<UndoEntry>` with enum above
- Update `undo()` and `redo()` to match on the new enum
- Add `pub fn raster_layer_mut(&mut self, idx: usize) -> Option<&mut RasterLayer>` — calls `invalidate_layer`
- Add `pub fn create_raster_layer(&mut self, name: String) -> Result<usize, String>` — inserts transparent RGBA8 layer of canvas dimensions above active layer

### `src/shaders/render.wgsl`
- Add `opacity: f32` to `TexturedUniforms` between `zoom` and `pan_offset` (fills alignment gap)
- Update `fs_textured`: `return vec4(tex_color.rgb, tex_color.a * textured_uniforms.opacity);`

### `src/renderer.rs`
- Add to `Renderer` struct:
  - `raster_texture_versions: Vec<u64>` — per-layer version last uploaded
  - `raster_bind_group_layout: Option<wgpu::BindGroupLayout>` — layout for textured pipeline
- Remove `#[allow(dead_code)]` from all four raster stub fields
- Implement `fn init_raster_pipeline(device, shader_module, surface_format, sample_count) -> (RenderPipeline, BindGroupLayout, Sampler)` — called from `init_wgpu()`; uses `vs_textured`/`fs_textured` entry points; sets `multisample.count = self.sample_count`
- Implement `fn upload_raster_layer(&mut self, device, queue, layer_idx, raster: &RasterLayer, canvas_layer_version: u64)`:
  1. Create `wgpu::Texture` (format `Rgba8Unorm`, usage `TEXTURE_BINDING | COPY_DST`)
  2. Call `queue.write_texture` with `raster.image.data`
  3. Create `TextureView` and `BindGroup` (uniform buffer + sampler + texture)
  4. Store in `raster_texture_cache[layer_idx]` and `raster_bind_group_cache[layer_idx]`
  5. Set `raster_texture_versions[layer_idx] = canvas_layer_version`
- Implement `fn ensure_raster_textures_valid(&mut self, device, queue, canvas: &Canvas)` — iterates raster layers, calls `upload_raster_layer` only when version mismatch; resizes caches when layer count changes
- Implement `fn build_textured_uniforms(canvas, viewport, raster: &RasterLayer, opacity: f32) -> [f32; 10]`
- Implement `fn build_raster_quad_vertices(raster: &RasterLayer) -> [[f32; 4]; 6]` — two triangles covering the layer's rect in canvas space, UV 0..1
- **Refactor `render_wgpu()`**: Replace flat-accumulation loop with per-layer ordered dispatch. Call `ensure_raster_textures_valid()` before the render pass.

### `src/ui.rs` + `src/geometry/ui_elements.rs`
- Add `UiElement::AddRasterLayer` variant
- Add "Add Raster Layer" button to the layer panel (below existing "Add Layer" button)

### `src/window_winit.rs`
- Handle `UiElement::AddRasterLayer` click → call `canvas.create_raster_layer()`

### Testing 4A
1. Draw vector strokes, select region, commit to raster → layer appears on canvas
2. Pan/zoom → raster layer moves and scales correctly
3. Toggle visibility → layer appears/disappears
4. Reorder layers → raster composites in correct stack order
5. Adjust layer opacity → raster dims correctly
6. Vector layer above raster → vector renders on top

---

## Phase 4B — Raster Painting Tool

**Goal**: User can paint pixels directly onto the active raster layer.

### `src/canvas.rs`
- Add `pub fn paint_raster_circle(&mut self, layer_idx: usize, cx: f32, cy: f32, radius: f32, color: Color, brush_opacity: f32) -> bool`:
  - Converts canvas-space `(cx, cy)` to pixel coords accounting for `RasterLayer.offset`
  - Iterates bounding box, paints pixels within radius using standard alpha-over compositing
  - Calls `invalidate_layer(layer_idx)` if any pixel changed
  - Returns whether any pixels were modified

### `src/window_winit.rs`
- Add to app state: `raster_painting: bool`, `last_raster_point: Option<(f32, f32)>`
- Mouse-button-down when active layer is raster:
  1. Push `UndoEntry::RasterSnapshot` before any pixels change
  2. Set `raster_painting = true`
  3. Call `canvas.paint_raster_circle()`
  4. Set `last_raster_point`
- Cursor-moved when `raster_painting`: interpolate between `last_raster_point` and current (prevents gaps at fast mouse speed), call `paint_raster_circle()` for each step
- Mouse-button-up: clear `raster_painting` and `last_raster_point`
- **Do NOT use `ActiveStroke`** — pixels commit directly on every mouse move, no stroke/commit cycle

### `src/window_gtk4.rs`
- Same raster painting changes (mirrors winit structure exactly)

### Testing 4B
1. Create raster layer, make it active, drag mouse → pixels appear
2. Paint with opacity < 1.0 → correct alpha blending
3. Brush size control changes radius
4. Undo restores pre-stroke state
5. Paint across canvas boundary → no panic
6. Vector layer above raster paints on top

---

## Phase 4C — OpenGL Parity (Linux)

**Goal**: Linux GTK4/OpenGL backend matches Windows WGPU backend.

### `src/opengl_renderer.rs`
- Add GLSL textured shader strings:
  - Vertex: position (vec2) + texcoord (vec2), same pan/zoom/offset/opacity uniforms
  - Fragment: `texture2D(tex, v_texcoord) * vec4(1.0, 1.0, 1.0, opacity)`
- Add to `GlRenderer`: `textured_program`, `raster_textures: Vec<Option<glow::Texture>>`, `raster_texture_versions: Vec<u64>`, `textured_vao`, `textured_vbo`
- Implement `fn compile_textured_shaders(gl) -> Result<glow::Program>`
- Implement `fn upload_raster_texture(gl, slot, raster)` — `gl.tex_image_2d` on first upload, `gl.tex_sub_image_2d` for updates of same size (faster)
- Implement `fn ensure_raster_textures_valid(&mut self, canvas)`
- Implement `fn draw_raster_layer(gl, raster, texture, viewport, window_size)`
- Refactor `render()` loop to per-layer ordered dispatch, mirroring WGPU side

### Testing 4C
Run on Linux / CI (Ubuntu + virtual display). Same test cases as 4A and 4B.

---

## Scope Boundary — What Is NOT in Phase 4

| Feature | Rationale |
|---------|-----------|
| Tile-based storage (Krita's KisTiledDataManager) | Overkill at ≤4K canvas; warranted only for infinite/sparse canvas |
| Layer-as-compositing-target (Option B in TODO) | Major rewrite; not needed for correct raster display |
| Blending modes (Multiply, Screen, Overlay) | `fs_textured` gets `blend_mode: u32` uniform in Phase 5 |
| Indirect painting / wet layer (Krita's KisIndirectPaintingSupport) | Phase 5 |
| Image import (PNG/JPG as raster layer) | Phase 5 — infrastructure here makes it straightforward |
| Raster redo | Requires double-storing snapshots; acceptable MVP gap |
| Pressure sensitivity | Separate tablet API concern |
| 16-bit / float color depth | RGBA8 is correct for Phase 4 |

---

## Implementation Order

Execute in sequence for testable checkpoints:

1. **`canvas.rs`** — pure data layer; existing test suite guards against regressions
2. **`render.wgsl`** — add opacity field and shader line
3. **`renderer.rs`** — raster pipeline init, texture upload, ensure_valid, render loop refactor → test that `commit_selection_to_raster()` results appear on Windows
4. **`window_winit.rs`** — raster painting input → test raster brush on Windows
5. **`opengl_renderer.rs`** — Phase 4C → test on Linux / CI
6. **`window_gtk4.rs`** — Linux painting input
7. **`ui.rs` / `ui_elements.rs`** — AddRasterLayer button (last; doesn't block anything)

## Critical Files

| File | Change |
|------|--------|
| `src/canvas.rs` | `layer_versions`, `UndoEntry` enum, `paint_raster_circle`, `raster_layer_mut`, `create_raster_layer` |
| `src/shaders/render.wgsl` | `opacity` in `TexturedUniforms`, apply in `fs_textured` |
| `src/renderer.rs` | Raster pipeline init, texture upload, `ensure_valid`, render loop refactor |
| `src/opengl_renderer.rs` | Textured GLSL shaders, texture upload, per-layer render loop |
| `src/window_winit.rs` | Raster painting input, `UndoEntry` push on mouse-down |
| `src/window_gtk4.rs` | Same as winit |
| `src/ui.rs` + `src/geometry/ui_elements.rs` | AddRasterLayer button |
