//! Round dab - antialiased circular brush tip

use crate::brush::DabMask;

pub struct RoundDab;

impl RoundDab {
    pub fn generate(size: u32) -> DabMask {
        if size <= 2 {
            return DabMask::new(size);
        }

        let mut dab = DabMask::new(size);
        let cx = size as f32 / 2.0;
        let cy = size as f32 / 2.0;
        let radius = (size as f32 - 1.0) / 2.0;
        let radius_sq = radius * radius;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist_sq = dx * dx + dy * dy;
                let alpha = if dist_sq <= radius_sq {
                    let dist = dist_sq.sqrt();
                    let edge_dist = radius - dist;
                    ((edge_dist / radius) * 255.0) as u8
                } else {
                    0
                };
                dab.set_alpha(x, y, alpha);
            }
        }
        dab
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_dab_center_has_high_opacity() {
        let dab = RoundDab::generate(5);
        let alpha = dab.get_alpha(2, 2);
        assert!(alpha > 128);
    }

    #[test]
    fn test_round_dab_corners_have_zero_opacity() {
        let dab = RoundDab::generate(5);
        assert_eq!(dab.get_alpha(0, 0), 0);
        assert_eq!(dab.get_alpha(4, 4), 0);
    }

    #[test]
    fn test_round_dab_edge_pixels_are_antialiased() {
        let dab = RoundDab::generate(5);
        let center_alpha = dab.get_alpha(2, 2);
        let corner_alpha = dab.get_alpha(0, 0);
        assert!(center_alpha > corner_alpha);
        assert!(center_alpha > 0);
        assert_eq!(corner_alpha, 0);
    }

    #[test]
    fn test_round_dab_small_size() {
        let dab = RoundDab::generate(1);
        assert!(dab.get_alpha(0, 0) > 0);
    }
}
