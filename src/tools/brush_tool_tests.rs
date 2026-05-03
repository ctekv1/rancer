//! Tests for BrushTool with eraser mode
//! Following TDD: RED → GREEN cycle

use crate::tools::{BrushTool, Tool, BrushSettings, BrushType};
use crate::brush::{BrushEngine, RoundDab};
use crate::canvas::{Canvas, Color};

/// Test 1: BrushTool can be created with eraser mode OFF
#[test]
fn test_brush_tool_new_has_eraser_off() {
    let tool = BrushTool::new();
    assert!(!tool.is_eraser);
}

/// Test 2: BrushTool can toggle eraser mode
#[test]
fn test_brush_tool_set_eraser_mode() {
    let mut tool = BrushTool::new();
    tool.is_eraser = true;
    assert!(tool.is_eraser);
    
    tool.is_eraser = false;
    assert!(!tool.is_eraser);
}

/// Test 3: on_press with eraser mode = false paints normally
#[test]
fn test_brush_tool_paint_mode_on_press() {
    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    let mut tool = BrushTool::new();
    tool.is_eraser = false;
    
    // Paint at (50,50)
    tool.on_press(50.0, 50.0, &mut canvas);
    
    // Should have painted something
    let layer_idx = canvas.active_layer();
    let raster = &canvas.layers()[layer_idx].content;
    let pixel = raster.get_pixel(50, 50).unwrap();
    assert!(pixel.0 > 0 || pixel.1 > 0 || pixel.2 > 0);
}

/// Test 4: on_press with eraser mode = true ERASES (reduces alpha)
#[test]
fn test_brush_tool_eraser_mode_on_press() {
    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    // First, paint a pixel
    let mut paint_tool = BrushTool::new();
    paint_tool.is_eraser = false;
    paint_tool.on_press(50.0, 50.0, &mut canvas);
    
    // Verify pixel has color
    let layer_idx = canvas.active_layer();
    let raster = &canvas.layers()[layer_idx].content;
    let pixel_before = raster.get_pixel(50, 50).unwrap();
    assert!(pixel_before.3 > 0); // Has alpha
    
    // Now erase at same spot
    let mut eraser_tool = BrushTool::new();
    eraser_tool.is_eraser = true;
    eraser_tool.on_press(50.0, 50.0, &mut canvas);
    
    // Pixel should have reduced alpha (or be transparent)
    let raster = &canvas.layers()[layer_idx].content;
    let pixel_after = raster.get_pixel(50, 50).unwrap();
    assert!(pixel_after.3 < pixel_before.3); // Alpha reduced
}

/// Test 5: on_drag with eraser mode erases along path
#[test]
fn test_brush_tool_eraser_mode_on_drag() {
    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    // Paint a line first
    let mut paint_tool = BrushTool::new();
    paint_tool.is_eraser = false;
    paint_tool.on_press(50.0, 50.0, &mut canvas);
    paint_tool.on_drag(60.0, 50.0, &mut canvas);
    
    // Now erase over it
    let mut eraser_tool = BrushTool::new();
    eraser_tool.is_eraser = true;
    eraser_tool.on_press(50.0, 50.0, &mut canvas);
    eraser_tool.on_drag(60.0, 50.0, &mut canvas);
    
    // Pixels along path should have reduced alpha
    let layer_idx = canvas.active_layer();
    let raster = &canvas.layers()[layer_idx].content;
    let px1 = raster.get_pixel(50, 50).unwrap();
    let px2 = raster.get_pixel(55, 50).unwrap();
    assert!(px1.3 < 255); // Alpha reduced
    assert!(px2.3 < 255); // Alpha reduced
}

/// Test 6: eraser mode uses background color (canvas.background_color)
#[test]
fn test_eraser_uses_background_color() {
    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    canvas.background_color = Color { r: 255, g: 0, b: 0, a: 0 }; // Red background
    
    // Paint a pixel
    let mut tool = BrushTool::new();
    tool.is_eraser = false;
    tool.on_press(50.0, 50.0, &mut canvas);
    
    // Erase it
    tool.is_eraser = true;
    tool.on_press(50.0, 50.0, &mut canvas);
    
    // Pixel should now match background color (red, alpha=0)
    let layer_idx = canvas.active_layer();
    let raster = &canvas.layers()[layer_idx].content;
    let px = raster.get_pixel(50, 50).unwrap();
    assert_eq!(px.0, canvas.background_color.r); // Red
    assert_eq!(px.3, canvas.background_color.a); // Alpha=0
}

/// Test 7: Separate paint/eraser settings
#[test]
fn test_paint_and_eraser_have_separate_settings() {
    let mut tool = BrushTool::new();
    
    // Set paint settings
    tool.paint_settings = BrushSettings {
        size: 20,
        opacity: 1.0,
        color: Color { r: 255, g: 0, b: 0, a: 255 },
        brush_type: BrushType::Round,
    };
    
    // Set eraser settings
    tool.eraser_settings = crate::tools::brush_tool::EraserSettings {
        size: 40,
        opacity: 0.5,
    };
    
    // Verify they're separate
    assert_eq!(tool.paint_settings.size, 20);
    assert_eq!(tool.eraser_settings.size, 40);
    assert_ne!(tool.paint_settings.opacity, tool.eraser_settings.opacity);
}
