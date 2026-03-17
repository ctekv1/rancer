//! Canvas module for Rancer
//!
//! Provides the core canvas functionality for digital art operations.
//! This is a placeholder implementation that will be expanded with
//! actual drawing, rendering, and GPU integration.

/// Represents a 2D point in canvas space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Represents a color in RGBA format
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
}

/// Represents a brush stroke or drawing operation
#[derive(Debug, Clone)]
pub struct Stroke {
    /// Points that make up the stroke path
    pub points: Vec<Point>,
    /// Color of the stroke
    pub color: Color,
    /// Width of the stroke
    pub width: f32,
    /// Opacity of the stroke (0.0 to 1.0)
    pub opacity: f32,
}

/// The main canvas for drawing operations
pub struct Canvas {
    /// Canvas width in pixels
    width: u32,
    /// Canvas height in pixels
    height: u32,
    /// Background color
    background_color: Color,
    /// Current drawing strokes
    strokes: Vec<Stroke>,
    /// Undo history for strokes
    undo_stack: Vec<Stroke>,
    /// Redo history for strokes
    redo_stack: Vec<Stroke>,
}

impl Canvas {
    /// Create a new canvas with default settings
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            background_color: Color::WHITE,
            strokes: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Create a new canvas with specified dimensions
    pub fn with_size(width: u32, height: u32) -> Self {
        let mut canvas = Self::new();
        canvas.resize(width, height);
        canvas
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // TODO: Handle existing content scaling/clipping
    }

    /// Set the background color
    pub fn set_background(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Add a new stroke to the canvas
    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    /// Clear all strokes from the canvas
    pub fn clear(&mut self) {
        self.strokes.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get canvas dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get all current strokes
    pub fn strokes(&self) -> &[Stroke] {
        &self.strokes
    }

    /// Undo the last stroke
    pub fn undo(&mut self) {
        if let Some(stroke) = self.strokes.pop() {
            self.undo_stack.push(stroke);
        }
    }

    /// Redo the last undone stroke
    pub fn redo(&mut self) {
        if let Some(stroke) = self.undo_stack.pop() {
            self.strokes.push(stroke);
        }
    }

    /// Export canvas to a simple representation
    /// TODO: Replace with actual image export (PNG, etc.)
    pub fn export(&self) -> CanvasExport {
        CanvasExport {
            width: self.width,
            height: self.height,
            background: self.background_color,
            stroke_count: self.strokes.len(),
        }
    }
}

/// Simple export representation of canvas state
/// TODO: Replace with actual image data export
#[derive(Debug)]
pub struct CanvasExport {
    pub width: u32,
    pub height: u32,
    pub background: Color,
    pub stroke_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new();
        assert_eq!(canvas.size(), (1920, 1080));
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_canvas_with_size() {
        let canvas = Canvas::with_size(800, 600);
        assert_eq!(canvas.size(), (800, 600));
    }

    #[test]
    fn test_stroke_operations() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
        };

        canvas.add_stroke(stroke.clone());
        assert_eq!(canvas.strokes().len(), 1);

        canvas.undo();
        assert_eq!(canvas.strokes().len(), 0);

        canvas.redo();
        assert_eq!(canvas.strokes().len(), 1);
    }
}