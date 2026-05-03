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
    pub eraser_mode: bool,  // true = eraser mode (BrushTool with is_eraser=true)
    pub color_picker_open: bool,  // Color picker popup state
    pub pending_color: Option<crate::canvas::Color>,  // Color to apply after picker closes
    
    // Panel visibility
    pub show_tool_panel: bool,
    pub show_brush_panel: bool,
    pub show_layer_panel: bool,
    
    // Theme
    pub use_dark_theme: bool,
    
    // Pre-loaded SVG icon textures (to prevent disappearance on hover)
    #[cfg(feature = "svg")]
    pub icon_textures: std::collections::HashMap<&'static str, egui_sdl2::egui::TextureHandle>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            active_tool: ToolType::Brush,
            eraser_mode: false,
            color_picker_open: false,
            pending_color: None,
            show_tool_panel: true,
            show_brush_panel: true,
            show_layer_panel: true,
            use_dark_theme: true,
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
                // Check if we need to create a new tool or just toggle eraser mode
                if app.tool_name() != "Brush" {
                    let mut tool = BrushTool::new();
                    tool.set_eraser_mode(self.eraser_mode);
                    app.set_active_tool(Box::new(tool));
                } else {
                    // Tool is already Brush, just update eraser mode
                    if let Some(brush_tool) = app.active_tool_mut().as_any_mut().downcast_mut::<BrushTool>() {
                        brush_tool.set_eraser_mode(self.eraser_mode);
                    }
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
