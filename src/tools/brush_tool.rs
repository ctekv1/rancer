//! Brush tool implementation
//!
//! Stamps brush dabs onto RasterImage buffers with alpha compositing.
//! Also supports eraser mode (erase to background color).

use crate::brush::{BrushEngine, BrushType, DabMask, RoundDab};
use crate::canvas::{Canvas, Color};
use crate::tools::{BrushConfig, BrushSettings, Tool};

/// Eraser settings (separate from brush settings)
#[derive(Clone, Copy)]
pub struct EraserSettings {
    pub size: u32,
    pub opacity: f32,  // Eraser strength (0.0 to 1.0)
}

impl Default for EraserSettings {
    fn default() -> Self {
        Self {
            size: 10,
            opacity: 1.0,
        }
    }
}

/// Tool for freehand brush drawing (with optional eraser mode)
pub struct BrushTool {
    is_drawing: bool,
    paint_settings: BrushSettings,
    eraser_settings: EraserSettings,
    is_eraser: bool,  // Mode toggle: false = paint, true = erase
    last_x: f32,
    last_y: f32,
}

impl BrushTool {
    pub fn new() -> Self {
        Self {
            is_drawing: false,
            paint_settings: BrushSettings {
                size: 10,
                opacity: 1.0,
                color: Color { r: 0, g: 0, b: 0, a: 255 },
                brush_type: BrushType::Round,
            },
            eraser_settings: EraserSettings {
                size: 10,
                opacity: 1.0,
            },
            is_eraser: false,
            last_x: 0.0,
            last_y: 0.0,
        }
    }

    pub fn is_drawing(&self) -> bool {
        self.is_drawing
    }
    
    pub fn paint_settings(&self) -> BrushSettings {
        self.paint_settings
    }
    
    pub fn eraser_settings(&self) -> EraserSettings {
        self.eraser_settings
    }
    
    pub fn set_paint_settings(&mut self, settings: BrushSettings) {
        self.paint_settings = settings;
    }
    
    pub fn set_eraser_settings(&mut self, settings: EraserSettings) {
        self.eraser_settings = settings;
    }
    
    pub fn set_brush_color(&mut self, color: Color) {
        self.paint_settings.color = color;
    }

    fn stamp_at(&mut self, x: f32, y: f32, canvas: &mut Canvas) {
        if self.is_eraser {
            self.erase_at(x, y, canvas);
        } else {
            let layer_idx = canvas.active_layer_index();
            let canvas_width = canvas.width();
            let canvas_height = canvas.height();
            let raster = &mut canvas.layers_mut()[layer_idx].content;
            
            let dab = match self.paint_settings.brush_type {
                BrushType::Round => RoundDab::generate(self.paint_settings.size),
                BrushType::Square => DabMask::new(self.paint_settings.size),
            };
            
            let color = Color {
                r: self.paint_settings.color.r,
                g: self.paint_settings.color.g,
                b: self.paint_settings.color.b,
                a: (self.paint_settings.color.a as f32 * self.paint_settings.opacity) as u8,
            };
            
            let half = (self.paint_settings.size as f32 / 2.0).ceil() as i32;
            let min_x = (x as i32 - half).max(0) as u32;
            let min_y = (y as i32 - half).max(0) as u32;
            let max_x = (x as i32 + half).min(canvas_width as i32 - 1) as u32;
            let max_y = (y as i32 + half).min(canvas_height as i32 - 1) as u32;
            
            BrushEngine::stamp_dab(raster, x as i32, y as i32, &dab, color);
            canvas.mark_dirty_rect(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
        }
    }
    
    fn erase_at(&mut self, x: f32, y: f32, canvas: &mut Canvas) {
        let layer_idx = canvas.active_layer_index();
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();
        let bg_color = canvas.background_color;
        let erase_opacity = self.eraser_settings.opacity;
        let raster = &mut canvas.layers_mut()[layer_idx].content;
        
        let dab = match self.paint_settings.brush_type {
            BrushType::Round => RoundDab::generate(self.eraser_settings.size),
            BrushType::Square => DabMask::new(self.eraser_settings.size),
        };
        
        let half = (self.eraser_settings.size as f32 / 2.0).ceil() as i32;
        let min_x = (x as i32 - half).max(0) as u32;
        let min_y = (y as i32 - half).max(0) as u32;
        let max_x = (x as i32 + half).min(canvas_width as i32 - 1) as u32;
        let max_y = (y as i32 + half).min(canvas_height as i32 - 1) as u32;
        
        BrushEngine::erase_dab(raster, x as i32, y as i32, &dab, bg_color, erase_opacity);
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
            let canvas_width = canvas.width() as f32;
            let canvas_height = canvas.height() as f32;
            
            let x = x.clamp(0.0, canvas_width - 1.0);
            let y = y.clamp(0.0, canvas_height - 1.0);
            
            let dx = x - self.last_x;
            let dy = y - self.last_y;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist > 0.0 {
                let step = (self.paint_settings.size as f32 / 2.0).max(1.0);
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
    }

    fn name(&self) -> &str {
        "Brush"
    }

    fn brush_settings(&self) -> Option<BrushSettings> {
        Some(self.paint_settings)
    }

    fn as_brush_config(&mut self) -> Option<&mut dyn BrushConfig> {
        Some(self)
    }
}

impl BrushConfig for BrushTool {
    fn brush_settings(&self) -> BrushSettings {
        self.paint_settings
    }

    fn set_brush_size(&mut self, size: u32) {
        if self.is_eraser {
            self.eraser_settings.size = size.max(1);
        } else {
            self.paint_settings.size = size.max(1);
        }
    }

    fn set_brush_opacity(&mut self, opacity: f32) {
        if self.is_eraser {
            self.eraser_settings.opacity = opacity.clamp(0.0, 1.0);
        } else {
            self.paint_settings.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    fn set_brush_color(&mut self, color: Color) {
        self.paint_settings.color = color;
    }

    fn set_brush_type(&mut self, brush_type: BrushType) {
        self.paint_settings.brush_type = brush_type;
    }

    fn set_eraser_mode(&mut self, enabled: bool) {
        self.is_eraser = enabled;
    }

    fn is_eraser(&self) -> bool {
        self.is_eraser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Canvas, Color};
    use crate::tools::{BrushConfig, BrushSettings};
    
    #[test]
    fn test_brush_tool_new_has_eraser_off() {
        let tool = BrushTool::new();
        assert!(!tool.is_eraser());
    }
    
    #[test]
    fn test_brush_tool_set_eraser_mode() {
        let mut tool = BrushTool::new();
        tool.set_eraser_mode(true);
        assert!(tool.is_eraser());
        
        tool.set_eraser_mode(false);
        assert!(!tool.is_eraser());
    }
    
    #[test]
    fn test_brush_tool_paint_mode_on_press() {
        let mut canvas = Canvas::new();
        canvas.resize(100, 100);
        let mut tool = BrushTool::new();
        tool.set_eraser_mode(false);
        
        tool.on_press(50.0, 50.0, &mut canvas);
        
        let layer_idx = canvas.active_layer();
        let raster = &canvas.layers()[layer_idx].content;
        let pixel = raster.get_pixel(50, 50).unwrap();
        // Pixel should have alpha > 0 after painting (even if RGB is 0 for black)
        assert!(pixel.3 > 0);
    }
    
    #[test]
    fn test_brush_tool_eraser_mode_on_press() {
        let mut canvas = Canvas::new();
        canvas.resize(100, 100);
        
        let mut paint_tool = BrushTool::new();
        paint_tool.set_eraser_mode(false);
        paint_tool.on_press(50.0, 50.0, &mut canvas);
        
        let layer_idx = canvas.active_layer();
        let raster = &canvas.layers()[layer_idx].content;
        let pixel_before = raster.get_pixel(50, 50).unwrap();
        assert!(pixel_before.3 > 0);
        
        let mut eraser_tool = BrushTool::new();
        eraser_tool.set_eraser_mode(true);
        eraser_tool.on_press(50.0, 50.0, &mut canvas);
        
        let raster = &canvas.layers()[layer_idx].content;
        let pixel_after = raster.get_pixel(50, 50).unwrap();
        assert!(pixel_after.3 < pixel_before.3);
    }
    
    #[test]
    fn test_brush_tool_eraser_mode_on_drag() {
        let mut canvas = Canvas::new();
        canvas.resize(100, 100);
        
        let mut paint_tool = BrushTool::new();
        paint_tool.set_eraser_mode(false);
        paint_tool.on_press(50.0, 50.0, &mut canvas);
        paint_tool.on_drag(60.0, 50.0, &mut canvas);
        
        let mut eraser_tool = BrushTool::new();
        eraser_tool.set_eraser_mode(true);
        eraser_tool.on_press(50.0, 50.0, &mut canvas);
        eraser_tool.on_drag(60.0, 50.0, &mut canvas);
        
        let layer_idx = canvas.active_layer();
        let raster = &canvas.layers()[layer_idx].content;
        let px1 = raster.get_pixel(50, 50).unwrap();
        let px2 = raster.get_pixel(55, 50).unwrap();
        assert!(px1.3 < 255);
        assert!(px2.3 < 255);
    }
    
    #[test]
    fn test_eraser_uses_background_color() {
        let mut canvas = Canvas::new();
        canvas.resize(100, 100);
        canvas.background_color = Color { r: 255, g: 0, b: 0, a: 0 };
        
        let mut tool = BrushTool::new();
        tool.set_eraser_mode(false);
        tool.on_press(50.0, 50.0, &mut canvas);
        
        tool.set_eraser_mode(true);
        tool.on_press(50.0, 50.0, &mut canvas);
        
        let layer_idx = canvas.active_layer();
        let raster = &canvas.layers()[layer_idx].content;
        let px = raster.get_pixel(50, 50).unwrap();
        assert_eq!(px.0, canvas.background_color.r);
        assert_eq!(px.3, canvas.background_color.a);
    }
    
    #[test]
    fn test_paint_and_eraser_have_separate_settings() {
        let mut tool = BrushTool::new();
        
        tool.set_paint_settings(BrushSettings {
            size: 20,
            opacity: 1.0,
            color: Color { r: 255, g: 0, b: 0, a: 255 },
            brush_type: crate::brush::BrushType::Round,
        });
        
        tool.set_eraser_settings(EraserSettings {
            size: 40,
            opacity: 0.5,
        });
        
        assert_eq!(tool.paint_settings().size, 20);
        assert_eq!(tool.eraser_settings().size, 40);
        assert_ne!(tool.paint_settings().opacity, tool.eraser_settings().opacity);
    }
}
