//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2 with OpenGL rendering.

use glow::HasContext;
use sdl2::event::Event;
use sdl2::video::GLContext;
use sdl2::video::Window;

use crate::canvas::Canvas;
use crate::preferences::Preferences;
use crate::viewport::{Viewport, DEFAULT_CANVAS_COLOR};

pub struct Sdl2App {
    window: Window,
    gl: glow::Context,
    gl_context: sdl2::video::GLContext,
    width: u32,
    height: u32,
    viewport: Viewport,
    canvas: Canvas,
    program: glow::Program,
    texture: glow::Texture,
    vao: glow::VertexArray,
}

#[cfg(test)]
impl Sdl2App {
    pub fn program(&self) -> &glow::Program {
        &self.program
    }
    
    pub fn texture(&self) -> &glow::Texture {
        &self.texture
    }
}

pub const VERTEX_SHADER: &str = r#"
#version 450 core
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texCoord;
out vec2 v_texCoord;
void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_texCoord = a_texCoord;
}
"#;

pub const FRAGMENT_SHADER: &str = r#"
#version 450 core
precision mediump float;
in vec2 v_texCoord;
out vec4 fragColor;
uniform sampler2D u_texture;
void main() {
    fragColor = texture(u_texture, v_texCoord);
}
"#;

fn create_shader_program(gl: &glow::Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
    unsafe {
        let vshader = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| e.to_string())?;
        gl.shader_source(vshader, vs);
        gl.compile_shader(vshader);
        if !gl.get_shader_compile_status(vshader) {
            let log = gl.get_shader_info_log(vshader);
            gl.delete_shader(vshader);
            return Err(format!("Vertex shader compilation failed: {}", log));
        }

        let fshader = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| e.to_string())?;
        gl.shader_source(fshader, fs);
        gl.compile_shader(fshader);
        if !gl.get_shader_compile_status(fshader) {
            let log = gl.get_shader_info_log(fshader);
            gl.delete_shader(vshader);
            gl.delete_shader(fshader);
            return Err(format!("Fragment shader compilation failed: {}", log));
        }

        let program = gl.create_program().map_err(|e| e.to_string())?;
        gl.attach_shader(program, vshader);
        gl.attach_shader(program, fshader);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_shader(vshader);
            gl.delete_shader(fshader);
            gl.delete_program(program);
            return Err(format!("Shader program linking failed: {}", log));
        }

        gl.delete_shader(vshader);
        gl.delete_shader(fshader);
        Ok(program)
    }
}

fn create_quad_vao(gl: &glow::Context) -> Result<glow::VertexArray, String> {
    unsafe {
        let vao = gl.create_vertex_array().map_err(|e| e.to_string())?;
        gl.bind_vertex_array(Some(vao));

        // Position vertices (full screen quad in NDC)
        let pos: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let pos_buf = gl.create_buffer().map_err(|e| e.to_string())?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(pos_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &pos.to_vec().iter().flat_map(|f| f32::to_le_bytes(*f)).collect::<Vec<u8>>(), glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);

        // Texture coordinates
        let tex: [f32; 8] = [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
        let tex_buf = gl.create_buffer().map_err(|e| e.to_string())?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(tex_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &tex.to_vec().iter().flat_map(|f| f32::to_le_bytes(*f)).collect::<Vec<u8>>(), glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);
        Ok(vao)
    }
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

        // Compile shaders
        let program = create_shader_program(&gl, VERTEX_SHADER, FRAGMENT_SHADER)?;

        // Create texture
        let texture = unsafe { gl.create_texture().map_err(|e| e.to_string())? };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        }

        // Create VAO with quad geometry
        let vao = create_quad_vao(&gl)?;

        unsafe {
            gl.clear_color(0.94, 0.94, 0.94, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        window.gl_swap_window();

        let size = window.size();
        let width = size.0 as u32;
        let height = size.1 as u32;

        let canvas = Canvas::new();

        Ok(Self {
            window,
            gl,
            gl_context,
            width,
            height,
            viewport: Viewport::new(1280, 720),
            canvas,
            program,
            texture,
            vao,
        })
    }

    pub fn run(&mut self) {
        let mut event_pump = self.window.subsystem().sdl()
            .event_pump()
            .map_err(|e| format!("Failed to create event pump: {}", e))
            .unwrap();

        self.window.gl_make_current(&self.gl_context).ok();

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

            self.render_frame(canvas_r, canvas_g, canvas_b);
            self.window.gl_swap_window();

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

fn render_frame(&mut self, r: f32, g: f32, b: f32) {
        let layer = &self.canvas.layers()[self.canvas.active_layer()];
        let raster = match &layer.content {
            crate::canvas::LayerContent::Raster(r) => &r.image,
            _ => return,
        };
        
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                raster.width as i32,
                raster.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&raster.data),
            );

            self.gl.clear_color(r, g, b, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            self.gl.use_program(Some(self.program));
            
            let tex_loc = self.gl.get_uniform_location(self.program, "u_texture");
            if let Some(loc) = tex_loc {
                self.gl.uniform_1_i32(Some(&loc), 0);
            }

            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
            self.gl.disable(glow::BLEND);
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