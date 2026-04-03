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
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}

/// Maximum number of custom colors that can be saved
pub const MAX_CUSTOM_COLORS: usize = 10;

/// Represents HSV color values (Hue, Saturation, Value)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvColor {
    pub h: f32, // Hue: 0.0 to 360.0
    pub s: f32, // Saturation: 0.0 to 100.0
    pub v: f32, // Value: 0.0 to 100.0
}

impl Default for HsvColor {
    fn default() -> Self {
        Self {
            h: 0.0,   // Red
            s: 100.0, // Full saturation
            v: 100.0, // Full value
        }
    }
}

impl HsvColor {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }

    pub fn to_rgb(&self) -> Color {
        hsv_to_rgb(self.h, self.s, self.v)
    }
}

/// Convert HSV to RGB color
/// h: 0-360, s: 0-100, v: 0-100
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let s = s / 100.0;
    let v = v / 100.0;

    let h_norm = h / 60.0;
    let i = h_norm.floor() as i32 % 6;
    let f = h_norm - h_norm.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color {
        r: (r * 255.0).round() as u8,
        g: (g * 255.0).round() as u8,
        b: (b * 255.0).round() as u8,
        a: 255,
    }
}

/// Convert RGB to HSV
pub fn rgb_to_hsv(color: Color) -> HsvColor {
    let r = color.r as f32 / 255.0;
    let g = color.g as f32 / 255.0;
    let b = color.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max * 100.0;

    let s = if max.abs() < f32::EPSILON {
        0.0
    } else {
        delta / max * 100.0
    };

    let h = if delta.abs() < f32::EPSILON {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < f32::EPSILON {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    HsvColor { h, s, v }
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
    /// Undo history for strokes (also used for redo)
    undo_stack: Vec<Stroke>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            background_color: Color::WHITE,
            strokes: Vec::new(),
            undo_stack: Vec::new(),
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
        self.undo_stack.clear();
    }

    /// Clear all strokes from the canvas
    pub fn clear(&mut self) {
        self.strokes.clear();
        self.undo_stack.clear();
    }

    /// Get canvas dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get background color
    pub fn background_color(&self) -> Color {
        self.background_color
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

    /// Check if there are strokes available to undo
    pub fn can_undo(&self) -> bool {
        !self.strokes.is_empty()
    }

    /// Check if there are strokes available to redo
    pub fn can_redo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
}

/// Default brush sizes available in the application
pub const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

/// Cycle brush size up (larger) - returns new size
/// If already at max, stays at max
pub fn brush_size_up(current: f32) -> f32 {
    match BRUSH_SIZES
        .iter()
        .position(|&s| (s - current).abs() < f32::EPSILON)
    {
        Some(pos) if pos < BRUSH_SIZES.len() - 1 => BRUSH_SIZES[pos + 1],
        _ => current,
    }
}

/// Cycle brush size down (smaller) - returns new size
/// If already at min, stays at min
pub fn brush_size_down(current: f32) -> f32 {
    match BRUSH_SIZES
        .iter()
        .position(|&s| (s - current).abs() < f32::EPSILON)
    {
        Some(pos) if pos > 0 => BRUSH_SIZES[pos - 1],
        _ => current,
    }
}

/// Default opacity presets available in the application
pub const OPACITY_PRESETS: [f32; 4] = [0.25, 0.5, 0.75, 1.0];

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

    const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new();
        assert_eq!(canvas.size(), (1280, 720));
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
    fn test_active_stroke_creation() {
        let active_stroke = ActiveStroke::new(RED, 3.0, 0.8);

        assert_eq!(active_stroke.color(), RED);
        assert_eq!(active_stroke.width(), 3.0);
        assert_eq!(active_stroke.opacity(), 0.8);
        assert!(active_stroke.is_empty());
        assert_eq!(active_stroke.points().len(), 0);
    }

    #[test]
    fn test_active_stroke_point_addition() {
        let mut active_stroke = ActiveStroke::new(RED, 2.0, 1.0);

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
        let mut active_stroke = ActiveStroke::new(BLUE, 4.0, 0.5);

        active_stroke.add_point(Point { x: 0.0, y: 0.0 });
        active_stroke.add_point(Point { x: 5.0, y: 5.0 });

        let committed_stroke = active_stroke.commit().expect("Should commit successfully");

        assert_eq!(committed_stroke.color, BLUE);
        assert_eq!(committed_stroke.width, 4.0);
        assert_eq!(committed_stroke.opacity, 0.5);
        assert_eq!(committed_stroke.points.len(), 2);
        assert_eq!(committed_stroke.points[0], Point { x: 0.0, y: 0.0 });
        assert_eq!(committed_stroke.points[1], Point { x: 5.0, y: 5.0 });
    }

    #[test]
    fn test_active_stroke_commit_empty() {
        let active_stroke = ActiveStroke::new(GREEN, 1.0, 1.0);

        let result = active_stroke.commit();
        assert!(result.is_none(), "Empty stroke should not commit");
    }

    #[test]
    fn test_canvas_active_stroke_integration() {
        let mut canvas = Canvas::new();

        let mut active_stroke = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        assert_eq!(active_stroke.color(), Color::BLACK);
        assert_eq!(active_stroke.width(), 2.0);
        assert_eq!(active_stroke.opacity(), 1.0);

        active_stroke.add_point(Point { x: 10.0, y: 10.0 });
        active_stroke.add_point(Point { x: 20.0, y: 20.0 });

        assert!(canvas.commit_stroke(active_stroke).is_ok());
        assert_eq!(canvas.strokes().len(), 1);

        let committed_stroke = &canvas.strokes()[0];
        assert_eq!(committed_stroke.color, Color::BLACK);
        assert_eq!(committed_stroke.width, 2.0);
        assert_eq!(committed_stroke.opacity, 1.0);
        assert_eq!(committed_stroke.points.len(), 2);
    }

    #[test]
    fn test_canvas_commit_empty_stroke() {
        let mut canvas = Canvas::new();

        let active_stroke = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);

        let result = canvas.commit_stroke(active_stroke);
        assert!(result.is_err());
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_canvas_multiple_strokes_with_different_colors() {
        let mut canvas = Canvas::new();

        let mut stroke1 = canvas.begin_stroke(RED, 3.0, 1.0);
        stroke1.add_point(Point { x: 0.0, y: 0.0 });
        stroke1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(stroke1).unwrap();

        let mut stroke2 = canvas.begin_stroke(BLUE, 2.0, 0.8);
        stroke2.add_point(Point { x: 20.0, y: 20.0 });
        stroke2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(stroke2).unwrap();

        assert_eq!(canvas.strokes().len(), 2);
        assert_eq!(canvas.strokes()[0].color, RED);
        assert_eq!(canvas.strokes()[1].color, BLUE);
        assert_eq!(canvas.strokes()[0].width, 3.0);
        assert_eq!(canvas.strokes()[1].width, 2.0);
    }

    #[test]
    fn test_undo_on_empty_canvas() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.strokes().len(), 0);
        canvas.undo();
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_redo_with_empty_stack() {
        let mut canvas = Canvas::new();
        canvas.redo();
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_new_stroke_clears_undo_stack() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        canvas.undo();
        assert!(canvas.can_redo());
        assert_eq!(canvas.strokes().len(), 0);

        let mut s2 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s2.add_point(Point { x: 20.0, y: 20.0 });
        s2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        assert!(!canvas.can_redo());
        assert_eq!(canvas.strokes().len(), 1);
        canvas.redo();
        assert_eq!(canvas.strokes().len(), 1);
    }

    #[test]
    fn test_can_undo_can_redo() {
        let mut canvas = Canvas::new();
        assert!(!canvas.can_undo());
        assert!(!canvas.can_redo());

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s1.add_point(Point { x: 0.0, y: 0.0 });
        canvas.commit_stroke(s1).unwrap();

        assert!(canvas.can_undo());
        assert!(!canvas.can_redo());

        canvas.undo();
        assert!(!canvas.can_undo());
        assert!(canvas.can_redo());

        canvas.redo();
        assert!(canvas.can_undo());
        assert!(!canvas.can_redo());
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        let mut s2 = canvas.begin_stroke(Color::BLACK, 3.0, 1.0);
        s2.add_point(Point { x: 20.0, y: 20.0 });
        s2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        assert_eq!(canvas.strokes().len(), 2);

        canvas.undo();
        assert_eq!(canvas.strokes().len(), 1);

        canvas.redo();
        assert_eq!(canvas.strokes().len(), 2);

        canvas.undo();
        canvas.undo();
        assert_eq!(canvas.strokes().len(), 0);

        canvas.undo();
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_clear_resets_all_stacks() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        canvas.undo();
        assert_eq!(canvas.strokes().len(), 0);

        canvas.clear();
        canvas.redo();
        assert_eq!(canvas.strokes().len(), 0);
    }

    #[test]
    fn test_set_background() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.background_color(), Color::WHITE);

        canvas.set_background(Color::BLACK);
        assert_eq!(canvas.background_color(), Color::BLACK);
    }

    #[test]
    fn test_resize() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.size(), (1280, 720));

        canvas.resize(800, 600);
        assert_eq!(canvas.size(), (800, 600));
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(
            Color::WHITE,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(
            Color::BLACK,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert_eq!(
            Color::TRANSPARENT,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            }
        );
    }

    #[test]
    fn test_add_stroke_directly() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: RED,
            width: 5.0,
            opacity: 1.0,
        };
        canvas.add_stroke(stroke);
        assert_eq!(canvas.strokes().len(), 1);
    }

    #[test]
    fn test_color_equality() {
        let c1 = Color {
            r: 100,
            g: 150,
            b: 200,
            a: 255,
        };
        let c2 = Color {
            r: 100,
            g: 150,
            b: 200,
            a: 255,
        };
        let c3 = Color {
            r: 100,
            g: 150,
            b: 201,
            a: 255,
        };
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_color_constants_comprehensive() {
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::WHITE.g, 255);
        assert_eq!(Color::WHITE.b, 255);
        assert_eq!(Color::WHITE.a, 255);

        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::BLACK.g, 0);
        assert_eq!(Color::BLACK.b, 0);
        assert_eq!(Color::BLACK.a, 255);

        assert_eq!(Color::TRANSPARENT.a, 0);
    }

    #[test]
    fn test_multiple_undo_redo_cycles() {
        let mut canvas = Canvas::new();

        for i in 0..5 {
            let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
            s.add_point(Point {
                x: i as f32 * 10.0,
                y: i as f32 * 10.0,
            });
            canvas.commit_stroke(s).unwrap();
        }
        assert_eq!(canvas.strokes().len(), 5);
        assert!(canvas.can_undo());
        assert!(!canvas.can_redo());

        for _ in 0..3 {
            canvas.undo();
        }
        assert_eq!(canvas.strokes().len(), 2);
        assert!(canvas.can_undo());
        assert!(canvas.can_redo());

        canvas.redo();
        assert_eq!(canvas.strokes().len(), 3);

        canvas.redo();
        assert_eq!(canvas.strokes().len(), 4);
    }

    #[test]
    fn test_stroke_iteration() {
        let mut canvas = Canvas::new();

        for i in 0..3 {
            let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
            s.add_point(Point {
                x: i as f32,
                y: i as f32,
            });
            canvas.commit_stroke(s).unwrap();
        }

        let mut count = 0;
        for stroke in canvas.strokes() {
            assert!(!stroke.points.is_empty());
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_active_stroke_with_opacity() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 5.0, 0.5);
        s.add_point(Point { x: 0.0, y: 0.0 });
        s.add_point(Point { x: 10.0, y: 10.0 });

        assert_eq!(s.width(), 5.0);
        assert_eq!(s.opacity(), 0.5);
        assert_eq!(s.points().len(), 2);

        canvas.commit_stroke(s).unwrap();
        assert_eq!(canvas.strokes().len(), 1);
        assert_eq!(canvas.strokes()[0].opacity, 0.5);
    }

    #[test]
    fn test_canvas_clear_with_active_stroke() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0);
        s.add_point(Point { x: 0.0, y: 0.0 });
        canvas.commit_stroke(s).unwrap();

        canvas.clear();
        assert_eq!(canvas.strokes().len(), 0);
        assert!(!canvas.can_undo());
        assert!(!canvas.can_redo());
    }

    #[test]
    fn test_stroke_with_many_points() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 3.0, 1.0);
        for i in 0..100 {
            s.add_point(Point {
                x: i as f32,
                y: i as f32,
            });
        }
        assert_eq!(s.points().len(), 100);

        canvas.commit_stroke(s).unwrap();
        assert_eq!(canvas.strokes().len(), 1);
        assert_eq!(canvas.strokes()[0].points.len(), 100);
    }

    // --- brush_size_up/down tests ---

    #[test]
    fn test_brush_size_up_middle() {
        assert_eq!(brush_size_up(5.0), 10.0);
    }

    #[test]
    fn test_brush_size_up_at_max() {
        assert_eq!(brush_size_up(50.0), 50.0);
    }

    #[test]
    fn test_brush_size_down_middle() {
        assert_eq!(brush_size_down(10.0), 5.0);
    }

    #[test]
    fn test_brush_size_down_at_min() {
        assert_eq!(brush_size_down(3.0), 3.0);
    }

    #[test]
    fn test_brush_size_invalid_current() {
        assert_eq!(brush_size_up(7.0), 7.0);
        assert_eq!(brush_size_down(7.0), 7.0);
    }

    #[test]
    fn test_opacity_presets_constant() {
        assert_eq!(OPACITY_PRESETS.len(), 4);
        assert!(OPACITY_PRESETS.contains(&0.25));
        assert!(OPACITY_PRESETS.contains(&0.5));
        assert!(OPACITY_PRESETS.contains(&0.75));
        assert!(OPACITY_PRESETS.contains(&1.0));
    }

    // --- HSV conversion tests ---

    #[test]
    fn test_hsv_to_rgb_red() {
        let color = hsv_to_rgb(0.0, 100.0, 100.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_hsv_to_rgb_green() {
        let color = hsv_to_rgb(120.0, 100.0, 100.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_hsv_to_rgb_blue() {
        let color = hsv_to_rgb(240.0, 100.0, 100.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_hsv_to_rgb_white() {
        let color = hsv_to_rgb(0.0, 0.0, 100.0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_hsv_to_rgb_black() {
        let color = hsv_to_rgb(0.0, 0.0, 0.0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let hsv = rgb_to_hsv(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        });
        assert!((hsv.h - 0.0).abs() < 1.0);
        assert!((hsv.s - 100.0).abs() < 1.0);
        assert!((hsv.v - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_rgb_to_hsv_gray() {
        let hsv = rgb_to_hsv(Color {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        });
        assert!((hsv.s - 0.0).abs() < 1.0);
        assert!((hsv.v - 50.0).abs() < 2.0);
    }

    #[test]
    fn test_hsv_roundtrip() {
        let original = hsv_to_rgb(180.0, 75.0, 90.0);
        let hsv = rgb_to_hsv(original);
        let roundtrip = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
        assert_eq!(original.r, roundtrip.r);
        assert_eq!(original.g, roundtrip.g);
        assert_eq!(original.b, roundtrip.b);
    }
}
