//! WGPU rendering module for Rancer
//!
//! Provides GPU-accelerated rendering for the canvas using wgpu.
//! The renderer is stateless — all render data is passed via `RenderFrame`.

use crate::canvas::{ActiveStroke, BrushType, Canvas, Color, LayerContent, Stroke};
use crate::geometry::{self, DrawMode, StrokeMesh};
use crate::logger;
use std::cell::RefCell;

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
            msaa_samples: 4,
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

/// Committed stroke mesh for a single stroke
type CachedStrokeMesh = Vec<[f32; 6]>;

/// Committed stroke data for a single layer
#[derive(Clone)]
struct LayerStrokeCache {
    /// Strokes using TriangleStrip mode (each stroke is a separate cached mesh)
    strip_strokes: Vec<CachedStrokeMesh>,
    /// Strokes using Triangles mode (each stroke is a separate cached mesh)
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

/// UI state needed for rendering a frame
pub struct UiRenderState<'a> {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub custom_colors: &'a [[u8; 3]],
    pub selected_custom_index: i32,
    pub brush_size: f32,
    pub opacity: f32,
    pub is_eraser: bool,
    pub brush_type: BrushType,
    pub selection_tool_active: bool,
    pub selection_rect: Option<crate::canvas::Rect>,
    pub selection_time: f32,
    pub selected_strokes: Option<&'a [crate::canvas::Stroke]>,
}

/// Viewport state for canvas transform
pub struct ViewportState {
    pub zoom: f32,
    pub pan_offset: (f32, f32),
}

/// All data needed to render a single frame.
///
/// This is the single source of truth for render data.
/// The `Renderer` holds no application state — it only owns WGPU internals.
pub struct RenderFrame<'a> {
    pub canvas: &'a Canvas,
    pub active_stroke: Option<&'a ActiveStroke>,
    pub ui: UiRenderState<'a>,
    pub viewport: ViewportState,
}

/// UI cache key for WGPU renderer
#[derive(Clone, Debug, PartialEq)]
struct UiCacheKey {
    hue: f32,
    saturation: f32,
    value: f32,
    selected_custom_index: i32,
    brush_size: f32,
    opacity: f32,
    is_eraser: bool,
    brush_type: BrushType,
    selection_tool_active: bool,
    can_undo: bool,
    can_redo: bool,
    active_layer: usize,
    layer_count: usize,
}

/// WGPU-based renderer for the canvas
pub struct Renderer {
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
    /// WGPU render pipeline for strokes (TriangleStrip)
    render_pipeline: Option<wgpu::RenderPipeline>,
    /// WGPU render pipeline for spray/other triangle-list strokes
    spray_render_pipeline: Option<wgpu::RenderPipeline>,
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
    /// Multisampled texture for MSAA (only when sample_count > 1)
    msaa_texture: Option<wgpu::Texture>,
    /// Actual sample count in use
    sample_count: u32,
    /// Committed stroke cache per layer (index = layer index)
    layer_stroke_cache: Vec<LayerStrokeCache>,
    /// GPU buffer for committed stroke vertices (TriangleStrip)
    committed_strip_buffer: RefCell<Option<wgpu::Buffer>>,
    /// GPU buffer for committed stroke vertices (TriangleList)
    committed_tri_buffer: RefCell<Option<wgpu::Buffer>>,
    /// Canvas version when cache was last populated
    canvas_version_cached: u64,
    /// Cached UI vertices (reused when UI state unchanged)
    ui_vertex_cache: Vec<[f32; 6]>,
    /// UI state key for cache validation
    ui_cache_key: Option<UiCacheKey>,
    /// GPU texture for raster layers
    #[allow(dead_code)]
    raster_texture_cache: Vec<Option<wgpu::Texture>>,
    /// Bind group for raster layers
    #[allow(dead_code)]
    raster_bind_group_cache: Vec<Option<wgpu::BindGroup>>,
    /// Sampler for raster textures
    #[allow(dead_code)]
    raster_sampler: Option<wgpu::Sampler>,
    /// Raster render pipeline
    #[allow(dead_code)]
    raster_pipeline: Option<wgpu::RenderPipeline>,
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

        match Self::init_wgpu(&window, window_size, &config).await {
            Ok((
                device,
                queue,
                surface,
                surface_config,
                render_pipeline,
                spray_render_pipeline,
                ui_pipeline,
                pipeline_layout,
                shader,
                msaa_texture,
                sample_count,
            )) => {
                logger::info("✅ WGPU initialized successfully!");
                logger::info("   - Backend: GPU (WGPU)");
                logger::info(&format!("   - Device: {:?}", device));
                logger::info(&format!("   - Surface format: {:?}", surface_config.format));
                Ok(Self {
                    config,
                    backend: RenderBackend::Wgpu,
                    device: Some(device),
                    queue: Some(queue),
                    surface: Some(surface),
                    surface_config: Some(surface_config),
                    render_pipeline: Some(render_pipeline),
                    spray_render_pipeline: Some(spray_render_pipeline),
                    ui_pipeline: Some(ui_pipeline),
                    window_size,
                    pipeline_layout: Some(pipeline_layout),
                    shader: Some(shader),
                    window: Some(window),
                    msaa_texture,
                    sample_count,
                    layer_stroke_cache: Vec::new(),
                    committed_strip_buffer: RefCell::new(None),
                    committed_tri_buffer: RefCell::new(None),
                    canvas_version_cached: 0,
                    ui_vertex_cache: Vec::new(),
                    ui_cache_key: None,
                    raster_texture_cache: Vec::new(),
                    raster_bind_group_cache: Vec::new(),
                    raster_sampler: None,
                    raster_pipeline: None,
                })
            }
            Err(e) => {
                logger::error(&format!("WGPU initialization failed: {}", e));
                #[cfg(target_os = "linux")]
                {
                    logger::warn("Falling back to Cairo software rendering (Linux)");
                    logger::info("   - Backend: Cairo (CPU)");
                    Ok(Self {
                        config,
                        backend: RenderBackend::Cairo,
                        device: None,
                        queue: None,
                        surface: None,
                        surface_config: None,
                        render_pipeline: None,
                        spray_render_pipeline: None,
                        ui_pipeline: None,
                        window_size,
                        pipeline_layout: None,
                        shader: None,
                        window: Some(window),
                        msaa_texture: None,
                        sample_count: 1,
                        layer_stroke_cache: Vec::new(),
                        committed_strip_buffer: RefCell::new(None),
                        committed_tri_buffer: RefCell::new(None),
                        canvas_version_cached: 0,
                        ui_vertex_cache: Vec::new(),
                        ui_cache_key: None,
                        raster_texture_cache: Vec::new(),
                        raster_bind_group_cache: Vec::new(),
                        raster_sampler: None,
                        raster_pipeline: None,
                    })
                }
                #[cfg(target_os = "windows")]
                {
                    logger::error("No software fallback available on Windows");
                    Err(format!("WGPU initialization failed: {}. No fallback renderer is available on Windows.", e).into())
                }
                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                {
                    logger::error("No software fallback available on this platform");
                    return Err(format!("WGPU initialization failed: {}. No fallback renderer is available on this platform.", e).into());
                }
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
    ) -> (
        wgpu::RenderPipeline,
        wgpu::RenderPipeline,
        wgpu::RenderPipeline,
    ) {
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
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

        let spray_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Spray Render Pipeline"),
                layout: Some(pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
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

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
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

        (render_pipeline, spray_render_pipeline, ui_pipeline)
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
            wgpu::RenderPipeline,
            wgpu::RenderPipeline,
            wgpu::PipelineLayout,
            wgpu::ShaderModule,
            Option<wgpu::Texture>,
            u32,
        ),
        Box<dyn std::error::Error>,
    > {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // SAFETY: We transmute the surface to 'static lifetime because:
        // 1. The Renderer stores an Arc<Window> (line 102) that keeps the window alive
        // 2. The surface is dropped before the Arc<Window> in the Renderer's Drop impl
        // 3. This is the documented workaround for wgpu's surface lifetime requirements
        // See: https://github.com/gfx-rs/wgpu/issues/3123
        #[allow(clippy::missing_transmute_annotations)]
        let surface = unsafe {
            std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'static>>(
                instance.create_surface(window)?,
            )
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        logger::info(&format!("Selected adapter: {:?}", adapter.get_info()));

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

        let device_limits = device.limits();
        let max_texture_size = device_limits.max_texture_dimension_2d;
        logger::info(&format!("Max texture dimension: {}", max_texture_size));

        let surface_width = window_size.0.max(1).min(max_texture_size);
        let surface_height = window_size.1.max(1).min(max_texture_size);
        if surface_width != window_size.0 || surface_height != window_size.1 {
            logger::warn(&format!(
                "Window size {}x{} adjusted to {}x{} (GPU limits / non-zero requirement)",
                window_size.0, window_size.1, surface_width, surface_height
            ));
        }

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
        });

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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let sample_count = config.msaa_samples;
        logger::info(&format!(
            "Using MSAA sample count: {}{}",
            sample_count,
            if sample_count > 1 {
                " (with resolve target)"
            } else {
                " (swapchain rendering)"
            }
        ));

        let (render_pipeline, spray_render_pipeline, ui_pipeline) = Self::create_pipelines(
            &device,
            &shader,
            &pipeline_layout,
            surface_format,
            sample_count,
        );

        let msaa_texture = if sample_count > 1 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Texture"),
                size: wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            logger::info("Created MSAA resolve texture");
            Some(texture)
        } else {
            None
        };

        Ok((
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            spray_render_pipeline,
            ui_pipeline,
            pipeline_layout,
            shader,
            msaa_texture,
            sample_count,
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

            if let (Some(device), Some(pipeline_layout), Some(shader)) =
                (&self.device, &self.pipeline_layout, &self.shader)
            {
                let (render_pipeline, spray_render_pipeline, ui_pipeline) = Self::create_pipelines(
                    device,
                    shader,
                    pipeline_layout,
                    surface_format,
                    self.sample_count,
                );
                self.render_pipeline = Some(render_pipeline);
                self.spray_render_pipeline = Some(spray_render_pipeline);
                self.ui_pipeline = Some(ui_pipeline);
            }

            if self.sample_count > 1 {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("MSAA Texture"),
                    size: wgpu::Extent3d {
                        width: surface_width,
                        height: surface_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                self.msaa_texture = Some(texture);
            }
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

                // Only process vector layers
                let strokes = match &layer.content {
                    LayerContent::Vector(s) => s,
                    LayerContent::Raster(_) => continue,
                };

                for stroke in strokes {
                    if stroke.points.len() >= 2 {
                        let mesh = stroke_to_mesh_7(stroke, layer.opacity);
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

    /// Create a GPU texture from raster image data (placeholder - needs full implementation)
    #[allow(dead_code)]
    fn create_raster_texture(
        &self,
        device: &wgpu::Device,
        _image: &crate::canvas::RasterImage,
    ) -> Option<wgpu::Texture> {
        // Placeholder - creates empty texture
        // Full implementation would use queue.write_texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Raster Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Some(texture)
    }

    /// Ensure GPU buffers are created and sized for the given vertex counts.
    /// Reuses existing buffers if large enough, recreates if needed.
    fn ensure_gpu_buffers(&self, device: &wgpu::Device, strip_count: usize, tri_count: usize) {
        let strip_bytes = strip_count * std::mem::size_of::<[f32; 6]>();
        let tri_bytes = tri_count * std::mem::size_of::<[f32; 6]>();

        // Strip buffer
        {
            let mut strip_buf = self.committed_strip_buffer.borrow_mut();
            if !matches!(&*strip_buf, Some(buf) if buf.size() >= strip_bytes as u64) {
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Committed Stroke Buffer (TriangleStrip)"),
                    size: strip_bytes.max(1) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                *strip_buf = Some(buf);
            }
        }

        // Tri buffer
        {
            let mut tri_buf = self.committed_tri_buffer.borrow_mut();
            if !matches!(&*tri_buf, Some(buf) if buf.size() >= tri_bytes as u64) {
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Committed Stroke Buffer (TriangleList)"),
                    size: tri_bytes.max(1) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                *tri_buf = Some(buf);
            }
        }
    }

    /// Render the current frame using data from `frame`
    pub fn render(&mut self, frame: &RenderFrame) -> Result<(), wgpu::SurfaceError> {
        let _timer = logger::Timer::new("Full render frame");
        match self.backend {
            RenderBackend::Wgpu => {
                logger::debug("[RENDER] Using WGPU backend (GPU-accelerated)");
                self.render_wgpu(frame)
            }
            RenderBackend::Cairo => {
                logger::debug("[RENDER] Using Cairo backend (CPU software rendering)");
                Ok(())
            }
        }
    }

    /// Render using WGPU
    fn render_wgpu(&mut self, frame: &RenderFrame) -> Result<(), wgpu::SurfaceError> {
        use wgpu::util::DeviceExt;

        let _cache_timer = logger::Timer::new("Stroke cache update");

        // Ensure committed stroke cache is valid first (before borrowing self for GPU resources)
        self.ensure_cache_valid(frame.canvas);

        // Collect committed stroke vertices (cached) + active stroke
        let mut strip_vertices: Vec<[f32; 6]> = Vec::new();
        let mut strip_ranges: Vec<std::ops::Range<u32>> = Vec::new();
        let mut tri_vertices: Vec<[f32; 6]> = Vec::new();
        let mut tri_ranges: Vec<std::ops::Range<u32>> = Vec::new();

        let active_layer_idx = frame.canvas.active_layer();
        let layers = frame.canvas.layers();

        // First pass: collect vector strokes and prepare raster layer data
        let mut raster_layers_data: Vec<(usize, &crate::canvas::RasterLayer)> = Vec::new();

        for (layer_idx, layer) in layers.iter().enumerate().rev() {
            if !layer.visible {
                continue;
            }

            // Collect raster layers for separate rendering
            if let LayerContent::Raster(raster) = &layer.content {
                raster_layers_data.push((layer_idx, raster));
                continue;
            }

            // Add cached committed strokes for this layer (each stroke gets its own range)
            if layer_idx < self.layer_stroke_cache.len() {
                let cache = &self.layer_stroke_cache[layer_idx];

                // TriangleStrip strokes - each stroke gets its own range
                for stroke_vertices in &cache.strip_strokes {
                    let start = strip_vertices.len() as u32;
                    strip_vertices.extend_from_slice(stroke_vertices);
                    let end = strip_vertices.len() as u32;
                    strip_ranges.push(start..end);
                }

                // Triangles strokes - each stroke gets its own range
                for stroke_vertices in &cache.tri_strokes {
                    let start = tri_vertices.len() as u32;
                    tri_vertices.extend_from_slice(stroke_vertices);
                    let end = tri_vertices.len() as u32;
                    tri_ranges.push(start..end);
                }
            }

            // Insert active stroke at the active layer position
            if layer_idx == active_layer_idx
                && let Some(active_stroke) = frame.active_stroke
                && active_stroke.points().len() >= 2
            {
                let mesh = active_stroke_to_mesh_7(active_stroke, layer.opacity);
                collect_mesh(
                    &mesh,
                    &mut strip_vertices,
                    &mut strip_ranges,
                    &mut tri_vertices,
                    &mut tri_ranges,
                );
            }
        }

        // Extract GPU resources from self
        let surface = match self.surface.as_ref() {
            Some(s) => s,
            None => return Err(wgpu::SurfaceError::Lost),
        };
        let device = match self.device.as_ref() {
            Some(d) => d,
            None => return Err(wgpu::SurfaceError::Lost),
        };
        let queue = match self.queue.as_ref() {
            Some(q) => q,
            None => return Err(wgpu::SurfaceError::Lost),
        };
        let pipeline = match self.render_pipeline.as_ref() {
            Some(p) => p,
            None => return Err(wgpu::SurfaceError::Lost),
        };

        // Prepare GPU buffers - needs device reference
        let strip_count = strip_vertices.len();
        let tri_count = tri_vertices.len();
        self.ensure_gpu_buffers(device, strip_count, tri_count);

        // Skip rendering to a zero-area surface (e.g. minimised window).
        if self.window_size.0 == 0 || self.window_size.1 == 0 {
            return Ok(());
        }

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

        let output = surface.get_current_texture()?;

        if output.suboptimal {
            logger::debug("Surface suboptimal, reconfiguring");
            if let Some(config) = &self.surface_config {
                // SurfaceTexture must be dropped before reconfiguring the surface.
                drop(output);
                surface.configure(device, config);
            }
            return Ok(());
        }

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_view = self
            .msaa_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));

        let texture_width = output.texture.width() as f32;
        let texture_height = output.texture.height() as f32;

        // WGSL alignment: vec2<f32> has align=8, so pan_offset sits at offset 16
        // (4 bytes of implicit padding after zoom at offset 8).
        // Layout: [canvas_w, canvas_h, zoom, _pad, pan_x, pan_y]
        let uniform_data = [
            texture_width,
            texture_height,
            frame.viewport.zoom,
            0.0, // padding — aligns pan_offset to offset 16
            frame.viewport.pan_offset.0,
            frame.viewport.pan_offset.1,
        ];
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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
            let (color_view, resolve_target) = if let Some(ref msaa_view) = msaa_view {
                (msaa_view, Some(&view))
            } else {
                (&view, None)
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target,
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

            // Draw triangle strip strokes
            if !strip_vertices.is_empty()
                && let Some(buf) = self.committed_strip_buffer.borrow().as_ref()
            {
                queue.write_buffer(buf, 0, bytemuck::cast_slice(&strip_vertices));
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, buf.slice(..));
                for range in &strip_ranges {
                    render_pass.draw(range.clone(), 0..1);
                }
            }

            // Draw triangle list strokes (spray, etc.)
            if !tri_vertices.is_empty()
                && let Some(spray_pipeline) = &self.spray_render_pipeline
                && let Some(buf) = self.committed_tri_buffer.borrow().as_ref()
            {
                queue.write_buffer(buf, 0, bytemuck::cast_slice(&tri_vertices));
                render_pass.set_pipeline(spray_pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, buf.slice(..));
                for range in &tri_ranges {
                    render_pass.draw(range.clone(), 0..1);
                }
            }

            // Render selected strokes (overlay) - use same transform as canvas
            if let Some(selected_strokes) = frame.ui.selected_strokes {
                let mut selected_strip_vertices: Vec<[f32; 6]> = Vec::new();
                let mut selected_strip_ranges: Vec<std::ops::Range<u32>> = Vec::new();
                let mut selected_tri_vertices: Vec<[f32; 6]> = Vec::new();
                let mut selected_tri_ranges: Vec<std::ops::Range<u32>> = Vec::new();

                for stroke in selected_strokes {
                    if stroke.points.len() < 2 {
                        continue;
                    }
                    let mesh = geometry::generate_stroke_vertices_with_opacity(stroke, 1.0);
                    collect_mesh(
                        &mesh,
                        &mut selected_strip_vertices,
                        &mut selected_strip_ranges,
                        &mut selected_tri_vertices,
                        &mut selected_tri_ranges,
                    );
                }

                if !selected_strip_vertices.is_empty() {
                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Selected Stroke Vertex Buffer (TriangleStrip)"),
                            contents: bytemuck::cast_slice(&selected_strip_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    for range in &selected_strip_ranges {
                        render_pass.draw(range.clone(), 0..1);
                    }
                }

                if !selected_tri_vertices.is_empty()
                    && let Some(spray_pipeline) = &self.spray_render_pipeline
                {
                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Selected Stroke Vertex Buffer (TriangleList)"),
                            contents: bytemuck::cast_slice(&selected_tri_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(spray_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    for range in &selected_tri_ranges {
                        render_pass.draw(range.clone(), 0..1);
                    }
                }
            }

            // Reset zoom to 1.0 and pan to (0,0) for UI (UI stays fixed on screen)
            let ui_uniform_data = [uniform_data[0], uniform_data[1], 1.0, 0.0, 0.0, 0.0];
            let ui_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Uniform Buffer"),
                contents: bytemuck::cast_slice(&ui_uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let ui_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("UI Uniform Bind Group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ui_uniform_buffer.as_entire_binding(),
                }],
            });

            if let Some(ui_pipeline) = &self.ui_pipeline {
                // Build UI cache key (exclude selection_rect which animates every frame)
                let cache_key = UiCacheKey {
                    hue: frame.ui.hue,
                    saturation: frame.ui.saturation,
                    value: frame.ui.value,
                    selected_custom_index: frame.ui.selected_custom_index,
                    brush_size: frame.ui.brush_size,
                    opacity: frame.ui.opacity,
                    is_eraser: frame.ui.is_eraser,
                    brush_type: frame.ui.brush_type,
                    selection_tool_active: frame.ui.selection_tool_active,
                    can_undo: frame.canvas.can_undo(),
                    can_redo: frame.canvas.can_redo(),
                    active_layer: frame.canvas.active_layer(),
                    layer_count: frame.canvas.layer_count(),
                };

                // Regenerate UI only if state changed (excluding selection_rect which animates)
                if Some(&cache_key) != self.ui_cache_key.as_ref() {
                    let mut new_cache: Vec<[f32; 6]> = Vec::new();

                    new_cache.extend(flat_to_vertices(&geometry::generate_hsv_sliders(
                        frame.ui.hue,
                        frame.ui.saturation,
                        frame.ui.value,
                    )));
                    new_cache.extend(flat_to_vertices(&geometry::generate_custom_palette(
                        frame.ui.custom_colors,
                        frame.ui.selected_custom_index as usize,
                    )));
                    new_cache.extend(flat_to_vertices(&geometry::generate_brush_size_vertices(
                        frame.ui.brush_size,
                    )));
                    new_cache.extend(flat_to_vertices(
                        &geometry::generate_eraser_button_vertices(frame.ui.is_eraser),
                    ));
                    new_cache.extend(flat_to_vertices(&geometry::generate_clear_button_vertices()));
                    new_cache.extend(flat_to_vertices(&geometry::generate_undo_button_vertices(
                        frame.canvas.can_undo(),
                    )));
                    new_cache.extend(flat_to_vertices(&geometry::generate_redo_button_vertices(
                        frame.canvas.can_redo(),
                    )));
                    new_cache.extend(flat_to_vertices(
                        &geometry::generate_export_button_vertices(),
                    ));
                    new_cache.extend(flat_to_vertices(
                        &geometry::generate_zoom_in_button_vertices(),
                    ));
                    new_cache.extend(flat_to_vertices(
                        &geometry::generate_zoom_out_button_vertices(),
                    ));
                    new_cache.extend(flat_to_vertices(
                        &geometry::generate_opacity_preset_vertices(frame.ui.opacity),
                    ));
                    new_cache.extend(flat_to_vertices(&geometry::generate_brush_type_vertices(
                        frame.ui.brush_type,
                    )));
                    new_cache.extend(flat_to_vertices(&geometry::generate_selection_tool_button(
                        frame.ui.selection_tool_active,
                    )));

                    let layers: Vec<(String, bool, f32, bool)> = frame
                        .canvas
                        .layers()
                        .iter()
                        .map(|l| (l.name.clone(), l.visible, l.opacity, l.locked))
                        .collect();
                    new_cache.extend(flat_to_vertices(&geometry::generate_layer_panel_vertices(
                        &layers,
                        frame.canvas.active_layer(),
                        self.window_size.0 as f32,
                    )));

                    self.ui_vertex_cache = new_cache;
                    self.ui_cache_key = Some(cache_key);
                }

                // Selection rect uses Triangles mode (dashes are independent quads)
                // This is NOT cached because the selection rect animates (marching ants)
                let mut selection_rect_vertices: Vec<[f32; 6]> = Vec::new();
                if let Some(rect) = frame.ui.selection_rect {
                    let flat =
                        geometry::generate_selection_rect_vertices(rect, frame.ui.selection_time);
                    selection_rect_vertices.extend(flat_to_vertices(&flat));
                }

                if !self.ui_vertex_cache.is_empty() {
                    let ui_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Combined UI Vertex Buffer"),
                            contents: bytemuck::cast_slice(&self.ui_vertex_cache),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(ui_pipeline);
                    render_pass.set_bind_group(0, &ui_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
                    render_pass.draw(0..self.ui_vertex_cache.len() as u32, 0..1);
                }

                if !selection_rect_vertices.is_empty()
                    && let Some(spray_pipeline) = &self.spray_render_pipeline
                {
                    let sr_vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Selection Rect Vertex Buffer"),
                            contents: bytemuck::cast_slice(&selection_rect_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    render_pass.set_pipeline(spray_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, sr_vertex_buffer.slice(..));
                    render_pass.draw(0..selection_rect_vertices.len() as u32, 0..1);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));

        if let Some(ref window) = self.window {
            window.pre_present_notify();
        }

        output.present();

        Ok(())
    }

    /// Get the current backend
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
                logger::info(&format!("MSAA samples: {}", self.sample_count));
            }
            RenderBackend::Cairo => {
                logger::info("Backend: CPU (Cairo)");
                logger::info("Note: Using software rendering fallback");
            }
        }
        logger::info(&format!("Window size: {:?}", self.window_size));
        logger::info(&format!(
            "MSAA samples (config): {}",
            self.config.msaa_samples
        ));
        logger::info("======================");
    }
}

/// Convert flat vertex data (6 floats/vertex) to WGPU format (6 floats/vertex)
fn flat_to_vertices(flat: &[f32]) -> Vec<[f32; 6]> {
    flat.chunks(6)
        .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]])
        .collect()
}

/// Convert a stroke mesh to WGPU vertex format with layer opacity,
/// returning the vertices and the draw mode.
fn mesh_to_vertices(mesh: &StrokeMesh) -> Vec<[f32; 6]> {
    mesh.vertices
        .chunks(6)
        .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]])
        .collect()
}

/// Convert stroke to WGPU mesh format with layer opacity
fn stroke_to_mesh_7(stroke: &Stroke, layer_opacity: f32) -> StrokeMesh {
    geometry::generate_stroke_vertices_with_opacity(stroke, layer_opacity)
}

/// Convert active stroke to WGPU mesh format with layer opacity
fn active_stroke_to_mesh_7(active: &ActiveStroke, layer_opacity: f32) -> StrokeMesh {
    geometry::generate_active_stroke_vertices_with_opacity(active, layer_opacity)
}

/// Collect a mesh into the appropriate buffer (TriangleStrip or TriangleList)
fn collect_mesh(
    mesh: &StrokeMesh,
    strip_vertices: &mut Vec<[f32; 6]>,
    strip_ranges: &mut Vec<std::ops::Range<u32>>,
    tri_vertices: &mut Vec<[f32; 6]>,
    tri_ranges: &mut Vec<std::ops::Range<u32>>,
) {
    let converted = mesh_to_vertices(mesh);
    let count = converted.len() as u32;
    match mesh.mode {
        DrawMode::TriangleStrip => {
            let start = strip_vertices.len() as u32;
            strip_vertices.extend(converted);
            strip_ranges.push(start..start + count);
        }
        DrawMode::Triangles => {
            let start = tri_vertices.len() as u32;
            tri_vertices.extend(converted);
            tri_ranges.push(start..start + count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::BrushType;

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

    #[test]
    fn test_combined_stroke_buffer_tracks_ranges() {
        use crate::canvas::{Canvas, Point, Stroke};

        let mut canvas = Canvas::new();
        let stroke1 = Stroke {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        let stroke2 = Stroke {
            points: vec![Point { x: 100.0, y: 100.0 }, Point { x: 110.0, y: 110.0 }],
            color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(stroke1, 0);
        canvas.add_stroke_to_layer(stroke2, 0);

        let mut strip_vertices: Vec<[f32; 6]> = Vec::new();
        let mut strip_ranges: Vec<std::ops::Range<u32>> = Vec::new();
        let mut tri_vertices: Vec<[f32; 6]> = Vec::new();
        let mut tri_ranges: Vec<std::ops::Range<u32>> = Vec::new();
        for (stroke, layer_opacity) in canvas.all_strokes() {
            if stroke.points.len() >= 2 {
                let mesh =
                    crate::geometry::generate_stroke_vertices_with_opacity(stroke, layer_opacity);
                let converted = mesh
                    .vertices
                    .chunks(6)
                    .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]])
                    .collect::<Vec<_>>();
                let count = converted.len() as u32;
                match mesh.mode {
                    DrawMode::TriangleStrip => {
                        let start = strip_vertices.len() as u32;
                        strip_vertices.extend(converted);
                        strip_ranges.push(start..start + count);
                    }
                    DrawMode::Triangles => {
                        let start = tri_vertices.len() as u32;
                        tri_vertices.extend(converted);
                        tri_ranges.push(start..start + count);
                    }
                }
            }
        }

        assert_eq!(strip_ranges.len(), 2);
        assert!(strip_ranges[0].end <= strip_ranges[1].start);
        let total_from_ranges: u32 = strip_ranges.iter().map(|r| r.end - r.start).sum();
        assert_eq!(total_from_ranges, strip_vertices.len() as u32);
    }

    #[test]
    fn test_combined_buffer_empty_canvas() {
        let canvas = Canvas::new();
        let mut strip_vertices: Vec<[f32; 6]> = Vec::new();
        let mut strip_ranges: Vec<std::ops::Range<u32>> = Vec::new();
        let mut tri_vertices: Vec<[f32; 6]> = Vec::new();
        let mut tri_ranges: Vec<std::ops::Range<u32>> = Vec::new();
        for (stroke, layer_opacity) in canvas.all_strokes() {
            if stroke.points.len() >= 2 {
                let mesh =
                    crate::geometry::generate_stroke_vertices_with_opacity(stroke, layer_opacity);
                let converted = mesh
                    .vertices
                    .chunks(6)
                    .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5]])
                    .collect::<Vec<_>>();
                let count = converted.len() as u32;
                match mesh.mode {
                    DrawMode::TriangleStrip => {
                        let start = strip_vertices.len() as u32;
                        strip_vertices.extend(converted);
                        strip_ranges.push(start..start + count);
                    }
                    DrawMode::Triangles => {
                        let start = tri_vertices.len() as u32;
                        tri_vertices.extend(converted);
                        tri_ranges.push(start..start + count);
                    }
                }
            }
        }
        assert!(strip_ranges.is_empty());
        assert!(strip_vertices.is_empty());
    }

    #[test]
    fn test_single_point_stroke_excluded() {
        use crate::canvas::{Canvas, Point, Stroke};

        let mut canvas = Canvas::new();
        let single_point = Stroke {
            points: vec![Point { x: 50.0, y: 50.0 }],
            color: Color::BLACK,
            width: 2.0,
            opacity: 1.0,
            brush_type: BrushType::default(),
        };
        canvas.add_stroke_to_layer(single_point, 0);

        let mut count = 0;
        for (stroke, _) in canvas.all_strokes() {
            if stroke.points.len() >= 2 {
                count += 1;
            }
        }
        assert_eq!(count, 0);
    }
}
