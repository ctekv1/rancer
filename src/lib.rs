//! Rancer - A high-performance digital art application
//!
//! This library provides the core canvas and drawing engine for the Rancer application.
//! Features GPU-accelerated rendering, stroke management, and user preferences.

pub mod canvas;
pub mod export;
pub mod geometry;
pub mod logger;
pub mod preferences;
pub mod renderer;
pub mod ui;
pub mod window_backend;
pub mod window_winit;

#[cfg(target_os = "linux")]
pub mod window_gtk4;

#[cfg(target_os = "linux")]
pub mod opengl_renderer;

#[cfg(target_os = "linux")]
pub mod gl_loader;
