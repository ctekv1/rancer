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
fn app_state_resize_does_not_change_canvas_size() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    let (orig_w, orig_h) = state.canvas().size();

    state.handle_event(AppEvent::Resize { width: 800, height: 600 });

    assert_eq!(state.canvas().size(), (orig_w, orig_h));
    assert_eq!(state.canvas().width(), 1280);
    assert_eq!(state.canvas().height(), 720);
}

#[test]
fn app_state_tracks_viewport_size_after_resize() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    assert_eq!(state.viewport_width(), 1280);
    assert_eq!(state.viewport_height(), 720);

    state.handle_event(AppEvent::Resize { width: 1920, height: 1080 });

    assert_eq!(state.viewport_width(), 1920);
    assert_eq!(state.viewport_height(), 1080);
    // Canvas should still be original size
    assert_eq!(state.canvas().width(), 1280);
    assert_eq!(state.canvas().height(), 720);
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

#[test]
fn app_event_wheel_contains_position_and_delta() {
    use crate::events::AppEvent;
    let evt = AppEvent::Wheel { x: 100.0, y: 200.0, delta: 1 };
    match evt {
        AppEvent::Wheel { x, y, delta } => {
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
            assert_eq!(delta, 1);
        }
        _ => panic!("expected Wheel event"),
    }
}

#[test]
fn app_event_pan_contains_deltas() {
    use crate::events::AppEvent;
    let evt = AppEvent::Pan { dx: 10.0, dy: 20.0 };
    match evt {
        AppEvent::Pan { dx, dy } => {
            assert_eq!(dx, 10.0);
            assert_eq!(dy, 20.0);
        }
        _ => panic!("expected Pan event"),
    }
}

#[test]
fn pan_event_moves_viewport_offset() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    let before_x = state.viewport().offset_x;
    let before_y = state.viewport().offset_y;

    state.handle_event(AppEvent::Pan { dx: 50.0, dy: 30.0 });

    assert!((state.viewport().offset_x - before_x - 50.0).abs() < 1e-6);
    assert!((state.viewport().offset_y - before_y - 30.0).abs() < 1e-6);
}

#[test]
fn wheel_event_zooms_in() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    let orig_scale = state.viewport().scale;

    state.handle_event(AppEvent::Wheel { x: 640.0, y: 360.0, delta: 1 });

    assert!(state.viewport().scale > orig_scale);
}

#[test]
fn wheel_event_zooms_out() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    state.viewport_mut().scale = 2.0;
    let orig_scale = state.viewport().scale;

    state.handle_event(AppEvent::Wheel { x: 640.0, y: 360.0, delta: -1 });

    assert!(state.viewport().scale < orig_scale);
}

#[test]
fn app_state_press_at_centered_viewport_maps_to_canvas_coord() {
    use crate::app::AppState;
    use crate::events::AppEvent;

    let mut state = AppState::new(1280, 720);
    // Resize viewport larger than canvas to trigger centering
    state.handle_event(AppEvent::Resize { width: 1920, height: 1080 });

    // Screen (400, 200) → canvas (80, 20) when canvas is centered in 1920x1080
    state.handle_event(AppEvent::Press { x: 400.0, y: 200.0 });

    let layer = &state.canvas().layers()[state.canvas().active_layer()];
    let pixel = layer.content.get_pixel(80, 20);
    assert!(pixel.is_some_and(|(_, _, _, a)| a > 0));
}
