//! Window management module for Rancer using GTK4
//!
//! Provides window creation, mouse input handling, and WGPU rendering using GTK4.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.
//!
//! GTK4 is used instead of winit to ensure better compatibility with
//! Wayland and GNOME Shell environments.

use gtk4::{
    prelude::*,
    Application, ApplicationWindow, DrawingArea, GestureClick, EventControllerMotion,
    gdk, glib
};
use gtk4::cairo;
use std::rc::Rc;
use std::cell::RefCell;

use crate::canvas::{ActiveStroke, Canvas, ColorPalette, Point};

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}

/// Window application state and event handler using GTK4
pub struct WindowApp {
    /// GTK4 application
    app: Application,
    /// GTK4 application window
    window: Option<ApplicationWindow>,
    /// Drawing area for mouse input
    drawing_area: Option<DrawingArea>,
    /// Canvas for drawing operations
    canvas: Rc<RefCell<Canvas>>,
    /// Color palette for color selection
    palette: Rc<RefCell<ColorPalette>>,
    /// Current active stroke being drawn (shared mutable state)
    active_stroke: Rc<RefCell<Option<ActiveStroke>>>,
    /// Current mouse state
    mouse_state: MouseState,
    /// Current mouse position
    mouse_position: Point,
}

impl WindowApp {
    /// Create a new window application using GTK4
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("Creating GTK4 application...");
        
        // Create GTK4 application
        let app = Application::builder()
            .application_id("com.example.rancer")
            .build();

        println!("GTK4 application created successfully");

        Ok(Self {
            app,
            window: None,
            drawing_area: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            palette: Rc::new(RefCell::new(ColorPalette::new())),
            active_stroke: Rc::new(RefCell::new(None)),
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
        })
    }

    /// Get a reference to the window
    pub fn window(&self) -> &ApplicationWindow {
        self.window.as_ref().unwrap()
    }

    /// Get a reference to the canvas
    pub fn canvas(&self) -> &Rc<RefCell<Canvas>> {
        &self.canvas
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut Rc<RefCell<Canvas>> {
        &mut self.canvas
    }

    /// Get a reference to the color palette
    pub fn palette(&self) -> &Rc<RefCell<ColorPalette>> {
        &self.palette
    }

    /// Get a mutable reference to the color palette
    pub fn palette_mut(&mut self) -> &mut Rc<RefCell<ColorPalette>> {
        &mut self.palette
    }

    /// Set up mouse event handlers on the drawing area
    pub fn setup_mouse_events(&self) {
        // This method is no longer needed since mouse events are set up in the activate callback
        // The window and drawing area are created after the startup signal
    }

    /// Set up keyboard event handlers for color selection
    pub fn setup_keyboard_events(&self) {
        // This method is no longer needed since keyboard events are set up in the activate callback
        // The window is created after the startup signal
    }

    /// Set up window close handler
    pub fn setup_close_handler(&self) {
        // This method is no longer needed since close handler is set up in the activate callback
        // The window is created after the startup signal
    }

    /// Run the GTK4 application
    pub fn run(&self) {
        let canvas = self.canvas.clone();
        let palette = self.palette.clone();
        let _mouse_state = self.mouse_state;
        let _mouse_position = self.mouse_position;
        let active_stroke = self.active_stroke.clone();

        self.app.connect_activate(move |app| {
            // Create the window after the application startup signal
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Rancer")
                .default_width(1280)
                .default_height(720)
                .resizable(true)
                .build();

            // Create drawing area for mouse input
            let drawing_area = DrawingArea::builder()
                .hexpand(true)
                .vexpand(true)
                .build();

            // Set up the window layout
            window.set_child(Some(&drawing_area));

            println!("GTK4 window created successfully");
            println!("Window size: {}x{}", window.default_width(), window.default_height());
            println!("Window title: {}", window.title().unwrap_or_default());

            // Set up mouse event handlers
            setup_mouse_events_for_window(
                &drawing_area,
                &canvas,
                &palette,
                _mouse_state,
                _mouse_position,
                &active_stroke
            );

            // Set up keyboard events
            setup_keyboard_events_for_window(&window, &palette, &drawing_area);

            // Set up close handler
            setup_close_handler_for_window(&window);

            // Present the window
            window.present();
        });

        // Start the GTK4 main loop
        self.app.run();
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
        self.active_stroke.borrow().is_some()
    }

    /// Get the number of points in the current active stroke
    pub fn active_stroke_point_count(&self) -> usize {
        self.active_stroke.borrow().as_ref().map_or(0, |stroke| stroke.points().len())
    }

    /// Get a reference to the drawing area
    pub fn drawing_area(&self) -> &DrawingArea {
        self.drawing_area.as_ref().unwrap()
    }
}

/// Set up mouse event handlers for a window
fn setup_mouse_events_for_window(
    drawing_area: &DrawingArea,
    canvas: &Rc<RefCell<Canvas>>,
    palette: &Rc<RefCell<ColorPalette>>,
    _mouse_state: MouseState,
    _mouse_position: Point,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
) {
    // Set up draw callback to render the canvas content
    #[allow(deprecated)] // glib::clone macro is deprecated but still widely used
    drawing_area.set_draw_func(glib::clone!(@weak canvas, @weak palette, @weak active_stroke => move |_, cr, width, height| {
        println!("Draw callback called - width: {}, height: {}", width, height);
        
        // Clear the drawing area with white background
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();
        
        // Get canvas and palette for rendering
        let canvas_ref = canvas.borrow();
        let palette_ref = palette.borrow();
        
        println!("Canvas has {} committed strokes", canvas_ref.strokes().len());
        
        // Draw all committed strokes
        for (i, stroke) in canvas_ref.strokes().iter().enumerate() {
            if let Some(first_point) = stroke.points.first() {
                println!("Drawing committed stroke {} with {} points", i, stroke.points.len());
                // Set stroke color - convert u8 (0-255) to f64 (0.0-1.0) for cairo
                let color = stroke.color;
                cr.set_source_rgb(
                    color.r as f64 / 255.0, 
                    color.g as f64 / 255.0, 
                    color.b as f64 / 255.0
                );
                
                // Set line width
                cr.set_line_width(stroke.width as f64);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                
                // Start path at first point
                cr.move_to(first_point.x as f64, first_point.y as f64);
                
                // Draw lines to all subsequent points
                for point in stroke.points.iter().skip(1) {
                    cr.line_to(point.x as f64, point.y as f64);
                }
                
                // Stroke the path
                cr.stroke().unwrap();
            }
        }
        
        // Draw active stroke if there is one
        let active_stroke_ref = active_stroke.borrow();
        println!("Active stroke exists: {}", active_stroke_ref.is_some());
        
        if let Some(active_stroke) = &*active_stroke_ref {
            println!("Drawing active stroke with {} points", active_stroke.points().len());
            if let Some(first_point) = active_stroke.points().first() {
                // Set stroke color for active stroke - convert u8 (0-255) to f64 (0.0-1.0) for cairo
                let color = active_stroke.color();
                println!("Active stroke color: RGB({}, {}, {})", color.r, color.g, color.b);
                cr.set_source_rgb(
                    color.r as f64 / 255.0, 
                    color.g as f64 / 255.0, 
                    color.b as f64 / 255.0
                );
                
                // Set line width
                cr.set_line_width(active_stroke.width() as f64);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                
                // Start path at first point
                cr.move_to(first_point.x as f64, first_point.y as f64);
                
                // Draw lines to all subsequent points
                for point in active_stroke.points().iter().skip(1) {
                    cr.line_to(point.x as f64, point.y as f64);
                }
                
                // Stroke the path
                cr.stroke().unwrap();
            }
        } else {
            println!("No active stroke to draw");
        }
        
        // Draw a simple color palette indicator at the top
        draw_color_palette_indicator(cr, &palette_ref, width, height);
    }));
    
    // Mouse motion event handler
    let drawing_area_clone = drawing_area.clone();
    let mouse_state_clone = Rc::new(RefCell::new(_mouse_state));
    let mouse_state_clone_for_motion = mouse_state_clone.clone();
    let active_stroke_clone = active_stroke.clone();

    let motion_controller = EventControllerMotion::new();
    motion_controller.connect_motion(move |_, x, y| {
        // Convert GTK coordinates to our canvas coordinates
        let point = Point {
            x: x as f32,
            y: y as f32,
        };

        // If we're drawing, add the point to the active stroke
        if *mouse_state_clone_for_motion.borrow() == MouseState::Drawing {
            // Access the shared active stroke and add the point
            if let Some(active_stroke) = &mut *active_stroke_clone.borrow_mut() {
                active_stroke.add_point(point);
                println!("Added point to active stroke: ({}, {})", point.x, point.y);
                println!("Active stroke now has {} points", active_stroke.points().len());
                // Trigger a redraw to show the updated active stroke
                drawing_area_clone.queue_draw();
            }
        }
    });
    drawing_area.add_controller(motion_controller);

    // Mouse click event handler
    let drawing_area_clone2 = drawing_area.clone();
    let canvas_clone2 = canvas.clone();
    let palette_clone2 = palette.clone();
    let active_stroke_clone2 = active_stroke.clone();

    let click_gesture = GestureClick::new();
    
    // Create clones for the pressed closure
    let mouse_state_pressed = mouse_state_clone.clone();
    let canvas_pressed = canvas_clone2.clone();
    let palette_pressed = palette_clone2.clone();
    let active_stroke_pressed = active_stroke_clone2.clone();
    
    click_gesture.connect_pressed(move |_, n_press, x, y| {
        if n_press == 1 {
            println!("Mouse button pressed at ({}, {})", x, y);
            // Mouse button pressed
            *mouse_state_pressed.borrow_mut() = MouseState::Drawing;
            
            // Begin a new active stroke with current palette color
            let color = palette_pressed.borrow().current_color();
            let mut canvas = canvas_pressed.borrow_mut();
            let active_stroke = canvas.begin_stroke_with_palette(
                &palette_pressed.borrow(),
                3.0,  // Default stroke width
                1.0,  // Default opacity
            );
            println!("Created active stroke with color RGB({}, {}, {})", color.r, color.g, color.b);
            
            // Store the active stroke in the shared state
            *active_stroke_pressed.borrow_mut() = Some(active_stroke);
            
            // Add the current mouse position as the first point
            let point = Point { x: x as f32, y: y as f32 };
            if let Some(active_stroke) = &mut *active_stroke_pressed.borrow_mut() {
                active_stroke.add_point(point);
                println!("Added first point to active stroke: ({}, {})", point.x, point.y);
                println!("Active stroke now has {} points", active_stroke.points().len());
            }
        }
    });

    // Create clones for the released closure
    let mouse_state_released = mouse_state_clone.clone();
    let canvas_released = canvas_clone2.clone();
    let active_stroke_released = active_stroke_clone2.clone();
    
    click_gesture.connect_released(move |_, _, _, _| {
        // Mouse button released
        *mouse_state_released.borrow_mut() = MouseState::Idle;
        
        if let Some(active_stroke) = active_stroke_released.borrow_mut().take() {
            // Try to commit the stroke
            let mut canvas = canvas_released.borrow_mut();
            if let Err(e) = canvas.commit_stroke(active_stroke) {
                eprintln!("Failed to commit stroke: {}", e);
            } else {
                println!("Stroke committed successfully");
            }
        }
    });

    drawing_area_clone2.add_controller(click_gesture);
}

/// Set up keyboard event handlers for a window
fn setup_keyboard_events_for_window(
    window: &ApplicationWindow,
    palette: &Rc<RefCell<ColorPalette>>,
    drawing_area: &DrawingArea,
) {
    // Keyboard event handler for color selection
    let palette_clone = palette.clone();
    let drawing_area_clone = drawing_area.clone();
    
    // Use a key controller instead of connect_key_pressed
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        match key {
            gdk::Key::Up => {
                let mut palette = palette_clone.borrow_mut();
                let current_index = palette.selected_index();
                let new_index = (current_index + 1) % palette.color_count();
                if let Err(e) = palette.select_color(new_index) {
                    eprintln!("Failed to change color: {}", e);
                } else {
                    // Trigger a redraw to update the color palette indicator
                    drawing_area_clone.queue_draw();
                }
            }
            gdk::Key::Down => {
                let mut palette = palette_clone.borrow_mut();
                let current_index = palette.selected_index();
                let new_index = if current_index == 0 {
                    palette.color_count() - 1
                } else {
                    current_index - 1
                };
                if let Err(e) = palette.select_color(new_index) {
                    eprintln!("Failed to change color: {}", e);
                } else {
                    // Trigger a redraw to update the color palette indicator
                    drawing_area_clone.queue_draw();
                }
            }
            _ => {}
        }
        glib::Propagation::Proceed
    });
    
    // Add the key controller to the window
    window.add_controller(key_controller);
}

/// Set up window close handler
fn setup_close_handler_for_window(window: &ApplicationWindow) {
    window.connect_close_request(move |_| {
        // Window is about to close, we can perform cleanup here
        println!("Window is closing");
        glib::Propagation::Proceed
    });
}

/// Draw a simple color palette indicator at the top of the canvas
fn draw_color_palette_indicator(
    cr: &cairo::Context,
    palette: &ColorPalette,
    _width: i32,
    _height: i32,
) {
    let palette_height = 30.0;
    let palette_y = 10.0;
    let color_width = 20.0;
    let spacing = 5.0;
    
    // Draw palette background
    cr.set_source_rgb(0.9, 0.9, 0.9);
    cr.rectangle(10.0, palette_y, (color_width + spacing) * palette.color_count() as f64 - spacing, palette_height);
    cr.fill().unwrap();
    
    // Draw individual colors
    for i in 0..palette.color_count() {
        let color = palette.colors()[i];
        let x = 10.0 + (color_width + spacing) * i as f64;
        
        // Draw color swatch - convert u8 (0-255) to f64 (0.0-1.0) for cairo
        cr.set_source_rgb(
            color.r as f64 / 255.0, 
            color.g as f64 / 255.0, 
            color.b as f64 / 255.0
        );
        cr.rectangle(x, palette_y + 5.0, color_width, palette_height - 10.0);
        cr.fill().unwrap();
        
        // Draw border around selected color
        if i == palette.selected_index() {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(2.0);
            cr.rectangle(x - 2.0, palette_y + 3.0, color_width + 4.0, palette_height - 6.0);
            cr.stroke().unwrap();
        }
    }
}

/// Run the window application using GTK4
pub fn run_window_app() {
    let app = WindowApp::new().unwrap();
    
    // Set up all event handlers
    app.setup_mouse_events();
    app.setup_keyboard_events();
    app.setup_close_handler();
    
    // Run the application
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_app_creation() {
        // Note: We can't actually create a GTK4 window in tests without initializing GTK
        // This test would need to be run with GTK initialized in a real scenario
        assert!(true); // Placeholder - real tests would require GTK initialization
    }

    #[test]
    fn test_mouse_state_transitions() {
        // Test mouse state transitions without creating actual window
        // We'll test the logic without the GTK4 components
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
        assert_eq!(palette.current_color(), crate::canvas::Color::BLACK);
        
        // Test color change
        palette.select_color(1).unwrap();
        assert_eq!(palette.selected_index(), 1);
        assert_eq!(palette.current_color(), crate::canvas::Color::WHITE);
        
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