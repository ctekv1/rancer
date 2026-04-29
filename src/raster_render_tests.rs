//! Tests for Phase 2: Raster layer rendering

#[test]
fn canvas_has_layers() {
    use crate::canvas::Canvas;
    let canvas = Canvas::new();
    assert!(canvas.layers().len() > 0);
}

#[test]
fn raster_layer_has_image_data() {
    use crate::canvas::{RasterLayer, RasterImage};
    let layer = RasterLayer::new(4, 4, 1.0);
    assert_eq!(layer.image.data.len(), 4 * 4 * 4);
}

#[test]
fn canvas_has_active_layer_index() {
    use crate::canvas::Canvas;
    let canvas = Canvas::new();
    let idx = canvas.active_layer();
    assert!(idx < canvas.layers().len());
}