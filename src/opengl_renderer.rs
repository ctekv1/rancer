//! OpenGL renderer module for GTK4 GLArea
//!
//! Provides GPU-accelerated rendering for the canvas using OpenGL ES 2.0.
//! Used by the GTK4 backend on Linux for GPU-accelerated rendering.

use glow::HasContext;
use std::rc::Rc;

use crate::canvas::{ActiveStroke, BrushType, Canvas};
use crate::geometry::{self, DrawMode, StrokeMesh};

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

/// UI state needed for rendering a frame
pub struct GlUiState {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub custom_colors: Vec<[u8; 3]>,
    pub selected_custom_index: i32,
    pub brush_size: f32,
    pub opacity: f32,
    pub is_eraser: bool,
    pub brush_type: BrushType,
    pub selection_tool_active: bool,
    pub selection_rect: Option<crate::canvas::Rect>,
    pub selection_time: f32,
}

/// Viewport state for canvas transform
pub struct GlViewportState {
    pub zoom: f32,
    pub pan_offset: (f32, f32),
}

/// All data needed to render a single frame with the OpenGL renderer.
///
/// This is the single source of truth for render data.
/// The `GlRenderer` holds no application state — it only owns OpenGL internals.
pub struct GlRenderFrame<'a> {
    pub canvas: &'a Canvas,
    pub active_stroke: &'a Option<ActiveStroke>,
    pub ui: GlUiState,
    pub viewport: GlViewportState,
    pub window_size: (i32, i32),
}

/// Committed stroke mesh for a single stroke
type CachedStrokeMesh = Vec<[f32; 6]>;

/// Committed stroke data for a single layer
#[derive(Clone)]
struct LayerStrokeCache {
    strip_strokes: Vec<CachedStrokeMesh>,
    tri_strokes: Vec<CachedStrokeMesh>,
}

impl LayerStrokeCache {
    fn new() -> Self {
        Self {
            strip_strokes: Vec::new(),
            tri_strokes: Vec::new(),
        }
    }
}

/// OpenGL renderer for GTK4 GLArea
pub struct GlRenderer {
    gl: Rc<glow::Context>,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    canvas_size_uniform: glow::UniformLocation,
    zoom_uniform: glow::UniformLocation,
    pan_offset_uniform: glow::UniformLocation,
    /// Committed stroke cache per layer (index = layer index)
    layer_stroke_cache: Vec<LayerStrokeCache>,
    /// Canvas version when cache was last populated
    canvas_version_cached: u64,
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
            let pos_loc = gl
                .get_attrib_location(program, "position")
                .ok_or_else(|| "Failed to get 'position' attribute location".to_string())?;
            gl.enable_vertex_attrib_array(pos_loc);
            gl.vertex_attrib_pointer_f32(pos_loc, 2, glow::FLOAT, false, stride, 0);

            // color attribute (location 1): vec4
            let color_loc = gl
                .get_attrib_location(program, "color")
                .ok_or_else(|| "Failed to get 'color' attribute location".to_string())?;
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
                layer_stroke_cache: Vec::new(),
                canvas_version_cached: 0,
            })
        }
    }

    /// Ensure the committed stroke cache is valid.
    /// If the canvas version has changed, regenerate the cache.
    fn ensure_cache_valid(&mut self, canvas: &Canvas) {
        if canvas.version() != self.canvas_version_cached {
            let layer_count = canvas.layers().len();
            self.layer_stroke_cache.clear();
            self.layer_stroke_cache
                .resize(layer_count, LayerStrokeCache::new());

            for (layer_idx, layer) in canvas.layers().iter().enumerate() {
                if !layer.visible {
                    continue;
                }
                let cache = &mut self.layer_stroke_cache[layer_idx];

                for stroke in &layer.strokes {
                    if stroke.points.len() >= 2 {
                        let mesh =
                            geometry::generate_stroke_vertices_with_opacity(stroke, layer.opacity);
                        let converted = mesh_to_vertices(&mesh);
                        if !converted.is_empty() {
                            match mesh.mode {
                                DrawMode::TriangleStrip => {
                                    cache.strip_strokes.push(converted);
                                }
                                DrawMode::Triangles => {
                                    cache.tri_strokes.push(converted);
                                }
                            }
                        }
                    }
                }
            }

            self.canvas_version_cached = canvas.version();
        }
    }

    /// Convert a stroke mesh to flat vertex array
    fn mesh_to_vertices(mesh: &StrokeMesh) -> Vec<[f32; 6]> {
        mesh.vertices
            .chunks(6)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]])
            .collect()
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

    /// Render a frame: clear, draw strokes, draw UI
    pub fn render(&mut self, frame: &GlRenderFrame) {
        let (width, height) = frame.window_size;

        // Ensure committed stroke cache is valid
        self.ensure_cache_valid(frame.canvas);

        unsafe {
            self.gl.viewport(0, 0, width, height);
            self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.use_program(Some(self.program));
            self.gl
                .uniform_2_f32(Some(&self.canvas_size_uniform), width as f32, height as f32);

            self.gl
                .uniform_1_f32(Some(&self.zoom_uniform), frame.viewport.zoom);
            self.gl.uniform_2_f32(
                Some(&self.pan_offset_uniform),
                frame.viewport.pan_offset.0,
                frame.viewport.pan_offset.1,
            );

            self.gl.bind_vertex_array(Some(self.vao));

            // Draw committed strokes per-layer using cache, inserting active stroke
            // at the active layer position so it renders in correct order.
            let active_layer_idx = frame.canvas.active_layer();
            let layers = frame.canvas.layers();
            for (layer_idx, layer) in layers.iter().enumerate().rev() {
                if !layer.visible {
                    continue;
                }

                // Use cached committed strokes
                if layer_idx < self.layer_stroke_cache.len() {
                    let cache = &self.layer_stroke_cache[layer_idx];

                    for stroke_vertices in &cache.strip_strokes {
                        let flat: Vec<f32> = stroke_vertices.iter().flatten().copied().collect();
                        self.upload_and_draw(&flat, glow::TRIANGLE_STRIP);
                    }

                    for stroke_vertices in &cache.tri_strokes {
                        let flat: Vec<f32> = stroke_vertices.iter().flatten().copied().collect();
                        self.upload_and_draw(&flat, glow::TRIANGLES);
                    }
                }

                // Insert active stroke at the active layer position
                if layer_idx == active_layer_idx
                    && let Some(active) = frame.active_stroke
                {
                    let mesh = geometry::generate_active_stroke_vertices_with_opacity(
                        active,
                        layer.opacity,
                    );
                    if !mesh.is_empty() {
                        let mode = match mesh.mode {
                            DrawMode::TriangleStrip => glow::TRIANGLE_STRIP,
                            DrawMode::Triangles => glow::TRIANGLES,
                        };
                        self.upload_and_draw(&mesh.vertices, mode);
                    }
                }
            }

            // Reset zoom/pan for UI elements (UI stays fixed on screen)
            self.gl.uniform_1_f32(Some(&self.zoom_uniform), 1.0);
            self.gl
                .uniform_2_f32(Some(&self.pan_offset_uniform), 0.0, 0.0);

            // Batch all UI vertices into a single upload and draw call
            let mut all_ui_vertices: Vec<f32> = Vec::new();

            all_ui_vertices.extend(geometry::generate_hsv_sliders(
                frame.ui.hue,
                frame.ui.saturation,
                frame.ui.value,
            ));
            all_ui_vertices.extend(geometry::generate_custom_palette(
                &frame.ui.custom_colors,
                frame.ui.selected_custom_index as usize,
            ));
            all_ui_vertices.extend(geometry::generate_brush_size_vertices(frame.ui.brush_size));
            all_ui_vertices.extend(geometry::generate_eraser_button_vertices(
                frame.ui.is_eraser,
            ));
            all_ui_vertices.extend(geometry::generate_clear_button_vertices());
            all_ui_vertices.extend(geometry::generate_undo_button_vertices(
                frame.canvas.can_undo(),
            ));
            all_ui_vertices.extend(geometry::generate_redo_button_vertices(
                frame.canvas.can_redo(),
            ));
            all_ui_vertices.extend(geometry::generate_export_button_vertices());
            all_ui_vertices.extend(geometry::generate_zoom_in_button_vertices());
            all_ui_vertices.extend(geometry::generate_zoom_out_button_vertices());
            all_ui_vertices.extend(geometry::generate_opacity_preset_vertices(frame.ui.opacity));
            all_ui_vertices.extend(geometry::generate_brush_type_vertices(frame.ui.brush_type));
            all_ui_vertices.extend(geometry::generate_selection_tool_button(
                frame.ui.selection_tool_active,
            ));
            // Selection rect is rendered in a separate overlay pass (render_selection_overlay)

            let layer_data: Vec<(String, bool, f32, bool)> = frame
                .canvas
                .layers()
                .iter()
                .map(|l| (l.name.clone(), l.visible, l.opacity, l.locked))
                .collect();
            all_ui_vertices.extend(geometry::generate_layer_panel_vertices(
                &layer_data,
                frame.canvas.active_layer(),
                width as f32,
            ));

            if !all_ui_vertices.is_empty() {
                self.upload_and_draw(&all_ui_vertices, glow::TRIANGLES);
            }

            self.gl.bind_vertex_array(None);
        }
    }

    /// Render selection rectangle as a separate overlay pass.
    /// This is called after the main render to ensure the marching ants
    /// and moved selection strokes are always drawn on top.
    #[allow(clippy::too_many_arguments)]
    pub fn render_selection_overlay(
        &self,
        rect: crate::canvas::Rect,
        time_offset: f32,
        width: u32,
        height: u32,
        selection_strokes: &[(crate::canvas::Stroke, f32)],
        zoom: f32,
        pan_offset: (f32, f32),
    ) {
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);

            // Set canvas-space transform for both strokes and rect
            self.gl.uniform_1_f32(Some(&self.zoom_uniform), zoom);
            self.gl
                .uniform_2_f32(Some(&self.pan_offset_uniform), pan_offset.0, pan_offset.1);

            self.gl.bind_vertex_array(Some(self.vao));

            // Draw moved selection strokes in canvas space
            for (stroke, opacity) in selection_strokes {
                if stroke.points.len() >= 2 {
                    let mesh = geometry::generate_stroke_vertices_with_opacity(stroke, *opacity);
                    if !mesh.is_empty() {
                        let mode = match mesh.mode {
                            DrawMode::TriangleStrip => glow::TRIANGLE_STRIP,
                            DrawMode::Triangles => glow::TRIANGLES,
                        };
                        self.upload_and_draw(&mesh.vertices, mode);
                    }
                }
            }

            // Draw marching ants rect in canvas space
            let vertices = geometry::generate_selection_rect_vertices(rect, time_offset);
            if !vertices.is_empty() {
                self.upload_and_draw(&vertices, glow::TRIANGLES);
            }

            // Force GPU to complete the draw
            self.gl.finish();

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
