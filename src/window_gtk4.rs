//! Window management module for Rancer using GTK4
//!
//! Provides window creation, mouse input handling, and OpenGL-accelerated rendering using GTK4.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.
//!
//! GTK4 is used for Linux/Wayland compatibility with OpenGL rendering via GLArea.

use gtk4::gdk::ModifierType;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, EventControllerKey, EventControllerMotion, GLArea, GestureClick,
};
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
#[allow(dead_code)]
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
    /// Eraser mode active
    is_eraser: bool,
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

        let mut palette = ColorPalette::new();
        let _ = palette.select_color(preferences.palette.selected_index);

        Self {
            app,
            window: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            palette: Rc::new(RefCell::new(palette)),
            active_stroke: Rc::new(RefCell::new(None)),
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
            brush_size: preferences.brush.default_size,
            is_eraser: false,
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
        let is_eraser = self.is_eraser;
        let preferences = Rc::new(RefCell::new(self.preferences.clone()));

        self.app.connect_activate(move |app| {
            let prefs = preferences.borrow();

            // Create the window using preferences
            let window = ApplicationWindow::builder()
                .application(app)
                .title(&prefs.window.title)
                .default_width(prefs.window.width as i32)
                .default_height(prefs.window.height as i32)
                .build();

            logger::info(&format!(
                "Window size: {}x{}",
                prefs.window.width, prefs.window.height
            ));
            logger::info(&format!("Window title: {}", prefs.window.title));
            logger::info("========================");

            // Drop borrow before moving preferences into closures
            drop(prefs);

            // Create GLArea for OpenGL rendering
            let gl_area = GLArea::builder().hexpand(true).vexpand(true).build();
            gl_area.set_focusable(true);

            window.set_child(Some(&gl_area));

            // Shared state
            let brush_size_shared = Rc::new(RefCell::new(brush_size));
            let is_eraser_shared = Rc::new(RefCell::new(is_eraser));
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
                is_eraser_shared.clone(),
                preferences.clone(),
            );

            // Set up keyboard handler
            let key_controller = EventControllerKey::new();
            let canvas_kb = canvas.clone();
            let palette_kb = palette.clone();
            let prefs_kb = preferences.clone();
            let gl_area_kb = gl_area.clone();

            key_controller.connect_key_pressed(move |_controller, key, _keycode, state| {
                match key {
                    gtk4::gdk::Key::s | gtk4::gdk::Key::S => {
                        // Export canvas to PNG
                        let canvas_ref = canvas_kb.borrow();
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let filename = format!("rancer_export_{timestamp}.png");
                        let export_path = dirs::picture_dir()
                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                            .join(filename);

                        logger::info(&format!("Exporting canvas to: {export_path:?}"));

                        match crate::export::export_to_png(&canvas_ref, &export_path) {
                            Ok(_) => {
                                logger::info(&format!("Export successful: {export_path:?}"));
                                println!("Exported to: {export_path:?}");
                            }
                            Err(e) => {
                                logger::error(&format!("Export failed: {e}"));
                                eprintln!("Export failed: {e}");
                            }
                        }
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Up => {
                        // Navigate color palette
                        let mut palette = palette_kb.borrow_mut();
                        let current = palette.selected_index();
                        let new_index = (current + 1) % palette.color_count();
                        let _ = palette.select_color(new_index);
                        let mut prefs = prefs_kb.borrow_mut();
                        prefs.palette.selected_index = new_index;
                        let _ = crate::preferences::save(&prefs);
                        gl_area_kb.queue_render();
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Down => {
                        // Navigate color palette
                        let mut palette = palette_kb.borrow_mut();
                        let current = palette.selected_index();
                        let count = palette.color_count();
                        let new_index = if current == 0 { count - 1 } else { current - 1 };
                        let _ = palette.select_color(new_index);
                        let mut prefs = prefs_kb.borrow_mut();
                        prefs.palette.selected_index = new_index;
                        let _ = crate::preferences::save(&prefs);
                        gl_area_kb.queue_render();
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::z | gtk4::gdk::Key::Z => {
                        if state.contains(ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK) {
                            // Ctrl+Shift+Z: Redo
                            let mut canvas = canvas_kb.borrow_mut();
                            if canvas.can_redo() {
                                canvas.redo();
                                logger::info("Redo: restored last undone stroke");
                                println!("Redo: restored last undone stroke");
                                gl_area_kb.queue_render();
                            }
                            glib::Propagation::Stop
                        } else if state.contains(ModifierType::CONTROL_MASK) {
                            // Ctrl+Z: Undo
                            let mut canvas = canvas_kb.borrow_mut();
                            if canvas.can_undo() {
                                canvas.undo();
                                logger::info("Undo: removed last stroke");
                                println!("Undo: removed last stroke");
                                gl_area_kb.queue_render();
                            }
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    gtk4::gdk::Key::y | gtk4::gdk::Key::Y => {
                        if state.contains(ModifierType::CONTROL_MASK) {
                            // Ctrl+Y: Redo (alternative)
                            let mut canvas = canvas_kb.borrow_mut();
                            if canvas.can_redo() {
                                canvas.redo();
                                logger::info("Redo: restored last undone stroke");
                                println!("Redo: restored last undone stroke");
                                gl_area_kb.queue_render();
                            }
                            glib::Propagation::Stop
                        } else {
                            glib::Propagation::Proceed
                        }
                    }
                    gtk4::gdk::Key::Delete => {
                        if state.contains(ModifierType::CONTROL_MASK) {
                            // Ctrl+Delete: Clear canvas
                            let mut canvas = canvas_kb.borrow_mut();
                            canvas.clear();
                            logger::info("Canvas cleared");
                            gl_area_kb.queue_render();
                        }
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            });

            gl_area.add_controller(key_controller);

            // Set up OpenGL render callback
            let gl_renderer_clone = gl_renderer.clone();
            let canvas_clone = canvas.clone();
            let palette_clone = palette.clone();
            let active_stroke_clone = active_stroke.clone();
            let brush_for_render = brush_size_shared.clone();
            let is_eraser_for_render = is_eraser_shared.clone();

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
                    let current_brush_size = *brush_for_render.borrow();
                    let is_eraser = *is_eraser_for_render.borrow();
                    let width = gl_area.width();
                    let height = gl_area.height();

                    renderer.render(
                        &canvas,
                        &palette,
                        &active_stroke,
                        current_brush_size,
                        is_eraser,
                        width,
                        height,
                    );
                }

                glib::Propagation::Proceed
            });

            // Save preferences on window close
            let prefs_close = preferences.clone();
            let palette_close = palette.clone();
            let brush_close = brush_size_shared.clone();
            window.connect_close_request(move |_window| {
                let mut prefs = prefs_close.borrow_mut();
                prefs.palette.selected_index = palette_close.borrow().selected_index();
                prefs.brush.default_size = *brush_close.borrow();

                if let Err(e) = crate::preferences::save(&prefs) {
                    logger::error(&format!("Failed to save preferences on close: {e}"));
                }

                glib::Propagation::Proceed
            });

            window.present();
            gl_area.grab_focus();
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
#[allow(clippy::too_many_arguments)]
fn setup_mouse_events(
    gl_area: &GLArea,
    canvas: &Rc<RefCell<Canvas>>,
    palette: &Rc<RefCell<ColorPalette>>,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
    mouse_state: MouseState,
    mouse_position: Point,
    brush_size: Rc<RefCell<f32>>,
    is_eraser: Rc<RefCell<bool>>,
    preferences: Rc<RefCell<crate::preferences::Preferences>>,
) {
    // Mouse click handler for left button
    let click_gesture = GestureClick::new();
    click_gesture.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let canvas_clone = canvas.clone();
    let palette_clone = palette.clone();
    let active_stroke_clone = active_stroke.clone();
    let mouse_state_clone = Rc::new(RefCell::new(mouse_state));
    let mouse_position_clone = Rc::new(RefCell::new(mouse_position));
    let brush_size_clone = brush_size.clone();
    let is_eraser_clone = is_eraser.clone();
    let prefs_clone = preferences.clone();

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
        if (10.0..=30.0).contains(&y) {
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
                        let mut prefs = prefs_clone.borrow_mut();
                        prefs.palette.selected_index = i;
                        if let Err(e) = crate::preferences::save(&prefs) {
                            eprintln!("Failed to save preferences: {e}");
                        }
                    }
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                    return;
                }
            }
        } else if (50.0..=80.0).contains(&y) {
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
                    let mut prefs = prefs_clone.borrow_mut();
                    prefs.brush.default_size = size;
                    if let Err(e) = crate::preferences::save(&prefs) {
                        eprintln!("Failed to save preferences: {e}");
                    }
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                    return;
                }
            }
        } else if (85.0..=115.0).contains(&y) && (10.0..=40.0).contains(&x) {
            // Eraser button area - toggle eraser
            let mut is_eraser = is_eraser_clone.borrow_mut();
            *is_eraser = !*is_eraser;
            println!("Eraser mode: {}", if *is_eraser { "ON" } else { "OFF" });
            if let Some(widget) = gesture.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        } else if (85.0..=115.0).contains(&y) && (50.0..=80.0).contains(&x) {
            // Clear button area - clear canvas
            canvas_clone.borrow_mut().clear();
            logger::info("Canvas cleared");
            println!("Canvas cleared");
            if let Some(widget) = gesture.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        }

        // If not on UI, start drawing
        let is_eraser = *is_eraser_clone.borrow();
        let color = if is_eraser {
            crate::canvas::Color::WHITE
        } else {
            palette_clone.borrow().current_color()
        };
        let current_brush_size = *brush_size_clone.borrow();
        let mut canvas = canvas_clone.borrow_mut();
        let active_stroke = canvas.begin_stroke(color, current_brush_size, 1.0);
        println!(
            "Created {}stroke with color RGB({}, {}, {}) and width {}",
            if is_eraser { "eraser " } else { "" },
            color.r,
            color.g,
            color.b,
            current_brush_size
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

        if *mouse_state_clone4.borrow() == MouseState::Drawing
            && let Some(active_stroke) = &mut *active_stroke_clone3.borrow_mut()
        {
            active_stroke.add_point(point);
            println!(
                "Active stroke now has {} points",
                active_stroke.points().len()
            );
            if let Some(widget) = controller.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
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
