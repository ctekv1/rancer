# Rancer Roadmap

## Tier 1 - Core Usability (v0.0.6 Target)

- [ ] Custom Color Picker - HSV/RGB picker dialog for any color
- [ ] Brush Opacity Control - Slider or presets (25%, 50%, 75%, 100%)
- [x] Keyboard Shortcuts - Eraser toggle (E), brush size (+/-)
- [x] Undo/Redo UI - Visual buttons or status indicator
- [x] Canvas Clear - Button or shortcut to clear canvas

## Tier 2 - Professional Features (v0.0.7+)

- [ ] Layer System - Multiple layers, reorder, visibility toggle
- [ ] Selection Tool - Rectangular selection with move/copy
- [ ] Transform Tools - Scale, rotate, flip canvas/strokes
- [ ] Brush Types - Round, square, spray, calligraphy
- [ ] Zoom & Pan - Mouse wheel zoom, space+drag pan

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

## Linux MSAA Implementation (Parallel Track)

- [ ] Step 1: Configure GLArea for multisampling (window_gtk4.rs)
- [ ] Step 2: Add msaa_samples config to GlRenderer struct
- [ ] Step 3: Create multisample FBO setup method
- [ ] Step 4: Modify render to use multisample FBO
- [ ] Step 5: Handle window resize with FBO recreation
- [ ] Step 6: Cleanup FBO resources in Drop
