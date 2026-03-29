//! Window management module for Rancer using GTK4
//!
//! Provides window creation, mouse input handling, and OpenGL-accelerated rendering using GTK4.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.
//!
//! GTK4 is used for Linux/Wayland compatibility with OpenGL rendering via GLArea.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, EventControllerMotion, GLArea, GestureClick};
use std::cell::RefCell;
use std::rc::Rc;

use crate::canvas::{ActiveStroke, Canvas, ColorPalette, Point};
use crate::logger;
use crate::opengl_renderer::GlRenderer;
use crate::preferences::Preferences;
use crate::window_backend::{MouseState as BackendMouseState, WindowBackend};

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

            // Create GLArea for OpenGL rendering
            let gl_area = GLArea::builder().hexpand(true).vexpand(true).build();

            window.set_child(Some(&gl_area));

            logger::info("GTK4 window created successfully");
            logger::info("Window size: 1280x720");
            logger::info("Window title: Rancer");
            logger::info("========================");

            // Shared state
            let brush_size_shared = Rc::new(RefCell::new(brush_size));
            let gl_renderer: Rc<RefCell<Option<GlRenderer>>> = Rc::new(RefCell::new(None));

            // Set up mouse event handlers
            setup_mouse_events(
                &gl_area,
                &canvas,
                &palette,
                &active_stroke,
                mouse_state,
                mouse_position,
                brush_size_shared.clone(),
            );

            // Set up OpenGL render callback
            let gl_renderer_clone = gl_renderer.clone();
            let canvas_clone = canvas.clone();
            let palette_clone = palette.clone();
            let active_stroke_clone = active_stroke.clone();
            let brush_size_clone = brush_size_shared;

            gl_area.connect_render(move |gl_area, _context| {
                // Ensure GL context is current before any GL operations
                gl_area.make_current();

                let _gl_context = match gl_area.context() {
                    Some(ctx) => ctx,
                    None => {
                        logger::warn("GLArea has no context, skipping render");
                        return glib::Propagation::Stop;
                    }
                };

                // Lazily initialize the GL renderer on first render
                if gl_renderer_clone.borrow().is_none() {
                    if let Err(e) = crate::gl_loader::init_gl_library() {
                        logger::error(&format!("Failed to initialize GL library: {e}"));
                        return glib::Propagation::Stop;
                    }

                    let gl = Rc::new(unsafe {
                        glow::Context::from_loader_function(crate::gl_loader::create_gl_loader())
                    });
                    match GlRenderer::new(gl) {
                        Ok(renderer) => {
                            logger::info("OpenGL renderer initialized successfully");
                            *gl_renderer_clone.borrow_mut() = Some(renderer);
                        }
                        Err(e) => {
                            logger::error(&format!("Failed to initialize OpenGL renderer: {e}"));
                            return glib::Propagation::Stop;
                        }
                    }
                }

                if let Some(ref renderer) = *gl_renderer_clone.borrow() {
                    let canvas = canvas_clone.borrow();
                    let palette = palette_clone.borrow();
                    let active_stroke = active_stroke_clone.borrow();
                    let current_brush_size = *brush_size_clone.borrow();
                    let width = gl_area.width();
                    let height = gl_area.height();

                    renderer.render(
                        &canvas,
                        &palette,
                        &active_stroke,
                        current_brush_size,
                        width,
                        height,
                    );
                }

                glib::Propagation::Proceed
            });

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
        self.active_stroke
            .borrow()
            .as_ref()
            .map_or(0, |stroke| stroke.points().len())
    }
}

/// Set up mouse event handlers for the GLArea
fn setup_mouse_events(
    gl_area: &GLArea,
    canvas: &Rc<RefCell<Canvas>>,
    palette: &Rc<RefCell<ColorPalette>>,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
    mouse_state: MouseState,
    mouse_position: Point,
    brush_size: Rc<RefCell<f32>>,
) {
    // Mouse click handler
    let click_gesture = GestureClick::new();
    click_gesture.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let canvas_clone = canvas.clone();
    let palette_clone = palette.clone();
    let active_stroke_clone = active_stroke.clone();
    let mouse_state_clone = Rc::new(RefCell::new(mouse_state));
    let mouse_position_clone = Rc::new(RefCell::new(mouse_position));
    let brush_size_clone = brush_size.clone();

    // Clone Rc's for use in other closures
    let mouse_state_clone2 = mouse_state_clone.clone();
    let mouse_position_clone2 = mouse_position_clone.clone();

    click_gesture.connect_pressed(move |gesture, _n_press, x, y| {
        let point = Point {
            x: x as f32,
            y: y as f32,
        };
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
                let color_x = (palette_x + (color_width + spacing) * i as f32) as f64;
                let color_width_f64 = color_width as f64;
                if x >= color_x && x <= color_x + color_width_f64 {
                    if let Err(e) = palette_clone.borrow_mut().select_color(i) {
                        eprintln!("Failed to select color: {e}");
                    } else {
                        println!("Selected color at index {i}");
                    }
                    if let Some(widget) = gesture.widget() {
                        if let Some(gl_area) = widget.downcast_ref::<GLArea>() {
                            gl_area.queue_render();
                        }
                    }
                    return;
                }
            }
        } else if y >= 50.0 && y <= 80.0 {
            // Brush size selector area
            let selector_x = 10.0;
            let button_size = 30.0;
            let spacing = 10.0;

            for (i, &size) in BRUSH_SIZES.iter().enumerate() {
                let button_x = (selector_x + (button_size + spacing) * i as f32) as f64;
                let button_size_f64 = button_size as f64;
                if x >= button_x && x <= button_x + button_size_f64 {
                    *brush_size_clone.borrow_mut() = size;
                    println!("Selected brush size: {size}");
                    if let Some(widget) = gesture.widget() {
                        if let Some(gl_area) = widget.downcast_ref::<GLArea>() {
                            gl_area.queue_render();
                        }
                    }
                    return;
                }
            }
        }

        // If not on UI, start drawing
        let color = palette_clone.borrow().current_color();
        let current_brush_size = *brush_size_clone.borrow();
        let mut canvas = canvas_clone.borrow_mut();
        let active_stroke =
            canvas.begin_stroke_with_palette(&palette_clone.borrow(), current_brush_size, 1.0);
        println!(
            "Created active stroke with color RGB({}, {}, {}) and width {}",
            color.r, color.g, color.b, current_brush_size
        );

        *active_stroke_clone.borrow_mut() = Some(active_stroke);

        if let Some(active_stroke) = &mut *active_stroke_clone.borrow_mut() {
            active_stroke.add_point(point);
            println!(
                "Added first point to active stroke: ({}, {})",
                point.x, point.y
            );
            println!(
                "Active stroke now has {} points",
                active_stroke.points().len()
            );
        }
    });

    gl_area.add_controller(click_gesture);

    // Mouse release handler
    let click_gesture_release = GestureClick::new();
    click_gesture_release.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let active_stroke_clone2 = active_stroke.clone();
    let canvas_clone2 = canvas.clone();
    let mouse_state_clone3 = mouse_state_clone2.clone();

    click_gesture_release.connect_released(move |_gesture, _n_press, _x, _y| {
        *mouse_state_clone3.borrow_mut() = MouseState::Idle;

        if let Some(active_stroke) = active_stroke_clone2.borrow_mut().take() {
            let mut canvas = canvas_clone2.borrow_mut();
            if let Err(e) = canvas.commit_stroke(active_stroke) {
                eprintln!("Failed to commit stroke: {e}");
            } else {
                println!("Stroke committed successfully");
            }
        }
    });

    gl_area.add_controller(click_gesture_release);

    // Mouse motion handler
    let motion_controller = EventControllerMotion::new();

    let active_stroke_clone3 = active_stroke.clone();
    let mouse_state_clone4 = mouse_state_clone2.clone();
    let mouse_position_clone3 = mouse_position_clone2.clone();

    motion_controller.connect_motion(move |controller, x, y| {
        let point = Point {
            x: x as f32,
            y: y as f32,
        };
        *mouse_position_clone3.borrow_mut() = point;

        if *mouse_state_clone4.borrow() == MouseState::Drawing {
            if let Some(active_stroke) = &mut *active_stroke_clone3.borrow_mut() {
                active_stroke.add_point(point);
                println!(
                    "Active stroke now has {} points",
                    active_stroke.points().len()
                );
                if let Some(widget) = controller.widget() {
                    if let Some(gl_area) = widget.downcast_ref::<GLArea>() {
                        gl_area.queue_render();
                    }
                }
            }
        }
    });

    gl_area.add_controller(motion_controller);
}

/// Run the GTK4 window application
pub fn run_window_app(preferences: Preferences) {
    logger::info("Starting GTK4 window application...");

    let mut app = WindowApp::new(preferences);

    if let Err(e) = app.init() {
        logger::error(&format!("Failed to initialize GTK4 window: {e}"));
        return;
    }

    app.run();
}
