//! OpenGL renderer module for GTK4 GLArea
//!
//! Provides GPU-accelerated rendering for the canvas using OpenGL ES 2.0.
//! Used by the GTK4 backend on Linux as a replacement for Cairo software rendering.

use glow::HasContext;
use std::rc::Rc;

use crate::canvas::{ActiveStroke, Canvas, ColorPalette, Stroke};

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
    pub fn render(
        &self,
        canvas: &Canvas,
        palette: &ColorPalette,
        active_stroke: &Option<ActiveStroke>,
        brush_size: f32,
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

            self.gl.bind_vertex_array(None);
        }
    }

    /// Upload vertex data and draw
    unsafe fn upload_and_draw(&self, vertices: &[f32], mode: u32) {
        unsafe {
            let byte_data = std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<f32>(),
            );
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, byte_data, glow::DYNAMIC_DRAW);
            let vertex_count = (vertices.len() / 6) as i32;
            self.gl.draw_arrays(mode, 0, vertex_count);
        }
    }

    /// Generate vertex data for a committed stroke as a triangle strip
    fn generate_stroke_vertices(stroke: &Stroke) -> Vec<f32> {
        let mut vertices = Vec::new();
        let r = stroke.color.r as f32 / 255.0;
        let g = stroke.color.g as f32 / 255.0;
        let b = stroke.color.b as f32 / 255.0;
        let a = stroke.opacity;
        let half_width = stroke.width / 2.0;

        if stroke.points.len() < 2 {
            return vertices;
        }

        for i in 0..stroke.points.len() {
            let p = &stroke.points[i];

            let (dx, dy) = if i == 0 {
                let next = &stroke.points[i + 1];
                (next.x - p.x, next.y - p.y)
            } else if i == stroke.points.len() - 1 {
                let prev = &stroke.points[i - 1];
                (p.x - prev.x, p.y - prev.y)
            } else {
                let prev = &stroke.points[i - 1];
                let next = &stroke.points[i + 1];
                (next.x - prev.x, next.y - prev.y)
            };

            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.001 {
                continue;
            }

            let nx = -dy / len * half_width;
            let ny = dx / len * half_width;

            // Two vertices per point (left and right of path)
            // Each vertex: [x, y, r, g, b, a] (6 floats — line_width not needed for GL)
            vertices.extend_from_slice(&[p.x + nx, p.y + ny, r, g, b, a]);
            vertices.extend_from_slice(&[p.x - nx, p.y - ny, r, g, b, a]);
        }

        vertices
    }

    /// Generate vertex data for an active stroke being drawn
    fn generate_active_stroke_vertices(active: &ActiveStroke) -> Vec<f32> {
        let mut vertices = Vec::new();
        let r = active.color().r as f32 / 255.0;
        let g = active.color().g as f32 / 255.0;
        let b = active.color().b as f32 / 255.0;
        let a = active.opacity();
        let half_width = active.width() / 2.0;
        let points = active.points();

        if points.len() < 2 {
            return vertices;
        }

        for i in 0..points.len() {
            let p = &points[i];

            let (dx, dy) = if i == 0 {
                let next = &points[i + 1];
                (next.x - p.x, next.y - p.y)
            } else if i == points.len() - 1 {
                let prev = &points[i - 1];
                (p.x - prev.x, p.y - prev.y)
            } else {
                let prev = &points[i - 1];
                let next = &points[i + 1];
                (next.x - prev.x, next.y - prev.y)
            };

            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.001 {
                continue;
            }

            let nx = -dy / len * half_width;
            let ny = dx / len * half_width;

            vertices.extend_from_slice(&[p.x + nx, p.y + ny, r, g, b, a]);
            vertices.extend_from_slice(&[p.x - nx, p.y - ny, r, g, b, a]);
        }

        vertices
    }

    /// Generate vertices for a filled rectangle
    fn generate_rect(x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32) -> Vec<f32> {
        vec![
            // Triangle 1
            x,
            y,
            r,
            g,
            b,
            a,
            x + w,
            y,
            r,
            g,
            b,
            a,
            x,
            y + h,
            r,
            g,
            b,
            a,
            // Triangle 2
            x + w,
            y,
            r,
            g,
            b,
            a,
            x + w,
            y + h,
            r,
            g,
            b,
            a,
            x,
            y + h,
            r,
            g,
            b,
            a,
        ]
    }

    /// Generate vertices for the color palette UI
    fn generate_palette_vertices(palette: &ColorPalette) -> Vec<f32> {
        let mut vertices = Vec::new();
        let colors = palette.colors();

        let palette_x = 10.0;
        let palette_y = 10.0;
        let color_width = 20.0;
        let color_height = 20.0;
        let spacing = 5.0;
        let border_width = 2.0;

        for (i, color) in colors.iter().enumerate() {
            let x = palette_x + (color_width + spacing) * i as f32;
            let cr = color.r as f32 / 255.0;
            let cg = color.g as f32 / 255.0;
            let cb = color.b as f32 / 255.0;

            // Border for selected color
            if i == palette.selected_index() {
                // Top
                vertices.extend(Self::generate_rect(
                    x - border_width,
                    palette_y - border_width,
                    color_width + border_width * 2.0,
                    border_width,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ));
                // Bottom
                vertices.extend(Self::generate_rect(
                    x - border_width,
                    palette_y + color_height,
                    color_width + border_width * 2.0,
                    border_width,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ));
                // Left
                vertices.extend(Self::generate_rect(
                    x - border_width,
                    palette_y - border_width,
                    border_width,
                    color_height + border_width * 2.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ));
                // Right
                vertices.extend(Self::generate_rect(
                    x + color_width,
                    palette_y - border_width,
                    border_width,
                    color_height + border_width * 2.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ));
            }

            // Color swatch
            vertices.extend(Self::generate_rect(
                x,
                palette_y,
                color_width,
                color_height,
                cr,
                cg,
                cb,
                1.0,
            ));
        }

        vertices
    }

    /// Generate vertices for brush size selector UI
    fn generate_brush_size_vertices(selected_size: f32) -> Vec<f32> {
        let mut vertices = Vec::new();
        let brush_sizes: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];

        let selector_x = 10.0;
        let selector_y = 50.0;
        let button_size = 30.0;
        let spacing = 10.0;

        for (i, &size) in brush_sizes.iter().enumerate() {
            let x = selector_x + (button_size + spacing) * i as f32;

            // Button background (gray)
            vertices.extend(Self::generate_rect(
                x,
                selector_y,
                button_size,
                button_size,
                0.8,
                0.8,
                0.8,
                1.0,
            ));

            // Brush size indicator (black square centered)
            let indicator_size = size.min(button_size - 4.0);
            let ix = x + (button_size - indicator_size) / 2.0;
            let iy = selector_y + (button_size - indicator_size) / 2.0;

            vertices.extend(Self::generate_rect(
                ix,
                iy,
                indicator_size,
                indicator_size,
                0.0,
                0.0,
                0.0,
                1.0,
            ));

            // Border for selected size
            if (size - selected_size).abs() < 0.1 {
                let bw = 2.0;
                // Top
                vertices.extend(Self::generate_rect(
                    x - bw,
                    selector_y - bw,
                    button_size + bw * 2.0,
                    bw,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                ));
                // Bottom
                vertices.extend(Self::generate_rect(
                    x - bw,
                    selector_y + button_size,
                    button_size + bw * 2.0,
                    bw,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                ));
                // Left
                vertices.extend(Self::generate_rect(
                    x - bw,
                    selector_y - bw,
                    bw,
                    button_size + bw * 2.0,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                ));
                // Right
                vertices.extend(Self::generate_rect(
                    x + button_size,
                    selector_y - bw,
                    bw,
                    button_size + bw * 2.0,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                ));
            }
        }

        vertices
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
