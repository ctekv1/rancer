//! Brush tool implementation

use crate::brush::{BrushEngine, BrushType, RoundDab, SquareDab};
use crate::canvas::{Canvas, Color};
use crate::tools::{BrushSettings, Tool};

/// Tool for freehand brush drawing
pub struct BrushTool {
    is_drawing: bool,
    brush_settings: BrushSettings,
    last_x: f32,
    last_y: f32,
}

impl BrushTool {
    pub fn new() -> Self {
        Self {
            is_drawing: false,
            brush_settings: BrushSettings {
                size: 10,
                opacity: 1.0,
                color: Color { r: 0, g: 0, b: 0, a: 255 },
                brush_type: BrushType::Round,
            },
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    pub fn is_drawing(&self) -> bool {
        self.is_drawing
    }

    fn stamp_at(&mut self, x: f32, y: f32, canvas: &mut Canvas) {
        let layer_idx = canvas.active_layer_index();
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();
        let raster = &mut canvas.layers_mut()[layer_idx].content;
        
        let dab = match self.brush_settings.brush_type {
            BrushType::Round => RoundDab::generate(self.brush_settings.size),
            BrushType::Square => SquareDab::generate(self.brush_settings.size),
        };
        
        let color = Color {
            r: self.brush_settings.color.r,
            g: self.brush_settings.color.g,
            b: self.brush_settings.color.b,
            a: (self.brush_settings.opacity * 255.0) as u8,
        };

        let half = (self.brush_settings.size as f32 / 2.0).ceil() as i32;
        let min_x = (x as i32 - half).max(0) as u32;
        let min_y = (y as i32 - half).max(0) as u32;
        let max_x = (x as i32 + half).min(canvas_width as i32 - 1) as u32;
        let max_y = (y as i32 + half).min(canvas_height as i32 - 1) as u32;

        BrushEngine::stamp_dab(&mut raster.image, x as i32, y as i32, &dab, color);
        canvas.mark_dirty_rect(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
    }
}

impl Default for BrushTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for BrushTool {
    fn on_press(&mut self, x: f32, y: f32, canvas: &mut Canvas) {
        self.is_drawing = true;
        self.last_x = x;
        self.last_y = y;
        self.stamp_at(x, y, canvas);
    }

    fn on_drag(&mut self, x: f32, y: f32, canvas: &mut Canvas) {
        if self.is_drawing {
            // Professional apps: ignore drags that start outside canvas
            // or clamp coordinates to valid range
            let canvas_width = canvas.width() as f32;
            let canvas_height = canvas.height() as f32;
            
            // Clamp to canvas bounds
            let x = x.clamp(0.0, canvas_width - 1.0);
            let y = y.clamp(0.0, canvas_height - 1.0);
            
            let dx = x - self.last_x;
            let dy = y - self.last_y;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist > 0.0 {
                let step = (self.brush_settings.size as f32 / 2.0).max(1.0);
                let steps = (dist / step).ceil() as i32;
                
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let ix = self.last_x + dx * t;
                    let iy = self.last_y + dy * t;
                    self.stamp_at(ix, iy, canvas);
                }
            }
            
            self.last_x = x;
            self.last_y = y;
        }
    }

    fn on_release(&mut self, _x: f32, _y: f32, _canvas: &mut Canvas) {
        self.is_drawing = false;
    }

    fn on_key(&mut self, _code: &str) {
        // Brush tool doesn't handle keys directly
    }

    fn name(&self) -> &str {
        "Brush"
    }

    fn brush_settings(&self) -> BrushSettings {
        self.brush_settings
    }

    fn set_brush_size(&mut self, size: u32) {
        self.brush_settings.size = size.max(1);
    }

    fn set_brush_opacity(&mut self, opacity: f32) {
        self.brush_settings.opacity = opacity.clamp(0.0, 1.0);
    }

    fn set_brush_color(&mut self, color: Color) {
        self.brush_settings.color = color;
    }

    fn set_brush_type(&mut self, brush_type: BrushType) {
        self.brush_settings.brush_type = brush_type;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
