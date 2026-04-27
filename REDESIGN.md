# Rancer Redesign Plan

## Why This Redesign

Rancer v0.0.7 has two structural problems that compound as the codebase grows:

1. **Duplicated platform backends.** ~1,200 LOC of event handling (zoom, pan, selection, tool
   logic, UI interaction) is copy-pasted between `window_winit.rs` (Windows) and
   `window_gtk4.rs` (Linux). The renderers (`renderer.rs` WGPU / `opengl_renderer.rs` OpenGL)
   share no abstraction, so every feature is implemented twice.

2. **No extensibility model.** Adding a brush, tool, or layer type requires touching 5–8 files
   with no clear contracts. Tools are hardcoded into event handlers. UI layout is hardcoded pixel
   coordinates. There is no registry or trait-based system anywhere.

Additional drivers that pushed this redesign:
- WGPU produces unresolvable black artifacts on Windows window resize (see `KNOWN_ISSUES.md`)
- winit's OpenGL story depends on `glutin`, which has known Wayland issues — winit's dominance
  in the Rust ecosystem is driven by wgpu, not OpenGL
- GTK4 on Windows requires a non-standard build toolchain and 50–100MB of bundled DLLs with
  no established Rust shipping precedent
- The stroke-based selection system is broken by design — it cannot reliably select and move
  partial stroke paths
- The vector stroke model adds GPU mesh generation complexity for a problem better solved with
  raster pixel buffers

---

## Settled Decisions

| Component | Decision | Rationale |
|-----------|----------|-----------|
| Renderer | **OpenGL only** | Battle-tested for 2D apps; no resize artifacts; GTK4/SDL2 handle context creation cleanly |
| Windowing | **SDL2 on both platforms** | Designed for window + GL context; native Wayland on Linux; trivial on Windows; `bundled` feature statically compiles SDL2 |
| Drawing model | **Raster-first** | All professional painting apps store pixel buffers; simplifies selection, export, undo |
| Brush engine | **CPU dab-based** | Dabs stamped into `Vec<u8>` buffers; correct starting point; GPU path can be added later |
| Undo/Redo | **`undo` crate v0.8** | Delta-based Command pattern; does not clone full state; supports branching history |
| Selection | **Pixel-region based** | Copy buffer → move → merge; replaces broken stroke-based selection |
| UI | **egui via `egui_glow`** | Only Rust toolkit with a proven story for custom GPU canvas + overlay UI; design sheet to be applied once available |
| Stroke geometry | **Replaced** | `geometry/stroke.rs` removed; dab shapes replace vertex meshes |
| File format | `.rancer` ZIP + JSON | Future work (v0.0.9) |
| Export | Extend current + headless GL | Future work |
| Preferences | Unchanged | Keep TOML + `dirs` |

---

## Architecture

### Drawing pipeline — current vs target

```
Current (vector):
  Layer → Vec<Stroke> → geometry::stroke → GPU vertex mesh → OpenGL draw call

Target (raster):
  Layer → RasterImage (Vec<u8>) → CPU brush dabs → OpenGL texture upload → compositor shader
```

### Brush engine

Each brush type defines a **dab** — a pixel mask representing the brush tip stamped repeatedly
along the drawn path. Dab spacing is typically `brush_size * 0.25`.

| Brush | Dab shape |
|-------|-----------|
| Square | Filled square at given size |
| Round | Antialiased circle |
| Spray | Randomized points within a circle |
| Calligraphy | Angled ellipse |

The active stroke paints dabs into a temporary `RasterImage` overlay. On commit, the overlay is
alpha-composited into the active layer's pixel buffer. The overlay is uploaded as a separate GL
texture each frame so the in-progress stroke is visible in real time.

### Selection

```
begin_selection(rect)  → copy pixel region → float_buffer; clear source region
move_selection(dx, dy) → update rect position
commit_selection()     → merge float_buffer into active layer at new position
cancel_selection()     → restore float_buffer back to original position
```

### Coordinate system (required before any raster operations)

Screen coordinates from SDL2 must be transformed into canvas coordinates accounting for zoom
and pan offset before any brush dab, selection rect, or hit-test. A `viewport` utility module
handles this transform. All raster operations work exclusively in canvas space.

### Undo memory

`PaintStroke` commands store only the dirty rectangle's pixel data (before + after), not the
full layer. Default undo depth: 30 levels, configurable in preferences. This keeps memory
bounded regardless of canvas size.

### UI ceiling

egui covers the current requirements (toolbar, sliders, color picker, layer panel, modals).
The UI layer is isolated behind `AppState` — if a more capable toolkit is needed in the future
(GPUI, Xilem, or a custom system), only `src/ui/` needs to change. Core app logic is
unaffected.

---

## Target Module Layout

```
src/
├── main.rs                    # SDL2 application init, boots AppState
├── app.rs                     # AppState: canvas, active_tool, history, ui
├── events.rs                  # AppEvent enum — translated from SDL2 events
├── viewport.rs                # Screen ↔ canvas coordinate transforms (zoom, pan)
│
├── model/
│   ├── mod.rs
│   ├── color.rs               # Color, HsvColor, hsv_to_rgb, rgb_to_hsv
│   ├── brush.rs               # BrushType enum, BRUSH_SIZES, brush_size_up/down
│   ├── layer.rs               # Layer, LayerContent, RasterImage, RasterLayer
│   └── canvas.rs              # Canvas struct: Vec<Layer>, active_layer, version
│
├── brush/
│   ├── mod.rs                 # BrushEngine: dab_positions(), apply_dab()
│   ├── dab.rs                 # DabMask type
│   ├── square.rs              # Square dab
│   ├── round.rs               # Round dab (antialiased)
│   ├── spray.rs               # Spray dab
│   └── calligraphy.rs         # Calligraphy dab (angled ellipse)
│
├── history.rs                 # undo crate Command impls: PaintStroke, AddLayer, etc.
├── selection.rs               # PixelSelection: rect, float_buffer, commit, cancel
│
├── tools/
│   ├── mod.rs                 # Tool trait: on_press, on_drag, on_release, on_key
│   ├── brush_tool.rs          # BrushTool — drives BrushEngine, emits PaintStroke commands
│   ├── selection_tool.rs      # SelectionTool — pixel-region select, move, commit
│   └── pan_tool.rs            # PanTool — pan + zoom
│
├── renderer/
│   └── mod.rs                 # OpenGL renderer: texture upload + compositor shader
│
├── window/
│   └── sdl2.rs                # SDL2 window + GL context — SDL2 events → AppEvent → AppState
│
├── ui/
│   └── mod.rs                 # egui_glow integration, toolbar, layer panel, color picker
│
├── export.rs                  # PNG export (extend to headless GL + JPEG/WebP later)
└── preferences.rs             # TOML config (unchanged)
```

---

## Files Deleted

| File | Reason |
|------|--------|
| `src/renderer.rs` | WGPU renderer |
| `src/window_winit.rs` | winit backend |
| `src/window_gtk4.rs` | GTK4 backend |
| `src/geometry/stroke.rs` | Vector mesh generation — replaced by brush engine |
| `src/geometry/ui_elements.rs` | UI vertex generation — replaced by egui |
| `src/ui.rs` | Hardcoded hit-testing — replaced by egui |

---

## Files Migrated

| Current | Becomes | Change |
|---------|---------|--------|
| `src/opengl_renderer.rs` | `src/renderer/mod.rs` | Refactored to composite raster textures |
| `src/canvas.rs` | `src/model/*` + `src/history.rs` + `src/selection.rs` | Split into focused modules |
| `src/export.rs` | `src/export.rs` | Kept, extended later |
| `src/preferences.rs` | `src/preferences.rs` | Unchanged |

---

## Cargo.toml Changes

**Remove:**
```toml
winit
wgpu
tokio
bytemuck
raw-window-handle
gtk4          # linux only
libloading    # linux only
```

**Add:**
```toml
sdl2 = { version = "0.37", features = ["bundled"] }
undo = "0.8"
egui = "0.29"
egui_glow = "0.29"
```

**Keep:** `glow`, `rfd`, `image`, `serde`, `toml`, `dirs`, `chrono`, `log`, `env_logger`

---

## Migration Phases

### Phase 1 — SDL2 + OpenGL, drop WGPU / winit / GTK4
- Delete `src/renderer.rs`, `src/window_winit.rs`, `src/window_gtk4.rs`
- Move `src/opengl_renderer.rs` → `src/renderer/mod.rs`
- Create `src/window/sdl2.rs` — SDL2 window, GL context, event loop skeleton
- Update `Cargo.toml` (remove wgpu/winit/tokio/bytemuck/gtk4/libloading/raw-window-handle; add sdl2 bundled)
- Update `src/main.rs` — single SDL2 boot path, no `#[cfg(target_os)]`
- **Verify:** builds and runs on Windows and Linux (Wayland)

### Phase 2 — Raster layer migration
- Add `src/viewport.rs` — screen ↔ canvas coordinate transform, zoom/pan state
- Modify `LayerContent`: `Vector` variant becomes a no-op stub; `Raster` is the only active path
- Change `Canvas::add_layer()` to create `LayerContent::Raster(RasterImage::new(w, h))`
- Update renderer to composite `RasterImage` buffers as GL textures
- Remove all dependencies on `geometry/stroke.rs`
- **Verify:** app opens, blank raster canvas renders, layers add/remove correctly

### Phase 3 — Tool trait + AppEvent + AppState
- Define `AppEvent` in `src/events.rs`
- Define `Tool` trait in `src/tools/mod.rs`
- Define `AppState` in `src/app.rs` — owns `Canvas`, `active_tool: Box<dyn Tool>`, `history`
- Move all event handling out of the SDL2 window loop into `AppState::handle_event`
- SDL2 loop becomes a thin translator: SDL2 events → `AppEvent` → `AppState::handle_event`
- **Verify:** zoom, pan, layer switching all work through the new abstraction

### Phase 4 — `undo` crate + Command pattern
- Add `undo = "0.8"` to `Cargo.toml`
- Define commands: `PaintStroke { layer_idx, dirty_rect, pixels_before, pixels_after }`
- Define: `AddLayer`, `RemoveLayer`, `MoveLayer`, `ToggleVisibility`, `SetOpacity`
- Replace `Canvas.undo_stack: Vec<(usize, Stroke)>` with `undo::History<Canvas>`
- Cap undo depth at 30 (configurable via preferences)
- Wire Ctrl+Z / Ctrl+Shift+Z through `AppState`
- **Verify:** draw → undo restores pixels; add layer → undo removes it

### Phase 5 — Pixel-region selection
- Create `src/selection.rs`: `PixelSelection { rect, float_buffer: RasterImage }`
- Implement `begin_selection`, `move_selection`, `commit_selection`, `cancel_selection`
- Implement as `SelectionTool` in `src/tools/selection_tool.rs`
- **Verify:** select region → move → commit; select → cancel restores original pixels

### Phase 6 — Brush engine
- Create `src/brush/` with `DabMask` type and all four dab implementations
- `BrushEngine::apply_stroke(points, brush_type, size, opacity, buffer: &mut RasterImage)`
- Active stroke overlay: temporary `RasterImage`, composited live, merged on mouse release
- Implement as `BrushTool` in `src/tools/brush_tool.rs`
- **Verify:** all 4 brush types draw correctly; opacity and size controls work

### Phase 7 — egui UI
- Add `egui`, `egui_glow` to `Cargo.toml`
- Render egui pass after the canvas composite pass in the SDL2 loop
- Implement: brush toolbar, size/opacity sliders, color picker, layer panel
- Delete `src/ui.rs`, `src/geometry/ui_elements.rs`
- Apply design reference via `egui::Visuals` once provided
- **Verify:** full UI works; canvas interaction passes through egui correctly

---

## Verification Checklist

After each phase, run:
```
cargo test
cargo clippy -- -D warnings
```

End-state checklist:
- [ ] Builds on Windows and Linux with no platform `#[cfg]` branching in `main.rs`
- [ ] No `wgpu`, `winit`, `gtk4` in `Cargo.toml`
- [ ] Draw with all 4 brush types; strokes appear correctly
- [ ] Undo/redo works for paint strokes and layer operations
- [ ] Select a region, move it, commit — pixels land correctly
- [ ] Add, remove, reorder, lock, and toggle visibility on layers
- [ ] Export PNG at canvas resolution
- [ ] Zoom and pan work; all raster operations use canvas coordinates not screen coordinates

---

## Known Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Undo memory on large canvases | Dirty-rect delta storage + 30-level cap + configurable depth |
| CPU brush lag on large canvases / big brushes | Acceptable for v1; GPU dab path is a future upgrade |
| egui styling ceiling | UI layer isolated in `src/ui/`; swap cost is contained |
| SDL2 `bundled` build time | One-time compile; cached by cargo after first build |
