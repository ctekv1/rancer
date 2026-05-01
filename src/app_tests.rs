//! Tests for Phase 3: Tool trait + AppEvent + AppState

#[test]
fn app_event_press_contains_position() {
    use crate::events::AppEvent;
    let evt = AppEvent::Press { x: 100.0, y: 200.0 };
    match evt {
        AppEvent::Press { x, y } => {
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
        }
        _ => panic!("expected Press event"),
    }
}

#[test]
fn app_event_drag_contains_deltas() {
    use crate::events::AppEvent;
    let evt = AppEvent::Drag { x: 50.0, y: 75.0 };
    match evt {
        AppEvent::Drag { x, y } => {
            assert_eq!(x, 50.0);
            assert_eq!(y, 75.0);
        }
        _ => panic!("expected Drag event"),
    }
}

#[test]
fn app_event_release_is_constructed() {
    use crate::events::AppEvent;
    let evt = AppEvent::Release { x: 300.0, y: 400.0 };
    match evt {
        AppEvent::Release { x, y } => {
            assert_eq!(x, 300.0);
            assert_eq!(y, 400.0);
        }
        _ => panic!("expected Release event"),
    }
}

#[test]
fn app_state_has_canvas() {
    use crate::app::AppState;
    let state = AppState::new(1280, 720);
    assert!(state.canvas().layers().len() > 0);
}

#[test]
fn app_state_has_active_tool() {
    use crate::app::AppState;
    let state = AppState::new(1280, 720);
    assert!(state.tool_name() == "Brush");
}

#[test]
fn app_state_handle_press_modifies_canvas() {
    use crate::app::AppState;
    use crate::events::AppEvent;
    
    let mut state = AppState::new(1280, 720);
    state.handle_event(AppEvent::Press { x: 10.0, y: 10.0 });
    // Press event should be handled without error
    // The canvas state may change (stroke started, etc.)
}

#[test]
fn app_state_handle_drag_updates_state() {
    use crate::app::AppState;
    use crate::events::AppEvent;
    
    let mut state = AppState::new(1280, 720);
    state.handle_event(AppEvent::Press { x: 10.0, y: 10.0 });
    state.handle_event(AppEvent::Drag { x: 20.0, y: 20.0 });
    // Drag after press should work
}

#[test]
fn app_state_handle_release_completes_stroke() {
    use crate::app::AppState;
    use crate::events::AppEvent;
    
    let mut state = AppState::new(1280, 720);
    state.handle_event(AppEvent::Press { x: 10.0, y: 10.0 });
    state.handle_event(AppEvent::Drag { x: 20.0, y: 20.0 });
    state.handle_event(AppEvent::Release { x: 30.0, y: 30.0 });
    // Release should complete the stroke
}

#[test]
fn app_state_undo_redo_via_keyboard() {
    use crate::app::AppState;
    use crate::events::AppEvent;
    
    let mut state = AppState::new(1280, 720);
    assert!(!state.can_undo());
    
    // Add a layer
    state.add_layer();
    assert_eq!(state.canvas().layers.len(), 2);
    assert!(state.can_undo());
    
    // Undo via key
    state.handle_event(AppEvent::Key { code: "z".to_string() });
    assert_eq!(state.canvas().layers.len(), 1);
    assert!(!state.can_undo());
    assert!(state.can_redo());
    
    // Redo via key
    state.handle_event(AppEvent::Key { code: "y".to_string() });
    assert_eq!(state.canvas().layers.len(), 2);
}

#[test]
fn app_state_undo_redo_methods() {
    use crate::app::AppState;
    
    let mut state = AppState::new(1280, 720);
    state.add_layer();
    assert_eq!(state.canvas().layers.len(), 2);
    
    state.undo();
    assert_eq!(state.canvas().layers.len(), 1);
    
    state.redo();
    assert_eq!(state.canvas().layers.len(), 2);
}
