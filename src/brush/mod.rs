//! Brush engine module
//!
//! CPU-based dab painting system for raster canvas drawing.

pub mod dab;
pub mod square;
pub mod round;
pub mod engine;

pub use dab::DabMask;
pub use square::SquareDab;
pub use round::RoundDab;
pub use engine::BrushEngine;

/// Available brush types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushType {
    Round,
    Square,
}