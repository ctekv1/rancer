//! Canvas module for Rancer
//!
//! Provides the core canvas functionality using raster layers.

use crate::export;
use serde::{Deserialize, Serialize};

/// Represents a 2D point in canvas space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Represents a color in RGBA format
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

pub const MAX_CUSTOM_COLORS: usize = 10;

/// Represents HSV color values (Hue, Saturation, Value)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HsvColor {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Default for HsvColor {
    fn default() -> Self {
        Self { h: 0.0, s: 100.0, v: 100.0 }
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

    HsvColor { h, s, v }
}

/// Raster image data (RGBA pixels)
#[derive(Debug, Clone)]
pub struct RasterImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl RasterImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some((
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ))
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = r;
        self.data[idx + 1] = g;
        self.data[idx + 2] = b;
        self.data[idx + 3] = a;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.data.resize((width * height * 4) as usize, 0);
    }

    pub fn fill(&mut self, color: Color) {
        let bytes = [color.r, color.g, color.b, color.a];
        for chunk in self.data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&bytes);
        }
    }
}

/// Raster layer containing a bitmap image
#[derive(Debug, Clone)]
pub struct RasterLayer {
    pub image: RasterImage,
    pub opacity: f32,
}

impl Default for RasterLayer {
    fn default() -> Self {
        Self {
            image: RasterImage::new(1280, 720),
            opacity: 1.0,
        }
    }
}

impl RasterLayer {
    pub fn new(width: u32, height: u32, opacity: f32) -> Self {
        Self {
            image: RasterImage::new(width, height),
            opacity,
        }
    }

    pub fn width(&self) -> u32 {
        self.image.width
    }

    pub fn height(&self) -> u32 {
        self.image.height
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.image.resize(width, height);
    }
}

/// Represents a single layer in the canvas (raster only)
#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub content: LayerContent,
    pub visible: bool,
    pub opacity: f32,
    pub locked: bool,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: "Layer 1".to_string(),
            content: LayerContent::default(),
            visible: true,
            opacity: 1.0,
            locked: false,
        }
    }
}

impl Layer {
    pub fn new(name: String, width: u32, height: u32, opacity: f32) -> Self {
        Self {
            name,
            content: LayerContent::new(width, height),
            visible: true,
            opacity,
            locked: false,
        }
    }

    pub fn raster_mut(&mut self) -> &mut RasterLayer {
        match &mut self.content {
            LayerContent::Raster(r) => r,
        }
    }

    pub fn raster(&self) -> &RasterLayer {
        match &self.content {
            LayerContent::Raster(r) => r,
        }
    }

    pub fn is_raster(&self) -> bool {
        true
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }
}

/// Layer content - only raster supported in this version
#[derive(Debug, Clone)]
pub enum LayerContent {
    Raster(RasterLayer),
}

impl Default for LayerContent {
    fn default() -> Self {
        LayerContent::Raster(RasterLayer::default())
    }
}

impl LayerContent {
    pub fn new(width: u32, height: u32) -> Self {
        LayerContent::Raster(RasterLayer::new(width, height, 1.0))
    }

    pub fn is_raster(&self) -> bool {
        true
    }

    pub fn opacity(&self) -> f32 {
        match self {
            LayerContent::Raster(r) => r.opacity,
        }
    }

    pub fn clear(&mut self) {
        match self {
            LayerContent::Raster(r) => r.image.fill(Color::TRANSPARENT),
        }
    }
}

/// The main canvas for drawing operations
#[derive(Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub background_color: Color,
    pub layers: Vec<Layer>,
    pub active_layer: usize,
    pub undo_stack: Vec<usize>,
    pub selection: Option<Selection>,
    pub version: u64,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            background_color: Color::WHITE,
            layers: vec![Layer::new("Background".to_string(), 1280, 720, 1.0)],
            active_layer: 0,
            undo_stack: Vec::new(),
            selection: None,
            version: 0,
        }
    }
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(width: u32, height: u32) -> Self {
        let mut canvas = Self::new();
        canvas.resize(width, height);
        canvas
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.invalidate();
    }

    pub fn set_background(&mut self, color: Color) {
        self.background_color = color;
        self.invalidate();
    }

    pub fn clear(&mut self) {
        for layer in &mut self.layers {
            layer.clear();
        }
        self.undo_stack.clear();
        self.invalidate();
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    fn invalidate(&mut self) {
        self.version += 1;
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn active_layer(&self) -> usize {
        self.active_layer
    }

    pub fn set_active_layer(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.active_layer = index;
        Ok(())
    }

    pub fn add_layer(&mut self, name: Option<String>) -> Result<(), String> {
        const MAX_LAYERS: usize = 20;
        if self.layers.len() >= MAX_LAYERS {
            return Err("Maximum number of layers reached".to_string());
        }
        let layer_name = name.unwrap_or_else(|| format!("Layer {}", self.layers.len()));
        self.layers.push(Layer::new(layer_name, self.width, self.height, 1.0));
        self.invalidate();
        Ok(())
    }

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

    pub fn move_layer(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from >= self.layers.len() || to >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
        if self.active_layer == from {
            self.active_layer = to;
        }
        self.invalidate();
        Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].visible = !self.layers[index].visible;
        self.invalidate();
        Ok(())
    }

    pub fn set_layer_opacity(&mut self, index: usize, opacity: f32) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].opacity = opacity.clamp(0.0, 1.0);
        self.invalidate();
        Ok(())
    }

    pub fn toggle_layer_lock(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].locked = !self.layers[index].locked;
        self.invalidate();
        Ok(())
    }

    pub fn clear_layer(&mut self, index: usize) -> Result<(), String> {
        if index >= self.layers.len() {
            return Err("Invalid layer index".to_string());
        }
        self.layers[index].clear();
        Ok(())
    }

    pub fn active_layer_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.active_layer]
    }

    pub fn undo(&mut self) {
        if let Some(layer_idx) = self.undo_stack.pop() {
            self.invalidate();
        }
    }

    pub fn redo(&mut self) {
        self.invalidate();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn all_layers_visible_strokes(&self) -> Vec<(&RasterLayer, f32)> {
        let mut result = Vec::new();
        for layer in self.layers.iter().rev() {
            if layer.visible {
                result.push((layer.raster(), layer.opacity));
            }
        }
        result
    }

    pub fn is_active_layer_locked(&self) -> bool {
        self.layers[self.active_layer].locked
    }

    // ─── Selection ─────────────────────────────────────────────

    pub fn begin_selection(&mut self, _rect: (f32, f32, f32, f32)) {
        self.selection = Some(Selection {
            rect: (_rect.0, _rect.1, _rect.2, _rect.3),
            bitmap: Some(RasterImage::new(100, 100)),
            strokes: Vec::new(),
        });
        self.invalidate();
    }

    pub fn move_selection(&mut self, dx: f32, dy: f32) {
        if let Some(ref mut sel) = self.selection {
            sel.rect = (sel.rect.0 + dx, sel.rect.1 + dy, sel.rect.2, sel.rect.3);
            self.invalidate();
        }
    }

    pub fn copy_selection(&mut self) {
        // raster-only: no strokes to copy
    }

    pub fn commit_selection(&mut self) {
        self.selection = None;
        self.invalidate();
    }

    pub fn commit_selection_to_raster(&mut self) {
        if let Some(selection) = self.selection.take() {
            if let Some(bitmap) = &selection.bitmap {
                let mut layer = RasterLayer::new(bitmap.width, bitmap.height, 1.0);
                layer.image = bitmap.clone();
                let layer_obj = Layer {
                    name: "Selection".to_string(),
                    content: LayerContent::Raster(layer),
                    visible: true,
                    opacity: 1.0,
                    locked: false,
                };
                self.layers.push(layer_obj);
            }
        }
        self.invalidate();
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.invalidate();
    }

    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }
}

/// Represents a rectangular selection
#[derive(Debug, Clone)]
pub struct Selection {
    pub rect: (f32, f32, f32, f32),
    pub bitmap: Option<RasterImage>,
    pub strokes: Vec<()>,
}