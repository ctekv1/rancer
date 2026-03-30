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
- **Cairo fallback**: Software rendering for Linux/GTK4 compatibility
- **Vertex generation**: Quad-based thick line rendering with proper normals
- **Separate UI pipeline**: Independent rendering for UI elements
- **Cross-platform compilation**: Conditional compilation for Windows/Linux

### Dependencies
- winit 0.30 for Windows window management
- wgpu 28.0 for GPU-accelerated rendering
- GTK4 0.9 for Linux/Wayland support
- Cairo-rs 0.20 for Linux rendering
- tokio for async runtime support

### Platform Support
- **Windows**: Full WGPU support with winit window management
- **Linux**: GTK4 backend with Cairo rendering (Wayland compatible)