//! SDL2 window backend
//!
//! Provides cross-platform window management using SDL2 with OpenGL rendering.

use glow::HasContext;
use sdl2::event::Event;

use crate::app::AppState;
use crate::events::AppEvent;
use crate::preferences::Preferences;
use crate::ui::egui_integration::EguiIntegration;
use crate::ui::UiState;

/// Convert an SDL2 event to an AppEvent
pub fn sdl_event_to_app_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Quit { .. } => Some(AppEvent::Quit),
        Event::MouseButtonDown { x, y, .. } => Some(AppEvent::Press {
            x: x as f32,
            y: y as f32,
        }),
        Event::MouseButtonUp { x, y, .. } => Some(AppEvent::Release {
            x: x as f32,
            y: y as f32,
        }),
        Event::MouseMotion {
            x,
            y,
            mousestate,
            ..
        } => {
            if mousestate.left() {
                Some(AppEvent::Drag {
                    x: x as f32,
                    y: y as f32,
                })
            } else {
                None
            }
        }
        Event::KeyDown {
            keycode: Some(keycode),
            ..
        } => Some(AppEvent::Key {
            code: format!("{:?}", keycode).to_lowercase(),
        }),
        _ => None,
    }
}

pub struct Sdl2App {
    window: sdl2::video::Window,
    gl: glow::Context,
    gl_context: sdl2::video::GLContext,
    app_state: AppState,
    ui_state: UiState,
    icon_cache: crate::ui::egui_impl::IconCache,
    egui: EguiIntegration,
    program: glow::Program,
    texture: glow::Texture,
    vao: glow::VertexArray,
    last_rendered_version: u64,
    has_rendered: bool,
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

        let app_state = AppState::new(width, height);
        let ui_state = UiState::new();
        
        // Create egui integration
        let mut egui = EguiIntegration::new(&window, &gl_context, &gl)
            .map_err(|e| format!("Failed to create egui integration: {}", e))?;
        
        // Create icon cache (needs egui context)
        let icon_cache = crate::ui::egui_impl::IconCache::new(egui.ctx());
        
        Ok(Self {
            window,
            gl,
            gl_context,
            app_state,
            ui_state,
            icon_cache,
            egui,
            program,
            texture,
            vao,
            last_rendered_version: 0,
            has_rendered: false,
        })
    }

    pub fn run(&mut self) {
        let mut event_pump = self.window.subsystem().sdl()
            .event_pump()
            .map_err(|e| format!("Failed to create event pump: {}", e))
            .unwrap();

        self.window.gl_make_current(&self.gl_context).ok();

        // Enable VSync to prevent screen tearing/flash
        self.window.subsystem().sdl().video()
            .unwrap()
            .gl_set_swap_interval(1)
            .ok();

        // Use canvas's actual background color for clear, not DEFAULT_CANVAS_COLOR
        let bg = self.app_state.canvas().background_color;
        let canvas_r = bg.r as f32 / 255.0;
        let canvas_g = bg.g as f32 / 255.0;
        let canvas_b = bg.b as f32 / 255.0;

        'running: loop {
            let mut has_work = false;
            for event in event_pump.poll_iter() {
                // Pass event to egui first
                self.egui.handle_event(&self.window, &event);
                
                // Then convert to AppEvent for the app
                if let Some(app_event) = sdl_event_to_app_event(event) {
                    has_work = true;
                    match app_event {
                        AppEvent::Quit => break 'running,
                        _ => self.app_state.handle_event(app_event),
                    }
                }
            }

            // Check if canvas changed since last render
            let current_version = self.app_state.canvas().version();
            if current_version != self.last_rendered_version {
                has_work = true;
            }

            // Render and swap
            self.render_frame(canvas_r, canvas_g, canvas_b);
            
            // Render egui on top
                self.egui.run_and_render(&self.window, |ctx: &egui_sdl2::egui::Context| {
                    self.ui_state.apply_to_app(&mut self.app_state);
                    crate::ui::show_ui(ctx, &mut self.app_state, &mut self.ui_state, &self.icon_cache);
                });
            
            self.window.gl_swap_window();
            self.last_rendered_version = self.app_state.canvas().version();

            // Yield CPU when idle
            if !has_work {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    fn render_frame(&mut self, r: f32, g: f32, b: f32) {
        let canvas_width = self.app_state.canvas().width() as i32;
        let canvas_height = self.app_state.canvas().height() as i32;
        let current_version = self.app_state.canvas().version();
        let needs_update = !self.has_rendered || current_version != self.last_rendered_version;
        
        if needs_update {
            unsafe {
                if !self.has_rendered {
                    // First frame: full composite + texture allocation
                    let composite = self.app_state.canvas().composite_all();
                    self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                    self.gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA as i32,
                        canvas_width,
                        canvas_height,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(Some(&composite.data[..])),
                    );
                } else {
                    // Subsequent frames: use dirty rect for partial update
                    let dirty = self.app_state.canvas().dirty_rect().clone();
                    
                    if !dirty.is_empty() && (dirty.width as i64 * dirty.height as i64) < (canvas_width as i64 * canvas_height as i64 / 2) {
                        // Small dirty region: partial update
                        let composite = self.app_state.canvas().composite_rect(
                            dirty.x, dirty.y, dirty.width, dirty.height
                        );
                        
                        if composite.width > 0 && composite.height > 0 {
                            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                            self.gl.tex_sub_image_2d(
                                glow::TEXTURE_2D,
                                0,
                                dirty.x as i32,
                                dirty.y as i32,
                                composite.width as i32,
                                composite.height as i32,
                                glow::RGBA,
                                glow::UNSIGNED_BYTE,
                                glow::PixelUnpackData::Slice(Some(&composite.data[..])),
                            );
                        }
                    } else {
                        // Large/empty dirty region: full update
                        let composite = self.app_state.canvas().composite_all();
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
                        self.gl.tex_sub_image_2d(
                            glow::TEXTURE_2D,
                            0,
                            0,
                            0,
                            canvas_width,
                            canvas_height,
                            glow::RGBA,
                            glow::UNSIGNED_BYTE,
                            glow::PixelUnpackData::Slice(Some(&composite.data[..])),
                        );
                    }
                }
            }
            
            self.last_rendered_version = current_version;
            self.has_rendered = true;
            
            // Consume the dirty rect so it doesn't accumulate
            self.app_state.canvas_mut().consume_dirty_rect();
        }
        
        unsafe {
            // Set viewport to match canvas size
            self.gl.viewport(0, 0, canvas_width, canvas_height);
            
            // Clear with canvas background color (matches composite background)
            self.gl.clear_color(r, g, b, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.disable(glow::BLEND);

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
