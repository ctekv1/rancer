//! Tests for Phase 7: egui UI state management

#[test]
fn ui_state_created_with_default_brush_settings() {
    use crate::ui::UiState;

    let state = UiState::new();
    assert_eq!(state.active_tool, crate::ui::ToolType::Brush);
}

#[test]
fn brush_settings_read_through_tool_trait() {
    use crate::app::AppState;

    let app = AppState::new(1280, 720);
    
    // Read brush settings through trait
    let settings = app.active_tool().brush_settings();
    assert_eq!(settings.size, 10);
    assert_eq!(settings.opacity, 1.0);
}

#[test]
fn brush_settings_write_through_tool_trait() {
    use crate::app::AppState;
    use crate::canvas::Color;

    let mut app = AppState::new(1280, 720);
    
    // Write brush settings through trait
    app.active_tool_mut().set_brush_size(25);
    app.active_tool_mut().set_brush_opacity(0.5);
    app.active_tool_mut().set_brush_color(Color { r: 255, g: 0, b: 0, a: 255 });
    
    // Verify settings were applied
    let settings = app.active_tool().brush_settings();
    assert_eq!(settings.size, 25);
    assert_eq!(settings.opacity, 0.5);
    assert_eq!(settings.color.r, 255);
}

#[test]
fn ui_state_switches_to_brush_tool() {
    use crate::ui::UiState;
    use crate::ui::ToolType;

    let mut state = UiState::new();
    state.active_tool = ToolType::Selection;
    state.set_tool(ToolType::Brush);
    assert_eq!(state.active_tool, ToolType::Brush);
}

#[test]
fn ui_state_switches_to_selection_tool() {
    use crate::ui::UiState;
    use crate::ui::ToolType;

    let mut state = UiState::new();
    state.active_tool = ToolType::Brush;
    state.set_tool(ToolType::Selection);
    assert_eq!(state.active_tool, ToolType::Selection);
}

#[test]
fn ui_state_apply_tool_selection_to_brush_tool() {
    use crate::ui::UiState;
    use crate::app::AppState;
    use crate::ui::ToolType;

    let mut ui = UiState::new();
    ui.set_tool(ToolType::Brush);

    let mut app = AppState::new(1280, 720);
    ui.apply_to_app(&mut app);

    assert_eq!(app.tool_name(), "Brush");
}

#[test]
fn ui_state_layer_operations() {
    use crate::ui::UiState;
    use crate::app::AppState;

    let mut ui = UiState::new();
    let mut app = AppState::new(1280, 720);
    
    // Start with 1 layer
    assert_eq!(app.canvas().layers.len(), 1);
    
    // Add layer via UI
    ui.add_layer(&mut app);
    assert_eq!(app.canvas().layers.len(), 2);
}

#[test]
fn ui_state_toggle_layer_visibility() {
    use crate::ui::UiState;
    use crate::app::AppState;

    let mut ui = UiState::new();
    let mut app = AppState::new(1280, 720);
    
    // Layer starts visible
    assert!(app.canvas().layers[0].visible);
    
    // Toggle visibility
    ui.toggle_layer_visibility(&mut app, 0);
    assert!(!app.canvas().layers[0].visible);
    
    // Toggle back
    ui.toggle_layer_visibility(&mut app, 0);
    assert!(app.canvas().layers[0].visible);
}

#[test]
fn ui_state_remove_layer() {
    use crate::ui::UiState;
    use crate::app::AppState;

    let mut ui = UiState::new();
    let mut app = AppState::new(1280, 720);
    
    // Add a layer first
    ui.add_layer(&mut app);
    assert_eq!(app.canvas().layers.len(), 2);
    
    // Remove the added layer (index 1)
    ui.remove_layer(&mut app, 1);
    assert_eq!(app.canvas().layers.len(), 1);
}

#[test]
fn ui_state_undo_redo_via_ui() {
    use crate::ui::UiState;
    use crate::app::AppState;

    let mut ui = UiState::new();
    let mut app = AppState::new(1280, 720);
    
    // Add layer
    ui.add_layer(&mut app);
    assert!(app.can_undo());
    
    // Undo via UI
    ui.undo(&mut app);
    assert!(!app.can_undo());
    assert!(app.can_redo());
    
    // Redo via UI
    ui.redo(&mut app);
    assert!(app.can_undo());
}

#[test]
fn ui_state_panel_visibility_toggles() {
    use crate::ui::UiState;

    let mut state = UiState::new();
    
    // Tool panel visible by default
    assert!(state.show_tool_panel);
    assert!(state.show_brush_panel);
    assert!(state.show_layer_panel);
    
    // Toggle panels
    state.show_tool_panel = false;
    state.show_brush_panel = false;
    state.show_layer_panel = false;
    
    assert!(!state.show_tool_panel);
    assert!(!state.show_brush_panel);
    assert!(!state.show_layer_panel);
}
