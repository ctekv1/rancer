# OpenGL Migration Plan

## Goal
Replace WGPU with OpenGL (via glutin) across both platforms. Keep GTK4 on Linux for windowing, use winit + glutin on Windows.

## Rationale
- Current Windows resize issue is a wgpu-specific bug with no fix available
- wgpu has closed relevant issues as "not planned"
- glutin provides cross-platform OpenGL context creation
- glow (already dependency) handles OpenGL API

## Current Architecture

| Platform | Windowing | Renderer |
|----------|-----------|----------|
| Windows | winit | WGPU |
| Linux | GTK4 | OpenGL via GLArea |

## Target Architecture

| Platform | Windowing | Renderer |
|----------|-----------|----------|
| Windows | winit | OpenGL via glutin |
| Linux | GTK4 | OpenGL via glutin |

GTK4 windowing stays on Linux. Only the GL context provider changes (GLArea → glutin).

## Implementation Steps

### Phase 1: Dependencies
- [ ] Add `glutin = "0.32"` to Cargo.toml (both platforms)
- [ ] Verify cross-platform GL context creation

### Phase 2: Renderer Unification
- [x] `src/opengl_renderer.rs` is already generic (takes `Rc<glow::Context>`)
- [x] Linux: GTK4 GLArea provides GL context, works as-is
- [x] Fix winit 0.30 API change: `set_wait_timeout` → `WaitUntil`
- [ ] Windows: Will use glutin to create GL context (Phase 3)

### Phase 3: Windows Backend Migration
- [ ] Add glutin context initialization to `src/window_winit.rs`
  - Glutin API is complex, requires careful implementation
  - Started work, needs completion
- [ ] Replace WGPU initialization with glutin + OpenGL setup
- [ ] Remove WGPU-specific code paths
- [ ] Test window resize behavior on Windows (primary fix target)

### Phase 4: Cleanup
- [ ] Remove `src/renderer.rs` (WGPU implementation)
- [ ] Remove `wgpu` from Cargo.toml
- [ ] Update `src/lib.rs` to remove wgpu cfg flags
- [ ] Update TODO.md with completed items

## Dependencies After
```toml
# Core
winit = "0.30"
glow = "0.14"
glutin = "0.32"

# Existing
bytemuck = "1.25"
chrono = "0.4"
dirs = "5.0"
image = "0.24"
log = "0.4"
serde = "1.0"
toml = "0.8"

# Platform-specific (kept)
gtk4 = "0.9"     # Linux windowing only
rfd = "0.15"      # Native dialogs (both platforms)
libloading = "0.8" # Linux GL loading

# Removed
# - wgpu
# - raw-window-handle (wgpu dep)
```

## Expected Outcome
| Metric | Before | After |
|--------|--------|-------|
| Renderer code | ~1981 lines | ~520 lines |
| WGPU deps | ~1461 lines | removed |
| Dependencies | 8 crates | 8 crates (swapped) |

## Testing Checklist
- [ ] Window creation on Windows
- [ ] Window creation on Linux
- [ ] OpenGL context creation (both platforms)
- [ ] Stroke rendering
- [ ] UI rendering
- [ ] Zoom/Pan functionality
- [ ] **Window resize (Windows)** — Primary fix target
- [ ] Layer system
- [ ] Selection tool
- [ ] Export functionality

## References
- `docs/window-resize-issue.md` — Current WGPU resize bug
- `src/opengl_renderer.rs` — Existing OpenGL implementation
- `src/window_winit.rs` — Existing winit implementation
- `src/renderer.rs` — WGPU implementation (to be removed)