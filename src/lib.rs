//! Rancer - A high-performance digital art application
//!
//! This library provides the core canvas and drawing engine for the Rancer application.
//! Features GPU-accelerated rendering, stroke management, and user preferences.

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
    /// 
    /// Note: Window system, GPU context, and preferences are initialized
    /// in the platform-specific window backends (winit/GTK4).
    pub fn init(&mut self) {
        println!("Rancer application initialized");
    }

    /// Main application loop
    /// 
    /// Note: The main event loop, window event handling, canvas rendering,
    /// and input processing are handled by the platform-specific window backends.
    pub fn run(&mut self) {
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