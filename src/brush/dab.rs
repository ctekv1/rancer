//! DabMask: 2D pixel mask for brush tips
//!
//! Each DabMask is a small RGBA bitmap representing the brush tip shape.
//! Alpha values determine how much each pixel is affected when stamping.

#[derive(Debug, Clone, PartialEq)]
pub struct DabMask {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl DabMask {
    pub fn new(size: u32) -> Self {
        let size = size.max(1);
        Self {
            width: size,
            height: size,
            data: vec![255; (size * size * 4) as usize],
        }
    }

    pub fn get_alpha(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let idx = ((y * self.width + x) * 4 + 3) as usize;
        self.data.get(idx).copied().unwrap_or(0)
    }

    pub fn set_alpha(&mut self, x: u32, y: u32, alpha: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4 + 3) as usize;
        if let Some(v) = self.data.get_mut(idx) {
            *v = alpha;
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dab_mask_creates_square_buffer() {
        let dab = DabMask::new(3);
        assert_eq!(dab.width, 3);
        assert_eq!(dab.height, 3);
        assert_eq!(dab.data.len(), 36);
    }

    #[test]
    fn test_dab_mask_get_alpha_returns_full_opacity_by_default() {
        let dab = DabMask::new(2);
        assert_eq!(dab.get_alpha(0, 0), 255);
        assert_eq!(dab.get_alpha(1, 1), 255);
    }

    #[test]
    fn test_dab_mask_get_alpha_returns_zero_for_out_of_bounds() {
        let dab = DabMask::new(2);
        assert_eq!(dab.get_alpha(5, 0), 0);
        assert_eq!(dab.get_alpha(0, 5), 0);
    }

    #[test]
    fn test_dab_mask_set_alpha_updates_pixel() {
        let mut dab = DabMask::new(2);
        dab.set_alpha(0, 0, 128);
        assert_eq!(dab.get_alpha(0, 0), 128);
    }

    #[test]
    fn test_dab_mask_set_alpha_ignores_out_of_bounds() {
        let mut dab = DabMask::new(2);
        dab.set_alpha(5, 5, 128);
        assert_eq!(dab.get_alpha(5, 5), 0);
    }

    #[test]
    fn test_dab_mask_clamped_to_minimum_size() {
        let dab = DabMask::new(0);
        assert_eq!(dab.width, 1);
        assert_eq!(dab.height, 1);
    }
}