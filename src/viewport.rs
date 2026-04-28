//! Viewport transform system
//!
//! Handles screen↔canvas coordinate transformations including zoom and pan.

use crate::canvas::Color;

/// Viewport state for coordinate transformations
#[derive(Debug, Clone)]
pub struct Viewport {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            canvas_width: 1280,
            canvas_height: 720,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            canvas_width,
            canvas_height,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn canvas_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        let screen_x = (x - self.offset_x) * self.zoom;
        let screen_y = (y - self.offset_y) * self.zoom;
        (screen_x, screen_y)
    }

    pub fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let x = screen_x / self.zoom + self.offset_x;
        let y = screen_y / self.zoom + self.offset_y;
        (x, y)
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx / self.zoom;
        self.offset_y += dy / self.zoom;
    }

    pub fn zoom_at(&mut self, factor: f32, center_screen: (f32, f32)) {
        let (cx, cy) = self.screen_to_canvas(center_screen.0, center_screen.1);
        let _old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);
        let new_cx = cx - center_screen.0 / self.zoom;
        let new_cy = cy - center_screen.1 / self.zoom;
        self.offset_x += new_cx - cx;
        self.offset_y += new_cy - cy;
    }

    pub fn get_transform(&self) -> (f32, f32, f32) {
        (self.offset_x, self.offset_y, self.zoom)
    }
}

pub const DEFAULT_CANVAS_COLOR: Color = Color { r: 240, g: 240, b: 240, a: 255 };