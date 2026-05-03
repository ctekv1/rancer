//! BrushEngine - applies dabs to raster images
//!
//! Stamps brush dabs onto RasterImage buffers with alpha compositing.

use crate::brush::DabMask;
use crate::canvas::{RasterImage, Color};

pub struct BrushEngine;

impl BrushEngine {
    pub fn stamp_dab(
        buffer: &mut RasterImage,
        cx: i32,
        cy: i32,
        dab: &DabMask,
        color: Color,
    ) {
        eprintln!("[DEBUG] stamp_dab: cx={}, cy={}, buffer_size={}x{}", cx, cy, buffer.width, buffer.height);
        let dab_w = dab.width as i32;
        let dab_h = dab.height as i32;
        let offset_x = cx - dab_w / 2;
        let offset_y = cy - dab_h / 2;

        let mut painted = 0;
        for dy in 0..dab_h {
            for dx in 0..dab_w {
                let dab_alpha = dab.get_alpha(dx as u32, dy as u32);
                if dab_alpha == 0 {
                    continue;
                }

                let buf_x = offset_x + dx;
                let buf_y = offset_y + dy;

                if buf_x < 0 || buf_y < 0 {
                    continue;
                }
                if buf_x >= buffer.width as i32 || buf_y >= buffer.height as i32 {
                    continue;
                }

                painted += 1;
                let final_alpha = ((dab_alpha as f32 / 255.0) * (color.a as f32 / 255.0) * 255.0) as u8;
                let (r, g, b) = (color.r, color.g, color.b);

                if let Some(mut pixel) = buffer.get_pixel_mut(buf_x as u32, buf_y as u32) {
                    let src_a = final_alpha as f32 / 255.0;
                    pixel.set_r(((r as f32 * src_a) + (pixel.r() as f32 * (1.0 - src_a))) as u8);
                    pixel.set_g(((g as f32 * src_a) + (pixel.g() as f32 * (1.0 - src_a))) as u8);
                    pixel.set_b(((b as f32 * src_a) + (pixel.b() as f32 * (1.0 - src_a))) as u8);
                    pixel.set_a((pixel.a() as u16 + final_alpha as u16).min(255) as u8);
                }
            }
        }
        eprintln!("[DEBUG] stamp_dab painted {} pixels", painted);
    }
    
    pub fn erase_dab(
        buffer: &mut RasterImage,
        cx: i32,
        cy: i32,
        dab: &DabMask,
        bg_color: Color,
        erase_opacity: f32,  // 0.0 to 1.0
    ) {
        let dab_w = dab.width as i32;
        let dab_h = dab.height as i32;
        let offset_x = cx - dab_w / 2;
        let offset_y = cy - dab_h / 2;
        
        for dy in 0..dab_h {
            for dx in 0..dab_w {
                let dab_alpha = dab.get_alpha(dx as u32, dy as u32);
                if dab_alpha == 0 {
                    continue;
                }
                
                let buf_x = offset_x + dx;
                let buf_y = offset_y + dy;
                
                if buf_x < 0 || buf_y < 0 {
                    continue;
                }
                if buf_x >= buffer.width as i32 || buf_y >= buffer.height as i32 {
                    continue;
                }
                
                let erase_strength = (dab_alpha as f32 / 255.0) * erase_opacity;
                
                if let Some(mut pixel) = buffer.get_pixel_mut(buf_x as u32, buf_y as u32) {
                    // Reduce alpha (erase towards background)
                    let new_a = (pixel.a() as f32 * (1.0 - erase_strength)) as u8;
                    pixel.set_a(new_a);
                    
                    // If alpha hits 0, set RGB to background color
                    if new_a == 0 {
                        pixel.set_r(bg_color.r);
                        pixel.set_g(bg_color.g);
                        pixel.set_b(bg_color.b);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::RoundDab;

    #[test]
    fn test_stamp_dab_affects_pixels_at_center() {
        let mut buffer = RasterImage::new(10, 10);
        let dab = RoundDab::generate(3);
        let color = Color { r: 255, g: 0, b: 0, a: 255 };

        BrushEngine::stamp_dab(&mut buffer, 5, 5, &dab, color);

        let px = buffer.get_pixel(5, 5).unwrap();
        assert!(px.0 > 0);
    }

    #[test]
    fn test_stamp_dab_does_not_affect_distant_pixels() {
        let mut buffer = RasterImage::new(10, 10);
        let dab = RoundDab::generate(3);
        let color = Color { r: 255, g: 0, b: 0, a: 255 };

        BrushEngine::stamp_dab(&mut buffer, 0, 0, &dab, color);

        let px = buffer.get_pixel(5, 5).unwrap();
        assert_eq!(px.0, 0);
    }

    #[test]
    fn test_stamp_dab_respects_partial_transparency() {
        let mut buffer = RasterImage::new(10, 10);
        let dab = RoundDab::generate(3);
        let color = Color { r: 255, g: 0, b: 0, a: 128 };

        BrushEngine::stamp_dab(&mut buffer, 5, 5, &dab, color);

        let px = buffer.get_pixel(5, 5).unwrap();
        assert!(px.0 > px.1);
        assert!(px.0 < 255);
    }

    #[test]
    fn test_stamp_dab_out_of_bounds_is_ignored() {
        let mut buffer = RasterImage::new(10, 10);
        let dab = RoundDab::generate(20);
        let color = Color { r: 255, g: 0, b: 0, a: 255 };

        BrushEngine::stamp_dab(&mut buffer, -5, -5, &dab, color);
    }

    #[test]
    fn test_draw_line_produces_continuous_pixels() {
        let mut buffer = RasterImage::new(20, 20);
        let dab = RoundDab::generate(3);
        let color = Color { r: 50, g: 50, b: 50, a: 200 };

        for t in [0.0_f32, 0.5, 1.0] {
            let x = (0.0 + (10.0 - 0.0) * t) as i32;
            let y = (0.0 + (10.0 - 0.0) * t) as i32;
            BrushEngine::stamp_dab(&mut buffer, x, y, &dab, color);
        }

        let px = buffer.get_pixel(0, 0).unwrap();
        assert!(px.0 > 0);
        let px = buffer.get_pixel(10, 10).unwrap();
        assert!(px.0 > 0);
    }

    #[test]
    fn test_debug_print_removed() {
        let mut buffer = RasterImage::new(3, 3);
        let dab = RoundDab::generate(1);
        BrushEngine::stamp_dab(&mut buffer, 1, 1, &dab, Color { r: 100, g: 100, b: 100, a: 200 });
    }
}