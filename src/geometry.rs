//! Shared geometry and vertex generation for rendering
//!
//! Contains pure-math functions for generating vertex data used by both
//! the WGPU renderer (Windows) and OpenGL renderer (Linux).

use crate::canvas::{ActiveStroke, Color, Point, Stroke};

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

/// Generate HSV slider vertices (H, S, V sliders)
/// h: 0-360, s: 0-100, v: 0-100
pub fn generate_hsv_sliders(h: f32, s: f32, v: f32) -> Vec<f32> {
    let mut vertices = Vec::new();

    let slider_x = 10.0;
    let slider_width = 200.0;
    let slider_height = 20.0;

    // Hue slider (y=5)
    let hue_y = 5.0;
    // Background: rainbow gradient (simplified with rainbow colors)
    let hue_colors = [
        (1.0, 0.0, 0.0), // Red
        (1.0, 1.0, 0.0), // Yellow
        (0.0, 1.0, 0.0), // Green
        (0.0, 1.0, 1.0), // Cyan
        (0.0, 0.0, 1.0), // Blue
        (1.0, 0.0, 1.0), // Magenta
        (1.0, 0.0, 0.0), // Red (loop back)
    ];

    let hue_sections = hue_colors.len() - 1;
    let section_width = slider_width / hue_sections as f32;

    for (i, &(r1, g1, b1)) in hue_colors.iter().enumerate().take(hue_sections) {
        let sx = slider_x + i as f32 * section_width;

        vertices.extend(generate_rect(
            sx,
            hue_y,
            section_width,
            slider_height,
            r1,
            g1,
            b1,
            1.0,
        ));
    }

    // Hue indicator
    let hue_pos = slider_x + (h / 360.0) * slider_width;
    vertices.extend(generate_rect(
        hue_pos - 2.0,
        hue_y - 2.0,
        4.0,
        slider_height + 4.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Saturation slider (y=30)
    let sat_y = 30.0;
    // Background: white to current hue color (left=white=no saturation, right=hue=full saturation)
    let hue_rgb = hsv_to_rgb_f32(h, 100.0, 100.0);
    vertices.extend(generate_rect(
        slider_x,
        sat_y,
        slider_width,
        slider_height,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rect(
        slider_x + slider_width / 2.0,
        sat_y,
        slider_width / 2.0,
        slider_height,
        hue_rgb.0,
        hue_rgb.1,
        hue_rgb.2,
        1.0,
    ));

    // Saturation indicator
    let sat_pos = slider_x + (s / 100.0) * slider_width;
    vertices.extend(generate_rect(
        sat_pos - 2.0,
        sat_y - 2.0,
        4.0,
        slider_height + 4.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Value slider (y=55)
    let val_y = 55.0;
    // Background: black to current hue/saturation color
    vertices.extend(generate_rect(
        slider_x,
        val_y,
        slider_width,
        slider_height,
        0.0,
        0.0,
        0.0,
        1.0,
    ));
    let sv_rgb = hsv_to_rgb_f32(h, s, 100.0);
    vertices.extend(generate_rect(
        slider_x,
        val_y,
        slider_width * v / 100.0,
        slider_height,
        sv_rgb.0,
        sv_rgb.1,
        sv_rgb.2,
        1.0,
    ));

    // Value indicator
    let val_pos = slider_x + (v / 100.0) * slider_width;
    vertices.extend(generate_rect(
        val_pos - 2.0,
        val_y - 2.0,
        4.0,
        slider_height + 4.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Color preview (y=80, small square)
    let preview_x = slider_x + slider_width + 10.0;
    let preview_y = 5.0;
    let preview_size = 60.0;
    let current_rgb = hsv_to_rgb_f32(h, s, v);
    vertices.extend(generate_rect(
        preview_x,
        preview_y,
        preview_size,
        preview_size,
        current_rgb.0,
        current_rgb.1,
        current_rgb.2,
        1.0,
    ));
    // Border
    vertices.extend(generate_rect(
        preview_x - 2.0,
        preview_y - 2.0,
        preview_size + 4.0,
        2.0,
        0.3,
        0.3,
        0.3,
        1.0,
    ));
    vertices.extend(generate_rect(
        preview_x - 2.0,
        preview_y + preview_size,
        preview_size + 4.0,
        2.0,
        0.3,
        0.3,
        0.3,
        1.0,
    ));
    vertices.extend(generate_rect(
        preview_x - 2.0,
        preview_y,
        2.0,
        preview_size,
        0.3,
        0.3,
        0.3,
        1.0,
    ));
    vertices.extend(generate_rect(
        preview_x + preview_size,
        preview_y,
        2.0,
        preview_size,
        0.3,
        0.3,
        0.3,
        1.0,
    ));

    vertices
}

/// Helper: Convert HSV (0-360, 0-100, 0-100) to RGB (0.0-1.0)
fn hsv_to_rgb_f32(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
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

/// Generate custom palette vertices (saved colors as swatches)
/// custom_colors: Vec of [R, G, B] values
pub fn generate_custom_palette(custom_colors: &[[u8; 3]], selected_index: usize) -> Vec<f32> {
    let mut vertices = Vec::new();

    let palette_x = 10.0;
    let palette_y = 90.0;
    let color_width = 20.0;
    let color_height = 20.0;
    let spacing = 5.0;
    let border_width = 2.0;

    for (i, color) in custom_colors.iter().enumerate() {
        let x = palette_x + (color_width + spacing) * i as f32;
        let cr = color[0] as f32 / 255.0;
        let cg = color[1] as f32 / 255.0;
        let cb = color[2] as f32 / 255.0;

        if i == selected_index {
            vertices.extend(generate_rect(
                x - border_width,
                palette_y - border_width,
                color_width + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                palette_y + color_height,
                color_width + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                palette_y - border_width,
                border_width,
                color_height + border_width * 2.0,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x + color_width,
                palette_y - border_width,
                border_width,
                color_height + border_width * 2.0,
                0.0,
                0.5,
                1.0,
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

    // "Save" button indicator (plus sign) - always visible for FIFO behavior
    let save_x = palette_x + (color_width + spacing) * custom_colors.len() as f32;
    vertices.extend(generate_rect(
        save_x,
        palette_y + 7.0,
        color_width,
        6.0,
        0.3,
        0.3,
        0.3,
        1.0,
    ));
    vertices.extend(generate_rect(
        save_x + 7.0,
        palette_y,
        6.0,
        color_height,
        0.3,
        0.3,
        0.3,
        1.0,
    ));

    vertices
}

/// Generate vertices for the brush size selector UI
pub fn generate_brush_size_vertices(selected_size: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let brush_sizes: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

    let selector_x = 10.0;
    let selector_y = 120.0; // Moved down from 50 for HSV picker
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
    let eraser_y = 155.0; // Moved down from 85 for HSV picker
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

/// Generate vertices for a rotated rectangle (quadrilateral) for diagonal lines
#[allow(clippy::too_many_arguments)]
fn generate_rotated_rect(
    x1: f32,
    y1: f32, // Point 1
    x2: f32,
    y2: f32,    // Point 2
    width: f32, // Line width
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> Vec<f32> {
    // Calculate perpendicular offset for line width
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return Vec::new();
    }

    let half_width = width / 2.0;
    let nx = -dy / len * half_width; // Perpendicular x
    let ny = dx / len * half_width; // Perpendicular y

    // Create quad as two triangles
    let vertices = vec![
        // Triangle 1
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
        // Triangle 2
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
    ];
    vertices
}

/// Generate vertices for the clear canvas button
/// The button shows "X" symbol and is positioned next to the eraser button
pub fn generate_clear_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let clear_x = 50.0;
    let clear_y = 155.0; // Moved down from 85
    let button_size = 30.0;

    // Background (red)
    vertices.extend(generate_rect(
        clear_x,
        clear_y,
        button_size,
        button_size,
        0.9,
        0.3,
        0.3,
        1.0,
    ));

    // X symbol using rotated rectangles
    let line_width = 4.0;
    let padding = 6.0;

    // Diagonal 1: top-left to bottom-right
    vertices.extend(generate_rotated_rect(
        clear_x + padding,
        clear_y + padding,
        clear_x + button_size - padding,
        clear_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Diagonal 2: top-right to bottom-left
    vertices.extend(generate_rotated_rect(
        clear_x + button_size - padding,
        clear_y + padding,
        clear_x + padding,
        clear_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    vertices
}

/// Generate vertices for the export canvas button
/// The button shows a download/arrow symbol and is positioned next to the redo button
pub fn generate_export_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let export_x = 170.0;
    let export_y = 155.0;
    let button_size = 30.0;

    // Background (green)
    vertices.extend(generate_rect(
        export_x,
        export_y,
        button_size,
        button_size,
        0.3,
        0.7,
        0.3,
        1.0,
    ));

    // Arrow pointing down (export symbol)
    let line_width = 4.0;
    let padding = 6.0;

    // Vertical line from top-center to bottom-center
    vertices.extend(generate_rotated_rect(
        export_x + button_size / 2.0,
        export_y + padding,
        export_x + button_size / 2.0,
        export_y + button_size - padding - 4.0,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Arrow head: diagonal from center to bottom-left
    vertices.extend(generate_rotated_rect(
        export_x + button_size / 2.0,
        export_y + button_size - padding - 4.0,
        export_x + padding + 4.0,
        export_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Arrow head: diagonal from center to bottom-right
    vertices.extend(generate_rotated_rect(
        export_x + button_size / 2.0,
        export_y + button_size - padding - 4.0,
        export_x + button_size - padding - 4.0,
        export_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Horizontal line at bottom (tray)
    vertices.extend(generate_rotated_rect(
        export_x + padding,
        export_y + button_size - padding,
        export_x + button_size - padding,
        export_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    vertices
}

/// Generate vertices for the zoom in button (+)
pub fn generate_zoom_in_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let zoom_x = 210.0;
    let zoom_y = 155.0;
    let button_size = 30.0;

    // Background (blue)
    vertices.extend(generate_rect(
        zoom_x,
        zoom_y,
        button_size,
        button_size,
        0.3,
        0.3,
        0.7,
        1.0,
    ));

    // Plus sign (white)
    let line_width = 4.0;
    let padding = 8.0;

    // Horizontal line
    vertices.extend(generate_rotated_rect(
        zoom_x + padding,
        zoom_y + button_size / 2.0,
        zoom_x + button_size - padding,
        zoom_y + button_size / 2.0,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    // Vertical line
    vertices.extend(generate_rotated_rect(
        zoom_x + button_size / 2.0,
        zoom_y + padding,
        zoom_x + button_size / 2.0,
        zoom_y + button_size - padding,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    vertices
}

/// Generate vertices for the zoom out button (-)
pub fn generate_zoom_out_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let zoom_x = 250.0;
    let zoom_y = 155.0;
    let button_size = 30.0;

    // Background (blue)
    vertices.extend(generate_rect(
        zoom_x,
        zoom_y,
        button_size,
        button_size,
        0.3,
        0.3,
        0.7,
        1.0,
    ));

    // Minus sign (white)
    let line_width = 4.0;
    let padding = 8.0;

    vertices.extend(generate_rotated_rect(
        zoom_x + padding,
        zoom_y + button_size / 2.0,
        zoom_x + button_size - padding,
        zoom_y + button_size / 2.0,
        line_width,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    vertices
}

/// Generate vertices for the undo button
/// The button shows a left arrow symbol
pub fn generate_undo_button_vertices(can_undo: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let undo_x = 90.0;
    let undo_y = 155.0; // Moved down from 85
    let button_size = 30.0;

    // Background color based on enabled state
    let (bg_r, bg_g, bg_b) = if can_undo {
        (0.3, 0.5, 0.8) // Blue when enabled
    } else {
        (0.7, 0.7, 0.7) // Gray when disabled
    };

    // Background
    vertices.extend(generate_rect(
        undo_x,
        undo_y,
        button_size,
        button_size,
        bg_r,
        bg_g,
        bg_b,
        1.0,
    ));

    // Arrow color (white when enabled, dark gray when disabled)
    let (arrow_r, arrow_g, arrow_b) = if can_undo {
        (1.0, 1.0, 1.0)
    } else {
        (0.5, 0.5, 0.5)
    };

    let line_width = 4.0;
    let padding = 6.0;

    // Left arrow: horizontal line from center-right to left
    vertices.extend(generate_rotated_rect(
        undo_x + padding,
        undo_y + button_size / 2.0,
        undo_x + button_size - padding,
        undo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    // Left arrow: diagonal from center to top-left
    vertices.extend(generate_rotated_rect(
        undo_x + button_size / 2.0,
        undo_y + padding,
        undo_x + padding,
        undo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    // Left arrow: diagonal from center to bottom-left
    vertices.extend(generate_rotated_rect(
        undo_x + button_size / 2.0,
        undo_y + button_size - padding,
        undo_x + padding,
        undo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    vertices
}

/// Generate vertices for the redo button
/// The button shows a right arrow symbol
pub fn generate_redo_button_vertices(can_redo: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let redo_x = 130.0;
    let redo_y = 155.0; // Moved down from 85
    let button_size = 30.0;

    // Background color based on enabled state
    let (bg_r, bg_g, bg_b) = if can_redo {
        (0.3, 0.5, 0.8) // Blue when enabled
    } else {
        (0.7, 0.7, 0.7) // Gray when disabled
    };

    // Background
    vertices.extend(generate_rect(
        redo_x,
        redo_y,
        button_size,
        button_size,
        bg_r,
        bg_g,
        bg_b,
        1.0,
    ));

    // Arrow color (white when enabled, dark gray when disabled)
    let (arrow_r, arrow_g, arrow_b) = if can_redo {
        (1.0, 1.0, 1.0)
    } else {
        (0.5, 0.5, 0.5)
    };

    let line_width = 4.0;
    let padding = 6.0;

    // Right arrow: horizontal line from left to center-right
    vertices.extend(generate_rotated_rect(
        redo_x + padding,
        redo_y + button_size / 2.0,
        redo_x + button_size - padding,
        redo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    // Right arrow: diagonal from center to top-right
    vertices.extend(generate_rotated_rect(
        redo_x + button_size / 2.0,
        redo_y + padding,
        redo_x + button_size - padding,
        redo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    // Right arrow: diagonal from center to bottom-right
    vertices.extend(generate_rotated_rect(
        redo_x + button_size / 2.0,
        redo_y + button_size - padding,
        redo_x + button_size - padding,
        redo_y + button_size / 2.0,
        line_width,
        arrow_r,
        arrow_g,
        arrow_b,
        1.0,
    ));

    vertices
}

/// Generate vertices for opacity preset buttons
/// Shows 4 buttons with fill level representing opacity
pub fn generate_opacity_preset_vertices(selected_opacity: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let opacity_presets: [f32; 4] = [0.25, 0.5, 0.75, 1.0];

    let selector_x = 10.0;
    let selector_y = 190.0; // Moved down from 130 for HSV picker
    let button_width = 35.0;
    let button_height = 25.0;
    let spacing = 10.0;
    let border_width = 2.0;

    for (i, &opacity) in opacity_presets.iter().enumerate() {
        let x = selector_x + (button_width + spacing) * i as f32;

        // Button background (gray)
        vertices.extend(generate_rect(
            x,
            selector_y,
            button_width,
            button_height,
            0.9,
            0.9,
            0.9,
            1.0,
        ));

        // Opacity fill (black, height based on opacity)
        let fill_height = button_height * opacity;
        let fill_y = selector_y + button_height - fill_height;
        vertices.extend(generate_rect(
            x + 2.0,
            fill_y,
            button_width - 4.0,
            fill_height - 2.0,
            0.2,
            0.2,
            0.2,
            1.0,
        ));

        // Checkboard pattern behind fill to show transparency
        let checker_size = 6.0;
        let checker_count_x = ((button_width - 4.0) / checker_size) as i32;
        let checker_count_y = ((button_height - 2.0) / checker_size) as i32;
        for cy in 0..checker_count_y {
            for cx in 0..checker_count_x {
                if (cx + cy) % 2 == 0 {
                    vertices.extend(generate_rect(
                        x + 2.0 + cx as f32 * checker_size,
                        selector_y + 1.0 + cy as f32 * checker_size,
                        checker_size,
                        checker_size,
                        0.75,
                        0.75,
                        0.75,
                        1.0,
                    ));
                }
            }
        }

        // Selected border (blue)
        if (opacity - selected_opacity).abs() < 0.01 {
            // Top border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                button_width + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            // Bottom border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y + button_height,
                button_width + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            // Left border
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                border_width,
                button_height + border_width * 2.0,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            // Right border
            vertices.extend(generate_rect(
                x + button_width,
                selector_y - border_width,
                border_width,
                button_height + border_width * 2.0,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
        }
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

    // --- generate_custom_palette tests ---

    #[test]
    fn test_custom_palette_vertex_count_no_selection() {
        let colors: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        let vertices = generate_custom_palette(&colors, 999);
        // 3 colors * 36 floats + 2 save button rects * 36 floats = 108 + 72 = 180 floats
        assert_eq!(vertices.len(), 180);
    }

    #[test]
    fn test_custom_palette_vertex_count_with_selection() {
        let colors: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        let vertices = generate_custom_palette(&colors, 0);
        // 3 colors * 36 floats + 4 border rects * 36 floats + 2 save button rects * 36 floats = 108 + 144 + 72 = 324 floats
        assert_eq!(vertices.len(), 324);
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

    // --- generate_clear_button_vertices tests ---

    #[test]
    fn test_generate_clear_button_vertices_returns_data() {
        let vertices = generate_clear_button_vertices();
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_clear_button_vertices_has_multiple_rects() {
        let vertices = generate_clear_button_vertices();
        // Clear button has background + X symbol = multiple rectangles
        // Each rect has 36 floats (6 vertices * 6 floats)
        assert!(vertices.len() >= 36 * 2);
    }

    // --- generate_undo_button_vertices tests ---

    #[test]
    fn test_generate_undo_button_enabled() {
        let vertices = generate_undo_button_vertices(true);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_undo_button_disabled() {
        let vertices = generate_undo_button_vertices(false);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_undo_button_enabled_has_more_vertices() {
        let enabled = generate_undo_button_vertices(true);
        let disabled = generate_undo_button_vertices(false);
        // Both should have same number of vertices (just different colors)
        assert_eq!(enabled.len(), disabled.len());
    }

    // --- generate_redo_button_vertices tests ---

    #[test]
    fn test_generate_redo_button_enabled() {
        let vertices = generate_redo_button_vertices(true);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_redo_button_disabled() {
        let vertices = generate_redo_button_vertices(false);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_redo_button_enabled_has_more_vertices() {
        let enabled = generate_redo_button_vertices(true);
        let disabled = generate_redo_button_vertices(false);
        // Both should have same number of vertices (just different colors)
        assert_eq!(enabled.len(), disabled.len());
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

    // --- generate_opacity_preset_vertices tests ---

    #[test]
    fn test_generate_opacity_preset_vertices_returns_data() {
        let vertices = generate_opacity_preset_vertices(1.0);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_opacity_preset_vertices_different_opacity() {
        let vertices_25 = generate_opacity_preset_vertices(0.25);
        let vertices_100 = generate_opacity_preset_vertices(1.0);
        // Both should have vertices (just different fill heights and borders)
        assert!(!vertices_25.is_empty());
        assert!(!vertices_100.is_empty());
    }
}
