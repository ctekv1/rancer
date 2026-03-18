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
    config: RendererConfig,
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

/// Simple shader for rendering strokes
/// This is a basic implementation - real stroke rendering would be more complex
const VERTEX_SHADER: &str = r#"
struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

struct Uniforms {
    canvas_size: vec2<f32>;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert from canvas coordinates to clip space (-1 to 1)
    let pos = vertex.position / uniforms.canvas_size * 2.0 - 1.0;
    output.clip_position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.color = vertex.color;
    
    return output;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    return input.color;
}
"#;

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
