//! Square dab - filled rectangular brush tip

use crate::brush::DabMask;

pub struct SquareDab;

impl SquareDab {
    pub fn generate(size: u32) -> DabMask {
        DabMask::new(size)
    }
}