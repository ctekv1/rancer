# OpenGL Migration Plan

## Goal
Replace WGPU with OpenGL across both platforms, using winit + glow for unified windowing and rendering.

## Rationale
- Current Windows resize issue is a wgpu-specific bug with no fix available
- wgpu has closed relevant issues as "not planned"
- winit + OpenGL provides a more stable cross-platform solution
- Maximum code reuse: winit already used, glow already a dependency

## Current Architecture

| Platform | Windowing | Renderer |
|----------|-----------|----------|
| Windows | winit | WGPU |
| Linux | GTK4 | OpenGL (glow) |

## Target Architecture

| Platform | Windowing | Renderer |
|----------|-----------|----------|
| Windows | winit | OpenGL (glow) |
| Linux | winit | OpenGL (glow) |

## Implementation Steps

### Phase 1: OpenGL Context Abstraction
- [ ] Create `src/gl_context.rs` — cross-platform OpenGL context creation
- [ ] Support both Windows (WGL) and Linux (GLX/EGL) via glutin or platform-native
- [ ] Provide unified API for GL context management

### Phase 2: Renderer Unification
- [ ] Adapt `src/opengl_renderer.rs` to work without GTK4 GLArea dependency
- [ ] Use the new `gl_context.rs` for context management
- [ ] Ensure feature parity with current WGPU renderer (MSAA, etc.)

### Phase 3: Windows Backend Migration
- [ ] Modify `src/window_winit.rs` to use OpenGL instead of WGPU
- [ ] Remove WGPU-specific code paths
- [ ] Test window resize behavior on Windows

### Phase 4: Linux Backend Simplification
- [ ] Remove GTK4 dependency from `Cargo.toml`
- [ ] Update `src/window_gtk4.rs` → `src/window_sdl2.rs` or reuse winit
- [ ] Remove platform-specific windowing code

### Phase 5: Cleanup
- [ ] Remove `src/renderer.rs` (WGPU implementation)
- [ ] Remove WGPU from `Cargo.toml`
- [ ] Update `src/lib.rs` to remove WGPU cfg flags
- [ ] Update TODO.md with completed items

## Expected Outcome
- Single renderer: ~520 lines (existing OpenGL renderer)
- Single window backend: ~1300 lines (existing winit, adapted)
- Removed: ~1461 lines of WGPU renderer
- Simplified dependency tree

## Dependencies After
```toml
# Core
winit = "0.30"
glow = "0.14"
glutin = "0.32"  # or platform-native

# Existing
bytemuck = "1.25"
chrono = "0.4"
dirs = "5.0"
image = "0.24"
log = "0.4"
serde = "1.0"
toml = "0.8"

# Platform-specific (removed)
# - wgpu (removed)
# - gtk4 (removed)
# - raw-window-handle (may still be needed)
```

## Testing Checklist
- [ ] Window creation on Windows
- [ ] Window creation on Linux
- [ ] OpenGL context creation
- [ ] Stroke rendering
- [ ] UI rendering
- [ ] Zoom/Pan functionality
- [ ] Window resize (primary fix target)
- [ ] Layer system
- [ ] Selection tool
- [ ] Export functionality

## References
- `docs/window-resize-issue.md` — Current WGPU resize bug
- `src/opengl_renderer.rs` — Existing OpenGL implementation
- `src/window_winit.rs` — Existing winit implementation
