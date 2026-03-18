//! Rancer - Digital Art Application
//!
//! Main entry point for the application.
//! Uses GTK4 for window management and mouse input handling with WGPU rendering.

use rancer::window;

fn main() {
    println!("Starting Rancer application with GTK4 window and WGPU rendering...");
    
    // Initialize GTK4
    gtk4::init().expect("Failed to initialize GTK4");
    
    // Run the GTK4 window application
    window::run_window_app();
    
    println!("Rancer application closed successfully");
}
