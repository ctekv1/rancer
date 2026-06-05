//! Tool trait — interface for all drawing and interaction tools
//!
//! Each tool implements this trait to handle user input events.
//! Brush-specific configuration lives in the separate `BrushConfig` trait.

use crate::brush::BrushType;
use crate::canvas::{Canvas, Color};

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
}

/// Trait for all tools (brush, pan, selection, etc.)
pub trait Tool {
    fn on_press(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    fn on_drag(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    fn on_release(&mut self, x: f32, y: f32, canvas: &mut Canvas);
    fn on_key(&mut self, code: &str);
    fn name(&self) -> &str;
    fn brush_settings(&self) -> Option<BrushSettings> {
        None
    }
    fn as_brush_config(&mut self) -> Option<&mut dyn BrushConfig> {
        None
    }
}

/// Brush-specific configuration (size, opacity, color, eraser mode).
/// Only brush-like tools implement this.
pub trait BrushConfig {
    fn brush_settings(&self) -> BrushSettings;
    fn set_brush_size(&mut self, size: u32);
    fn set_brush_opacity(&mut self, opacity: f32);
    fn set_brush_color(&mut self, color: Color);
    fn set_brush_type(&mut self, brush_type: BrushType);
    fn set_eraser_mode(&mut self, enabled: bool);
    fn is_eraser(&self) -> bool;
}

pub mod brush_tool;

pub use brush_tool::BrushTool;
