//! Brush engine module
//!
//! CPU-based dab painting system for raster canvas drawing.

pub mod dab;
pub mod engine;
pub mod round;

pub use dab::DabMask;
pub use engine::BrushEngine;
pub use round::RoundDab;

/// Available brush types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushType {
    Round,
    Square,
}
