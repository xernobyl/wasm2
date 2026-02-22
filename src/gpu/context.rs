//! WebGPU device, queue, surface, and pipelines. Async init for wasm (request_adapter / request_device).

use std::cell::RefCell;
use std::rc::Rc;

use crate::view::ViewState;
use wgpu::util::DeviceExt;

use super::targets::GbufferSet;
use super::warehouse::{WarehouseUniforms, FULLSCREEN_TRIANGLE};

const WAREHOUSE_WGSL: &str = include_str!("../wgsl/warehouse.wgsl");
const WAREHOUSE_GBUFFER_WGSL: &str = include_str!("../wgsl/warehouse_gbuffer.wgsl");
const TAA_WGSL: &str = include_str!("../wgsl/taa.wgsl");
const PRESENT_WGSL: &str = include_str!("../wgsl/present.wgsl");
const BRIGHTNESS_WGSL: &str = include_str!("../wgsl/brightness.wgsl");
const BLUR_WGSL: &str = include_str!("../wgsl/blur.wgsl");
const SCREEN_PARITY_WGSL: &str = include_str!("../wgsl/screen_parity.wgsl");

/// Device, queue, surface, and adapter (for resize config). Created once via [init_gpu].
pub struct GpuContext {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    /// Last configured surface size; reconfigure when app size changes.
    config_size: (u32, u32),
    pub surface_format: wgpu::TextureFormat,
    warehouse_pipeline: wgpu::RenderPipeline,
    warehouse_gbuffer_pipeline: wgpu::RenderPipeline,
    warehouse_bind_group: wgpu::BindGroup,
    warehouse_uniform_buffer: wgpu::Buffer,
    fullscreen_vertex_buffer: wgpu::Buffer,
    linear_sampler: wgpu::Sampler,
    taa_pipeline: wgpu::RenderPipeline,
    taa_bind_group_layout: wgpu::BindGroupLayout,
    present_pipeline: wgpu::RenderPipeline,
    present_bind_group_layout: wgpu::BindGroupLayout,
    brightness_pipeline: wgpu::RenderPipeline,
    brightness_bind_group_layout: wgpu::BindGroupLayout,
    blur_pipeline: wgpu::RenderPipeline,
    blur_bind_group_layout: wgpu::BindGroupLayout,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group_layout: wgpu::BindGroupLayout,
    /// TAA ping-pong: 0 or 1, flip each frame.
    taa_history_index: usize,
    /// Depth for main pass (warehouse + cubes) when not using gbuffer. Reversed Z (clear 0).
    main_depth: Option<(wgpu::Texture, wgpu::TextureView)>,
    main_depth_size: (u32, u32),
}

impl GpuContext {
    /// Ensures the surface is configured for the given size. Call on resize and before get_current_texture.
    pub fn configure_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.config_size == (width, height) {
            return;
        }
        self.config_size = (width, height);
        let config = self
            .surface
            .get_default_config(&self.adapter, width, height)
            .expect("surface not supported by adapter");
        self.surface.configure(&self.device, &config);

        if self.main_depth_size != (width, height) {
            self.main_depth_size = (width, height);
            let depth = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("main_depth"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = depth.create_view(&Default::default());
            self.main_depth = Some((depth, view));
        }
    }

    /// Returns the main-pass depth view if present (same size as current config). Use for depth when drawing cubes.
    pub fn main_depth_view(&self) -> Option<&wgpu::TextureView> {
        self.main_depth.as_ref().map(|(_, v)| v)
    }

    /// Clears the current swap chain texture to the given color and presents. Call each frame when no other passes run.
    pub fn render_clear(&mut self, width: u32, height: u32) {
        self.configure_surface(width, height);

        let Ok(frame) = self.surface.get_current_texture() else {
            return;
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                multiview_mask: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    /// Records the warehouse raymarch fullscreen pass into the current render pass.
    /// When `background_only` is true, shader outputs solid gray (no raymarch); use when warehouse is disabled so cubes draw on top.
    pub fn draw_warehouse(
        &self,
        pass: &mut wgpu::RenderPass,
        view: &ViewState,
        time_s: f32,
        fb_width: u32,
        fb_height: u32,
        background_only: bool,
    ) {
        let uniforms = WarehouseUniforms::from_view(view, time_s, fb_width, fb_height, background_only);
        self.queue
            .write_buffer(&self.warehouse_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        pass.set_pipeline(&self.warehouse_pipeline);
        pass.set_bind_group(0, &self.warehouse_bind_group, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Same as [draw_warehouse] but uses the G-buffer pipeline (2 color + depth). Use in a pass with color+velocity+depth attachments.
    pub fn draw_warehouse_gbuffer(
        &self,
        pass: &mut wgpu::RenderPass,
        view: &ViewState,
        time_s: f32,
        fb_width: u32,
        fb_height: u32,
    ) {
        let uniforms = WarehouseUniforms::from_view(view, time_s, fb_width, fb_height, false);
        self.queue
            .write_buffer(&self.warehouse_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        pass.set_pipeline(&self.warehouse_gbuffer_pipeline);
        pass.set_bind_group(0, &self.warehouse_bind_group, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Ensures a [GbufferSet] exists for the given size (for mono). Recreates on resize.
    /// Stores the result in `gbuffer_rc` so the caller can borrow it separately from `self`.
    pub fn ensure_gbuffer(&mut self, width: u32, height: u32, gbuffer_rc: &Rc<RefCell<Option<GbufferSet>>>) {
        if width == 0 || height == 0 {
            panic!("ensure_gbuffer called with zero size");
        }
        let need_new = gbuffer_rc
            .borrow()
            .as_ref()
            .map(|g| g.width != width || g.height != height)
            .unwrap_or(true);
        if need_new {
            *gbuffer_rc.borrow_mut() = Some(GbufferSet::new(&self.device, width, height));
        }
    }

    /// Returns current TAA history index and flips it for next frame.
    pub fn taa_history_index(&mut self) -> usize {
        let i = self.taa_history_index;
        self.taa_history_index = 1 - self.taa_history_index;
        i
    }

    /// Records TAA resolve pass: read color + velocity + history, write resolve + updated history. Ping-pong: history_read_index 0 or 1.
    pub fn run_taa_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        gbuffer: &GbufferSet,
        history_read_index: usize,
    ) {
        let history_write_index = 1 - history_read_index;
        let color_view = gbuffer.color_view();
        let velocity_view = gbuffer.velocity_view();
        let history_read_view = gbuffer.history_view(history_read_index);
        let resolve_view = gbuffer.resolve_view();
        let history_write_view = gbuffer.history_view(history_write_index);

        let taa_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("taa_bind_group"),
            layout: &self.taa_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&velocity_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&history_read_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                },
            ],
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("taa"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &resolve_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &history_write_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                multiview_mask: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.taa_pipeline);
            pass.set_bind_group(0, &taa_bind_group, &[]);
            pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
            pass.draw(0..3, 0..1);
        }
    }

    /// Records brightness pass: resolve -> bloom (half-res). viewport_w/h = bloom size.
    pub fn run_brightness_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resolve_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        viewport_w: u32,
        viewport_h: u32,
    ) {
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("brightness_bg"),
            layout: &self.brightness_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(resolve_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("brightness"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: bloom_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            multiview_mask: None,
            occlusion_query_set: None,
        });
        pass.set_viewport(0.0, 0.0, viewport_w as f32, viewport_h as f32, 0.0, 1.0);
        pass.set_pipeline(&self.brightness_pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Records blur pass: bloom -> blur (same size). viewport_w/h = bloom size.
    pub fn run_blur_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bloom_view: &wgpu::TextureView,
        blur_view: &wgpu::TextureView,
        viewport_w: u32,
        viewport_h: u32,
    ) {
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blur_bg"),
            layout: &self.blur_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(bloom_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blur"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: blur_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            multiview_mask: None,
            occlusion_query_set: None,
        });
        pass.set_viewport(0.0, 0.0, viewport_w as f32, viewport_h as f32, 0.0, 1.0);
        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Records screen pass: resolve + bloom -> swap chain. Tonemap, add bloom, vignette.
    pub fn run_screen_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resolve_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        swap_chain_view: &wgpu::TextureView,
    ) {
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen_bg"),
            layout: &self.screen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(resolve_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(bloom_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("screen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: swap_chain_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.02, b: 0.08, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            multiview_mask: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.screen_pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Records present pass: fullscreen sample resolve texture and draw to swap_chain_view.
    pub fn run_present_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resolve_view: &wgpu::TextureView,
        swap_chain_view: &wgpu::TextureView,
    ) {
        let present_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present_bind_group"),
            layout: &self.present_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(resolve_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("present"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: swap_chain_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.08,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            multiview_mask: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.present_pipeline);
        pass.set_bind_group(0, &present_bind_group, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }
}

/// Async init: request adapter, request device, create surface from canvas.
/// On wasm32 the canvas is used with [wgpu::SurfaceTarget::Canvas].
/// Returns None if adapter or device request fails (e.g. WebGPU not available).
/// On non-wasm32 returns None (no window/canvas integration yet).
pub async fn init_gpu(canvas: web_sys::HtmlCanvasElement) -> Option<GpuContext> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::BROWSER_WEBGPU,
        #[cfg(not(target_arch = "wasm32"))]
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    #[cfg(target_arch = "wasm32")]
    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
        .ok()?;

    #[cfg(not(target_arch = "wasm32"))]
    return None;

    #[cfg(target_arch = "wasm32")]
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok()?;

    #[cfg(target_arch = "wasm32")]
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })
        .await
        .ok()?;

    #[cfg(target_arch = "wasm32")]
    let surface_format = surface
        .get_default_config(&adapter, 1, 1)
        .map(|c| c.format)
        .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);
    #[cfg(target_arch = "wasm32")]
    let (warehouse_pipeline, warehouse_gbuffer_pipeline, warehouse_bind_group, warehouse_uniform_buffer, fullscreen_vertex_buffer) =
        create_warehouse_pipelines(&device, surface_format);

    #[cfg(target_arch = "wasm32")]
    log!("[GPU] Creating TAA and present pipelines...");
    #[cfg(target_arch = "wasm32")]
    let (taa_pipeline, taa_bind_group_layout, present_pipeline, present_bind_group_layout, linear_sampler) =
        create_taa_and_present_pipelines(&device, surface_format);

    #[cfg(target_arch = "wasm32")]
    log!("[GPU] Creating post pipelines (brightness, blur, screen)...");
    #[cfg(target_arch = "wasm32")]
    let (brightness_pipeline, brightness_bind_group_layout, blur_pipeline, blur_bind_group_layout, screen_pipeline, screen_bind_group_layout) =
        create_post_pipelines(&device, surface_format);
    #[cfg(target_arch = "wasm32")]
    log!("[GPU] All pipelines created.");

    #[cfg(target_arch = "wasm32")]
    Some(GpuContext {
        adapter,
        device,
        queue,
        surface,
        config_size: (0, 0),
        surface_format,
        warehouse_pipeline,
        warehouse_gbuffer_pipeline,
        warehouse_bind_group,
        warehouse_uniform_buffer,
        fullscreen_vertex_buffer,
        linear_sampler,
        taa_pipeline,
        taa_bind_group_layout,
        present_pipeline,
        present_bind_group_layout,
        brightness_pipeline,
        brightness_bind_group_layout,
        blur_pipeline,
        blur_bind_group_layout,
        screen_pipeline,
        screen_bind_group_layout,
        taa_history_index: 0,
        main_depth: None,
        main_depth_size: (0, 0),
    })
}

#[cfg(target_arch = "wasm32")]
fn create_warehouse_pipelines(
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::BindGroup,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    log!("[GPU] Creating warehouse shaders and pipelines...");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("warehouse"),
        source: wgpu::ShaderSource::Wgsl(WAREHOUSE_WGSL.into()),
    });
    let shader_gbuffer = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("warehouse_gbuffer"),
        source: wgpu::ShaderSource::Wgsl(WAREHOUSE_GBUFFER_WGSL.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("warehouse_bind_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("warehouse_layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });

    let uniform_buffer_size = (std::mem::size_of::<WarehouseUniforms>() + 255) & !255;
    let warehouse_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("warehouse_uniforms"),
        size: uniform_buffer_size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let warehouse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("warehouse_bind_group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: warehouse_uniform_buffer.as_entire_binding(),
        }],
    });

    let fullscreen_vertex_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen_triangle"),
            contents: bytemuck::cast_slice(&FULLSCREEN_TRIANGLE),
            usage: wgpu::BufferUsages::VERTEX,
        });

    let warehouse_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("warehouse"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::GreaterEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    let warehouse_gbuffer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("warehouse_gbuffer"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_gbuffer,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_gbuffer,
            entry_point: Some("fs"),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rg16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    log!("[GPU] Warehouse pipelines created.");
    (
        warehouse_pipeline,
        warehouse_gbuffer_pipeline,
        warehouse_bind_group,
        warehouse_uniform_buffer,
        fullscreen_vertex_buffer,
    )
}

#[cfg(target_arch = "wasm32")]
fn create_taa_and_present_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::Sampler,
) {
    let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("linear_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let taa_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("taa"),
        source: wgpu::ShaderSource::Wgsl(TAA_WGSL.into()),
    });
    let taa_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("taa_bind_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let taa_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("taa_layout"),
        bind_group_layouts: &[&taa_bind_group_layout],
        immediate_size: 0,
    });
    let taa_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("taa"),
        layout: Some(&taa_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &taa_shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &taa_shader,
            entry_point: Some("fs"),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("present"),
        source: wgpu::ShaderSource::Wgsl(PRESENT_WGSL.into()),
    });
    let present_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("present_bind_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let present_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("present_layout"),
        bind_group_layouts: &[&present_bind_group_layout],
        immediate_size: 0,
    });
    let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("present"),
        layout: Some(&present_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &present_shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &present_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    (
        taa_pipeline,
        taa_bind_group_layout,
        present_pipeline,
        present_bind_group_layout,
        linear_sampler,
    )
}

#[cfg(target_arch = "wasm32")]
fn create_post_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
) {
    let brightness_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("brightness"),
        source: wgpu::ShaderSource::Wgsl(BRIGHTNESS_WGSL.into()),
    });
    let brightness_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("brightness_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let brightness_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("brightness_pl"),
        bind_group_layouts: &[&brightness_layout],
        immediate_size: 0,
    });
    let brightness_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("brightness"),
        layout: Some(&brightness_pl),
        vertex: wgpu::VertexState {
            module: &brightness_shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &brightness_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blur"),
        source: wgpu::ShaderSource::Wgsl(BLUR_WGSL.into()),
    });
    let blur_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blur_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let blur_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blur_pl"),
        bind_group_layouts: &[&blur_layout],
        immediate_size: 0,
    });
    let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blur"),
        layout: Some(&blur_pl),
        vertex: wgpu::VertexState {
            module: &blur_shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &blur_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    let screen_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("screen_parity"),
        source: wgpu::ShaderSource::Wgsl(SCREEN_PARITY_WGSL.into()),
    });
    let screen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("screen_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let screen_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("screen_pl"),
        bind_group_layouts: &[&screen_layout],
        immediate_size: 0,
    });
    let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("screen_parity"),
        layout: Some(&screen_pl),
        vertex: wgpu::VertexState {
            module: &screen_shader,
            entry_point: Some("vs"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &screen_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    (
        brightness_pipeline,
        brightness_layout,
        blur_pipeline,
        blur_layout,
        screen_pipeline,
        screen_layout,
    )
}
