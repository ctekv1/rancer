//! OpenGL renderer module for GTK4 GLArea
//!
//! Provides GPU-accelerated rendering for the canvas using OpenGL ES 2.0.
//! Used by the GTK4 backend on Linux as a replacement for Cairo software rendering.

use glow::HasContext;
use std::rc::Rc;

use crate::canvas::{ActiveStroke, Canvas, ColorPalette};
use crate::geometry;

const VERTEX_SHADER_SOURCE: &str = r#"
    attribute vec2 position;
    attribute vec4 color;
    uniform vec2 canvas_size;
    varying vec4 v_color;
    void main() {
        vec2 pos = position / canvas_size * 2.0 - 1.0;
        gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
        v_color = color;
    }
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
    precision mediump float;
    varying vec4 v_color;
    void main() {
        gl_FragColor = v_color;
    }
"#;

/// OpenGL renderer for GTK4 GLArea
pub struct GlRenderer {
    gl: Rc<glow::Context>,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    canvas_size_uniform: glow::UniformLocation,
}

impl GlRenderer {
    /// Create a new OpenGL renderer from a glow context
    pub fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        unsafe {
            let program = Self::compile_shaders(&gl)?;
            let canvas_size_uniform = gl
                .get_uniform_location(program, "canvas_size")
                .ok_or_else(|| "Failed to get canvas_size uniform location".to_string())?;

            let vao = gl
                .create_vertex_array()
                .map_err(|e| format!("Failed to create VAO: {e}"))?;
            let vbo = gl
                .create_buffer()
                .map_err(|e| format!("Failed to create VBO: {e}"))?;

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let stride = (6 * std::mem::size_of::<f32>()) as i32;

            // position attribute (location 0): vec2
            let pos_loc = gl.get_attrib_location(program, "position").unwrap();
            gl.enable_vertex_attrib_array(pos_loc);
            gl.vertex_attrib_pointer_f32(pos_loc, 2, glow::FLOAT, false, stride, 0);

            // color attribute (location 1): vec4
            let color_loc = gl.get_attrib_location(program, "color").unwrap();
            gl.enable_vertex_attrib_array(color_loc);
            gl.vertex_attrib_pointer_f32(color_loc, 4, glow::FLOAT, false, stride, 2 * 4);

            gl.bind_vertex_array(None);

            // Enable blending for alpha support
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            Ok(Self {
                gl,
                program,
                vao,
                vbo,
                canvas_size_uniform,
            })
        }
    }

    /// Compile vertex and fragment shaders into a program
    unsafe fn compile_shaders(gl: &glow::Context) -> Result<glow::Program, String> {
        unsafe {
            let vs = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| format!("Failed to create vertex shader: {e}"))?;
            gl.shader_source(vs, VERTEX_SHADER_SOURCE);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                let log = gl.get_shader_info_log(vs);
                gl.delete_shader(vs);
                return Err(format!("Vertex shader compilation failed: {log}"));
            }

            let fs = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| format!("Failed to create fragment shader: {e}"))?;
            gl.shader_source(fs, FRAGMENT_SHADER_SOURCE);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                let log = gl.get_shader_info_log(fs);
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                return Err(format!("Fragment shader compilation failed: {log}"));
            }

            let program = gl
                .create_program()
                .map_err(|e| format!("Failed to create program: {e}"))?;
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                gl.delete_program(program);
                return Err(format!("Shader program linking failed: {log}"));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            Ok(program)
        }
    }

    /// Render a frame: clear, draw strokes, draw UI
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        canvas: &Canvas,
        palette: &ColorPalette,
        active_stroke: &Option<ActiveStroke>,
        brush_size: f32,
        is_eraser: bool,
        width: i32,
        height: i32,
    ) {
        unsafe {
            self.gl.viewport(0, 0, width, height);
            self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.program));
            self.gl
                .uniform_2_f32(Some(&self.canvas_size_uniform), width as f32, height as f32);
            self.gl.bind_vertex_array(Some(self.vao));

            // Draw committed strokes
            for stroke in canvas.strokes() {
                let vertices = Self::generate_stroke_vertices(stroke);
                if !vertices.is_empty() {
                    self.upload_and_draw(&vertices, glow::TRIANGLE_STRIP);
                }
            }

            // Draw active stroke
            if let Some(active) = active_stroke {
                let vertices = Self::generate_active_stroke_vertices(active);
                if !vertices.is_empty() {
                    self.upload_and_draw(&vertices, glow::TRIANGLE_STRIP);
                }
            }

            // Draw color palette UI
            let palette_vertices = Self::generate_palette_vertices(palette);
            if !palette_vertices.is_empty() {
                self.upload_and_draw(&palette_vertices, glow::TRIANGLES);
            }

            // Draw brush size selector UI
            let brush_vertices = Self::generate_brush_size_vertices(brush_size);
            if !brush_vertices.is_empty() {
                self.upload_and_draw(&brush_vertices, glow::TRIANGLES);
            }

            // Draw eraser button UI
            let eraser_vertices = Self::generate_eraser_button_vertices(is_eraser);
            if !eraser_vertices.is_empty() {
                self.upload_and_draw(&eraser_vertices, glow::TRIANGLES);
            }

            // Draw clear button UI
            let clear_vertices = Self::generate_clear_button_vertices();
            if !clear_vertices.is_empty() {
                self.upload_and_draw(&clear_vertices, glow::TRIANGLES);
            }

            self.gl.bind_vertex_array(None);
        }
    }

    /// Upload vertex data and draw
    unsafe fn upload_and_draw(&self, vertices: &[f32], mode: u32) {
        unsafe {
            let byte_data = std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                std::mem::size_of_val(vertices),
            );
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, byte_data, glow::DYNAMIC_DRAW);
            let vertex_count = (vertices.len() / 6) as i32;
            self.gl.draw_arrays(mode, 0, vertex_count);
        }
    }

    /// Generate vertex data for a committed stroke as a triangle strip
    fn generate_stroke_vertices(stroke: &crate::canvas::Stroke) -> Vec<f32> {
        geometry::generate_stroke_vertices(stroke)
    }

    /// Generate vertex data for an active stroke being drawn
    fn generate_active_stroke_vertices(active: &ActiveStroke) -> Vec<f32> {
        geometry::generate_active_stroke_vertices(active)
    }

    /// Generate vertices for the color palette UI
    fn generate_palette_vertices(palette: &ColorPalette) -> Vec<f32> {
        geometry::generate_palette_vertices(palette, palette.selected_index())
    }

    /// Generate vertices for brush size selector UI
    fn generate_brush_size_vertices(selected_size: f32) -> Vec<f32> {
        geometry::generate_brush_size_vertices(selected_size)
    }

    /// Generate vertices for eraser button UI
    fn generate_eraser_button_vertices(is_active: bool) -> Vec<f32> {
        geometry::generate_eraser_button_vertices(is_active)
    }

    /// Generate vertices for clear button UI
    fn generate_clear_button_vertices() -> Vec<f32> {
        geometry::generate_clear_button_vertices()
    }
}

impl Drop for GlRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}
