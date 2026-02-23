//! WebGPU device, queue, surface, and pipelines. Async init for wasm (request_adapter / request_device).

use std::cell::RefCell;
use std::rc::Rc;

use crate::view::ViewState;
use wgpu::util::DeviceExt;

use super::targets::{GbufferSet, BLOOM_MIP_LEVELS};
use super::warehouse::{WarehouseUniforms, FULLSCREEN_TRIANGLE};

const WAREHOUSE_WGSL: &str = include_str!("../wgsl/warehouse.wgsl");
const WAREHOUSE_GBUFFER_WGSL: &str = include_str!("../wgsl/warehouse_gbuffer.wgsl");
const TAA_WGSL: &str = include_str!("../wgsl/taa.wgsl");
const PRESENT_WGSL: &str = include_str!("../wgsl/present.wgsl");
const BRIGHTNESS_WGSL: &str = include_str!("../wgsl/brightness.wgsl");
const BLUR_WGSL: &str = include_str!("../wgsl/blur.wgsl");
const BLUR_UPSAMPLE_WGSL: &str = include_str!("../wgsl/blur_upsample.wgsl");
const LENS_WGSL: &str = include_str!("../wgsl/lens.wgsl");
const SCREEN_PARITY_WGSL: &str = include_str!("../wgsl/screen_parity.wgsl");

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniforms {
    pub camera_dir: [f32; 3],
    pub _pad: f32,
}

/// Device, queue, surface, and adapter (for resize config). Created once via [init_gpu].
pub struct GpuContext {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
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
    /// Shared layout for all single-texture fullscreen passes (brightness, downsample, upsample, lens).
    bloom_tex_sampler_layout: wgpu::BindGroupLayout,
    brightness_pipeline: wgpu::RenderPipeline,
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,
    lens_pipeline: wgpu::RenderPipeline,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group_layout: wgpu::BindGroupLayout,
    screen_uniform_buffer: wgpu::Buffer,
    taa_history_index: usize,
    main_depth: Option<(wgpu::Texture, wgpu::TextureView)>,
    main_depth_size: (u32, u32),
}

impl GpuContext {
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

    pub fn main_depth_view(&self) -> Option<&wgpu::TextureView> {
        self.main_depth.as_ref().map(|(_, v)| v)
    }

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

    pub fn taa_history_index(&mut self) -> usize {
        let i = self.taa_history_index;
        self.taa_history_index = 1 - self.taa_history_index;
        i
    }

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

    /// Helper: runs a single-texture fullscreen pass (brightness, downsample, upsample, or lens).
    fn run_fullscreen_tex_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        label: &str,
        pipeline: &wgpu::RenderPipeline,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        viewport_w: u32,
        viewport_h: u32,
    ) {
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.bloom_tex_sampler_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(source_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bg, &[]);
        pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
        pass.draw(0..3, 0..1);
    }

    /// Runs the full bloom mip chain: brightness+downsample, downsample loop, lens, upsample loop.
    pub fn run_bloom_passes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resolve_view: &wgpu::TextureView,
        gbuffer: &GbufferSet,
    ) {
        let n_downsample = BLOOM_MIP_LEVELS - 1; // 4

        // 1. Brightness extraction + initial downsample: resolve -> mip[0]
        let mip0_view = gbuffer.bloom_mip_view(0);
        let (w, h) = gbuffer.bloom_mip_size(0);
        self.run_fullscreen_tex_pass(
            encoder, "bloom_brightness", &self.brightness_pipeline,
            resolve_view, &mip0_view, w, h,
        );

        // 2. Downsample chain: mip[i] -> mip[i+1]
        for i in 0..n_downsample {
            let src_view = gbuffer.bloom_mip_view(i);
            let dst_view = gbuffer.bloom_mip_view(i + 1);
            let (w, h) = gbuffer.bloom_mip_size(i + 1);
            self.run_fullscreen_tex_pass(
                encoder, "bloom_downsample", &self.downsample_pipeline,
                &src_view, &dst_view, w, h,
            );
        }

        // 3. Lens flare pass: mip[4] -> mip[5]
        let lens_src = gbuffer.bloom_mip_view(BLOOM_MIP_LEVELS - 1);
        let lens_dst = gbuffer.bloom_mip_view(BLOOM_MIP_LEVELS);
        let (w, h) = gbuffer.bloom_mip_size(BLOOM_MIP_LEVELS);
        self.run_fullscreen_tex_pass(
            encoder, "bloom_lens", &self.lens_pipeline,
            &lens_src, &lens_dst, w, h,
        );

        // 4. Upsample chain: mip[i+1] -> mip[i], first iteration reads from mip[5] (lens output)
        for j in 0..n_downsample {
            let i = n_downsample - 1 - j;
            let src_idx = if j == 0 { BLOOM_MIP_LEVELS } else { i + 1 };
            let src_view = gbuffer.bloom_mip_view(src_idx);
            let dst_view = gbuffer.bloom_mip_view(i);
            let (w, h) = gbuffer.bloom_mip_size(i);
            self.run_fullscreen_tex_pass(
                encoder, "bloom_upsample", &self.upsample_pipeline,
                &src_view, &dst_view, w, h,
            );
        }
    }

    /// Records screen pass: resolve + bloom mip[0] -> swap chain. ACES tonemap, sRGB, starburst, vignette.
    pub fn run_screen_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resolve_view: &wgpu::TextureView,
        bloom_view: &wgpu::TextureView,
        swap_chain_view: &wgpu::TextureView,
        camera_dir: [f32; 3],
    ) {
        let uniforms = ScreenUniforms { camera_dir, _pad: 0.0 };
        self.queue.write_buffer(&self.screen_uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen_bg"),
            layout: &self.screen_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(resolve_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(bloom_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.linear_sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: self.screen_uniform_buffer.as_entire_binding() },
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
    log!("[GPU] Creating post pipelines (brightness, downsample, upsample, lens, screen)...");
    #[cfg(target_arch = "wasm32")]
    let (
        bloom_tex_sampler_layout,
        brightness_pipeline,
        downsample_pipeline,
        upsample_pipeline,
        lens_pipeline,
        screen_pipeline,
        screen_bind_group_layout,
        screen_uniform_buffer,
    ) = create_post_pipelines(&device, surface_format);
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
        bloom_tex_sampler_layout,
        brightness_pipeline,
        downsample_pipeline,
        upsample_pipeline,
        lens_pipeline,
        screen_pipeline,
        screen_bind_group_layout,
        screen_uniform_buffer,
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

    let vb_layout = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        }],
    };

    let warehouse_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("warehouse"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[vb_layout.clone()],
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
            buffers: &[vb_layout],
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

    let vb_layout = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
    };

    let taa_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("taa"),
        layout: Some(&taa_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &taa_shader,
            entry_point: Some("vs"),
            buffers: &[vb_layout.clone()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &taa_shader,
            entry_point: Some("fs"),
            targets: &[
                Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba16Float, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba16Float, blend: None, write_mask: wgpu::ColorWrites::ALL }),
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
            buffers: &[vb_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &present_shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState { format: surface_format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        cache: None,
        multiview_mask: None,
    });

    (taa_pipeline, taa_bind_group_layout, present_pipeline, present_bind_group_layout, linear_sampler)
}

#[cfg(target_arch = "wasm32")]
fn create_post_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (
    wgpu::BindGroupLayout,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::Buffer,
) {
    let bloom_tex_sampler_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bloom_tex_sampler_layout"),
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
    let bloom_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bloom_pl"),
        bind_group_layouts: &[&bloom_tex_sampler_layout],
        immediate_size: 0,
    });

    let vb_layout = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
    };
    let bloom_target = [Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba16Float,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let make_bloom_pipeline = |label: &'static str, source: &str| -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&bloom_pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[vb_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &bloom_target,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        })
    };

    let brightness_pipeline = make_bloom_pipeline("brightness", BRIGHTNESS_WGSL);
    let downsample_pipeline = make_bloom_pipeline("downsample", BLUR_WGSL);
    let upsample_pipeline = make_bloom_pipeline("upsample", BLUR_UPSAMPLE_WGSL);
    let lens_pipeline = make_bloom_pipeline("lens", LENS_WGSL);

    // Screen pipeline: 2 textures + sampler + uniform buffer
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
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
            buffers: &[vb_layout],
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

    let screen_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screen_uniforms"),
        size: std::mem::size_of::<ScreenUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    (
        bloom_tex_sampler_layout,
        brightness_pipeline,
        downsample_pipeline,
        upsample_pipeline,
        lens_pipeline,
        screen_pipeline,
        screen_layout,
        screen_uniform_buffer,
    )
}
