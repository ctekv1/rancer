//! Rancer - A high-performance digital art application
//!
//! This library provides the core canvas and drawing engine for the Rancer application.
//! Currently in early development with basic structure placeholders.

pub mod canvas;
pub mod window_winit;
pub mod window_backend;
pub mod renderer;
pub mod logger;
pub mod preferences;

#[cfg(target_os = "linux")]
pub mod window_gtk4;

/// Core application state and configuration
pub struct RancerApp {
    /// Application configuration
    pub config: AppConfig,
    /// Canvas instance for drawing operations
    pub canvas: canvas::Canvas,
}

/// Application configuration settings
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Window width in pixels
    pub window_width: u32,
    /// Window height in pixels  
    pub window_height: u32,
    /// Target frames per second
    pub target_fps: u32,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            target_fps: 60,
            debug: false,
        }
    }
}

impl RancerApp {
    /// Create a new Rancer application instance
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            canvas: canvas::Canvas::new(),
        }
    }

    /// Initialize the application
    pub fn init(&mut self) {
        // TODO: Initialize window system (Tauri v2)
        // TODO: Initialize GPU context (wgpu)
        // TODO: Load user preferences
        println!("Rancer application initialized");
    }

    /// Main application loop
    pub fn run(&mut self) {
        // TODO: Implement main event loop
        // TODO: Handle window events
        // TODO: Render canvas
        // TODO: Process input
        println!("Rancer application running...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let config = AppConfig::default();
        let app = RancerApp::new(config);
        
        assert_eq!(app.config.window_width, 1280);
        assert_eq!(app.config.window_height, 720);
        assert_eq!(app.config.target_fps, 60);
    }
}