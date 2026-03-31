# Technical Debt Fixes — v0.0.6

This document covers the technical debt fixes applied in v0.0.6, the bugs they resolved, and the tests added to prevent regressions.

---

## 1. Dead Code Removal

**What:** Removed `RancerApp` and `AppConfig` structs from `lib.rs` — they were never used. The actual entry point in `main.rs` dispatches directly to platform-specific backends.

**Why:** Confusing dead code that misled anyone reading the codebase about the app's architecture.

**Files:** `src/lib.rs`

---

## 2. Canvas Resize Coordinate Mapping

**What:** Added `canvas.resize()` call in the `WindowEvent::Resized` handler so the canvas coordinate space matches the window size after resize.

**Bug:** Strokes drawn before a window resize ended up at wrong positions relative to the new window size.

**Files:** `src/window_winit.rs`

---

## 3. Shared UI Hit Detection

**What:** Extracted 449 lines of duplicated UI hit detection logic from `window_winit.rs` and `window_gtk4.rs` into a shared `src/ui.rs` module.

**Bug fixed:** The winit backend never set `slider_drag` on click, meaning slider dragging only worked on GTK4. The unified module ensures consistent behavior.

**Also fixed:** GTK4 checked the save button *before* custom colors while winit checked them in reverse order. The unified version eliminates this inconsistency.

**Files:** `src/ui.rs` (new), `src/window_winit.rs`, `src/window_gtk4.rs`, `src/lib.rs`

**Tests:** 16 new tests in `ui::tests` covering all UI element hit detection.

---

## 4. Combined Vertex Buffer with Stroke Separation

**What:** Combined all stroke vertices into a single GPU buffer per frame, but draw each stroke separately using vertex range offsets.

**Bug:** The initial implementation drew all vertices in a single `draw()` call, which caused the GPU to treat all strokes as one continuous triangle strip — connecting separate strokes together with visible lines.

**Fix:** Track `start..end` ranges per stroke while building the combined buffer, then issue separate `draw(range, 0..1)` calls per stroke. This gives the best of both worlds: one GPU allocation, but correct stroke separation.

**Performance:** Reduces GPU buffer allocations from N+1 per frame (one per stroke + one per UI element) to 2 (one combined stroke buffer + one combined UI buffer).

**Files:** `src/renderer.rs`

**Tests:** 3 new tests in `renderer::tests`:
- `test_combined_stroke_buffer_tracks_ranges` — verifies ranges are non-overlapping and cover all vertices
- `test_combined_buffer_empty_canvas` — verifies empty canvas produces no ranges
- `test_single_point_stroke_excluded` — verifies single-point strokes are skipped

---

## 5. WGPU Error Handling on Windows

**What:** Made the Cairo fallback Linux-only via `#[cfg]` attributes. On Windows, WGPU failure now returns a clear error instead of silently entering an invalid backend state.

**Bug:** If WGPU surface creation failed on Windows, the renderer fell back to `RenderBackend::Cairo` — but Cairo is Linux-only. On Windows, this silently set an invalid backend.

**Files:** `src/renderer.rs`

---

## 6. Zero-Size Window Guard

**What:** Added a guard in `window_winit.rs` that falls back to preferences dimensions when `window.inner_size()` returns `(0, 0)` during the `resumed` phase.

**Bug:** On some Windows systems, `window.inner_size()` returns `(0, 0)` during the `resumed` callback — before the OS has fully realized the window. This zero size was passed to `surface.configure()`, causing a wgpu panic:
```
wgpu error: Validation Error
In Surface::configure
Both `Surface` width and height must be non-zero.
```

**Two-layer defense:**
1. **`window_winit.rs`** — If `inner_size()` returns zero, fall back to `preferences.window.width/height` (1280x720)
2. **`renderer.rs`** — `.max(1)` on surface dimensions as a second line of defense

**Files:** `src/window_winit.rs`, `src/renderer.rs`

**Tests:** 3 new tests in `window_winit::tests`:
- `test_zero_size_window_guard` — verifies fallback to preferences when size is (0, 0)
- `test_nonzero_size_window_uses_actual_size` — verifies actual size is used when valid
- `test_partial_zero_size_guard` — verifies guard works when only one dimension is zero

---

## Test Summary

| Module | Tests Added | Total |
|--------|-------------|-------|
| `ui::tests` | 16 | 16 |
| `renderer::tests` | 3 | 5 |
| `window_winit::tests` | 3 | 8 |
| **Total** | **22** | **146** |

All 146 tests pass with no regressions.
