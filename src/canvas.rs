//! Canvas module for Rancer
//!
//! Provides the core canvas functionality for digital art operations.
//! This is a placeholder implementation that will be expanded with
//! actual drawing, rendering, and GPU integration.

use serde::{Deserialize, Serialize};

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
    /// Type of brush used for this stroke
    pub brush_type: BrushType,
}

/// Maximum number of layers allowed
pub const MAX_LAYERS: usize = 20;

/// Represents a single layer in the canvas
#[derive(Debug, Clone)]
pub struct Layer {
    /// Name of the layer
    pub name: String,
    /// Strokes on this layer
    pub strokes: Vec<Stroke>,
    /// Whether the layer is visible
    pub visible: bool,
    /// Opacity of the layer (0.0 to 1.0)
    pub opacity: f32,
    /// Whether the layer is locked (cannot draw on it)
    pub locked: bool,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: "Layer 1".to_string(),
            strokes: Vec::new(),
            visible: true,
            opacity: 1.0,
            locked: false,
        }
    }
}

impl Layer {
    /// Create a new layer with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Clear all strokes from this layer
    pub fn clear(&mut self) {
        self.strokes.clear();
    }
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
    /// Layers containing strokes (bottom to top)
    layers: Vec<Layer>,
    /// Currently active layer index
    active_layer: usize,
    /// Undo stack: stores (layer_index, stroke) tuples
    undo_stack: Vec<(usize, Stroke)>,
    /// Active selection (rectangular, point-based)
    selection: Option<Selection>,
    /// Version counter for cache invalidation
    version: u64,
}

/// Represents a rectangular selection of stroke segments
#[derive(Debug, Clone)]
pub struct Selection {
    /// Selection rectangle in canvas coordinates
    pub rect: Rect,
    /// Selected stroke segments (moved versions)
    pub strokes: Vec<Stroke>,
    /// Original layer index for each selected stroke
    pub original_layer_indices: Vec<usize>,
    /// Original stroke data for each layer (stored for clear/commit)
    /// Each entry: (layer_index, Vec<Stroke>) — the strokes that were removed from that layer
    pub removed_strokes: Vec<(usize, Vec<Stroke>)>,
}

/// Represents a rectangle in canvas space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        let (rx, ry, rw, rh) = self.normalized();
        px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
    }

    /// Returns (x, y, w, h) with w and h always positive
    pub fn normalized(&self) -> (f32, f32, f32, f32) {
        let x = if self.w < 0.0 {
            self.x + self.w
        } else {
            self.x
        };
        let y = if self.h < 0.0 {
            self.y + self.h
        } else {
            self.y
        };
        let w = self.w.abs();
        let h = self.h.abs();
        (x, y, w, h)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            background_color: Color::WHITE,
            layers: vec![Layer::new("Background".to_string())],
            active_layer: 0,
            undo_stack: Vec::new(),
            selection: None,
            version: 0,
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
        self.invalidate();
    }

    /// Set the background color
    pub fn set_background(&mut self, color: Color) {
        self.background_color = color;
        self.invalidate();
    }

    /// Clear all strokes from the canvas
    pub fn clear(&mut self) {
        for layer in &mut self.layers {
            layer.strokes.clear();
        }
        self.undo_stack.clear();
        self.invalidate();
    }

    /// Get canvas dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get background color
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Get the current canvas version (for cache invalidation)
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Invalidate the canvas cache by incrementing the version counter
    fn invalidate(&mut self) {
        self.version += 1;
    }

    /// Get all layers
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Get the number of layers
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get the active layer index
    pub fn active_layer(&self) -> usize {
        self.active_layer
    }

    /// Set the active layer index
    pub fn set_active_layer(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.active_layer = index;
        Ok(())
    }

    /// Add a new layer with the given name
    pub fn add_layer(&mut self, name: Option<String>) -> Result<(), String> {
        if self.layers.len() >= MAX_LAYERS {
            return Err("Maximum number of layers reached".to_string());
        }
        let layer_name = name.unwrap_or_else(|| format!("Layer {}", self.layers.len()));
        self.layers.push(Layer::new(layer_name));
        self.invalidate();
        Ok(())
    }

    /// Remove a layer at the given index (cannot remove background layer 0)
    pub fn remove_layer(&mut self, index: usize) -> Result<(), String> {
        if index == 0 {
            return Err("Cannot remove background layer".to_string());
        }
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers.remove(index);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
        self.invalidate();
        Ok(())
    }

    /// Move a layer from one position to another
    pub fn move_layer(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from >= self.layers.len() || to >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        // Update active layer if needed
        if self.active_layer == from {
            self.active_layer = to;
        } else if from < self.active_layer && to >= self.active_layer {
            self.active_layer -= 1;
        } else if from > self.active_layer && to <= self.active_layer {
            self.active_layer += 1;
        }
        self.invalidate();
        Ok(())
    }

    /// Toggle layer visibility
    pub fn toggle_layer_visibility(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].visible = !self.layers[index].visible;
        self.invalidate();
        Ok(())
    }

    /// Set layer opacity
    pub fn set_layer_opacity(&mut self, index: usize, opacity: f32) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].opacity = opacity.clamp(0.0, 1.0);
        self.invalidate();
        Ok(())
    }

    /// Toggle layer lock
    pub fn toggle_layer_lock(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].locked = !self.layers[index].locked;
        self.invalidate();
        Ok(())
    }

    /// Clear strokes on a specific layer
    pub fn clear_layer(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].strokes.clear();
        Ok(())
    }

    /// Get mutable reference to active layer
    pub fn active_layer_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.active_layer]
    }

    /// Get strokes from a specific layer (for testing)
    #[cfg(test)]
    pub fn layer_strokes(&self, layer_index: usize) -> &[Stroke] {
        if layer_index < self.layers.len() {
            &self.layers[layer_index].strokes
        } else {
            &[]
        }
    }

    /// Add a stroke to the active layer
    pub fn add_stroke_to_active_layer(&mut self, stroke: Stroke) {
        self.layers[self.active_layer].strokes.push(stroke);
        self.undo_stack.clear();
        self.invalidate();
    }

    /// Add a stroke to a specific layer (for testing)
    #[cfg(test)]
    pub fn add_stroke_to_layer(&mut self, stroke: Stroke, layer_index: usize) {
        if layer_index < self.layers.len() {
            self.layers[layer_index].strokes.push(stroke);
            self.invalidate();
        }
    }

    /// Undo the last stroke on the active layer
    pub fn undo(&mut self) {
        if let Some(stroke) = self.layers[self.active_layer].strokes.pop() {
            self.undo_stack.push((self.active_layer, stroke));
            self.invalidate();
        }
    }

    /// Redo the last undone stroke
    pub fn redo(&mut self) {
        if let Some((layer_index, stroke)) = self.undo_stack.pop()
            && layer_index < self.layers.len()
        {
            self.layers[layer_index].strokes.push(stroke);
            self.invalidate();
        }
    }

    /// Check if there are strokes available to undo on active layer
    pub fn can_undo(&self) -> bool {
        !self.layers[self.active_layer].strokes.is_empty()
    }

    /// Check if there are strokes available to redo
    pub fn can_redo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Get all strokes from all visible layers (for rendering)
    /// Layers are rendered back-to-front (bottom layer first, top layer last)
    pub fn all_strokes(&self) -> Vec<(&Stroke, f32)> {
        let mut result = Vec::new();
        for layer in self.layers.iter().rev() {
            if layer.visible {
                for stroke in &layer.strokes {
                    if stroke.points.len() >= 2 {
                        result.push((stroke, layer.opacity));
                    }
                }
            }
        }
        result
    }

    /// Check if active layer is locked
    pub fn is_active_layer_locked(&self) -> bool {
        self.layers[self.active_layer].locked
    }

    // ─── Selection ──────────────────────────────────────────────

    /// Create a selection from all strokes that have at least one point
    /// within the given rectangle. Entire strokes are selected (not partial).
    pub fn begin_selection(&mut self, rect: Rect) {
        let mut strokes = Vec::new();
        let mut original_layer_indices = Vec::new();
        let mut removed_strokes: Vec<(usize, Vec<Stroke>)> = Vec::new();

        for (layer_idx, layer) in self.layers.iter_mut().enumerate() {
            if !layer.visible {
                continue;
            }
            let mut layer_removed: Vec<Stroke> = Vec::new();
            let mut kept_strokes: Vec<Stroke> = Vec::new();

            for stroke in layer.strokes.drain(..) {
                // Check if any point of this stroke is inside the rect
                let has_point_in_rect = stroke.points.iter().any(|p| rect.contains(p.x, p.y));
                if has_point_in_rect {
                    // Select the entire stroke
                    layer_removed.push(stroke.clone());
                    strokes.push(stroke);
                    original_layer_indices.push(layer_idx);
                } else {
                    // Keep this stroke in the layer
                    kept_strokes.push(stroke);
                }
            }

            layer.strokes = kept_strokes;
            if !layer_removed.is_empty() {
                removed_strokes.push((layer_idx, layer_removed));
            }
        }

        if strokes.is_empty() {
            self.selection = None;
        } else {
            self.selection = Some(Selection {
                rect,
                strokes,
                original_layer_indices,
                removed_strokes,
            });
        }
        self.invalidate();
    }

    /// Offset all selected stroke points by the given delta.
    pub fn move_selection(&mut self, dx: f32, dy: f32) {
        if let Some(ref mut selection) = self.selection {
            for stroke in &mut selection.strokes {
                for point in &mut stroke.points {
                    point.x += dx;
                    point.y += dy;
                }
            }
            selection.rect = Rect::new(
                selection.rect.x + dx,
                selection.rect.y + dy,
                selection.rect.w,
                selection.rect.h,
            );
        }
        self.invalidate();
    }

    /// Duplicate the current selection and add the copies to the active layer.
    /// The original strokes remain unchanged.
    pub fn copy_selection(&mut self) {
        if let Some(ref selection) = self.selection {
            for stroke in &selection.strokes {
                self.layers[self.active_layer].strokes.push(stroke.clone());
            }
        }
        self.invalidate();
    }

    /// Commit the selection: add the (possibly moved) selected strokes
    /// to the active layer. Originals were already removed on begin_selection.
    pub fn commit_selection(&mut self) {
        if let Some(selection) = self.selection.take() {
            for stroke in &selection.strokes {
                self.layers[self.active_layer].strokes.push(stroke.clone());
            }
            // removed_strokes are NOT restored — the moved strokes replace them
        }
        self.invalidate();
    }

    /// Discard the selection without committing any changes.
    /// Restores the original strokes to their layers.
    pub fn clear_selection(&mut self) {
        if let Some(selection) = self.selection.take() {
            for (layer_idx, removed) in selection.removed_strokes {
                if layer_idx < self.layers.len() {
                    self.layers[layer_idx].strokes.extend(removed);
                }
            }
        }
        self.invalidate();
    }

    /// Returns a reference to the active selection, if any.
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Returns true if there is an active selection.
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }
}

/// Default brush sizes available in the application
pub const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

/// Available brush types for drawing
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum BrushType {
    #[default]
    Square,
    Round,
    Spray,
    Calligraphy,
}

impl std::str::FromStr for BrushType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Square" => Ok(BrushType::Square),
            "Round" => Ok(BrushType::Round),
            "Spray" => Ok(BrushType::Spray),
            "Calligraphy" => Ok(BrushType::Calligraphy),
            _ => Err(()),
        }
    }
}

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
    /// Type of brush used for this stroke
    brush_type: BrushType,
}

impl ActiveStroke {
    /// Create a new active stroke with the given properties
    pub fn new(color: Color, width: f32, opacity: f32, brush_type: BrushType) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
            opacity,
            brush_type,
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

    /// Get the brush type of the active stroke
    pub fn brush_type(&self) -> BrushType {
        self.brush_type
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
                brush_type: self.brush_type,
            })
        }
    }
}

impl Canvas {
    /// Begin a new active stroke with the specified properties
    pub fn begin_stroke(
        &mut self,
        color: Color,
        width: f32,
        opacity: f32,
        brush_type: BrushType,
    ) -> ActiveStroke {
        ActiveStroke::new(color, width, opacity, brush_type)
    }

    /// Commit an active stroke to the active layer
    pub fn commit_stroke(&mut self, active_stroke: ActiveStroke) -> Result<(), String> {
        if let Some(stroke) = active_stroke.commit() {
            self.add_stroke_to_active_layer(stroke);
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
        assert_eq!(canvas.layers()[0].strokes.len(), 0);
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
            brush_type: BrushType::default(),
        };

        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
    }

    #[test]
    fn test_active_stroke_creation() {
        let active_stroke = ActiveStroke::new(RED, 3.0, 0.8, BrushType::default());

        assert_eq!(active_stroke.color(), RED);
        assert_eq!(active_stroke.width(), 3.0);
        assert_eq!(active_stroke.opacity(), 0.8);
        assert!(active_stroke.is_empty());
        assert_eq!(active_stroke.points().len(), 0);
    }

    #[test]
    fn test_active_stroke_point_addition() {
        let mut active_stroke = ActiveStroke::new(RED, 2.0, 1.0, BrushType::default());

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
        let mut active_stroke = ActiveStroke::new(BLUE, 4.0, 0.5, BrushType::default());

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
        let active_stroke = ActiveStroke::new(GREEN, 1.0, 1.0, BrushType::default());

        let result = active_stroke.commit();
        assert!(result.is_none(), "Empty stroke should not commit");
    }

    #[test]
    fn test_canvas_active_stroke_integration() {
        let mut canvas = Canvas::new();

        let mut active_stroke = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        assert_eq!(active_stroke.color(), Color::BLACK);
        assert_eq!(active_stroke.width(), 2.0);
        assert_eq!(active_stroke.opacity(), 1.0);

        active_stroke.add_point(Point { x: 10.0, y: 10.0 });
        active_stroke.add_point(Point { x: 20.0, y: 20.0 });

        assert!(canvas.commit_stroke(active_stroke).is_ok());

        let all_strokes = canvas.all_strokes();
        assert_eq!(all_strokes.len(), 1);

        let (committed_stroke, _) = &all_strokes[0];
        assert_eq!(committed_stroke.color, Color::BLACK);
        assert_eq!(committed_stroke.width, 2.0);
        assert_eq!(committed_stroke.opacity, 1.0);
        assert_eq!(committed_stroke.points.len(), 2);
    }

    #[test]
    fn test_canvas_commit_empty_stroke() {
        let mut canvas = Canvas::new();

        let active_stroke = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());

        let result = canvas.commit_stroke(active_stroke);
        assert!(result.is_err());
        assert_eq!(canvas.all_strokes().len(), 0);
    }

    #[test]
    fn test_canvas_multiple_strokes_with_different_colors() {
        let mut canvas = Canvas::new();

        let mut stroke1 = canvas.begin_stroke(RED, 3.0, 1.0, BrushType::default());
        stroke1.add_point(Point { x: 0.0, y: 0.0 });
        stroke1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(stroke1).unwrap();

        let mut stroke2 = canvas.begin_stroke(BLUE, 2.0, 0.8, BrushType::default());
        stroke2.add_point(Point { x: 20.0, y: 20.0 });
        stroke2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(stroke2).unwrap();

        let all_strokes = canvas.all_strokes();
        assert_eq!(all_strokes.len(), 2);
        assert_eq!(all_strokes[0].0.color, RED);
        assert_eq!(all_strokes[1].0.color, BLUE);
        assert_eq!(all_strokes[0].0.width, 3.0);
        assert_eq!(all_strokes[1].0.width, 2.0);
    }

    #[test]
    fn test_undo_on_empty_canvas() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.all_strokes().len(), 0);
        canvas.undo();
        assert_eq!(canvas.all_strokes().len(), 0);
    }

    #[test]
    fn test_redo_with_empty_stack() {
        let mut canvas = Canvas::new();
        canvas.redo();
        assert_eq!(canvas.all_strokes().len(), 0);
    }

    #[test]
    fn test_new_stroke_clears_undo_stack() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        canvas.undo();
        assert_eq!(canvas.all_strokes().len(), 0);

        let mut s2 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s2.add_point(Point { x: 20.0, y: 20.0 });
        s2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        assert_eq!(canvas.all_strokes().len(), 1);
    }

    #[test]
    fn test_can_undo_can_redo() {
        let mut canvas = Canvas::new();
        assert!(!canvas.can_undo());
        assert!(!canvas.can_redo());

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        canvas.commit_stroke(s1).unwrap();

        assert!(canvas.can_undo());
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        let mut s2 = canvas.begin_stroke(Color::BLACK, 3.0, 1.0, BrushType::default());
        s2.add_point(Point { x: 20.0, y: 20.0 });
        s2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        assert_eq!(canvas.all_strokes().len(), 2);

        canvas.undo();
        assert_eq!(canvas.all_strokes().len(), 1);

        canvas.redo();
        assert_eq!(canvas.all_strokes().len(), 2);
    }

    #[test]
    fn test_undo_redo_with_multiple_strokes() {
        let mut canvas = Canvas::new();

        for i in 0..5 {
            let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
            s.add_point(Point {
                x: i as f32 * 10.0,
                y: i as f32 * 10.0,
            });
            s.add_point(Point {
                x: i as f32 * 10.0 + 1.0,
                y: i as f32 * 10.0 + 1.0,
            });
            canvas.commit_stroke(s).unwrap();
        }
        assert_eq!(canvas.all_strokes().len(), 5);
        assert!(canvas.can_undo());
        assert!(!canvas.can_redo());

        for _ in 0..3 {
            canvas.undo();
        }
        assert_eq!(canvas.all_strokes().len(), 2);
        assert!(canvas.can_undo());
        assert!(canvas.can_redo());

        canvas.redo();
        assert_eq!(canvas.all_strokes().len(), 3);

        canvas.redo();
        assert_eq!(canvas.all_strokes().len(), 4);
    }

    #[test]
    fn test_clear_resets_all_stacks() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        canvas.undo();
        assert_eq!(canvas.all_strokes().len(), 0);

        canvas.clear();
        assert_eq!(canvas.all_strokes().len(), 0);
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
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.all_strokes().len(), 1);
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
    fn test_stroke_iteration() {
        let mut canvas = Canvas::new();

        for i in 0..3 {
            let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
            s.add_point(Point {
                x: i as f32,
                y: i as f32,
            });
            s.add_point(Point {
                x: i as f32 + 1.0,
                y: i as f32 + 1.0,
            });
            canvas.commit_stroke(s).unwrap();
        }
        assert_eq!(canvas.all_strokes().len(), 3);
    }

    #[test]
    fn test_active_stroke_with_opacity() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 5.0, 0.5, BrushType::default());
        s.add_point(Point { x: 0.0, y: 0.0 });
        s.add_point(Point { x: 10.0, y: 10.0 });

        assert_eq!(s.width(), 5.0);
        assert_eq!(s.opacity(), 0.5);
        assert_eq!(s.points().len(), 2);

        canvas.commit_stroke(s).unwrap();
        let all_strokes = canvas.all_strokes();
        assert_eq!(all_strokes.len(), 1);
        assert_eq!(all_strokes[0].0.opacity, 0.5);
    }

    #[test]
    fn test_canvas_clear_with_active_stroke() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s.add_point(Point { x: 0.0, y: 0.0 });
        canvas.commit_stroke(s).unwrap();

        canvas.clear();
        assert_eq!(canvas.all_strokes().len(), 0);
        assert!(!canvas.can_undo());
        assert!(!canvas.can_redo());
    }

    #[test]
    fn test_stroke_with_many_points() {
        let mut canvas = Canvas::new();

        let mut s = canvas.begin_stroke(Color::BLACK, 3.0, 1.0, BrushType::default());
        for i in 0..100 {
            s.add_point(Point {
                x: i as f32,
                y: i as f32,
            });
        }
        assert_eq!(s.points().len(), 100);

        canvas.commit_stroke(s).unwrap();
        assert_eq!(canvas.all_strokes().len(), 1);
        assert_eq!(canvas.all_strokes()[0].0.points.len(), 100);
    }

    // --- Layer tests ---

    #[test]
    fn test_layer_default_values() {
        let layer = Layer::default();
        assert_eq!(layer.name, "Layer 1");
        assert!(layer.visible);
        assert_eq!(layer.opacity, 1.0);
        assert!(!layer.locked);
        assert!(layer.strokes.is_empty());
    }

    #[test]
    fn test_canvas_starts_with_background_layer() {
        let canvas = Canvas::new();
        assert_eq!(canvas.layer_count(), 1);
        assert_eq!(canvas.active_layer(), 0);
        assert_eq!(canvas.layers()[0].name, "Background");
    }

    #[test]
    fn test_add_layer() {
        let mut canvas = Canvas::new();
        assert!(canvas.add_layer(Some("TestLayer".to_string())).is_ok());
        assert_eq!(canvas.layer_count(), 2);
        assert_eq!(canvas.layers()[1].name, "TestLayer");
    }

    #[test]
    fn test_add_layer_default_name() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.layers()[1].name, "Layer 1");
    }

    #[test]
    fn test_add_layer_max_limit() {
        let mut canvas = Canvas::new();
        for _ in 0..19 {
            canvas.add_layer(None).unwrap();
        }
        assert_eq!(canvas.layer_count(), 20);
        assert!(canvas.add_layer(None).is_err());
    }

    #[test]
    fn test_remove_layer() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.layer_count(), 3);
        assert!(canvas.remove_layer(1).is_ok());
        assert_eq!(canvas.layer_count(), 2);
    }

    #[test]
    fn test_cannot_remove_background_layer() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert!(canvas.remove_layer(0).is_err());
    }

    #[test]
    fn test_remove_layer_adjusts_active() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        canvas.add_layer(None).unwrap();
        canvas.set_active_layer(2).unwrap();
        canvas.remove_layer(2).unwrap();
        assert_eq!(canvas.active_layer(), 1);
    }

    #[test]
    fn test_move_layer() {
        let mut canvas = Canvas::new();
        canvas.add_layer(Some("A".to_string())).unwrap();
        canvas.add_layer(Some("B".to_string())).unwrap();
        // Initial: [Background, A, B]
        canvas.move_layer(0, 2).unwrap();
        // After moving index 0 to index 2: [A, B, Background]
        assert_eq!(canvas.layers()[0].name, "A");
        assert_eq!(canvas.layers()[1].name, "B");
        assert_eq!(canvas.layers()[2].name, "Background");
    }

    #[test]
    fn test_toggle_layer_visibility() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert!(canvas.layers()[1].visible);
        canvas.toggle_layer_visibility(1).unwrap();
        assert!(!canvas.layers()[1].visible);
    }

    #[test]
    fn test_set_layer_opacity() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        canvas.set_layer_opacity(1, 0.5).unwrap();
        assert_eq!(canvas.layers()[1].opacity, 0.5);
    }

    #[test]
    fn test_set_layer_opacity_clamped() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        canvas.set_layer_opacity(1, 2.0).unwrap();
        assert_eq!(canvas.layers()[1].opacity, 1.0);
        canvas.set_layer_opacity(1, -1.0).unwrap();
        assert_eq!(canvas.layers()[1].opacity, 0.0);
    }

    #[test]
    fn test_toggle_layer_lock() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert!(!canvas.layers()[1].locked);
        canvas.toggle_layer_lock(1).unwrap();
        assert!(canvas.layers()[1].locked);
    }

    #[test]
    fn test_clear_layer() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        canvas.clear_layer(0).unwrap();
        assert_eq!(canvas.layers()[0].strokes.len(), 0);
    }

    #[test]
    fn test_all_strokes_respects_visibility() {
        let mut canvas = Canvas::new();
        let stroke1 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        canvas.add_layer(None).unwrap();
        let stroke2 = Stroke {
            points: vec![Point { x: 20.0, y: 20.0 }, Point { x: 30.0, y: 30.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke2, 1);
        assert_eq!(canvas.all_strokes().len(), 2);
        canvas.toggle_layer_visibility(0).unwrap();
        assert_eq!(canvas.all_strokes().len(), 1);
    }

    #[test]
    fn test_all_strokes_applies_layer_opacity() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        canvas.set_layer_opacity(0, 0.5).unwrap();
        let strokes = canvas.all_strokes();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].1, 0.5);
    }

    #[test]
    fn test_undo_redo_with_multiple_layers() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        canvas.set_active_layer(0).unwrap();
        let mut s1 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 10.0, y: 10.0 });
        canvas.commit_stroke(s1).unwrap();

        canvas.set_active_layer(1).unwrap();
        let mut s2 = canvas.begin_stroke(Color::BLACK, 2.0, 1.0, BrushType::default());
        s2.add_point(Point { x: 20.0, y: 20.0 });
        s2.add_point(Point { x: 30.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        assert_eq!(canvas.all_strokes().len(), 2);

        canvas.set_active_layer(1).unwrap();
        canvas.undo();
        assert_eq!(canvas.all_strokes().len(), 1);

        canvas.redo();
        assert_eq!(canvas.all_strokes().len(), 2);
    }

    #[test]
    fn test_is_active_layer_locked() {
        let mut canvas = Canvas::new();
        assert!(!canvas.is_active_layer_locked());
        canvas.toggle_layer_lock(0).unwrap();
        assert!(canvas.is_active_layer_locked());
    }

    #[test]
    fn test_clear_clears_all_layers() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        let stroke1 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        let stroke2 = Stroke {
            points: vec![Point { x: 20.0, y: 20.0 }, Point { x: 30.0, y: 30.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke2, 1);
        assert_eq!(canvas.all_strokes().len(), 2);
        canvas.clear();
        assert_eq!(canvas.all_strokes().len(), 0);
        assert!(!canvas.can_redo());
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

    // ─── Selection Tests ──────────────────────────────────────────────

    #[test]
    fn test_selection_captures_strokes_in_rect() {
        let mut canvas = Canvas::new();
        // Add a stroke fully inside the selection rect
        let stroke1 = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        // Add a stroke outside the selection rect
        let stroke2 = Stroke {
            points: vec![Point { x: 200.0, y: 200.0 }, Point { x: 210.0, y: 210.0 }],
            color: BLUE,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        canvas.add_stroke_to_layer(stroke2, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);

        assert!(canvas.has_selection());
        let selection = canvas.selection().unwrap();
        assert_eq!(selection.strokes.len(), 1);
        assert_eq!(selection.strokes[0].color, RED);
        // stroke2 should remain in the layer
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        assert_eq!(canvas.layers()[0].strokes[0].color, BLUE);
    }

    #[test]
    fn test_selection_captures_partial_overlap() {
        let mut canvas = Canvas::new();
        // Stroke with one point inside rect, one outside — entire stroke should be selected
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 150.0, y: 150.0 }],
            color: GREEN,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);

        assert!(canvas.has_selection());
        let selection = canvas.selection().unwrap();
        assert_eq!(selection.strokes.len(), 1);
        assert_eq!(selection.strokes[0].color, GREEN);
        // Layer should be empty (entire stroke was selected)
        assert_eq!(canvas.layers()[0].strokes.len(), 0);
    }

    #[test]
    fn test_selection_move_offsets_points() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.move_selection(10.0, 20.0);

        let selection = canvas.selection().unwrap();
        assert_eq!(selection.strokes[0].points[0].x, 60.0);
        assert_eq!(selection.strokes[0].points[0].y, 70.0);
        assert_eq!(selection.strokes[0].points[1].x, 70.0);
        assert_eq!(selection.strokes[0].points[1].y, 80.0);
        // Rect should also be offset
        assert_eq!(selection.rect.x, 10.0);
        assert_eq!(selection.rect.y, 20.0);
    }

    #[test]
    fn test_selection_copy_duplicates_strokes() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.copy_selection();

        // Original stroke was removed from layer, copy added to active layer
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        assert_eq!(canvas.layers()[0].strokes[0].color, RED);
        // Selection still active
        assert!(canvas.has_selection());
    }

    #[test]
    fn test_selection_commit_adds_moved_strokes() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.move_selection(100.0, 100.0);
        canvas.commit_selection();

        // Selection cleared after commit
        assert!(!canvas.has_selection());
        // Moved stroke added to active layer
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        assert_eq!(canvas.layers()[0].strokes[0].points[0].x, 150.0);
        assert_eq!(canvas.layers()[0].strokes[0].points[0].y, 150.0);
    }

    #[test]
    fn test_selection_clear_restores_originals() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.move_selection(10.0, 10.0);
        canvas.clear_selection();

        // Selection cleared
        assert!(!canvas.has_selection());
        // Original stroke restored
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        assert_eq!(canvas.layers()[0].strokes[0].points[0].x, 50.0);
        assert_eq!(canvas.layers()[0].strokes[0].points[0].y, 50.0);
    }

    #[test]
    fn test_selection_respects_layer_visibility() {
        let mut canvas = Canvas::new();
        let stroke1 = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let stroke2 = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: BLUE,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        canvas.add_stroke_to_layer(stroke2, 0);

        // Make layer invisible
        canvas.toggle_layer_visibility(0).unwrap();

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);

        // No selection because layer is invisible
        assert!(!canvas.has_selection());
        // Both strokes remain in layer
        assert_eq!(canvas.layers()[0].strokes.len(), 2);
    }

    #[test]
    fn test_selection_empty_rect_creates_no_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 200.0, y: 200.0 }, Point { x: 210.0, y: 210.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        canvas.begin_selection(rect);

        assert!(!canvas.has_selection());
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
    }

    #[test]
    fn test_selection_multiple_strokes_from_same_layer() {
        let mut canvas = Canvas::new();
        let stroke1 = Stroke {
            points: vec![Point { x: 10.0, y: 10.0 }, Point { x: 20.0, y: 20.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let stroke2 = Stroke {
            points: vec![Point { x: 30.0, y: 30.0 }, Point { x: 40.0, y: 40.0 }],
            color: BLUE,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let stroke3 = Stroke {
            points: vec![Point { x: 200.0, y: 200.0 }, Point { x: 210.0, y: 210.0 }],
            color: GREEN,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        canvas.add_stroke_to_layer(stroke2, 0);
        canvas.add_stroke_to_layer(stroke3, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);

        assert!(canvas.has_selection());
        let selection = canvas.selection().unwrap();
        assert_eq!(selection.strokes.len(), 2);
        // stroke3 should remain in layer
        assert_eq!(canvas.layers()[0].strokes.len(), 1);
        assert_eq!(canvas.layers()[0].strokes[0].color, GREEN);
    }

    #[test]
    fn test_selection_commit_after_copy() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: RED,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.copy_selection();
        canvas.commit_selection();

        // After copy + commit: original was restored, then moved stroke added
        assert!(!canvas.has_selection());
        assert_eq!(canvas.layers()[0].strokes.len(), 2);
    }

    // ─── Version tracking tests ──────────────────────────────────────────────

    #[test]
    fn test_canvas_version_starts_at_zero() {
        let canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
    }

    #[test]
    fn test_canvas_version_increments_on_resize() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
        canvas.resize(800, 600);
        assert_eq!(canvas.version(), 1);
    }

    #[test]
    fn test_canvas_version_increments_on_set_background() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
        canvas.set_background(Color::BLACK);
        assert_eq!(canvas.version(), 1);
    }

    #[test]
    fn test_canvas_version_increments_on_clear() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
        canvas.clear();
        assert_eq!(canvas.version(), 1);
    }

    #[test]
    fn test_canvas_version_increments_on_add_stroke() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.version(), 1);
    }

    #[test]
    fn test_canvas_version_increments_on_add_layer() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 1);
    }

    #[test]
    fn test_canvas_version_increments_on_remove_layer() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 1);
        canvas.remove_layer(1).unwrap();
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_move_layer() {
        let mut canvas = Canvas::new();
        canvas.add_layer(Some("A".to_string())).unwrap();
        canvas.add_layer(Some("B".to_string())).unwrap();
        assert_eq!(canvas.version(), 2);
        canvas.move_layer(0, 2).unwrap();
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_increments_on_toggle_layer_visibility() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 1);
        canvas.toggle_layer_visibility(1).unwrap();
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_set_layer_opacity() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 1);
        canvas.set_layer_opacity(1, 0.5).unwrap();
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_toggle_layer_lock() {
        let mut canvas = Canvas::new();
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 1);
        canvas.toggle_layer_lock(1).unwrap();
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_undo() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.version(), 1);
        canvas.undo();
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_redo() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        canvas.undo();
        assert_eq!(canvas.version(), 2);
        canvas.redo();
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_increments_on_begin_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.version(), 1);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        assert_eq!(canvas.version(), 2);
    }

    #[test]
    fn test_canvas_version_increments_on_move_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        assert_eq!(canvas.version(), 2);
        canvas.move_selection(10.0, 20.0);
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_increments_on_copy_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        assert_eq!(canvas.version(), 2);
        canvas.copy_selection();
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_increments_on_commit_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        canvas.commit_selection();
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_increments_on_clear_selection() {
        let mut canvas = Canvas::new();
        let stroke = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }, Point { x: 60.0, y: 60.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        canvas.begin_selection(rect);
        assert_eq!(canvas.version(), 2);
        canvas.clear_selection();
        assert_eq!(canvas.version(), 3);
    }

    #[test]
    fn test_canvas_version_multiple_operations() {
        let mut canvas = Canvas::new();
        assert_eq!(canvas.version(), 0);

        // Add strokes
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke, 0);
        assert_eq!(canvas.version(), 1);

        // Resize
        canvas.resize(800, 600);
        assert_eq!(canvas.version(), 2);

        // Add layer
        canvas.add_layer(None).unwrap();
        assert_eq!(canvas.version(), 3);

        // Undo
        canvas.undo();
        assert_eq!(canvas.version(), 4);

        // Redo
        canvas.redo();
        assert_eq!(canvas.version(), 5);

        // Multiple increments don't break anything
        canvas.clear();
        assert_eq!(canvas.version(), 6);
    }
}
