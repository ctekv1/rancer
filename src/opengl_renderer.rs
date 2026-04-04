//! OpenGL renderer module for GTK4 GLArea
//!
//! Provides GPU-accelerated rendering for the canvas using OpenGL ES 2.0.
//! Used by the GTK4 backend on Linux as a replacement for Cairo software rendering.

use glow::HasContext;
use std::rc::Rc;

use crate::canvas::{ActiveStroke, Canvas};
use crate::geometry;

const VERTEX_SHADER_SOURCE: &str = r#"
    attribute vec2 position;
    attribute vec4 color;
    uniform vec2 canvas_size;
    uniform float zoom;
    uniform vec2 pan_offset;
    varying vec4 v_color;
    void main() {
        vec2 world_pos = (position - pan_offset) * zoom;
        vec2 pos = world_pos / canvas_size * 2.0 - 1.0;
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
    zoom_uniform: glow::UniformLocation,
    pan_offset_uniform: glow::UniformLocation,
    // store logical canvas size for DPI-aware rendering
    canvas_logical_size: std::cell::Cell<(f32, f32)>,
    // viewport transform state
    zoom: std::cell::Cell<f32>,
    pan_offset: std::cell::Cell<(f32, f32)>,
}

impl GlRenderer {
    /// Create a new OpenGL renderer from a glow context
    pub fn new(gl: Rc<glow::Context>) -> Result<Self, String> {
        unsafe {
            let program = Self::compile_shaders(&gl)?;
            let canvas_size_uniform = gl
                .get_uniform_location(program, "canvas_size")
                .ok_or_else(|| "Failed to get canvas_size uniform location".to_string())?;
            let zoom_uniform = gl
                .get_uniform_location(program, "zoom")
                .ok_or_else(|| "Failed to get zoom uniform location".to_string())?;
            let pan_offset_uniform = gl
                .get_uniform_location(program, "pan_offset")
                .ok_or_else(|| "Failed to get pan_offset uniform location".to_string())?;

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
                zoom_uniform,
                pan_offset_uniform,
                canvas_logical_size: std::cell::Cell::new((1.0, 1.0)),
                zoom: std::cell::Cell::new(1.0),
                pan_offset: std::cell::Cell::new((0.0, 0.0)),
            })
        }
    }

    // Update the canvas logical size (width/height after DPI scaling)
    pub fn set_canvas_logical_size(&self, w: f32, h: f32) {
        self.canvas_logical_size.set((w, h));
    }

    // Set zoom level (1.0 = 100%)
    pub fn set_zoom(&self, zoom: f32) {
        self.zoom.set(zoom);
    }

    // Get current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom.get()
    }

    // Set pan offset (in canvas coordinates)
    pub fn set_pan(&self, offset: (f32, f32)) {
        self.pan_offset.set(offset);
    }

    // Get current pan offset
    pub fn pan_offset(&self) -> (f32, f32) {
        self.pan_offset.get()
    }

    /// Compile vertex and fragment shaders into a program
    #[allow(clippy::unnecessary_safety_comment)]
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

    /// Render a frame: clear, draw strokes, draw UI (HSV version)
    #[allow(clippy::too_many_arguments)]
    pub fn render_hsv(
        &self,
        canvas: &Canvas,
        active_stroke: &Option<ActiveStroke>,
        brush_size: f32,
        is_eraser: bool,
        opacity: f32,
        width: i32,
        height: i32,
        hue: f32,
        saturation: f32,
        value: f32,
        custom_colors: Vec<[u8; 3]>,
        selected_custom_index: i32,
    ) {
        unsafe {
            self.gl.viewport(0, 0, width, height);
            self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.program));
            // Use logical canvas size for coordinate mapping
            let (lw, lh) = self.canvas_logical_size.get();
            self.gl
                .uniform_2_f32(Some(&self.canvas_size_uniform), lw, lh);

            // Set zoom and pan uniforms
            let zoom = self.zoom.get();
            let (pan_x, pan_y) = self.pan_offset.get();
            self.gl.uniform_1_f32(Some(&self.zoom_uniform), zoom);
            self.gl
                .uniform_2_f32(Some(&self.pan_offset_uniform), pan_x, pan_y);

            self.gl.bind_vertex_array(Some(self.vao));

            // Draw committed strokes per-layer, inserting active stroke
            // at the active layer position so it renders in correct order.
            let active_layer_idx = canvas.active_layer();
            let layers = canvas.layers();
            for (layer_idx, layer) in layers.iter().enumerate().rev() {
                if !layer.visible {
                    continue;
                }
                for stroke in &layer.strokes {
                    if stroke.points.len() >= 2 {
                        let vertices =
                            Self::generate_stroke_vertices_with_opacity(stroke, layer.opacity);
                        if !vertices.is_empty() {
                            self.upload_and_draw(&vertices, glow::TRIANGLE_STRIP);
                        }
                    }
                }
                // Draw active stroke at the active layer position
                if layer_idx == active_layer_idx {
                    if let Some(active) = active_stroke {
                        let vertices = Self::generate_active_stroke_vertices_with_opacity(
                            active,
                            layer.opacity,
                        );
                        if !vertices.is_empty() {
                            self.upload_and_draw(&vertices, glow::TRIANGLE_STRIP);
                        }
                    }
                }
            }

            // Reset zoom/pan for UI elements (UI stays fixed on screen)
            self.gl.uniform_1_f32(Some(&self.zoom_uniform), 1.0);
            self.gl
                .uniform_2_f32(Some(&self.pan_offset_uniform), 0.0, 0.0);

            // Draw HSV sliders UI
            let hsv_vertices = Self::generate_hsv_slider_vertices(hue, saturation, value);
            if !hsv_vertices.is_empty() {
                self.upload_and_draw(&hsv_vertices, glow::TRIANGLES);
            }

            // Draw custom palette UI
            let palette_vertices =
                Self::generate_custom_palette_vertices(&custom_colors, selected_custom_index);
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

            // Draw undo button UI
            let undo_vertices = Self::generate_undo_button_vertices(canvas.can_undo());
            if !undo_vertices.is_empty() {
                self.upload_and_draw(&undo_vertices, glow::TRIANGLES);
            }

            // Draw redo button UI
            let redo_vertices = Self::generate_redo_button_vertices(canvas.can_redo());
            if !redo_vertices.is_empty() {
                self.upload_and_draw(&redo_vertices, glow::TRIANGLES);
            }

            // Draw export button UI
            let export_vertices = Self::generate_export_button_vertices();
            if !export_vertices.is_empty() {
                self.upload_and_draw(&export_vertices, glow::TRIANGLES);
            }

            // Draw zoom in button UI
            let zoom_in_vertices = Self::generate_zoom_in_button_vertices();
            if !zoom_in_vertices.is_empty() {
                self.upload_and_draw(&zoom_in_vertices, glow::TRIANGLES);
            }

            // Draw zoom out button UI
            let zoom_out_vertices = Self::generate_zoom_out_button_vertices();
            if !zoom_out_vertices.is_empty() {
                self.upload_and_draw(&zoom_out_vertices, glow::TRIANGLES);
            }

            // Draw opacity preset buttons UI
            let opacity_vertices = Self::generate_opacity_preset_vertices(opacity);
            if !opacity_vertices.is_empty() {
                self.upload_and_draw(&opacity_vertices, glow::TRIANGLES);
            }

            // Draw layer panel UI
            let layer_panel_vertices = Self::generate_layer_panel_vertices(
                canvas.layers(),
                canvas.active_layer(),
                width as f32,
            );
            if !layer_panel_vertices.is_empty() {
                self.upload_and_draw(&layer_panel_vertices, glow::TRIANGLES);
            }

            self.gl.bind_vertex_array(None);
        }
    }

    /// Upload vertex data and draw
    #[allow(clippy::unnecessary_safety_comment)]
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

    /// Generate vertex data for a committed stroke with layer opacity
    fn generate_stroke_vertices_with_opacity(
        stroke: &crate::canvas::Stroke,
        layer_opacity: f32,
    ) -> Vec<f32> {
        geometry::generate_stroke_vertices_with_opacity(stroke, layer_opacity)
    }

    /// Generate vertex data for an active stroke with layer opacity
    fn generate_active_stroke_vertices_with_opacity(
        active: &ActiveStroke,
        layer_opacity: f32,
    ) -> Vec<f32> {
        geometry::generate_active_stroke_vertices_with_opacity(active, layer_opacity)
    }

    /// Generate vertices for HSV slider UI
    fn generate_hsv_slider_vertices(hue: f32, saturation: f32, value: f32) -> Vec<f32> {
        geometry::generate_hsv_sliders(hue, saturation, value)
    }

    /// Generate vertices for custom palette UI
    fn generate_custom_palette_vertices(colors: &[[u8; 3]], selected_index: i32) -> Vec<f32> {
        geometry::generate_custom_palette(colors, selected_index as usize)
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

    /// Generate vertices for undo button UI
    fn generate_undo_button_vertices(can_undo: bool) -> Vec<f32> {
        geometry::generate_undo_button_vertices(can_undo)
    }

    /// Generate vertices for redo button UI
    fn generate_redo_button_vertices(can_redo: bool) -> Vec<f32> {
        geometry::generate_redo_button_vertices(can_redo)
    }

    /// Generate vertices for export button UI
    fn generate_export_button_vertices() -> Vec<f32> {
        geometry::generate_export_button_vertices()
    }

    /// Generate vertices for zoom in button UI
    fn generate_zoom_in_button_vertices() -> Vec<f32> {
        geometry::generate_zoom_in_button_vertices()
    }

    /// Generate vertices for zoom out button UI
    fn generate_zoom_out_button_vertices() -> Vec<f32> {
        geometry::generate_zoom_out_button_vertices()
    }

    /// Generate vertices for opacity preset buttons UI
    fn generate_opacity_preset_vertices(opacity: f32) -> Vec<f32> {
        geometry::generate_opacity_preset_vertices(opacity)
    }

    /// Generate vertices for layer panel UI
    fn generate_layer_panel_vertices(
        layers: &[crate::canvas::Layer],
        active_layer: usize,
        window_width: f32,
    ) -> Vec<f32> {
        let layer_data: Vec<(String, bool, f32, bool)> = layers
            .iter()
            .map(|l| (l.name.clone(), l.visible, l.opacity, l.locked))
            .collect();
        geometry::generate_layer_panel_vertices(&layer_data, active_layer, window_width)
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
