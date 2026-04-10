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
    Application, ApplicationWindow, EventControllerKey, EventControllerMotion,
    EventControllerScroll, GLArea, GestureClick,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::canvas::{ActiveStroke, BrushType, Canvas, Point};
use crate::logger;
use crate::opengl_renderer::{GlRenderFrame, GlRenderer, GlUiState, GlViewportState};
use crate::preferences::Preferences;
use crate::ui::{self, SliderType};
use crate::window_backend::{MouseState as BackendMouseState, WindowBackend};

/// Consolidated render state shared across all GTK4 closures.
/// Replaces ~20 individual Rc<RefCell<...>> variables.
struct GlRenderState {
    hue: f32,
    saturation: f32,
    value: f32,
    custom_colors: Vec<[u8; 3]>,
    selected_custom_index: i32,
    brush_size: f32,
    opacity: f32,
    is_eraser: bool,
    zoom: f32,
    pan_offset: (f32, f32),
    is_panning: bool,
    last_mouse_position: Point,
    slider_drag: Option<SliderType>,
    mouse_state: MouseState,
    mouse_position: Point,
    brush_type: BrushType,
    selection_tool_active: bool,
    selection_drawing: bool,
    selection_start: Point,
    selection_moving: bool,
    selection_move_offset: (f32, f32),
    selection_copy_mode: bool,
    ctrl_pressed: bool,
}

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    /// No mouse button is pressed
    Idle,
    /// Left mouse button is pressed and drawing
    Drawing,
}

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
    /// Viewport zoom level (1.0 = 100%)
    zoom: f32,
    /// Viewport pan offset (in canvas coordinates)
    pan_offset: (f32, f32),
    /// Whether space key is held for panning
    is_panning: bool,
    /// Last mouse position for panning delta calculation
    last_mouse_position: Point,
    /// Index of the active layer
    active_layer: usize,
    /// Current brush type
    brush_type: BrushType,
    /// Time tracking for animations
    start_time: Instant,
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
            brush_type: preferences.brush.default_type.parse().unwrap_or_default(),
            is_eraser: false,
            slider_drag: None,
            preferences,
            zoom: 1.0,
            pan_offset: (0.0, 0.0),
            is_panning: false,
            last_mouse_position: Point { x: 0.0, y: 0.0 },
            active_layer: 0,
            start_time: Instant::now(),
        }
    }
}

impl WindowBackend for WindowApp {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        logger::info("=== WINDOW CREATION ===");

        let canvas = self.canvas.clone();
        let active_stroke = self.active_stroke.clone();
        let preferences = Rc::new(RefCell::new(self.preferences.clone()));

        // Consolidated render state — replaces ~20 individual Rc<RefCell<...>>
        let render_state = Rc::new(RefCell::new(GlRenderState {
            hue: self.hue,
            saturation: self.saturation,
            value: self.value,
            custom_colors: self.custom_colors.clone(),
            selected_custom_index: self.selected_custom_index,
            brush_size: self.brush_size,
            opacity: self.opacity,
            is_eraser: self.is_eraser,
            zoom: 1.0,
            pan_offset: (0.0, 0.0),
            is_panning: false,
            last_mouse_position: Point { x: 0.0, y: 0.0 },
            slider_drag: None,
            mouse_state: MouseState::Idle,
            mouse_position: Point { x: 0.0, y: 0.0 },
            brush_type: self.brush_type,
            selection_tool_active: false,
            selection_drawing: false,
            selection_start: Point { x: 0.0, y: 0.0 },
            selection_moving: false,
            selection_move_offset: (0.0, 0.0),
            selection_copy_mode: false,
            ctrl_pressed: false,
        }));

        self.app.connect_activate({
            let canvas = canvas.clone();
            let active_stroke = active_stroke.clone();
            let render_state = render_state.clone();
            let preferences = preferences.clone();
            let start_time = self.start_time;

            move |app| {
                let prefs = preferences.borrow();

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

                drop(prefs);

                let gl_area = GLArea::builder().hexpand(true).vexpand(true).build();
                gl_area.set_focusable(true);
                window.set_child(Some(&gl_area));

                // Use GTK4's tick callback for smooth animation - fires every frame synced to display refresh
                let gl_area_anim = gl_area.clone();
                gl_area.add_tick_callback(move |_widget, _frame_clock| {
                    gl_area_anim.queue_render();
                    glib::ControlFlow::Continue
                });

                let gl_renderer: Rc<RefCell<Option<GlRenderer>>> = Rc::new(RefCell::new(None));

                setup_mouse_events(
                    &gl_area,
                    &canvas,
                    &active_stroke,
                    &render_state,
                    &preferences,
                );

                // Keyboard handler
                let key_controller = EventControllerKey::new();
                let canvas_kb = canvas.clone();
                let active_stroke_kb = active_stroke.clone();
                let gl_area_kb = gl_area.clone();
                let render_state_kb = render_state.clone();

                key_controller.connect_key_pressed(move |_controller, key, _keycode, state| {
                    let is_ctrl = state.contains(ModifierType::CONTROL_MASK);
                    render_state_kb.borrow_mut().ctrl_pressed = is_ctrl;

                    if key == gtk4::gdk::Key::space {
                        render_state_kb.borrow_mut().is_panning = true;
                    }

                    match key {
                        gtk4::gdk::Key::s | gtk4::gdk::Key::S => {
                            let canvas_ref = canvas_kb.borrow();
                            let gl_area_ref = gl_area_kb.clone();

                            if let Some(path) = crate::export_ui::show_save_dialog() {
                                match crate::export::export_to_png(&canvas_ref, &path) {
                                    Ok(_) => {
                                        crate::export_ui::notify_export_result(true, &path, None);
                                    }
                                    Err(e) => {
                                        crate::export_ui::notify_export_result(false, &path, Some(&e.to_string()));
                                    }
                                }
                                gl_area_ref.queue_render();
                            }
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::Up => {
                            let mut state = render_state_kb.borrow_mut();
                            if !state.custom_colors.is_empty() {
                                let new_index = if state.selected_custom_index < 0 {
                                    0
                                } else {
                                    ((state.selected_custom_index as usize + 1) % state.custom_colors.len()) as i32
                                };
                                state.selected_custom_index = new_index;
                                let color = state.custom_colors[new_index as usize];
                                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                    r: color[0], g: color[1], b: color[2], a: 255,
                                });
                                state.hue = hsv.h;
                                state.saturation = hsv.s;
                                state.value = hsv.v;
                            }
                            drop(state);
                            gl_area_kb.queue_render();
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::Down => {
                            let mut state = render_state_kb.borrow_mut();
                            if !state.custom_colors.is_empty() {
                                let new_index = if state.selected_custom_index <= 0 {
                                    state.custom_colors.len() as i32 - 1
                                } else {
                                    state.selected_custom_index - 1
                                };
                                state.selected_custom_index = new_index;
                                let color = state.custom_colors[new_index as usize];
                                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                    r: color[0], g: color[1], b: color[2], a: 255,
                                });
                                state.hue = hsv.h;
                                state.saturation = hsv.s;
                                state.value = hsv.v;
                            }
                            drop(state);
                            gl_area_kb.queue_render();
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::z | gtk4::gdk::Key::Z => {
                            if state.contains(ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK) {
                                let mut canvas = canvas_kb.borrow_mut();
                                if canvas.can_redo() {
                                    canvas.redo();
                                    logger::info("Redo: restored last undone stroke");
                                    gl_area_kb.queue_render();
                                }
                                glib::Propagation::Stop
                            } else if state.contains(ModifierType::CONTROL_MASK) {
                                let mut canvas = canvas_kb.borrow_mut();
                                if canvas.can_undo() {
                                    canvas.undo();
                                    logger::info("Undo: removed last stroke");
                                    gl_area_kb.queue_render();
                                }
                                glib::Propagation::Stop
                            } else {
                                glib::Propagation::Proceed
                            }
                        }
                        gtk4::gdk::Key::y | gtk4::gdk::Key::Y => {
                            if state.contains(ModifierType::CONTROL_MASK) {
                                let mut canvas = canvas_kb.borrow_mut();
                                if canvas.can_redo() {
                                    canvas.redo();
                                    logger::info("Redo: restored last undone stroke");
                                    gl_area_kb.queue_render();
                                }
                                glib::Propagation::Stop
                            } else {
                                glib::Propagation::Proceed
                            }
                        }
                        gtk4::gdk::Key::Delete => {
                            if state.contains(ModifierType::CONTROL_MASK) {
                                let mut canvas = canvas_kb.borrow_mut();
                                canvas.clear();
                                logger::info("Canvas cleared");
                                gl_area_kb.queue_render();
                            } else {
                                // Commit selection
                                let mut canvas = canvas_kb.borrow_mut();
                                if canvas.has_selection() {
                                    canvas.commit_selection();
                                    logger::info("Selection committed");
                                    gl_area_kb.queue_render();
                                }
                            }
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::Escape => {
                            let mut canvas = canvas_kb.borrow_mut();
                            if canvas.has_selection() {
                                canvas.clear_selection();
                                logger::info("Selection cleared");
                                gl_area_kb.queue_render();
                            }
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::d | gtk4::gdk::Key::D => {
                            if state.contains(ModifierType::CONTROL_MASK) {
                                let mut canvas = canvas_kb.borrow_mut();
                                if canvas.has_selection() {
                                    canvas.clear_selection();
                                    logger::info("Selection deselected (Ctrl+D)");
                                    gl_area_kb.queue_render();
                                }
                                glib::Propagation::Stop
                            } else {
                                glib::Propagation::Proceed
                            }
                        }
                        gtk4::gdk::Key::e | gtk4::gdk::Key::E => {
                            let is_drawing = active_stroke_kb.borrow().is_some();
                            if !is_drawing {
                                let mut state = render_state_kb.borrow_mut();
                                state.is_eraser = !state.is_eraser;
                                let is_on = state.is_eraser;
                                drop(state);
                                logger::info(&format!(
                                    "Eraser mode: {}",
                                    if is_on { "ON" } else { "OFF" }
                                ));
                                gl_area_kb.queue_render();
                            }
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::plus | gtk4::gdk::Key::equal => {
                            let mut state = render_state_kb.borrow_mut();
                            state.brush_size = crate::canvas::brush_size_up(state.brush_size);
                            logger::info(&format!("Brush size: {}", state.brush_size));
                            drop(state);
                            gl_area_kb.queue_render();
                            glib::Propagation::Stop
                        }
                        gtk4::gdk::Key::minus => {
                            let mut state = render_state_kb.borrow_mut();
                            state.brush_size = crate::canvas::brush_size_down(state.brush_size);
                            logger::info(&format!("Brush size: {}", state.brush_size));
                            drop(state);
                            gl_area_kb.queue_render();
                            glib::Propagation::Stop
                        }
                        _ => glib::Propagation::Proceed,
                    }
                });

                key_controller.connect_key_released({
                    let render_state = render_state.clone();
                    move |_controller, key, _keycode, _state| {
                        if key == gtk4::gdk::Key::space {
                            render_state.borrow_mut().is_panning = false;
                        }
                    }
                });

                gl_area.add_controller(key_controller);

                // Scroll handler for zoom
                let gl_area_scroll = gl_area.clone();
                let render_state_scroll = render_state.clone();
                let scroll_controller =
                    EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
                scroll_controller.connect_scroll(move |_controller, _dx, dy| {
                    let mut state = render_state_scroll.borrow_mut();
                    let old_zoom = state.zoom;
                    let zoom_factor = 1.25;
                    let new_zoom = if dy < 0.0 {
                        (old_zoom * zoom_factor).min(10.0)
                    } else if dy > 0.0 {
                        (old_zoom / zoom_factor).max(0.1)
                    } else {
                        old_zoom
                    };

                    if (new_zoom - old_zoom).abs() > 0.001 {
                        let mouse_pos = state.mouse_position;
                        let mouse_canvas_x = mouse_pos.x / old_zoom + state.pan_offset.0;
                        let mouse_canvas_y = mouse_pos.y / old_zoom + state.pan_offset.1;

                        let new_pan_x = mouse_canvas_x - mouse_pos.x / new_zoom;
                        let new_pan_y = mouse_canvas_y - mouse_pos.y / new_zoom;

                        state.zoom = new_zoom;
                        state.pan_offset = (new_pan_x, new_pan_y);

                        gl_area_scroll.queue_render();
                    }
                    glib::Propagation::Stop
                });
                gl_area.add_controller(scroll_controller);

                // OpenGL render callback
                let gl_renderer_render = gl_renderer.clone();
                let canvas_render = canvas.clone();
                let active_stroke_render = active_stroke.clone();
                let render_state_render = render_state.clone();
                let start_time_render = start_time;

                gl_area.connect_render(move |gl_area, _context| {
                    // Attach buffers to ensure proper framebuffer state
                    gl_area.attach_buffers();

                    let _gl_context = match gl_area.context() {
                        Some(ctx) => ctx,
                        None => {
                            logger::warn("GLArea has no context, skipping render");
                            return glib::Propagation::Stop;
                        }
                    };

                    if gl_renderer_render.borrow().is_none() {
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
                                *gl_renderer_render.borrow_mut() = Some(renderer);
                            }
                            Err(e) => {
                                logger::error(&format!("Failed to initialize OpenGL renderer: {e}"));
                                return glib::Propagation::Stop;
                            }
                        }
                    }

                    if let Some(ref mut renderer) = *gl_renderer_render.borrow_mut() {
                        let state = render_state_render.borrow();
                        let canvas_ref = canvas_render.borrow();
                        let active_ref = active_stroke_render.borrow();
                        let width = gl_area.width();
                        let height = gl_area.height();

                        let frame = GlRenderFrame {
                            canvas: &canvas_ref,
                            active_stroke: &active_ref,
                                ui: GlUiState {
                                    hue: state.hue,
                                    saturation: state.saturation,
                                    value: state.value,
                                    custom_colors: state.custom_colors.clone(),
                                    selected_custom_index: state.selected_custom_index,
                                    brush_size: state.brush_size,
                                    opacity: state.opacity,
                                    is_eraser: state.is_eraser,
                                    brush_type: state.brush_type,
                                    selection_tool_active: state.selection_tool_active,
                                    selection_rect: {
                                        let canvas_rect = canvas_ref.selection().map(|s| s.rect);
                                        if state.selection_drawing {
                                            let zoom = state.zoom;
                                            let pan = state.pan_offset;
                                            let start = state.selection_start;
                                            let end = state.mouse_position;
                                            // Convert screen coords to canvas coords
                                            let cx1 = start.x / zoom + pan.0;
                                            let cy1 = start.y / zoom + pan.1;
                                            let cx2 = end.x / zoom + pan.0;
                                            let cy2 = end.y / zoom + pan.1;
                                            Some(crate::canvas::Rect::new(
                                                cx1, cy1,
                                                cx2 - cx1,
                                                cy2 - cy1,
                                            ))
                                        } else {
                                            canvas_rect
                                        }
                                    },
                                    selection_time: start_time_render.elapsed().as_secs_f32(),
                                },
                            viewport: GlViewportState {
                                zoom: state.zoom,
                                pan_offset: state.pan_offset,
                            },
                            window_size: (width, height),
                        };
                        renderer.render(&frame);

                        // Render selection overlay (moved strokes + marching ants)
                        // Use canvas selection if exists, otherwise fall back to in-progress drawing rect
                        let selection_rect = canvas_ref.selection()
                            .map(|s| s.rect)
                            .or(frame.ui.selection_rect);

                        if let Some(rect) = selection_rect {
                            let selection_strokes: Vec<(crate::canvas::Stroke, f32)> = canvas_ref
                                .selection()
                                .map(|s| s.strokes.iter().map(|s| (s.clone(), 1.0)).collect())
                                .unwrap_or_default();
                            renderer.render_selection_overlay(
                                rect,
                                frame.ui.selection_time,
                                width as u32,
                                height as u32,
                                &selection_strokes,
                                state.zoom,
                                state.pan_offset,
                            );
                        }
                    } else {
                        // No renderer yet - queue render to try again
                        gl_area.queue_render();
                    }

                    glib::Propagation::Proceed
                });

                // Save preferences on window close
                let render_state_close = render_state.clone();
                let prefs_close = preferences.clone();
                window.connect_close_request(move |_window| {
                    let state = render_state_close.borrow();
                    let mut prefs = prefs_close.borrow_mut();
                    prefs.palette.h = state.hue;
                    prefs.palette.s = state.saturation;
                    prefs.palette.v = state.value;
                    prefs.palette.custom_colors = state.custom_colors.clone();
                    prefs.brush.default_size = state.brush_size;
                    prefs.brush.default_opacity = state.opacity;
                    prefs.brush.default_type = format!("{:?}", state.brush_type);
                    drop(state);

                    if let Err(e) = crate::preferences::save(&prefs) {
                        logger::error(&format!("Failed to save preferences on close: {e}"));
                    }

                    glib::Propagation::Proceed
                });

                window.present();
                gl_area.grab_focus();
            }
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
fn setup_mouse_events(
    gl_area: &GLArea,
    canvas: &Rc<RefCell<Canvas>>,
    active_stroke: &Rc<RefCell<Option<ActiveStroke>>>,
    render_state: &Rc<RefCell<GlRenderState>>,
    preferences: &Rc<RefCell<Preferences>>,
) {
    // Mouse click handler
    let click_gesture = GestureClick::new();
    click_gesture.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let canvas_click = canvas.clone();
    let active_stroke_click = active_stroke.clone();
    let render_state_click = render_state.clone();
    let preferences_click = preferences.clone();

    click_gesture.connect_pressed(move |gesture, _n_press, x, y| {
        let point = Point {
            x: x as f32,
            y: y as f32,
        };
        let mut state = render_state_click.borrow_mut();
        state.mouse_position = point;
        state.mouse_state = MouseState::Drawing;

        let custom_colors_snapshot = state.custom_colors.clone();
        drop(state);

        let canvas_hit = canvas_click.borrow();
        let layer_count = canvas_hit.layer_count();
        let active_layer = canvas_hit.active_layer();
        let selection_rect_hit = canvas_hit.selection().map(|s| s.rect);
        let zoom = render_state_click.borrow().zoom;
        let pan = render_state_click.borrow().pan_offset;
        drop(canvas_hit);

        // Convert canvas-space selection rect to screen coordinates for hit testing
        let selection_rect_screen = selection_rect_hit.map(|r| {
            crate::canvas::Rect::new(
                (r.x - pan.0) * zoom,
                (r.y - pan.1) * zoom,
                r.w * zoom,
                r.h * zoom,
            )
        });

        let window_width = if let Some(widget) = gesture.widget() {
            widget.allocated_width() as f32
        } else {
            1280.0
        };
        let hit = ui::hit_test(
            x as f32,
            y as f32,
            &custom_colors_snapshot,
            layer_count,
            active_layer,
            window_width,
            selection_rect_screen,
        );

        match hit {
            ui::UiElement::HueSlider(value) => {
                let mut state = render_state_click.borrow_mut();
                state.hue = value;
                state.selected_custom_index = -1;
                state.slider_drag = Some(SliderType::Hue);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SaturationSlider(value) => {
                let mut state = render_state_click.borrow_mut();
                state.saturation = value;
                state.selected_custom_index = -1;
                state.slider_drag = Some(SliderType::Saturation);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::ValueSlider(value) => {
                let mut state = render_state_click.borrow_mut();
                state.value = value;
                state.selected_custom_index = -1;
                state.slider_drag = Some(SliderType::Value);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::CustomColor(idx) => {
                let mut state = render_state_click.borrow_mut();
                let color = state.custom_colors[idx];
                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: 255,
                });
                state.hue = hsv.h;
                state.saturation = hsv.s;
                state.value = hsv.v;
                state.selected_custom_index = idx as i32;
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SaveColor => {
                let mut state = render_state_click.borrow_mut();
                let current = crate::canvas::hsv_to_rgb(state.hue, state.saturation, state.value);
                if state.custom_colors.len() >= 10 {
                    state.custom_colors.remove(0);
                }
                state.custom_colors.push([current.r, current.g, current.b]);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::BrushSize(size) => {
                render_state_click.borrow_mut().brush_size = size;
                logger::info(&format!("Selected brush size: {size}"));
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Eraser => {
                let mut state = render_state_click.borrow_mut();
                state.is_eraser = !state.is_eraser;
                logger::info(&format!(
                    "Eraser mode: {}",
                    if state.is_eraser { "ON" } else { "OFF" }
                ));
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::BrushType(brush_type) => {
                let mut state = render_state_click.borrow_mut();
                state.brush_type = brush_type;
                drop(state);
                {
                    let mut prefs = preferences_click.borrow_mut();
                    prefs.brush.default_type = format!("{:?}", brush_type);
                    if let Err(e) = crate::preferences::save(&prefs) {
                        logger::error(&format!("Failed to save brush type: {e}"));
                    }
                }
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Clear => {
                canvas_click.borrow_mut().clear();
                logger::info("Canvas cleared");
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Undo => {
                let mut canvas = canvas_click.borrow_mut();
                if canvas.can_undo() {
                    canvas.undo();
                    logger::info("Undo: removed last stroke");
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::Redo => {
                let mut canvas = canvas_click.borrow_mut();
                if canvas.can_redo() {
                    canvas.redo();
                    logger::info("Redo: restored last stroke");
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::Opacity(opacity) => {
                render_state_click.borrow_mut().opacity = opacity;
                logger::info(&format!("Opacity: {}", opacity));
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::Export => {
                let canvas_ref = canvas_click.borrow();

                if let Some(path) = crate::export_ui::show_save_dialog() {
                    match crate::export::export_to_png(&canvas_ref, &path) {
                        Ok(_) => {
                            crate::export_ui::notify_export_result(true, &path, None);
                        }
                        Err(e) => {
                            crate::export_ui::notify_export_result(
                                false,
                                &path,
                                Some(&e.to_string()),
                            );
                        }
                    }
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::ZoomIn => {
                let mut state = render_state_click.borrow_mut();
                let new_zoom = (state.zoom * 1.25).min(10.0);
                if (new_zoom - state.zoom).abs() > 0.001 {
                    state.zoom = new_zoom;
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::ZoomOut => {
                let mut state = render_state_click.borrow_mut();
                let new_zoom = (state.zoom / 1.25).max(0.1);
                if (new_zoom - state.zoom).abs() > 0.001 {
                    state.zoom = new_zoom;
                    if let Some(widget) = gesture.widget()
                        && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                    {
                        gl_area.queue_render();
                    }
                }
                return;
            }
            ui::UiElement::LayerRow(index) => {
                let _ = canvas_click.borrow_mut().set_active_layer(index);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::LayerVisibility(index) => {
                let _ = canvas_click.borrow_mut().toggle_layer_visibility(index);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::AddLayer => {
                let mut canvas = canvas_click.borrow_mut();
                if canvas.layer_count() < crate::canvas::MAX_LAYERS {
                    let _ = canvas.add_layer(None);
                    let new_index = canvas.layer_count() - 1;
                    let _ = canvas.set_active_layer(new_index);
                }
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::DeleteLayer => {
                let mut canvas = canvas_click.borrow_mut();
                if canvas.layer_count() > 1 {
                    let current = canvas.active_layer();
                    let _ = canvas.remove_layer(current);
                }
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::MoveLayerUp => {
                let mut canvas = canvas_click.borrow_mut();
                let idx = canvas.active_layer();
                if idx > 0 {
                    let _ = canvas.move_layer(idx, idx - 1);
                }
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::MoveLayerDown => {
                let mut canvas = canvas_click.borrow_mut();
                let idx = canvas.active_layer();
                if idx < canvas.layer_count() - 1 {
                    let _ = canvas.move_layer(idx, idx + 1);
                }
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SelectionTool => {
                let mut state = render_state_click.borrow_mut();
                state.selection_tool_active = !state.selection_tool_active;
                if !state.selection_tool_active {
                    canvas_click.borrow_mut().clear_selection();
                }
                drop(state);
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                return;
            }
            ui::UiElement::SelectionRect => {
                // Start moving/copying selection
                let mut state = render_state_click.borrow_mut();
                let copy_mode = state.ctrl_pressed;
                state.selection_moving = true;
                state.selection_copy_mode = copy_mode;
                state.selection_move_offset = (x as f32, y as f32);
                drop(state);
                return;
            }
            ui::UiElement::SelectionStart(_, _) => {
                // Start drawing selection rect — handled in motion
            }
            ui::UiElement::Canvas => {
                // Not on any UI element — start drawing
            }
        }

        // If selection tool is active, handle selection logic
        let is_selection_active = render_state_click.borrow().selection_tool_active;
        if is_selection_active {
            let mut state = render_state_click.borrow_mut();
            // If there's an existing selection and we're not clicking on it, clear it first
            if (state.selection_drawing || canvas_click.borrow().has_selection())
                && canvas_click.borrow().has_selection()
            {
                drop(state);
                canvas_click.borrow_mut().clear_selection();
                if let Some(widget) = gesture.widget()
                    && let Some(gl_area) = widget.downcast_ref::<GLArea>()
                {
                    gl_area.queue_render();
                }
                state = render_state_click.borrow_mut();
            }
            state.selection_drawing = true;
            let canvas_point = Point {
                x: x as f32,
                y: y as f32,
            };
            state.selection_start = canvas_point;
            return;
        }
        if canvas_click.borrow().is_active_layer_locked() {
            return;
        }
        let state = render_state_click.borrow();
        let is_eraser = state.is_eraser;
        let h = state.hue;
        let s = state.saturation;
        let v = state.value;
        let color = if is_eraser {
            crate::canvas::Color::WHITE
        } else {
            crate::canvas::hsv_to_rgb(h, s, v)
        };
        let current_brush_size = state.brush_size;
        let current_opacity = state.opacity;
        let current_brush_type = state.brush_type;
        drop(state);

        let mut canvas = canvas_click.borrow_mut();
        let active_stroke = canvas.begin_stroke(
            color,
            current_brush_size,
            current_opacity,
            current_brush_type,
        );
        logger::info(&format!(
            "Created {}stroke with color RGB({}, {}, {}) and width {}",
            if is_eraser { "eraser " } else { "" },
            color.r,
            color.g,
            color.b,
            current_brush_size
        ));

        *active_stroke_click.borrow_mut() = Some(active_stroke);

        let state = render_state_click.borrow();
        let zoom = state.zoom;
        let pan = state.pan_offset;
        drop(state);

        let canvas_x = point.x / zoom + pan.0;
        let canvas_y = point.y / zoom + pan.1;
        if let Some(active_stroke) = &mut *active_stroke_click.borrow_mut() {
            active_stroke.add_point(Point {
                x: canvas_x,
                y: canvas_y,
            });
            logger::debug(&format!(
                "Added first point to active stroke: ({}, {})",
                point.x, point.y
            ));
            logger::debug(&format!(
                "Active stroke now has {} points",
                active_stroke.points().len()
            ));
        }
    });

    gl_area.add_controller(click_gesture);

    // Mouse release handler
    let click_gesture_release = GestureClick::new();
    click_gesture_release.set_button(gtk4::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

    let active_stroke_release = active_stroke.clone();
    let canvas_release = canvas.clone();
    let render_state_release = render_state.clone();

    click_gesture_release.connect_released(move |gesture, _n_press, x, y| {
        let mut state = render_state_release.borrow_mut();
        state.mouse_state = MouseState::Idle;
        state.slider_drag = None;

        // If we were drawing a selection rect, commit it
        if state.selection_drawing {
            state.selection_drawing = false;
            let zoom = state.zoom;
            let pan = state.pan_offset;
            let start = state.selection_start;
            let end = Point {
                x: x as f32,
                y: y as f32,
            };
            // Convert screen coords to canvas coords
            let canvas_start_x = start.x / zoom + pan.0;
            let canvas_start_y = start.y / zoom + pan.1;
            let canvas_end_x = end.x / zoom + pan.0;
            let canvas_end_y = end.y / zoom + pan.1;
            let rect = crate::canvas::Rect::new(
                canvas_start_x,
                canvas_start_y,
                canvas_end_x - canvas_start_x,
                canvas_end_y - canvas_start_y,
            );
            drop(state);
            canvas_release.borrow_mut().begin_selection(rect);
            if let Some(widget) = gesture.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
        } else if state.selection_moving {
            state.selection_moving = false;
            let copy_mode = state.selection_copy_mode;
            drop(state);
            if copy_mode {
                canvas_release.borrow_mut().copy_selection();
            }
            // Keep selection active after move (don't commit)
            // User can press Delete to commit or Esc to clear
        } else {
            drop(state);
        }

        if let Some(active_stroke) = active_stroke_release.borrow_mut().take() {
            let mut canvas = canvas_release.borrow_mut();
            if let Err(e) = canvas.commit_stroke(active_stroke) {
                logger::error(&format!("Failed to commit stroke: {e}"));
            } else {
                logger::info("Stroke committed successfully");
            }
        }
    });

    gl_area.add_controller(click_gesture_release);

    // Mouse motion handler
    let motion_controller = EventControllerMotion::new();

    let active_stroke_motion = active_stroke.clone();
    let canvas_motion = canvas.clone();
    let render_state_motion = render_state.clone();

    motion_controller.connect_motion(move |controller, x, y| {
        let point = Point {
            x: x as f32,
            y: y as f32,
        };
        let mut state = render_state_motion.borrow_mut();
        state.mouse_position = point;

        // Handle panning
        if state.is_panning {
            let zoom = state.zoom;
            let old_pan = state.pan_offset;
            let last_pos = state.last_mouse_position;
            let dx = point.x - last_pos.x;
            let dy = point.y - last_pos.y;
            state.pan_offset = (old_pan.0 - dx / zoom, old_pan.1 - dy / zoom);
            state.last_mouse_position = point;
            drop(state);
            if let Some(widget) = controller.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        }

        state.last_mouse_position = point;

        // Selection rect drawing
        if state.selection_drawing {
            drop(state);
            if let Some(widget) = controller.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        }

        // Selection moving
        if state.selection_moving {
            let dx = point.x - state.selection_move_offset.0;
            let dy = point.y - state.selection_move_offset.1;
            state.selection_move_offset = (point.x, point.y);
            let zoom = state.zoom;
            let canvas_dx = dx / zoom;
            let canvas_dy = dy / zoom;
            drop(state);
            canvas_motion
                .borrow_mut()
                .move_selection(canvas_dx, canvas_dy);
            if let Some(widget) = controller.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        }

        if state.mouse_state == MouseState::Drawing
            && let Some(active_stroke) = &mut *active_stroke_motion.borrow_mut()
        {
            let zoom = state.zoom;
            let pan = state.pan_offset;
            let canvas_x = point.x / zoom + pan.0;
            let canvas_y = point.y / zoom + pan.1;
            active_stroke.add_point(Point {
                x: canvas_x,
                y: canvas_y,
            });
            logger::debug(&format!(
                "Active stroke now has {} points",
                active_stroke.points().len()
            ));
            drop(state);
            if let Some(widget) = controller.widget()
                && let Some(gl_area) = widget.downcast_ref::<GLArea>()
            {
                gl_area.queue_render();
            }
            return;
        }

        // Handle slider dragging
        let active_slider = state.slider_drag;
        drop(state);

        if let Some((slider, value)) = ui::slider_drag(x as f32, y as f32, active_slider) {
            let mut state = render_state_motion.borrow_mut();
            match slider {
                SliderType::Hue => {
                    state.hue = value;
                    state.selected_custom_index = -1;
                }
                SliderType::Saturation => {
                    state.saturation = value;
                    state.selected_custom_index = -1;
                }
                SliderType::Value => {
                    state.value = value;
                    state.selected_custom_index = -1;
                }
            }
            drop(state);
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
