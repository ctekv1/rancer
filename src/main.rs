//! Rancer - Digital Art Application
//!
//! Main entry point for the application.
//! Uses winit for window management and mouse input handling.

use rancer::window;

fn main() {
    println!("Starting Rancer application with winit window...");
    
    // Run the window application
    window::run_window_app();
    println!("Rancer application closed successfully");
}
