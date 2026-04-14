# Rancer Integration Testing Checklist

This document covers manual integration testing for features that cannot be unit tested (GUI, rendering, platform-specific code).

## When to Test

- Before releasing a new version
- After adding significant new features
- After refactoring window/renderer code
- When testing on a new platform

---

## Core Functionality Tests

### Canvas & Drawing

- [ ] Open application without errors
- [ ] Draw a stroke with each brush type:
  - [ ] Round brush (default)
  - [ ] Square brush
  - [ ] Spray brush
  - [ ] Calligraphy brush
- [ ] Drawing speed affects stroke differently for each brush
- [ ] Eraser removes strokes correctly
- [ ] Undo/Redo works correctly
- [ ] Clear canvas clears all strokes

### Color & Brush Settings

- [ ] HSV sliders update brush color
- [ ] Custom color palette saves/loads colors
- [ ] Brush size changes work
- [ ] Opacity presets work (25%, 50%, 75%, 100%)
- [ ] Keyboard shortcuts work (+, -, E for eraser)

### Layers

- [ ] Add new layer
- [ ] Delete layer (not background)
- [ ] Reorder layers (move up/down)
- [ ] Toggle visibility
- [ ] Toggle lock
- [ ] Layer opacity affects strokes

### Selection Tool

- [ ] Draw rectangular selection
- [ ] Move selection
- [ ] Copy selection (Ctrl+C)
- [ ] Paste selection (Ctrl+V)
- [ ] Delete selection
- [ ] Marching ants animation visible
- [ ] Commit selection converts to strokes

### Zoom & Pan

- [ ] Mouse wheel zooms toward cursor
- [ ] Zoom buttons work (+/-)
- [ ] Space+drag pans canvas
- [ ] Viewport state persists after draw

### Export

- [ ] Export to PNG works
- [ ] Native save dialog opens
- [ ] OS notification shows on success
- [ ] Exported image matches canvas content

---

## Platform-Specific Tests

### Linux (GTK4)

- [ ] Window opens on Wayland
- [ ] Window opens on X11
- [ ] Menu bar works
- [ ] Close/minimize/maximize buttons work
- [ ] Keyboard input works

### Windows (winit/WGPU)

- [ ] Window opens correctly
- [ ] Close/minimize/maximize buttons work
- [ ] GPU rendering active (check log)
- [ ] Falls back to software if no GPU

---

## Performance Checks

- [ ] Drawing is smooth (no lag)
- [ ] No memory leaks after 100+ strokes
- [ ] Export completes in <5 seconds
- [ ] Startup time <3 seconds

---

## Testing Notes

- Some features (raster layers, texture rendering) are Phase 3+ and may need additional manual testing
- Compare exported PNG with canvas screenshot to verify accuracy
- Test with various canvas sizes (small, medium, large)
- Test with many strokes (100+, 1000+) for performance