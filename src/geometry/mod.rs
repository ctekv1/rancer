//! Shared geometry and vertex generation for rendering
//!
//! Contains pure-math functions for generating vertex data used by both
//! the WGPU renderer (Windows) and OpenGL renderer (Linux).

mod stroke;
mod ui_elements;

use crate::canvas::Color;

// Re-export all sub-module items so consumers can continue using `geometry::*`
pub use stroke::*;
pub use ui_elements::*;

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

/// Generate vertices for a rotated rectangle (quadrilateral) for diagonal lines
#[allow(clippy::too_many_arguments)]
pub fn generate_rotated_rect(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Vec<f32> {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return Vec::new();
    }

    let half_width = width / 2.0;
    let nx = -dy / len * half_width;
    let ny = dx / len * half_width;

    vec![
        x1 + nx,
        y1 + ny,
        r,
        g,
        b,
        a,
        x1 - nx,
        y1 - ny,
        r,
        g,
        b,
        a,
        x2 + nx,
        y2 + ny,
        r,
        g,
        b,
        a,
        x1 - nx,
        y1 - ny,
        r,
        g,
        b,
        a,
        x2 - nx,
        y2 - ny,
        r,
        g,
        b,
        a,
        x2 + nx,
        y2 + ny,
        r,
        g,
        b,
        a,
    ]
}

/// Helper: Convert HSV (0-360, 0-100, 0-100) to RGB (0.0-1.0)
pub fn hsv_to_rgb_f32(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let s = s / 100.0;
    let v = v / 100.0;

    let h_norm = h / 60.0;
    let i = h_norm.floor() as i32 % 6;
    let f = h_norm - h_norm.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_generate_rect_vertex_count() {
        let vertices = generate_rect(0.0, 0.0, 10.0, 10.0, 1.0, 0.0, 0.0, 1.0);
        assert_eq!(vertices.len(), 36);
    }

    #[test]
    fn test_generate_rect_positions() {
        let vertices = generate_rect(10.0, 20.0, 5.0, 5.0, 0.0, 0.0, 0.0, 1.0);
        assert_eq!([vertices[0], vertices[1]], [10.0, 20.0]);
        assert_eq!([vertices[6], vertices[7]], [15.0, 20.0]);
        assert_eq!([vertices[12], vertices[13]], [10.0, 25.0]);
    }

    #[test]
    fn test_generate_rect_colors() {
        let vertices = generate_rect(0.0, 0.0, 10.0, 10.0, 0.5, 0.3, 0.1, 0.9);
        assert_eq!(&vertices[2..6], &[0.5, 0.3, 0.1, 0.9]);
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
}
