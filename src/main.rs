//! Rancer - Digital Art Application
//!
//! Main entry point for the application.
//! Currently contains placeholder window loop that will be replaced
//! with Tauri v2 integration for cross-platform window management.

use rancer::{RancerApp, AppConfig};

fn main() {
    // Initialize application configuration
    let config = AppConfig {
        window_width: 1280,
        window_height: 720,
        target_fps: 60,
        debug: true,
    };

    // Create and initialize the application
    let mut app = RancerApp::new(config);
    app.init();

    // TODO: Replace this placeholder loop with Tauri v2 window management
    // Current placeholder demonstrates the intended structure
    println!("Starting Rancer application...");
    
    // Placeholder main loop
    let mut running = true;
    let mut frame_count = 0;
    
    while running {
        // TODO: Handle window events (mouse, keyboard, resize)
        // TODO: Process input events
        // TODO: Update application state
        // TODO: Render canvas using wgpu
        // TODO: Handle frame timing for target FPS
        
        frame_count += 1;
        
        // Simple exit condition for now
        if frame_count >= 300 { // Run for ~5 seconds at 60 FPS
            running = false;
        }
    }

    println!("Rancer application closed");
}
