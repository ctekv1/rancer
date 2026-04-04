//! Shared UI hit detection for Rancer
//!
//! Provides a single source of truth for UI element hit testing,
//! eliminating duplication between winit and GTK4 backends.
//! All coordinate constants match the vertex positions in geometry.rs.

use crate::canvas::{BrushType, Color};

/// UI element that was hit by a mouse click
#[derive(Debug, Clone, PartialEq)]
pub enum UiElement {
    /// Hue slider clicked at position (0-360)
    HueSlider(f32),
    /// Saturation slider clicked at position (0-100)
    SaturationSlider(f32),
    /// Value slider clicked at position (0-100)
    ValueSlider(f32),
    /// Custom color swatch clicked
    CustomColor(usize),
    /// Save current color button clicked
    SaveColor,
    /// Brush size button clicked
    BrushSize(f32),
    /// Eraser toggle button clicked
    Eraser,
    /// Clear canvas button clicked
    Clear,
    /// Undo button clicked
    Undo,
    /// Redo button clicked
    Redo,
    /// Export canvas button clicked
    Export,
    /// Zoom in button clicked
    ZoomIn,
    /// Zoom out button clicked
    ZoomOut,
    /// Opacity preset button clicked
    Opacity(f32),
    /// Brush type button clicked
    BrushType(BrushType),
    /// Layer row clicked (select layer)
    LayerRow(usize),
    /// Layer visibility toggle clicked
    LayerVisibility(usize),
    /// Add layer button clicked
    AddLayer,
    /// Delete layer button clicked
    DeleteLayer,
    /// Move active layer up button clicked
    MoveLayerUp,
    /// Move active layer down button clicked
    MoveLayerDown,
    /// Not on any UI element — canvas area
    Canvas,
}

/// Available brush sizes (must match canvas::BRUSH_SIZES and geometry.rs)
const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

/// Opacity preset values (must match canvas::OPACITY_PRESETS and geometry.rs)
const OPACITY_PRESETS: [f32; 4] = [0.25, 0.5, 0.75, 1.0];

/// Brush types (must match geometry/ui_elements.rs order)
const BRUSH_TYPES: [BrushType; 4] = [
    BrushType::Square,
    BrushType::Round,
    BrushType::Spray,
    BrushType::Calligraphy,
];

/// Perform hit testing at the given coordinates
///
/// Returns the UI element that was hit, or `UiElement::Canvas` if not on any UI element.
/// Coordinates are in logical pixels, matching the geometry.rs vertex positions.
pub fn hit_test(
    x: f32,
    y: f32,
    custom_colors: &[[u8; 3]],
    layer_count: usize,
    _active_layer: usize,
    window_width: f32,
) -> UiElement {
    // HSV Sliders area (y=5-80)
    if (5.0..=80.0).contains(&y) {
        let slider_x = 10.0;
        let slider_width = 200.0;

        if x >= slider_x && x <= slider_x + slider_width {
            if (5.0..=25.0).contains(&y) {
                let value = ((x - slider_x) / slider_width * 360.0).clamp(0.0, 360.0);
                return UiElement::HueSlider(value);
            } else if (30.0..=50.0).contains(&y) {
                let value = ((x - slider_x) / slider_width * 100.0).clamp(0.0, 100.0);
                return UiElement::SaturationSlider(value);
            } else if (55.0..=75.0).contains(&y) {
                let value = ((x - slider_x) / slider_width * 100.0).clamp(0.0, 100.0);
                return UiElement::ValueSlider(value);
            }
        }
    }

    // Custom palette area (y=90-110)
    if (90.0..=110.0).contains(&y) {
        let palette_x = 10.0;
        let color_width = 20.0;
        let spacing = 5.0;

        // Check save button first (after all existing colors)
        let save_x = palette_x + (color_width + spacing) * custom_colors.len() as f32;
        if x >= save_x && x <= save_x + color_width {
            return UiElement::SaveColor;
        }

        // Check custom color swatches
        for (i, _) in custom_colors.iter().enumerate() {
            let color_x = palette_x + (color_width + spacing) * i as f32;
            if x >= color_x && x <= color_x + color_width {
                return UiElement::CustomColor(i);
            }
        }
    }

    // Brush size selector (y=120-150)
    if (120.0..=150.0).contains(&y) {
        let selector_x = 10.0;
        let button_size = 30.0;
        let spacing = 10.0;

        for (i, &size) in BRUSH_SIZES.iter().enumerate() {
            let button_x = selector_x + (button_size + spacing) * i as f32;
            if x >= button_x && x <= button_x + button_size {
                return UiElement::BrushSize(size);
            }
        }
    }

    // Eraser button (y=155-185, x=10-40)
    if (155.0..=185.0).contains(&y) && (10.0..=40.0).contains(&x) {
        return UiElement::Eraser;
    }

    // Clear button (y=155-185, x=50-80)
    if (155.0..=185.0).contains(&y) && (50.0..=80.0).contains(&x) {
        return UiElement::Clear;
    }

    // Undo button (y=155-185, x=90-120)
    if (155.0..=185.0).contains(&y) && (90.0..=120.0).contains(&x) {
        return UiElement::Undo;
    }

    // Redo button (y=155-185, x=130-160)
    if (155.0..=185.0).contains(&y) && (130.0..=160.0).contains(&x) {
        return UiElement::Redo;
    }

    // Export button (y=155-185, x=170-200)
    if (155.0..=185.0).contains(&y) && (170.0..=200.0).contains(&x) {
        return UiElement::Export;
    }

    // Zoom in button (y=155-185, x=210-240)
    if (155.0..=185.0).contains(&y) && (210.0..=240.0).contains(&x) {
        return UiElement::ZoomIn;
    }

    // Zoom out button (y=155-185, x=250-280)
    if (155.0..=185.0).contains(&y) && (250.0..=280.0).contains(&x) {
        return UiElement::ZoomOut;
    }

    // Opacity presets (y=190-215)
    if (190.0..=215.0).contains(&y) {
        let selector_x = 10.0;
        let button_width = 35.0;
        let spacing = 10.0;

        for (i, &opacity) in OPACITY_PRESETS.iter().enumerate() {
            let bx = selector_x + (button_width + spacing) * i as f32;
            if x >= bx && x <= bx + button_width {
                return UiElement::Opacity(opacity);
            }
        }
    }

    // Brush type buttons (y=225-255)
    if (225.0..=255.0).contains(&y) {
        let selector_x = 10.0;
        let button_size = 30.0;
        let spacing = 10.0;

        for (i, &brush_type) in BRUSH_TYPES.iter().enumerate() {
            let bx = selector_x + (button_size + spacing) * i as f32;
            if x >= bx && x <= bx + button_size {
                return UiElement::BrushType(brush_type);
            }
        }
    }

    // Layer panel (right side)
    let panel_width = 150.0;
    let panel_x = window_width - panel_width - 10.0;
    let panel_top_y = 10.0;
    let row_height = 25.0;
    let max_visible_rows = 8;
    let panel_bottom_y = panel_top_y + (max_visible_rows as f32 * row_height) + 65.0;

    if x >= panel_x && x <= panel_x + panel_width && y >= panel_top_y && y <= panel_bottom_y {
        // Add layer button (below rows)
        let add_btn_y = panel_top_y + (max_visible_rows as f32 * row_height) + 5.0;
        let add_btn_height = 25.0;
        let add_btn_width = 60.0;
        if y >= add_btn_y && y <= add_btn_y + add_btn_height && x <= panel_x + add_btn_width {
            return UiElement::AddLayer;
        }

        // Delete layer button (next to add)
        let del_btn_x = panel_x + add_btn_width + 10.0;
        if y >= add_btn_y
            && y <= add_btn_y + add_btn_height
            && x >= del_btn_x
            && x <= del_btn_x + 60.0
        {
            return UiElement::DeleteLayer;
        }

        // Move up button (below add/delete)
        let move_btn_y = add_btn_y + add_btn_height + 5.0;
        let move_btn_width = 60.0;
        if y >= move_btn_y && y <= move_btn_y + add_btn_height && x <= panel_x + move_btn_width {
            return UiElement::MoveLayerUp;
        }

        // Move down button (next to move up)
        let move_down_x = panel_x + move_btn_width + 10.0;
        if y >= move_btn_y
            && y <= move_btn_y + add_btn_height
            && x >= move_down_x
            && x <= move_down_x + 60.0
        {
            return UiElement::MoveLayerDown;
        }

        // Layer rows
        let visible_count = layer_count.min(max_visible_rows);
        for i in 0..visible_count {
            let row_y = panel_top_y + (i as f32 * row_height);
            let row_bottom = row_y + row_height;
            if y >= row_y && y <= row_bottom {
                // Visibility toggle (left 20px of row)
                if x >= panel_x + 2.0 && x <= panel_x + 22.0 {
                    return UiElement::LayerVisibility(i);
                }
                // Rest of row (select layer)
                return UiElement::LayerRow(i);
            }
        }
    }

    UiElement::Canvas
}

/// Handle slider dragging during mouse motion
///
/// Returns the updated slider value if dragging a slider, or None otherwise.
pub fn slider_drag(
    x: f32,
    _y: f32,
    active_slider: Option<SliderType>,
) -> Option<(SliderType, f32)> {
    let slider = active_slider?;

    let slider_x = 10.0;
    let slider_width = 200.0;

    if x < slider_x || x > slider_x + slider_width {
        return None;
    }

    let value = match slider {
        SliderType::Hue => ((x - slider_x) / slider_width * 360.0).clamp(0.0, 360.0),
        SliderType::Saturation => ((x - slider_x) / slider_width * 100.0).clamp(0.0, 100.0),
        SliderType::Value => ((x - slider_x) / slider_width * 100.0).clamp(0.0, 100.0),
    };

    Some((slider, value))
}

/// Which slider is currently being dragged
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SliderType {
    Hue,
    Saturation,
    Value,
}

/// Determine which slider (if any) is being dragged based on click position
pub fn slider_type_at(x: f32, y: f32) -> Option<SliderType> {
    let slider_x = 10.0;
    let slider_width = 200.0;

    if x < slider_x || x > slider_x + slider_width {
        return None;
    }

    if (5.0..=25.0).contains(&y) {
        Some(SliderType::Hue)
    } else if (30.0..=50.0).contains(&y) {
        Some(SliderType::Saturation)
    } else if (55.0..=75.0).contains(&y) {
        Some(SliderType::Value)
    } else {
        None
    }
}

/// Convert HSV to RGB (shared helper for UI color preview)
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    crate::canvas::hsv_to_rgb(h, s, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_test_hue_slider() {
        let result = hit_test(110.0, 15.0, &[], 0, 0, 1280.0);
        match result {
            UiElement::HueSlider(val) => assert!((val - 180.0).abs() < 1.0),
            _ => panic!("Expected HueSlider, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_saturation_slider() {
        let result = hit_test(110.0, 40.0, &[], 0, 0, 1280.0);
        match result {
            UiElement::SaturationSlider(val) => assert!((val - 50.0).abs() < 1.0),
            _ => panic!("Expected SaturationSlider, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_value_slider() {
        let result = hit_test(110.0, 65.0, &[], 0, 0, 1280.0);
        match result {
            UiElement::ValueSlider(val) => assert!((val - 50.0).abs() < 1.0),
            _ => panic!("Expected ValueSlider, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_canvas_area() {
        let result = hit_test(500.0, 500.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Canvas);
    }

    #[test]
    fn test_hit_test_eraser_button() {
        let result = hit_test(25.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Eraser);
    }

    #[test]
    fn test_hit_test_clear_button() {
        let result = hit_test(65.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Clear);
    }

    #[test]
    fn test_hit_test_undo_button() {
        let result = hit_test(105.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Undo);
    }

    #[test]
    fn test_hit_test_redo_button() {
        let result = hit_test(145.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Redo);
    }

    #[test]
    fn test_hit_test_export_button() {
        let result = hit_test(185.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::Export);
    }

    #[test]
    fn test_hit_test_brush_size() {
        let result = hit_test(25.0, 135.0, &[], 0, 0, 1280.0);
        match result {
            UiElement::BrushSize(size) => assert_eq!(size, 3.0),
            _ => panic!("Expected BrushSize, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_opacity() {
        let result = hit_test(27.0, 200.0, &[], 0, 0, 1280.0);
        match result {
            UiElement::Opacity(val) => assert!((val - 0.25).abs() < 0.01),
            _ => panic!("Expected Opacity, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_brush_type_square() {
        let result = hit_test(25.0, 240.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::BrushType(BrushType::Square));
    }

    #[test]
    fn test_hit_test_brush_type_round() {
        let result = hit_test(65.0, 240.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::BrushType(BrushType::Round));
    }

    #[test]
    fn test_hit_test_brush_type_spray() {
        let result = hit_test(105.0, 240.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::BrushType(BrushType::Spray));
    }

    #[test]
    fn test_hit_test_brush_type_calligraphy() {
        let result = hit_test(145.0, 240.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::BrushType(BrushType::Calligraphy));
    }

    #[test]
    fn test_hit_test_custom_color() {
        let colors: [[u8; 3]; 3] = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        let result = hit_test(20.0, 100.0, &colors, 0, 0, 1280.0);
        match result {
            UiElement::CustomColor(idx) => assert_eq!(idx, 0),
            _ => panic!("Expected CustomColor, got {:?}", result),
        }
    }

    #[test]
    fn test_hit_test_save_color_button() {
        let colors: [[u8; 3]; 2] = [[255, 0, 0], [0, 255, 0]];
        let result = hit_test(70.0, 100.0, &colors, 0, 0, 1280.0);
        assert_eq!(result, UiElement::SaveColor);
    }

    #[test]
    fn test_slider_type_at() {
        assert_eq!(slider_type_at(110.0, 15.0), Some(SliderType::Hue));
        assert_eq!(slider_type_at(110.0, 40.0), Some(SliderType::Saturation));
        assert_eq!(slider_type_at(110.0, 65.0), Some(SliderType::Value));
        assert_eq!(slider_type_at(5.0, 15.0), None);
        assert_eq!(slider_type_at(110.0, 100.0), None);
    }

    #[test]
    fn test_slider_drag() {
        let result = slider_drag(110.0, 15.0, Some(SliderType::Hue));
        assert!(result.is_some());
        let (slider, value) = result.unwrap();
        assert_eq!(slider, SliderType::Hue);
        assert!((value - 180.0).abs() < 1.0);
    }

    #[test]
    fn test_slider_drag_no_active() {
        let result = slider_drag(110.0, 15.0, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_slider_drag_out_of_bounds() {
        let result = slider_drag(5.0, 15.0, Some(SliderType::Hue));
        assert!(result.is_none());
    }

    #[test]
    fn test_hit_test_zoom_in_button() {
        let result = hit_test(225.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::ZoomIn);
    }

    #[test]
    fn test_hit_test_zoom_out_button() {
        let result = hit_test(265.0, 170.0, &[], 0, 0, 1280.0);
        assert_eq!(result, UiElement::ZoomOut);
    }
}
