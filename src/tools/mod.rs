//! Tool trait — interface for all drawing and interaction tools
//!
//! Each tool implements this trait to handle user input events.

use std::any::Any;

use crate::canvas::{Canvas, Color};
use crate::brush::BrushType;

/// Shared brush settings that tools can read/write
#[derive(Debug, Clone, Copy)]
pub struct BrushSettings {
    pub size: u32,
    pub opacity: f32,
    pub color: Color,
    pub brush_type: BrushType,
}

/// Available tool types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolType {
    Brush,
    Selection,
}

/// Trait for all tools (brush, pan, selection, etc.)
pub trait Tool: Any {
    /// Called when mouse is pressed
    fn on_press(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    /// Called when mouse is dragged
    fn on_drag(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    /// Called when mouse is released
    fn on_release(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    /// Called when keyboard input occurs
    fn on_key(&mut self, code: &str);
    /// Tool name for UI display
    fn name(&self) -> &str;
    /// Get brush settings (default: returns defaults for non-brush tools)
    fn brush_settings(&self) -> BrushSettings {
        BrushSettings {
            size: 10,
            opacity: 1.0,
            color: Color { r: 0, g: 0, b: 0, a: 255 },
            brush_type: BrushType::Round,
        }
    }
    /// Set brush size (no-op for non-brush tools)
    fn set_brush_size(&mut self, _size: u32) {}
    /// Set brush opacity (no-op for non-brush tools)
    fn set_brush_opacity(&mut self, _opacity: f32) {}
    /// Set brush color (no-op for non-brush tools)
    fn set_brush_color(&mut self, _color: Color) {}
    /// Set brush type (no-op for non-brush tools)
    fn set_brush_type(&mut self, _brush_type: BrushType) {}
    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Cast to Any mut for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub mod selection_tool;
pub mod brush_tool;

pub use brush_tool::BrushTool;
pub use selection_tool::SelectionTool;
