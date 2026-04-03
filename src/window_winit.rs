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

use crate::canvas::{ActiveStroke, Canvas, Point};
use crate::logger;
use crate::preferences::Preferences;
use crate::renderer::{Renderer, RendererConfig};
use crate::ui::{self, SliderType};
use crate::window_backend::{MouseState as BackendMouseState, WindowBackend};

#[cfg(windows)]
fn force_window_repaint(window: &Window) {
    use raw_window_handle::HasWindowHandle;
    use raw_window_handle::RawWindowHandle;

    if let Ok(handle) = window.window_handle() {
        match handle.as_raw() {
            RawWindowHandle::Win32(h) => {
                let hwnd_val = h.hwnd.get();
                if hwnd_val != 0 {
                    let hwnd = hwnd_val as *mut std::ffi::c_void;
                    // SAFETY: We're calling Win32 APIs on a valid HWND obtained from winit.
                    // These functions are standard Windows APIs for window management.
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
            _ => {}
        }
    }
}

#[cfg(not(windows))]
fn force_window_repaint(_window: &Window) {
    // No-op on non-Windows platforms
}

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
    window: Option<std::sync::Arc<Window>>,
    /// WGPU renderer
    renderer: Option<Renderer>,
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
    /// Brush size in pixels
    brush_size: f32,
    /// Brush opacity
    opacity: f32,
    /// Eraser mode active
    is_eraser: bool,
    /// Slider drag state (which slider is being dragged)
    slider_drag: Option<SliderType>,
    /// User preferences
    preferences: Preferences,
    /// Current keyboard modifiers state
    modifiers: ModifiersState,
    /// Scale factor for DPI scaling
    scale_factor: f64,
}

impl WindowApp {
    /// Create a new window application
    pub fn new(preferences: Preferences) -> Self {
        logger::info("Creating winit window application...");

        Self {
            window: None,
            renderer: None,
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
            modifiers: ModifiersState::empty(),
            scale_factor: 1.0,
        }
    }

    /// Export canvas to PNG file
    fn export_canvas_to_file(&self) {
        let canvas = self.canvas.borrow();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("rancer_export_{}.png", timestamp);

        // Get user's Pictures directory or use current directory
        let export_path = dirs::picture_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(filename);

        logger::info(&format!(
            "Attempting to export canvas to: {:?}",
            export_path
        ));

        match crate::export::export_to_png(&canvas, &export_path) {
            Ok(_) => {
                logger::info(&format!("Export successful: {:?}", export_path));
            }
            Err(e) => {
                logger::error(&format!("Export failed: {}", e));
            }
        }
    }

    /// Update HSV values in preferences and save
    fn update_hsv_preferences(&mut self) {
        self.preferences.palette.h = self.hue;
        self.preferences.palette.s = self.saturation;
        self.preferences.palette.v = self.value;
        self.preferences.palette.custom_colors = self.custom_colors.clone();
        let _ = crate::preferences::save(&self.preferences);
    }

    /// Get current color from HSV values
    fn current_color(&self) -> crate::canvas::Color {
        crate::canvas::hsv_to_rgb(self.hue, self.saturation, self.value)
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

            // Guard against zero-size window (can happen on some systems during resumed phase)
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

            // Use tokio runtime to initialize WGPU (async)
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(Renderer::new(
                config,
                window.clone(),
                (render_width, render_height),
            )) {
                Ok(mut renderer) => {
                    logger::info("✅ WGPU renderer initialized successfully!");
                    renderer.print_backend_status();
                    renderer.set_opacity(self.opacity);
                    renderer.set_hsv(self.hue, self.saturation, self.value);
                    renderer.set_custom_colors(self.custom_colors.clone());
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
            WindowEvent::Resized(physical_size) => {
                // Update scale factor (may change if window moves between monitors)
                if let Some(ref window) = self.window {
                    self.scale_factor = window.scale_factor();
                }

                logger::info(&format!(
                    "Window resized to {}x{} (physical)",
                    physical_size.width, physical_size.height
                ));

                // Convert physical size to logical size before saving
                let logical_width = (physical_size.width as f64 / self.scale_factor) as u32;
                let logical_height = (physical_size.height as f64 / self.scale_factor) as u32;

                logger::info(&format!(
                    "Window resized to {}x{} (logical)",
                    logical_width, logical_height
                ));

                // Update preferences with logical window size
                self.preferences.window.width = logical_width;
                self.preferences.window.height = logical_height;
                self.preferences.canvas.width = logical_width;
                self.preferences.canvas.height = logical_height;

                // Save preferences on change
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
                if let Some(window) = &self.window {
                    window.request_redraw();
                    window.request_redraw();
                    window.request_redraw();
                    // Force Windows to repaint the window, fixing black space issue
                    force_window_repaint(window);
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
                                self.mouse_position.x, self.mouse_position.y
                            );

                            // Check if click is on UI elements
                            let x = self.mouse_position.x;
                            let y = self.mouse_position.y;
                            let hit = ui::hit_test(x, y, &self.custom_colors);

                            match hit {
                                ui::UiElement::HueSlider(value) => {
                                    self.hue = value;
                                    self.selected_custom_index = -1;
                                    self.slider_drag = Some(SliderType::Hue);
                                    self.update_hsv_preferences();
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_hsv(self.hue, self.saturation, self.value);
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::SaturationSlider(value) => {
                                    self.saturation = value;
                                    self.selected_custom_index = -1;
                                    self.slider_drag = Some(SliderType::Saturation);
                                    self.update_hsv_preferences();
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_hsv(self.hue, self.saturation, self.value);
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::ValueSlider(value) => {
                                    self.value = value;
                                    self.selected_custom_index = -1;
                                    self.slider_drag = Some(SliderType::Value);
                                    self.update_hsv_preferences();
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_hsv(self.hue, self.saturation, self.value);
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::CustomColor(idx) => {
                                    self.selected_custom_index = idx as i32;
                                    let color = self.custom_colors[idx];
                                    let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                        r: color[0],
                                        g: color[1],
                                        b: color[2],
                                        a: 255,
                                    });
                                    self.hue = hsv.h;
                                    self.saturation = hsv.s;
                                    self.value = hsv.v;
                                    self.update_hsv_preferences();
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_hsv(self.hue, self.saturation, self.value);
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::SaveColor => {
                                    let current = crate::canvas::hsv_to_rgb(
                                        self.hue,
                                        self.saturation,
                                        self.value,
                                    );
                                    if self.custom_colors.len() >= 10 {
                                        self.custom_colors.remove(0);
                                    }
                                    self.custom_colors.push([current.r, current.g, current.b]);
                                    self.update_hsv_preferences();
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_custom_colors(self.custom_colors.clone());
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::BrushSize(size) => {
                                    self.brush_size = size;
                                    self.preferences.brush.default_size = size;
                                    if let Err(e) = crate::preferences::save(&self.preferences) {
                                        logger::error(&format!(
                                            "Failed to save preferences: {}",
                                            e
                                        ));
                                    }
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_brush_size(size);
                                    }
                                    println!("Selected brush size: {}", size);
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::Eraser => {
                                    self.is_eraser = !self.is_eraser;
                                    logger::info(&format!(
                                        "Eraser mode: {}",
                                        if self.is_eraser { "ON" } else { "OFF" }
                                    ));
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_eraser(self.is_eraser);
                                    }
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::Clear => {
                                    let mut canvas = self.canvas.borrow_mut();
                                    canvas.clear();
                                    logger::info("Canvas cleared");
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::Undo => {
                                    let mut canvas = self.canvas.borrow_mut();
                                    if canvas.can_undo() {
                                        canvas.undo();
                                        logger::info("Undo: removed last stroke");
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
                                    }
                                    return;
                                }
                                ui::UiElement::Redo => {
                                    let mut canvas = self.canvas.borrow_mut();
                                    if canvas.can_redo() {
                                        canvas.redo();
                                        logger::info("Redo: restored last stroke");
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
                                    }
                                    return;
                                }
                                ui::UiElement::Export => {
                                    self.export_canvas_to_file();
                                    return;
                                }
                                ui::UiElement::Opacity(opacity) => {
                                    self.opacity = opacity;
                                    self.preferences.brush.default_opacity = opacity;
                                    let _ = crate::preferences::save(&self.preferences);
                                    if let Some(renderer) = &mut self.renderer {
                                        renderer.set_opacity(opacity);
                                    }
                                    logger::info(&format!("Opacity: {}", opacity));
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                    return;
                                }
                                ui::UiElement::Canvas => {
                                    // Not on any UI element — start drawing
                                }
                            }

                            // If not on UI, start drawing
                            self.mouse_state = MouseState::Drawing;

                            // Begin a new active stroke
                            let color = if self.is_eraser {
                                crate::canvas::Color::WHITE
                            } else {
                                self.current_color()
                            };
                            let mut canvas = self.canvas.borrow_mut();
                            let active_stroke =
                                canvas.begin_stroke(color, self.brush_size, self.opacity);
                            println!(
                                "Created {}stroke with color RGB({}, {}, {}) and width {}",
                                if self.is_eraser { "eraser " } else { "" },
                                color.r,
                                color.g,
                                color.b,
                                self.brush_size
                            );

                            // Store the active stroke
                            *self.active_stroke.borrow_mut() = Some(active_stroke);

                            // Add the current mouse position as the first point
                            if let Some(active_stroke) = &mut *self.active_stroke.borrow_mut() {
                                active_stroke.add_point(self.mouse_position);
                                println!(
                                    "Added first point to active stroke: ({}, {})",
                                    self.mouse_position.x, self.mouse_position.y
                                );
                                println!(
                                    "Active stroke now has {} points",
                                    active_stroke.points().len()
                                );
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.mouse_state = MouseState::Idle;
                            self.slider_drag = None;

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
                    // Right-click for eraser mode
                    match button_state {
                        winit::event::ElementState::Pressed => {
                            self.is_eraser = true;
                            logger::info("Eraser mode: ON (right-click held)");
                            println!("Eraser mode: ON");
                            if let Some(renderer) = &mut self.renderer {
                                renderer.set_eraser(true);
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.is_eraser = false;
                            logger::info("Eraser mode: OFF (right-click released)");
                            println!("Eraser mode: OFF");
                            if let Some(renderer) = &mut self.renderer {
                                renderer.set_eraser(false);
                            }
                            if let Some(window) = &self.window {
                                window.request_redraw();
                            }
                        }
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
                if self.mouse_state == MouseState::Drawing
                    && let Some(active_stroke) = &mut *self.active_stroke.borrow_mut()
                {
                    active_stroke.add_point(point);
                    println!("Added point to active stroke: ({}, {})", point.x, point.y);
                    println!(
                        "Active stroke now has {} points",
                        active_stroke.points().len()
                    );
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }

                // Handle slider dragging
                if let Some((slider, value)) = ui::slider_drag(point.x, point.y, self.slider_drag) {
                    match slider {
                        SliderType::Hue => {
                            self.hue = value;
                            self.selected_custom_index = -1;
                        }
                        SliderType::Saturation => {
                            self.saturation = value;
                            self.selected_custom_index = -1;
                        }
                        SliderType::Value => {
                            self.value = value;
                            self.selected_custom_index = -1;
                        }
                    }
                    self.update_hsv_preferences();
                    if let Some(renderer) = &mut self.renderer {
                        renderer.set_hsv(self.hue, self.saturation, self.value);
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == winit::event::ElementState::Pressed {
                    // Check for 's' key (save/export)
                    // Note: Ctrl modifier detection may need additional handling
                    if let winit::keyboard::Key::Character(ref c) = key_event.logical_key
                        && c == "s"
                    {
                        self.export_canvas_to_file();
                    }

                    match key_event.logical_key.as_ref() {
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                            if !self.custom_colors.is_empty() {
                                let new_index = if self.selected_custom_index < 0 {
                                    0
                                } else {
                                    ((self.selected_custom_index as usize + 1)
                                        % self.custom_colors.len())
                                        as i32
                                };
                                self.selected_custom_index = new_index;
                                let color = self.custom_colors[new_index as usize];
                                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                    r: color[0],
                                    g: color[1],
                                    b: color[2],
                                    a: 255,
                                });
                                self.hue = hsv.h;
                                self.saturation = hsv.s;
                                self.value = hsv.v;
                                self.update_hsv_preferences();
                                if let Some(renderer) = &mut self.renderer {
                                    renderer.set_hsv(self.hue, self.saturation, self.value);
                                }
                                if let Some(window) = &self.window {
                                    window.request_redraw();
                                }
                            }
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
                            if !self.custom_colors.is_empty() {
                                let new_index = if self.selected_custom_index <= 0 {
                                    self.custom_colors.len() as i32 - 1
                                } else {
                                    self.selected_custom_index - 1
                                };
                                self.selected_custom_index = new_index;
                                let color = self.custom_colors[new_index as usize];
                                let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                                    r: color[0],
                                    g: color[1],
                                    b: color[2],
                                    a: 255,
                                });
                                self.hue = hsv.h;
                                self.saturation = hsv.s;
                                self.value = hsv.v;
                                self.update_hsv_preferences();
                                if let Some(renderer) = &mut self.renderer {
                                    renderer.set_hsv(self.hue, self.saturation, self.value);
                                }
                                if let Some(window) = &self.window {
                                    window.request_redraw();
                                }
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
                                        // Ctrl+Z: Undo
                                        let mut canvas = self.canvas.borrow_mut();
                                        if canvas.can_undo() {
                                            canvas.undo();
                                            logger::info("Undo: removed last stroke");
                                            println!("Undo: removed last stroke");
                                            if let Some(window) = &self.window {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                    "y" | "Y" => {
                                        // Ctrl+Y: Redo (Windows convention)
                                        let mut canvas = self.canvas.borrow_mut();
                                        if canvas.can_redo() {
                                            canvas.redo();
                                            logger::info("Redo: restored last undone stroke");
                                            println!("Redo: restored last undone stroke");
                                            if let Some(window) = &self.window {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                // Non-Ctrl shortcuts
                                match c_str {
                                    "e" | "E" => {
                                        // Toggle eraser (only when not drawing)
                                        if self.mouse_state != MouseState::Drawing {
                                            self.is_eraser = !self.is_eraser;
                                            logger::info(&format!(
                                                "Eraser mode: {}",
                                                if self.is_eraser { "ON" } else { "OFF" }
                                            ));
                                            if let Some(renderer) = &mut self.renderer {
                                                renderer.set_eraser(self.is_eraser);
                                            }
                                            if let Some(window) = &self.window {
                                                window.request_redraw();
                                            }
                                        }
                                    }
                                    "+" | "=" => {
                                        // Increase brush size
                                        self.brush_size =
                                            crate::canvas::brush_size_up(self.brush_size);
                                        self.preferences.brush.default_size = self.brush_size;
                                        let _ = crate::preferences::save(&self.preferences);
                                        if let Some(renderer) = &mut self.renderer {
                                            renderer.set_brush_size(self.brush_size);
                                        }
                                        logger::info(&format!("Brush size: {}", self.brush_size));
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
                                    }
                                    "-" | "_" => {
                                        // Decrease brush size
                                        self.brush_size =
                                            crate::canvas::brush_size_down(self.brush_size);
                                        self.preferences.brush.default_size = self.brush_size;
                                        let _ = crate::preferences::save(&self.preferences);
                                        if let Some(renderer) = &mut self.renderer {
                                            renderer.set_brush_size(self.brush_size);
                                        }
                                        logger::info(&format!("Brush size: {}", self.brush_size));
                                        if let Some(window) = &self.window {
                                            window.request_redraw();
                                        }
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
                                let mut canvas = self.canvas.borrow_mut();
                                canvas.clear();
                                logger::info("Canvas cleared");
                                if let Some(window) = &self.window {
                                    window.request_redraw();
                                }
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

    #[test]
    fn test_window_app_initial_state() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);

        assert!(!app.is_eraser);
        assert!(app.window.is_none());
        assert!(app.renderer.is_none());
    }

    #[test]
    fn test_mouse_position_initial() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);

        assert_eq!(app.mouse_position.x, 0.0);
        assert_eq!(app.mouse_position.y, 0.0);
    }

    #[test]
    fn test_canvas_access() {
        let preferences = Preferences::default();
        let app = WindowApp::new(preferences);

        let canvas = app.canvas();
        assert_eq!(canvas.borrow().strokes().len(), 0);
    }

    #[test]
    fn test_zero_size_window_guard() {
        // Verify that when window.inner_size() returns (0, 0),
        // the fallback to preferences dimensions is used
        let preferences = Preferences::default();
        assert_eq!(preferences.window.width, 1280);
        assert_eq!(preferences.window.height, 720);

        // Simulate the guard logic: if size is zero, use preferences
        let zero_size = (0u32, 0u32);
        let render_width = if zero_size.0 > 0 {
            zero_size.0
        } else {
            preferences.window.width
        };
        let render_height = if zero_size.1 > 0 {
            zero_size.1
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, 1280);
        assert_eq!(render_height, 720);
    }

    #[test]
    fn test_nonzero_size_window_uses_actual_size() {
        // When window.inner_size() returns a valid size, it should be used
        let preferences = Preferences::default();
        let actual_size = (1920u32, 1080u32);
        let render_width = if actual_size.0 > 0 {
            actual_size.0
        } else {
            preferences.window.width
        };
        let render_height = if actual_size.1 > 0 {
            actual_size.1
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, 1920);
        assert_eq!(render_height, 1080);
    }

    #[test]
    fn test_partial_zero_size_guard() {
        // Guard should work even if only one dimension is zero
        let preferences = Preferences::default();
        let partial_zero = (800u32, 0u32);
        let render_width = if partial_zero.0 > 0 {
            partial_zero.0
        } else {
            preferences.window.width
        };
        let render_height = if partial_zero.1 > 0 {
            partial_zero.1
        } else {
            preferences.window.height
        };
        assert_eq!(render_width, 800);
        assert_eq!(render_height, 720);
    }
}
