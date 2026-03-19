#!/bin/bash
echo "Building Rancer v0.0.3 for Linux from Windows..."
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed or not in PATH."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Add Linux target
echo "Adding Linux target..."
rustup target add x86_64-unknown-linux-gnu

# Install cross-compilation tools if not present
if ! command -v x86_64-linux-gnu-gcc &> /dev/null; then
    echo "Installing cross-compilation tools..."
    # This would typically require installing mingw-w64 or similar
    # For now, we'll try building without explicit cross-compiler
    echo "Note: You may need to install mingw-w64 for full cross-compilation support"
fi

echo "Building release for Linux..."
cargo build --release --target x86_64-unknown-linux-gnu

if [ $? -eq 0 ]; then
    echo ""
    echo "Build successful!"
    echo "Linux binary: target/x86_64-unknown-linux-gnu/release/rancer"
    echo ""
    echo "To run on Linux: ./target/x86_64-unknown-linux-gnu/release/rancer"
    echo ""
    echo "Binary size:"
    ls -lh target/x86_64-unknown-linux-gnu/release/rancer
else
    echo ""
    echo "Build failed. Please check the error messages above."
    echo "Common issues:"
    echo "1. Missing Linux cross-compilation toolchain"
    echo "2. GTK4 development libraries not available"
    echo "3. Network connectivity issues"
    echo ""
    echo "For full cross-compilation support, consider:"
    echo "- Using WSL (Windows Subsystem for Linux)"
    echo "- Installing mingw-w64 toolchain"
    echo "- Using a Linux VM or Docker container"
fi