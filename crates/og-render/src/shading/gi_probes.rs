//! Probe global illumination (`docs/design.md` §2.6): dynamic diffuse
//! global illumination (Majercik et al., "Dynamic Diffuse Global
//! Illumination with Ray-Traced Irradiance Fields", JCGT 2019) in cascades
//! that scroll ahead of the camera, traced in software against a simplified scene
//! (`gi_scene.rs`), the path Lumen takes where there is no hardware ray
//! tracing. A port of Wicked Engine's DDGI (MIT,
//! `third_party/wicked-ddgi`); NVIDIA's RTXGI is under the NVIDIA RTX SDK
//! licence, not ported.
//!
//! Each frame, before the lit passes ([`crate::post`]'s `gi-probes`):
//!
//! 1. **Trace** (`gi_trace.wgsl`): each probe of this frame's slice casts
//!    [`ProbeGlobalIllumination::rays`] rays (a spherical Fibonacci set
//!    turned at random each frame) through the scene's two-level BVH.
//!    A ray leaving the scene takes the sky; a hit takes the sun and one
//!    local light behind shadow rays, the probes' own light there (so each
//!    frame adds a bounce) and the surface's emission.
//! 2. **Blend** (`gi_blend.wgsl`): each probe gathers its rays into a 6 × 6
//!    octahedral radiance map filtered in time by the multiscale mean
//!    estimator, projected onto first-order harmonics (the light volumes'
//!    `[c1, c0]` irradiance over π, deringed), and a 12 × 12 map of the
//!    distance moments its visibility test reads; it moves off the
//!    surfaces nearest it, and one inside geometry is marked so the lit
//!    passes leave it out.
//!
//! The probes live in the light volumes' buffer (frame binding 10) after
//! [`crate::lightmap::PROBE_CAPACITY`], described by header rows after the
//! volumes' (`gi_common.wgsl`), so the lit pipelines gain no binding (their
//! sixteen texture slots are full). `ambient_light` (frame.wgsl) blends
//! them over the environment for every lit surface that reads it: meshes,
//! terrain, grass, scattered copies, particles' lit look. Baked lightmaps,
//! baked light volumes and realtime light volumes stay over them where a
//! scene has them.
//!
//! The cascades are [`COUNTS`] probes each, the first
//! [`ProbeGlobalIllumination::spacing`] apart, each next one twice as far
//! apart, centred [`AHEAD`] of their extent ahead of the camera: on the
//! world's cells of their spacing, so a camera moving or turning through
//! a cell's width moves the grid by one plane of probes and the rest keep
//! what they hold. Cascade `c` updates a `2^c`-th of its probes a
//! frame (all of them on the first frame).

use crate::gi_scene::{GiScene, Placed, Source};
#[cfg(test)]
use glam::IVec2;
use glam::{IVec3, Mat3, Vec3};

/// Probe GI ([`crate::PostEffects::gi_probes`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbeGlobalIllumination {
    /// The finest cascade's distance between probes, in metres (each next
    /// cascade's is twice its).
    pub spacing: f32,
    /// Cascades, 1 to [`CASCADES`]: each doubles the reach.
    pub cascades: u32,
    /// Rays a probe casts each time it is updated, 32 to 256.
    pub rays: u32,
    /// Scales the light surfaces bounce (1 physical); the sky's light and
    /// emission are not scaled.
    pub intensity: f32,
}

impl Default for ProbeGlobalIllumination {
    fn default() -> Self {
        ProbeGlobalIllumination {
            spacing: 1.0,
            cascades: 3,
            rays: 64,
            intensity: 1.0,
        }
    }
}

/// The most cascades.
pub const CASCADES: usize = 4;
/// How far ahead of the camera a cascade's centre lies, as a share of its
/// extent along each axis ([`plan`]): looking along an axis, five of its
/// sixteen cells, so the camera sits three to four cells in from the face
/// behind it, clear of the fade over a cascade's outer two cells, and the
/// cascade reaches nine cells ahead at full weight.
const AHEAD: f32 = 0.3125;
/// Each cascade's probes along x, y and z.
pub const COUNTS: [u32; 3] = [16, 8, 16];
/// The header rows after the light volumes' (`gi_common.wgsl`).
pub(crate) const HEADER_ROWS: usize = 1 + 2 * CASCADES;
/// Where those rows start (`GI_HEADER_AT`).
const HEADER_AT: usize = 2 * crate::lightmap::PROBE_VOLUMES;
/// A probe's vec4 (`GI_STRIDE`): its state, harmonics and distance map.
pub(crate) const STRIDE: usize = 76;
/// The distance map's texels a side (`GI_DEPTH`).
pub(crate) const DEPTH: usize = 12;
/// The radiance map's texels (`GI_RADIANCE`²) and the estimator's vec4 a
/// texel (`gi_blend.wgsl`).
const RADIANCE_TEXELS: usize = 36;
const ESTIMATOR_WORDS: usize = 3;
/// The bias of the probes' visibility test, in spacings: along the normal
/// and towards the eye (RTXGI's DDGIGetSurfaceBias, its defaults scaled
/// as its documentation recommends).
const NORMAL_BIAS: f32 = 0.1;
const VIEW_BIAS: f32 = 0.3;
/// The radiance's short window blend (Wicked's blendSpeed) and the
/// distance maps' blend a frame.
const SHORT_BLEND: f32 = 0.08;
const DEPTH_BLEND: f32 = 0.05;
/// The parameters' stride: one per slice, at dynamic offsets.
const PARAMS_STRIDE: u64 = 256;
/// The slices a frame can hold (every cascade at once on a reset).
const MAX_SLICES: usize = CASCADES;

const _: () = assert!(STRIDE == 1 + 3 + DEPTH * DEPTH / 2);
const _: () = assert!(HEADER_AT == 16, "GI_HEADER_AT in gi_common.wgsl");

/// The probes of one cascade.
const fn probes_per_cascade() -> usize {
    (COUNTS[0] * COUNTS[1] * COUNTS[2]) as usize
}

/// `gi_oct_wrap` (gi_common.wgsl): the texel of an `n` × `n` octahedral
/// map that a tap one past an edge reads.
#[cfg(test)]
fn oct_wrap(p: IVec2, n: i32) -> IVec2 {
    let c = p.clamp(IVec2::ZERO, IVec2::splat(n - 1));
    let (out_x, out_y) = (p.x < 0 || p.x >= n, p.y < 0 || p.y >= n);
    match (out_x, out_y) {
        (true, true) => IVec2::splat(n - 1) - c,
        (true, false) => IVec2::new(c.x, n - 1 - c.y),
        (false, true) => IVec2::new(n - 1 - c.x, c.y),
        (false, false) => p,
    }
}

/// `gi_slot` and `gi_key` (gi_common.wgsl): where world cell `w` is
/// stored in a grid of `counts`, and the key saying which cell it is.
#[cfg(test)]
fn slot_and_key(w: IVec3, counts: IVec3) -> (IVec3, f32) {
    let s = ((w % counts) + counts) % counts;
    let g = ((w - s) / counts) & IVec3::splat(255);
    (s, (g.x + g.y * 256 + g.z * 65536 + 1) as f32)
}

/// One cascade this frame: its first cell, spacing and first probe vec4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cascade {
    pub start: IVec3,
    pub spacing: f32,
    pub first: usize,
}

/// The probes of a cascade one frame updates: those whose slot index is
/// `phase` mod `period`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Slice {
    pub cascade: usize,
    pub period: u32,
    pub phase: u32,
    pub probes: u32,
    /// The slice's first ray in the ray buffer.
    pub first_ray: u32,
}

/// The cascades of `desc` for the camera at `eye` looking along unit
/// `forward`, their probes from vec4 `region`, and the slices frame
/// `frame` updates (`reset`: every probe of every cascade).
///
/// Each cascade is centred [`AHEAD`] of its extent ahead of the eye,
/// not on it: what the frame shows lies ahead, so a camera-centred grid
/// spends half its probes behind the camera and lights the surfaces it
/// looks at from the next, twice as coarse cascade. A camera 8 m from a
/// room's back wall lit it from 1.5 m probes instead of 0.75 m ones and
/// lost a third of the red wall's bleed on the floor and walls beside it.
pub(crate) fn plan(
    desc: &ProbeGlobalIllumination,
    eye: Vec3,
    forward: Vec3,
    region: usize,
    frame: u64,
    reset: bool,
) -> (Vec<Cascade>, Vec<Slice>) {
    let counts = IVec3::from_array(COUNTS.map(|c| c as i32));
    let n = desc.cascades.clamp(1, CASCADES as u32) as usize;
    let per = probes_per_cascade();
    let rays = rays(desc);
    let mut cascades = Vec::with_capacity(n);
    let mut slices = Vec::with_capacity(n);
    let mut first_ray = 0u32;
    for c in 0..n {
        let spacing = desc.spacing.max(0.05) * (1 << c) as f32;
        let ahead = forward * Vec3::from(COUNTS.map(|c| c as f32)) * (spacing * AHEAD);
        let centre = ((eye + ahead) / spacing).floor().as_ivec3();
        cascades.push(Cascade {
            start: centre - counts / 2,
            spacing,
            first: region + c * per * STRIDE,
        });
        let period = if reset { 1 } else { 1u32 << c };
        let phase = (frame % period as u64) as u32;
        let probes = (per as u32 - phase).div_ceil(period);
        slices.push(Slice {
            cascade: c,
            period,
            phase,
            probes,
            first_ray,
        });
        first_ray += probes * rays;
    }
    (cascades, slices)
}

fn rays(desc: &ProbeGlobalIllumination) -> u32 {
    desc.rays.clamp(32, 256)
}

/// The probe region's vec4 for `desc`.
pub(crate) fn region_vec4s(desc: &ProbeGlobalIllumination) -> usize {
    desc.cascades.clamp(1, CASCADES as u32) as usize * probes_per_cascade() * STRIDE
}

/// The header rows for `cascades` (none: off).
pub(crate) fn header(desc: &ProbeGlobalIllumination, cascades: &[Cascade]) -> Vec<[f32; 4]> {
    let mut rows = vec![[0.0; 4]; HEADER_ROWS];
    if cascades.is_empty() {
        return rows;
    }
    rows[0] = [
        cascades.len() as f32,
        desc.intensity.max(0.0),
        NORMAL_BIAS,
        VIEW_BIAS,
    ];
    for (c, k) in cascades.iter().enumerate() {
        rows[1 + 2 * c] = k.start.as_vec3().extend(k.spacing).to_array();
        rows[2 + 2 * c] = [
            COUNTS[0] as f32,
            COUNTS[1] as f32,
            COUNTS[2] as f32,
            k.first as f32,
        ];
    }
    rows
}

/// A rotation drawn from frame `frame` alone (the rays turn every frame,
/// the same way on every run): a unit quaternion from a hash of it.
fn rotation(frame: u64) -> Mat3 {
    let mut s = frame.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0xd1b5_4a32_d192_ed03;
    let mut next = || {
        s ^= s >> 33;
        s = s.wrapping_mul(0xff51_afd7_ed55_8ccd);
        s ^= s >> 33;
        (s >> 40) as f32 / (1u64 << 24) as f32
    };
    // Shoemake's uniform random rotation.
    let (u1, u2, u3) = (next(), next(), next());
    let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
    let tau = std::f32::consts::TAU;
    let q = glam::Quat::from_xyzw(
        a * (tau * u2).sin(),
        a * (tau * u2).cos(),
        b * (tau * u3).sin(),
        b * (tau * u3).cos(),
    );
    Mat3::from_quat(q.normalize())
}

/// The instances the GI traces: every opaque mesh instance that casts
/// shadows and every terrain with its heights on the CPU, the nearest
/// [`crate::gi_scene::MAX_INSTANCES`] to `eye`.
fn placed(scene: &crate::RenderScene, gi: &mut GiScene, eye: Vec3) -> Vec<Placed> {
    let mut out = Vec::with_capacity(scene.objects.instances.len() + scene.world.terrains.len());
    let mean = |gi: &mut GiScene, slot: u32| {
        crate::TextureHandle::from_slot(slot).map_or(Vec3::ONE, |t| {
            gi.texture_mean(
                t.index(),
                scene.texture_revision(t.index()),
                scene.texture(t),
            )
        })
    };
    for inst in &scene.objects.instances {
        if !inst.cast_shadows || !scene.contains_mesh(inst.mesh) {
            continue;
        }
        let Some(m) = scene.standard_material(inst.material) else {
            continue;
        };
        if m.transparent != 0 && m.opacity < 0.5 || m.transmission > 0.5 {
            continue;
        }
        if scene.mesh(inst.mesh).is_none_or(|d| d.indices.is_empty()) {
            continue;
        }
        let albedo = m.color * mean(gi, m.map);
        let emissive = m.emissive * m.emissiveIntensity.max(0.0) * mean(gi, m.emissiveMap);
        out.push(Placed {
            source: Source::Mesh(inst.mesh.index(), scene.mesh_revision(inst.mesh.index())),
            transform: inst.transform,
            albedo: albedo.clamp(Vec3::ZERO, Vec3::ONE),
            emissive,
            two_sided: m.side != 0,
        });
    }
    for t in &scene.world.terrains {
        if !t.visible || scene.terrain(t.terrain).is_none() {
            continue;
        }
        let albedo = t.layers.first().map_or(Vec3::splat(0.5), |l| {
            l.color
                * l.albedo.map_or(Vec3::ONE, |h| {
                    gi.texture_mean(
                        h.index(),
                        scene.texture_revision(h.index()),
                        scene.texture(h),
                    )
                })
        });
        let index = t.terrain.index();
        out.push(Placed {
            source: Source::Terrain(index, scene.terrain_revision(index)),
            transform: t.transform,
            albedo: albedo.clamp(Vec3::ZERO, Vec3::ONE),
            emissive: Vec3::ZERO,
            two_sided: false,
        });
    }
    if out.len() > crate::gi_scene::MAX_INSTANCES {
        let d = |p: &Placed| p.transform.w_axis.truncate().distance_squared(eye);
        out.sort_by(|a, b| d(a).total_cmp(&d(b)));
        out.truncate(crate::gi_scene::MAX_INSTANCES);
    }
    out
}

/// `GiParams` (gi_trace.wgsl, gi_blend.wgsl), one per slice.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GiParams {
    rotation: [[f32; 4]; 3],
    grid0: [f32; 4],
    grid1: [f32; 4],
    slice: [u32; 4],
    trace: [f32; 4],
    blend: [f32; 4],
    misc: [u32; 4],
}

/// A storage buffer grown to fit (in bytes, to a power of two).
struct Grown {
    label: &'static str,
    buffer: Option<wgpu::Buffer>,
    /// Bumped when the buffer is made again.
    generation: u64,
}

impl Grown {
    fn new(label: &'static str) -> Grown {
        Grown {
            label,
            buffer: None,
            generation: 0,
        }
    }

    /// The buffer, at least `bytes` long; true when it was made again.
    fn fit(&mut self, device: &wgpu::Device, bytes: u64) -> bool {
        if self.buffer.as_ref().is_some_and(|b| b.size() >= bytes) {
            return false;
        }
        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label),
            size: bytes.max(256).next_power_of_two(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.generation += 1;
        true
    }

    fn get(&self) -> &wgpu::Buffer {
        self.buffer.as_ref().expect("fitted first")
    }

    fn bytes(&self) -> u64 {
        self.buffer.as_ref().map_or(0, wgpu::Buffer::size)
    }
}

/// The trace module (`crate::shaders` validates it): the frame group with
/// the probe sampling, the light tables, the trace.
pub(crate) fn trace_source() -> String {
    format!(
        "{}{}{}",
        crate::FRAME_WGSL,
        crate::clusters::wgsl(),
        wgsl!("gi_trace.wgsl")
    )
}

/// The blend module.
pub(crate) fn blend_source() -> String {
    format!("{}{}", wgsl!("gi_common.wgsl"), wgsl!("gi_blend.wgsl"))
}

/// The probe GI's GPU side.
pub(crate) struct GiProbes {
    /// The frame group's bindings the trace reads (0, 3, 4, 6, 7, 10),
    /// visible to compute.
    pub frame_layout: wgpu::BindGroupLayout,
    trace_layout: wgpu::BindGroupLayout,
    blend_layout: wgpu::BindGroupLayout,
    trace: wgpu::ComputePipeline,
    blend: wgpu::ComputePipeline,
    params: wgpu::Buffer,
    bvh: Grown,
    rays: Grown,
    estimator: Grown,
    scene: GiScene,
    /// The settings the probes were traced with (another starts again).
    desc: Option<ProbeGlobalIllumination>,
    /// Frames traced since the last start.
    frame: u64,
    /// This frame's slices, to record.
    slices: Vec<Slice>,
    /// The header rows to write after the light volumes' and those written.
    header: Vec<[f32; 4]>,
    written: Vec<[f32; 4]>,
}

impl GiProbes {
    pub fn new(device: &wgpu::Device) -> GiProbes {
        let compute = wgpu::ShaderStages::COMPUTE;
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: compute,
            ty,
            count: None,
        };
        let storage = |read_only| wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let cube = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::Cube,
            multisampled: false,
        };
        let params = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GiParams>() as u64),
        };
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gi probes frame"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(3, cube),
                entry(4, cube),
                entry(
                    6,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                entry(7, storage(true)),
                entry(10, storage(true)),
            ],
        });
        let trace_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gi probes trace"),
            entries: &[
                entry(0, storage(true)),
                entry(1, storage(false)),
                entry(2, params),
            ],
        });
        let blend_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gi probes blend"),
            entries: &[
                entry(0, storage(true)),
                entry(1, storage(false)),
                entry(2, storage(false)),
                entry(3, params),
            ],
        });
        let pipeline = |label: &str,
                        layouts: &[Option<&wgpu::BindGroupLayout>],
                        source: String,
                        entry: &str| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: layouts,
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let trace = pipeline(
            "gi probes trace",
            &[Some(&frame_layout), Some(&trace_layout)],
            trace_source(),
            "cs_trace",
        );
        let blend = pipeline(
            "gi probes blend",
            &[Some(&blend_layout)],
            blend_source(),
            "cs_blend",
        );
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gi probes params"),
            size: PARAMS_STRIDE * MAX_SLICES as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        GiProbes {
            frame_layout,
            trace_layout,
            blend_layout,
            trace,
            blend,
            params,
            bvh: Grown::new("gi probes scene"),
            rays: Grown::new("gi probes rays"),
            estimator: Grown::new("gi probes estimator"),
            scene: GiScene::default(),
            desc: None,
            frame: 0,
            slices: Vec::new(),
            header: vec![[0.0; 4]; HEADER_ROWS],
            written: Vec::new(),
        }
    }

    /// Starts the probes again on the next frame (the buffer holding them
    /// was made again).
    pub fn restart(&mut self) {
        self.desc = None;
        self.written.clear();
    }

    /// Turns the probes off: the lit passes stop reading them.
    pub fn disable(&mut self) {
        self.desc = None;
        self.slices.clear();
        self.header = vec![[0.0; 4]; HEADER_ROWS];
    }

    /// Plans this frame: the traced scene built and written, the slices
    /// and their parameters, the header rows (written by
    /// [`GiProbes::write_header`] after the light volumes' rows).
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        desc: ProbeGlobalIllumination,
        region: usize,
    ) {
        let reset = self.desc != Some(desc);
        if reset {
            self.frame = 0;
        }
        self.desc = Some(desc);
        let eye = scene.frame.camera.position;
        let forward = (scene.frame.camera.target - eye).normalize_or_zero();
        let placed = placed(scene, &mut self.scene, eye);
        self.scene.update(&placed, |s| match s {
            Source::Mesh(i, _) => scene
                .mesh_handle(i)
                .and_then(|h| scene.mesh(h))
                .map(|m| {
                    let p: Vec<Vec3> = m.vertices.iter().map(|v| Vec3::from(v.position)).collect();
                    crate::gi_scene::mesh_triangles(
                        &p,
                        &m.indices,
                        crate::gi_scene::MAX_BLAS_TRIANGLES,
                    )
                })
                .unwrap_or_default(),
            Source::Terrain(i, _) => scene
                .world
                .terrains
                .iter()
                .find(|t| t.terrain.index() == i)
                .and_then(|t| scene.terrain(t.terrain))
                .map(crate::gi_scene::terrain_triangles)
                .unwrap_or_default(),
        });
        let words = &self.scene.words;
        if self.bvh.fit(device, (words.len() * 16) as u64) || self.scene.static_dirty {
            queue.write_buffer(self.bvh.get(), 0, bytemuck::cast_slice(words));
            self.scene.static_dirty = false;
        } else {
            let at = self.scene.static_len;
            queue.write_buffer(
                self.bvh.get(),
                (at * 16) as u64,
                bytemuck::cast_slice(&words[at..]),
            );
        }
        let (cascades, slices) = plan(&desc, eye, forward, region, self.frame, reset);
        let rays = rays(&desc);
        let probes = cascades.len() * probes_per_cascade();
        // A reset traces every probe at once.
        self.rays.fit(device, (probes * rays as usize * 16) as u64);
        self.estimator.fit(
            device,
            (probes * RADIANCE_TEXELS * ESTIMATOR_WORDS * 16) as u64,
        );
        let r = rotation(self.frame);
        let reach = cascades
            .last()
            .map_or(0.0, |c| c.spacing * COUNTS[0] as f32);
        let mut bytes = vec![0u8; PARAMS_STRIDE as usize * slices.len()];
        for (k, s) in slices.iter().enumerate() {
            let c = &cascades[s.cascade];
            let p = GiParams {
                rotation: [
                    r.x_axis.extend(0.0).to_array(),
                    r.y_axis.extend(0.0).to_array(),
                    r.z_axis.extend(0.0).to_array(),
                ],
                grid0: c.start.as_vec3().extend(c.spacing).to_array(),
                grid1: [
                    COUNTS[0] as f32,
                    COUNTS[1] as f32,
                    COUNTS[2] as f32,
                    c.first as f32,
                ],
                slice: [s.period, s.phase, s.probes, rays],
                trace: [
                    self.scene.root as f32,
                    self.scene.instances as f32,
                    desc.intensity.max(0.0),
                    // Far enough to leave the coarsest cascade.
                    2.0 * reach,
                ],
                blend: [
                    1.5 * c.spacing,
                    SHORT_BLEND,
                    DEPTH_BLEND,
                    reset as u32 as f32,
                ],
                misc: [
                    (self.frame % (1 << 24)) as u32,
                    s.first_ray,
                    (s.cascade * probes_per_cascade() * RADIANCE_TEXELS * ESTIMATOR_WORDS) as u32,
                    scene.lighting.lights.len().min(crate::MAX_LIGHTS) as u32,
                ],
            };
            let o = k * PARAMS_STRIDE as usize;
            bytes[o..o + std::mem::size_of::<GiParams>()].copy_from_slice(bytemuck::bytes_of(&p));
        }
        queue.write_buffer(&self.params, 0, &bytes);
        self.slices = slices;
        self.header = header(&desc, &cascades);
        self.frame += 1;
    }

    /// Writes the header rows into the light volumes' buffer when they
    /// changed (after [`crate::lightmap::Lightmaps::prepare`], which may
    /// have written the volumes' header again).
    pub fn write_header(&mut self, queue: &wgpu::Queue, probes: &wgpu::Buffer, force: bool) {
        if !force && self.header == self.written {
            return;
        }
        queue.write_buffer(
            probes,
            (HEADER_AT * 16) as u64,
            bytemuck::cast_slice(&self.header),
        );
        self.written = self.header.clone();
    }

    /// Whether this frame traces.
    pub fn due(&self) -> bool {
        !self.slices.is_empty()
    }

    /// Records this frame's trace and blend, `frame` the trace's frame group
    /// (made with [`GiProbes::frame_layout`]).
    pub fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &wgpu::BindGroup,
        probes: &wgpu::Buffer,
    ) {
        let params = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.params,
            offset: 0,
            size: wgpu::BufferSize::new(std::mem::size_of::<GiParams>() as u64),
        });
        let trace = crate::volumetric::bind_at(
            device,
            "gi probes trace",
            &self.trace_layout,
            &[
                (0, self.bvh.get().as_entire_binding()),
                (1, self.rays.get().as_entire_binding()),
                (2, params.clone()),
            ],
        );
        let blend = crate::volumetric::bind_at(
            device,
            "gi probes blend",
            &self.blend_layout,
            &[
                (0, self.rays.get().as_entire_binding()),
                (1, probes.as_entire_binding()),
                (2, self.estimator.get().as_entire_binding()),
                (3, params),
            ],
        );
        let rays = self.desc.map_or(64, |d| rays(&d));
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("gi probes"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.trace);
        pass.set_bind_group(0, frame, &[]);
        for (k, s) in self.slices.iter().enumerate() {
            pass.set_bind_group(1, &trace, &[(k as u64 * PARAMS_STRIDE) as u32]);
            pass.dispatch_workgroups((s.probes * rays).div_ceil(64), 1, 1);
        }
        pass.set_pipeline(&self.blend);
        for (k, s) in self.slices.iter().enumerate() {
            pass.set_bind_group(0, &blend, &[(k as u64 * PARAMS_STRIDE) as u32]);
            pass.dispatch_workgroups(s.probes, 1, 1);
        }
    }

    /// The GPU memory it holds, in bytes.
    pub fn bytes(&self) -> u64 {
        self.bvh.bytes() + self.rays.bytes() + self.estimator.bytes() + self.params.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tap one past an edge of the octahedral map reads the texel the
    /// map folds onto there: the same as Wicked's stored borders
    /// (DDGI_DEPTH_BORDER_OFFSETS, for its 16-texel map: border (x, 0)
    /// copies (17 - x, 1), (0, y) copies (1, 17 - y), and the corner
    /// (0, 0) copies (16, 16), in its one-based coordinates).
    #[test]
    fn the_octahedral_fold_matches_wicked_s_borders() {
        let n = 16;
        // One-based with a border, as Wicked's table, to zero-based.
        let z = |x: i32, y: i32| IVec2::new(x - 1, y - 1);
        for k in 1..=16 {
            assert_eq!(oct_wrap(z(k, 0), n), z(17 - k, 1), "top border at {k}");
            assert_eq!(oct_wrap(z(k, 17), n), z(17 - k, 16), "bottom border at {k}");
            assert_eq!(oct_wrap(z(0, k), n), z(1, 17 - k), "left border at {k}");
            assert_eq!(oct_wrap(z(17, k), n), z(16, 17 - k), "right border at {k}");
        }
        assert_eq!(oct_wrap(z(0, 0), n), z(16, 16));
        assert_eq!(oct_wrap(z(17, 0), n), z(1, 16));
        assert_eq!(oct_wrap(z(0, 17), n), z(16, 1));
        assert_eq!(oct_wrap(z(17, 17), n), z(1, 1));
        assert_eq!(oct_wrap(IVec2::new(5, 7), n), IVec2::new(5, 7));
    }

    /// The camera crossing one cell moves the grid by one plane: every
    /// other cell keeps its slot and key, and the slot the cell that left
    /// held now holds the one that came in under another key (so it is
    /// traced afresh and, meanwhile, left out by the lit passes).
    #[test]
    fn a_scrolled_grid_keeps_its_cells_where_they_were() {
        let counts = IVec3::new(16, 8, 16);
        let desc = ProbeGlobalIllumination::default();
        let (a, _) = plan(&desc, Vec3::new(-3.2, 1.0, 40.5), Vec3::NEG_Z, 0, 0, false);
        let (b, _) = plan(&desc, Vec3::new(-2.2, 1.0, 40.5), Vec3::NEG_Z, 0, 0, false);
        assert_eq!(b[0].start - a[0].start, IVec3::X);
        let cells = |s: IVec3| {
            (0..counts.x).flat_map(move |x| {
                (0..counts.y).flat_map(move |y| (0..counts.z).map(move |z| s + IVec3::new(x, y, z)))
            })
        };
        let mut slots = std::collections::HashMap::new();
        for w in cells(a[0].start) {
            slots.insert(slot_and_key(w, counts).0, w);
        }
        assert_eq!(slots.len(), 2048, "a slot for every probe");
        let mut moved = 0;
        for w in cells(b[0].start) {
            let (s, key) = slot_and_key(w, counts);
            let before = slots[&s];
            if before == w {
                assert_eq!(slot_and_key(before, counts).1, key);
            } else {
                moved += 1;
                assert_eq!(w - before, IVec3::new(16, 0, 0));
                assert_ne!(slot_and_key(before, counts).1, key);
            }
        }
        assert_eq!(moved, 8 * 16, "one plane of probes");
    }

    /// What the camera looks at is lit by the finest cascade at full
    /// weight (gi_probes.wgsl `gi_cascade`'s fade over the outer two
    /// cells), and so is the camera's own spot: gi-room's camera, 8 m in
    /// front of the back wall, saw that wall from the second cascade's
    /// 1.5 m probes while half the first cascade lay behind it.
    #[test]
    fn the_finest_cascade_covers_what_the_camera_looks_at() {
        let weight = |c: &Cascade, p: Vec3| {
            let g = p / c.spacing - c.start.as_vec3();
            let top = Vec3::from(COUNTS.map(|n| n as f32)) - 1.0;
            (g.min(top - g).min_element() * 0.5).clamp(0.0, 1.0)
        };
        let desc = ProbeGlobalIllumination {
            spacing: 0.75,
            cascades: 2,
            ..Default::default()
        };
        let (eye, target) = (Vec3::new(0.0, 2.2, 6.0), Vec3::new(0.0, 0.9, -1.0));
        let (c, _) = plan(&desc, eye, (target - eye).normalize(), 0, 0, false);
        assert_eq!(weight(&c[0], eye), 1.0, "under the camera");
        for p in [
            Vec3::new(-2.5, 1.5, -2.4),
            Vec3::new(2.5, 0.0, -2.4),
            Vec3::new(0.0, 0.0, 1.0),
        ] {
            assert!(weight(&c[0], p) >= 0.9, "{p}: {}", weight(&c[0], p));
        }
    }

    /// Cascade c updates a 2^c-th of its probes a frame, so every probe
    /// over 2^c frames, each once; a reset updates them all at once, and
    /// the slices' rays lie one after another.
    #[test]
    fn the_slices_cover_each_cascade_over_its_period() {
        let desc = ProbeGlobalIllumination {
            cascades: 3,
            rays: 64,
            ..Default::default()
        };
        for c in 0..3 {
            let period = 1u32 << c;
            let mut seen = vec![0u32; probes_per_cascade()];
            for frame in 0..period as u64 {
                let s = plan(&desc, Vec3::ZERO, Vec3::NEG_Z, 0, frame, false).1[c];
                assert_eq!(s.period, period);
                for k in 0..s.probes {
                    seen[(s.phase + k * s.period) as usize] += 1;
                }
            }
            assert!(seen.iter().all(|&n| n == 1), "cascade {c}");
        }
        let (_, all) = plan(&desc, Vec3::ZERO, Vec3::NEG_Z, 0, 5, true);
        assert!(all.iter().all(|s| s.period == 1 && s.probes == 2048));
        assert_eq!(all[2].first_ray, 2 * 2048 * 64);
        let (c, _) = plan(
            &desc,
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::NEG_Z,
            1000,
            0,
            false,
        );
        assert_eq!(c[1].first, 1000 + 2048 * STRIDE);
        assert_eq!(c[2].spacing, 4.0);
        assert_eq!(region_vec4s(&desc), 3 * 2048 * STRIDE);
    }

    /// The rays turn every frame, the same way on every run.
    #[test]
    fn the_rotation_is_a_rotation_of_the_frame_alone() {
        for f in 0..32 {
            let r = rotation(f);
            assert!((r.determinant() - 1.0).abs() < 1e-4);
            assert!((r * r.transpose() - Mat3::IDENTITY).abs_diff_eq(Mat3::ZERO, 1e-4));
            assert_eq!(r, rotation(f));
        }
        assert_ne!(rotation(1), rotation(2));
    }
}
