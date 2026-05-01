//! UI state management for egui interface
//!
//! Holds tool selection, layer operations, and panel visibility.
//! Brush settings are stored in the active tool and accessed via the Tool trait.

use crate::app::AppState;
use crate::commands::{CanvasCommand, RemoveLayer, SetOpacity, ToggleVisibility};
use crate::tools::{BrushTool, SelectionTool};

pub use crate::tools::ToolType;

/// UI state containing all user-facing settings and operations
pub struct UiState {
    // Tool selection
    pub active_tool: ToolType,
    
    // Panel visibility
    pub show_tool_panel: bool,
    pub show_brush_panel: bool,
    pub show_layer_panel: bool,
    pub show_color_panel: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            active_tool: ToolType::Brush,
            show_tool_panel: true,
            show_brush_panel: true,
            show_layer_panel: true,
            show_color_panel: true,
        }
    }

    /// Set the active tool
    pub fn set_tool(&mut self, tool: ToolType) {
        self.active_tool = tool;
    }

    /// Apply UI tool selection to the AppState
    pub fn apply_to_app(&mut self, app: &mut AppState) {
        match self.active_tool {
            ToolType::Brush => {
                if app.tool_name() != "Brush" {
                    app.set_active_tool(Box::new(BrushTool::new()));
                }
            }
            ToolType::Selection => {
                if app.tool_name() != "Selection" {
                    app.set_active_tool(Box::new(SelectionTool::new()));
                }
            }
        }
    }

    /// Add a new layer
    pub fn add_layer(&mut self, app: &mut AppState) {
        app.add_layer();
    }

    /// Remove a layer by index
    pub fn remove_layer(&mut self, app: &mut AppState, index: usize) {
        if index > 0 && index < app.canvas().layer_count() {
            app.execute_command(CanvasCommand::RemoveLayer(RemoveLayer::new(index)));
        }
    }

    /// Toggle layer visibility
    pub fn toggle_layer_visibility(&mut self, app: &mut AppState, index: usize) {
        if index < app.canvas().layer_count() {
            app.execute_command(CanvasCommand::ToggleVisibility(ToggleVisibility::new(index)));
        }
    }

    /// Set layer opacity
    pub fn set_layer_opacity(&mut self, app: &mut AppState, index: usize, opacity: f32) {
        if index < app.canvas().layer_count() {
            app.execute_command(CanvasCommand::SetOpacity(SetOpacity::new(index, opacity)));
        }
    }

    /// Undo the last action
    pub fn undo(&mut self, app: &mut AppState) {
        app.undo();
    }

    /// Redo the last undone action
    pub fn redo(&mut self, app: &mut AppState) {
        app.redo();
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
