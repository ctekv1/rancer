//! WGPU rendering module for Rancer
//!
//! Provides GPU-accelerated rendering for the canvas using wgpu.
//! This module handles the rendering pipeline, shaders, and drawing operations.
//! Falls back to cairo rendering if WGPU is not available.

use crate::canvas::{Canvas, Color};
use crate::logger;

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
    /// Color palette for UI
    palette: crate::canvas::ColorPalette,
    /// Active stroke being drawn (if any)
    active_stroke: Option<crate::canvas::ActiveStroke>,
    /// Current brush size for UI
    brush_size: f32,
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
    /// Vertex buffer for strokes
    vertex_buffer: Option<wgpu::Buffer>,
    /// Uniform buffer for canvas size
    uniform_buffer: Option<wgpu::Buffer>,
    /// Number of vertices to render
    vertex_count: u32,
    /// Window size
    window_size: (u32, u32),
}

impl Renderer {
    /// Create a new renderer with WGPU initialization and cairo fallback
    pub async fn new(
        config: RendererConfig,
        window: &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync),
        window_size: (u32, u32),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        logger::info("=== RENDERER INITIALIZATION START ===");
        logger::info("Attempting WGPU initialization...");
        
        // Try to initialize WGPU
        match Self::init_wgpu(window, window_size, &config).await {
            Ok((device, queue, surface, surface_config, render_pipeline, ui_pipeline)) => {
                logger::info("✅ WGPU initialized successfully!");
                logger::info("   - Backend: GPU (WGPU)");
                logger::info(&format!("   - Device: {:?}", device));
                logger::info(&format!("   - Surface format: {:?}", surface_config.format));
                logger::info(&format!("   - MSAA samples: {}", config.msaa_samples));
                Ok(Self {
                    canvas: Canvas::new(),
                    palette: crate::canvas::ColorPalette::new(),
                    active_stroke: None,
                    brush_size: 3.0, // Default brush size
                    config,
                    backend: RenderBackend::Wgpu,
                    device: Some(device),
                    queue: Some(queue),
                    surface: Some(surface),
                    surface_config: Some(surface_config),
                    render_pipeline: Some(render_pipeline),
                    ui_pipeline: Some(ui_pipeline),
                    vertex_buffer: None,
                    uniform_buffer: None,
                    vertex_count: 0,
                    window_size,
                })
            }
            Err(e) => {
                logger::error(&format!("❌ WGPU initialization failed: {}", e));
                logger::warn("   Falling back to Cairo software rendering");
                logger::info("   - Backend: Cairo (CPU)");
                Ok(Self {
                    canvas: Canvas::new(),
                    palette: crate::canvas::ColorPalette::new(),
                    active_stroke: None,
                    brush_size: 3.0, // Default brush size
                    config,
                    backend: RenderBackend::Cairo,
                    device: None,
                    queue: None,
                    surface: None,
                    surface_config: None,
                    render_pipeline: None,
                    ui_pipeline: None,
                    vertex_buffer: None,
                    uniform_buffer: None,
                    vertex_count: 0,
                    window_size,
                })
            }
        }
    }

    /// Initialize WGPU device, queue, surface, and pipeline
    async fn init_wgpu(
        window: &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Sync),
        window_size: (u32, u32),
        config: &RendererConfig,
    ) -> Result<
        (
            wgpu::Device,
            wgpu::Queue,
            wgpu::Surface<'static>,
            wgpu::SurfaceConfiguration,
            wgpu::RenderPipeline,
            wgpu::RenderPipeline, // UI pipeline
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
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Fifo,
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

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // Position (x, y)
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // Color (r, g, b, a)
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        // Line width
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
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
                count: config.msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Create UI pipeline for rendering rectangles
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // Position (x, y)
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // Color (r, g, b, a)
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        // Line width (set to 0 for UI elements)
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
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
                count: config.msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok((device, queue, surface, surface_config, render_pipeline, ui_pipeline))
    }

    /// Resize the renderer for a new window size
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.window_size = new_size;
        
        if let (Some(surface), Some(device), Some(config)) = 
            (&self.surface, &self.device, &self.surface_config) {
            if new_size.0 > 0 && new_size.1 > 0 {
                let mut new_config = config.clone();
                new_config.width = new_size.0;
                new_config.height = new_size.1;
                surface.configure(device, &new_config);
                self.surface_config = Some(new_config);
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

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Generate vertices from canvas strokes (one continuous buffer)
        let vertices = self.generate_vertices();
        
        // Create uniform buffer for canvas size
        let uniform_data = [self.window_size.0 as f32, self.window_size.1 as f32];
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

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            // Draw each stroke separately to avoid degenerate vertices
            for stroke in self.canvas.strokes() {
                if stroke.points.len() >= 2 {
                    let vertices = self.generate_stroke_vertices(stroke);
                    let vertex_count = vertices.len() as u32;
                    if vertex_count > 0 {
                        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
                        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Active Stroke Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..vertex_count, 0..1);
                    }
                }
            }

            // Draw UI elements (color palette)
            if let Some(ui_pipeline) = &self.ui_pipeline {
                let palette_vertices = self.generate_palette_vertices(self.palette.selected_index());
                if !palette_vertices.is_empty() {
                    let ui_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("UI Vertex Buffer"),
                        contents: bytemuck::cast_slice(&palette_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    render_pass.set_pipeline(ui_pipeline);
                    render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
                    render_pass.draw(0..palette_vertices.len() as u32, 0..1);
                }

                // Draw brush size selector
                let brush_size_vertices = self.generate_brush_size_vertices(self.brush_size);
                if !brush_size_vertices.is_empty() {
                    let brush_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Brush Size Vertex Buffer"),
                        contents: bytemuck::cast_slice(&brush_size_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    render_pass.set_vertex_buffer(0, brush_vertex_buffer.slice(..));
                    render_pass.draw(0..brush_size_vertices.len() as u32, 0..1);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Generate vertices for a single stroke (as a smooth triangle strip)
    fn generate_stroke_vertices(&self, stroke: &crate::canvas::Stroke) -> Vec<[f32; 7]> {
        let mut vertices = Vec::new();
        let color = [
            stroke.color.r as f32 / 255.0,
            stroke.color.g as f32 / 255.0,
            stroke.color.b as f32 / 255.0,
            stroke.opacity,
        ];
        let half_width = stroke.width / 2.0;

        if stroke.points.len() < 2 {
            return vertices;
        }

        // Generate smooth stroke using triangle strip
        // For each point, we generate two vertices (left and right of the path)
        for i in 0..stroke.points.len() {
            let p = &stroke.points[i];
            
            // Calculate direction vector (tangent)
            let (dx, dy) = if i == 0 {
                // First point: use direction to next point
                let next = &stroke.points[i + 1];
                (next.x - p.x, next.y - p.y)
            } else if i == stroke.points.len() - 1 {
                // Last point: use direction from previous point
                let prev = &stroke.points[i - 1];
                (p.x - prev.x, p.y - prev.y)
            } else {
                // Middle point: use average direction
                let prev = &stroke.points[i - 1];
                let next = &stroke.points[i + 1];
                (next.x - prev.x, next.y - prev.y)
            };
            
            let len = (dx * dx + dy * dy).sqrt();
            
            if len < 0.001 {
                // If direction is zero, skip this point
                continue;
            }
            
            // Calculate perpendicular vector (normalized)
            let nx = -dy / len * half_width;
            let ny = dx / len * half_width;
            
            // Generate two vertices for this point (left and right)
            vertices.push([p.x + nx, p.y + ny, color[0], color[1], color[2], color[3], stroke.width]);
            vertices.push([p.x - nx, p.y - ny, color[0], color[1], color[2], color[3], stroke.width]);
        }

        vertices
    }

    /// Generate vertices for an active stroke being drawn (as a smooth triangle strip)
    fn generate_active_stroke_vertices(&self, active_stroke: &crate::canvas::ActiveStroke) -> Vec<[f32; 7]> {
        let mut vertices = Vec::new();
        let color = [
            active_stroke.color().r as f32 / 255.0,
            active_stroke.color().g as f32 / 255.0,
            active_stroke.color().b as f32 / 255.0,
            active_stroke.opacity(),
        ];
        let half_width = active_stroke.width() / 2.0;
        let points = active_stroke.points();

        if points.len() < 2 {
            return vertices;
        }

        // Generate smooth stroke using triangle strip
        // For each point, we generate two vertices (left and right of the path)
        for i in 0..points.len() {
            let p = &points[i];
            
            // Calculate direction vector (tangent)
            let (dx, dy) = if i == 0 {
                // First point: use direction to next point
                let next = &points[i + 1];
                (next.x - p.x, next.y - p.y)
            } else if i == points.len() - 1 {
                // Last point: use direction from previous point
                let prev = &points[i - 1];
                (p.x - prev.x, p.y - prev.y)
            } else {
                // Middle point: use average direction
                let prev = &points[i - 1];
                let next = &points[i + 1];
                (next.x - prev.x, next.y - prev.y)
            };
            
            let len = (dx * dx + dy * dy).sqrt();
            
            if len < 0.001 {
                // If direction is zero, skip this point
                continue;
            }
            
            // Calculate perpendicular vector (normalized)
            let nx = -dy / len * half_width;
            let ny = dx / len * half_width;
            
            // Generate two vertices for this point (left and right)
            vertices.push([p.x + nx, p.y + ny, color[0], color[1], color[2], color[3], active_stroke.width()]);
            vertices.push([p.x - nx, p.y - ny, color[0], color[1], color[2], color[3], active_stroke.width()]);
        }

        vertices
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

    /// Generate vertices for a rectangle (two triangles)
    fn generate_rectangle_vertices(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) -> Vec<[f32; 7]> {
        let r = color[0];
        let g = color[1];
        let b = color[2];
        let a = color[3];

        vec![
            // First triangle
            [x, y, r, g, b, a, 0.0],
            [x + width, y, r, g, b, a, 0.0],
            [x, y + height, r, g, b, a, 0.0],
            // Second triangle
            [x + width, y, r, g, b, a, 0.0],
            [x + width, y + height, r, g, b, a, 0.0],
            [x, y + height, r, g, b, a, 0.0],
        ]
    }

    /// Generate vertices for the color palette UI
    fn generate_palette_vertices(&self, selected_index: usize) -> Vec<[f32; 7]> {
        let mut vertices = Vec::new();
        let colors = self.palette.colors();
        
        let palette_x = 10.0;
        let palette_y = 10.0;
        let color_width = 20.0;
        let color_height = 20.0;
        let spacing = 5.0;
        let border_width = 2.0;

        for (i, color) in colors.iter().enumerate() {
            let x = palette_x + (color_width + spacing) * i as f32;
            let color_f32 = [
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0,
                1.0,
            ];
            
            // Draw border first (if selected)
            if i == selected_index {
                let border_color = [0.0, 0.0, 0.0, 1.0]; // Black border
                
                // Top border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    palette_y - border_width,
                    color_width + border_width * 2.0,
                    border_width,
                    border_color,
                ));
                
                // Bottom border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    palette_y + color_height,
                    color_width + border_width * 2.0,
                    border_width,
                    border_color,
                ));
                
                // Left border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    palette_y - border_width,
                    border_width,
                    color_height + border_width * 2.0,
                    border_color,
                ));
                
                // Right border
                vertices.extend(self.generate_rectangle_vertices(
                    x + color_width,
                    palette_y - border_width,
                    border_width,
                    color_height + border_width * 2.0,
                    border_color,
                ));
            }
            
            // Draw color swatch on top of border
            vertices.extend(self.generate_rectangle_vertices(
                x,
                palette_y,
                color_width,
                color_height,
                color_f32,
            ));
        }

        vertices
    }

    /// Generate vertices for brush size selector
    fn generate_brush_size_vertices(&self, selected_size: f32) -> Vec<[f32; 7]> {
        let mut vertices = Vec::new();
        let brush_sizes: [f32; 5] = [3.0, 5.0, 10.0, 25.0, 50.0];
        
        let selector_x = 10.0;
        let selector_y = 50.0;
        let button_size = 30.0;
        let spacing = 10.0;

        for (i, &size) in brush_sizes.iter().enumerate() {
            let x = selector_x + (button_size + spacing) * i as f32;
            
            // Draw button background (gray)
            let bg_color = [0.8, 0.8, 0.8, 1.0];
            vertices.extend(self.generate_rectangle_vertices(
                x,
                selector_y,
                button_size,
                button_size,
                bg_color,
            ));

            // Draw brush size indicator (circle approximation with rectangle)
            let indicator_size = size.min(button_size - 4.0);
            let indicator_x = x + (button_size - indicator_size) / 2.0;
            let indicator_y = selector_y + (button_size - indicator_size) / 2.0;
            let indicator_color = [0.0, 0.0, 0.0, 1.0]; // Black
            
            vertices.extend(self.generate_rectangle_vertices(
                indicator_x,
                indicator_y,
                indicator_size,
                indicator_size,
                indicator_color,
            ));

            // Draw border for selected size
            if (size - selected_size).abs() < 0.1 {
                let border_color = [0.0, 0.0, 1.0, 1.0]; // Blue border
                let border_width = 2.0;
                
                // Top border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    selector_y - border_width,
                    button_size + border_width * 2.0,
                    border_width,
                    border_color,
                ));
                
                // Bottom border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    selector_y + button_size,
                    button_size + border_width * 2.0,
                    border_width,
                    border_color,
                ));
                
                // Left border
                vertices.extend(self.generate_rectangle_vertices(
                    x - border_width,
                    selector_y - border_width,
                    border_width,
                    button_size + border_width * 2.0,
                    border_color,
                ));
                
                // Right border
                vertices.extend(self.generate_rectangle_vertices(
                    x + button_size,
                    selector_y - border_width,
                    border_width,
                    button_size + border_width * 2.0,
                    border_color,
                ));
            }
        }

        vertices
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
