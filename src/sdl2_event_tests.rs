//! Tests for SDL2 event → AppEvent mapping

#[test]
fn sdl2_mouse_button_down_maps_to_press() {
    use sdl2::event::Event;
    use sdl2::mouse::MouseButton;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::MouseButtonDown {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x: 100,
        y: 200,
    };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Press { x, y }) => {
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
        }
        _ => panic!("expected Press event"),
    }
}

#[test]
fn sdl2_mouse_button_up_maps_to_release() {
    use sdl2::event::Event;
    use sdl2::mouse::MouseButton;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::MouseButtonUp {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mouse_btn: MouseButton::Left,
        clicks: 1,
        x: 150,
        y: 250,
    };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Release { x, y }) => {
            assert_eq!(x, 150.0);
            assert_eq!(y, 250.0);
        }
        _ => panic!("expected Release event"),
    }
}

#[test]
fn sdl2_mouse_motion_with_left_button_maps_to_drag() {
    use sdl2::event::Event;
    use sdl2::mouse::MouseState;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let mouse_state = MouseState::from_sdl_state(1); // SDL_BUTTON_LMASK = 1

    let sdl_event = Event::MouseMotion {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mousestate: mouse_state,
        xrel: 0,
        yrel: 0,
        x: 50,
        y: 75,
    };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Drag { x, y }) => {
            assert_eq!(x, 50.0);
            assert_eq!(y, 75.0);
        }
        _ => panic!("expected Drag event"),
    }
}

#[test]
fn sdl2_mouse_motion_without_button_maps_to_none() {
    use sdl2::event::Event;
    use sdl2::mouse::MouseState;
    use crate::window::sdl2::sdl_event_to_app_event;

    let mouse_state = MouseState::from_sdl_state(0); // no buttons pressed

    let sdl_event = Event::MouseMotion {
        timestamp: 0,
        window_id: 0,
        which: 0,
        mousestate: mouse_state,
        xrel: 0,
        yrel: 0,
        x: 50,
        y: 75,
    };

    let result = sdl_event_to_app_event(sdl_event);
    assert!(result.is_none());
}

#[test]
fn sdl2_key_down_maps_to_key_event() {
    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::KeyDown {
        timestamp: 0,
        window_id: 0,
        keycode: Some(Keycode::A),
        scancode: None,
        keymod: sdl2::keyboard::Mod::empty(),
        repeat: false,
    };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Key { code }) => {
            // Keycode::A maps to some string representation containing "a"
            assert!(code.contains("a") || code.contains("97"));
        }
        _ => panic!("expected Key event"),
    }
}

#[test]
fn sdl2_quit_maps_to_quit_event() {
    use sdl2::event::Event;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::Quit { timestamp: 0 };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Quit) => {}
        _ => panic!("expected Quit event"),
    }
}

#[test]
fn sdl2_window_size_changed_maps_to_resize() {
    use sdl2::event::Event;
    use crate::events::AppEvent;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::Window {
        timestamp: 0,
        window_id: 0,
        win_event: sdl2::event::WindowEvent::SizeChanged(800, 600),
    };

    let result = sdl_event_to_app_event(sdl_event);
    match result {
        Some(AppEvent::Resize { width, height }) => {
            assert_eq!(width, 800);
            assert_eq!(height, 600);
        }
        _ => panic!("expected Resize event"),
    }
}

#[test]
fn unhandled_sdl_event_maps_to_none() {
    use sdl2::event::Event;
    use crate::window::sdl2::sdl_event_to_app_event;

    let sdl_event = Event::Window {
        timestamp: 0,
        window_id: 0,
        win_event: sdl2::event::WindowEvent::Shown,
    };

    let result = sdl_event_to_app_event(sdl_event);
    assert!(result.is_none());
}
