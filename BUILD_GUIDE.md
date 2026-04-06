# Rancer Build Guide

## Version 0.0.7

### Building for Windows (Native)

**Requirements:**
- Rust 1.94 or later
- Windows 10/11

**Steps:**
1. Install Rust from https://rustup.rs/
2. Run the build script:
   ```batch
   build-windows.bat
   ```
3. The binary will be at: `target/release/rancer.exe`

### Building for Linux (Cross-compilation from Windows)

**Current Limitation:**
Cross-compiling from Windows to Linux requires additional toolchain setup that may not work on all systems.

**Recommended Alternatives:**

1. **Use WSL (Windows Subsystem for Linux):**
   ```bash
   # In WSL terminal
   git clone <repository>
   cd rancer
   cargo build --release
   ```

2. **Use a Linux VM or Docker:**
   ```bash
   # In Linux environment
   git clone <repository>
   cd rancer
   cargo build --release
   ```

3. **Use GitHub Actions CI/CD:**
   Set up automated builds for multiple platforms.

### Building for Linux (Native)

**Requirements:**
- Rust 1.94 or later
- GTK4 development libraries
- Linux system (Ubuntu, Fedora, etc.)

**Steps:**
```bash
# Install GTK4 development libraries (Ubuntu/Debian)
sudo apt-get install libgtk-4-dev

# Install GTK4 development libraries (Fedora)
sudo dnf install gtk4-devel

# Build the project
cargo build --release
```

### Dependencies

**Windows:**
- winit 0.30 (window management)
- wgpu 28.0 (GPU rendering)

**Linux:**
- GTK4 0.9 (window management)
- OpenGL/glow 0.14 (GPU rendering)

### Platform-specific Notes

**Windows:**
- Uses winit for window management
- Uses WGPU for GPU-accelerated rendering
- No additional dependencies required

**Linux:**
- Uses GTK4 for window management (Wayland compatible)
- Uses OpenGL for GPU-accelerated rendering
- Requires GTK4 development libraries

### Troubleshooting

**Windows Build Issues:**
- Ensure Rust is properly installed
- Check that all dependencies can be downloaded
- Verify Windows development tools are installed

**Linux Build Issues:**
- Install GTK4 development libraries
- Ensure Rust target is properly configured
- Check that all Linux dependencies are available

**Cross-compilation Issues:**
- Consider using WSL instead of native cross-compilation
- Use Docker containers for consistent build environments
- Set up CI/CD for automated multi-platform builds