//! UI element vertex generation for rendering
//!
//! Generates vertex data for all UI elements: HSV sliders, color palette,
//! brush size selector, tool buttons, opacity presets, and layer panel.

use super::{generate_rect, generate_rotated_rect, hsv_to_rgb_f32};

/// Generate HSV slider vertices (H, S, V sliders)
/// h: 0-360, s: 0-100, v: 0-100
pub fn generate_hsv_sliders(h: f32, s: f32, v: f32) -> Vec<f32> {
    let mut vertices = Vec::new();

    let slider_x = 10.0;
    let slider_width = 200.0;
    let slider_height = 20.0;

    // Hue slider (y=5)
    let hue_y = 5.0;
    let hue_colors = [
        (1.0, 0.0, 0.0),
        (1.0, 1.0, 0.0),
        (0.0, 1.0, 0.0),
        (0.0, 1.0, 1.0),
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 1.0),
        (1.0, 0.0, 0.0),
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

/// Generate custom palette vertices (saved colors as swatches)
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
    let selector_y = 120.0;
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
    let eraser_y = 155.0;
    let button_size = 30.0;
    let border_width = 2.0;

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

    let top_height = body_height * 0.35;
    let top_width = body_width - 4.0;
    let top_x = eraser_x + body_padding + 2.0;
    let top_y = eraser_y + body_padding;
    vertices.extend(generate_rect(
        top_x, top_y, top_width, top_height, 0.95, 0.92, 0.85, 1.0,
    ));

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

/// Generate vertices for the brush type selector buttons
/// 4 buttons at y=225: Square, Round, Spray, Calligraphy
pub fn generate_brush_type_vertices(selected_type: crate::canvas::BrushType) -> Vec<f32> {
    use crate::canvas::BrushType;

    let mut vertices = Vec::new();

    let selector_x = 10.0;
    let selector_y = 225.0;
    let button_size = 30.0;
    let spacing = 10.0;
    let border_width = 2.0;

    let brush_types = [
        BrushType::Square,
        BrushType::Round,
        BrushType::Spray,
        BrushType::Calligraphy,
    ];

    for (i, brush_type) in brush_types.iter().enumerate() {
        let x = selector_x + (button_size + spacing) * i as f32;

        // Button background
        vertices.extend(generate_rect(
            x,
            selector_y,
            button_size,
            button_size,
            0.85,
            0.85,
            0.85,
            1.0,
        ));

        match brush_type {
            BrushType::Square => {
                // Solid square icon
                let icon_size = 14.0;
                let ix = x + (button_size - icon_size) / 2.0;
                let iy = selector_y + (button_size - icon_size) / 2.0;
                vertices.extend(generate_rect(
                    ix, iy, icon_size, icon_size, 0.0, 0.0, 0.0, 1.0,
                ));
            }
            BrushType::Round => {
                // Circle icon: ring of dots forming a circle outline
                let center_x = x + button_size / 2.0;
                let center_y = selector_y + button_size / 2.0;
                let dot = 3.0;
                let radius = 7.0;
                let angles: [f32; 8] = [0.0, 0.785, 1.571, 2.356, 3.142, 3.927, 4.712, 5.498];
                for angle in angles {
                    let dx = angle.cos() * radius;
                    let dy = angle.sin() * radius;
                    vertices.extend(generate_rect(
                        center_x + dx - dot / 2.0,
                        center_y + dy - dot / 2.0,
                        dot,
                        dot,
                        0.0,
                        0.0,
                        0.0,
                        1.0,
                    ));
                }
            }
            BrushType::Spray => {
                // Scattered dots — sparse and spread out
                let center_x = x + button_size / 2.0;
                let center_y = selector_y + button_size / 2.0;
                let dot = 2.0;
                let positions = [
                    (-9.0, -6.0),
                    (8.0, -8.0),
                    (-7.0, 7.0),
                    (10.0, 3.0),
                    (-10.0, 1.0),
                    (3.0, 9.0),
                    (6.0, -1.0),
                ];
                for (dx, dy) in positions {
                    vertices.extend(generate_rect(
                        center_x + dx - dot / 2.0,
                        center_y + dy - dot / 2.0,
                        dot,
                        dot,
                        0.0,
                        0.0,
                        0.0,
                        1.0,
                    ));
                }
            }
            BrushType::Calligraphy => {
                // Diagonal rect at 45°
                let center_x = x + button_size / 2.0;
                let center_y = selector_y + button_size / 2.0;
                vertices.extend(generate_rotated_rect(
                    center_x - 7.0,
                    center_y + 7.0,
                    center_x + 7.0,
                    center_y - 7.0,
                    5.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ));
            }
        }

        // Blue selection border
        if *brush_type == selected_type {
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                button_size + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                selector_y + button_size,
                button_size + border_width * 2.0,
                border_width,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x - border_width,
                selector_y - border_width,
                border_width,
                button_size + border_width * 2.0,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
            vertices.extend(generate_rect(
                x + button_size,
                selector_y - border_width,
                border_width,
                button_size + border_width * 2.0,
                0.0,
                0.5,
                1.0,
                1.0,
            ));
        }
    }

    vertices
}

/// Generate vertices for the selection tool toggle button
pub fn generate_selection_tool_button(is_active: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let btn_x = 10.0;
    let btn_y = 265.0;
    let button_size = 30.0;
    let border_width = 2.0;

    // Button background
    vertices.extend(generate_rect(
        btn_x,
        btn_y,
        button_size,
        button_size,
        0.85,
        0.85,
        0.85,
        1.0,
    ));

    // Dashed rectangle icon (smaller, centered)
    let icon_x = btn_x + 7.0;
    let icon_y = btn_y + 7.0;
    let icon_w = 16.0;
    let icon_h = 16.0;
    let dash_len: f32 = 3.0;
    let gap_len: f32 = 2.0;

    // Top edge
    let mut x = icon_x;
    while x < icon_x + icon_w {
        let seg_w = dash_len.min(icon_x + icon_w - x);
        vertices.extend(generate_rect(x, icon_y, seg_w, 2.0, 0.0, 0.0, 0.0, 1.0));
        x += dash_len + gap_len;
    }
    // Bottom edge
    x = icon_x;
    while x < icon_x + icon_w {
        let seg_w = dash_len.min(icon_x + icon_w - x);
        vertices.extend(generate_rect(
            x,
            icon_y + icon_h - 2.0,
            seg_w,
            2.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ));
        x += dash_len + gap_len;
    }
    // Left edge
    let mut y = icon_y;
    while y < icon_y + icon_h {
        let seg_h = dash_len.min(icon_y + icon_h - y);
        vertices.extend(generate_rect(icon_x, y, 2.0, seg_h, 0.0, 0.0, 0.0, 1.0));
        y += dash_len + gap_len;
    }
    // Right edge
    y = icon_y;
    while y < icon_y + icon_h {
        let seg_h = dash_len.min(icon_y + icon_h - y);
        vertices.extend(generate_rect(
            icon_x + icon_w - 2.0,
            y,
            2.0,
            seg_h,
            0.0,
            0.0,
            0.0,
            1.0,
        ));
        y += dash_len + gap_len;
    }

    // Blue selection border when active
    if is_active {
        vertices.extend(generate_rect(
            btn_x - border_width,
            btn_y - border_width,
            button_size + border_width * 2.0,
            border_width,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            btn_x - border_width,
            btn_y + button_size,
            button_size + border_width * 2.0,
            border_width,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            btn_x - border_width,
            btn_y - border_width,
            border_width,
            button_size + border_width * 2.0,
            0.0,
            0.5,
            1.0,
            1.0,
        ));
        vertices.extend(generate_rect(
            btn_x + button_size,
            btn_y - border_width,
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

/// Generate vertices for a dashed selection rectangle with marching ants animation.
/// `time_offset` (0.0..dash_len+gap_len) shifts the dashes for animation.
pub fn generate_selection_rect_vertices(rect: crate::canvas::Rect, time_offset: f32) -> Vec<f32> {
    let mut vertices = Vec::new();

    let (rx, ry, rw, rh) = rect.normalized();
    let dash_len: f32 = 8.0;
    let gap_len: f32 = 6.0;
    let period = dash_len + gap_len;
    let line_w: f32 = 3.0;
    let speed: f32 = 30.0;

    // Use modulo to create wrapping animation
    let offset = (time_offset * speed) % period;

    // Top edge
    let mut x = rx + offset;
    while x < rx + rw {
        let seg_w = dash_len.min(rx + rw - x);
        if seg_w > 0.1 {
            vertices.extend(generate_rect(x, ry, seg_w, line_w, 0.0, 0.0, 0.0, 0.8));
        }
        x += period;
    }
    // Bottom edge
    x = rx + offset;
    while x < rx + rw {
        let seg_w = dash_len.min(rx + rw - x);
        if seg_w > 0.1 {
            vertices.extend(generate_rect(
                x,
                ry + rh - line_w,
                seg_w,
                line_w,
                0.0,
                0.0,
                0.0,
                0.8,
            ));
        }
        x += period;
    }
    // Left edge
    let mut y = ry + offset;
    while y < ry + rh {
        let seg_h = dash_len.min(ry + rh - y);
        if seg_h > 0.1 {
            vertices.extend(generate_rect(rx, y, line_w, seg_h, 0.0, 0.0, 0.0, 0.8));
        }
        y += period;
    }
    // Right edge
    y = ry + offset;
    while y < ry + rh {
        let seg_h = dash_len.min(ry + rh - y);
        if seg_h > 0.1 {
            vertices.extend(generate_rect(
                rx + rw - line_w,
                y,
                line_w,
                seg_h,
                0.0,
                0.0,
                0.0,
                0.8,
            ));
        }
        y += period;
    }

    vertices
}

/// Generate vertices for the clear canvas button
pub fn generate_clear_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let clear_x = 50.0;
    let clear_y = 155.0;
    let button_size = 30.0;

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

    let line_width = 4.0;
    let padding = 6.0;

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
pub fn generate_export_button_vertices() -> Vec<f32> {
    let mut vertices = Vec::new();
    let export_x = 170.0;
    let export_y = 155.0;
    let button_size = 30.0;

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

    let line_width = 4.0;
    let padding = 6.0;

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
pub fn generate_undo_button_vertices(can_undo: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let undo_x = 90.0;
    let undo_y = 155.0;
    let button_size = 30.0;

    let (bg_r, bg_g, bg_b) = if can_undo {
        (0.3, 0.5, 0.8)
    } else {
        (0.7, 0.7, 0.7)
    };
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

    let (arrow_r, arrow_g, arrow_b) = if can_undo {
        (1.0, 1.0, 1.0)
    } else {
        (0.5, 0.5, 0.5)
    };
    let line_width = 4.0;
    let padding = 6.0;

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
pub fn generate_redo_button_vertices(can_redo: bool) -> Vec<f32> {
    let mut vertices = Vec::new();
    let redo_x = 130.0;
    let redo_y = 155.0;
    let button_size = 30.0;

    let (bg_r, bg_g, bg_b) = if can_redo {
        (0.3, 0.5, 0.8)
    } else {
        (0.7, 0.7, 0.7)
    };
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

    let (arrow_r, arrow_g, arrow_b) = if can_redo {
        (1.0, 1.0, 1.0)
    } else {
        (0.5, 0.5, 0.5)
    };
    let line_width = 4.0;
    let padding = 6.0;

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
pub fn generate_opacity_preset_vertices(selected_opacity: f32) -> Vec<f32> {
    let mut vertices = Vec::new();
    let opacity_presets: [f32; 4] = [0.25, 0.5, 0.75, 1.0];

    let selector_x = 10.0;
    let selector_y = 190.0;
    let button_width = 35.0;
    let button_height = 25.0;
    let spacing = 10.0;
    let border_width = 2.0;

    for (i, &opacity) in opacity_presets.iter().enumerate() {
        let x = selector_x + (button_width + spacing) * i as f32;

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

        if (opacity - selected_opacity).abs() < 0.01 {
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

/// Generate vertices for the layer panel UI on the right side of the window
pub fn generate_layer_panel_vertices(
    layers: &[(String, bool, f32, bool)],
    active_layer: usize,
    window_width: f32,
) -> Vec<f32> {
    let mut vertices = Vec::new();
    let panel_width = 150.0;
    let panel_x = window_width - panel_width - 10.0;
    let panel_top_y = 10.0;
    let row_height = 25.0;
    let max_visible_rows = 8;

    let panel_height = (max_visible_rows as f32 * row_height) + 60.0;
    vertices.extend(generate_rect(
        panel_x - 5.0,
        panel_top_y - 5.0,
        panel_width + 10.0,
        panel_height + 10.0,
        0.15,
        0.15,
        0.15,
        0.9,
    ));

    let border = 1.0;
    vertices.extend(generate_rect(
        panel_x - 5.0 - border,
        panel_top_y - 5.0 - border,
        panel_width + 10.0 + border * 2.0,
        border,
        0.4,
        0.4,
        0.4,
        1.0,
    ));
    vertices.extend(generate_rect(
        panel_x - 5.0 - border,
        panel_top_y + panel_height + 5.0,
        panel_width + 10.0 + border * 2.0,
        border,
        0.4,
        0.4,
        0.4,
        1.0,
    ));
    vertices.extend(generate_rect(
        panel_x - 5.0 - border,
        panel_top_y - 5.0,
        border,
        panel_height + 10.0,
        0.4,
        0.4,
        0.4,
        1.0,
    ));
    vertices.extend(generate_rect(
        panel_x + panel_width + 5.0,
        panel_top_y - 5.0,
        border,
        panel_height + 10.0,
        0.4,
        0.4,
        0.4,
        1.0,
    ));

    let visible_count = layers.len().min(max_visible_rows);
    for (i, (_name, visible, opacity, locked)) in layers.iter().enumerate().take(visible_count) {
        let row_y = panel_top_y + (i as f32 * row_height);
        let is_active = i == active_layer;

        let bg_alpha = if is_active { 0.3 } else { 0.0 };
        if bg_alpha > 0.0 {
            vertices.extend(generate_rect(
                panel_x,
                row_y,
                panel_width,
                row_height - 2.0,
                0.3,
                0.3,
                0.7,
                bg_alpha,
            ));
        }

        let vis_x = panel_x + 3.0;
        let vis_y = row_y + 5.0;
        let vis_size = 15.0;
        if *visible {
            vertices.extend(generate_rect(
                vis_x, vis_y, vis_size, vis_size, 0.2, 0.8, 0.2, 1.0,
            ));
        } else {
            vertices.extend(generate_rect(
                vis_x, vis_y, vis_size, vis_size, 0.8, 0.2, 0.2, 0.6,
            ));
        }

        if *locked {
            vertices.extend(generate_rect(
                vis_x + vis_size + 2.0,
                vis_y + 2.0,
                11.0,
                11.0,
                0.9,
                0.8,
                0.0,
                1.0,
            ));
        }

        let name_x = panel_x + 25.0;
        let name_y = row_y + 10.0;
        let name_w = panel_width - 30.0;
        let name_h = 5.0;
        let _opacity = *opacity;
        vertices.extend(generate_rect(
            name_x, name_y, name_w, name_h, 0.9, 0.9, 0.9, 1.0,
        ));

        let op_bar_x = panel_x + 25.0;
        let op_bar_y = row_y + 17.0;
        let op_bar_w = (panel_width - 30.0) * opacity;
        vertices.extend(generate_rect(
            op_bar_x,
            op_bar_y,
            op_bar_w.max(1.0),
            3.0,
            0.5,
            0.5,
            0.5,
            0.8,
        ));
    }

    let btn_y = panel_top_y + (max_visible_rows as f32 * row_height) + 5.0;
    let btn_width = 60.0;
    let btn_height = 25.0;
    vertices.extend(generate_rect(
        panel_x + 5.0,
        btn_y,
        btn_width,
        btn_height,
        0.2,
        0.6,
        0.2,
        1.0,
    ));
    let plus_x = panel_x + 5.0 + btn_width / 2.0;
    let plus_y = btn_y + btn_height / 2.0;
    let line_w = 3.0;
    vertices.extend(generate_rotated_rect(
        panel_x + 15.0,
        plus_y,
        panel_x + 5.0 + btn_width - 15.0,
        plus_y,
        line_w,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rotated_rect(
        plus_x,
        btn_y + 6.0,
        plus_x,
        btn_y + btn_height - 6.0,
        line_w,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    let del_x = panel_x + btn_width + 15.0;
    vertices.extend(generate_rect(
        del_x, btn_y, btn_width, btn_height, 0.6, 0.2, 0.2, 1.0,
    ));
    vertices.extend(generate_rotated_rect(
        del_x + 10.0,
        btn_y + btn_height / 2.0,
        del_x + btn_width - 10.0,
        btn_y + btn_height / 2.0,
        line_w,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    let move_btn_y = btn_y + btn_height + 5.0;
    vertices.extend(generate_rect(
        panel_x + 5.0,
        move_btn_y,
        btn_width,
        btn_height,
        0.4,
        0.4,
        0.7,
        1.0,
    ));
    let up_cx = panel_x + 5.0 + btn_width / 2.0;
    let up_cy = move_btn_y + btn_height / 2.0;
    vertices.extend(generate_rect(
        up_cx - 8.0,
        up_cy + 3.0,
        16.0,
        2.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rotated_rect(
        up_cx - 6.0,
        up_cy + 2.0,
        up_cx,
        up_cy - 4.0,
        1.5,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rotated_rect(
        up_cx + 6.0,
        up_cy + 2.0,
        up_cx,
        up_cy - 4.0,
        1.5,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    let move_down_x = panel_x + btn_width + 15.0;
    vertices.extend(generate_rect(
        move_down_x,
        move_btn_y,
        btn_width,
        btn_height,
        0.4,
        0.4,
        0.7,
        1.0,
    ));
    let down_cx = move_down_x + btn_width / 2.0;
    let down_cy = move_btn_y + btn_height / 2.0;
    vertices.extend(generate_rect(
        down_cx - 8.0,
        down_cy - 5.0,
        16.0,
        2.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rotated_rect(
        down_cx - 6.0,
        down_cy - 2.0,
        down_cx,
        down_cy + 4.0,
        1.5,
        1.0,
        1.0,
        1.0,
        1.0,
    ));
    vertices.extend(generate_rotated_rect(
        down_cx + 6.0,
        down_cy - 2.0,
        down_cx,
        down_cy + 4.0,
        1.5,
        1.0,
        1.0,
        1.0,
        1.0,
    ));

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_palette_vertex_count_no_selection() {
        let colors: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        let vertices = generate_custom_palette(&colors, 999);
        assert_eq!(vertices.len(), 180);
    }

    #[test]
    fn test_custom_palette_vertex_count_with_selection() {
        let colors: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        let vertices = generate_custom_palette(&colors, 0);
        assert_eq!(vertices.len(), 324);
    }

    #[test]
    fn test_brush_size_vertex_count() {
        let vertices = generate_brush_size_vertices(3.0);
        assert_eq!(vertices.len(), 504);
    }

    #[test]
    fn test_brush_size_different_selections() {
        let v3 = generate_brush_size_vertices(3.0);
        let v50 = generate_brush_size_vertices(50.0);
        assert_eq!(v3.len(), v50.len());
        assert_ne!(v3, v50);
    }

    #[test]
    fn test_brush_size_invalid_selection() {
        let vertices = generate_brush_size_vertices(100.0);
        assert_eq!(vertices.len(), 360);
    }

    #[test]
    fn test_eraser_button_inactive() {
        let vertices = generate_eraser_button_vertices(false);
        assert_eq!(vertices.len(), 144);
    }

    #[test]
    fn test_eraser_button_active() {
        let vertices = generate_eraser_button_vertices(true);
        assert_eq!(vertices.len(), 288);
    }

    #[test]
    fn test_eraser_button_active_larger_than_inactive() {
        let inactive = generate_eraser_button_vertices(false);
        let active = generate_eraser_button_vertices(true);
        assert!(active.len() > inactive.len());
    }

    #[test]
    fn test_generate_clear_button_vertices_returns_data() {
        let vertices = generate_clear_button_vertices();
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_clear_button_vertices_has_multiple_rects() {
        let vertices = generate_clear_button_vertices();
        assert!(vertices.len() >= 36 * 2);
    }

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
        assert_eq!(enabled.len(), disabled.len());
    }

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
        assert_eq!(enabled.len(), disabled.len());
    }

    #[test]
    fn test_generate_opacity_preset_vertices_returns_data() {
        let vertices = generate_opacity_preset_vertices(1.0);
        assert!(!vertices.is_empty());
    }

    #[test]
    fn test_generate_opacity_preset_vertices_different_opacity() {
        let vertices_25 = generate_opacity_preset_vertices(0.25);
        let vertices_100 = generate_opacity_preset_vertices(1.0);
        assert!(!vertices_25.is_empty());
        assert!(!vertices_100.is_empty());
    }
}
