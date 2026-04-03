//! Export module for Rancer
//!
//! Provides canvas export functionality to PNG format.
//! Uses software rendering to convert strokes to image pixels.

use crate::canvas::{Canvas, Stroke};
use crate::geometry;
use crate::logger;
use image::{ImageBuffer, Rgba};
use std::path::Path;

/// Export canvas to PNG file
pub fn export_to_png(canvas: &Canvas, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    logger::info(&format!("Exporting canvas to PNG: {:?}", path));

    let image = render_canvas_to_image(canvas)?;
    image.save(path)?;

    logger::info(&format!("Export successful: {:?}", path));
    Ok(())
}

/// Render canvas to image buffer using software rendering
fn render_canvas_to_image(
    canvas: &Canvas,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let (width, height) = canvas.size();
    let mut image = ImageBuffer::new(width, height);

    // Fill background
    let bg_color = canvas.background_color();
    for pixel in image.pixels_mut() {
        *pixel = Rgba([bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
    }

    // Render each stroke from all visible layers
    for (stroke, layer_opacity) in canvas.all_strokes() {
        // Apply layer opacity to stroke
        let adjusted_stroke = Stroke {
            opacity: stroke.opacity * layer_opacity,
            ..stroke.clone()
        };
        render_stroke_to_image(&mut image, &adjusted_stroke)?;
    }

    Ok(image)
}

/// Render a single stroke to image buffer
fn render_stroke_to_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    stroke: &Stroke,
) -> Result<(), Box<dyn std::error::Error>> {
    let vertices = geometry::generate_stroke_vertices(stroke);

    if vertices.len() < 18 {
        // Need at least 3 vertices * 6 floats = 18 floats for one triangle
        return Ok(());
    }

    // Each vertex is [x, y, r, g, b, a] = 6 floats
    // Walk the triangle strip: triangles are (0,1,2), (1,2,3), (2,3,4), ...
    let color = Rgba([
        stroke.color.r,
        stroke.color.g,
        stroke.color.b,
        (stroke.opacity * 255.0) as u8,
    ]);

    for i in (0..vertices.len() - 12).step_by(12) {
        let v0 = [vertices[i], vertices[i + 1]];
        let v1 = [vertices[i + 6], vertices[i + 7]];
        let v2 = [vertices[i + 12], vertices[i + 13]];

        render_triangle(image, &v0, &v1, &v2, color);

        if i + 18 <= vertices.len() {
            let v3 = [vertices[i + 18], vertices[i + 19]];
            render_triangle(image, &v1, &v2, &v3, color);
        }
    }

    Ok(())
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
            if geometry::point_in_triangle(px, py, v0, v1, v2) {
                image.put_pixel(x, y, color);
            }
        }
    }
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
    fn test_point_in_triangle_via_geometry() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];

        // Point inside triangle
        assert!(geometry::point_in_triangle(5.0, 5.0, &v0, &v1, &v2));

        // Point outside triangle
        assert!(!geometry::point_in_triangle(0.0, 10.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_export_with_strokes() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0);
        s1.add_point(crate::canvas::Point { x: 10.0, y: 10.0 });
        s1.add_point(crate::canvas::Point { x: 100.0, y: 100.0 });
        canvas.commit_stroke(s1).unwrap();

        let mut s2 = canvas.begin_stroke(crate::canvas::Color::BLACK, 3.0, 0.5);
        s2.add_point(crate::canvas::Point { x: 50.0, y: 20.0 });
        s2.add_point(crate::canvas::Point { x: 200.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_with_strokes.png");

        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export with strokes should succeed");
        assert!(path.exists(), "PNG file should be created");

        // Verify file has non-zero size
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "PNG file should not be empty");

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_single_point_stroke() {
        let mut canvas = Canvas::new();

        let mut s1 = canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0);
        s1.add_point(crate::canvas::Point { x: 10.0, y: 10.0 });
        // Single point commits (not rejected by canvas)
        let result = canvas.commit_stroke(s1);
        assert!(result.is_ok(), "Single-point stroke commits successfully");
        // Single-point strokes don't add to all_strokes (they render as empty)
        assert_eq!(canvas.all_strokes().len(), 0);

        // Export should succeed even with a single-point stroke
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_single_point.png");
        let export_result = export_to_png(&canvas, &path);
        assert!(
            export_result.is_ok(),
            "Export with single-point stroke should succeed"
        );

        let _ = std::fs::remove_file(&path);
    }
}
