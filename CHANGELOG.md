## [0.0.7] - 2026-04-04

### Added
- **Brush types**: Square, Round (soft-edged), Spray (scattered dots), Calligraphy (45° broad-nib)
- **Brush type UI**: 4 icon buttons with selection border at y=225
- **Brush type hit testing**: Click to switch brush types, saves preference to TOML
- **StrokeMesh abstraction**: `StrokeMesh` struct with `DrawMode` (TriangleStrip / Triangles) for flexible brush geometry
- **WGPU spray pipeline**: Separate TriangleList render pipeline for spray brush (scattered quads)
- **188 unit tests** including brush type generation, hit testing, and selection tool
- **Selection tool**: Rectangular selection with move/copy, marching ants animation, keyboard shortcuts
- **Selection UI**: Toggle button at y=265, dashed rectangle with animated marching ants
- **Selection hit testing**: Tool button, selection rect (move/copy), canvas area (draw new selection)
- **Selection keyboard shortcuts**: Delete (commit), Escape (clear), Ctrl+D (deselect), Ctrl+Delete (clear canvas)
- **Selection rendering**: Separate overlay pass with `gl.finish()` for proper presentation
- **GTK4 tick callback**: `add_tick_callback` for continuous marching ants animation synced to display refresh
- **10 selection unit tests**: Capture, move, copy, commit, clear, layer visibility, multi-stroke, empty rect, commit after copy

### Changed
- **Added `brush_type` field to `Stroke` struct** — committed strokes retain their brush type for correct re-rendering
- **Refactored `geometry/stroke.rs`**: BrushType dispatcher routes to 4 generators with configurable constants (`ROUND_SEGMENTS`, `SPRAY_DOTS_PER_SIZE`, `SPRAY_DOT_SIZE`, `CALLIGRAPHY_ANGLE_DEGREES`)
- **Refactored `renderer.rs`**: Dual pipeline (TriangleStrip + TriangleList), split stroke buffers by draw mode
- **Refactored `opengl_renderer.rs`**: Uses `StrokeMesh.mode` to select `glow::TRIANGLE_STRIP` or `glow::TRIANGLES`
- **Refactored `export.rs`**: Handles both TriangleStrip and Triangles draw modes in rasterization
- **Round brush**: Replaced dotted-line approach with soft-edged 4-vertex ribbon (inner/outer alpha gradient)
- **Added `default_type` field to `BrushPreferences`** (saves on click, not loaded on startup yet)
- **Selection uses whole-stroke capture**: If any point of a stroke is inside the rect, the entire stroke is selected (no segment splitting, no data loss)
- **Selection removes originals on begin**: Original strokes removed from layers, stored in `removed_strokes` for clear/restore
- **Selection rect in canvas coordinates**: Proper zoom/pan transform for both drawing and rendering

### Fixed
- **GTK4 eraser hotkey crash**: Fixed double `RefCell` borrow panic on 'E' key press
- **Selection rect coordinate space**: Fixed screen-to-canvas coordinate conversion during drawing
- **Selection hit test**: Fixed `selection_rect` being passed as `None` to hit test (never returned `SelectionRect`)
- **Selection overlay rendering**: Fixed zoom/pan uniforms not being set for selection stroke rendering
- **Marching ants only animating on mouse move**: Fixed by using GTK4's `add_tick_callback` for continuous frame-synced animation
- **Selection strokes disappearing on commit**: Fixed `commit_selection` to properly add moved strokes to active layer
- **Clicking off selection loses data**: Fixed to clear selection and restore original strokes when clicking outside

## [0.0.6] - 2026-03-30

### Added
- **Layer system**: Full layer support — multiple layers (up to 20), visibility toggle, opacity, lock, reorder, add/delete
- **Zoom & Pan**: Mouse wheel zoom toward cursor, space+drag panning, zoom in/out UI buttons
- **MSAA support**: Multisampled rendering with resolve texture (WGPU backend, configurable sample count)
- **Native export dialog**: File save dialog via `rfd`, OS notifications (`notify-send` on Linux), console feedback
- **Export bounding box**: Export now captures all stroke content regardless of position (min 100x100, max 4096x4096)
- **HSV color picker**: Three sliders (H: 0-360°, S: 0-100%, V: 0-100%) with click-and-drag support
- **Custom colors palette**: Save up to 10 colors (FIFO), click to select, arrow keys to navigate
- **168 unit tests** across canvas, geometry, export, preferences, renderer, and UI modules

### Changed
- **Refactored `geometry.rs`**: Split into `geometry/mod.rs`, `geometry/stroke.rs`, `geometry/ui_elements.rs`
- **Refactored `renderer.rs`**: Introduced `RenderFrame` pattern, eliminated duplicated state, removed 12 setter methods and 12 proxy vertex methods
- **Refactored `opengl_renderer.rs`**: Introduced `GlRenderFrame` pattern, batched UI rendering (12 GPU uploads → 1)
- **Refactored `window_gtk4.rs`**: Consolidated ~20 `Rc<RefCell<...>>` into single `GlRenderState`, debounced preference saves
- **Refactored `window_winit.rs`**: Extracted `handle_ui_click()`, `handle_keyboard()`, `handle_cursor_moved()` methods, consolidated state into `WinitRenderState`
- **Export UX**: Replaced silent auto-save with native file dialog, added OS notifications

### Fixed
- **Layer rendering order**: Layers now render back-to-front correctly (bottom layer first)
- **Active stroke layer placement**: Active stroke now renders at the correct layer position instead of on top of all layers
- **Slider drag regression**: Fixed GTK4 slider drag being blocked by drawing state check
- **Export window-area clipping**: Export now computes stroke bounding box and captures all content
- **Duplicate `#[test]` attribute** on `test_active_stroke_with_opacity` in canvas.rs
- **Unused import warnings** across geometry submodules

### Dependencies
- Added `rfd = "0.15"` (Linux + Windows) for native file dialogs

## [0.0.5] - 2026-03-29

### Added
- **Undo/Redo system**: Ctrl+Z (undo), Ctrl+Shift+Z/Ctrl+Y (redo)
- **Eraser tool**: Button toggle (click to on/off), paints white when active
- **Test suite expansion**: 75 → 101 tests across canvas, geometry, preferences, and window modules

### Documentation
- **Updated CI workflow**: Added format check, clippy components, improved caching
- **Updated README.md**: Added undo/redo, eraser features, version 0.0.5

### Next Tasks
- Custom color picker
- Brush opacity control
- Layer support

## [0.0.4] - 2026-03-27

### Added
- **User preferences system**: TOML-based configuration with platform-specific storage
- **Auto-saving preferences**: Settings saved on window resize, brush size change, color selection
- **Config file management**: Auto-creates config with defaults if not found
- **Hex color support**: Colors stored as hex strings (e.g., "#FFFFFF") in preferences

### Fixed
- **Clippy warnings**: Removed unused variables and dead code
- **Canvas default size**: Changed from 1920x1080 to 1280x720 to match window size
- **Documentation accuracy**: Removed misleading TODOs from lib.rs

### Documentation
- **Updated CLAUDE.md**: Added preferences module and configuration details
- **Updated README.md**: Added preferences features, config file locations, version 0.0.4
- **Updated CHANGELOG.md**: Added v0.0.4 entry
- **Updated lib.rs**: Removed outdated TODOs, improved module documentation

### Dependencies
- Added dirs 5.0 for platform-specific config directories
- Added serde 1.0 with derive feature for serialization
- Added toml 0.8 for configuration file format

### Configuration
- **Windows**: `%APPDATA%\rancer\config.toml`
- **Linux**: `~/.config/rancer/config.toml`
- **Settings**: Window size, canvas size, brush defaults, color palette, renderer config

## [0.0.3] - 2026-03-19

### Added
- **Cross-platform window backends**: GTK4 for Linux/Wayland, winit for Windows
- **GPU-accelerated rendering**: WGPU backend for high-performance graphics
- **Color palette UI**: Interactive 10-color palette with selection indicator
- **Brush size selector**: 5 preset brush sizes (3, 5, 10, 25, 50 pixels)
- **Real-time stroke rendering**: Active strokes render as you draw
- **Smooth stroke rendering**: Triangle strip topology for gapless strokes
- **Thick line support**: Variable brush widths with proper line rendering
- **Mouse event handling**: Click, drag, and release detection
- **Keyboard shortcuts**: Arrow keys for color selection
- **Window resize handling**: Automatic canvas resize on window resize

### Technical Implementation
- **WGPU rendering pipeline**: GPU-accelerated graphics with shader support

- **Vertex generation**: Quad-based thick line rendering with proper normals
- **Separate UI pipeline**: Independent rendering for UI elements
- **Cross-platform compilation**: Conditional compilation for Windows/Linux

### Dependencies
- winit 0.30 for Windows window management
- wgpu 28.0 for GPU-accelerated rendering
- GTK4 0.9 for Linux/Wayland support
- tokio (rt-multi-thread only)

### Platform Support
- **Windows**: Full WGPU support with winit window management
- **Linux**: GTK4 backend with Cairo rendering (Wayland compatible)
