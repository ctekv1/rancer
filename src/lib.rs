//! Rancer - A high-performance digital art application
//!
//! This library provides the core canvas and drawing engine for the Rancer application.

pub mod canvas;
pub mod export;
pub mod export_ui;
pub mod geometry;
pub mod logger;
pub mod preferences;
pub mod viewport;
pub mod window;

#[cfg(test)]
pub mod window_tests;

#[cfg(test)]
pub mod raster_render_tests;

pub mod gl {
    pub use glow::*;
}

#[cfg(target_os = "linux")]
pub mod gl_loader;

#[cfg(target_os = "linux")]
pub mod opengl_renderer;