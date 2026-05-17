use crate::canvas::Canvas;

/// Result of compositing all visible layers
#[derive(Debug, Clone)]
pub struct CompositeResult {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

fn blend_pixel(dst: &mut [u8], src_r: f32, src_g: f32, src_b: f32, src_a: f32) {
    let dst_r = dst[0] as f32 / 255.0;
    let dst_g = dst[1] as f32 / 255.0;
    let dst_b = dst[2] as f32 / 255.0;
    let dst_a = dst[3] as f32 / 255.0;

    let out_a = src_a + dst_a * (1.0 - src_a);
    let inv_dst_a = 1.0 - src_a;

    let out_r = (src_r * src_a + dst_r * dst_a * inv_dst_a) / out_a;
    let out_g = (src_g * src_a + dst_g * dst_a * inv_dst_a) / out_a;
    let out_b = (src_b * src_a + dst_b * dst_a * inv_dst_a) / out_a;

    dst[0] = (out_r * 255.0).clamp(0.0, 255.0) as u8;
    dst[1] = (out_g * 255.0).clamp(0.0, 255.0) as u8;
    dst[2] = (out_b * 255.0).clamp(0.0, 255.0) as u8;
    dst[3] = (out_a * 255.0).clamp(0.0, 255.0) as u8;
}

pub struct Compositor {
    last_composited_version: u64,
    has_composited: bool,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            last_composited_version: 0,
            has_composited: false,
        }
    }

    pub fn composite_all(&self, canvas: &Canvas) -> CompositeResult {
        let pixel_count = (canvas.width * canvas.height) as usize;
        let mut data = vec![0u8; pixel_count * 4];

        for i in 0..pixel_count {
            data[i * 4] = canvas.background_color.r;
            data[i * 4 + 1] = canvas.background_color.g;
            data[i * 4 + 2] = canvas.background_color.b;
            data[i * 4 + 3] = 255;
        }

        for layer in &canvas.layers {
            if !layer.visible {
                continue;
            }

            let opacity = layer.opacity;
            let layer_data = &layer.content.data;

            for i in 0..pixel_count {
                let src_r = layer_data[i * 4] as f32 / 255.0;
                let src_g = layer_data[i * 4 + 1] as f32 / 255.0;
                let src_b = layer_data[i * 4 + 2] as f32 / 255.0;
                let src_a = (layer_data[i * 4 + 3] as f32 / 255.0) * opacity;

                if src_a <= 0.0 {
                    continue;
                }

                blend_pixel(&mut data[i * 4..i * 4 + 4], src_r, src_g, src_b, src_a);
            }
        }

        CompositeResult {
            width: canvas.width,
            height: canvas.height,
            data,
        }
    }

    pub fn composite_rect(&self, canvas: &Canvas, x: u32, y: u32, w: u32, h: u32) -> CompositeResult {
        if w == 0 || h == 0 {
            return CompositeResult {
                width: 0,
                height: 0,
                data: Vec::new(),
            };
        }

        let x = x.min(canvas.width);
        let y = y.min(canvas.height);
        let w = w.min(canvas.width - x);
        let h = h.min(canvas.height - y);

        let pixel_count = (w * h) as usize;
        let mut data = vec![0u8; pixel_count * 4];

        for i in 0..pixel_count {
            data[i * 4] = canvas.background_color.r;
            data[i * 4 + 1] = canvas.background_color.g;
            data[i * 4 + 2] = canvas.background_color.b;
            data[i * 4 + 3] = 255;
        }

        for layer in &canvas.layers {
            if !layer.visible {
                continue;
            }

            let opacity = layer.opacity;
            let layer_data = &layer.content.data;
            let layer_width = layer.content.width;

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

                    blend_pixel(&mut data[out_idx..out_idx + 4], src_r, src_g, src_b, src_a);
                }
            }
        }

        CompositeResult {
            width: w,
            height: h,
            data,
        }
    }

    /// Returns (CompositeResult, x_offset, y_offset) where x/y is the position
    /// of the composited region within the canvas.
    /// Full-canvas composites have (0, 0); partial composites have the dirty rect origin.
    pub fn render(&mut self, canvas: &mut Canvas) -> Option<(CompositeResult, u32, u32)> {
        let current_version = canvas.version();

        if !self.has_composited {
            self.has_composited = true;
            self.last_composited_version = current_version;
            let result = self.composite_all(canvas);
            canvas.consume_dirty_rect();
            return Some((result, 0, 0));
        }

        if current_version == self.last_composited_version {
            return None;
        }

        self.last_composited_version = current_version;
        let dirty = *canvas.dirty_rect();

        let (result, x, y) = if !dirty.is_empty()
            && (dirty.width as u64 * dirty.height as u64)
                < (canvas.width as u64 * canvas.height as u64 / 2)
        {
            (self.composite_rect(canvas, dirty.x, dirty.y, dirty.width, dirty.height), dirty.x, dirty.y)
        } else {
            (self.composite_all(canvas), 0, 0)
        };

        canvas.consume_dirty_rect();
        Some((result, x, y))
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}
