//! Export module for Rancer
//!
//! Provides canvas export functionality to PNG format.
//! Uses software rendering to convert strokes to image pixels.
//!
//! ## Export Size Limits
//! - Minimum: 100x100 (empty canvas or single-point strokes)
//! - Maximum: 4096x4096 (content exceeding this is clipped)
//! - Padding: 20px around the stroke bounding box

use crate::canvas::{Canvas, Stroke};
use crate::geometry::{self, DrawMode};
use crate::logger;
use image::{ImageBuffer, Rgba};
use std::path::Path;

const EXPORT_PADDING: f32 = 20.0;
const MIN_EXPORT_SIZE: u32 = 100;
const MAX_EXPORT_SIZE: u32 = 4096;

/// Export canvas to PNG file
pub fn export_to_png(canvas: &Canvas, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    logger::info(&format!("Exporting canvas to PNG: {:?}", path));

    let image = render_canvas_to_image(canvas)?;
    image.save(path)?;

    logger::info(&format!("Export successful: {:?}", path));
    Ok(())
}

/// Compute the bounding box of all strokes across all visible layers.
/// Returns (min_x, min_y, max_x, max_y) or None if no strokes exist.
fn compute_stroke_bounding_box(canvas: &Canvas) -> Option<(f32, f32, f32, f32)> {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut has_strokes = false;

    for (stroke, _) in canvas.all_strokes() {
        for point in &stroke.points {
            let half_w = stroke.width / 2.0;
            min_x = min_x.min(point.x - half_w);
            min_y = min_y.min(point.y - half_w);
            max_x = max_x.max(point.x + half_w);
            max_y = max_y.max(point.y + half_w);
            has_strokes = true;
        }
    }

    if has_strokes {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Render canvas to image buffer using software rendering
fn render_canvas_to_image(
    canvas: &Canvas,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let (width, height) =
        if let Some((min_x, min_y, max_x, max_y)) = compute_stroke_bounding_box(canvas) {
            let content_w = max_x - min_x + EXPORT_PADDING * 2.0;
            let content_h = max_y - min_y + EXPORT_PADDING * 2.0;
            if content_w > MAX_EXPORT_SIZE as f32 || content_h > MAX_EXPORT_SIZE as f32 {
                logger::warn(&format!(
                    "Content exceeds max export size ({}x{}), export will be clipped to {}x{}",
                    content_w.ceil() as u32,
                    content_h.ceil() as u32,
                    MAX_EXPORT_SIZE,
                    MAX_EXPORT_SIZE,
                ));
            }
            let w = content_w
                .ceil()
                .max(MIN_EXPORT_SIZE as f32)
                .min(MAX_EXPORT_SIZE as f32) as u32;
            let h = content_h
                .ceil()
                .max(MIN_EXPORT_SIZE as f32)
                .min(MAX_EXPORT_SIZE as f32) as u32;
            (w, h)
        } else {
            (MIN_EXPORT_SIZE, MIN_EXPORT_SIZE)
        };

    let mut image = ImageBuffer::new(width, height);

    // Fill background
    let bg_color = canvas.background_color();
    for pixel in image.pixels_mut() {
        *pixel = Rgba([bg_color.r, bg_color.g, bg_color.b, bg_color.a]);
    }

    // Compute offset: shift all strokes so the bounding box fits in the image
    let (offset_x, offset_y) =
        if let Some((min_x, min_y, _, _)) = compute_stroke_bounding_box(canvas) {
            (min_x - EXPORT_PADDING, min_y - EXPORT_PADDING)
        } else {
            (0.0, 0.0)
        };

    // Render each stroke from all visible layers
    for (stroke, layer_opacity) in canvas.all_strokes() {
        let adjusted_stroke = Stroke {
            opacity: stroke.opacity * layer_opacity,
            brush_type: stroke.brush_type,
            ..stroke.clone()
        };
        render_stroke_to_image(&mut image, &adjusted_stroke, offset_x, offset_y)?;
    }

    Ok(image)
}

/// Render a single stroke to image buffer
fn render_stroke_to_image(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    stroke: &Stroke,
    offset_x: f32,
    offset_y: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mesh = geometry::generate_stroke_vertices_with_opacity(stroke, stroke.opacity);

    if mesh.vertices.len() < 18 {
        return Ok(());
    }

    let color = Rgba([
        stroke.color.r,
        stroke.color.g,
        stroke.color.b,
        (stroke.opacity * 255.0) as u8,
    ]);

    match mesh.mode {
        DrawMode::TriangleStrip => {
            for i in (0..mesh.vertices.len() - 12).step_by(12) {
                let v0 = [mesh.vertices[i] - offset_x, mesh.vertices[i + 1] - offset_y];
                let v1 = [
                    mesh.vertices[i + 6] - offset_x,
                    mesh.vertices[i + 7] - offset_y,
                ];
                let v2 = [
                    mesh.vertices[i + 12] - offset_x,
                    mesh.vertices[i + 13] - offset_y,
                ];

                render_triangle(image, &v0, &v1, &v2, color);

                if i + 18 <= mesh.vertices.len() {
                    let v3 = [
                        mesh.vertices[i + 18] - offset_x,
                        mesh.vertices[i + 19] - offset_y,
                    ];
                    render_triangle(image, &v1, &v2, &v3, color);
                }
            }
        }
        DrawMode::Triangles => {
            for i in (0..mesh.vertices.len()).step_by(18) {
                if i + 18 > mesh.vertices.len() {
                    break;
                }
                let v0 = [mesh.vertices[i] - offset_x, mesh.vertices[i + 1] - offset_y];
                let v1 = [
                    mesh.vertices[i + 6] - offset_x,
                    mesh.vertices[i + 7] - offset_y,
                ];
                let v2 = [
                    mesh.vertices[i + 12] - offset_x,
                    mesh.vertices[i + 13] - offset_y,
                ];

                render_triangle(image, &v0, &v1, &v2, color);
            }
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

    let min_x = v0[0].min(v1[0]).min(v2[0]).max(0.0).floor() as u32;
    let max_x = v0[0].max(v1[0]).max(v2[0]).min(width - 1.0).ceil() as u32;
    let min_y = v0[1].min(v1[1]).min(v2[1]).max(0.0).floor() as u32;
    let max_y = v0[1].max(v1[1]).max(v2[1]).min(height - 1.0).ceil() as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            if geometry::point_in_triangle(px, py, v0, v1, v2) {
                image.put_pixel(x, y, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{BrushType, Canvas, Point};

    #[test]
    fn test_export_creates_valid_png() {
        let canvas = Canvas::new();
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export.png");

        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export should succeed");
        assert!(path.exists(), "PNG file should be created");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_point_in_triangle_via_geometry() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];

        assert!(geometry::point_in_triangle(5.0, 5.0, &v0, &v1, &v2));
        assert!(!geometry::point_in_triangle(0.0, 10.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_export_with_strokes() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(crate::canvas::Point { x: 10.0, y: 10.0 });
        s1.add_point(crate::canvas::Point { x: 100.0, y: 100.0 });
        canvas.commit_stroke(s1).unwrap();

        let mut s2 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 3.0, 0.5, BrushType::default());
        s2.add_point(crate::canvas::Point { x: 50.0, y: 20.0 });
        s2.add_point(crate::canvas::Point { x: 200.0, y: 30.0 });
        canvas.commit_stroke(s2).unwrap();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_with_strokes.png");

        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export with strokes should succeed");
        assert!(path.exists(), "PNG file should be created");

        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "PNG file should not be empty");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_single_point_stroke() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(crate::canvas::Point { x: 10.0, y: 10.0 });
        let result = canvas.commit_stroke(s1);
        assert!(result.is_ok(), "Single-point stroke commits successfully");
        assert_eq!(canvas.all_strokes().len(), 0);

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_single_point.png");
        let export_result = export_to_png(&canvas, &path);
        assert!(
            export_result.is_ok(),
            "Export with single-point stroke should succeed"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_captures_strokes_far_from_origin() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(Point {
            x: 5000.0,
            y: 5000.0,
        });
        s1.add_point(Point {
            x: 5100.0,
            y: 5100.0,
        });
        canvas.commit_stroke(s1).unwrap();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_far_stroke.png");

        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export should succeed");
        assert!(path.exists(), "PNG file should be created");

        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "PNG file should contain stroke data");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_captures_negative_coordinates() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(Point {
            x: -200.0,
            y: -200.0,
        });
        s1.add_point(Point {
            x: -100.0,
            y: -100.0,
        });
        canvas.commit_stroke(s1).unwrap();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_export_negative.png");

        let result = export_to_png(&canvas, &path);
        assert!(result.is_ok(), "Export should succeed");
        assert!(path.exists(), "PNG file should be created");

        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "PNG file should contain stroke data");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_empty_canvas_minimum_size() {
        let canvas = Canvas::new();
        let image = render_canvas_to_image(&canvas).unwrap();
        assert_eq!(image.width(), MIN_EXPORT_SIZE);
        assert_eq!(image.height(), MIN_EXPORT_SIZE);
    }

    #[test]
    fn test_export_scattered_strokes_captures_all() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point { x: 50.0, y: 50.0 });
        canvas.commit_stroke(s1).unwrap();

        let mut s2 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s2.add_point(Point {
            x: 2000.0,
            y: 2000.0,
        });
        s2.add_point(Point {
            x: 2050.0,
            y: 2050.0,
        });
        canvas.commit_stroke(s2).unwrap();

        let image = render_canvas_to_image(&canvas).unwrap();
        assert!(image.width() > 2000);
        assert!(image.height() > 2000);
    }

    #[test]
    fn test_compute_stroke_bounding_box_empty() {
        let canvas = Canvas::new();
        assert!(compute_stroke_bounding_box(&canvas).is_none());
    }

    #[test]
    fn test_compute_stroke_bounding_box_single_stroke() {
        let mut canvas = Canvas::new();
        let mut s =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 10.0, 1.0, BrushType::default());
        s.add_point(Point { x: 100.0, y: 200.0 });
        s.add_point(Point { x: 300.0, y: 400.0 });
        canvas.commit_stroke(s).unwrap();

        let bbox = compute_stroke_bounding_box(&canvas).unwrap();
        assert!(bbox.0 <= 95.0);
        assert!(bbox.1 <= 195.0);
        assert!(bbox.2 >= 305.0);
        assert!(bbox.3 >= 405.0);
    }

    #[test]
    fn test_export_capped_at_max_size() {
        let mut canvas = Canvas::new();

        let mut s1 =
            canvas.begin_stroke(crate::canvas::Color::BLACK, 5.0, 1.0, BrushType::default());
        s1.add_point(Point { x: 0.0, y: 0.0 });
        s1.add_point(Point {
            x: 5000.0,
            y: 5000.0,
        });
        canvas.commit_stroke(s1).unwrap();

        let image = render_canvas_to_image(&canvas).unwrap();
        assert_eq!(image.width(), MAX_EXPORT_SIZE);
        assert_eq!(image.height(), MAX_EXPORT_SIZE);
    }
}
