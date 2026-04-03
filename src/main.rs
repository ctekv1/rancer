//! Rancer - Digital Art Application
//!
//! Main entry point for the application.
//! Uses winit on Windows and GTK4 on Linux for window management.

use rancer::logger;
use rancer::preferences;

#[cfg(target_os = "windows")]
use rancer::window_winit;

#[cfg(target_os = "linux")]
use rancer::window_gtk4;

fn main() {
    // Initialize logger
    logger::init().expect("Failed to initialize logger");

    // Load preferences
    let prefs = preferences::load();
    logger::info(&format!(
        "Config file: {:?}",
        preferences::get_config_path()
    ));

    #[cfg(target_os = "windows")]
    {
        logger::info("Starting Rancer v0.0.7 with winit window and WGPU rendering...");
        window_winit::run_window_app(prefs);
    }

    #[cfg(target_os = "linux")]
    {
        logger::info("Starting Rancer v0.0.7 with GTK4 window and OpenGL rendering...");
        window_gtk4::run_window_app(prefs);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        logger::error("Unsupported platform. Only Windows and Linux are supported.");
    }
}
