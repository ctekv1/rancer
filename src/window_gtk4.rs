//! Window management module for Rancer using GTK4
//!
//! Provides window creation, mouse input handling, and WGPU rendering using GTK4.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.
//!
//! GTK4 is used for Linux/Wayland compatibility.

use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea, GestureClick, GestureDrag, gdk};
use gtk4::glib;

use crate::canvas::{ActiveStroke, Canvas, ColorPalette, Point};
use crate::renderer::{Renderer, RendererConfig};
use crate::logger;
use crate::preferences::Preferences;
use crate::window_backend::{WindowBackend, MouseState as BackendMouseState};

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}

/// Brush size options
const BRUSH_SIZES: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

/// Window application state using GTK4
pub struct WindowApp {
    /// The GTK4 application
    app: Application,
    /// The GTK4 application window
    window: Option<ApplicationWindow>,
    /// The drawing area
    drawing_area: Option<DrawingArea>,
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
    /// User preferences
    preferences: Preferences,
}

impl WindowApp {
    /// Create a new window application
    pub fn new(preferences: Preferences) -> Self {
        logger::info("Creating GTK4 window application...");
        
        let app = Application::builder()
            .application_id("com.example.rancer")
            .build();
        
        Self {
            app,
            window: None,
            drawing_area: None,
            renderer: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            palette: Rc::new(RefCell::new(ColorPalette::new())),
            active_stroke: Rc::new(RefCell::new(None)),
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
            brush_size: preferences.brush.default_size,
            preferences,
        }
    }
}

impl WindowBackend for WindowApp {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        logger::info("=== WINDOW CREATION ===");
        
        let canvas = self.canvas.clone();
        let palette = self.palette.clone();
        let active_stroke = self.active_stroke.clone();
        let mouse_state = self.mouse_state;
        let mouse_position = self.mouse_position;
        let brush_size = self.brush_size;
        
        self.app.connect_activate(move |app| {
            // Create the window
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Rancer")
                .default_width(1280)
                .default_height(720)
                .build();
            
            // Create drawing area
            let drawing_area = DrawingArea::builder()
                .hexpand(true)
                .vexpand(true)
                .build();
            
            window.set_child(Some(&drawing_area));
            
            logger::info("GTK4 window created successfully");
            logger::info("Window size: 1280x720");
            logger::info("Window title: Rancer");
            logger::info("========================");
            
            // Initialize WGPU renderer
            logger::info("=== RENDERER INITIALIZATION ===");
            
            // Note: GTK4 doesn't easily expose raw window handles
            // For now, we'll log this limitation
            logger::info("⚠️  GTK4 window handle integration needed for WGPU");
            logger::info("   For full WGPU support, use winit backend on Linux");
            logger::info("===============================");
            
            // Set up mouse event handlers
            setup_mouse_events(&drawing_area, &canvas, &palette, &active_stroke, mouse_state, mouse_position, brush_size);
            
            // Set up draw callback
            setup_draw_callback(&drawing_area, &canvas, &palette, &active_stroke);
            
            window.present();
        });
        
        Ok(())
    }
    
    fn run(&self) {
        logger::info("Starting GTK4 event loop...");
        self.app.run();
        logger::info("Rancer application closed successfully");
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

/// Set up mouse event handlers for the drawing area
fn setup_mouse_events(
    drawing_area: &DrawingArea,
    canvas: &Rc<RefCell<Canvas>>,
    palette: &Rc<RefCell<ColorPalette>>,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
    mouse_state: MouseState,
    mouse_position: Point,
    brush_size: f32,
) {
    // Mouse click handler
    let click_gesture = GestureClick::new();
    click_gesture.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
    
    let canvas_clone = canvas.clone();
    let palette_clone = palette.clone();
    let active_stroke_clone = active_stroke.clone();
    let mouse_state_clone = Rc::new(RefCell::new(mouse_state));
    let mouse_position_clone = Rc::new(RefCell::new(mouse_position));
    let brush_size_clone = Rc::new(RefCell::new(brush_size));
    
    click_gesture.connect_pressed(move |gesture, _n_press, x, y| {
        let point = Point { x: x as f32, y: y as f32 };
        *mouse_position_clone.borrow_mut() = point;
        *mouse_state_clone.borrow_mut() = MouseState::Drawing;
        
        // Check if click is on UI elements
        if y >= 10.0 && y <= 30.0 {
            // Color palette area
            let palette_x = 10.0;
            let color_width = 20.0;
            let spacing = 5.0;
            let color_count = palette_clone.borrow().color_count();
            
            for i in 0..color_count {
                let color_x = palette_x + (color_width + spacing) * i as f32;
                if x >= color_x && x <= color_x + color_width {
                    if let Err(e) = palette_clone.borrow_mut().select_color(i) {
                        eprintln!("Failed to select color: {}", e);
                    } else {
                        println!("Selected color at index {}", i);
                    }
                    gesture.widget().queue_draw();
                    return;
                }
            }
        } else if y >= 50.0 && y <= 80.0 {
            // Brush size selector area
            let selector_x = 10.0;
            let button_size = 30.0;
            let spacing = 10.0;
            
            for (i, &size) in BRUSH_SIZES.iter().enumerate() {
                let button_x = selector_x + (button_size + spacing) * i as f32;
                if x >= button_x && x <= button_x + button_size {
                    *brush_size_clone.borrow_mut() = size;
                    println!("Selected brush size: {}", size);
                    gesture.widget().queue_draw();
                    return;
                }
            }
        }
        
        // If not on UI, start drawing
        let color = palette_clone.borrow().current_color();
        let current_brush_size = *brush_size_clone.borrow();
        let mut canvas = canvas_clone.borrow_mut();
        let active_stroke = canvas.begin_stroke_with_palette(
            &palette_clone.borrow(),
            current_brush_size,
            1.0,
        );
        println!("Created active stroke with color RGB({}, {}, {}) and width {}", 
            color.r, color.g, color.b, current_brush_size);
        
        *active_stroke_clone.borrow_mut() = Some(active_stroke);
        
        if let Some(active_stroke) = &mut *active_stroke_clone.borrow_mut() {
            active_stroke.add_point(point);
            println!("Added first point to active stroke: ({}, {})", point.x, point.y);
            println!("Active stroke now has {} points", active_stroke.points().len());
        }
    });
    
    drawing_area.add_controller(click_gesture);
    
    // Mouse release handler
    let click_gesture_release = GestureClick::new();
    click_gesture_release.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
    
    let active_stroke_clone2 = active_stroke.clone();
    let canvas_clone2 = canvas.clone();
    let mouse_state_clone2 = mouse_state_clone.clone();
    
    click_gesture_release.connect_released(move |_gesture, _n_press, _x, _y| {
        *mouse_state_clone2.borrow_mut() = MouseState::Idle;
        
        if let Some(active_stroke) = active_stroke_clone2.borrow_mut().take() {
            let mut canvas = canvas_clone2.borrow_mut();
            if let Err(e) = canvas.commit_stroke(active_stroke) {
                eprintln!("Failed to commit stroke: {}", e);
            } else {
                println!("Stroke committed successfully");
            }
        }
    });
    
    drawing_area.add_controller(click_gesture_release);
    
    // Mouse motion handler
    let drag_gesture = GestureDrag::new();
    
    let active_stroke_clone3 = active_stroke.clone();
    let mouse_state_clone3 = mouse_state_clone.clone();
    let mouse_position_clone2 = mouse_position_clone.clone();
    
    drag_gesture.connect_drag_move(move |gesture, x, y| {
        let point = Point { x: x as f32, y: y as f32 };
        *mouse_position_clone2.borrow_mut() = point;
        
        if *mouse_state_clone3.borrow() == MouseState::Drawing {
            if let Some(active_stroke) = &mut *active_stroke_clone3.borrow_mut() {
                active_stroke.add_point(point);
                println!("Added point to active stroke: ({}, {})", point.x, point.y);
                println!("Active stroke now has {} points", active_stroke.points().len());
                gesture.widget().queue_draw();
            }
        }
    });
    
    drawing_area.add_controller(drag_gesture);
}

/// Set up draw callback for the drawing area
fn setup_draw_callback(
    drawing_area: &DrawingArea,
    canvas: &Rc<RefCell<Canvas>>,
    palette: &Rc<RefCell<ColorPalette>>,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
) {
    let canvas_clone = canvas.clone();
    let palette_clone = palette.clone();
    let active_stroke_clone = active_stroke.clone();
    
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        // Clear background with white
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint().unwrap();
        
        let canvas = canvas_clone.borrow();
        let palette = palette_clone.borrow();
        let active_stroke = active_stroke_clone.borrow();
        
        // Draw committed strokes
        for stroke in canvas.strokes() {
            if let Some(first_point) = stroke.points.first() {
                cr.set_source_rgb(
                    stroke.color.r as f64 / 255.0,
                    stroke.color.g as f64 / 255.0,
                    stroke.color.b as f64 / 255.0,
                );
                cr.set_line_width(stroke.width as f64);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                
                cr.move_to(first_point.x as f64, first_point.y as f64);
                for point in stroke.points.iter().skip(1) {
                    cr.line_to(point.x as f64, point.y as f64);
                }
                cr.stroke().unwrap();
            }
        }
        
        // Draw active stroke
        if let Some(active_stroke) = &*active_stroke {
            if let Some(first_point) = active_stroke.points().first() {
                cr.set_source_rgb(
                    active_stroke.color().r as f64 / 255.0,
                    active_stroke.color().g as f64 / 255.0,
                    active_stroke.color().b as f64 / 255.0,
                );
                cr.set_line_width(active_stroke.width() as f64);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                
                cr.move_to(first_point.x as f64, first_point.y as f64);
                for point in active_stroke.points().iter().skip(1) {
                    cr.line_to(point.x as f64, point.y as f64);
                }
                cr.stroke().unwrap();
            }
        }
        
        // Draw color palette
        draw_color_palette(cr, &palette, width, height);
        
        // Draw brush size selector
        draw_brush_size_selector(cr, 3.0, width, height); // Default brush size for now
    });
}

/// Draw color palette
fn draw_color_palette(cr: &cairo::Context, palette: &ColorPalette, _width: i32, _height: i32) {
    let colors = palette.colors();
    let palette_x = 10.0;
    let palette_y = 10.0;
    let color_width = 20.0;
    let color_height = 20.0;
    let spacing = 5.0;
    
    for (i, color) in colors.iter().enumerate() {
        let x = palette_x + (color_width + spacing) * i as f64;
        
        // Draw color swatch
        cr.set_source_rgb(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
        );
        cr.rectangle(x, palette_y, color_width, color_height);
        cr.fill().unwrap();
        
        // Draw border for selected color
        if i == palette.selected_index() {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(2.0);
            cr.rectangle(x - 2.0, palette_y - 2.0, color_width + 4.0, color_height + 4.0);
            cr.stroke().unwrap();
        }
    }
}

/// Draw brush size selector
fn draw_brush_size_selector(cr: &cairo::Context, selected_size: f32, _width: i32, _height: i32) {
    let selector_x = 10.0;
    let selector_y = 50.0;
    let button_size = 30.0;
    let spacing = 10.0;
    
    for (i, &size) in BRUSH_SIZES.iter().enumerate() {
        let x = selector_x + (button_size + spacing) * i as f64;
        
        // Draw button background
        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.rectangle(x, selector_y, button_size, button_size);
        cr.fill().unwrap();
        
        // Draw brush size indicator
        let indicator_size = (size as f64).min(button_size - 4.0);
        let indicator_x = x + (button_size - indicator_size) / 2.0;
        let indicator_y = selector_y + (button_size - indicator_size) / 2.0;
        
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.rectangle(indicator_x, indicator_y, indicator_size, indicator_size);
        cr.fill().unwrap();
        
        // Draw border for selected size
        if (size - selected_size).abs() < 0.1 {
            cr.set_source_rgb(0.0, 0.0, 1.0);
            cr.set_line_width(2.0);
            cr.rectangle(x - 2.0, selector_y - 2.0, button_size + 4.0, button_size + 4.0);
            cr.stroke().unwrap();
        }
    }
}

/// Run the GTK4 window application
pub fn run_window_app(preferences: Preferences) {
    logger::info("Starting GTK4 window application...");
    
    let mut app = WindowApp::new(preferences);
    
    if let Err(e) = app.init() {
        logger::error(&format!("Failed to initialize GTK4 window: {}", e));
        return;
    }
    
    app.run();
}