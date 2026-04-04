//! Stroke vertex generation for rendering
//!
//! Generates triangle strip vertex data for brush strokes,
//! used by both WGPU and OpenGL renderers.

use crate::canvas::{ActiveStroke, Point, Stroke};

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
    generate_stroke_vertices_with_opacity(stroke, 1.0)
}

/// Generate vertex data for a committed stroke with layer opacity applied
pub fn generate_stroke_vertices_with_opacity(stroke: &Stroke, layer_opacity: f32) -> Vec<f32> {
    let color = [
        stroke.color.r as f32 / 255.0,
        stroke.color.g as f32 / 255.0,
        stroke.color.b as f32 / 255.0,
        stroke.opacity * layer_opacity,
    ];
    generate_stroke_vertex_strip(&stroke.points, color, stroke.width / 2.0)
}

/// Generate vertex data for an active stroke being drawn
pub fn generate_active_stroke_vertices(active: &ActiveStroke) -> Vec<f32> {
    generate_active_stroke_vertices_with_opacity(active, 1.0)
}

/// Generate vertex data for an active stroke with layer opacity applied
pub fn generate_active_stroke_vertices_with_opacity(
    active: &ActiveStroke,
    layer_opacity: f32,
) -> Vec<f32> {
    let color = [
        active.color().r as f32 / 255.0,
        active.color().g as f32 / 255.0,
        active.color().b as f32 / 255.0,
        active.opacity() * layer_opacity,
    ];
    generate_stroke_vertex_strip(active.points(), color, active.width() / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Color;

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
        let left_x = vertices[0];
        let right_x = vertices[6];
        assert!((left_x - 95.0).abs() < 0.01);
        assert!((right_x - 105.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_stroke_vertices_three_points() {
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
    fn test_generate_active_stroke_vertices() {
        let active = ActiveStroke::new(Color::WHITE, 5.0, 0.8);
        let vertices = generate_active_stroke_vertices(&active);
        assert_eq!(vertices.len(), 0);
    }
}
