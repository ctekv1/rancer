//! Window backend trait for platform-specific implementations
//!
//! This module defines a common interface for window operations,
//! allowing different backends (GTK4 for Linux, winit for Windows)
//! to be used interchangeably.

use crate::canvas::{Canvas, Point};
use std::cell::RefCell;
use std::rc::Rc;

/// Trait defining the window backend interface
pub trait WindowBackend {
    /// Initialize the window
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Run the main event loop
    fn run(&self);

    /// Get the canvas reference
    fn canvas(&self) -> &Rc<RefCell<Canvas>>;

    /// Get current mouse position
    fn mouse_position(&self) -> Point;

    /// Get current mouse state
    fn mouse_state(&self) -> MouseState;

    /// Check if there's an active stroke
    fn has_active_stroke(&self) -> bool;

    /// Get the number of points in active stroke
    fn active_stroke_point_count(&self) -> usize;
}

/// Mouse interaction state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}
