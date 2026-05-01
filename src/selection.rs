//! Pixel-region selection
//!
//! Implements copy-buffer → move → merge selection workflow.

use crate::canvas::{Canvas, RasterImage};

/// Rectangle representing selection bounds
#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Pixel selection state
pub struct PixelSelection {
    pub rect: SelectionRect,
    pub float_buffer: Option<RasterImage>,
    pub original_pixels: Option<RasterImage>,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl PixelSelection {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            rect: SelectionRect { x, y, width, height },
            float_buffer: None,
            original_pixels: None,
            offset_x: 0,
            offset_y: 0,
        }
    }

    /// Begin selection by cutting pixels from the active layer
    pub fn begin_selection(&mut self, canvas: &mut Canvas) {
        let layer_idx = canvas.active_layer_index();
        let width = canvas.width();
        let height = canvas.height();
        
        // Extract pixels from the selection region
        let mut extracted = RasterImage::new(self.rect.width, self.rect.height);
        let mut original = RasterImage::new(self.rect.width, self.rect.height);

        for sy in 0..self.rect.height {
            for sx in 0..self.rect.width {
                let canvas_x = (self.rect.x + sx as i32) as u32;
                let canvas_y = (self.rect.y + sy as i32) as u32;

                if canvas_x < width && canvas_y < height {
                    let raster = &canvas.layers()[layer_idx].content;
                    if let Some((r, g, b, a)) = raster.image.get_pixel(canvas_x, canvas_y) {
                        extracted.set_pixel(sx as u32, sy as u32, r, g, b, a);
                        original.set_pixel(sx as u32, sy as u32, r, g, b, a);
                    }
                }
            }
        }

        self.float_buffer = Some(extracted);
        self.original_pixels = Some(original);

        // Clear the pixels from the original layer
        for sy in 0..self.rect.height {
            for sx in 0..self.rect.width {
                let canvas_x = (self.rect.x + sx as i32) as u32;
                let canvas_y = (self.rect.y + sy as i32) as u32;
                if canvas_x < width && canvas_y < height {
                    let raster = &mut canvas.layers_mut()[layer_idx].content;
                    raster.image.set_pixel(canvas_x, canvas_y, 0, 0, 0, 0);
                }
            }
        }
    }

    /// Move selection by delta offset
    pub fn move_selection(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx as i32;
        self.offset_y += dy as i32;
    }

    /// Commit selection by merging float buffer back to canvas
    pub fn commit_selection(&mut self, canvas: &mut Canvas) {
        if let Some(ref float_buffer) = self.float_buffer {
            let layer_idx = canvas.active_layer_index();
            let raster = &mut canvas.layers_mut()[layer_idx].content;
            // Merge float buffer at new position (with offset)
            for sy in 0..float_buffer.height {
                for sx in 0..float_buffer.width {
                    let canvas_x = (self.rect.x as i32 + self.offset_x + sx as i32) as u32;
                    let canvas_y = (self.rect.y as i32 + self.offset_y + sy as i32) as u32;

                    if let Some((r, g, b, a)) = float_buffer.get_pixel(sx, sy) {
                        if a > 0 {
                            raster.image.set_pixel(canvas_x, canvas_y, r, g, b, a);
                        }
                    }
                }
            }
        }

        self.float_buffer = None;
        self.original_pixels = None;
        self.offset_x = 0;
        self.offset_y = 0;
    }

    /// Cancel selection by restoring original pixels
    pub fn cancel_selection(&mut self, canvas: &mut Canvas) {
        if let Some(ref original) = self.original_pixels {
            let layer_idx = canvas.active_layer_index();
            let raster = &mut canvas.layers_mut()[layer_idx].content;
            // Restore original pixels
            for sy in 0..original.height {
                for sx in 0..original.width {
                    let canvas_x = (self.rect.x + sx as i32) as u32;
                    let canvas_y = (self.rect.y + sy as i32) as u32;

                    if let Some((r, g, b, a)) = original.get_pixel(sx, sy) {
                        raster.image.set_pixel(canvas_x, canvas_y, r, g, b, a);
                    }
                }
            }
        }

        self.float_buffer = None;
        self.original_pixels = None;
        self.offset_x = 0;
        self.offset_y = 0;
    }
}
