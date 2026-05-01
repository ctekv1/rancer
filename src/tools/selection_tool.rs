//! Selection tool implementation

use crate::canvas::Canvas;
use crate::selection::PixelSelection;
use crate::tools::Tool;

/// Tool for pixel-region selection
pub struct SelectionTool {
    pub selection: Option<PixelSelection>,
    is_selecting: bool,
    start_x: f32,
    start_y: f32,
}

impl SelectionTool {
    pub fn new() -> Self {
        Self {
            selection: None,
            is_selecting: false,
            start_x: 0.0,
            start_y: 0.0,
        }
    }

    pub fn is_selecting(&self) -> bool {
        self.is_selecting
    }
}

impl Default for SelectionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for SelectionTool {
    fn on_press(&mut self, x: f32, y: f32, _canvas: &mut Canvas) {
        self.start_x = x;
        self.start_y = y;
        self.is_selecting = true;
    }

    fn on_drag(&mut self, x: f32, y: f32, _canvas: &mut Canvas) {
        if self.is_selecting {
            let min_x = self.start_x.min(x) as i32;
            let min_y = self.start_y.min(y) as i32;
            let width = (x - self.start_x).abs() as u32;
            let height = (y - self.start_y).abs() as u32;

            if width > 0 && height > 0 {
                self.selection = Some(PixelSelection::new(min_x, min_y, width, height));
            }
        } else if let Some(ref mut selection) = self.selection {
            // Move existing selection
            let dx = x - self.start_x;
            let dy = y - self.start_y;
            selection.move_selection(dx, dy);
            self.start_x = x;
            self.start_y = y;
        }
    }

    fn on_release(&mut self, _x: f32, _y: f32, _canvas: &mut Canvas) {
        if self.is_selecting {
            self.is_selecting = false;
            // Finalize the selection
            if let Some(_selection) = self.selection.as_ref() {
                // Could trigger begin_selection here or keep as pending
            }
        }
    }

    fn on_key(&mut self, code: &str) {
        match code {
            "enter" | "return" => {
                // Commit selection
                if let Some(_selection) = self.selection.as_ref() {
                    // selection.commit_selection(canvas);
                }
            }
            "escape" => {
                // Cancel selection
                if let Some(_selection) = self.selection.as_ref() {
                    // selection.cancel_selection(canvas);
                }
                self.selection = None;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "Selection"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
