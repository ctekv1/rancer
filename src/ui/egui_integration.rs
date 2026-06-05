//! egui integration with SDL2 using egui_sdl2
//!
//! Provides egui rendering and event handling via the egui_sdl2 crate.

use sdl2::event::Event;
use sdl2::video::Window;
use egui_sdl2::EguiGlow;
use std::sync::Arc;

/// Manages egui integration with SDL2 + glow
pub struct EguiIntegration {
    egui: EguiGlow,
}

impl EguiIntegration {
    /// Create a new egui integration for the given SDL2 window and GL context
    pub fn new(
        window: &Window,
        _gl_context: &sdl2::video::GLContext,
        _gl: &glow::Context,
    ) -> Result<Self, String> {
        // Note: glow::Context doesn't implement Clone, but EguGlow needs Arc<glow::Context>
        // We need to pass the context by reference and let EguGlow handle it
        // For now, we'll use a workaround by creating a new context from the existing one
        let glow_ctx = unsafe {
            glow::Context::from_loader_function(|s| {
                window.subsystem().gl_get_proc_address(s) as *const std::os::raw::c_void
            })
        };
        let glow_ctx = Arc::new(glow_ctx);
        let egui = EguiGlow::new(window, glow_ctx, None, false);
        Ok(Self { egui })
    }

    /// Handle an SDL2 event and pass it to egui
    /// Returns true if egui consumed the event (e.g. interacting with a widget)
    pub fn handle_event(&mut self, window: &Window, event: &Event) -> bool {
        self.egui.state.on_event(window, event).consumed
    }

    /// Run egui UI and render it on top of the canvas
    pub fn run_and_render<F>(&mut self, _window: &Window, mut ui_fn: F)
    where
        F: FnMut(&egui_sdl2::egui::Context),
    {
        self.egui.run(|ctx| ui_fn(ctx));
        self.egui.paint();
        // Present is handled by the main window swap
    }

    /// Get the egui context
    pub fn ctx(&mut self) -> &egui_sdl2::egui::Context {
        &self.egui.ctx
    }
}
