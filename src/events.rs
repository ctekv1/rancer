//! AppEvent enum — translated from SDL2 events to domain events
//!
//! This module defines the events that the AppState handles,
//! decoupled from SDL2's event representation.

/// Events that AppState can handle
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// Mouse button pressed at position
    Press { x: f32, y: f32 },
    /// Mouse dragged to new position
    Drag { x: f32, y: f32 },
    /// Mouse button released at position
    Release { x: f32, y: f32 },
    /// Keyboard event
    Key { code: String },
    /// Window resized to new dimensions
    Resize { width: u32, height: u32 },
    /// Mouse wheel scrolled at position; positive delta = zoom in, negative = zoom out
    Wheel { x: f32, y: f32, delta: i32 },
    /// Canvas panned by delta pixels (space+drag or middle-mouse)
    Pan { dx: f32, dy: f32 },
    /// Application quit requested
    Quit,
}
