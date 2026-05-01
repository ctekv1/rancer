//! Tests for Phase 5: Pixel-region selection

#[test]
fn pixel_selection_created_with_rect() {
    use crate::selection::PixelSelection;

    let selection = PixelSelection::new(10, 20, 30, 40);
    assert_eq!(selection.rect.x, 10);
    assert_eq!(selection.rect.y, 20);
    assert_eq!(selection.rect.width, 30);
    assert_eq!(selection.rect.height, 40);
}

#[test]
fn pixel_selection_has_empty_float_buffer_initially() {
    use crate::selection::PixelSelection;

    let selection = PixelSelection::new(0, 0, 10, 10);
    assert!(selection.float_buffer.is_none());
    assert!(selection.original_pixels.is_none());
}

#[test]
fn begin_selection_cuts_pixels_from_layer() {
    use crate::canvas::Canvas;
    use crate::selection::PixelSelection;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Paint some pixels
    {
        let raster = &mut canvas.layers_mut()[0].content;
        raster.image.set_pixel(10, 10, 255, 0, 0, 255);
        raster.image.set_pixel(11, 10, 255, 0, 0, 255);
    }

    let mut selection = PixelSelection::new(10, 10, 5, 5);
    selection.begin_selection(&mut canvas);

    // Pixels should be cut from the original layer
    let raster = &canvas.layers()[0].content;
    let (_r, _g, _b, a) = raster.image.get_pixel(10, 10).unwrap();
    assert_eq!(a, 0, "Pixel should be cut");
}

#[test]
fn move_selection_updates_offset() {
    use crate::selection::PixelSelection;

    let mut selection = PixelSelection::new(10, 10, 5, 5);
    assert_eq!(selection.offset_x, 0);
    assert_eq!(selection.offset_y, 0);

    selection.move_selection(5.0, 5.0);
    assert_eq!(selection.offset_x, 5);
    assert_eq!(selection.offset_y, 5);

    selection.move_selection(-2.0, 3.0);
    assert_eq!(selection.offset_x, 3);
    assert_eq!(selection.offset_y, 8);
}

#[test]
fn commit_selection_merges_float_buffer_back() {
    use crate::canvas::Canvas;
    use crate::selection::PixelSelection;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    // Draw some pixels
    {
        let raster = &mut canvas.layers_mut()[0].content;
        raster.image.set_pixel(10, 10, 255, 0, 0, 255);
    }

    let mut selection = PixelSelection::new(10, 10, 5, 5);
    selection.begin_selection(&mut canvas);
    
    // Move the selection
    selection.move_selection(10.0, 10.0);
    
    // Commit
    selection.commit_selection(&mut canvas);

    // Pixels should be at new location
    let raster = &canvas.layers()[0].content;
    // Original position should be transparent
    let old_pixel = raster.image.get_pixel(10, 10);
    assert_eq!(old_pixel, Some((0, 0, 0, 0)));

    // New position should have the pixel (offset by 10,10)
    let new_pixel = raster.image.get_pixel(20, 20);
    assert_eq!(new_pixel, Some((255, 0, 0, 255)));

    // Selection should be cleared
    assert!(selection.float_buffer.is_none());
}

#[test]
fn cancel_selection_restores_original_pixels() {
    use crate::canvas::Canvas;
    use crate::selection::PixelSelection;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    // Draw some pixels
    {
        let raster = &mut canvas.layers_mut()[0].content;
        raster.image.set_pixel(10, 10, 255, 0, 0, 255);
        raster.image.set_pixel(11, 10, 0, 255, 0, 255);
    }

    let mut selection = PixelSelection::new(10, 10, 5, 5);
    selection.begin_selection(&mut canvas);
    
    // Move the selection
    selection.move_selection(10.0, 10.0);
    
    // Cancel
    selection.cancel_selection(&mut canvas);

    // Original pixels should be restored
    let raster = &canvas.layers()[0].content;
    let pixel1 = raster.image.get_pixel(10, 10);
    assert_eq!(pixel1, Some((255, 0, 0, 255)));

    let pixel2 = raster.image.get_pixel(11, 10);
    assert_eq!(pixel2, Some((0, 255, 0, 255)));

    // Selection should be cleared
    assert!(selection.float_buffer.is_none());
}

#[test]
fn selection_tool_created_with_name() {
    use crate::tools::selection_tool::SelectionTool;
    use crate::tools::Tool;

    let tool = SelectionTool::new();
    assert_eq!(tool.name(), "Selection");
}

#[test]
fn selection_tool_press_begins_selection() {
    use crate::canvas::Canvas;
    use crate::tools::selection_tool::SelectionTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = SelectionTool::new();
    tool.on_press(10.0, 20.0, &mut canvas);

    // Tool should be in selecting state
    assert!(tool.is_selecting());
}

#[test]
fn selection_tool_drag_updates_selection_rect() {
    use crate::canvas::Canvas;
    use crate::tools::selection_tool::SelectionTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = SelectionTool::new();
    tool.on_press(10.0, 20.0, &mut canvas);
    tool.on_drag(30.0, 40.0, &mut canvas);

    // Selection rect should be updated
    assert!(tool.is_selecting());
    if let Some(selection) = &tool.selection {
        assert_eq!(selection.rect.x, 10);
        assert_eq!(selection.rect.y, 20);
        assert_eq!(selection.rect.width, 20);
        assert_eq!(selection.rect.height, 20);
    }
}

#[test]
fn selection_tool_release_completes_selection() {
    use crate::canvas::Canvas;
    use crate::tools::selection_tool::SelectionTool;
    use crate::tools::Tool;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);
    
    let mut tool = SelectionTool::new();
    tool.on_press(10.0, 20.0, &mut canvas);
    tool.on_drag(30.0, 40.0, &mut canvas);
    tool.on_release(30.0, 40.0, &mut canvas);

    // Tool should have completed selection
    assert!(!tool.is_selecting());
    assert!(tool.selection.is_some());
}
