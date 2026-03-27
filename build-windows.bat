@echo off
echo Building Rancer v0.0.4 for Windows...
echo.

REM Check if Rust is installed
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Rust is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo Building release for Windows...
cargo build --release

if %errorlevel% equ 0 (
    echo.
    echo Build successful!
    echo Windows binary: target\release\rancer.exe
    echo.
    echo To run: target\release\rancer.exe
    echo.
    echo Binary size:
    dir target\release\rancer.exe | find "rancer.exe"
) else (
    echo.
    echo Build failed. Please check the error messages above.
    echo Common issues:
    echo 1. Missing Windows development tools
    echo 2. Network connectivity issues
    echo 3. Outdated dependencies
)

pause