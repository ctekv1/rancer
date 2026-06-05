pub const ZOOM_MIN: f64 = 0.1;
pub const ZOOM_MAX: f64 = 64.0;
pub const ZOOM_FACTOR: f64 = 1.15;

pub struct ViewportState {
    pub offset_x: f64,
    pub offset_y: f64,
    pub scale: f64,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub window_width: u32,
    pub window_height: u32,
}

impl ViewportState {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let (offset_x, offset_y) = if canvas_width <= window_width && canvas_height <= window_height
        {
            (
                (window_width - canvas_width) as f64 / 2.0,
                (window_height - canvas_height) as f64 / 2.0,
            )
        } else {
            (0.0, 0.0)
        };

        Self {
            offset_x,
            offset_y,
            scale: 1.0,
            canvas_width,
            canvas_height,
            window_width,
            window_height,
        }
    }

    fn zoom_impl(&mut self, center_x: f64, center_y: f64, factor: f64) {
        let old = self.scale;
        self.scale = (self.scale * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let cx = (center_x - self.offset_x) / old;
        let cy = (center_y - self.offset_y) / old;
        self.offset_x = center_x - cx * self.scale;
        self.offset_y = center_y - cy * self.scale;
    }

    pub fn zoom_in(&mut self) {
        self.zoom_impl(
            self.window_width as f64 / 2.0,
            self.window_height as f64 / 2.0,
            ZOOM_FACTOR,
        );
    }

    pub fn zoom_out(&mut self) {
        self.zoom_impl(
            self.window_width as f64 / 2.0,
            self.window_height as f64 / 2.0,
            1.0 / ZOOM_FACTOR,
        );
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx as f64;
        self.offset_y += dy as f64;
    }

    pub fn zoom_toward(&mut self, sx: f32, sy: f32, factor: f64) {
        self.zoom_impl(sx as f64, sy as f64, factor);
    }

    pub fn zoom_to_fit(&mut self) {
        let scale_x = self.window_width as f64 / self.canvas_width as f64;
        let scale_y = self.window_height as f64 / self.canvas_height as f64;
        self.scale = scale_x.min(scale_y).min(1.0);
        self.offset_x = (self.window_width as f64 - self.canvas_width as f64 * self.scale) / 2.0;
        self.offset_y = (self.window_height as f64 - self.canvas_height as f64 * self.scale) / 2.0;
    }

    pub fn zoom_to_100(&mut self) {
        self.scale = 1.0;
        self.offset_x = ((self.window_width as f64 - self.canvas_width as f64) / 2.0).max(0.0);
        self.offset_y = ((self.window_height as f64 - self.canvas_height as f64) / 2.0).max(0.0);
    }

    pub fn resize_window(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        let (offset_x, offset_y) = if self.canvas_width <= width && self.canvas_height <= height {
            (
                (width - self.canvas_width) as f64 / 2.0,
                (height - self.canvas_height) as f64 / 2.0,
            )
        } else {
            (0.0, 0.0)
        };
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    pub fn screen_to_canvas(&self, sx: f32, sy: f32) -> Option<(f32, f32)> {
        let cx = (sx as f64 - self.offset_x) / self.scale;
        let cy = (sy as f64 - self.offset_y) / self.scale;
        if cx >= 0.0 && cx < self.canvas_width as f64 && cy >= 0.0 && cy < self.canvas_height as f64
        {
            Some((cx as f32, cy as f32))
        } else {
            None
        }
    }

    /// Returns (x, y, width, height) of the visible canvas region in screen coordinates.
    /// The rectangle is clipped to the window bounds.
    pub fn viewport_rect(&self) -> (i32, i32, i32, i32) {
        let left = self.offset_x;
        let top = self.offset_y;
        let right = self.offset_x + self.canvas_width as f64 * self.scale;
        let bottom = self.offset_y + self.canvas_height as f64 * self.scale;

        let vx = left.max(0.0) as i32;
        let vy = top.max(0.0) as i32;
        let vw = (right.min(self.window_width as f64) - vx as f64) as i32;
        let vh = (bottom.min(self.window_height as f64) - vy as f64) as i32;

        (vx.max(0), vy.max(0), vw.max(0), vh.max(0))
    }

    /// Returns texture UV coordinates [x1,y1, x2,y2, x3,y3, x4,y4] for the fullscreen quad.
    /// Maps the visible portion of the canvas to the viewport rect.
    /// Uses OpenGL convention where v=0 is the bottom of the texture (canvas row 0, top of image)
    /// and v=1 is the top of the texture (last canvas row, bottom of image).
    pub fn texture_uv(&self) -> [f32; 8] {
        let (vx, vy, vw, vh) = self.viewport_rect();

        let canvas_x = (vx as f64 - self.offset_x) / self.scale;
        let canvas_y = (vy as f64 - self.offset_y) / self.scale;
        let canvas_rx = ((vx + vw) as f64 - self.offset_x) / self.scale;
        let canvas_by = ((vy + vh) as f64 - self.offset_y) / self.scale;

        let u0 = (canvas_x / self.canvas_width as f64).clamp(0.0, 1.0) as f32;
        let u1 = (canvas_rx / self.canvas_width as f64).clamp(0.0, 1.0) as f32;
        let v_bottom = (canvas_by / self.canvas_height as f64).clamp(0.0, 1.0) as f32;
        let v_top = (canvas_y / self.canvas_height as f64).clamp(0.0, 1.0) as f32;

        [u0, v_bottom, u1, v_bottom, u1, v_top, u0, v_top]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_viewport_centers_canvas_when_fitting_in_window() {
        let vp = ViewportState::new(1280, 720, 1920, 1080);
        assert_eq!(vp.offset_x, 320.0);
        assert_eq!(vp.offset_y, 180.0);
        assert_eq!(vp.scale, 1.0);
        assert_eq!(vp.canvas_width, 1280);
        assert_eq!(vp.canvas_height, 720);
        assert_eq!(vp.window_width, 1920);
        assert_eq!(vp.window_height, 1080);
    }

    #[test]
    fn new_viewport_puts_top_left_at_origin_when_canvas_larger_than_window() {
        let vp = ViewportState::new(1280, 720, 800, 600);
        assert_eq!(vp.offset_x, 0.0);
        assert_eq!(vp.offset_y, 0.0);
        assert_eq!(vp.scale, 1.0);
    }

    #[test]
    fn screen_to_canvas_maps_window_coord_to_canvas_coord_when_centered() {
        let vp = ViewportState::new(1280, 720, 1920, 1080);
        let result = vp.screen_to_canvas(400.0, 200.0);
        assert_eq!(result, Some((80.0, 20.0)));
    }

    #[test]
    fn screen_to_canvas_returns_none_for_letterbox_click() {
        let vp = ViewportState::new(1280, 720, 1920, 1080);
        assert_eq!(vp.screen_to_canvas(100.0, 100.0), None);
    }

    #[test]
    fn resize_window_recomputes_offset_when_window_grows() {
        let mut vp = ViewportState::new(1280, 720, 800, 600);
        assert_eq!(vp.offset_x, 0.0);
        assert_eq!(vp.offset_y, 0.0);

        vp.resize_window(1920, 1080);
        assert_eq!(vp.offset_x, 320.0);
        assert_eq!(vp.offset_y, 180.0);
        assert_eq!(vp.scale, 1.0);
    }

    #[test]
    fn zoom_to_fit_scales_down_oversized_canvas() {
        let mut vp = ViewportState::new(1280, 720, 800, 600);
        vp.zoom_to_fit();
        let expected_scale = (800.0_f64 / 1280.0).min(600.0_f64 / 720.0);
        assert!((vp.scale - expected_scale).abs() < 1e-12);
        assert!((vp.offset_x - 0.0).abs() < 1e-12);
        assert!((vp.offset_y - 75.0).abs() < 1e-12);
    }

    #[test]
    fn zoom_to_fit_does_not_upscale_when_canvas_fits() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.zoom_to_fit();
        assert_eq!(vp.scale, 1.0);
        assert_eq!(vp.offset_x, 320.0);
        assert_eq!(vp.offset_y, 180.0);
    }

    #[test]
    fn zoom_to_100_resets_scale_and_centers() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.scale = 2.0;
        vp.offset_x = 100.0;
        vp.offset_y = 50.0;

        vp.zoom_to_100();
        assert_eq!(vp.scale, 1.0);
        assert_eq!(vp.offset_x, 320.0);
        assert_eq!(vp.offset_y, 180.0);
    }

    #[test]
    fn zoom_in_increases_scale_and_preserves_viewport_center() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        let center = vp.screen_to_canvas(960.0, 540.0);

        vp.zoom_in();

        assert!(vp.scale > 1.0);
        let center_after = vp.screen_to_canvas(960.0, 540.0);
        assert_eq!(center, center_after);
    }

    #[test]
    fn zoom_out_decreases_scale_and_preserves_viewport_center() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        let center = vp.screen_to_canvas(960.0, 540.0);

        vp.zoom_out();

        assert!(vp.scale < 1.0);
        let center_after = vp.screen_to_canvas(960.0, 540.0);
        assert_eq!(center, center_after);
    }

    #[test]
    fn zoom_in_clamps_at_max() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.scale = ZOOM_MAX;

        vp.zoom_in();

        assert_eq!(vp.scale, ZOOM_MAX);
    }

    #[test]
    fn zoom_out_clamps_at_min() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.scale = ZOOM_MIN;

        vp.zoom_out();

        assert_eq!(vp.scale, ZOOM_MIN);
    }

    #[test]
    fn viewport_rect_full_canvas_when_equal_to_window() {
        let vp = ViewportState::new(1280, 720, 1280, 720);
        let (x, y, w, h) = vp.viewport_rect();
        assert_eq!((x, y, w, h), (0, 0, 1280, 720));
    }

    #[test]
    fn viewport_rect_centered_when_window_larger_than_canvas() {
        let vp = ViewportState::new(1280, 720, 1920, 1080);
        let (x, y, w, h) = vp.viewport_rect();
        assert_eq!((x, y, w, h), (320, 180, 1280, 720));
    }

    #[test]
    fn viewport_rect_full_window_when_canvas_larger_than_window() {
        let vp = ViewportState::new(1280, 720, 640, 480);
        let (x, y, w, h) = vp.viewport_rect();
        assert_eq!((x, y, w, h), (0, 0, 640, 480));
    }

    #[test]
    fn texture_uv_full_when_canvas_fits_in_window() {
        let vp = ViewportState::new(1280, 720, 1920, 1080);
        let uv = vp.texture_uv();
        assert_eq!(uv, [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn zoom_toward_clamps_at_max() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.scale = ZOOM_MAX;
        vp.zoom_toward(600.0, 300.0, ZOOM_FACTOR);
        assert_eq!(vp.scale, ZOOM_MAX);
    }

    #[test]
    fn zoom_toward_clamps_at_min() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.scale = ZOOM_MIN;
        vp.zoom_toward(600.0, 300.0, 1.0 / ZOOM_FACTOR);
        assert_eq!(vp.scale, ZOOM_MIN);
    }

    #[test]
    fn viewport_rect_vy_is_window_from_top_not_opengl_from_bottom() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.pan(0.0, -100.0);
        let (_, vy, _, vh) = vp.viewport_rect();
        let gl_y = vp.window_height as i32 - vy - vh;
        // After panning up 100px from center (vy=180), vy should be ~80
        // gl_y should be 1080 - 80 - 720 = 280
        assert_ne!(vy, gl_y);
        assert_eq!(gl_y, 280);
    }

    #[test]
    fn screen_to_canvas_at_viewport_top_left_maps_to_near_origin() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        vp.pan(0.0, -100.0);
        let (vx, vy, _, _) = vp.viewport_rect();
        let canvas = vp.screen_to_canvas(vx as f32, vy as f32);
        assert!(canvas.is_some());
        let (cx, cy) = canvas.unwrap();
        assert!((cx - 0.0).abs() < 1e-6);
        assert!((cy - 0.0).abs() < 1e-6);
    }

    #[test]
    fn pan_moves_offset() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        let before_x = vp.offset_x;
        let before_y = vp.offset_y;

        vp.pan(100.0, 50.0);

        assert!((vp.offset_x - before_x - 100.0).abs() < 1e-6);
        assert!((vp.offset_y - before_y - 50.0).abs() < 1e-6);
    }

    #[test]
    fn pan_with_negative_values() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        let before_x = vp.offset_x;
        let before_y = vp.offset_y;

        vp.pan(-30.0, -20.0);

        assert!((vp.offset_x - before_x + 30.0).abs() < 1e-6);
        assert!((vp.offset_y - before_y + 20.0).abs() < 1e-6);
    }

    #[test]
    fn zoom_toward_preserves_cursor_point() {
        let mut vp = ViewportState::new(1280, 720, 1920, 1080);
        let cursor = vp.screen_to_canvas(600.0, 300.0).unwrap();

        vp.zoom_toward(600.0, 300.0, ZOOM_FACTOR);

        assert!(vp.scale > 1.0);
        let cursor_after = vp.screen_to_canvas(600.0, 300.0).unwrap();
        assert!((cursor_after.0 - cursor.0).abs() < 1e-6);
        assert!((cursor_after.1 - cursor.1).abs() < 1e-6);
    }

    #[test]
    fn texture_uv_clipped_when_window_smaller_than_canvas() {
        let vp = ViewportState::new(1280, 720, 640, 480);
        let uv = vp.texture_uv();
        let uw = 640.0 / 1280.0;
        let vh = 480.0 / 720.0;
        // top-left sub-region: u in [0, uw], visible fraction vh of canvas height
        assert!((uv[0] - 0.0).abs() < 1e-6);
        assert!((uv[1] - vh).abs() < 1e-6);
        assert!((uv[2] - uw).abs() < 1e-6);
        assert!((uv[3] - vh).abs() < 1e-6);
        assert!((uv[4] - uw).abs() < 1e-6);
        assert!((uv[5] - 0.0).abs() < 1e-6);
        assert!((uv[6] - 0.0).abs() < 1e-6);
        assert!((uv[7] - 0.0).abs() < 1e-6);
    }
}
