//! Export module for Rancer
//!
//! Provides canvas export functionality to PNG format.

use crate::canvas::{Canvas, Rect, RasterImage};
use crate::logger;
use image::{ImageBuffer, Rgba};
use std::path::Path;

const MIN_EXPORT_SIZE: u32 = 100;
const MAX_EXPORT_SIZE: u32 = 4096;

pub fn export_to_png(_canvas: &Canvas, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    logger::info(&format!("Exporting canvas to {:?}", path));
    logger::info("Export not yet implemented for raster model");
    Ok(())
}

pub fn render_selection_region(
    _canvas: &Canvas,
    _rect: Rect,
) -> Result<RasterImage, Box<dyn std::error::Error>> {
    Ok(RasterImage::new(1, 1))
}