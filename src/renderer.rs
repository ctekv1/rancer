//! WGPU rendering module for Rancer
//!
//! Provides GPU-accelerated rendering for the canvas using wgpu.
//! This module handles the rendering pipeline, shaders, and drawing operations.

use crate::canvas::{Canvas, Color};

/// Configuration for the renderer
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Clear color for the background
    pub clear_color: Color,
    /// MSAA sample count (1, 2, 4, 8, 16)
    pub msaa_samples: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            clear_color: Color::WHITE,
            msaa_samples: 1,
        }
    }
}

/// WGPU-based renderer for the canvas
pub struct Renderer {
    /// Current canvas to render
    canvas: Canvas,
    /// Configuration
    pub config: RendererConfig,
}

impl Renderer {
    /// Create a new renderer
    pub async fn new(config: RendererConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            canvas: Canvas::new(),
            config,
        })
    }

    /// Resize the renderer for a new window size
    pub fn resize(&mut self, _new_size: (u32, u32)) {
        // TODO: Implement actual resize logic
    }

    /// Set the canvas to render
    pub fn set_canvas(&mut self, canvas: Canvas) {
        self.canvas = canvas;
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // TODO: Implement actual rendering
        Ok(())
    }

    /// Get the current canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Create a new renderer synchronously (simplified version for immediate window display)
    pub fn new_sync(config: RendererConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            canvas: Canvas::new(),
            config,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.clear_color, Color::WHITE);
        assert_eq!(config.msaa_samples, 1);
    }

    #[test]
    fn test_renderer_config_custom() {
        let config = RendererConfig {
            clear_color: Color::BLACK,
            msaa_samples: 4,
        };
        assert_eq!(config.clear_color, Color::BLACK);
        assert_eq!(config.msaa_samples, 4);
    }
}
