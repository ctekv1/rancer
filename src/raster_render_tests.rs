//! Tests for Phase 2: Raster layer rendering

#[test]
fn canvas_has_layers() {
    use crate::canvas::Canvas;
    let canvas = Canvas::new();
    assert!(!canvas.layers().is_empty());
}

#[test]
fn raster_layer_has_image_data() {
    use crate::canvas::RasterImage;
    let layer = RasterImage::new(4, 4);
    assert_eq!(layer.data.len(), 4 * 4 * 4);
}

#[test]
fn canvas_has_active_layer_index() {
    use crate::canvas::Canvas;
    let canvas = Canvas::new();
    let idx = canvas.active_layer();
    assert!(idx < canvas.layers().len());
}

#[test]
fn layer_content_is_raster_layer() {
    use crate::canvas::Layer;
    let layer = Layer::new("test".to_string(), 10, 10, 1.0);
    let raster = &layer.content;
    assert_eq!(raster.width, 10);
    assert_eq!(raster.height, 10);
}
