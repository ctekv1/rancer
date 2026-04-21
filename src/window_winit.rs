//! Window management module for Rancer using winit + OpenGL
//!
//! Provides window creation, mouse input handling, and OpenGL rendering using winit + glutin.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use crate::canvas::{ActiveStroke, BrushType, Canvas, Point};
use crate::logger;
use crate::preferences::Preferences;
use crate::renderer::{RenderFrame, Renderer, RendererConfig, UiRenderState, ViewportState};
use crate::ui::{self, SliderType};
use crate::window_backend::{MouseState as BackendMouseState, WindowBackend};

#[cfg(target_os = "linux")]
use crate::opengl_renderer::{GlRenderFrame, GlRenderer, GlUiState, GlViewportState};

#[cfg(windows)]
fn force_window_repaint(window: &Window) {
    use raw_window_handle::HasWindowHandle;
    use raw_window_handle::RawWindowHandle;

    if let Ok(handle) = window.window_handle()
        && let RawWindowHandle::Win32(h) = handle.as_raw()
    {
        let hwnd_val = h.hwnd.get();
        if hwnd_val != 0 {
            let hwnd = hwnd_val as *mut std::ffi::c_void;
            unsafe {
                unsafe extern "system" {
                    fn InvalidateRect(
                        hWnd: *mut std::ffi::c_void,
                        lpRect: *const std::ffi::c_void,
                        bErase: i32,
                    ) -> i32;
                    fn UpdateWindow(hWnd: *mut std::ffi::c_void) -> i32;
                }
                InvalidateRect(hwnd, std::ptr::null(), 0);
                UpdateWindow(hwnd);
            }
        }
    }
}

#[cfg(not(windows))]
fn force_window_repaint(_window: &Window) {}

/// Represents the current state of mouse interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseState {
    Idle,
    Drawing,
}

/// Consolidated UI/render state
struct WinitRenderState {
    hue: f32,
    saturation: f32,
    value: f32,
    custom_colors: Vec<[u8; 3]>,
    selected_custom_index: i32,
    brush_size: f32,
    opacity: f32,
    is_eraser: bool,
    slider_drag: Option<SliderType>,
    zoom: f32,
    pan_offset: (f32, f32),
    is_panning: bool,
    last_mouse_position: Point,
    mouse_state: MouseState,
    mouse_position: Point,
    active_layer: usize,
    brush_type: BrushType,
    selection_tool_active: bool,
    selection_drawing: bool,
    selection_start: Point,
    selection_moving: bool,
    selection_move_offset: (f32, f32),
}

/// Window application state using winit
pub struct WindowApp {
    /// The winit window
    window: Option<Arc<Window>>,
    /// WGPU renderer
    renderer: Option<Renderer>,
    /// Canvas for drawing operations
    canvas: Rc<RefCell<Canvas>>,
    /// Current active stroke being drawn
    active_stroke: Rc<RefCell<Option<ActiveStroke>>>,
    /// User preferences
    preferences: Preferences,
    /// Current keyboard modifiers state
    modifiers: ModifiersState,
    /// Scale factor for DPI scaling
    scale_factor: f64,
    /// Consolidated UI/render state
    render_state: WinitRenderState,
    /// Time tracking for animations
    start_time: Instant,
}

impl WindowApp {
    /// Create a new window application
    pub fn new(preferences: Preferences) -> Self {
        logger::info("Creating winit window application...");

        let brush_type = preferences.brush.default_type.parse().unwrap_or_default();

        Self {
            window: None,
            renderer: None,
            canvas: Rc::new(RefCell::new(Canvas::new())),
            active_stroke: Rc::new(RefCell::new(None)),
            preferences,
            modifiers: ModifiersState::empty(),
            scale_factor: 1.0,
            render_state: WinitRenderState {
                hue: 0.0,
                saturation: 0.0,
                value: 0.0,
                custom_colors: Vec::new(),
                selected_custom_index: -1,
                brush_size: 3.0,
                opacity: 1.0,
                is_eraser: false,
                slider_drag: None,
                zoom: 1.0,
                pan_offset: (0.0, 0.0),
                is_panning: false,
                last_mouse_position: Point { x: 0.0, y: 0.0 },
                mouse_state: MouseState::Idle,
                mouse_position: Point { x: 0.0, y: 0.0 },
                active_layer: 0,
                brush_type,
                selection_tool_active: false,
                selection_drawing: false,
                selection_start: Point { x: 0.0, y: 0.0 },
                selection_moving: false,
                selection_move_offset: (0.0, 0.0),
            },
            start_time: Instant::now(),
        }
    }

    /// Transform screen coordinates to canvas coordinates using current zoom/pan
    fn screen_to_canvas(&self, screen_x: f32, screen_y: f32) -> Point {
        let canvas_x = screen_x / self.render_state.zoom + self.render_state.pan_offset.0;
        let canvas_y = screen_y / self.render_state.zoom + self.render_state.pan_offset.1;
        Point {
            x: canvas_x,
            y: canvas_y,
        }
    }

    /// Request a redraw and apply Windows repaint workaround
    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
            force_window_repaint(window);
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn init_opengl_renderer(
        &mut self,
        _window: &Window,
        _width: u32,
        _height: u32,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Get current color from HSV values
    fn current_color(&self) -> crate::canvas::Color {
        crate::canvas::hsv_to_rgb(
            self.render_state.hue,
            self.render_state.saturation,
            self.render_state.value,
        )
    }

    /// Update HSV values in preferences and save
    fn update_hsv_preferences(&mut self) {
        self.preferences.palette.h = self.render_state.hue;
        self.preferences.palette.s = self.render_state.saturation;
        self.preferences.palette.v = self.render_state.value;
        self.preferences.palette.custom_colors = self.render_state.custom_colors.clone();
        let _ = crate::preferences::save(&self.preferences);
    }

    /// Export canvas to PNG file using a native save dialog
    #[cfg(target_os = "windows")]
    fn export_canvas_to_file(&self) {
        let Some(path) = crate::export_ui::show_save_dialog() else {
            return;
        };

        let canvas = self.canvas.borrow();
        match crate::export::export_to_png(&canvas, &path) {
            Ok(_) => {
                crate::export_ui::notify_export_result(true, &path, None);
            }
            Err(e) => {
                crate::export_ui::notify_export_result(false, &path, Some(&e.to_string()));
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn export_canvas_to_file(&self) {
        let filename = crate::export_ui::default_export_filename();
        let export_path = dirs::picture_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(filename);

        let canvas = self.canvas.borrow();
        match crate::export::export_to_png(&canvas, &export_path) {
            Ok(_) => {
                crate::export_ui::notify_export_result(true, &export_path, None);
            }
            Err(e) => {
                crate::export_ui::notify_export_result(false, &export_path, Some(&e.to_string()));
            }
        }
    }

    /// Handle UI element click — returns true if a UI element was hit
    fn handle_ui_click(&mut self, hit: ui::UiElement) -> bool {
        match hit {
            ui::UiElement::HueSlider(value) => {
                self.render_state.hue = value;
                self.render_state.selected_custom_index = -1;
                self.render_state.slider_drag = Some(SliderType::Hue);
                self.update_hsv_preferences();
            }
            ui::UiElement::SaturationSlider(value) => {
                self.render_state.saturation = value;
                self.render_state.selected_custom_index = -1;
                self.render_state.slider_drag = Some(SliderType::Saturation);
                self.update_hsv_preferences();
            }
            ui::UiElement::ValueSlider(value) => {
                self.render_state.value = value;
                self.render_state.selected_custom_index = -1;
                self.render_state.slider_drag = Some(SliderType::Value);
                self.update_hsv_preferences();
            }
            ui::UiElement::CustomColor(idx) => {
                self.render_state.selected_custom_index = idx as i32;
                let color = self.render_state.custom_colors[idx];
                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: 255,
                });
                self.render_state.hue = hsv.h;
                self.render_state.saturation = hsv.s;
                self.render_state.value = hsv.v;
                self.update_hsv_preferences();
            }
            ui::UiElement::SaveColor => {
                let current = crate::canvas::hsv_to_rgb(
                    self.render_state.hue,
                    self.render_state.saturation,
                    self.render_state.value,
                );
                if self.render_state.custom_colors.len() >= 10 {
                    self.render_state.custom_colors.remove(0);
                }
                self.render_state
                    .custom_colors
                    .push([current.r, current.g, current.b]);
                self.update_hsv_preferences();
            }
            ui::UiElement::BrushSize(size) => {
                self.render_state.brush_size = size;
                self.preferences.brush.default_size = size;
                let _ = crate::preferences::save(&self.preferences);
                logger::info(&format!("Selected brush size: {size}"));
            }
            ui::UiElement::Eraser => {
                self.render_state.is_eraser = !self.render_state.is_eraser;
                logger::info(&format!(
                    "Eraser mode: {}",
                    if self.render_state.is_eraser {
                        "ON"
                    } else {
                        "OFF"
                    }
                ));
            }
            ui::UiElement::Clear => {
                self.canvas.borrow_mut().clear();
                logger::info("Canvas cleared");
            }
            ui::UiElement::Undo => {
                let mut canvas = self.canvas.borrow_mut();
                if canvas.can_undo() {
                    canvas.undo();
                    logger::info("Undo: removed last stroke");
                }
            }
            ui::UiElement::Redo => {
                let mut canvas = self.canvas.borrow_mut();
                if canvas.can_redo() {
                    canvas.redo();
                    logger::info("Redo: restored last stroke");
                }
            }
            ui::UiElement::Export => {
                self.export_canvas_to_file();
            }
            ui::UiElement::ZoomIn => {
                let new_zoom = (self.render_state.zoom * 1.25).min(10.0);
                if (new_zoom - self.render_state.zoom).abs() > 0.001 {
                    self.render_state.zoom = new_zoom;
                }
            }
            ui::UiElement::ZoomOut => {
                let new_zoom = (self.render_state.zoom / 1.25).max(0.1);
                if (new_zoom - self.render_state.zoom).abs() > 0.001 {
                    self.render_state.zoom = new_zoom;
                }
            }
            ui::UiElement::Opacity(opacity) => {
                self.render_state.opacity = opacity;
                self.preferences.brush.default_opacity = opacity;
                let _ = crate::preferences::save(&self.preferences);
                logger::info(&format!("Opacity: {}", opacity));
            }
            ui::UiElement::BrushType(brush_type) => {
                self.render_state.brush_type = brush_type;
                self.preferences.brush.default_type = format!("{:?}", brush_type);
                let _ = crate::preferences::save(&self.preferences);
                logger::info(&format!("Brush type: {:?}", brush_type));
                self.request_redraw();
            }
            ui::UiElement::LayerRow(index) => {
                if self.canvas.borrow_mut().set_active_layer(index).is_ok() {
                    self.render_state.active_layer = index;
                }
            }
            ui::UiElement::LayerVisibility(index) => {
                let _ = self.canvas.borrow_mut().toggle_layer_visibility(index);
            }
            ui::UiElement::AddLayer => {
                let mut canvas = self.canvas.borrow_mut();
                if canvas.layer_count() < crate::canvas::MAX_LAYERS {
                    let _ = canvas.add_layer(None);
                    let new_index = canvas.layer_count() - 1;
                    let _ = canvas.set_active_layer(new_index);
                    self.render_state.active_layer = new_index;
                }
            }
            ui::UiElement::DeleteLayer => {
                let mut canvas = self.canvas.borrow_mut();
                if canvas.layer_count() > 1 {
                    let _ = canvas.remove_layer(self.render_state.active_layer);
                    self.render_state.active_layer = canvas.active_layer();
                }
            }
            ui::UiElement::MoveLayerUp => {
                let mut canvas = self.canvas.borrow_mut();
                let idx = self.render_state.active_layer;
                if idx > 0 {
                    let _ = canvas.move_layer(idx, idx - 1);
                    self.render_state.active_layer = canvas.active_layer();
                }
            }
            ui::UiElement::MoveLayerDown => {
                let mut canvas = self.canvas.borrow_mut();
                let idx = self.render_state.active_layer;
                if idx < canvas.layer_count() - 1 {
                    let _ = canvas.move_layer(idx, idx + 1);
                    self.render_state.active_layer = canvas.active_layer();
                }
            }
            ui::UiElement::SelectionTool => {
                self.render_state.selection_tool_active = !self.render_state.selection_tool_active;
                if !self.render_state.selection_tool_active {
                    self.canvas.borrow_mut().clear_selection();
                }
                self.request_redraw();
            }
            ui::UiElement::SelectionRect => {
                // Start moving/copying selection
                let canvas_point = self.screen_to_canvas(
                    self.render_state.mouse_position.x,
                    self.render_state.mouse_position.y,
                );
                self.render_state.selection_moving = true;
                self.render_state.selection_move_offset = (canvas_point.x, canvas_point.y);
                self.request_redraw();
            }
            ui::UiElement::SelectionStart(_, _) => {
                // Start drawing selection rect — handled in cursor_moved
            }
            ui::UiElement::Canvas => return false,
        }
        true
    }

    /// Handle keyboard input
    fn handle_keyboard(&mut self, key_event: &winit::event::KeyEvent) {
        // Space toggles panning — handle both press and release before the
        // pressed-only guard below.
        if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) = key_event.logical_key
        {
            if key_event.state == winit::event::ElementState::Pressed {
                // Sync last_mouse_position so the first panning delta is zero.
                self.render_state.last_mouse_position = self.render_state.mouse_position;
                self.render_state.is_panning = true;
            } else {
                self.render_state.is_panning = false;
            }
            return;
        }

        if key_event.state != winit::event::ElementState::Pressed {
            return;
        }

        // 's' key for export
        if let winit::keyboard::Key::Character(ref c) = key_event.logical_key
            && c == "s"
        {
            self.export_canvas_to_file();
            return;
        }

        match key_event.logical_key.as_ref() {
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                if !self.render_state.custom_colors.is_empty() {
                    let new_index = if self.render_state.selected_custom_index < 0 {
                        0
                    } else {
                        ((self.render_state.selected_custom_index as usize + 1)
                            % self.render_state.custom_colors.len()) as i32
                    };
                    self.render_state.selected_custom_index = new_index;
                    let color = self.render_state.custom_colors[new_index as usize];
                    let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: 255,
                    });
                    self.render_state.hue = hsv.h;
                    self.render_state.saturation = hsv.s;
                    self.render_state.value = hsv.v;
                    self.update_hsv_preferences();
                    self.request_redraw();
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                if !self.render_state.custom_colors.is_empty() {
                    let new_index = if self.render_state.selected_custom_index <= 0 {
                        self.render_state.custom_colors.len() as i32 - 1
                    } else {
                        self.render_state.selected_custom_index - 1
                    };
                    self.render_state.selected_custom_index = new_index;
                    let color = self.render_state.custom_colors[new_index as usize];
                    let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: 255,
                    });
                    self.render_state.hue = hsv.h;
                    self.render_state.saturation = hsv.s;
                    self.render_state.value = hsv.v;
                    self.update_hsv_preferences();
                    self.request_redraw();
                }
            }
            winit::keyboard::Key::Character(c) => {
                let c_str: &str = c;
                if self
                    .modifiers
                    .contains(winit::keyboard::ModifiersState::CONTROL)
                {
                    match c_str {
                        "c" | "C" => {
                            if self.canvas.borrow().has_selection() {
                                self.canvas.borrow_mut().copy_selection();
                                logger::info("Selection copied");
                                self.request_redraw();
                            }
                        }
                        "d" | "D" => {
                            self.canvas.borrow_mut().clear_selection();
                            self.request_redraw();
                        }
                        "z" | "Z" => {
                            let mut canvas = self.canvas.borrow_mut();
                            if canvas.can_undo() {
                                canvas.undo();
                                logger::info("Undo: removed last stroke");
                                self.request_redraw();
                            }
                        }
                        "y" | "Y" => {
                            let mut canvas = self.canvas.borrow_mut();
                            if canvas.can_redo() {
                                canvas.redo();
                                logger::info("Redo: restored last undone stroke");
                                self.request_redraw();
                            }
                        }
                        _ => {}
                    }
                } else {
                    match c_str {
                        "e" | "E" => {
                            if self.render_state.mouse_state != MouseState::Drawing {
                                self.render_state.is_eraser = !self.render_state.is_eraser;
                                logger::info(&format!(
                                    "Eraser mode: {}",
                                    if self.render_state.is_eraser {
                                        "ON"
                                    } else {
                                        "OFF"
                                    }
                                ));
                                self.request_redraw();
                            }
                        }
                        "+" | "=" => {
                            self.render_state.brush_size =
                                crate::canvas::brush_size_up(self.render_state.brush_size);
                            self.preferences.brush.default_size = self.render_state.brush_size;
                            let _ = crate::preferences::save(&self.preferences);
                            logger::info(&format!("Brush size: {}", self.render_state.brush_size));
                            self.request_redraw();
                        }
                        "-" | "_" => {
                            self.render_state.brush_size =
                                crate::canvas::brush_size_down(self.render_state.brush_size);
                            self.preferences.brush.default_size = self.render_state.brush_size;
                            let _ = crate::preferences::save(&self.preferences);
                            logger::info(&format!("Brush size: {}", self.render_state.brush_size));
                            self.request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) => {
                if self
                    .modifiers
                    .contains(winit::keyboard::ModifiersState::CONTROL)
                {
                    self.canvas.borrow_mut().clear();
                    logger::info("Canvas cleared");
                    self.request_redraw();
                } else if self.canvas.borrow().has_selection() {
                    self.canvas.borrow_mut().commit_selection();
                    logger::info("Selection committed");
                    self.request_redraw();
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                self.canvas.borrow_mut().clear_selection();
                self.render_state.selection_tool_active = false;
                self.request_redraw();
            }
            _ => {}
        }
    }

    /// Handle cursor motion
    fn handle_cursor_moved(&mut self, screen_point: Point) {
        self.render_state.mouse_position = screen_point;

        // Handle panning
        if self.render_state.is_panning {
            let dx = screen_point.x - self.render_state.last_mouse_position.x;
            let dy = screen_point.y - self.render_state.last_mouse_position.y;
            let (old_pan_x, old_pan_y) = self.render_state.pan_offset;
            self.render_state.pan_offset = (
                old_pan_x - dx / self.render_state.zoom,
                old_pan_y - dy / self.render_state.zoom,
            );
            self.render_state.last_mouse_position = screen_point;
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            return;
        }

        self.render_state.last_mouse_position = screen_point;

        // Selection rect drawing
        if self.render_state.selection_drawing {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            return;
        }

        // Selection moving
        if self.render_state.selection_moving {
            let start = self.render_state.selection_move_offset;
            let current = self.screen_to_canvas(screen_point.x, screen_point.y);
            let dx = current.x - start.0;
            let dy = current.y - start.1;
            self.canvas.borrow_mut().move_selection(dx, dy);
            self.render_state.selection_move_offset = (current.x, current.y);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            return;
        }

        // Drawing
        if self.render_state.mouse_state == MouseState::Drawing {
            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                let canvas_point = self.screen_to_canvas(screen_point.x, screen_point.y);
                active_stroke.add_point(canvas_point);
                logger::debug(&format!(
                    "Active stroke now has {} points",
                    active_stroke.points().len()
                ));
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            return;
        }

        // Slider dragging
        if let Some((slider, value)) = ui::slider_drag(
            screen_point.x,
            screen_point.y,
            self.render_state.slider_drag,
        ) {
            match slider {
                SliderType::Hue => {
                    self.render_state.hue = value;
                    self.render_state.selected_custom_index = -1;
                }
                SliderType::Saturation => {
                    self.render_state.saturation = value;
                    self.render_state.selected_custom_index = -1;
                }
                SliderType::Value => {
                    self.render_state.value = value;
                    self.render_state.selected_custom_index = -1;
                }
            }
            self.update_hsv_preferences();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            logger::info("=== WINDOW CREATION ===");

            // Determine window size: use saved preferences if valid, otherwise
            // fill the primary monitor's full resolution.
            let saved_w = self.preferences.window.width;
            let saved_h = self.preferences.window.height;
            let (window_width, window_height) = if saved_w > 0 && saved_h > 0 {
                (saved_w, saved_h)
            } else if let Some(monitor) = event_loop.primary_monitor() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                (
                    (size.width as f64 / scale) as u32,
                    (size.height as f64 / scale) as u32,
                )
            } else {
                (1280, 720)
            };

            let attributes = Window::default_attributes()
                .with_title(&self.preferences.window.title)
                .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height));

            let window = match event_loop.create_window(attributes) {
                Ok(w) => w,
                Err(e) => {
                    logger::error(&format!("Failed to create window: {}", e));
                    event_loop.exit();
                    return;
                }
            };
            let window = std::sync::Arc::new(window);

            self.scale_factor = window.scale_factor();
            logger::info("winit window created successfully");
            logger::info(&format!(
                "Window size: {}x{} (logical)",
                window_width, window_height
            ));
            logger::info(&format!(
                "Window physical size: {}x{} (with DPI scaling)",
                window.inner_size().width,
                window.inner_size().height
            ));
            logger::info(&format!("Window scale factor: {}", self.scale_factor));
            logger::info(&format!("Window title: {}", self.preferences.window.title));
            logger::info("========================");

            // Initialize WGPU renderer
            logger::info("=== RENDERER INITIALIZATION ===");

            let size = window.inner_size();
            let config = RendererConfig::default();

            let render_width = if size.width > 0 {
                size.width
            } else {
                self.preferences.window.width
            };
            let render_height = if size.height > 0 {
                size.height
            } else {
                self.preferences.window.height
            };

            if size.width == 0 || size.height == 0 {
                logger::warn(&format!(
                    "Window inner_size reported as {}x{}, falling back to preferences {}x{}",
                    size.width, size.height, render_width, render_height
                ));
            }

            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    logger::error(&format!("Failed to create tokio runtime: {e}"));
                    return;
                }
            };
            match rt.block_on(Renderer::new(
                config,
                window.clone(),
                (render_width, render_height),
            )) {
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
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_factor = 1.25;
                let old_zoom = self.render_state.zoom;

                let new_zoom = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, dy) => {
                        if dy > 0.0 {
                            (old_zoom * zoom_factor).min(10.0)
                        } else if dy < 0.0 {
                            (old_zoom / zoom_factor).max(0.1)
                        } else {
                            old_zoom
                        }
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {
                        if delta.y > 0.0 {
                            (old_zoom * zoom_factor).min(10.0)
                        } else if delta.y < 0.0 {
                            (old_zoom / zoom_factor).max(0.1)
                        } else {
                            old_zoom
                        }
                    }
                };

                if (new_zoom - old_zoom).abs() > 0.001 {
                    let mouse_canvas_x = self.render_state.mouse_position.x / old_zoom
                        + self.render_state.pan_offset.0;
                    let mouse_canvas_y = self.render_state.mouse_position.y / old_zoom
                        + self.render_state.pan_offset.1;

                    let new_pan_x = mouse_canvas_x - self.render_state.mouse_position.x / new_zoom;
                    let new_pan_y = mouse_canvas_y - self.render_state.mouse_position.y / new_zoom;

                    self.render_state.zoom = new_zoom;
                    self.render_state.pan_offset = (new_pan_x, new_pan_y);

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref window) = self.window {
                    self.scale_factor = window.scale_factor();
                }

                logger::info(&format!(
                    "Window resized to {}x{} (physical)",
                    physical_size.width, physical_size.height
                ));

                let logical_width = (physical_size.width as f64 / self.scale_factor) as u32;
                let logical_height = (physical_size.height as f64 / self.scale_factor) as u32;

                logger::info(&format!(
                    "Window resized to {}x{} (logical)",
                    logical_width, logical_height
                ));

                if logical_width == 0 || logical_height == 0 {
                    return;
                }
                self.preferences.window.width = logical_width;
                self.preferences.window.height = logical_height;
                self.preferences.canvas.width = logical_width;
                self.preferences.canvas.height = logical_height;

                if let Err(e) = crate::preferences::save(&self.preferences) {
                    logger::error(&format!("Failed to save preferences: {}", e));
                }

                if let Some(renderer) = &mut self.renderer {
                    renderer.resize((physical_size.width, physical_size.height));
                }
                {
                    let mut canvas = self.canvas.borrow_mut();
                    canvas.resize(physical_size.width, physical_size.height);
                }
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let has_selection = self.canvas.borrow().selection().is_some()
                    || self.render_state.selection_drawing;

                // Request another redraw for animation if selection is active
                if has_selection {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }

                let selection_rect = {
                    let canvas_rect = self.canvas.borrow().selection().map(|s| s.rect);
                    if self.render_state.selection_drawing {
                        let start = self.render_state.selection_start;
                        let zoom = self.render_state.zoom;
                        let pan = self.render_state.pan_offset;
                        let mx = self.render_state.mouse_position.x;
                        let my = self.render_state.mouse_position.y;
                        let end_x = mx / zoom + pan.0;
                        let end_y = my / zoom + pan.1;
                        Some(crate::canvas::Rect::new(
                            start.x,
                            start.y,
                            end_x - start.x,
                            end_y - start.y,
                        ))
                    } else {
                        canvas_rect
                    }
                };

                if let Some(renderer) = &mut self.renderer {
                    let canvas = self.canvas.borrow();
                    let active_stroke = self.active_stroke.borrow();

                    let frame = RenderFrame {
                        canvas: &canvas,
                        active_stroke: active_stroke.as_ref(),
                        ui: UiRenderState {
                            hue: self.render_state.hue,
                            saturation: self.render_state.saturation,
                            value: self.render_state.value,
                            custom_colors: &self.render_state.custom_colors,
                            selected_custom_index: self.render_state.selected_custom_index,
                            brush_size: self.render_state.brush_size,
                            opacity: self.render_state.opacity,
                            is_eraser: self.render_state.is_eraser,
                            brush_type: self.render_state.brush_type,
                            selection_tool_active: self.render_state.selection_tool_active,
                            selection_rect,
                            selection_time: self.start_time.elapsed().as_secs_f32(),
                            selected_strokes: canvas.selection().map(|s| s.strokes.as_slice()),
                        },
                        viewport: ViewportState {
                            zoom: self.render_state.zoom,
                            pan_offset: self.render_state.pan_offset,
                        },
                    };

                    if let Err(e) = renderer.render(&frame) {
                        logger::error(&format!("Render error: {:?}", e));
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                if button == winit::event::MouseButton::Left {
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            logger::info(&format!(
                                "Mouse button pressed at ({}, {})",
                                self.render_state.mouse_position.x,
                                self.render_state.mouse_position.y
                            ));

                            let x = self.render_state.mouse_position.x;
                            let y = self.render_state.mouse_position.y;
                            let window_width = self
                                .window
                                .as_ref()
                                .map(|w| w.inner_size().width as f32)
                                .unwrap_or(1280.0);
                            let canvas = self.canvas.borrow();
                            let layer_count = canvas.layer_count();
                            let active_layer = canvas.active_layer();
                            let selection_rect_hit = canvas.selection().map(|s| s.rect);
                            drop(canvas);

                            let zoom = self.render_state.zoom;
                            let pan = self.render_state.pan_offset;
                            let selection_rect_screen = selection_rect_hit.map(|r| {
                                crate::canvas::Rect::new(
                                    (r.x - pan.0) * zoom,
                                    (r.y - pan.1) * zoom,
                                    r.w * zoom,
                                    r.h * zoom,
                                )
                            });

                            let hit = ui::hit_test(
                                x,
                                y,
                                &self.render_state.custom_colors,
                                layer_count,
                                active_layer,
                                window_width,
                                selection_rect_screen,
                            );

                            if self.handle_ui_click(hit) {
                                self.request_redraw();
                                return;
                            }

                            // If selection tool is active and we clicked on canvas
                            if self.render_state.selection_tool_active {
                                // If there's an existing selection and we're not clicking on it, clear it first
                                if self.canvas.borrow().has_selection() {
                                    self.canvas.borrow_mut().clear_selection();
                                    self.request_redraw();
                                }
                                let canvas_point = self.screen_to_canvas(x, y);
                                self.render_state.selection_drawing = true;
                                self.render_state.selection_start = canvas_point;
                                return;
                            }

                            // Start drawing
                            if self.canvas.borrow().is_active_layer_locked() {
                                return;
                            }
                            self.render_state.mouse_state = MouseState::Drawing;

                            let color = if self.render_state.is_eraser {
                                crate::canvas::Color::WHITE
                            } else {
                                self.current_color()
                            };
                            let mut canvas = self.canvas.borrow_mut();
                            let active_stroke = canvas.begin_stroke(
                                color,
                                self.render_state.brush_size,
                                self.render_state.opacity,
                                self.render_state.brush_type,
                            );
                            logger::info(&format!(
                                "Created {}stroke with color RGB({}, {}, {}) and width {}",
                                if self.render_state.is_eraser {
                                    "eraser "
                                } else {
                                    ""
                                },
                                color.r,
                                color.g,
                                color.b,
                                self.render_state.brush_size
                            ));

                            *self.active_stroke.borrow_mut() = Some(active_stroke);

                            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                                let canvas_point = self.screen_to_canvas(
                                    self.render_state.mouse_position.x,
                                    self.render_state.mouse_position.y,
                                );
                                active_stroke.add_point(canvas_point);
                                logger::debug(&format!(
                                    "Active stroke now has {} points",
                                    active_stroke.points().len()
                                ));
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.render_state.mouse_state = MouseState::Idle;
                            self.render_state.slider_drag = None;

                            // Stop selection moving
                            if self.render_state.selection_moving {
                                self.render_state.selection_moving = false;
                                self.request_redraw();
                            }

                            // If we were drawing a selection rect, commit it
                            if self.render_state.selection_drawing {
                                self.render_state.selection_drawing = false;
                                let start = self.render_state.selection_start;
                                let end = self.screen_to_canvas(
                                    self.render_state.mouse_position.x,
                                    self.render_state.mouse_position.y,
                                );
                                let rect = crate::canvas::Rect::new(
                                    start.x,
                                    start.y,
                                    end.x - start.x,
                                    end.y - start.y,
                                );
                                self.canvas.borrow_mut().begin_selection(rect);
                                self.request_redraw();
                            }

                            if let Some(active_stroke) = self.active_stroke.borrow_mut().take() {
                                let mut canvas = self.canvas.borrow_mut();
                                if let Err(e) = canvas.commit_stroke(active_stroke) {
                                    logger::error(&format!("Failed to commit stroke: {e}"));
                                } else {
                                    logger::info("Stroke committed successfully");
                                }
                            }
                        }
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                } else if button == winit::event::MouseButton::Right {
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            self.render_state.is_eraser = true;
                            logger::info("Eraser mode: ON (right-click held)");
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.render_state.is_eraser = false;
                            logger::info("Eraser mode: OFF (right-click released)");
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let screen_point = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                self.handle_cursor_moved(screen_point);
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                self.handle_keyboard(&key_event);
            }
            _ => {}
        }
    }
}

impl WindowBackend for WindowApp {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn run(&self) {}

    fn canvas(&self) -> &Rc<RefCell<Canvas>> {
        &self.canvas
    }

    fn mouse_position(&self) -> Point {
        self.render_state.mouse_position
    }

    fn mouse_state(&self) -> BackendMouseState {
        match self.render_state.mouse_state {
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

/// Run the winit event loop with the window application
pub fn run_window_app(preferences: Preferences) {
    logger::info("Starting winit event loop...");

    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            logger::error(&format!("Failed to create event loop: {e}"));
            return;
        }
    };
    let max_fps = preferences.renderer.max_fps;
    let mut app = WindowApp::new(preferences);
    if max_fps == 0 {
        event_loop.set_control_flow(ControlFlow::Poll);
    } else {
        let target_duration = std::time::Duration::from_secs_f64(1.0 / max_fps as f64);
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + target_duration,
        ));
    }

    if let Err(e) = event_loop.run_app(&mut app) {
        logger::error(&format!("Event loop exited with error: {e}"));
    }

    logger::info("Rancer application closed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::Preferences;

    #[test]
    fn test_window_app_creation() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);
        assert_eq!(app.render_state.mouse_state, MouseState::Idle);
        assert_eq!(app.render_state.brush_size, 3.0);
    }

    #[test]
    fn test_mouse_state_transitions() {
        let mut mouse_state = MouseState::Idle;
        assert_eq!(mouse_state, MouseState::Idle);

        mouse_state = MouseState::Drawing;
        assert_eq!(mouse_state, MouseState::Drawing);
    }

    #[test]
    fn test_window_app_initial_state() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);

        assert!(!app.render_state.is_eraser);
        assert!(app.window.is_none());
        assert!(app.renderer.is_none());
    }

    #[test]
    fn test_mouse_position_initial() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);
        assert_eq!(app.render_state.mouse_position.x, 0.0);
        assert_eq!(app.render_state.mouse_position.y, 0.0);
    }

    #[test]
    fn test_canvas_access() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);
        assert_eq!(app.canvas.borrow().layer_count(), 1);
    }

    #[test]
    fn test_zero_size_window_guard() {
        let size = winit::dpi::PhysicalSize::new(0u32, 0u32);
        let preferences = Preferences::default();
        let render_width = if size.width > 0 {
            size.width
        } else {
            preferences.window.width
        };
        let render_height = if size.height > 0 {
            size.height
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, preferences.window.width);
        assert_eq!(render_height, preferences.window.height);
    }

    #[test]
    fn test_partial_zero_size_guard() {
        let size = winit::dpi::PhysicalSize::new(0u32, 768u32);
        let preferences = Preferences::default();
        let render_width = if size.width > 0 {
            size.width
        } else {
            preferences.window.width
        };
        let render_height = if size.height > 0 {
            size.height
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, preferences.window.width);
        assert_eq!(render_height, 768);
    }

    #[test]
    fn test_nonzero_size_window_uses_actual_size() {
        let size = winit::dpi::PhysicalSize::new(1280u32, 720u32);
        let preferences = Preferences::default();
        let render_width = if size.width > 0 {
            size.width
        } else {
            preferences.window.width
        };
        let render_height = if size.height > 0 {
            size.height
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, 1280);
        assert_eq!(render_height, 720);
    }
}
