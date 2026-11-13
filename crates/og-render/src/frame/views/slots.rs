//! A view's attachments: its HDR colour, depth and depth prepass.

use super::*;

/// One view's GPU state, kept across frames while the view exists.
pub(crate) struct ViewSlot {
    pub frame_buffer: wgpu::Buffer,
    pub fx_buffer: wgpu::Buffer,
    /// The view's light clusters: the table and the references.
    pub light_buffers: [wgpu::Buffer; 3],
    pub clusters: crate::clusters::ClusterBuilder,
    pub resolve_buffer: wgpu::Buffer,
    /// Its camera for the particle billboards (`ParticleViewGpu`) and for
    /// the lines and points (`LineUniform`).
    pub particle_buffer: wgpu::Buffer,
    pub line_buffer: wgpu::Buffer,
    /// Its image's size for the sprite shader (`SpriteFxFrame`), and its
    /// 2D light map (size, view) in a frame with 2D lighting: the lit
    /// sprite layers multiply with the lights as its camera sees them.
    pub sprite_fx_buffer: wgpu::Buffer,
    pub light_map: Option<((u32, u32), wgpu::TextureView)>,
    /// The lights' directions beside it, the same size, when the scene
    /// has normal-mapped textures (light2d.wgsl `fs_dir`).
    pub light_dir: Option<wgpu::TextureView>,
    /// Its 2D fluids' field targets of its size (`metaballs.rs`), as many
    /// as the frame's: the kernels drawn through its camera, composited in
    /// its sprite order.
    pub fields: Vec<wgpu::TextureView>,
    pub(super) fields_size: (u32, u32),
    /// Its sprite glow in a frame with og.Glow (`views_glow.wgsl`).
    pub glow: Option<ViewGlow>,
    /// (size, samples, depth texture drawn): the attachments below.
    key: ((u32, u32), u32, bool),
    pub hdr: wgpu::TextureView,
    pub msaa: Option<wgpu::TextureView>,
    pub depth: wgpu::TextureView,
    /// For a target with a depth texture: the view's depth prepass.
    pub prepass: Option<DepthPrepass>,
}

/// A view's depth prepass attachments, single-sampled as the main
/// prepass's (its pipelines'): the linear view depth the resolve reads, and
/// the normals and outline ids those pipelines also write.
pub(crate) struct DepthPrepass {
    pub gbuffer: wgpu::TextureView,
    pub lindepth: wgpu::TextureView,
    pub outline: wgpu::TextureView,
    pub depth: wgpu::TextureView,
}

impl DepthPrepass {
    /// The attachments of a camera of `size` (`label`: whose they are).
    pub fn new(device: &wgpu::Device, (w, h): (u32, u32), label: &str) -> DepthPrepass {
        let make = |what: &str, format, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("{label} prepass {what}")),
                    size: wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let bind = wgpu::TextureUsages::TEXTURE_BINDING;
        DepthPrepass {
            gbuffer: make("normals", crate::post::GBUFFER_FORMAT, attach),
            lindepth: make("depth", crate::post::LINDEPTH_FORMAT, attach | bind),
            outline: make("outline", crate::post::OUTLINE_FORMAT, attach),
            depth: make("depth buffer", crate::DEPTH_FORMAT, attach),
        }
    }
}

impl Views {
    /// The slot of view `index`, its attachments of `size` (made anew when
    /// that changes).
    #[allow(clippy::too_many_arguments)]
    pub fn slot(
        &mut self,
        device: &wgpu::Device,
        index: usize,
        size: (u32, u32),
        samples: u32,
        depth_drawn: bool,
        frame_bytes: u64,
        fx_bytes: u64,
    ) -> &mut ViewSlot {
        if self.slots.len() <= index {
            self.slots.resize_with(index + 1, || None);
        }
        let key = (size, samples, depth_drawn);
        if self.slots[index].as_ref().is_some_and(|s| s.key != key) {
            let old = self.slots[index].take().expect("checked");
            self.slots[index] = Some(ViewSlot {
                key,
                ..attachments(device, old, key)
            });
        }
        self.slots[index].get_or_insert_with(|| {
            let buffer = |label, size| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            };
            let (hdr, msaa, depth, prepass) = textures(device, key);
            ViewSlot {
                frame_buffer: buffer("view frame", frame_bytes),
                fx_buffer: buffer("view fx", fx_bytes),
                light_buffers: crate::clusters::buffers(device),
                clusters: Default::default(),
                resolve_buffer: buffer(
                    "view resolve",
                    std::mem::size_of::<ResolveUniform>() as u64,
                ),
                particle_buffer: buffer(
                    "view particles",
                    std::mem::size_of::<crate::particles_gpu::ParticleViewGpu>() as u64,
                ),
                line_buffer: buffer(
                    "view lines",
                    std::mem::size_of::<crate::lines::LineUniform>() as u64,
                ),
                sprite_fx_buffer: buffer("view sprite fx", 32),
                light_map: None,
                light_dir: None,
                fields: Vec::new(),
                fields_size: (0, 0),
                glow: None,
                key,
                hdr,
                msaa,
                depth,
                prepass,
            }
        })
    }

    /// Lets the slots of views the scene no longer has go.
    pub fn retain(&mut self, views: usize) {
        self.slots.truncate(views);
    }
}

fn textures(
    device: &wgpu::Device,
    ((w, h), samples, depth_drawn): ((u32, u32), u32, bool),
) -> (
    wgpu::TextureView,
    Option<wgpu::TextureView>,
    wgpu::TextureView,
    Option<DepthPrepass>,
) {
    let make = |label, format, samples, usage| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
            .create_view(&Default::default())
    };
    let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let bind = wgpu::TextureUsages::TEXTURE_BINDING;
    let hdr = make("view hdr", crate::HDR_FORMAT, 1, attach | bind);
    let msaa = (samples > 1).then(|| make("view hdr (msaa)", crate::HDR_FORMAT, samples, attach));
    let depth = make("view depth", crate::DEPTH_FORMAT, samples, attach);
    let prepass = depth_drawn.then(|| DepthPrepass::new(device, (w, h), "view"));
    (hdr, msaa, depth, prepass)
}

/// A view's HDR target of `size` that a pass draws and its main pass reads
/// (the 2D light map, a metaball field).
pub(super) fn hdr_texture(
    device: &wgpu::Device,
    label: &str,
    (w, h): (u32, u32),
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

fn attachments(device: &wgpu::Device, old: ViewSlot, key: ((u32, u32), u32, bool)) -> ViewSlot {
    let (hdr, msaa, depth, prepass) = textures(device, key);
    ViewSlot {
        hdr,
        msaa,
        depth,
        prepass,
        ..old
    }
}
