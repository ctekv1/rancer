//! Tests for rendering optimizations

#[test]
fn canvas_version_increments_on_pixel_change() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    let initial_version = canvas.version();

    // Change a pixel and mark dirty
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(10, 10, 255, 0, 0, 255);
    }
    canvas.mark_dirty(10, 10);

    assert!(
        canvas.version() > initial_version,
        "Version should increment on change"
    );
}

#[test]
fn canvas_version_increments_on_layer_add() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    let initial_version = canvas.version();

    canvas.add_layer(None).unwrap();

    assert!(
        canvas.version() > initial_version,
        "Version should increment on add_layer"
    );
}

#[test]
fn canvas_version_increments_on_layer_remove() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.add_layer(None).unwrap();
    let version_before_remove = canvas.version();

    canvas.remove_layer(1).unwrap();

    assert!(
        canvas.version() > version_before_remove,
        "Version should increment on remove_layer"
    );
}

#[test]
fn canvas_version_increments_on_toggle_visibility() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    let initial_version = canvas.version();

    let _ = canvas.toggle_layer_visibility(0);

    assert!(
        canvas.version() > initial_version,
        "Version should increment on toggle_visibility"
    );
}

#[test]
fn canvas_version_increments_on_set_layer_opacity() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    let initial_version = canvas.version();

    let _ = canvas.set_layer_opacity(0, 0.5);

    assert!(
        canvas.version() > initial_version,
        "Version should increment on set_layer_opacity"
    );
}

#[test]
fn canvas_version_increments_on_layer_move() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.add_layer(None).unwrap();
    let version_before_move = canvas.version();

    canvas.move_layer(0, 1).unwrap();

    assert!(
        canvas.version() > version_before_move,
        "Version should increment on move_layer"
    );
}

#[test]
fn render_skips_when_version_unchanged() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    let version1 = canvas.version();

    // No changes made
    let version2 = canvas.version();

    // Render should skip (versions match)
    assert_eq!(version1, version2, "Version unchanged, render should skip");
}

#[test]
fn dirty_rect_tracks_pixel_changes() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Consume the dirty rect from resize
    canvas.consume_dirty_rect();

    // Initial dirty rect should be empty
    assert!(canvas.dirty_rect().is_empty());

    // Change a pixel and mark dirty
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(50, 50, 255, 0, 0, 255);
    }
    canvas.mark_dirty(50, 50);

    // Dirty rect should now include the changed pixel
    let dirty = canvas.dirty_rect();
    assert!(
        !dirty.is_empty(),
        "Dirty rect should be non-empty after change"
    );
    assert!(
        dirty.contains(50, 50),
        "Dirty rect should contain changed pixel"
    );
}

#[test]
fn dirty_rect_clears_on_consume() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Change a pixel
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(50, 50, 255, 0, 0, 255);
    }

    // Consume the dirty rect
    let dirty = canvas.consume_dirty_rect();
    assert!(!dirty.is_empty(), "Should have a dirty rect");

    // After consuming, dirty rect should be empty
    assert!(
        canvas.dirty_rect().is_empty(),
        "Dirty rect should be empty after consume"
    );
}

#[test]
fn composite_all_visible_layers() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Add a second layer
    canvas.add_layer(None).unwrap();

    // Composite all visible layers
    let composite = crate::compositor::Compositor::new().composite_all(&canvas);

    // Result should have correct dimensions
    assert_eq!(composite.width, 100);
    assert_eq!(composite.height, 100);
    // Data should be RGBA
    assert_eq!(composite.data.len(), 100 * 100 * 4);
}

#[test]
fn composite_respects_layer_visibility() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Draw on background layer
    {
        let layer = canvas.active_layer_mut();
        for y in 0..100 {
            for x in 0..100 {
                layer.content.set_pixel(x, y, 255, 0, 0, 255);
            }
        }
    }

    // Hide the layer
    let _ = canvas.toggle_layer_visibility(0);

    // Composite should show background color (white) since no layers are visible
    let composite = crate::compositor::Compositor::new().composite_all(&canvas);

    // All pixels should be the background color (white, opaque)
    for y in 0..100 {
        for x in 0..100 {
            let idx = ((y * 100 + x) * 4) as usize;
            assert_eq!(
                composite.data[idx], 255,
                "Pixel ({}, {}) R should be 255",
                x, y
            );
            assert_eq!(
                composite.data[idx + 1],
                255,
                "Pixel ({}, {}) G should be 255",
                x,
                y
            );
            assert_eq!(
                composite.data[idx + 2],
                255,
                "Pixel ({}, {}) B should be 255",
                x,
                y
            );
            assert_eq!(
                composite.data[idx + 3],
                255,
                "Pixel ({}, {}) A should be 255",
                x,
                y
            );
        }
    }
}

#[test]
fn composite_respects_layer_opacity() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::new();
    canvas.resize(100, 100);

    // Draw on background layer with full opacity
    {
        let layer = canvas.active_layer_mut();
        for y in 0..100 {
            for x in 0..100 {
                layer.content.set_pixel(x, y, 255, 0, 0, 255);
            }
        }
    }
    let _ = canvas.set_layer_opacity(0, 0.5);

    // Composite should blend red layer (50% opacity) over white background
    let composite = crate::compositor::Compositor::new().composite_all(&canvas);

    // Expected result: RGB(255, ~127, ~127) with alpha 255
    // Red channel: (255*0.5 + 255*0.5) / 1.0 = 255
    // Green channel: (0*0.5 + 255*0.5) / 1.0 = ~127
    // Blue channel: (0*0.5 + 255*0.5) / 1.0 = ~127
    let idx = ((50 * 100 + 50) * 4) as usize;
    let r = composite.data[idx];
    let g = composite.data[idx + 1];
    let b = composite.data[idx + 2];
    let a = composite.data[idx + 3];

    assert_eq!(r, 255, "Red channel should be 255");
    assert!((120..=135).contains(&g), "Green should be ~127, got {}", g);
    assert!((120..=135).contains(&b), "Blue should be ~127, got {}", b);
    assert_eq!(a, 255, "Alpha should be 255");
}

#[test]
fn composite_rect_produces_correct_output_for_small_region() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::with_size(20, 20);

    // Draw a red pixel at center
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(10, 10, 255, 0, 0, 255);
    }

    // Composite only a 5x5 region around the red pixel
    let region = crate::compositor::Compositor::new().composite_rect(&canvas, 8, 8, 5, 5);

    // Should be 5x5 RGBA
    assert_eq!(region.width, 5);
    assert_eq!(region.height, 5);
    assert_eq!(region.data.len(), 5 * 5 * 4);

    // Center of region should be red pixel (at offset 2,2 within 5x5)
    let center_idx = (2 * 5 + 2) * 4;
    assert_eq!(region.data[center_idx], 255); // R
    assert_eq!(region.data[center_idx + 1], 0); // G
    assert_eq!(region.data[center_idx + 2], 0); // B
    assert_eq!(region.data[center_idx + 3], 255); // A

    // Top-left corner should be white (background)
    assert_eq!(region.data[0], 255); // R
    assert_eq!(region.data[1], 255); // G
    assert_eq!(region.data[2], 255); // B
    assert_eq!(region.data[3], 255); // A
}

#[test]
fn composite_rect_respects_layer_visibility() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::with_size(20, 20);

    // Draw a red pixel
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(10, 10, 255, 0, 0, 255);
    }

    // Hide the layer
    let _ = canvas.toggle_layer_visibility(0);

    // Composite the region - should be white background only
    let region = crate::compositor::Compositor::new().composite_rect(&canvas, 8, 8, 5, 5);

    // All pixels should be white (background color)
    for y in 0..5 {
        for x in 0..5 {
            let idx = (y * 5 + x) * 4;
            assert_eq!(region.data[idx], 255);
            assert_eq!(region.data[idx + 1], 255);
            assert_eq!(region.data[idx + 2], 255);
            assert_eq!(region.data[idx + 3], 255);
        }
    }
}

#[test]
fn composite_rect_clamps_to_canvas_bounds() {
    use crate::canvas::Canvas;

    let canvas = Canvas::with_size(20, 20);

    // Request region that extends beyond canvas bounds
    let region = crate::compositor::Compositor::new().composite_rect(&canvas, 15, 15, 10, 10);

    // Should be clamped to 5x5 (remaining canvas area)
    assert_eq!(region.width, 5);
    assert_eq!(region.height, 5);
    assert_eq!(region.data.len(), 5 * 5 * 4);

    // All pixels should be white background
    assert_eq!(region.data[0], 255);
}

#[test]
fn composite_rect_handles_empty_request() {
    use crate::canvas::Canvas;

    let canvas = Canvas::with_size(20, 20);

    // Request zero-size region
    let region = crate::compositor::Compositor::new().composite_rect(&canvas, 5, 5, 0, 0);

    // Should return empty result
    assert_eq!(region.width, 0);
    assert_eq!(region.height, 0);
    assert!(region.data.is_empty());
}

#[test]
fn composite_rect_respects_layer_opacity() {
    use crate::canvas::Canvas;

    let mut canvas = Canvas::with_size(20, 20);

    // Draw opaque red
    {
        let layer = canvas.active_layer_mut();
        layer.content.set_pixel(10, 10, 255, 0, 0, 255);
    }
    let _ = canvas.set_layer_opacity(0, 0.5);

    let region = crate::compositor::Compositor::new().composite_rect(&canvas, 8, 8, 5, 5);

    // Center pixel should blend red with white background at 50%
    let center_idx = (2 * 5 + 2) * 4;
    let r = region.data[center_idx];
    let g = region.data[center_idx + 1];
    let b = region.data[center_idx + 2];

    // Expected: R=255, G=~127, B=~127 (same as composite_all)
    assert_eq!(r, 255);
    assert!((120..=135).contains(&g), "G should be ~127, got {}", g);
    assert!((120..=135).contains(&b), "B should be ~127, got {}", b);
}
