//! Rancer - A high-performance digital art application
//!
//! This library provides the core canvas and drawing engine for the Rancer application.

pub mod canvas;
pub mod compositor;
pub mod export_ui;
pub mod logger;
pub mod preferences;
pub mod renderer;
pub mod viewport;
pub mod window;

#[cfg(test)]
pub mod window_tests;

#[cfg(test)]
pub mod raster_render_tests;

#[cfg(test)]
pub mod app_tests;

#[cfg(test)]
pub mod sdl2_event_tests;

#[cfg(test)]
pub mod undo_tests;

#[cfg(test)]
pub mod ui_tests;

#[cfg(test)]
pub mod render_optimization_tests;

pub mod app;
pub mod brush;
pub mod commands;
pub mod events;
pub mod tools;
pub mod ui;

pub mod gl {
    pub use glow::*;
}
