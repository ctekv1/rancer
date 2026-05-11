use glow::HasContext;

use crate::compositor::CompositeResult;

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

        let pos: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0];
        let pos_buf = gl.create_buffer().map_err(|e| e.to_string())?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(pos_buf));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &pos.to_vec().iter().flat_map(|f| f32::to_le_bytes(*f)).collect::<Vec<u8>>(), glow::STATIC_DRAW);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);

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

pub struct CanvasRenderer {
    program: glow::Program,
    texture: glow::Texture,
    vao: glow::VertexArray,
    width: u32,
    height: u32,
}

impl CanvasRenderer {
    pub fn new(gl: &glow::Context, width: u32, height: u32) -> Result<Self, String> {
        let program = create_shader_program(gl, VERTEX_SHADER, FRAGMENT_SHADER)?;

        let texture = unsafe { gl.create_texture().map_err(|e| e.to_string())? };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        }

        let vao = create_quad_vao(gl)?;

        Ok(Self {
            program,
            texture,
            vao,
            width,
            height,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn upload(
        &self,
        gl: &glow::Context,
        composite: &CompositeResult,
        x: u32,
        y: u32,
    ) {
        unsafe {
            if x == 0 && y == 0 && composite.width == self.width && composite.height == self.height {
                gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    self.width as i32,
                    self.height as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(&composite.data[..])),
                );
            } else {
                gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    x as i32,
                    y as i32,
                    composite.width as i32,
                    composite.height as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(&composite.data[..])),
                );
            }
        }
    }

    pub fn draw(
        &self,
        gl: &glow::Context,
        clear_r: f32,
        clear_g: f32,
        clear_b: f32,
    ) {
        unsafe {
            gl.viewport(0, 0, self.width as i32, self.height as i32);
            gl.clear_color(clear_r, clear_g, clear_b, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.disable(glow::BLEND);

            gl.use_program(Some(self.program));

            let tex_loc = gl.get_uniform_location(self.program, "u_texture");
            if let Some(loc) = tex_loc {
                gl.uniform_1_i32(Some(&loc), 0);
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}
