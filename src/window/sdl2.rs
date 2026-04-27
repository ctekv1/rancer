//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2.

use sdl2::event::Event;
use sdl2::video::Window;

use crate::preferences::Preferences;

pub struct Sdl2App {
    window: Window,
    width: u32,
    height: u32,
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
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        let size = window.size();
        let width = size.0 as u32;
        let height = size.1 as u32;

        Ok(Self {
            window,
            width,
            height,
        })
    }

    pub fn run(&mut self) {
        let window = &self.window;
        let sdl = window.subsystem().sdl();
        let mut event_pump = sdl
            .event_pump()
            .map_err(|e| format!("Failed to create event pump: {}", e))
            .unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }

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