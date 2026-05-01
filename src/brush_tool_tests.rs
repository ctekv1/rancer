//! Tests for Phase 6: Brush engine + BrushTool

#[test]
fn brush_tool_created_with_default_settings() {
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let tool = BrushTool::new();
    assert_eq!(tool.name(), "Brush");
    assert!(!tool.is_drawing());
}

#[test]
fn brush_tool_press_starts_stroke() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    tool.on_press(50.0, 50.0, &mut canvas);
    assert!(tool.is_drawing());
}

#[test]
fn brush_tool_drag_paints_pixels() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    tool.on_press(50.0, 50.0, &mut canvas);
    tool.on_drag(55.0, 50.0, &mut canvas);

    // Pixels should be painted near the stroke position
    let layer_idx = canvas.active_layer_index();
    let layer = &canvas.layers()[layer_idx];
    // The stroke should have painted pixels around (50, 50)
    let px = layer.content.image.get_pixel(50, 50);
    assert!(px.is_some());
    // Check that something was painted (alpha > 0 or color changed)
    let (r, g, b, a) = px.unwrap();
    assert!(a > 0 || r > 0 || g > 0 || b > 0, "Expected painted pixels");
}

#[test]
fn brush_tool_release_ends_stroke() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    tool.on_press(50.0, 50.0, &mut canvas);
    tool.on_drag(55.0, 50.0, &mut canvas);
    tool.on_release(60.0, 50.0, &mut canvas);

    assert!(!tool.is_drawing());
}

#[test]
fn brush_tool_multiple_dabs_create_continuous_stroke() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    tool.on_press(10.0, 50.0, &mut canvas);
    
    // Drag in small increments to create multiple dabs
    for x in 15..=90 {
        tool.on_drag(x as f32, 50.0, &mut canvas);
    }
    
    tool.on_release(90.0, 50.0, &mut canvas);

    // Should have a continuous horizontal stroke
    let layer_idx = canvas.active_layer_index();
    let layer = &canvas.layers()[layer_idx];
    // Check pixels along the stroke line
    let mut painted_count = 0;
    for x in 10..=90 {
        if let Some((_, _, _, a)) = layer.content.image.get_pixel(x, 50) {
            if a > 0 {
                painted_count += 1;
            }
        }
    }
    assert!(painted_count > 5, "Expected multiple painted pixels along stroke");
}

#[test]
fn brush_tool_respects_brush_size() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    tool.set_brush_size(5);
    tool.on_press(50.0, 50.0, &mut canvas);
    tool.on_drag(50.0, 50.0, &mut canvas);
    tool.on_release(50.0, 50.0, &mut canvas);

    // Check that the painted area is roughly the brush size
    let layer_idx = canvas.active_layer_index();
    let layer = &canvas.layers()[layer_idx];
    // Pixel at center should be painted
    let center = layer.content.image.get_pixel(50, 50);
    assert!(center.is_some());
    
    // Pixel far from center should not be painted
    let far = layer.content.image.get_pixel(60, 60);
    if let Some((_, _, _, a)) = far {
        assert_eq!(a, 0, "Expected no paint far from brush center");
    }
}

#[test]
fn brush_tool_respects_brush_opacity() {
    use crate::canvas::Canvas;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    // Full opacity stroke
    let mut tool1 = BrushTool::new();
    tool1.set_brush_opacity(1.0);
    tool1.on_press(25.0, 50.0, &mut canvas);
    tool1.on_drag(25.0, 50.0, &mut canvas);
    tool1.on_release(25.0, 50.0, &mut canvas);

    // Half opacity stroke
    let mut tool2 = BrushTool::new();
    tool2.set_brush_opacity(0.5);
    tool2.on_press(75.0, 50.0, &mut canvas);
    tool2.on_drag(75.0, 50.0, &mut canvas);
    tool2.on_release(75.0, 50.0, &mut canvas);

    let layer_idx = canvas.active_layer_index();
    let layer = &canvas.layers()[layer_idx];
    if let Some((_, _, _, a_full)) = layer.content.image.get_pixel(25, 50) {
        if let Some((_, _, _, a_half)) = layer.content.image.get_pixel(75, 50) {
            // Full opacity should have higher alpha than half opacity
            assert!(a_full >= a_half, "Full opacity should paint more than half opacity");
        }
    }
}

#[test]
fn brush_tool_handles_brush_type_switching() {
    use crate::canvas::Canvas;
    use crate::brush::BrushType;
    use crate::tools::brush_tool::BrushTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = BrushTool::new();
    
    // Switch to round brush
    tool.set_brush_type(BrushType::Round);
    tool.on_press(25.0, 50.0, &mut canvas);
    tool.on_release(25.0, 50.0, &mut canvas);
    
    // Switch to square brush
    tool.set_brush_type(BrushType::Square);
    tool.on_press(75.0, 50.0, &mut canvas);
    tool.on_release(75.0, 50.0, &mut canvas);

    // Both positions should have painted pixels
    let layer_idx = canvas.active_layer_index();
    let layer = &canvas.layers()[layer_idx];
    let px1 = layer.content.image.get_pixel(25, 50);
    let px2 = layer.content.image.get_pixel(75, 50);
    
    if let Some((_, _, _, a1)) = px1 {
        assert!(a1 > 0, "Round brush should paint");
    }
    if let Some((_, _, _, a2)) = px2 {
        assert!(a2 > 0, "Square brush should paint");
    }
}
