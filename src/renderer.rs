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

        Ok((
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            ui_pipeline,
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
            let mut new_config = config.clone();
            new_config.width = new_size.0;
            new_config.height = new_size.1;
            surface.configure(device, &new_config);
            self.surface_config = Some(new_config);
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
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Generate vertices from canvas strokes (one continuous buffer)
        let _vertices = self.generate_vertices();

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

            // Draw UI elements (color palette)
            if let Some(ui_pipeline) = &self.ui_pipeline {
                let palette_vertices =
                    self.generate_palette_vertices(self.palette.selected_index());
                if !palette_vertices.is_empty() {
                    let ui_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
                    let brush_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

    /// Generate vertices for the color palette UI
    fn generate_palette_vertices(&self, selected_index: usize) -> Vec<[f32; 7]> {
        let flat = geometry::generate_palette_vertices(&self.palette, selected_index);
        to_vertices_7(&flat, 0.0)
    }

    /// Generate vertices for brush size selector
    fn generate_brush_size_vertices(&self, selected_size: f32) -> Vec<[f32; 7]> {
        let flat = geometry::generate_brush_size_vertices(selected_size);
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
