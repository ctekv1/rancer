//! Window management module for Rancer using winit + WGPU
//!
//! Provides window creation, mouse input handling, and WGPU rendering using winit.
//! This module handles the window lifecycle, input events, and GPU-accelerated rendering
//! using the canvas data model.

use std::cell::RefCell;
use std::rc::Rc;
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

#[cfg(windows)]
fn force_window_repaint(window: &Window) {
    use raw_window_handle::HasWindowHandle;
    use raw_window_handle::RawWindowHandle;

    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
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
}

/// Window application state using winit
pub struct WindowApp {
    /// The winit window
    window: Option<std::sync::Arc<Window>>,
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
}

impl WindowApp {
    /// Create a new window application
    pub fn new(preferences: Preferences) -> Self {
        logger::info("Creating winit window application...");

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
            },
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
            window.request_redraw();
            window.request_redraw();
            force_window_repaint(window);
        }
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
        let filename = crate::export_ui::default_export_filename();
        let handle = rfd::FileDialog::new()
            .set_file_name(&filename)
            .add_filter("PNG Image", &["png"])
            .save_file();

        let Some(path) = handle else { return };

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
                println!("Selected brush size: {}", size);
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
            ui::UiElement::Canvas => return false,
        }
        true
    }

    /// Handle keyboard input
    fn handle_keyboard(&mut self, key_event: &winit::event::KeyEvent) {
        if key_event.state != winit::event::ElementState::Pressed {
            return;
        }

        // Space key for panning
        if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) = key_event.logical_key
        {
            self.render_state.is_panning = true;
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
                        "z" | "Z" => {
                            let mut canvas = self.canvas.borrow_mut();
                            if canvas.can_undo() {
                                canvas.undo();
                                logger::info("Undo: removed last stroke");
                                println!("Undo: removed last stroke");
                                self.request_redraw();
                            }
                        }
                        "y" | "Y" => {
                            let mut canvas = self.canvas.borrow_mut();
                            if canvas.can_redo() {
                                canvas.redo();
                                logger::info("Redo: restored last undone stroke");
                                println!("Redo: restored last undone stroke");
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
                }
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                if key_event.state == winit::event::ElementState::Released {
                    self.render_state.is_panning = false;
                }
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

        // Drawing
        if self.render_state.mouse_state == MouseState::Drawing {
            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                let canvas_point = self.screen_to_canvas(screen_point.x, screen_point.y);
                active_stroke.add_point(canvas_point);
                println!(
                    "Active stroke now has {} points",
                    active_stroke.points().len()
                );
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

            let attributes = Window::default_attributes()
                .with_title(&self.preferences.window.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.preferences.window.width,
                    self.preferences.window.height,
                ));

            let window = event_loop.create_window(attributes).unwrap();
            let window = std::sync::Arc::new(window);

            self.scale_factor = window.scale_factor();
            logger::info("winit window created successfully");
            logger::info(&format!(
                "Window size: {}x{} (logical)",
                self.preferences.window.width, self.preferences.window.height
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

            let rt = tokio::runtime::Runtime::new().unwrap();
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
                        },
                        viewport: ViewportState {
                            zoom: self.render_state.zoom,
                            pan_offset: self.render_state.pan_offset,
                        },
                    };

                    match renderer.render(&frame) {
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
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                if button == winit::event::MouseButton::Left {
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            println!(
                                "Mouse button pressed at ({}, {})",
                                self.render_state.mouse_position.x,
                                self.render_state.mouse_position.y
                            );

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
                            drop(canvas);
                            let hit = ui::hit_test(
                                x,
                                y,
                                &self.render_state.custom_colors,
                                layer_count,
                                active_layer,
                                window_width,
                            );

                            if self.handle_ui_click(hit) {
                                self.request_redraw();
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
                                BrushType::default(),
                            );
                            println!(
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
                            );

                            *self.active_stroke.borrow_mut() = Some(active_stroke);

                            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                                let canvas_point = self.screen_to_canvas(
                                    self.render_state.mouse_position.x,
                                    self.render_state.mouse_position.y,
                                );
                                active_stroke.add_point(canvas_point);
                                println!(
                                    "Active stroke now has {} points",
                                    active_stroke.points().len()
                                );
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.render_state.mouse_state = MouseState::Idle;
                            self.render_state.slider_drag = None;

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
                } else if button == winit::event::MouseButton::Right {
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            self.render_state.is_eraser = true;
                            logger::info("Eraser mode: ON (right-click held)");
                            println!("Eraser mode: ON");
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.render_state.is_eraser = false;
                            logger::info("Eraser mode: OFF (right-click released)");
                            println!("Eraser mode: OFF");
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

    let event_loop = EventLoop::new().unwrap();
    let mut app = WindowApp::new(preferences);

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut app).unwrap();

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
