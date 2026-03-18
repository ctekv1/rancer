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
#[derive(Clone)]
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

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            background_color: Color::WHITE,
            strokes: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl Canvas {
    /// Create a new canvas with default settings
    pub fn new() -> Self {
        Self::default()
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

/// A palette of predefined colors for drawing
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// List of available colors
    colors: Vec<Color>,
    /// Index of currently selected color
    selected_index: usize,
}

impl Default for ColorPalette {
    fn default() -> Self {
        let colors = vec![
            Color::BLACK,           // 0 - Black
            Color::WHITE,           // 1 - White
            Color { r: 255, g: 0, b: 0, a: 255 },     // 2 - Red
            Color { r: 0, g: 255, b: 0, a: 255 },     // 3 - Green
            Color { r: 0, g: 0, b: 255, a: 255 },     // 4 - Blue
            Color { r: 255, g: 255, b: 0, a: 255 },   // 5 - Yellow
            Color { r: 255, g: 0, b: 255, a: 255 },   // 6 - Magenta
            Color { r: 0, g: 255, b: 255, a: 255 },   // 7 - Cyan
            Color { r: 64, g: 64, b: 64, a: 255 },    // 8 - Dark Gray
            Color { r: 139, g: 69, b: 19, a: 255 },   // 9 - Brown
        ];
        
        Self {
            colors,
            selected_index: 0, // Default to black
        }
    }
}

impl ColorPalette {
    /// Create a new color palette with a default set of colors
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently selected color
    pub fn current_color(&self) -> Color {
        self.colors[self.selected_index]
    }

    /// Select a color by index
    pub fn select_color(&mut self, index: usize) -> Result<(), String> {
        if index < self.colors.len() {
            self.selected_index = index;
            Ok(())
        } else {
            Err(format!("Color index {} out of range (0-{})", index, self.colors.len() - 1))
        }
    }

    /// Get all available colors
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Get the index of the currently selected color
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Add a custom color to the palette
    pub fn add_color(&mut self, color: Color) {
        self.colors.push(color);
        // Keep selection valid if we were at the end
        if self.selected_index >= self.colors.len() - 1 {
            self.selected_index = self.colors.len() - 1;
        }
    }

    /// Get the number of colors in the palette
    pub fn color_count(&self) -> usize {
        self.colors.len()
    }
}

/// Represents an active stroke that is currently being drawn
#[derive(Debug, Clone)]
pub struct ActiveStroke {
    /// Points collected so far in this stroke
    points: Vec<Point>,
    /// Color of the stroke
    color: Color,
    /// Width of the stroke
    width: f32,
    /// Opacity of the stroke (0.0 to 1.0)
    opacity: f32,
}

impl ActiveStroke {
    /// Create a new active stroke with the given properties
    pub fn new(color: Color, width: f32, opacity: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
            opacity,
        }
    }

    /// Add a point to the active stroke
    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    /// Get all points in the active stroke
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Get the color of the active stroke
    pub fn color(&self) -> Color {
        self.color
    }

    /// Get the width of the active stroke
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the opacity of the active stroke
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Check if the stroke has any points
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Commit the active stroke to create a completed stroke
    /// Returns None if the stroke has no points
    pub fn commit(self) -> Option<Stroke> {
        if self.points.is_empty() {
            None
        } else {
            Some(Stroke {
                points: self.points,
                color: self.color,
                width: self.width,
                opacity: self.opacity,
            })
        }
    }
}

impl Canvas {
    /// Begin a new active stroke with the specified properties
    pub fn begin_stroke(&mut self, color: Color, width: f32, opacity: f32) -> ActiveStroke {
        ActiveStroke::new(color, width, opacity)
    }

    /// Begin a new active stroke using the current color from a palette
    pub fn begin_stroke_with_palette(&mut self, palette: &ColorPalette, width: f32, opacity: f32) -> ActiveStroke {
        let color = palette.current_color();
        self.begin_stroke(color, width, opacity)
    }

    /// Commit an active stroke to the canvas
    pub fn commit_stroke(&mut self, active_stroke: ActiveStroke) -> Result<(), String> {
        if let Some(stroke) = active_stroke.commit() {
            self.add_stroke(stroke);
            Ok(())
        } else {
            Err("Cannot commit empty stroke".to_string())
        }
    }
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

    #[test]
    fn test_color_palette_creation() {
        let palette = ColorPalette::new();
        
        // Should have 10 default colors
        assert_eq!(palette.color_count(), 10);
        
        // Default selection should be black (index 0)
        assert_eq!(palette.selected_index(), 0);
        assert_eq!(palette.current_color(), Color::BLACK);
        
        // Should have all expected default colors
        let colors = palette.colors();
        assert_eq!(colors[0], Color::BLACK);
        assert_eq!(colors[1], Color::WHITE);
        assert_eq!(colors[2], Color { r: 255, g: 0, b: 0, a: 255 }); // Red
        assert_eq!(colors[8], Color { r: 64, g: 64, b: 64, a: 255 }); // Dark Gray
        assert_eq!(colors[9], Color { r: 139, g: 69, b: 19, a: 255 }); // Brown
    }

    #[test]
    fn test_color_palette_selection() {
        let mut palette = ColorPalette::new();
        
        // Select red (index 2)
        assert!(palette.select_color(2).is_ok());
        assert_eq!(palette.selected_index(), 2);
        assert_eq!(palette.current_color(), Color { r: 255, g: 0, b: 0, a: 255 });
        
        // Select blue (index 4)
        assert!(palette.select_color(4).is_ok());
        assert_eq!(palette.selected_index(), 4);
        assert_eq!(palette.current_color(), Color { r: 0, g: 0, b: 255, a: 255 });
        
        // Try to select invalid index
        assert!(palette.select_color(15).is_err());
        assert_eq!(palette.selected_index(), 4); // Should remain unchanged
    }

    #[test]
    fn test_color_palette_add_color() {
        let mut palette = ColorPalette::new();
        let initial_count = palette.color_count();
        
        // Add a custom purple color
        let purple = Color { r: 128, g: 0, b: 128, a: 255 };
        palette.add_color(purple);
        
        assert_eq!(palette.color_count(), initial_count + 1);
        assert_eq!(palette.colors()[initial_count], purple);
        
        // Selecting the new color should work
        assert!(palette.select_color(initial_count).is_ok());
        assert_eq!(palette.current_color(), purple);
    }

    #[test]
    fn test_active_stroke_creation() {
        let color = Color { r: 255, g: 0, b: 0, a: 255 };
        let active_stroke = ActiveStroke::new(color, 3.0, 0.8);
        
        assert_eq!(active_stroke.color(), color);
        assert_eq!(active_stroke.width(), 3.0);
        assert_eq!(active_stroke.opacity(), 0.8);
        assert!(active_stroke.is_empty());
        assert_eq!(active_stroke.points().len(), 0);
    }

    #[test]
    fn test_active_stroke_point_addition() {
        let red_color = Color { r: 255, g: 0, b: 0, a: 255 };
        let mut active_stroke = ActiveStroke::new(red_color, 2.0, 1.0);
        
        // Add some points
        active_stroke.add_point(Point { x: 10.0, y: 20.0 });
        active_stroke.add_point(Point { x: 15.0, y: 25.0 });
        active_stroke.add_point(Point { x: 20.0, y: 30.0 });
        
        assert!(!active_stroke.is_empty());
        assert_eq!(active_stroke.points().len(), 3);
        assert_eq!(active_stroke.points()[0], Point { x: 10.0, y: 20.0 });
        assert_eq!(active_stroke.points()[2], Point { x: 20.0, y: 30.0 });
    }

    #[test]
    fn test_active_stroke_commit() {
        let blue_color = Color { r: 0, g: 0, b: 255, a: 255 };
        let mut active_stroke = ActiveStroke::new(blue_color, 4.0, 0.5);
        
        // Add points to the stroke
        active_stroke.add_point(Point { x: 0.0, y: 0.0 });
        active_stroke.add_point(Point { x: 5.0, y: 5.0 });
        
        // Commit the stroke
        let committed_stroke = active_stroke.commit().expect("Should commit successfully");
        
        assert_eq!(committed_stroke.color, blue_color);
        assert_eq!(committed_stroke.width, 4.0);
        assert_eq!(committed_stroke.opacity, 0.5);
        assert_eq!(committed_stroke.points.len(), 2);
        assert_eq!(committed_stroke.points[0], Point { x: 0.0, y: 0.0 });
        assert_eq!(committed_stroke.points[1], Point { x: 5.0, y: 5.0 });
    }

    #[test]
    fn test_active_stroke_commit_empty() {
        let green_color = Color { r: 0, g: 255, b: 0, a: 255 };
        let active_stroke = ActiveStroke::new(green_color, 1.0, 1.0);
        
        // Try to commit empty stroke
        let result = active_stroke.commit();
        assert!(result.is_none(), "Empty stroke should not commit");
    }

    #[test]
    fn test_canvas_active_stroke_integration() {
        let mut canvas = Canvas::new();
        let palette = ColorPalette::new();
        
        // Begin a stroke with palette color (default black)
        let mut active_stroke = canvas.begin_stroke_with_palette(&palette, 2.0, 1.0);
        assert_eq!(active_stroke.color(), Color::BLACK);
        assert_eq!(active_stroke.width(), 2.0);
        assert_eq!(active_stroke.opacity(), 1.0);
        
        // Add points to the active stroke
        active_stroke.add_point(Point { x: 10.0, y: 10.0 });
        active_stroke.add_point(Point { x: 20.0, y: 20.0 });
        
        // Commit the stroke
        assert!(canvas.commit_stroke(active_stroke).is_ok());
        assert_eq!(canvas.strokes().len(), 1);
        
        // Verify the committed stroke
        let committed_stroke = &canvas.strokes()[0];
        assert_eq!(committed_stroke.color, Color::BLACK);
        assert_eq!(committed_stroke.width, 2.0);
        assert_eq!(committed_stroke.opacity, 1.0);
        assert_eq!(committed_stroke.points.len(), 2);
    }

    #[test]
    fn test_canvas_commit_empty_stroke() {
        let mut canvas = Canvas::new();
        let palette = ColorPalette::new();
        
        // Begin a stroke but don't add any points
        let active_stroke = canvas.begin_stroke_with_palette(&palette, 2.0, 1.0);
        
        // Try to commit empty stroke
        let result = canvas.commit_stroke(active_stroke);
        assert!(result.is_err());
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_canvas_multiple_strokes_with_different_colors() {
        let mut canvas = Canvas::new();
        let mut palette = ColorPalette::new();
        
        // Draw first stroke in red
        palette.select_color(2).unwrap(); // Red
        let mut stroke1 = canvas.begin_stroke_with_palette(&palette, 3.0, 1.0);
        stroke1.add_point(Point { x: 0.0, y: 0.0 });
        stroke1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(stroke1).unwrap();
        
        // Draw second stroke in blue
        palette.select_color(4).unwrap(); // Blue
        let mut stroke2 = canvas.begin_stroke_with_palette(&palette, 2.0, 0.8);
        stroke2.add_point(Point { x: 20.0, y: 20.0 });
        stroke2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(stroke2).unwrap();
        
        // Verify both strokes are present with correct colors
        assert_eq!(canvas.strokes().len(), 2);
        assert_eq!(canvas.strokes()[0].color, Color { r: 255, g: 0, b: 0, a: 255 }); // Red
        assert_eq!(canvas.strokes()[1].color, Color { r: 0, g: 0, b: 255, a: 255 }); // Blue
        assert_eq!(canvas.strokes()[0].width, 3.0);
        assert_eq!(canvas.strokes()[1].width, 2.0);
    }
}