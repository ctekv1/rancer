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

use crate::canvas::{ActiveStroke, Canvas, Point};
use crate::logger;
use crate::opengl_renderer::GlRenderer;
use crate::preferences::Preferences;
use crate::ui::{self, SliderType};
use crate::window_backend::{MouseState as BackendMouseState, WindowBackend};

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}

/// Brush size options (defined in canvas::BRUSH_SIZES)

/// Window application state using GTK4
#[allow(dead_code)]
pub struct WindowApp {
    /// The GTK4 application
    app: Application,
    /// The GTK4 application window
    window: Option<ApplicationWindow>,
    /// Canvas for drawing operations
    canvas: Rc<RefCell<Canvas>>,
    /// HSV color values
    hue: f32,
    saturation: f32,
    value: f32,
    /// Custom saved colors
    custom_colors: Vec<[u8; 3]>,
    /// Selected custom color index (-1 if none)
    selected_custom_index: i32,
    /// Current active stroke being drawn
    active_stroke: Rc<RefCell<Option<ActiveStroke>>>,
    /// Current mouse state
    mouse_state: MouseState,
    /// Current mouse position
    mouse_position: Point,
    /// Current brush size in pixels
    brush_size: f32,
    /// Current brush opacity
    opacity: f32,
    /// Eraser mode active
    is_eraser: bool,
    /// Slider drag state (which slider is being dragged)
    slider_drag: Option<SliderType>,
    /// User preferences
    preferences: Preferences,
}

impl WindowApp {
    /// Create a new window application
    #[allow(dead_code)]
    pub fn new(preferences: Preferences) -> Self {
        logger::info("Creating GTK4 window application...");

        let app = Application::builder()
            .application_id("com.example.rancer")
            .build();

        Self {
            app,
            window: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            hue: preferences.palette.h,
            saturation: preferences.palette.s,
            value: preferences.palette.v,
            custom_colors: preferences.palette.custom_colors.clone(),
            selected_custom_index: -1,
            active_stroke: Rc::new(RefCell::new(None)),
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
            brush_size: preferences.brush.default_size,
            opacity: preferences.brush.default_opacity,
            is_eraser: false,
            slider_drag: None,
            preferences,
        }
    }

    /// Get current color from HSV values
    #[allow(dead_code)]
    fn current_color(&self) -> crate::canvas::Color {
        crate::canvas::hsv_to_rgb(self.hue, self.saturation, self.value)
    }

    /// Update HSV values in preferences and save
    #[allow(dead_code)]
    fn update_hsv_preferences(&mut self) {
        self.preferences.palette.h = self.hue;
        self.preferences.palette.s = self.saturation;
        self.preferences.palette.v = self.value;
        self.preferences.palette.custom_colors = self.custom_colors.clone();
        let _ = crate::preferences::save(&self.preferences);
    }
}

impl WindowBackend for WindowApp {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        logger::info("=== WINDOW CREATION ===");

        let canvas = self.canvas.clone();
        let active_stroke = self.active_stroke.clone();
        let mouse_state = self.mouse_state;
        let mouse_position = self.mouse_position;
        let brush_size = self.brush_size;
        let opacity = self.opacity;
        let is_eraser = self.is_eraser;
        let hue = self.hue;
        let saturation = self.saturation;
        let value = self.value;
        let custom_colors = self.custom_colors.clone();
        let selected_custom_index = self.selected_custom_index;
        let slider_drag = self.slider_drag;
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

            // Shared state (clone non-Copy types inside closure)
            let brush_size_shared = Rc::new(RefCell::new(brush_size));
            let opacity_shared = Rc::new(RefCell::new(opacity));
            let is_eraser_shared = Rc::new(RefCell::new(is_eraser));
            let hue_shared = Rc::new(RefCell::new(hue));
            let saturation_shared = Rc::new(RefCell::new(saturation));
            let value_shared = Rc::new(RefCell::new(value));
            let custom_colors_shared = Rc::new(RefCell::new(custom_colors.clone()));
            let selected_custom_index_shared = Rc::new(RefCell::new(selected_custom_index));
            let slider_drag_shared = Rc::new(RefCell::new(slider_drag));
            let gl_renderer: Rc<RefCell<Option<GlRenderer>>> = Rc::new(RefCell::new(None));

            // Set up mouse event handlers
            setup_mouse_events(
                &gl_area,
                &canvas,
                &active_stroke,
                mouse_state,
                mouse_position,
                brush_size_shared.clone(),
                opacity_shared.clone(),
                is_eraser_shared.clone(),
                hue_shared.clone(),
                saturation_shared.clone(),
                value_shared.clone(),
                custom_colors_shared.clone(),
                selected_custom_index_shared.clone(),
                slider_drag_shared.clone(),
                preferences.clone(),
            );

            // Set up keyboard handler
            let key_controller = EventControllerKey::new();
            let canvas_kb = canvas.clone();
            let prefs_kb = preferences.clone();
            let gl_area_kb = gl_area.clone();
            let is_eraser_kb = is_eraser_shared.clone();
            let brush_size_kb = brush_size_shared.clone();
            let active_stroke_kb = active_stroke.clone();
            let hue_kb = hue_shared.clone();
            let saturation_kb = saturation_shared.clone();
            let value_kb = value_shared.clone();
            let custom_colors_kb = custom_colors_shared.clone();
            let selected_custom_index_kb = selected_custom_index_shared.clone();

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
                        // Navigate custom colors
                        let custom_colors = custom_colors_kb.borrow();
                        if !custom_colors.is_empty() {
                            let mut selected = selected_custom_index_kb.borrow_mut();
                            let new_index = if *selected < 0 {
                                0
                            } else {
                                ((*selected as usize + 1) % custom_colors.len()) as i32
                            };
                            *selected = new_index;
                            let color = custom_colors[new_index as usize];
                            let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                r: color[0],
                                g: color[1],
                                b: color[2],
                                a: 255,
                            });
                            *hue_kb.borrow_mut() = hsv.h;
                            *saturation_kb.borrow_mut() = hsv.s;
                            *value_kb.borrow_mut() = hsv.v;
                            let mut prefs = prefs_kb.borrow_mut();
                            prefs.palette.h = hsv.h;
                            prefs.palette.s = hsv.s;
                            prefs.palette.v = hsv.v;
                            let _ = crate::preferences::save(&prefs);
                        }
                        gl_area_kb.queue_render();
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::Down => {
                        // Navigate custom colors
                        let custom_colors = custom_colors_kb.borrow();
                        if !custom_colors.is_empty() {
                            let mut selected = selected_custom_index_kb.borrow_mut();
                            let new_index = if *selected <= 0 {
                                custom_colors.len() as i32 - 1
                            } else {
                                *selected - 1
                            };
                            *selected = new_index;
                            let color = custom_colors[new_index as usize];
                            let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                r: color[0],
                                g: color[1],
                                b: color[2],
                                a: 255,
                            });
                            *hue_kb.borrow_mut() = hsv.h;
                            *saturation_kb.borrow_mut() = hsv.s;
                            *value_kb.borrow_mut() = hsv.v;
                            let mut prefs = prefs_kb.borrow_mut();
                            prefs.palette.h = hsv.h;
                            prefs.palette.s = hsv.s;
                            prefs.palette.v = hsv.v;
                            let _ = crate::preferences::save(&prefs);
                        }
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
                    gtk4::gdk::Key::e | gtk4::gdk::Key::E => {
                        // Toggle eraser (only when not drawing)
                        let is_drawing = active_stroke_kb.borrow().is_some();
                        if !is_drawing {
                            let mut is_eraser = is_eraser_kb.borrow_mut();
                            *is_eraser = !*is_eraser;
                            logger::info(&format!(
                                "Eraser mode: {}",
                                if *is_eraser { "ON" } else { "OFF" }
                            ));
                            gl_area_kb.queue_render();
                        }
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::plus | gtk4::gdk::Key::equal => {
                        // Increase brush size
                        let mut brush = brush_size_kb.borrow_mut();
                        *brush = crate::canvas::brush_size_up(*brush);
                        logger::info(&format!("Brush size: {}", *brush));
                        let mut prefs = prefs_kb.borrow_mut();
                        prefs.brush.default_size = *brush;
                        let _ = crate::preferences::save(&prefs);
                        gl_area_kb.queue_render();
                        glib::Propagation::Stop
                    }
                    gtk4::gdk::Key::minus => {
                        // Decrease brush size
                        let mut brush = brush_size_kb.borrow_mut();
                        *brush = crate::canvas::brush_size_down(*brush);
                        logger::info(&format!("Brush size: {}", *brush));
                        let mut prefs = prefs_kb.borrow_mut();
                        prefs.brush.default_size = *brush;
                        let _ = crate::preferences::save(&prefs);
                        gl_area_kb.queue_render();
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            });

            gl_area.add_controller(key_controller);

            // Set up OpenGL render callback
            let gl_renderer_clone = gl_renderer.clone();
            let canvas_clone = canvas.clone();
            let active_stroke_clone = active_stroke.clone();
            let brush_for_render = brush_size_shared.clone();
            let opacity_for_render = opacity_shared.clone();
            let is_eraser_for_render = is_eraser_shared.clone();
            let hue_for_render = hue_shared.clone();
            let saturation_for_render = saturation_shared.clone();
            let value_for_render = value_shared.clone();
            let custom_colors_for_render = custom_colors_shared.clone();
            let selected_custom_index_for_render = selected_custom_index_shared.clone();

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
                    let active_stroke = active_stroke_clone.borrow();
                    let current_brush_size = *brush_for_render.borrow();
                    let current_opacity = *opacity_for_render.borrow();
                    let is_eraser = *is_eraser_for_render.borrow();
                    let hue = *hue_for_render.borrow();
                    let saturation = *saturation_for_render.borrow();
                    let value = *value_for_render.borrow();
                    let custom_colors = custom_colors_for_render.borrow().clone();
                    let selected_custom_index = *selected_custom_index_for_render.borrow();
                    let width = gl_area.width();
                    let height = gl_area.height();

                    renderer.render_hsv(
                        &canvas,
                        &active_stroke,
                        current_brush_size,
                        is_eraser,
                        current_opacity,
                        width,
                        height,
                        hue,
                        saturation,
                        value,
                        custom_colors,
                        selected_custom_index,
                    );
                }

                glib::Propagation::Proceed
            });

            // Save preferences on window close
            let prefs_close = preferences.clone();
            let brush_close = brush_size_shared.clone();
            let hue_close = hue_shared.clone();
            let saturation_close = saturation_shared.clone();
            let value_close = value_shared.clone();
            let custom_colors_close = custom_colors_shared.clone();
            window.connect_close_request(move |_window| {
                let mut prefs = prefs_close.borrow_mut();
                prefs.palette.h = *hue_close.borrow();
                prefs.palette.s = *saturation_close.borrow();
                prefs.palette.v = *value_close.borrow();
                prefs.palette.custom_colors = custom_colors_close.borrow().clone();
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
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
    mouse_state: MouseState,
    mouse_position: Point,
    brush_size: Rc<RefCell<f32>>,
    opacity: Rc<RefCell<f32>>,
    is_eraser: Rc<RefCell<bool>>,
    hue: Rc<RefCell<f32>>,
    saturation: Rc<RefCell<f32>>,
    value: Rc<RefCell<f32>>,
    custom_colors: Rc<RefCell<Vec<[u8; 3]>>>,
    selected_custom_index: Rc<RefCell<i32>>,
    slider_drag: Rc<RefCell<Option<SliderDrag>>>,
    preferences: Rc<RefCell<crate::preferences::Preferences>>,
) {
    // Mouse click handler for left button
    let click_gesture = GestureClick::new();
    click_gesture.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let canvas_clone = canvas.clone();
    let active_stroke_clone = active_stroke.clone();
    let mouse_state_clone = Rc::new(RefCell::new(mouse_state));
    let mouse_position_clone = Rc::new(RefCell::new(mouse_position));
    let brush_size_clone = brush_size.clone();
    let opacity_clone = opacity.clone();
    let is_eraser_clone = is_eraser.clone();
    let hue_clone = hue.clone();
    let saturation_clone = saturation.clone();
    let value_clone = value.clone();
    let custom_colors_clone = custom_colors.clone();
    let selected_custom_index_clone = selected_custom_index.clone();
    let prefs_clone = preferences.clone();
    let slider_drag_clone = slider_drag.clone();

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

        // Use shared UI hit detection
        let custom_colors_snapshot = custom_colors_clone.borrow().clone();
        let hit = ui::hit_test(x as f32, y as f32, &custom_colors_snapshot);

        match hit {
            ui::UiElement::HueSlider(value) => {
                *hue_clone.borrow_mut() = value;
                *selected_custom_index_clone.borrow_mut() = -1;
                *slider_drag_clone.borrow_mut() = Some(SliderType::Hue);
                update_hsv_prefs(
                    &prefs_clone,
                    *hue_clone.borrow(),
                    *saturation_clone.borrow(),
                    *value_clone.borrow(),
                    custom_colors_clone.borrow().clone(),
                );
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SaturationSlider(value) => {
                *saturation_clone.borrow_mut() = value;
                *selected_custom_index_clone.borrow_mut() = -1;
                *slider_drag_clone.borrow_mut() = Some(SliderType::Saturation);
                update_hsv_prefs(
                    &prefs_clone,
                    *hue_clone.borrow(),
                    *saturation_clone.borrow(),
                    *value_clone.borrow(),
                    custom_colors_clone.borrow().clone(),
                );
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::ValueSlider(value) => {
                *value_clone.borrow_mut() = value;
                *selected_custom_index_clone.borrow_mut() = -1;
                *slider_drag_clone.borrow_mut() = Some(SliderType::Value);
                update_hsv_prefs(
                    &prefs_clone,
                    *hue_clone.borrow(),
                    *saturation_clone.borrow(),
                    *value_clone.borrow(),
                    custom_colors_clone.borrow().clone(),
                );
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::CustomColor(idx) => {
                let custom = custom_colors_clone.borrow();
                let color = custom[idx];
                drop(custom);
                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: 255,
                });
                *hue_clone.borrow_mut() = hsv.h;
                *saturation_clone.borrow_mut() = hsv.s;
                *value_clone.borrow_mut() = hsv.v;
                *selected_custom_index_clone.borrow_mut() = idx as i32;
                update_hsv_prefs(
                    &prefs_clone,
                    hsv.h,
                    hsv.s,
                    hsv.v,
                    custom_colors_clone.borrow().clone(),
                );
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SaveColor => {
                let h = *hue_clone.borrow();
                let s = *saturation_clone.borrow();
                let v = *value_clone.borrow();
                let current = crate::canvas::hsv_to_rgb(h, s, v);
                let mut colors = custom_colors_clone.borrow_mut();
                if colors.len() >= 10 {
                    colors.remove(0);
                }
                colors.push([current.r, current.g, current.b]);
                drop(colors);
                update_hsv_prefs(&prefs_clone, h, s, v, custom_colors_clone.borrow().clone());
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::BrushSize(size) => {
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
            ui::UiElement::Eraser => {
                let mut is_eraser = is_eraser_clone.borrow_mut();
                *is_eraser = !*is_eraser;
                println!("Eraser mode: {}", if *is_eraser { "ON" } else { "OFF" });
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Clear => {
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
            ui::UiElement::Undo => {
                let mut canvas = canvas_clone.borrow_mut();
                if canvas.can_undo() {
                    canvas.undo();
                    logger::info("Undo: removed last stroke");
                    println!("Undo: removed last stroke");
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::Redo => {
                let mut canvas = canvas_clone.borrow_mut();
                if canvas.can_redo() {
                    canvas.redo();
                    logger::info("Redo: restored last stroke");
                    println!("Redo: restored last stroke");
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::Opacity(opacity) => {
                *opacity_clone.borrow_mut() = opacity;
                prefs_clone.borrow_mut().brush.default_opacity = opacity;
                let _ = crate::preferences::save(&prefs_clone.borrow());
                logger::info(&format!("Opacity: {}", opacity));
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Canvas => {
                // Not on any UI element — start drawing
            }
        }

        // If not on UI, start drawing
        let is_eraser = *is_eraser_clone.borrow();
        let h = *hue_clone.borrow();
        let s = *saturation_clone.borrow();
        let v = *value_clone.borrow();
        let color = if is_eraser {
            crate::canvas::Color::WHITE
        } else {
            crate::canvas::hsv_to_rgb(h, s, v)
        };
        let current_brush_size = *brush_size_clone.borrow();
        let current_opacity = *opacity_clone.borrow();
        let mut canvas = canvas_clone.borrow_mut();
        let active_stroke = canvas.begin_stroke(color, current_brush_size, current_opacity);
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
    let slider_drag_release = slider_drag.clone();

    click_gesture_release.connect_released(move |_gesture, _n_press, _x, _y| {
        *mouse_state_clone3.borrow_mut() = MouseState::Idle;
        *slider_drag_release.borrow_mut() = None;

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
    let slider_drag_motion = slider_drag.clone();
    let hue_motion = hue.clone();
    let saturation_motion = saturation.clone();
    let value_motion = value.clone();
    let selected_custom_index_motion = selected_custom_index.clone();
    let prefs_motion = preferences.clone();
    let custom_colors_motion = custom_colors.clone();

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

        // Handle slider dragging
        let active_slider = *slider_drag_motion.borrow();
        if let Some((slider, value)) = ui::slider_drag(x as f32, y as f32, active_slider) {
            match slider {
                SliderType::Hue => {
                    *hue_motion.borrow_mut() = value;
                    *selected_custom_index_motion.borrow_mut() = -1;
                }
                SliderType::Saturation => {
                    *saturation_motion.borrow_mut() = value;
                    *selected_custom_index_motion.borrow_mut() = -1;
                }
                SliderType::Value => {
                    *value_motion.borrow_mut() = value;
                    *selected_custom_index_motion.borrow_mut() = -1;
                }
            }
            update_hsv_prefs(
                &prefs_motion,
                *hue_motion.borrow(),
                *saturation_motion.borrow(),
                *value_motion.borrow(),
                custom_colors_motion.borrow().clone(),
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

/// Update HSV preferences helper
fn update_hsv_prefs(
    prefs: &Rc<RefCell<crate::preferences::Preferences>>,
    h: f32,
    s: f32,
    v: f32,
    custom_colors: Vec<[u8; 3]>,
) {
    let mut p = prefs.borrow_mut();
    p.palette.h = h;
    p.palette.s = s;
    p.palette.v = v;
    p.palette.custom_colors = custom_colors;
    let _ = crate::preferences::save(&p);
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
