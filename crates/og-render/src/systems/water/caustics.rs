use super::*;
use crate::fields::window;

/// W2: a sea's caustics grid (`shaders/water_caustics.wgsl`, caustics.wgsl
/// `wave_caustics`): [`CAUSTIC_RES`]² texels of [`CAUSTIC_TEXEL`] metres
/// about the camera, each the drawn surface's Hessian in one word.
/// (16 KB, the downlevel limit of a uniform binding the renderer asks for.)
pub(crate) const CAUSTIC_RES: u32 = 64;
pub(crate) const CAUSTIC_TEXEL: f64 = 0.4;

/// The grid's buffer, a uniform in the frame group (bound in every frame
/// group, written by [`WaterRenderer::prepare`]'s caustics pass).
pub(crate) fn caustic_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("water caustics"),
        size: u64::from(CAUSTIC_RES * CAUSTIC_RES) * 4,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

pub(crate) fn caustic_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 12,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// The caustics grid about the camera at `eye` (frame) in a scene whose
/// frame's zero lies at `origin`: its texel (0, 0)'s centre in the frame's
/// xz, snapped to whole texels of the absolute world (so the grid does not
/// swim as the camera moves), the texel, the texels along an edge.
pub(crate) fn caustic_grid(eye: Vec3, origin: Vec3) -> [f32; 4] {
    let origin = glam::DVec2::new(f64::from(origin.x), f64::from(origin.z));
    let camera = glam::DVec2::new(f64::from(eye.x), f64::from(eye.z)) + origin;
    let first = window::first_cell(camera, CAUSTIC_TEXEL, CAUSTIC_RES);
    let corner = window::corner_from(first, CAUSTIC_TEXEL, origin);
    [corner.x, corner.y, CAUSTIC_TEXEL as f32, CAUSTIC_RES as f32]
}

/// W2: the pass that writes a sea's curvature into the caustics grid
/// (`shaders/water_caustics.wgsl`).
pub(super) struct CausticsPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    /// The frame's pass, planned ([`CausticsPass::plan`]) and not yet
    /// recorded.
    pending: Option<wgpu::BindGroup>,
}

const CAUSTICS_WGSL: &str = wgsl3d!("water_caustics.wgsl");

/// This module's own WGSL, for the shader check.
pub fn caustics_source() -> (&'static str, String) {
    ("water caustics", CAUSTICS_WGSL.to_string())
}

impl CausticsPass {
    pub(super) fn new(device: &wgpu::Device) -> CausticsPass {
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water caustics"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                entry(
                    3,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("water caustics"),
            source: wgpu::ShaderSource::Wgsl(CAUSTICS_WGSL.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water caustics"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("water caustics"),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water caustics"),
            size: std::mem::size_of::<CausticsGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // The cascades tile.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water caustics"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        CausticsPass {
            pipeline,
            layout,
            uniform,
            sampler,
            pending: None,
        }
    }

    /// Plans the frame's pass over the sea's `fields` into the grid `out`:
    /// writes its uniforms, for [`CausticsPass::encode`] to record after
    /// the fields.
    pub(super) fn plan(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fields: &wgpu::TextureView,
        params: &CausticsGpu,
        out: &wgpu::Buffer,
    ) {
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(params));
        self.pending = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("water caustics"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(fields),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: out.as_entire_binding(),
                },
            ],
        }));
    }

    /// Records the pass [`CausticsPass::plan`] planned: nothing without one.
    pub(super) fn encode(&mut self, encoder: &mut crate::Encoder) {
        let Some(group) = self.pending.take() else {
            return;
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("water caustics"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(CAUSTIC_RES / 8, CAUSTIC_RES / 8, 1);
    }
}

/// The cells a lake's ripples repeat over (caustics.wgsl `ripple_light`:
/// whole cycles across 16 cells, times two per octave, of which the
/// longest is the fifth up).
const RIPPLE_TILE: f64 = 512.0;

/// The frame's caustics: the first surface that has them, as
/// `FrameUniform::caustics` (height, strength, scale, fade; then the xz
/// rectangle; then where the scene's `origin` lies in the ripples' tile,
/// in cells, so the pattern stays put when it moves: `origin.rs`; under a
/// sea its curvature grid and a 1 in the last row's w); zero without one.
/// Then the wet shore ([`shore_wet`]).
pub(crate) fn caustics(waters: &[Water], origin: Vec3, eye: Vec3) -> [[f32; 4]; 6] {
    let mut rows = [[0.0; 4]; 6];
    [rows[4], rows[5]] = shore_wet(waters);
    let Some(w) = waters.iter().find(|w| w.caustics > 0.0) else {
        return rows;
    };
    // W2: under a sea its waves' own, from the caustics grid.
    if ocean(w).is_some() {
        let (_, at) = plane(w);
        let h = w.size * 0.5;
        let c = w.transform.w_axis;
        let (lo, hi) = if w.endless {
            (Vec2::splat(-1e9), Vec2::splat(1e9))
        } else {
            (
                Vec2::new(c.x - h.x, c.z - h.y),
                Vec2::new(c.x + h.x, c.z + h.y),
            )
        };
        rows[..4].copy_from_slice(&[
            [at.y, w.caustics, 0.0, 0.05],
            [lo.x, lo.y, hi.x, hi.y],
            caustic_grid(eye, origin),
            [0.0, 0.0, 0.0, 1.0],
        ]);
        return rows;
    }
    let (_, at) = plane(w);
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for (x, z) in [(-0.5f32, -0.5f32), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
        let p = w
            .transform
            .transform_point3(Vec3::new(x * w.size.x, 0.0, z * w.size.y));
        lo = lo.min(Vec2::new(p.x, p.z));
        hi = hi.max(Vec2::new(p.x, p.z));
    }
    let scale = w.caustics_scale.max(1e-3);
    // caustics.wgsl `ripple_caustics`: the ripples repeat over their tile,
    // so the origin's place within it is all the pattern needs of it.
    let within = |o: f32| (f64::from(o) * f64::from(scale)).rem_euclid(RIPPLE_TILE) as f32;
    rows[..4].copy_from_slice(&[
        [at.y, w.caustics, scale, 0.15],
        [lo.x, lo.y, hi.x, hi.y],
        [within(origin.x), within(origin.z), 0.0, 0.0],
        [0.0; 4],
    ]);
    rows
}

/// How much a wet shore is darkened at full wetness (weather_surface.wgsl:
/// a porous surface's albedo times 1 - 0.7 of this; wet sand is about half
/// as bright as dry).
const SHORE_WET: f32 = 0.7;

/// The shore the first sea with a seabed wets (W1, weather_surface.wgsl
/// `shore_wetness`): its rest level, the band its surf runs up (half the
/// sea's significant height times the trains' strength, the run-up of a
/// gentle beach: Stockdon et al. 2006's R2 ≈ 0.4–0.6 Hs), how wet; and the
/// seabed's xz rectangle in the frame. Zero without one.
fn shore_wet(waters: &[Water]) -> [[f32; 4]; 2] {
    let Some((w, s)) = waters
        .iter()
        .filter(|w| w.mesh.is_none())
        .find_map(|w| Some((w, w.shore.as_ref()?)))
    else {
        return [[0.0; 4]; 2];
    };
    let (_, at) = plane(w);
    // The trains' amplitude is a quarter of Hs times their strength.
    let band = (2.0 * s.trains.amplitude).max(0.15);
    let hi = s.origin
        + Vec2::new(
            (s.field.columns.max(1) - 1) as f32,
            (s.field.rows.max(1) - 1) as f32,
        ) * s.field.cell;
    [
        [at.y, band, SHORE_WET * s.foam.clamp(0.0, 1.0).max(0.5), 1.0],
        [s.origin.x, s.origin.y, hi.x, hi.y],
    ]
}
