# Windows Build Guide for Rancer v0.0.7

This guide helps Windows users build Rancer from source.

## Prerequisites

### 1. Install Rust
Download and install Rust from [rustup.rs](https://rustup.rs/):
```bash
# Run this in PowerShell or Command Prompt
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Windows Development Tools
Install the Windows SDK and development tools:
- Download from [Microsoft](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/)
- Or install via Visual Studio Build Tools

## Build Instructions

### Option 1: Using the Build Script (Recommended)
```bash
# Download the build script
curl -O https://raw.githubusercontent.com/ctekv1/rancer/v0.0.7/build-windows.bat

# Run the build script
build-windows.bat
```

### Option 2: Manual Build
```bash
# Clone the repository
git clone https://github.com/ctekv1/rancer.git
cd rancer

# Build for Windows (native target, no cross-compilation needed)
cargo build --release
```

## Expected Output
If successful, you'll find the Windows executable at:
```
target\release\rancer.exe
```

## Troubleshooting

### Common Issues

1. **Missing Windows Tools**
   ```
   error: could not compile `windows-result`
   ```
   **Solution**: Install Windows SDK and development tools

2. **Network Issues**
   ```
   error: failed to download from `https://crates.io/api/v1/crates/...`
   ```
   **Solution**: Check internet connection and try again

### Environment Variables
Ensure these are set in your PATH:
- Rust tools (usually added automatically by rustup)

## Running the Application
```bash
# Navigate to the release directory
cd target\release

# Run the application
rancher.exe
```

## Support
If you encounter issues:
1. Check that all prerequisites are installed
2. Verify environment variables are set correctly
3. Ensure you're using the v0.0.7 tag: `git checkout v0.0.7`
4. Report issues on the [GitHub repository](https://github.com/ctekv1/rancer/issues)
