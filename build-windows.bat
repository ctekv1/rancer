@echo off
echo Building Rancer v0.0.2 for Windows...
echo.

REM Check if Rust is installed
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Rust is not installed or not in PATH.
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Check if GTK4 is installed
where gtk4 >nul 2>&1
if %errorlevel% neq 0 (
    echo Warning: GTK4 may not be properly installed.
    echo Please install GTK4 from https://www.gtk.org/download/windows.php
    echo.
    echo Press any key to continue anyway or Ctrl+C to cancel...
    pause >nul
)

echo Installing GTK4 target...
rustup target add x86_64-pc-windows-gnu

echo Building release for Windows...
cargo build --target x86_64-pc-windows-gnu --release

if %errorlevel% equ 0 (
    echo.
    echo Build successful!
    echo Windows binary: target\x86_64-pc-windows-gnu\release\rancer.exe
    echo.
    echo To run: target\x86_64-pc-windows-gnu\release\rancer.exe
) else (
    echo.
    echo Build failed. Please check the error messages above.
    echo Common issues:
    echo 1. GTK4 not installed or not in PATH
    echo 2. Missing Windows development tools
    echo 3. Network connectivity issues
)

pause