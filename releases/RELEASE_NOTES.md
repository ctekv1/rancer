# Rancer v0.0.1 - Initial Release

## 🎨 What's New

This is the initial release of Rancer, a digital art application built in Rust with cross-platform support.

### Core Features
- **Real-time GPU-accelerated drawing** with WGPU 28.0
- **Stroke management** with undo/redo functionality
- **10-color palette** with keyboard navigation (Up/Down arrows)
- **GTK4 window management** (Wayland compatible)
- **Cross-platform support** (Linux, Windows, future WASM)

### Technical Details
- **Version**: 0.0.1
- **License**: GNU GPL-3.0
- **Rust Edition**: 2024
- **Dependencies**: GTK4, WGPU 28.0, Tokio, bytemuck

## 🚀 Quick Start

### Linux (Ubuntu/Debian)
```bash
# Install GTK4
sudo apt install libgtk-4-dev

# Download and run the binary
wget https://github.com/ctekv1/rancer/releases/download/v0.0.1/rancer-linux-x86_64
chmod +x rancer-linux-x86_64
./rancer-linux-x86_64
```

### Windows
```bash
# Install Rust and GTK4
# Download Rust from https://rustup.rs/
# Install GTK4 from https://www.gtk.org/download/windows.php

# Build from source
git clone https://github.com/ctekv1/rancer.git
cd rancer
cargo build --release
target/release/rancer.exe
```

### Build from Source (All Platforms)
```bash
git clone https://github.com/ctekv1/rancer.git
cd rancer
cargo build --release
cargo run --release
```

## 🎯 Usage

- **Draw**: Left click and drag
- **Colors**: Up/Down arrows to change
- **Window**: 1280x720 "Rancer" window

## 📦 Release Assets

- `rancer-linux-x86_64` - Linux binary for x86_64 architecture
- `LICENSE` - GNU GPL-3.0 license file
- `README.md` - Project documentation

## 🔧 Dependencies

### Linux
- GTK4 development libraries (`libgtk-4-dev` on Ubuntu/Debian)

### Windows
- GTK4 runtime libraries
- Rust toolchain

### Build Dependencies
- Rust 1.94+
- Cargo

## 🐛 Known Issues

- Windows binary not included in this release due to cross-compilation limitations
- GTK4 dependencies required on all platforms

## 🤝 Contributing

This project is licensed under the GNU GPL-3.0 License. See the [LICENSE](LICENSE) file for details.

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

---

**Note**: This is the initial release! 🎉 Feedback and contributions are welcome.