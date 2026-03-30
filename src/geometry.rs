//! Shared geometry and vertex generation for rendering
//!
//! Contains pure-math functions for generating vertex data used by both
//! the WGPU renderer (Windows) and OpenGL renderer (Linux).

use crate::canvas::{ActiveStroke, Color, ColorPalette, Point, Stroke};

/// Parse a hex color string into a Color
pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        Color { r, g, b, a: 255 }
    } else {
        Color::WHITE
    }
}

/// Check if a point is inside a triangle using barycentric coordinates
pub fn point_in_triangle(px: f32, py: f32, v0: &[f32; 2], v1: &[f32; 2], v2: &[f32; 2]) -> bool {
    let denominator = (v1[1] - v2[1]) * (v0[0] - v2[0]) + (v2[0] - v1[0]) * (v0[1] - v2[1]);

    if denominator.abs() < 0.0001 {
        return false;
    }

    let w0 = ((v1[1] - v2[1]) * (px - v2[0]) + (v2[0] - v1[0]) * (py - v2[1])) / denominator;
    let w1 = ((v2[1] - v0[1]) * (px - v2[0]) + (v0[0] - v2[0]) * (py - v2[1])) / denominator;
    let w2 = 1.0 - w0 - w1;

    let epsilon = 0.001;
    w0 >= -epsilon && w1 >= -epsilon && w2 >= -epsilon
}

/// Generate vertices for a stroke path as a triangle strip
///
/// Each vertex is [x, y, r, g, b, a].
/// Two vertices per point (left and right of path), forming a triangle strip.
fn generate_stroke_vertex_strip(points: &[Point], color: [f32; 4], half_width: f32) -> Vec<f32> {
    let mut vertices = Vec::new();

    if points.len() < 2 {
        return vertices;
    }

    for i in 0..points.len() {
        let p = &points[i];

        let (dx, dy) = if i == 0 {
            let next = &points[i + 1];
            (next.x - p.x, next.y - p.y)
        } else if i == points.len() - 1 {
            let prev = &points[i - 1];
            (p.x - prev.x, p.y - prev.y)
        } else {
            let prev = &points[i - 1];
            let next = &points[i + 1];
            (next.x - prev.x, next.y - prev.y)
        };

        let len = (dx * dx + dy * dy).sqrt();

        if len < 0.001 {
            continue;
        }

        let nx = -dy / len * half_width;
        let ny = dx / len * half_width;

        vertices.extend_from_slice(&[p.x + nx, p.y + ny, color[0], color[1], color[2], color[3]]);
        vertices.extend_from_slice(&[p.x - nx, p.y - ny, color[0], color[1], color[2], color[3]]);
    }

    vertices
}

/// Generate vertex data for a committed stroke
pub fn generate_stroke_vertices(stroke: &Stroke) -> Vec<f32> {
    let color = [
        stroke.color.r as f32 / 255.0,
        stroke.color.g as f32 / 255.0,
        stroke.color.b as f32 / 255.0,
        stroke.opacity,
    ];
    generate_stroke_vertex_strip(&stroke.points, color, stroke.width / 2.0)
}

/// Generate vertex data for an active stroke being drawn
pub fn generate_active_stroke_vertices(active: &ActiveStroke) -> Vec<f32> {
    let color = [
        active.color().r as f32 / 255.0,
        active.color().g as f32 / 255.0,
        active.color().b as f32 / 255.0,
        active.opacity(),
    ];
    generate_stroke_vertex_strip(active.points(), color, active.width() / 2.0)
}

/// Generate vertices for a filled rectangle (two triangles)
///
/// Each vertex is [x, y, r, g, b, a].
#[allow(clippy::too_many_arguments)]
pub fn generate_rect(x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) -> Vec<f32> {
    vec![
        x,
        y,
        r,
        g,
        b,
        a,
        x + w,
        y,
        r,
        g,
        b,
        a,
        x,
        y + h,
        r,
        g,
        b,
        a,
        x + w,
        y,
        r,
        g,
        b,
        a,
        x + w,
        y + h,
        r,
        g,
        b,
        a,
        x,
        y + h,
        r,
        g,
        b,
        a,
    ]
}

/// Generate vertices for the color palette UI
pub fn generate_palette_vertices(palette: &ColorPalette, selected_index: usize) -> Vec<f32> {
    let mut vertices = Vec::new();
    let colors = palette.colors();

    let palette_x = 10.0;
    let palette_y = 10.0;
    let color_width = 20.0;
    let color_height = 20.0;
    let spacing = 5.0;
    let border_width = 2.0;

    for (i, color) in colors.iter().enumerate() {
        let x = palette_x + (color_width + spacing) * i as f32;
        let cr = color.r as f32 / 255.0;
        let cg = color.g as f32 / 255.0;
        let cb = color.b as f32 / 255.0;

        if i == selected_index {
            vertices.extend(generate_rect(
                x - border_width,
                palette_y - border_width,
                color_width + border_width * 2.0,
                border_width,
                0.0,
                0.0,
                0.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                palette_y + color_height,
                color_width + border_width * 2.0,
                border_width,
                0.0,
                0.0,
                0.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                palette_y - border_width,
                border_width,
                color_height + border_width * 2.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x + color_width,
                palette_y - border_width,
                border_width,
                color_height + border_width * 2.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ));
        }

        vertices.extend(generate_rect(
            x,
            palette_y,
            color_width,
            color_height,
            cr,
            cg,
            cb,
            1.0,
        ));
    }

    vertices
}

/// Generate vertices for the brush size selector UI
pub fn generate_brush_size_vertices(selected_size: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let brush_sizes: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

    let selector_x = 10.0;
    let selector_y = 50.0;
    let button_size = 30.0;
    let spacing = 10.0;
    let border_width = 2.0;

    for (i, &size) in brush_sizes.iter().enumerate() {
        let x = selector_x + (button_size + spacing) * i as f32;

        vertices.extend(generate_rect(
            x,
            selector_y,
            button_size,
            button_size,
            0.8,
            0.8,
            0.8,
            1.0,
        ));

        let indicator_size = size.min(button_size - 4.0);
        let ix = x + (button_size - indicator_size) / 2.0;
        let iy = selector_y + (button_size - indicator_size) / 2.0;
        vertices.extend(generate_rect(
            ix,
            iy,
            indicator_size,
            indicator_size,
            0.0,
            0.0,
            0.0,
            1.0,
        ));

        if (size - selected_size).abs() < 0.1 {
            // Top border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                button_size + border_width * 2.0,
                border_width,
                0.0,
                0.0,
                1.0,
                1.0,
            ));
            // Bottom border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y + button_size,
                button_size + border_width * 2.0,
                border_width,
                0.0,
                0.0,
                1.0,
                1.0,
            ));
            // Left border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                border_width,
                button_size + border_width * 2.0,
                0.0,
                0.0,
                1.0,
                1.0,
            ));
            // Right border
            vertices.extend(generate_rect(
                x + button_size,
                selector_y - border_width,
                border_width,
                button_size + border_width * 2.0,
                0.0,
                0.0,
                1.0,
                1.0,
            ));
        }
    }

    vertices
}

/// Generate vertices for the eraser toggle button UI
pub fn generate_eraser_button_vertices(is_active: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let eraser_x = 10.0;
    let eraser_y = 85.0;
    let button_size = 30.0;
    let border_width = 2.0;

    // Background
    vertices.extend(generate_rect(
        eraser_x,
        eraser_y,
        button_size,
        button_size,
        0.85,
        0.85,
        0.85,
        1.0,
    ));

    // Eraser body (pink/coral)
    let body_padding = 4.0;
    let body_width = button_size - body_padding * 2.0;
    let body_height = button_size - body_padding * 2.0;
    vertices.extend(generate_rect(
        eraser_x + body_padding,
        eraser_y + body_padding,
        body_width,
        body_height,
        0.9,
        0.6,
        0.6,
        1.0,
    ));

    // Eraser top (cream) - top portion
    let top_height = body_height * 0.35;
    let top_width = body_width - 4.0;
    let top_x = eraser_x + body_padding + 2.0;
    let top_y = eraser_y + body_padding;
    vertices.extend(generate_rect(
        top_x, top_y, top_width, top_height, 0.95, 0.92, 0.85, 1.0,
    ));

    // Eraser stripe (darker rose) - bottom portion
    let stripe_height = body_height * 0.2;
    let stripe_y = eraser_y + body_padding + body_height - stripe_height;
    vertices.extend(generate_rect(
        eraser_x + body_padding,
        stripe_y,
        body_width,
        stripe_height,
        0.75,
        0.4,
        0.45,
        1.0,
    ));

    if is_active {
        vertices.extend(generate_rect(
            eraser_x - border_width,
            eraser_y - border_width,
            button_size + border_width * 2.0,
            border_width,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            eraser_x - border_width,
            eraser_y + button_size,
            button_size + border_width * 2.0,
            border_width,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            eraser_x - border_width,
            eraser_y - border_width,
            border_width,
            button_size + border_width * 2.0,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            eraser_x + button_size,
            eraser_y - border_width,
            border_width,
            button_size + border_width * 2.0,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- hex_to_color tests ---

    #[test]
    fn test_hex_valid_no_hash() {
        let c = hex_to_color("FF0000");
        assert_eq!(
            c,
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_valid_with_hash() {
        let c = hex_to_color("#00FF00");
        assert_eq!(
            c,
            Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_blue() {
        let c = hex_to_color("#0000FF");
        assert_eq!(
            c,
            Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_black() {
        let c = hex_to_color("#000000");
        assert_eq!(
            c,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_white() {
        let c = hex_to_color("#FFFFFF");
        assert_eq!(
            c,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_invalid_chars() {
        let c = hex_to_color("#ZZZZZZ");
        // Invalid chars default to 255
        assert_eq!(
            c,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
    }

    #[test]
    fn test_hex_too_short() {
        let c = hex_to_color("#FFF");
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn test_hex_too_long() {
        let c = hex_to_color("#FF0000FF");
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn test_hex_empty() {
        let c = hex_to_color("");
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn test_hex_lowercase() {
        let c = hex_to_color("#ff8800");
        assert_eq!(
            c,
            Color {
                r: 255,
                g: 136,
                b: 0,
                a: 255
            }
        );
    }

    // --- point_in_triangle tests ---

    #[test]
    fn test_point_inside_triangle() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];
        assert!(point_in_triangle(5.0, 5.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_point_outside_triangle() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];
        assert!(!point_in_triangle(0.0, 10.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_point_on_vertex() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];
        assert!(point_in_triangle(0.0, 0.0, &v0, &v1, &v2));
        assert!(point_in_triangle(10.0, 0.0, &v0, &v1, &v2));
        assert!(point_in_triangle(5.0, 10.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_degenerate_triangle() {
        // All three vertices are colinear
        let v0 = [0.0, 0.0];
        let v1 = [5.0, 5.0];
        let v2 = [10.0, 10.0];
        assert!(!point_in_triangle(5.0, 5.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_point_outside_far_away() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];
        assert!(!point_in_triangle(100.0, 100.0, &v0, &v1, &v2));
    }

    // --- generate_rect tests ---

    #[test]
    fn test_generate_rect_vertex_count() {
        let vertices = generate_rect(0.0, 0.0, 10.0, 10.0, 1.0, 0.0, 0.0, 1.0);
        // 6 vertices * 6 floats each = 36 floats
        assert_eq!(vertices.len(), 36);
    }

    #[test]
    fn test_generate_rect_positions() {
        let vertices = generate_rect(10.0, 20.0, 5.0, 5.0, 0.0, 0.0, 0.0, 1.0);
        // First triangle: (10,20), (15,20), (10,25)
        assert_eq!([vertices[0], vertices[1]], [10.0, 20.0]);
        assert_eq!([vertices[6], vertices[7]], [15.0, 20.0]);
        assert_eq!([vertices[12], vertices[13]], [10.0, 25.0]);
    }

    #[test]
    fn test_generate_rect_colors() {
        let vertices = generate_rect(0.0, 0.0, 10.0, 10.0, 0.5, 0.3, 0.1, 0.9);
        // First vertex color: [0.5, 0.3, 0.1, 0.9]
        assert_eq!(&vertices[2..6], &[0.5, 0.3, 0.1, 0.9]);
    }

    // --- generate_stroke_vertices tests ---

    #[test]
    fn test_stroke_two_points() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            width: 10.0,
            opacity: 1.0,
        };
        let vertices = generate_stroke_vertices(&stroke);
        // 2 points * 2 vertices each * 6 floats = 24 floats
        assert_eq!(vertices.len(), 24);
    }

    #[test]
    fn test_stroke_single_point_returns_empty() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }],
            color: Color::BLACK,
            width: 5.0,
            opacity: 1.0,
        };
        let vertices = generate_stroke_vertices(&stroke);
        assert!(vertices.is_empty());
    }

    #[test]
    fn test_stroke_empty_returns_empty() {
        let stroke = Stroke {
            points: vec![],
            color: Color::BLACK,
            width: 5.0,
            opacity: 1.0,
        };
        let vertices = generate_stroke_vertices(&stroke);
        assert!(vertices.is_empty());
    }

    #[test]
    fn test_stroke_three_points() {
        let stroke = Stroke {
            points: vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 50.0, y: 0.0 },
                Point { x: 100.0, y: 0.0 },
            ],
            color: Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
            width: 4.0,
            opacity: 1.0,
        };
        let vertices = generate_stroke_vertices(&stroke);
        // 3 points * 2 vertices * 6 floats = 36
        assert_eq!(vertices.len(), 36);
    }

    #[test]
    fn test_stroke_color_values() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 0.0 }],
            color: Color {
                r: 128,
                g: 64,
                b: 32,
                a: 255,
            },
            width: 2.0,
            opacity: 0.5,
        };
        let vertices = generate_stroke_vertices(&stroke);
        // First vertex color (index 2..6)
        let expected_r = 128.0 / 255.0;
        let expected_g = 64.0 / 255.0;
        let expected_b = 32.0 / 255.0;
        assert!((vertices[2] - expected_r).abs() < 0.01);
        assert!((vertices[3] - expected_g).abs() < 0.01);
        assert!((vertices[4] - expected_b).abs() < 0.01);
        assert!((vertices[5] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stroke_vertex_offset_by_width() {
        let stroke = Stroke {
            points: vec![Point { x: 100.0, y: 100.0 }, Point { x: 100.0, y: 200.0 }],
            color: Color::BLACK,
            width: 10.0,
            opacity: 1.0,
        };
        let vertices = generate_stroke_vertices(&stroke);
        // First vertex should be offset by half_width (5.0) perpendicular to path
        // Path is vertical (dx=0, dy=100), perpendicular is (-dy, dx) = (-100, 0)
        // Normalized: (-1, 0) * 5.0 = (-5, 0)
        // Left vertex: 100 + (-5) = 95.0, Right vertex: 100 - (-5) = 105.0
        let left_x = vertices[0];
        let right_x = vertices[6];
        assert!((left_x - 95.0).abs() < 0.01);
        assert!((right_x - 105.0).abs() < 0.01);
    }

    // --- generate_palette_vertices tests ---

    #[test]
    fn test_palette_vertex_count_no_selection() {
        let palette = ColorPalette::new();
        let vertices = generate_palette_vertices(&palette, 999);
        // 10 colors * 6 vertices each * 6 floats = 360
        assert_eq!(vertices.len(), 360);
    }

    #[test]
    fn test_palette_vertex_count_with_selection() {
        let palette = ColorPalette::new();
        let vertices = generate_palette_vertices(&palette, 0);
        // 10 colors * 6 vertices + 4 border rects * 6 vertices = 60 + 24 = 84 vertices * 6 floats = 504
        assert_eq!(vertices.len(), 504);
    }

    // --- generate_brush_size_vertices tests ---

    #[test]
    fn test_brush_size_vertex_count() {
        let vertices = generate_brush_size_vertices(3.0);
        // 5 buttons: each has bg (6 verts) + indicator (6 verts) + border (24 verts for selected) = 36 for selected + 12 for others
        // 1 selected (36) + 4 unselected (12) = 36 + 48 = 84 vertices * 6 floats = 504
        assert_eq!(vertices.len(), 504);
    }

    #[test]
    fn test_brush_size_different_selections() {
        let v3 = generate_brush_size_vertices(3.0);
        let v50 = generate_brush_size_vertices(50.0);
        // Both should produce same vertex count
        assert_eq!(v3.len(), v50.len());
        // But the actual vertex data should differ (different selection highlight positions)
        assert_ne!(v3, v50);
    }

    #[test]
    fn test_brush_size_invalid_selection() {
        // If selected size doesn't match any button, no border is drawn
        let vertices = generate_brush_size_vertices(100.0);
        // 5 buttons * (bg + indicator) = 5 * 12 = 60 vertices * 6 floats = 360
        assert_eq!(vertices.len(), 360);
    }

    // --- generate_eraser_button_vertices tests ---

    #[test]
    fn test_eraser_button_inactive() {
        let vertices = generate_eraser_button_vertices(false);
        // Button bg (6 verts) + body (6 verts) + top (6 verts) + stripe (6 verts) = 24 verts * 6 floats = 144
        assert_eq!(vertices.len(), 144);
    }

    #[test]
    fn test_eraser_button_active() {
        let vertices = generate_eraser_button_vertices(true);
        // Button bg (6) + body (6) + top (6) + stripe (6) + 4 border rects (24) = 48 verts * 6 floats = 288
        assert_eq!(vertices.len(), 288);
    }

    #[test]
    fn test_eraser_button_active_larger_than_inactive() {
        let inactive = generate_eraser_button_vertices(false);
        let active = generate_eraser_button_vertices(true);
        assert!(active.len() > inactive.len());
    }

    #[test]
    fn test_generate_rect_zero_dimensions() {
        let vertices = generate_rect(0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        let vertices2 = generate_rect(10.0, 10.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        let vertices3 = generate_rect(10.0, 10.0, 10.0, 0.0, 1.0, 1.0, 1.0, 1.0);
        let vertices4 = generate_rect(10.0, 10.0, 0.0, 10.0, 1.0, 1.0, 1.0, 1.0);
        assert!(!vertices.is_empty());
        assert!(!vertices2.is_empty());
        assert!(!vertices3.is_empty());
        assert!(!vertices4.is_empty());
    }

    #[test]
    fn test_generate_rect_negative_dimensions() {
        let vertices = generate_rect(-10.0, -10.0, -5.0, -5.0, 1.0, 0.0, 0.0, 1.0);
        let vertices2 = generate_rect(-10.0, -10.0, 5.0, 5.0, 1.0, 0.0, 0.0, 1.0);
        assert!(!vertices.is_empty());
        assert!(!vertices2.is_empty());
    }

    #[test]
    fn test_generate_rect_all_channels() {
        let vertices = generate_rect(0.0, 0.0, 10.0, 10.0, 0.1, 0.2, 0.3, 0.4);
        assert_eq!(vertices.len(), 36);
        assert!((vertices[2] - 0.1).abs() < 0.01);
        assert!((vertices[3] - 0.2).abs() < 0.01);
        assert!((vertices[4] - 0.3).abs() < 0.01);
        assert!((vertices[5] - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_generate_stroke_vertices_three_points() {
        use crate::canvas::{Color, Point, Stroke};

        let stroke = Stroke {
            points: vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 50.0, y: 50.0 },
                Point { x: 100.0, y: 100.0 },
            ],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
        };

        let vertices = generate_stroke_vertices(&stroke);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_stroke_vertices_different_widths() {
        use crate::canvas::{Color, Point, Stroke};

        let stroke1 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 2.0,
            opacity: 1.0,
        };

        let stroke2 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::BLACK,
            width: 20.0,
            opacity: 1.0,
        };

        let v1 = generate_stroke_vertices(&stroke1);
        let v2 = generate_stroke_vertices(&stroke2);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_point_in_triangle_center() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];

        assert!(point_in_triangle(5.0, 5.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_point_in_triangle_outside() {
        let v0 = [0.0, 0.0];
        let v1 = [10.0, 0.0];
        let v2 = [5.0, 10.0];

        assert!(!point_in_triangle(100.0, 100.0, &v0, &v1, &v2));
    }

    #[test]
    fn test_hex_variations() {
        let c1 = hex_to_color("#FF0000");
        let c2 = hex_to_color("FF0000");
        assert_eq!(c1, c2);

        let c3 = hex_to_color("ff0000");
        let c4 = hex_to_color("Ff0000");
        assert_eq!(c3, c4);
    }

    #[test]
    fn test_generate_active_stroke_vertices() {
        use crate::canvas::ActiveStroke;
        use crate::canvas::Color;

        let active = ActiveStroke::new(Color::WHITE, 5.0, 0.8);
        let vertices = generate_active_stroke_vertices(&active);
        assert_eq!(vertices.len(), 0);
    }
}
