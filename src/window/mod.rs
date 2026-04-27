//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2 with OpenGL ES 2.0.

pub mod sdl2;

pub use sdl2::{run_app, Sdl2App};