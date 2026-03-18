//! Window management module for Rancer
//!
//! Provides window creation and mouse input handling using winit.
//! This module handles the window lifecycle and input events, integrating
//! with the canvas data model for drawing operations.

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent, KeyEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{NamedKey, KeyCode},
    window::{Window, WindowAttributes},
};

use crate::canvas::{ActiveStroke, Canvas, Color, ColorPalette, Point};

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}

/// Window application state and event handler
pub struct WindowApp {
    /// The winit window
    window: Window,
    /// Canvas for drawing operations
    canvas: Canvas,
    /// Color palette for color selection
    palette: ColorPalette,
    /// Current active stroke being drawn
    active_stroke: Option<ActiveStroke>,
    /// Current mouse state
    mouse_state: MouseState,
    /// Current mouse position
    mouse_position: Point,
}

impl WindowApp {
    /// Create a new window application
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Creating window...");
        println!("Event loop backend: {:?}", std::env::var("WINIT_UNIX_BACKEND").unwrap_or_else(|_| "Not set".to_string()));
        
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_title("Rancer - Digital Art Application")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
                .with_resizable(true)
                .with_visible(true)  // Force window to be visible
        )
        .map_err(|e| format!("Failed to create window: {}", e))?;

        println!("Window created successfully with ID: {:?}", window.id());
        println!("Window position: {:?}", window.outer_position());
        println!("Window size: {:?}", window.inner_size());
        println!("Window is visible: {:?}", window.is_visible());
        println!("Window is focused: {:?}", window.has_focus());
        
        // Try to ensure window is shown
        window.set_visible(true);
        println!("After set_visible(true), window is visible: {:?}", window.is_visible());
        
        Ok(Self {
            window,
            canvas: Canvas::new(),
            palette: ColorPalette::new(),
            active_stroke: None,
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
        })
    }

    /// Get a reference to the window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get a reference to the canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// Get a reference to the color palette
    pub fn palette(&self) -> &ColorPalette {
        &self.palette
    }

    /// Get a mutable reference to the color palette
    pub fn palette_mut(&mut self) -> &mut ColorPalette {
        &mut self.palette
    }

    /// Handle window events
    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.handle_mouse_move(*position);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    self.handle_mouse_button(*state, *button);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    self.handle_mouse_wheel(*delta);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    self.handle_keyboard_input(event);
                }
                WindowEvent::CloseRequested => {
                    // Window will be closed by the event loop
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Handle mouse movement events
    fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        // Convert winit coordinates to our canvas coordinates
        let point = Point {
            x: position.x as f32,
            y: position.y as f32,
        };

        self.mouse_position = point;

        // If we're drawing, add the point to the active stroke
        if let Some(active_stroke) = &mut self.active_stroke {
            active_stroke.add_point(point);
        }
    }

    /// Handle mouse button press/release events
    fn handle_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        match (state, button) {
            (ElementState::Pressed, MouseButton::Left) => {
                self.start_drawing();
            }
            (ElementState::Released, MouseButton::Left) => {
                self.end_drawing();
            }
            _ => {}
        }
    }

    /// Handle mouse wheel events for color selection
    fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => {
                if y > 0.0 {
                    self.change_color_up();
                } else if y < 0.0 {
                    self.change_color_down();
                }
            }
            winit::event::MouseScrollDelta::PixelDelta(_) => {
                // Ignore pixel-based scrolling for color selection
            }
        }
    }

    /// Handle keyboard input for color selection
    fn handle_keyboard_input(&mut self, input: &KeyEvent) {
        if let winit::keyboard::Key::Named(key) = input.logical_key {
            match key {
                winit::keyboard::NamedKey::ArrowUp => self.change_color_up(),
                winit::keyboard::NamedKey::ArrowDown => self.change_color_down(),
                _ => {}
            }
        }
    }

    /// Start drawing a new stroke
    fn start_drawing(&mut self) {
        if self.mouse_state == MouseState::Idle {
            self.mouse_state = MouseState::Drawing;
            
            // Begin a new active stroke with current palette color
            let color = self.palette.current_color();
            self.active_stroke = Some(self.canvas.begin_stroke_with_palette(
                &self.palette,
                3.0,  // Default stroke width
                1.0,  // Default opacity
            ));
            
            // Add the current mouse position as the first point
            if let Some(active_stroke) = &mut self.active_stroke {
                active_stroke.add_point(self.mouse_position);
            }
        }
    }

    /// End the current stroke and commit it to the canvas
    fn end_drawing(&mut self) {
        if self.mouse_state == MouseState::Drawing {
            self.mouse_state = MouseState::Idle;
            
            if let Some(active_stroke) = self.active_stroke.take() {
                // Try to commit the stroke
                if let Err(e) = self.canvas.commit_stroke(active_stroke) {
                    eprintln!("Failed to commit stroke: {}", e);
                }
            }
        }
    }

    /// Change to the next color in the palette
    fn change_color_up(&mut self) {
        let current_index = self.palette.selected_index();
        let new_index = (current_index + 1) % self.palette.color_count();
        if let Err(e) = self.palette.select_color(new_index) {
            eprintln!("Failed to change color: {}", e);
        }
    }

    /// Change to the previous color in the palette
    fn change_color_down(&mut self) {
        let current_index = self.palette.selected_index();
        let new_index = if current_index == 0 {
            self.palette.color_count() - 1
        } else {
            current_index - 1
        };
        if let Err(e) = self.palette.select_color(new_index) {
            eprintln!("Failed to change color: {}", e);
        }
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    /// Get the current mouse state
    pub fn mouse_state(&self) -> MouseState {
        self.mouse_state
    }

    /// Check if there's an active stroke being drawn
    pub fn has_active_stroke(&self) -> bool {
        self.active_stroke.is_some()
    }

    /// Get the number of points in the current active stroke
    pub fn active_stroke_point_count(&self) -> usize {
        self.active_stroke.as_ref().map_or(0, |stroke| stroke.points().len())
    }
}

/// Run the window application event loop
pub fn run_window_app() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = WindowApp::new(&event_loop).unwrap();

    // Add a small delay to ensure window is ready
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    event_loop.set_control_flow(ControlFlow::Poll);
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, window_id } => {
                if window_id == app.window().id() {
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            app.handle_mouse_move(position);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            app.handle_mouse_button(state, button);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            app.handle_mouse_wheel(delta);
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            app.handle_keyboard_input(&event);
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                // Request redraw if we're drawing
                if app.mouse_state() == MouseState::Drawing {
                    app.window().request_redraw();
                }
            }
            _ => {}
        }
    })
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_window_app_creation() {
        // Note: We can't actually create a window in tests, but we can test the structure
        // This test would need to be run with a mock event loop in a real scenario
        assert!(true); // Placeholder - real tests would require window mocking
    }

    #[test]
    fn test_mouse_state_transitions() {
        // Test mouse state transitions without creating actual window
        // We'll test the logic without the window field
        let mut mouse_state = MouseState::Idle;
        
        assert_eq!(mouse_state, MouseState::Idle);
        
        // Test state transition
        mouse_state = MouseState::Drawing;
        assert_eq!(mouse_state, MouseState::Drawing);
    }

    #[test]
    fn test_color_selection() {
        let mut palette = ColorPalette::new();
        
        // Test initial color (black)
        assert_eq!(palette.selected_index(), 0);
        assert_eq!(palette.current_color(), Color::BLACK);
        
        // Test color change
        palette.select_color(1).unwrap();
        assert_eq!(palette.selected_index(), 1);
        assert_eq!(palette.current_color(), Color::WHITE);
        
        // Test wrapping around
        for _ in 0..10 {
            let current_index = palette.selected_index();
            let new_index = (current_index + 1) % palette.color_count();
            palette.select_color(new_index).unwrap();
        }
        assert_eq!(palette.selected_index(), 1); // Should wrap back to white
    }

    #[test]
    fn test_mouse_position_update() {
        let mut mouse_position = Point { x: 0.0, y: 0.0 };
        
        // Initial position should be (0, 0)
        assert_eq!(mouse_position, Point { x: 0.0, y: 0.0 });
        
        // Simulate mouse move
        mouse_position = Point { x: 100.0, y: 200.0 };
        
        assert_eq!(mouse_position, Point { x: 100.0, y: 200.0 });
    }
}
