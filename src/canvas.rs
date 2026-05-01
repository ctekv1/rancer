//! Canvas module for Rancer
//!
//! Provides the core canvas functionality using raster layers.

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

/// Pixel reference for mutable access
pub struct PixelRef<'a> {
    data: &'a mut [u8],
}

impl<'a> PixelRef<'a> {
    pub fn r(&self) -> u8 { self.data[0] }
    pub fn g(&self) -> u8 { self.data[1] }
    pub fn b(&self) -> u8 { self.data[2] }
    pub fn a(&self) -> u8 { self.data[3] }

    pub fn set_r(&mut self, r: u8) { self.data[0] = r; }
    pub fn set_g(&mut self, g: u8) { self.data[1] = g; }
    pub fn set_b(&mut self, b: u8) { self.data[2] = b; }
    pub fn set_a(&mut self, a: u8) { self.data[3] = a; }
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

    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<PixelRef<'_>> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some(PixelRef { data: &mut self.data[idx..idx + 4] })
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

#[cfg(test)]
mod pixel_ref_tests {
    use super::*;

    #[test]
    fn test_pixel_ref_reads_correct_values() {
        let mut image = RasterImage::new(2, 2);
        image.set_pixel(0, 0, 10, 20, 30, 40);
        
        if let Some(pixel) = image.get_pixel_mut(0, 0) {
            assert_eq!(pixel.r(), 10);
            assert_eq!(pixel.g(), 20);
            assert_eq!(pixel.b(), 30);
            assert_eq!(pixel.a(), 40);
        } else {
            panic!("Expected pixel");
        }
    }

    #[test]
    fn test_pixel_ref_writes_correct_values() {
        let mut image = RasterImage::new(2, 2);
        
        if let Some(mut pixel) = image.get_pixel_mut(0, 0) {
            pixel.set_r(100);
            pixel.set_g(150);
            pixel.set_b(200);
            pixel.set_a(250);
        }
        
        assert_eq!(image.get_pixel(0, 0), Some((100, 150, 200, 250)));
    }

    #[test]
    fn test_pixel_ref_returns_none_for_out_of_bounds() {
        let mut image = RasterImage::new(2, 2);
        assert!(image.get_pixel_mut(5, 0).is_none());
        assert!(image.get_pixel_mut(0, 5).is_none());
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
    pub content: RasterLayer,
    pub visible: bool,
    pub opacity: f32,
    pub locked: bool,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: "Layer 1".to_string(),
            content: RasterLayer::default(),
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
            content: RasterLayer::new(width, height, 1.0),
            visible: true,
            opacity,
            locked: false,
        }
    }

    pub fn raster_mut(&mut self) -> &mut RasterLayer {
        &mut self.content
    }

    pub fn raster(&self) -> &RasterLayer {
        &self.content
    }

    pub fn is_raster(&self) -> bool {
        true
    }

    pub fn clear(&mut self) {
        self.content.image.fill(Color::TRANSPARENT);
    }
}

/// The main canvas for drawing operations
#[derive(Clone)]
pub struct Canvas {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) background_color: Color,
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: usize,
    pub(crate) version: u64,
    dirty_rect: DirtyRect,
}

/// Rectangle representing a dirty region
#[derive(Debug, Clone, Copy)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    is_dirty: bool,
}

impl DirtyRect {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            is_dirty: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.is_dirty
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        self.is_dirty && x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn mark_pixel(&mut self, x: u32, y: u32) {
        if !self.is_dirty {
            self.x = x;
            self.y = y;
            self.width = 1;
            self.height = 1;
            self.is_dirty = true;
        } else {
            let new_min_x = self.x.min(x);
            let new_min_y = self.y.min(y);
            let new_max_x = (self.x + self.width).max(x + 1);
            let new_max_y = (self.y + self.height).max(y + 1);
            self.x = new_min_x;
            self.y = new_min_y;
            self.width = new_max_x - new_min_x;
            self.height = new_max_y - new_min_y;
        }
    }

    pub fn mark_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        if !self.is_dirty {
            self.x = x;
            self.y = y;
            self.width = w;
            self.height = h;
            self.is_dirty = true;
        } else {
            let new_min_x = self.x.min(x);
            let new_min_y = self.y.min(y);
            let new_max_x = (self.x + self.width).max(x + w);
            let new_max_y = (self.y + self.height).max(y + h);
            self.x = new_min_x;
            self.y = new_min_y;
            self.width = new_max_x - new_min_x;
            self.height = new_max_y - new_min_y;
        }
    }

    pub fn clear(&mut self) {
        self.is_dirty = false;
        self.x = 0;
        self.y = 0;
        self.width = 0;
        self.height = 0;
    }

    pub fn mark_full(&mut self, width: u32, height: u32) {
        self.x = 0;
        self.y = 0;
        self.width = width;
        self.height = height;
        self.is_dirty = true;
    }
}

impl Default for DirtyRect {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of compositing all visible layers
#[derive(Debug, Clone)]
pub struct CompositeResult {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            background_color: Color::WHITE,
            layers: vec![Layer::new("Background".to_string(), 1280, 720, 1.0)],
            active_layer: 0,
            version: 0,
            dirty_rect: DirtyRect::new(),
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

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn active_layer_index(&self) -> usize {
        self.active_layer
    }

    pub fn invalidate(&mut self) {
        self.version += 1;
        self.dirty_rect.mark_full(self.width, self.height);
    }

    pub fn mark_dirty(&mut self, x: u32, y: u32) {
        self.version += 1;
        self.dirty_rect.mark_pixel(x, y);
    }

    pub fn mark_dirty_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        self.version += 1;
        self.dirty_rect.mark_rect(x, y, w, h);
    }

    pub fn dirty_rect(&self) -> &DirtyRect {
        &self.dirty_rect
    }

    pub fn consume_dirty_rect(&mut self) -> DirtyRect {
        let dirty = self.dirty_rect;
        self.dirty_rect.clear();
        dirty
    }

    pub fn composite_all(&self) -> CompositeResult {
        let pixel_count = (self.width * self.height) as usize;
        let mut data = vec![0u8; pixel_count * 4];
        
        // Fill with background color first
        for i in 0..pixel_count {
            data[i * 4] = self.background_color.r;
            data[i * 4 + 1] = self.background_color.g;
            data[i * 4 + 2] = self.background_color.b;
            data[i * 4 + 3] = 255;
        }
        
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            
            let opacity = layer.opacity;
            let raster = &layer.content;
            let layer_data = &raster.image.data;
            
            for i in 0..pixel_count {
                let src_r = layer_data[i * 4] as f32 / 255.0;
                let src_g = layer_data[i * 4 + 1] as f32 / 255.0;
                let src_b = layer_data[i * 4 + 2] as f32 / 255.0;
                let src_a = (layer_data[i * 4 + 3] as f32 / 255.0) * opacity;
                
                if src_a <= 0.0 {
                    continue;
                }
                
                let dst_r = data[i * 4] as f32 / 255.0;
                let dst_g = data[i * 4 + 1] as f32 / 255.0;
                let dst_b = data[i * 4 + 2] as f32 / 255.0;
                let dst_a = data[i * 4 + 3] as f32 / 255.0;
                
                let out_a = src_a + dst_a * (1.0 - src_a);
                let inv_dst_a = 1.0 - src_a;
                
                let out_r = (src_r * src_a + dst_r * dst_a * inv_dst_a) / out_a;
                let out_g = (src_g * src_a + dst_g * dst_a * inv_dst_a) / out_a;
                let out_b = (src_b * src_a + dst_b * dst_a * inv_dst_a) / out_a;
                
                data[i * 4] = (out_r * 255.0).clamp(0.0, 255.0) as u8;
                data[i * 4 + 1] = (out_g * 255.0).clamp(0.0, 255.0) as u8;
                data[i * 4 + 2] = (out_b * 255.0).clamp(0.0, 255.0) as u8;
                data[i * 4 + 3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
            }
        }
        
        CompositeResult {
            width: self.width,
            height: self.height,
            data,
        }
    }

    pub fn composite_rect(&self, x: u32, y: u32, w: u32, h: u32) -> CompositeResult {
        if w == 0 || h == 0 {
            return CompositeResult {
                width: 0,
                height: 0,
                data: Vec::new(),
            };
        }
        
        // Clamp to canvas bounds
        let x = x.min(self.width);
        let y = y.min(self.height);
        let w = w.min(self.width - x);
        let h = h.min(self.height - y);
        
        let pixel_count = (w * h) as usize;
        let mut data = vec![0u8; pixel_count * 4];
        
        // Fill with background color
        for i in 0..pixel_count {
            data[i * 4] = self.background_color.r;
            data[i * 4 + 1] = self.background_color.g;
            data[i * 4 + 2] = self.background_color.b;
            data[i * 4 + 3] = 255;
        }
        
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            
            let opacity = layer.opacity;
            let raster = &layer.content;
            let layer_data = &raster.image.data;
            let layer_width = raster.image.width;
            
            for cy in 0..h {
                for cx in 0..w {
                    let canvas_x = x + cx;
                    let canvas_y = y + cy;
                    
                    let out_idx = ((cy * w + cx) * 4) as usize;
                    
                    let layer_idx = ((canvas_y * layer_width + canvas_x) * 4) as usize;
                    
                    let src_r = layer_data[layer_idx] as f32 / 255.0;
                    let src_g = layer_data[layer_idx + 1] as f32 / 255.0;
                    let src_b = layer_data[layer_idx + 2] as f32 / 255.0;
                    let src_a = (layer_data[layer_idx + 3] as f32 / 255.0) * opacity;
                    
                    if src_a <= 0.0 {
                        continue;
                    }
                    
                    let dst_r = data[out_idx] as f32 / 255.0;
                    let dst_g = data[out_idx + 1] as f32 / 255.0;
                    let dst_b = data[out_idx + 2] as f32 / 255.0;
                    let dst_a = data[out_idx + 3] as f32 / 255.0;
                    
                    let out_a = src_a + dst_a * (1.0 - src_a);
                    let inv_dst_a = 1.0 - src_a;
                    
                    let out_r = (src_r * src_a + dst_r * dst_a * inv_dst_a) / out_a;
                    let out_g = (src_g * src_a + dst_g * dst_a * inv_dst_a) / out_a;
                    let out_b = (src_b * src_a + dst_b * dst_a * inv_dst_a) / out_a;
                    
                    data[out_idx] = (out_r * 255.0).clamp(0.0, 255.0) as u8;
                    data[out_idx + 1] = (out_g * 255.0).clamp(0.0, 255.0) as u8;
                    data[out_idx + 2] = (out_b * 255.0).clamp(0.0, 255.0) as u8;
                    data[out_idx + 3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
                }
            }
        }
        
        CompositeResult {
            width: w,
            height: h,
            data,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        for layer in &mut self.layers {
            layer.content.image.resize(width, height);
        }
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
        self.invalidate();
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
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
}
