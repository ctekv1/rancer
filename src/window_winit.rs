//! Window management module for Rancer using winit + WGPU
//!
//! Provides window creation, mouse input handling, and WGPU rendering using winit.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.

use std::rc::Rc;
use std::cell::RefCell;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use crate::canvas::{ActiveStroke, Canvas, ColorPalette, Point};
use crate::renderer::{Renderer, RendererConfig};
use crate::logger;
use crate::window_backend::{WindowBackend, MouseState as BackendMouseState};

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}



/// Window application state using winit
pub struct WindowApp {
    /// The winit window
    window: Option<Rc<Window>>,
    /// WGPU renderer
    renderer: Option<Renderer>,
    /// Canvas for drawing operations
    canvas: Rc<RefCell<Canvas>>,
    /// Color palette for color selection
    palette: Rc<RefCell<ColorPalette>>,
    /// Current active stroke being drawn
    active_stroke: Rc<RefCell<Option<ActiveStroke>>>,
    /// Current mouse state
    mouse_state: MouseState,
    /// Current mouse position
    mouse_position: Point,
    /// Current brush size in pixels
    brush_size: f32,
}

impl WindowApp {
    /// Create a new window application
    pub fn new() -> Self {
        logger::info("Creating winit window application...");
        
        Self {
            window: None,
            renderer: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            palette: Rc::new(RefCell::new(ColorPalette::new())),
            active_stroke: Rc::new(RefCell::new(None)),
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
            brush_size: 3.0,
        }
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            logger::info("=== WINDOW CREATION ===");
            
            let attributes = Window::default_attributes()
                .with_title("Rancer")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            
            let window = event_loop.create_window(attributes).unwrap();
            let window = Rc::new(window);
            
            logger::info("winit window created successfully");
            logger::info(&format!("Window size: 1280x720"));
            logger::info("Window title: Rancer");
            logger::info("========================");
            
            // Initialize WGPU renderer
            logger::info("=== RENDERER INITIALIZATION ===");
            
            let size = window.inner_size();
            let config = RendererConfig::default();
            
            // Use tokio runtime to initialize WGPU (async)
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(Renderer::new(config, &*window, (size.width, size.height))) {
                Ok(renderer) => {
                    logger::info("✅ WGPU renderer initialized successfully!");
                    renderer.print_backend_status();
                    self.renderer = Some(renderer);
                }
                Err(e) => {
                    logger::error(&format!("❌ Failed to initialize renderer: {}", e));
                    logger::warn("   Application may not render correctly");
                }
            }
            logger::info("===============================");
            
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                logger::info("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                logger::info(&format!("Window resized to {}x{}", physical_size.width, physical_size.height));
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize((physical_size.width, physical_size.height));
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Render the frame
                if let Some(renderer) = &mut self.renderer {
                    // Update canvas in renderer
                    renderer.set_canvas(self.canvas.borrow().clone());
                    
                    // Update active stroke in renderer
                    let active_stroke = self.active_stroke.borrow().clone();
                    renderer.set_active_stroke(active_stroke);
                    
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = self.window.as_ref().unwrap().inner_size();
                            renderer.resize((size.width, size.height));
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            logger::error("Out of memory!");
                            event_loop.exit();
                        }
                        Err(e) => {
                            logger::error(&format!("Render error: {:?}", e));
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state: button_state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            println!("Mouse button pressed at ({}, {})", self.mouse_position.x, self.mouse_position.y);
                            
                            // Check if click is on UI elements
                            let x = self.mouse_position.x;
                            let y = self.mouse_position.y;
                            
                            // Check color palette click (top left, y=10 to y=30)
                            if y >= 10.0 && y <= 30.0 {
                                let palette_x = 10.0;
                                let color_width = 20.0;
                                let spacing = 5.0;
                                let color_count = self.palette.borrow().color_count();
                                
                                for i in 0..color_count {
                                    let color_x = palette_x + (color_width + spacing) * i as f32;
                                    if x >= color_x && x <= color_x + color_width {
                                        if let Err(e) = self.palette.borrow_mut().select_color(i) {
                                            eprintln!("Failed to select color: {}", e);
                                        } else {
                                            println!("Selected color at index {}", i);
                                        }
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
                                        return;
                                    }
                                }
                            }
                            
                            // Check brush size selector click (y=50 to y=80)
                            if y >= 50.0 && y <= 80.0 {
                                let selector_x = 10.0;
                                let button_size = 30.0;
                                let spacing = 10.0;
                                let brush_sizes = [3.0, 5.0, 10.0, 25.0, 50.0];
                                
                                for (i, &size) in brush_sizes.iter().enumerate() {
                                    let button_x = selector_x + (button_size + spacing) * i as f32;
                                    if x >= button_x && x <= button_x + button_size {
                                        self.brush_size = size;
                                        // Update renderer with new brush size
                                        if let Some(renderer) = &mut self.renderer {
                                            renderer.set_brush_size(size);
                                        }
                                        println!("Selected brush size: {}", size);
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
                                        return;
                                    }
                                }
                            }
                            
                            // If not on UI, start drawing
                            self.mouse_state = MouseState::Drawing;
                            
                            // Begin a new active stroke
                            let color = self.palette.borrow().current_color();
                            let mut canvas = self.canvas.borrow_mut();
                            let active_stroke = canvas.begin_stroke_with_palette(
                                &self.palette.borrow(),
                                self.brush_size,
                                1.0,
                            );
                            println!("Created active stroke with color RGB({}, {}, {}) and width {}", 
                                color.r, color.g, color.b, self.brush_size);
                            
                            // Store the active stroke
                            *self.active_stroke.borrow_mut() = Some(active_stroke);
                            
                            // Add the current mouse position as the first point
                            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                                active_stroke.add_point(self.mouse_position);
                                println!("Added first point to active stroke: ({}, {})", 
                                    self.mouse_position.x, self.mouse_position.y);
                                println!("Active stroke now has {} points", active_stroke.points().len());
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.mouse_state = MouseState::Idle;
                            
                            if let Some(active_stroke) = self.active_stroke.borrow_mut().take() {
                                let mut canvas = self.canvas.borrow_mut();
                                if let Err(e) = canvas.commit_stroke(active_stroke) {
                                    eprintln!("Failed to commit stroke: {}", e);
                                } else {
                                    println!("Stroke committed successfully");
                                }
                            }
                        }
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                self.mouse_position = point;
                
                // If we're drawing, add the point to the active stroke
                if self.mouse_state == MouseState::Drawing {
                    if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                        active_stroke.add_point(point);
                        println!("Added point to active stroke: ({}, {})", point.x, point.y);
                        println!("Active stroke now has {} points", active_stroke.points().len());
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    match key_event.logical_key.as_ref() {
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                            let mut palette = self.palette.borrow_mut();
                            let current_index = palette.selected_index();
                            let new_index = (current_index + 1) % palette.color_count();
                            if let Err(e) = palette.select_color(new_index) {
                                eprintln!("Failed to change color: {}", e);
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                            let mut palette = self.palette.borrow_mut();
                            let current_index = palette.selected_index();
                            let new_index = if current_index == 0 {
                                palette.color_count() - 1
                            } else {
                                current_index - 1
                            };
                            if let Err(e) = palette.select_color(new_index) {
                                eprintln!("Failed to change color: {}", e);
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl WindowBackend for WindowApp {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialization happens in resumed()
        Ok(())
    }

    fn run(&self) {
        // This is handled by the event loop in main
    }

    fn canvas(&self) -> &Rc<RefCell<Canvas>> {
        &self.canvas
    }

    fn palette(&self) -> &Rc<RefCell<ColorPalette>> {
        &self.palette
    }

    fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    fn mouse_state(&self) -> BackendMouseState {
        match self.mouse_state {
            MouseState::Idle => BackendMouseState::Idle,
            MouseState::Drawing => BackendMouseState::Drawing,
        }
    }

    fn has_active_stroke(&self) -> bool {
        self.active_stroke.borrow().is_some()
    }

    fn active_stroke_point_count(&self) -> usize {
        self.active_stroke.borrow().as_ref().map_or(0, |stroke| stroke.points().len())
    }
}

/// Run the winit event loop with the window application
pub fn run_window_app() {
    logger::info("Starting winit event loop...");
    
    let event_loop = EventLoop::new().unwrap();
    let mut app = WindowApp::new();
    
    event_loop.run_app(&mut app).unwrap();
    
    logger::info("Rancer application closed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_app_creation() {
        let app = WindowApp::new();
        assert_eq!(app.mouse_state, MouseState::Idle);
        assert_eq!(app.brush_size, 3.0);
    }

    #[test]
    fn test_mouse_state_transitions() {
        let mut mouse_state = MouseState::Idle;
        assert_eq!(mouse_state, MouseState::Idle);
        
        mouse_state = MouseState::Drawing;
        assert_eq!(mouse_state, MouseState::Drawing);
    }
}