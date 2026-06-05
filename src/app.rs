//! AppState — owns canvas, active tool, and handles events
//!
//! This is the core application state that bridges SDL2 events
//! to domain operations on the canvas.

use undo::Record;

use crate::canvas::Canvas;
use crate::commands::{AddLayer, CanvasCommand};
use crate::events::AppEvent;
use crate::tools::{BrushTool, Tool};
use crate::viewport::ViewportState;

/// Application state containing all mutable application data
pub struct AppState {
    canvas: Canvas,
    active_tool: Box<dyn Tool>,
    history: Record<CanvasCommand>,
    viewport: ViewportState,
}

impl AppState {
    /// Create a new AppState with default canvas and brush tool
    pub fn new(width: u32, height: u32) -> Self {
        let mut canvas = Canvas::new();
        // Initialize canvas with given dimensions
        canvas.resize(width, height);

        Self {
            viewport: ViewportState::new(width, height, width, height),
            canvas,
            active_tool: Box::new(BrushTool::new()),
            history: Record::new(),
        }
    }

    /// Get a reference to the canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Get the viewport width (window width)
    pub fn viewport_width(&self) -> u32 {
        self.viewport.window_width
    }

    /// Get the viewport height (window height)
    pub fn viewport_height(&self) -> u32 {
        self.viewport.window_height
    }

    /// Get a reference to the viewport state
    pub fn viewport(&self) -> &ViewportState {
        &self.viewport
    }

    /// Get a mutable reference to the viewport state
    pub fn viewport_mut(&mut self) -> &mut ViewportState {
        &mut self.viewport
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// Get a mutable reference to the undo history
    pub fn history_mut(&mut self) -> &mut Record<CanvasCommand> {
        &mut self.history
    }

    /// Get the name of the active tool
    pub fn tool_name(&self) -> &str {
        self.active_tool.name()
    }

    /// Get an immutable reference to the active tool
    pub fn active_tool(&self) -> &dyn Tool {
        self.active_tool.as_ref()
    }

    /// Get a mutable reference to the active tool as a trait object
    pub fn active_tool_mut(&mut self) -> &mut dyn Tool {
        self.active_tool.as_mut()
    }

    /// Set the active tool
    pub fn set_active_tool(&mut self, tool: Box<dyn Tool>) {
        self.active_tool = tool;
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Handle an application event
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Press { x, y } => {
                if let Some((cx, cy)) = self.viewport.screen_to_canvas(x, y) {
                    self.active_tool.on_press(cx, cy, &mut self.canvas);
                }
            }
            AppEvent::Drag { x, y } => {
                if let Some((cx, cy)) = self.viewport.screen_to_canvas(x, y) {
                    self.active_tool.on_drag(cx, cy, &mut self.canvas);
                }
            }
            AppEvent::Release { x, y } => {
                if let Some((cx, cy)) = self.viewport.screen_to_canvas(x, y) {
                    self.active_tool.on_release(cx, cy, &mut self.canvas);
                }
            }
            AppEvent::Key { code } => {
                self.handle_key(&code);
            }
            AppEvent::Resize { width, height } => {
                self.viewport.resize_window(width, height);
            }
            AppEvent::Wheel { x, y, delta } => {
                let factor = if delta > 0 {
                    crate::viewport::ZOOM_FACTOR
                } else {
                    1.0 / crate::viewport::ZOOM_FACTOR
                };
                self.viewport.zoom_toward(x, y, factor);
            }
            AppEvent::Pan { dx, dy } => {
                self.viewport.pan(dx, dy);
            }
            AppEvent::Quit => {
                // Handled by SDL2 loop
            }
        }
    }

    fn handle_key(&mut self, code: &str) {
        match code {
            "z" => {
                if self.history.can_undo() {
                    self.history.undo(&mut self.canvas);
                }
            }
            "y" => {
                if self.history.can_redo() {
                    self.history.redo(&mut self.canvas);
                }
            }
            _ => {
                self.active_tool.on_key(code);
            }
        }
    }

    /// Add a new layer through the undo system
    pub fn add_layer(&mut self) {
        let _ = self.history.edit(
            &mut self.canvas,
            CanvasCommand::AddLayer(AddLayer::default()),
        );
    }

    /// Undo the last action
    pub fn undo(&mut self) {
        if self.history.can_undo() {
            self.history.undo(&mut self.canvas);
        }
    }

    /// Redo the last undone action
    pub fn redo(&mut self) {
        if self.history.can_redo() {
            self.history.redo(&mut self.canvas);
        }
    }

    /// Execute a canvas command through the undo system
    pub fn execute_command(&mut self, command: CanvasCommand) {
        let _ = self.history.edit(&mut self.canvas, command);
    }
}
