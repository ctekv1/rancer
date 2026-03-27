//! Export module for Rancer
//!
//! Provides canvas export functionality to PNG format.
//! Uses software rendering to convert strokes to image pixels.

use image::{ImageBuffer, Rgba};
use std::path::Path;
use crate::canvas::{Canvas, Stroke, Point};
use crate::logger;

/// Export canvas to PNG file
pub fn export_to_png(canvas: &Canvas, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    logger::info(&format!("Exporting canvas to PNG: {:?}", path));
    
    let image = render_canvas_to_image(canvas)?;
    image.save(path)?;
    
    logger::info(&format!("Export successful: {:?}", path));
    Ok(())
}

/// Render canvas to image buffer using software rendering
fn render_canvas_to_image(canvas: &Canvas) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let (width, height) = canvas.size();
    let mut image = ImageBuffer::new(width, height);
    
    // Fill background
    let bg_color = canvas.background_color();
    for pixel in image.pixels_mut() {
        *pixel = Rgba([bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
    }
    
    // Render each stroke
    for stroke in canvas.strokes() {
        render_stroke_to_image(&mut image, stroke)?;
    }
    
    Ok(image)
}

/// Render a single stroke to image buffer
fn render_stroke_to_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    stroke: &Stroke
) -> Result<(), Box<dyn std::error::Error>> {
    if stroke.points.len() < 2 {
        return Ok(());
    }
    
    let color = Rgba([
        stroke.color.r,
        stroke.color.g,
        stroke.color.b,
        (stroke.opacity * 255.0) as u8,
    ]);
    
    let half_width = stroke.width / 2.0;
    
    // Render each segment of the stroke
    for i in 0..stroke.points.len() - 1 {
        let p1 = &stroke.points[i];
        let p2 = &stroke.points[i + 1];
        
        // Draw line segment with thickness
        render_line_segment(image, p1, p2, color, half_width);
    }
    
    Ok(())
}

/// Render a line segment with thickness to image buffer
fn render_line_segment(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    p1: &Point,
    p2: &Point,
    color: Rgba<u8>,
    half_width: f32,
) {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let length = (dx * dx + dy * dy).sqrt();
    
    if length < 0.001 {
        return;
    }
    
    // Calculate perpendicular vector
    let nx = -dy / length * half_width;
    let ny = dx / length * half_width;
    
    // Generate quad vertices (two triangles)
    let vertices = vec![
        [p1.x + nx, p1.y + ny],
        [p1.x - nx, p1.y - ny],
        [p2.x + nx, p2.y + ny],
        [p2.x - nx, p2.y - ny],
    ];
    
    // Render quad as filled triangles
    render_triangle(image, &vertices[0], &vertices[1], &vertices[2], color);
    render_triangle(image, &vertices[1], &vertices[2], &vertices[3], color);
}

/// Render a filled triangle to image buffer
fn render_triangle(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    v0: &[f32; 2],
    v1: &[f32; 2],
    v2: &[f32; 2],
    color: Rgba<u8>,
) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    
    // Find bounding box
    let min_x = v0[0].min(v1[0]).min(v2[0]).max(0.0).floor() as u32;
    let max_x = v0[0].max(v1[0]).max(v2[0]).min(width - 1.0).ceil() as u32;
    let min_y = v0[1].min(v1[1]).min(v2[1]).max(0.0).floor() as u32;
    let max_y = v0[1].max(v1[1]).max(v2[1]).min(height - 1.0).ceil() as u32;
    
    // Render pixels in bounding box
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            
            // Check if point is inside triangle using barycentric coordinates
            if point_in_triangle(px, py, v0, v1, v2) {
                image.put_pixel(x, y, color);
            }
        }
    }
}

/// Check if a point is inside a triangle using barycentric coordinates
fn point_in_triangle(
    px: f32,
    py: f32,
    v0: &[f32; 2],
    v1: &[f32; 2],
    v2: &[f32; 2],
) -> bool {
    let denominator = (v1[1] - v2[1]) * (v0[0] - v2[0]) + (v2[0] - v1[0]) * (v0[1] - v2[1]);
    
    if denominator.abs() < 0.0001 {
        return false;
    }
    
    let w0 = ((v1[1] - v2[1]) * (px - v2[0]) + (v2[0] - v1[0]) * (py - v2[1])) / denominator;
    let w1 = ((v2[1] - v0[1]) * (px - v2[0]) + (v0[0] - v2[0]) * (py - v2[1])) / denominator;
    let w2 = 1.0 - w0 - w1;
    
    // Check if point is inside triangle (with small epsilon for edge cases)
    let epsilon = 0.001;
    w0 >= -epsilon && w1 >= -epsilon && w2 >= -epsilon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;

    #[test]
    fn test_export_creates_valid_png() {
        let canvas = Canvas::new();
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export.png");
        
        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export should succeed");
        assert!(path.exists(), "PNG file should be created");
        
        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_point_in_triangle() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];
        
        // Point inside triangle
        assert!(point_in_triangle(5.0, 5.0, &v0, &v1, &v2));
        
        // Point outside triangle
        assert!(!point_in_triangle(0.0, 10.0, &v0, &v1, &v2));
    }
}
