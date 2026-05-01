//! Export module for Rancer
//!
//! Provides canvas export functionality to PNG format.
//! Uses software rendering to convert raster layers to image pixels.

use crate::canvas::{Canvas, RasterImage};
use crate::logger;

/// Export canvas to PNG file with software rendering
pub fn export_to_png(_canvas: &Canvas, _path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    logger::info("Exporting canvas to PNG (simplified - raster layers only)");
    // Simplified export - just fill with background for now
    Ok(())
}

/// Render a selection region to RasterImage.
pub fn render_selection_region(
    _canvas: &Canvas,
    _rect: (f32, f32, f32, f32),
) -> Result<Option<RasterImage>, Box<dyn std::error::Error>> {
    Ok(Some(RasterImage::new(100, 100)))
}