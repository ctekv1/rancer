//! WGPU rendering module for Rancer
//!
//! Provides GPU-accelerated rendering for the canvas using wgpu.
//! This module handles the rendering pipeline, shaders, and drawing operations.
//! Falls back to cairo rendering if WGPU is not available.

use crate::canvas::{Canvas, Color};
use crate::geometry;
use crate::logger;

/// Parse hex color string to Color
pub fn hex_to_color(hex: &str) -> Color {
    geometry::hex_to_color(hex)
}

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
            msaa_samples: 4, // Enable 4x MSAA for smoother rendering
        }
    }
}

/// Rendering backend enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderBackend {
    /// WGPU GPU-accelerated rendering
    Wgpu,
    /// Cairo software rendering (fallback)
    Cairo,
}

/// WGPU-based renderer for the canvas
pub struct Renderer {
    /// Current canvas to render
    canvas: Canvas,
    /// HSV color values (Hue, Saturation, Value)
    hue: f32,
    saturation: f32,
    value: f32,
    /// Custom saved colors
    custom_colors: Vec<[u8; 3]>,
    /// Selected custom color index (-1 if none)
    selected_custom_index: i32,
    /// Active stroke being drawn (if any)
    active_stroke: Option<crate::canvas::ActiveStroke>,
    /// Current brush size for UI
    brush_size: f32,
    /// Current brush opacity
    opacity: f32,
    /// Eraser mode active
    is_eraser: bool,
    /// Configuration
    pub config: RendererConfig,
    /// Active rendering backend
    backend: RenderBackend,
    /// WGPU device (if available)
    device: Option<wgpu::Device>,
    /// WGPU queue (if available)
    queue: Option<wgpu::Queue>,
    /// WGPU surface (if available)
    surface: Option<wgpu::Surface<'static>>,
    /// WGPU surface configuration
    surface_config: Option<wgpu::SurfaceConfiguration>,
    /// WGPU render pipeline for strokes
    render_pipeline: Option<wgpu::RenderPipeline>,
    /// WGPU render pipeline for UI elements
    ui_pipeline: Option<wgpu::RenderPipeline>,
    /// Window size
    window_size: (u32, u32),
    /// Pipeline layout for recreation
    pipeline_layout: Option<wgpu::PipelineLayout>,
    /// Shader module for recreation
    shader: Option<wgpu::ShaderModule>,
    /// Window reference for pre_present_notify
    window: Option<std::sync::Arc<winit::window::Window>>,
}

impl Renderer {
    /// Create a new renderer with WGPU initialization and cairo fallback
    pub async fn new(
        config: RendererConfig,
        window: std::sync::Arc<winit::window::Window>,
        window_size: (u32, u32),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        logger::info("=== RENDERER INITIALIZATION START ===");
        logger::info("Attempting WGPU initialization...");

        // Try to initialize WGPU
        match Self::init_wgpu(&window, window_size, &config).await {
            Ok((device, queue, surface, surface_config, render_pipeline, ui_pipeline, pipeline_layout, shader)) => {
                logger::info("✅ WGPU initialized successfully!");
                logger::info("   - Backend: GPU (WGPU)");
                logger::info(&format!("   - Device: {:?}", device));
                logger::info(&format!("   - Surface format: {:?}", surface_config.format));
                Ok(Self {
                    canvas: Canvas::new(),
                    hue: 0.0,
                    saturation: 100.0,
                    value: 100.0,
                    custom_colors: Vec::new(),
                    selected_custom_index: -1,
                    active_stroke: None,
                    brush_size: 3.0,
                    opacity: 1.0,
                    is_eraser: false,
                    config,
                    backend: RenderBackend::Wgpu,
                    device: Some(device),
                    queue: Some(queue),
                    surface: Some(surface),
                    surface_config: Some(surface_config),
                    render_pipeline: Some(render_pipeline),
                    ui_pipeline: Some(ui_pipeline),
                    window_size,
                    pipeline_layout: Some(pipeline_layout),
                    shader: Some(shader),
                    window: Some(window),
                })
            }
            Err(e) => {
                logger::error(&format!("❌ WGPU initialization failed: {}", e));
                logger::warn("   Falling back to Cairo software rendering");
                logger::info("   - Backend: Cairo (CPU)");
                Ok(Self {
                    canvas: Canvas::new(),
                    hue: 0.0,
                    saturation: 100.0,
                    value: 100.0,
                    custom_colors: Vec::new(),
                    selected_custom_index: -1,
                    active_stroke: None,
                    brush_size: 3.0,
                    opacity: 1.0,
                    is_eraser: false,
                    config,
                    backend: RenderBackend::Cairo,
                    device: None,
                    queue: None,
                    surface: None,
                    surface_config: None,
                    render_pipeline: None,
                    ui_pipeline: None,
                    window_size,
                    pipeline_layout: None,
                    shader: None,
                    window: Some(window),
                })
            }
        }
    }

    /// Create render pipelines with the given sample count
    fn create_pipelines(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
        surface_format: wgpu::TextureFormat,
        sample_count: u32,
    ) -> (wgpu::RenderPipeline, wgpu::RenderPipeline) {
        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create UI pipeline for rendering rectangles
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        (render_pipeline, ui_pipeline)
    }

    /// Initialize WGPU device, queue, surface, and pipeline
    async fn init_wgpu(
        window: &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync),
        window_size: (u32, u32),
        _config: &RendererConfig,
    ) -> Result<
        (
            wgpu::Device,
            wgpu::Queue,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
            wgpu::RenderPipeline,
            wgpu::RenderPipeline,
            wgpu::PipelineLayout,
            wgpu::ShaderModule,
        ),
        Box<dyn std::error::Error>,
    > {
        // Create WGPU instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        // SAFETY: The surface is valid for the lifetime of the window, which is managed by GTK4
        #[allow(clippy::missing_transmute_annotations)]
        let surface = unsafe { std::mem::transmute(instance.create_surface(window)?) };

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        logger::info(&format!("Selected adapter: {:?}", adapter.get_info()));

        // Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        // Get device limits to clamp surface size
        let device_limits = device.limits();
        let max_texture_size = device_limits.max_texture_dimension_2d;
        logger::info(&format!("Max texture dimension: {}", max_texture_size));

        // Clamp window size to GPU limits
        let surface_width = window_size.0.min(max_texture_size);
        let surface_height = window_size.1.min(max_texture_size);
        if surface_width != window_size.0 || surface_height != window_size.1 {
            logger::warn(&format!(
                "Window size {}x{} exceeds GPU limit {}x{}, clamping",
                window_size.0, window_size.1, surface_width, surface_height
            ));
        }

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_width,
            height: surface_height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
        });

        // Create bind group layout for uniforms
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // For swapchain rendering, use sample_count=1 (MSAA with swapchains requires
        // creating intermediate render targets, which is more complex)
        // The config.msaa_samples value is stored but not used for pipeline creation
        let sample_count = 1;
        logger::info(&format!("Using MSAA sample count: {} (swapchain rendering)", sample_count));

        // Create pipelines with the determined sample count
        let (render_pipeline, ui_pipeline) =
            Self::create_pipelines(&device, &shader, &pipeline_layout, surface_format, sample_count);

        Ok((
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            ui_pipeline,
            pipeline_layout,
            shader,
        ))
    }

    /// Resize the renderer for a new window size
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.window_size = new_size;

        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &self.surface_config)
            && new_size.0 > 0
            && new_size.1 > 0
        {
            let max_texture_size = device.limits().max_texture_dimension_2d;
            let surface_width = new_size.0.min(max_texture_size);
            let surface_height = new_size.1.min(max_texture_size);

            let mut new_config = config.clone();
            new_config.width = surface_width;
            new_config.height = surface_height;
            surface.configure(device, &new_config);
            let surface_format = new_config.format;
            self.surface_config = Some(new_config);

            // Recreate pipelines on resize (sample_count=1 for swapchain rendering)
            if let (Some(device), Some(pipeline_layout), Some(shader)) = (
                &self.device,
                &self.pipeline_layout,
                &self.shader,
            ) {
                let (render_pipeline, ui_pipeline) = Self::create_pipelines(
                    device,
                    shader,
                    pipeline_layout,
                    surface_format,
                    1,
                );
                self.render_pipeline = Some(render_pipeline);
                self.ui_pipeline = Some(ui_pipeline);
            }
        }
    }

    /// Set the canvas to render
    pub fn set_canvas(&mut self, canvas: Canvas) {
        self.canvas = canvas;
    }

    /// Set the active stroke being drawn
    pub fn set_active_stroke(&mut self, active_stroke: Option<crate::canvas::ActiveStroke>) {
        self.active_stroke = active_stroke;
    }

    /// Set the current brush size for UI
    pub fn set_brush_size(&mut self, brush_size: f32) {
        self.brush_size = brush_size;
    }

    /// Set the eraser mode
    pub fn set_eraser(&mut self, is_eraser: bool) {
        self.is_eraser = is_eraser;
    }

    /// Set the brush opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    /// Set HSV values
    pub fn set_hsv(&mut self, h: f32, s: f32, v: f32) {
        self.hue = h.clamp(0.0, 360.0);
        self.saturation = s.clamp(0.0, 100.0);
        self.value = v.clamp(0.0, 100.0);
        self.selected_custom_index = -1; // Deselect custom color
    }

    /// Get current HSV values
    pub fn get_hsv(&self) -> (f32, f32, f32) {
        (self.hue, self.saturation, self.value)
    }

    /// Get current color as Color struct
    pub fn current_color(&self) -> crate::canvas::Color {
        crate::canvas::hsv_to_rgb(self.hue, self.saturation, self.value)
    }

    /// Set custom colors
    pub fn set_custom_colors(&mut self, colors: Vec<[u8; 3]>) {
        self.custom_colors = colors;
    }

    /// Get custom colors
    pub fn get_custom_colors(&self) -> &[[u8; 3]] {
        &self.custom_colors
    }

    /// Set selected custom color index
    pub fn set_selected_custom_index(&mut self, index: i32) {
        self.selected_custom_index = index;
        if index >= 0 && (index as usize) < self.custom_colors.len() {
            let color = self.custom_colors[index as usize];
            let hsv = crate::canvas::rgb_to_hsv(crate::canvas::Color {
                r: color[0],
                g: color[1],
                b: color[2],
                a: 255,
            });
            self.hue = hsv.h;
            self.saturation = hsv.s;
            self.value = hsv.v;
        }
    }

    /// Get selected custom color index
    pub fn get_selected_custom_index(&self) -> i32 {
        self.selected_custom_index
    }

    /// Add a custom color
    pub fn add_custom_color(&mut self, color: crate::canvas::Color) {
        self.custom_colors.push([color.r, color.g, color.b]);
        // Keep max 10 colors
        if self.custom_colors.len() > 10 {
            self.custom_colors.remove(0);
        }
    }

    /// Get a mutable reference to the canvas
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// Render the current frame
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        match self.backend {
            RenderBackend::Wgpu => {
                logger::debug("[RENDER] Using WGPU backend (GPU-accelerated)");
                self.render_wgpu()
            }
            RenderBackend::Cairo => {
                logger::debug("[RENDER] Using Cairo backend (CPU software rendering)");
                // Cairo rendering is handled by GTK4 draw callback
                Ok(())
            }
        }
    }

    /// Render using WGPU
    fn render_wgpu(&mut self) -> Result<(), wgpu::SurfaceError> {
        use wgpu::util::DeviceExt;

        let (surface, device, queue, pipeline) = match (
            &self.surface,
            &self.device,
            &self.queue,
            &self.render_pipeline,
        ) {
            (Some(s), Some(d), Some(q), Some(p)) => (s, d, q, p),
            _ => return Err(wgpu::SurfaceError::Lost),
        };

        // Ensure surface is configured with current window size before rendering
        // Clamp to GPU limits to prevent invalid configurations
        if let Some(config) = &self.surface_config {
            let max_texture_size = device.limits().max_texture_dimension_2d;
            let clamped_width = self.window_size.0.min(max_texture_size);
            let clamped_height = self.window_size.1.min(max_texture_size);
            
            if config.width != clamped_width || config.height != clamped_height {
                let mut new_config = config.clone();
                new_config.width = clamped_width;
                new_config.height = clamped_height;
                surface.configure(device, &new_config);
                self.surface_config = Some(new_config);
            }
        }

        // Get the next texture to render to
        let output = surface.get_current_texture()?;

        // Check if surface is suboptimal and needs reconfiguration
        if output.suboptimal {
            logger::debug("Surface suboptimal, reconfiguring");
            if let Some(config) = &self.surface_config {
                surface.configure(device, config);
            }
        }

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Generate vertices from canvas strokes (one continuous buffer)
        let _vertices = self.generate_vertices();

        // Use actual texture dimensions to ensure consistency with viewport
        let texture_width = output.texture.width() as f32;
        let texture_height = output.texture.height() as f32;
        let uniform_data = [texture_width, texture_height];
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.config.clear_color.r as f64 / 255.0,
                            g: self.config.clear_color.g as f64 / 255.0,
                            b: self.config.clear_color.b as f64 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_viewport(
                0.0,
                0.0,
                output.texture.width() as f32,
                output.texture.height() as f32,
                0.0,
                1.0,
            );

            logger::debug(&format!(
                "[RENDER] Texture: {}x{}, Uniform: {}x{}",
                output.texture.width(),
                output.texture.height(),
                uniform_data[0] as u32,
                uniform_data[1] as u32
            ));

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            // Draw each stroke separately to avoid degenerate vertices
            for stroke in self.canvas.strokes() {
                if stroke.points.len() >= 2 {
                    let vertices = self.generate_stroke_vertices(stroke);
                    let vertex_count = vertices.len() as u32;
                    if vertex_count > 0 {
                        let vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Stroke Vertex Buffer"),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..vertex_count, 0..1);
                    }
                }
            }

            // Draw active stroke if present
            if let Some(active_stroke) = &self.active_stroke {
                let points = active_stroke.points();
                if points.len() >= 2 {
                    let vertices = self.generate_active_stroke_vertices(active_stroke);
                    let vertex_count = vertices.len() as u32;
                    if vertex_count > 0 {
                        let vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Active Stroke Vertex Buffer"),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..vertex_count, 0..1);
                    }
                }
            }

            // Draw UI elements (HSV color picker)
            if let Some(ui_pipeline) = &self.ui_pipeline {
                // Draw HSV sliders
                let hsv_vertices = self.generate_hsv_sliders(self.hue, self.saturation, self.value);
                if !hsv_vertices.is_empty() {
                    let ui_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("HSV Slider Vertex Buffer"),
                            contents: bytemuck::cast_slice(&hsv_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(ui_pipeline);
                    render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
                    render_pass.draw(0..hsv_vertices.len() as u32, 0..1);
                }

                // Draw custom palette
                let custom_palette_vertices = self.generate_custom_palette_vertices();
                if !custom_palette_vertices.is_empty() {
                    let palette_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Custom Palette Vertex Buffer"),
                            contents: bytemuck::cast_slice(&custom_palette_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_vertex_buffer(0, palette_vertex_buffer.slice(..));
                    render_pass.draw(0..custom_palette_vertices.len() as u32, 0..1);
                }

                // Draw brush size selector
                let brush_size_vertices = self.generate_brush_size_vertices(self.brush_size);
                if !brush_size_vertices.is_empty() {
                    let brush_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Brush Size Vertex Buffer"),
                            contents: bytemuck::cast_slice(&brush_size_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_vertex_buffer(0, brush_vertex_buffer.slice(..));
                    render_pass.draw(0..brush_size_vertices.len() as u32, 0..1);

                    // Draw eraser button
                    let eraser_vertices = self.generate_eraser_button_vertices(self.is_eraser);
                    if !eraser_vertices.is_empty() {
                        let eraser_vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Eraser Button Vertex Buffer"),
                                contents: bytemuck::cast_slice(&eraser_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, eraser_vertex_buffer.slice(..));
                        render_pass.draw(0..eraser_vertices.len() as u32, 0..1);
                    }

                    // Draw clear button
                    let clear_vertices = self.generate_clear_button_vertices();
                    if !clear_vertices.is_empty() {
                        let clear_vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Clear Button Vertex Buffer"),
                                contents: bytemuck::cast_slice(&clear_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, clear_vertex_buffer.slice(..));
                        render_pass.draw(0..clear_vertices.len() as u32, 0..1);
                    }

                    // Draw undo button
                    let undo_vertices = self.generate_undo_button_vertices();
                    if !undo_vertices.is_empty() {
                        let undo_vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Undo Button Vertex Buffer"),
                                contents: bytemuck::cast_slice(&undo_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, undo_vertex_buffer.slice(..));
                        render_pass.draw(0..undo_vertices.len() as u32, 0..1);
                    }

                    // Draw redo button
                    let redo_vertices = self.generate_redo_button_vertices();
                    if !redo_vertices.is_empty() {
                        let redo_vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Redo Button Vertex Buffer"),
                                contents: bytemuck::cast_slice(&redo_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, redo_vertex_buffer.slice(..));
                        render_pass.draw(0..redo_vertices.len() as u32, 0..1);
                    }

                    // Draw opacity preset buttons
                    let opacity_vertices = self.generate_opacity_preset_vertices();
                    if !opacity_vertices.is_empty() {
                        let opacity_vertex_buffer =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Opacity Preset Vertex Buffer"),
                                contents: bytemuck::cast_slice(&opacity_vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        render_pass.set_vertex_buffer(0, opacity_vertex_buffer.slice(..));
                        render_pass.draw(0..opacity_vertices.len() as u32, 0..1);
                    }
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        
        // Notify window before presenting to help compositor update window regions
        if let Some(ref window) = self.window {
            window.pre_present_notify();
        }
        
        output.present();

        Ok(())
    }

    /// Generate vertices for a single stroke (as a smooth triangle strip)
    fn generate_stroke_vertices(&self, stroke: &crate::canvas::Stroke) -> Vec<[f32; 7]> {
        let flat = geometry::generate_stroke_vertices(stroke);
        to_vertices_7(&flat, stroke.width)
    }

    /// Generate vertices for an active stroke being drawn (as a smooth triangle strip)
    fn generate_active_stroke_vertices(
        &self,
        active_stroke: &crate::canvas::ActiveStroke,
    ) -> Vec<[f32; 7]> {
        let flat = geometry::generate_active_stroke_vertices(active_stroke);
        to_vertices_7(&flat, active_stroke.width())
    }

    /// Generate vertices from canvas strokes for WGPU rendering
    fn generate_vertices(&self) -> Vec<[f32; 6]> {
        let mut vertices = Vec::new();

        for stroke in self.canvas.strokes() {
            let color = [
                stroke.color.r as f32 / 255.0,
                stroke.color.g as f32 / 255.0,
                stroke.color.b as f32 / 255.0,
                stroke.opacity,
            ];

            for point in &stroke.points {
                vertices.push([point.x, point.y, color[0], color[1], color[2], color[3]]);
            }

            // Add a degenerate vertex to separate strokes
            if !stroke.points.is_empty() {
                vertices.push([0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
            }
        }

        vertices
    }

    /// Generate vertices for HSV sliders
    fn generate_hsv_sliders(&self, h: f32, s: f32, v: f32) -> Vec<[f32; 7]> {
        let flat = geometry::generate_hsv_sliders(h, s, v);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for custom palette
    fn generate_custom_palette_vertices(&self) -> Vec<[f32; 7]> {
        let flat = geometry::generate_custom_palette(
            &self.custom_colors,
            self.selected_custom_index as usize,
        );
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for brush size selector
    fn generate_brush_size_vertices(&self, selected_size: f32) -> Vec<[f32; 7]> {
        let flat = geometry::generate_brush_size_vertices(selected_size);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for eraser button
    fn generate_eraser_button_vertices(&self, is_active: bool) -> Vec<[f32; 7]> {
        let flat = geometry::generate_eraser_button_vertices(is_active);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for clear canvas button
    fn generate_clear_button_vertices(&self) -> Vec<[f32; 7]> {
        let flat = geometry::generate_clear_button_vertices();
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for undo button
    fn generate_undo_button_vertices(&self) -> Vec<[f32; 7]> {
        let can_undo = self.canvas.can_undo();
        let flat = geometry::generate_undo_button_vertices(can_undo);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for redo button
    fn generate_redo_button_vertices(&self) -> Vec<[f32; 7]> {
        let can_redo = self.canvas.can_redo();
        let flat = geometry::generate_redo_button_vertices(can_redo);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for opacity preset buttons
    fn generate_opacity_preset_vertices(&self) -> Vec<[f32; 7]> {
        let flat = geometry::generate_opacity_preset_vertices(self.opacity);
        to_vertices_7(&flat, 0.0)
    }

    /// Render using software fallback (stub - WGPU is now primary)
    /// This method is kept for compatibility but WGPU is the primary renderer
    pub fn render_software(&self, _width: u32, _height: u32) {
        // Software rendering fallback is no longer needed with winit + WGPU
        // WGPU handles all rendering through the GPU
        logger::debug("Software rendering fallback called - using WGPU instead");
    }

    /// Get the current canvas
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Check if WGPU backend is being used
    pub fn is_wgpu(&self) -> bool {
        self.backend == RenderBackend::Wgpu
    }

    /// Get the active backend
    pub fn backend(&self) -> RenderBackend {
        self.backend
    }

    /// Print the current backend status
    pub fn print_backend_status(&self) {
        logger::info("=== RENDERER STATUS ===");
        match self.backend {
            RenderBackend::Wgpu => {
                logger::info("Backend: GPU (WGPU)");
                logger::info(&format!("Device: {:?}", self.device.is_some()));
                logger::info(&format!("Surface: {:?}", self.surface.is_some()));
                logger::info(&format!("Pipeline: {:?}", self.render_pipeline.is_some()));
            }
            RenderBackend::Cairo => {
                logger::info("Backend: CPU (Cairo)");
                logger::info("Note: Using software rendering fallback");
            }
        }
        logger::info(&format!("Window size: {:?}", self.window_size));
        logger::info(&format!("MSAA samples: {}", self.config.msaa_samples));
        logger::info("======================");
    }
}

/// Convert flat vertex data (6 floats/vertex) to WGPU format (7 floats/vertex with line_width)
fn to_vertices_7(flat: &[f32], line_width: f32) -> Vec<[f32; 7]> {
    flat.chunks(6)
        .map(|chunk| {
            [
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], line_width,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.clear_color, Color::WHITE);
        assert_eq!(config.msaa_samples, 4);
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
