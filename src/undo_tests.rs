//! Tests for Phase 4: undo/redo with command pattern

#[test]
fn add_layer_command_edits_canvas() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::AddLayer;

    let mut canvas = Canvas::new();
    assert_eq!(canvas.layer_count(), 1); // Starts with 1 default layer

    let mut record: Record<AddLayer> = Record::new();
    let _ = record.edit(&mut canvas, AddLayer::default());
    
    assert_eq!(canvas.layer_count(), 2);
}

#[test]
fn add_layer_command_undo_removes_layer() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::AddLayer;

    let mut canvas = Canvas::new();
    let mut record: Record<AddLayer> = Record::new();
    let _ = record.edit(&mut canvas, AddLayer::default());
    assert_eq!(canvas.layer_count(), 2);

    record.undo(&mut canvas);
    assert_eq!(canvas.layer_count(), 1);
}

#[test]
fn add_layer_command_redo_adds_layer_back() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::AddLayer;

    let mut canvas = Canvas::new();
    let mut record: Record<AddLayer> = Record::new();
    let _ = record.edit(&mut canvas, AddLayer::default());
    record.undo(&mut canvas);
    assert_eq!(canvas.layer_count(), 1);

    record.redo(&mut canvas);
    assert_eq!(canvas.layer_count(), 2);
}

#[test]
fn remove_layer_command_removes_and_undo_restores() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::{AddLayer, RemoveLayer};

    let mut canvas = Canvas::new();
    let mut add_record: Record<AddLayer> = Record::new();
    let _ = add_record.edit(&mut canvas, AddLayer::default());
    assert_eq!(canvas.layer_count(), 2);

    let removed_idx = canvas.layer_count() - 1;
    let mut remove_record: Record<RemoveLayer> = Record::new();
    let _ = remove_record.edit(&mut canvas, RemoveLayer::new(removed_idx));
    assert_eq!(canvas.layer_count(), 1);

    remove_record.undo(&mut canvas);
    assert_eq!(canvas.layer_count(), 2);
}

#[test]
fn toggle_visibility_command_toggles_and_undo_restores() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::ToggleVisibility;

    let mut canvas = Canvas::new();
    // Default layer is visible
    assert!(canvas.layers()[0].visible);

    let mut record: Record<ToggleVisibility> = Record::new();
    let _ = record.edit(&mut canvas, ToggleVisibility::new(0));
    assert!(!canvas.layers()[0].visible);

    record.undo(&mut canvas);
    assert!(canvas.layers()[0].visible);
}

#[test]
fn set_opacity_command_changes_and_undo_restores() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::SetOpacity;

    let mut canvas = Canvas::new();
    assert_eq!(canvas.layers()[0].opacity, 1.0);

    let mut record: Record<SetOpacity> = Record::new();
    let _ = record.edit(&mut canvas, SetOpacity::new(0, 0.5));
    assert_eq!(canvas.layers()[0].opacity, 0.5);

    record.undo(&mut canvas);
    assert_eq!(canvas.layers()[0].opacity, 1.0);
}

#[test]
fn canvas_undo_record_handles_multiple_commands() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::{CanvasCommand, AddLayer, ToggleVisibility};

    let mut canvas = Canvas::new();
    let mut record: Record<CanvasCommand> = Record::new();
    
    // Add a layer
    let _ = record.edit(&mut canvas, CanvasCommand::AddLayer(AddLayer::default()));
    assert_eq!(canvas.layer_count(), 2);
    
    // Toggle visibility
    let _ = record.edit(&mut canvas, CanvasCommand::ToggleVisibility(ToggleVisibility::new(0)));
    assert!(!canvas.layers()[0].visible);
    
    // Undo toggle
    record.undo(&mut canvas);
    assert!(canvas.layers()[0].visible);
    assert_eq!(canvas.layer_count(), 2);
    
    // Undo add layer
    record.undo(&mut canvas);
    assert_eq!(canvas.layer_count(), 1);
}

#[test]
fn canvas_can_undo_and_can_redo() {
    use undo::Record;
    use crate::canvas::Canvas;
    use crate::commands::{CanvasCommand, AddLayer};

    let mut canvas = Canvas::new();
    let mut record: Record<CanvasCommand> = Record::new();
    
    assert!(!record.can_undo());
    assert!(!record.can_redo());
    
    let _ = record.edit(&mut canvas, CanvasCommand::AddLayer(AddLayer::default()));
    assert!(record.can_undo());
    assert!(!record.can_redo());
    
    record.undo(&mut canvas);
    assert!(!record.can_undo());
    assert!(record.can_redo());
}
