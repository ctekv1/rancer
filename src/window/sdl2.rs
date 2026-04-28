//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2 with OpenGL rendering.

use glow::HasContext;
use sdl2::event::Event;
use sdl2::video::GLContext;
use sdl2::video::Window;

use crate::preferences::Preferences;
use crate::viewport::{Viewport, DEFAULT_CANVAS_COLOR};

pub struct Sdl2App {
    window: Window,
    gl_context: GLContext,
    width: u32,
    height: u32,
    viewport: Viewport,
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

        unsafe {
            gl.clear_color(0.94, 0.94, 0.94, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        window.gl_swap_window();

        let size = window.size();
        let width = size.0 as u32;
        let height = size.1 as u32;

        Ok(Self {
            window,
            gl_context,
            width,
            height,
            viewport: Viewport::new(1280, 720),
        })
    }

    pub fn run(&mut self) {
        let window = &self.window;
        let sdl = window.subsystem().sdl();
        let mut event_pump = sdl
            .event_pump()
            .map_err(|e| format!("Failed to create event pump: {}", e))
            .unwrap();

        let video = sdl.video().unwrap();
        let gl = unsafe {
            glow::Context::from_loader_function(move |s| {
                video.gl_get_proc_address(s) as *const std::os::raw::c_void
            })
        };

        let canvas_r = DEFAULT_CANVAS_COLOR.r as f32 / 255.0;
        let canvas_g = DEFAULT_CANVAS_COLOR.g as f32 / 255.0;
        let canvas_b = DEFAULT_CANVAS_COLOR.b as f32 / 255.0;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }

            unsafe {
                gl.clear_color(canvas_r, canvas_g, canvas_b, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            window.gl_swap_window();

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
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