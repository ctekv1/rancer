# Changelog

All notable changes to Rancer will be documented in this file.

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

## [0.0.2] - Previous Release
- Basic window creation and rendering
- Initial stroke drawing functionality

## [0.0.1] - Initial Release
- Project initialization
- Basic canvas structure