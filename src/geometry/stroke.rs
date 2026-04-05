//! Stroke vertex generation for rendering
//!
//! Generates vertex data for brush strokes,
//! used by both WGPU and OpenGL renderers.

use crate::canvas::{ActiveStroke, BrushType, Point, Stroke};

/// Configurable constants for brush types
pub const ROUND_SEGMENTS: usize = 8;
pub const SPRAY_DOTS_PER_SIZE: f32 = 2.0;
pub const SPRAY_DOT_SIZE: f32 = 1.0;
pub const CALLIGRAPHY_ANGLE_DEGREES: f32 = 45.0;

/// Draw mode for stroke meshes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawMode {
    TriangleStrip,
    Triangles,
}

/// Mesh generated from a stroke, containing vertices and the draw mode
#[derive(Debug, Clone)]
pub struct StrokeMesh {
    pub vertices: Vec<f32>,
    pub mode: DrawMode,
}

impl StrokeMesh {
    pub fn new(vertices: Vec<f32>, mode: DrawMode) -> Self {
        Self { vertices, mode }
    }

    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            mode: DrawMode::TriangleStrip,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Generate vertex data for a committed stroke
pub fn generate_stroke_vertices(stroke: &Stroke) -> StrokeMesh {
    generate_stroke_vertices_with_opacity(stroke, 1.0)
}

/// Generate vertex data for a committed stroke with layer opacity applied
pub fn generate_stroke_vertices_with_opacity(stroke: &Stroke, layer_opacity: f32) -> StrokeMesh {
    let color = [
        stroke.color.r as f32 / 255.0,
        stroke.color.g as f32 / 255.0,
        stroke.color.b as f32 / 255.0,
        stroke.opacity * layer_opacity,
    ];
    generate_stroke_for_brush_type(stroke.brush_type, &stroke.points, color, stroke.width / 2.0)
}

/// Generate vertex data for an active stroke being drawn
pub fn generate_active_stroke_vertices(active: &ActiveStroke) -> StrokeMesh {
    generate_active_stroke_vertices_with_opacity(active, 1.0)
}

/// Generate vertex data for an active stroke with layer opacity applied
pub fn generate_active_stroke_vertices_with_opacity(
    active: &ActiveStroke,
    layer_opacity: f32,
) -> StrokeMesh {
    let color = [
        active.color().r as f32 / 255.0,
        active.color().g as f32 / 255.0,
        active.color().b as f32 / 255.0,
        active.opacity() * layer_opacity,
    ];
    generate_stroke_for_brush_type(
        active.brush_type(),
        active.points(),
        color,
        active.width() / 2.0,
    )
}

/// Dispatcher: routes to the correct brush generator based on BrushType
fn generate_stroke_for_brush_type(
    brush_type: BrushType,
    points: &[Point],
    color: [f32; 4],
    half_width: f32,
) -> StrokeMesh {
    if points.len() < 2 {
        return StrokeMesh::empty();
    }
    match brush_type {
        BrushType::Square => generate_square_stroke(points, color, half_width),
        BrushType::Round => generate_round_stroke(points, color, half_width),
        BrushType::Spray => generate_spray_stroke(points, color, half_width),
        BrushType::Calligraphy => generate_calligraphy_stroke(points, color, half_width),
    }
}

/// Square brush: standard triangle strip with 2 vertices per cross-section
fn generate_square_stroke(points: &[Point], color: [f32; 4], half_width: f32) -> StrokeMesh {
    StrokeMesh::new(
        generate_stroke_vertex_strip(points, color, half_width, |_| half_width),
        DrawMode::TriangleStrip,
    )
}

/// Round brush: soft-edged ribbon with alpha falloff at edges
/// Generates 4 vertices per cross-section for smooth feathered edges.
fn generate_round_stroke(points: &[Point], color: [f32; 4], half_width: f32) -> StrokeMesh {
    let mut vertices = Vec::new();

    if points.len() < 2 {
        return StrokeMesh::empty();
    }

    let inner_ratio = 0.6;

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

        let nx = -dy / len;
        let ny = dx / len;

        let outer_hw = half_width;
        let inner_hw = half_width * inner_ratio;

        let lo_x = p.x + nx * outer_hw;
        let lo_y = p.y + ny * outer_hw;
        let li_x = p.x + nx * inner_hw;
        let li_y = p.y + ny * inner_hw;
        let ri_x = p.x - nx * inner_hw;
        let ri_y = p.y - ny * inner_hw;
        let ro_x = p.x - nx * outer_hw;
        let ro_y = p.y - ny * outer_hw;

        vertices.extend_from_slice(&[lo_x, lo_y, color[0], color[1], color[2], 0.0]);
        vertices.extend_from_slice(&[li_x, li_y, color[0], color[1], color[2], color[3]]);
        vertices.extend_from_slice(&[ri_x, ri_y, color[0], color[1], color[2], color[3]]);
        vertices.extend_from_slice(&[ro_x, ro_y, color[0], color[1], color[2], 0.0]);
    }

    StrokeMesh::new(vertices, DrawMode::TriangleStrip)
}

/// Spray brush: scattered dots within brush radius
/// Uses deterministic seeded RNG for stable patterns.
fn generate_spray_stroke(points: &[Point], color: [f32; 4], half_width: f32) -> StrokeMesh {
    let mut vertices = Vec::new();

    for (i, p) in points.iter().enumerate() {
        let dot_count = (half_width * 2.0 * SPRAY_DOTS_PER_SIZE).ceil() as usize;
        let mut seed = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(1);
        for _ in 0..dot_count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let rx = ((seed as i64 & 0xFFFF) as f32 / 32767.5 - 1.0) * half_width;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let ry = ((seed as i64 & 0xFFFF) as f32 / 32767.5 - 1.0) * half_width;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

            let dot_radius = SPRAY_DOT_SIZE;
            let cx = p.x + rx;
            let cy = p.y + ry;

            vertices.extend_from_slice(&[
                cx - dot_radius,
                cy - dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
                cx + dot_radius,
                cy - dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
                cx - dot_radius,
                cy + dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
                cx + dot_radius,
                cy - dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
                cx + dot_radius,
                cy + dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
                cx - dot_radius,
                cy + dot_radius,
                color[0],
                color[1],
                color[2],
                color[3],
            ]);
        }
    }

    StrokeMesh::new(vertices, DrawMode::Triangles)
}

/// Calligraphy brush: 45-degree broad-nib effect
/// Width varies with stroke direction angle relative to nib angle.
fn generate_calligraphy_stroke(points: &[Point], color: [f32; 4], half_width: f32) -> StrokeMesh {
    let nib_angle_rad = CALLIGRAPHY_ANGLE_DEGREES.to_radians();
    StrokeMesh::new(
        generate_stroke_vertex_strip(points, color, half_width, |direction_angle| {
            let angle_diff = (direction_angle - nib_angle_rad).cos().abs();
            half_width * (angle_diff * 0.7 + 0.3)
        }),
        DrawMode::TriangleStrip,
    )
}

/// Shared vertex strip generator with a variable-width callback
fn generate_stroke_vertex_strip<F>(
    points: &[Point],
    color: [f32; 4],
    _base_half_width: f32,
    width_fn: F,
) -> Vec<f32>
where
    F: Fn(f32) -> f32,
{
    let mut vertices = Vec::new();

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

        let direction_angle = dy.atan2(dx);
        let hw = width_fn(direction_angle);

        let nx = -dy / len * hw;
        let ny = dx / len * hw;

        vertices.extend_from_slice(&[p.x + nx, p.y + ny, color[0], color[1], color[2], color[3]]);
        vertices.extend_from_slice(&[p.x - nx, p.y - ny, color[0], color[1], color[2], color[3]]);
    }

    vertices
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
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.mode, DrawMode::TriangleStrip);
    }

    #[test]
    fn test_stroke_single_point_returns_empty() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }],
            color: Color::BLACK,
            width: 5.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_stroke_empty_returns_empty() {
        let stroke = Stroke {
            points: vec![],
            color: Color::BLACK,
            width: 5.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert!(mesh.is_empty());
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
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert_eq!(mesh.vertices.len(), 36);
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
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        let expected_r = 128.0 / 255.0;
        let expected_g = 64.0 / 255.0;
        let expected_b = 32.0 / 255.0;
        assert!((mesh.vertices[2] - expected_r).abs() < 0.01);
        assert!((mesh.vertices[3] - expected_g).abs() < 0.01);
        assert!((mesh.vertices[4] - expected_b).abs() < 0.01);
        assert!((mesh.vertices[5] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stroke_vertex_offset_by_width() {
        let stroke = Stroke {
            points: vec![Point { x: 100.0, y: 100.0 }, Point { x: 100.0, y: 200.0 }],
            color: Color::BLACK,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        let left_x = mesh.vertices[0];
        let right_x = mesh.vertices[6];
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
            brush_type: BrushType::default(),
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn test_generate_stroke_vertices_different_widths() {
        let stroke1 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let stroke2 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::BLACK,
            width: 20.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let m1 = generate_stroke_vertices(&stroke1);
        let m2 = generate_stroke_vertices(&stroke2);
        assert_ne!(m1.vertices, m2.vertices);
    }

    #[test]
    fn test_generate_active_stroke_vertices() {
        let active = ActiveStroke::new(Color::WHITE, 5.0, 0.8, BrushType::default());
        let mesh = generate_active_stroke_vertices(&active);
        assert_eq!(mesh.vertices.len(), 0);
    }

    #[test]
    fn test_round_brush_produces_more_vertices_than_square() {
        let stroke_square = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Square,
        };
        let stroke_round = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Round,
        };
        let square_mesh = generate_stroke_vertices(&stroke_square);
        let round_mesh = generate_stroke_vertices(&stroke_round);
        assert!(round_mesh.vertices.len() > square_mesh.vertices.len());
        assert_eq!(round_mesh.mode, DrawMode::TriangleStrip);
    }

    #[test]
    fn test_spray_brush_uses_triangles_mode() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Spray,
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert!(!mesh.is_empty());
        assert_eq!(mesh.mode, DrawMode::Triangles);
        assert!(
            mesh.vertices.len().is_multiple_of(36),
            "Spray vertex should be multiples of 6 vertices per triangle (6 floats * 6 = 36 per quad)"
        );
    }

    #[test]
    fn test_calligraphy_brush_uses_triangle_strip() {
        let stroke = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Calligraphy,
        };
        let mesh = generate_stroke_vertices(&stroke);
        assert!(!mesh.is_empty());
        assert_eq!(mesh.mode, DrawMode::TriangleStrip);
    }

    #[test]
    fn test_calligraphy_width_varies_with_direction() {
        let horizontal = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Calligraphy,
        };
        let diagonal = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 100.0 }],
            color: Color::WHITE,
            width: 10.0,
            opacity: 1.0,
            brush_type: BrushType::Calligraphy,
        };
        let h_mesh = generate_stroke_vertices(&horizontal);
        let d_mesh = generate_stroke_vertices(&diagonal);
        assert_ne!(h_mesh.vertices, d_mesh.vertices);
    }

    #[test]
    fn test_stroke_mesh_empty_constructor() {
        let mesh = StrokeMesh::empty();
        assert!(mesh.is_empty());
        assert_eq!(mesh.mode, DrawMode::TriangleStrip);
    }

    #[test]
    fn test_stroke_mesh_new() {
        let verts = vec![1.0, 2.0, 3.0];
        let mesh = StrokeMesh::new(verts.clone(), DrawMode::Triangles);
        assert_eq!(mesh.vertices, verts);
        assert_eq!(mesh.mode, DrawMode::Triangles);
        assert!(!mesh.is_empty());
    }
}
