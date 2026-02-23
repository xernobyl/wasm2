//! Render targets: G-buffer (color, depth reversed-Z, velocity), TAA history, resolve, bloom mip chain.

pub const BLOOM_MIP_LEVELS: usize = 5;
pub const BLOOM_MIP_COUNT: usize = BLOOM_MIP_LEVELS + 1;

/// One set of G-buffer + TAA + bloom mip chain for a single viewport.
#[derive(Debug)]
pub struct GbufferSet {
    pub color: wgpu::Texture,
    pub depth: wgpu::Texture,
    pub velocity: wgpu::Texture,
    pub resolve: wgpu::Texture,
    /// Ping-pong: [0] and [1] for TAA history.
    pub history: [wgpu::Texture; 2],
    /// Bloom mip chain: [0..4] are downsample levels (1/2 .. 1/32), [5] is lens output (same size as [4]).
    pub bloom_mips: [wgpu::Texture; BLOOM_MIP_COUNT],
    pub width: u32,
    pub height: u32,
}

impl GbufferSet {
    pub fn color_view(&self) -> wgpu::TextureView {
        self.color.create_view(&Default::default())
    }
    pub fn depth_view(&self) -> wgpu::TextureView {
        self.depth.create_view(&Default::default())
    }
    pub fn velocity_view(&self) -> wgpu::TextureView {
        self.velocity.create_view(&Default::default())
    }
    pub fn resolve_view(&self) -> wgpu::TextureView {
        self.resolve.create_view(&Default::default())
    }
    pub fn history_view(&self, index: usize) -> wgpu::TextureView {
        self.history[index].create_view(&Default::default())
    }
    pub fn bloom_mip_view(&self, index: usize) -> wgpu::TextureView {
        self.bloom_mips[index].create_view(&Default::default())
    }

    /// Width/height of a bloom mip level (0-based, where 0 = half-res).
    pub fn bloom_mip_size(&self, index: usize) -> (u32, u32) {
        let shift = (index.min(BLOOM_MIP_LEVELS - 1) + 1) as u32;
        let w = (self.width >> shift).max(1);
        let h = (self.height >> shift).max(1);
        (w, h)
    }

    /// Create or recreate targets for the given size. Uses reversed-Z depth (GREATER, clear 0).
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_color"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let velocity = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_velocity"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let resolve = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("taa_resolve"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let history = [
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("taa_history_0"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("taa_history_1"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
        ];

        const BLOOM_MIP_LABELS: [&str; BLOOM_MIP_COUNT] = [
            "bloom_mip_0", "bloom_mip_1", "bloom_mip_2", "bloom_mip_3", "bloom_mip_4", "bloom_lens",
        ];
        let bloom_mips = std::array::from_fn(|i| {
            let (w, h) = {
                let shift = (i.min(BLOOM_MIP_LEVELS - 1) + 1) as u32;
                ((width >> shift).max(1), (height >> shift).max(1))
            };
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(BLOOM_MIP_LABELS[i]),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        });

        Self {
            color,
            depth,
            velocity,
            resolve,
            history,
            bloom_mips,
            width,
            height,
        }
    }
}
