//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2 with OpenGL rendering.

use glow::HasContext;
use sdl2::event::Event;

use crate::app::AppState;
use crate::events::AppEvent;
use crate::preferences::Preferences;
use crate::ui::egui_integration::EguiIntegration;
use crate::ui::UiState;

/// Convert an SDL2 event to an AppEvent
pub fn sdl_event_to_app_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Quit { .. } => Some(AppEvent::Quit),
        Event::MouseButtonDown { x, y, .. } => Some(AppEvent::Press {
            x: x as f32,
            y: y as f32,
        }),
        Event::MouseButtonUp { x, y, .. } => Some(AppEvent::Release {
            x: x as f32,
            y: y as f32,
        }),
        Event::MouseMotion {
            x,
            y,
            mousestate,
            ..
        } => {
            if mousestate.left() {
                Some(AppEvent::Drag {
                    x: x as f32,
                    y: y as f32,
                })
            } else {
                None
            }
        }
        Event::KeyDown {
            keycode: Some(keycode),
            ..
        } => Some(AppEvent::Key {
            code: format!("{:?}", keycode).to_lowercase(),
        }),
        Event::Window {
            win_event: sdl2::event::WindowEvent::SizeChanged(w, h) | sdl2::event::WindowEvent::Resized(w, h),
            ..
        } => Some(AppEvent::Resize {
            width: w.max(0) as u32,
            height: h.max(0) as u32,
        }),
        _ => None,
    }
}

pub struct Sdl2App {
    window: sdl2::video::Window,
    gl: glow::Context,
    gl_context: sdl2::video::GLContext,
    app_state: AppState,
    ui_state: UiState,
    icon_cache: crate::ui::egui_impl::IconCache,
    egui: EguiIntegration,
    compositor: crate::compositor::Compositor,
    renderer: crate::renderer::CanvasRenderer,
    preferences: Preferences,
}

impl Sdl2App {
    pub fn new(preferences: Preferences) -> Result<Self, String> {
        let sdl = sdl2::init().map_err(|e| format!("Failed to initialize SDL2: {}", e))?;
        let video = sdl
            .video()
            .map_err(|e| format!("Failed to initialize video subsystem: {}", e))?;

        let window_width = preferences.window.width;
        let window_height = preferences.window.height;

        let window = video
            .window(&preferences.window.title, window_width, window_height)
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        let gl_context = window
            .gl_create_context()
            .map_err(|e| format!("Failed to create GL context: {}", e))?;

        window.gl_make_current(&gl_context).ok();

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const std::os::raw::c_void
            })
        };

        let (width, height) = {
            let size = window.size();
            (size.0 as u32, size.1 as u32)
        };

        let renderer = crate::renderer::CanvasRenderer::new(&gl, width, height)?;

        unsafe {
            gl.clear_color(0.94, 0.94, 0.94, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        window.gl_swap_window();

        let app_state = AppState::new(width, height);
        let ui_state = UiState::new();
        
        // Create egui integration
        let mut egui = EguiIntegration::new(&window, &gl_context, &gl)
            .map_err(|e| format!("Failed to create egui integration: {}", e))?;
        
        // Create icon cache (needs egui context)
        let icon_cache = crate::ui::egui_impl::IconCache::new(egui.ctx());
        
        Ok(Self {
            window,
            gl,
            gl_context,
            app_state,
            ui_state,
            icon_cache,
            egui,
            compositor: crate::compositor::Compositor::new(),
            renderer,
            preferences,
        })
    }

    pub fn run(&mut self) {
        let mut event_pump = self.window.subsystem().sdl()
            .event_pump()
            .map_err(|e| format!("Failed to create event pump: {}", e))
            .unwrap();

        self.window.gl_make_current(&self.gl_context).ok();

        // Enable VSync to prevent screen tearing/flash
        self.window.subsystem().sdl().video()
            .unwrap()
            .gl_set_swap_interval(1)
            .ok();

        'running: loop {
            let mut has_work = false;
            for event in event_pump.poll_iter() {
                // Handle resize before egui — egui must not swallow window resize
                if let sdl2::event::Event::Window {
                    win_event:
                        sdl2::event::WindowEvent::SizeChanged(w, h)
                        | sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } = &event
                {
                    let w = (*w).max(0) as u32;
                    let h = (*h).max(0) as u32;
                    self.renderer.resize_viewport(&self.gl, w, h);
                    self.preferences.update_window_size(w, h);
                    let _ = crate::preferences::save(&self.preferences);
                    self.app_state.handle_event(AppEvent::Resize { width: w, height: h });
                    has_work = true;
                }

                // Pass event to egui (resize events also need to reach egui for layout)
                let consumed = self.egui.handle_event(&self.window, &event);

                // Skip canvas events when egui consumed them (e.g. color picker interaction)
                if consumed {
                    continue;
                }

                // Then convert to AppEvent for the app
                if let Some(app_event) = sdl_event_to_app_event(event) {
                    has_work = true;
                    match app_event {
                        AppEvent::Quit => break 'running,
                        _ => self.app_state.handle_event(app_event),
                    }
                }
            }

            // Render and swap
            self.render_frame();
            
            // Render egui on top
                self.egui.run_and_render(&self.window, |ctx: &egui_sdl2::egui::Context| {
                    self.ui_state.apply_to_app(&mut self.app_state);
                    crate::ui::show_ui(ctx, &mut self.app_state, &mut self.ui_state, &self.icon_cache);
                });
            
            self.window.gl_swap_window();

            // Yield CPU when idle
            if !has_work {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    fn render_frame(&mut self) {
        if let Some((composite, x, y)) = self.compositor.render(&mut self.app_state.canvas_mut()) {
            self.renderer.upload(&self.gl, &composite, x, y);
        }
        self.renderer.draw(&self.gl);
    }
}

pub fn run_app(preferences: Preferences) {
    if let Err(e) = crate::logger::init() {
        eprintln!("Warning: File logging failed ({e}), using console-only logging");
    }

    crate::logger::info("Starting Rancer v0.0.7 with SDL2...");

    match Sdl2App::new(preferences) {
        Ok(mut app) => {
            crate::logger::info("SDL2 window initialized successfully");
            app.run();
        }
        Err(e) => {
            crate::logger::error(&format!("Failed to initialize SDL2 application: {}", e));
        }
    }

    crate::logger::info("Rancer application closed successfully");
}
