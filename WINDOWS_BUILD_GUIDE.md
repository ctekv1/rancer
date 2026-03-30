# Windows Build Guide for Rancer v0.0.6

This guide helps Windows users build Rancer from source.

## Prerequisites

### 1. Install Rust
Download and install Rust from [rustup.rs](https://rustup.rs/):
```bash
# Run this in PowerShell or Command Prompt
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install GTK4 for Windows
Download GTK4 from [gtk.org](https://www.gtk.org/download/windows.php):
- Download the 64-bit installer
- Run the installer and note the installation path
- Add GTK4 bin directory to your PATH environment variable

### 3. Install Windows Development Tools
Install the Windows SDK and development tools:
- Download from [Microsoft](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/)
- Or install via Visual Studio Build Tools

## Build Instructions

### Option 1: Using the Build Script (Recommended)
```bash
# Download the build script
curl -O https://raw.githubusercontent.com/ctekv1/rancer/v0.0.5/build-windows.bat

# Run the build script
build-windows.bat
```

### Option 2: Manual Build
```bash
# Clone the repository
git clone https://github.com/ctekv1/rancer.git
cd rancer
git checkout v0.0.5

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --target x86_64-pc-windows-gnu --release
```

## Expected Output
If successful, you'll find the Windows executable at:
```
target\x86_64-pc-windows-gnu\release\rancer.exe
```

## Troubleshooting

### Common Issues

1. **GTK4 Not Found**
   ```
   error: failed to run custom build command for `glib-sys v0.20.10`
   ```
   **Solution**: Ensure GTK4 is installed and its bin directory is in PATH

2. **Missing Windows Tools**
   ```
   error: could not compile `windows-result`
   ```
   **Solution**: Install Windows SDK and development tools

3. **Network Issues**
   ```
   error: failed to download from `https://crates.io/api/v1/crates/...`
   ```
   **Solution**: Check internet connection and try again

### Environment Variables
Ensure these are set in your PATH:
- GTK4 bin directory (e.g., `C:\gtk64\bin`)
- Rust tools (usually added automatically by rustup)

## Running the Application
```bash
# Navigate to the release directory
cd target\x86_64-pc-windows-gnu\release

# Run the application
rancer.exe
```

## Support
If you encounter issues:
1. Check that all prerequisites are installed
2. Verify environment variables are set correctly
3. Ensure you're using the v0.0.4 tag: `git checkout v0.0.5`
4. Report issues on the [GitHub repository](https://github.com/ctekv1/rancer/issues)