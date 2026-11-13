//! Baked lighting in the lit shaders (docs/design.md §2.6): lightmaps on
//! static meshes and light volumes for everything else.
//!
//! `og bake lightmaps` (og-bake) traces the light arriving from the scene
//! (bounced off its static models, from the sky, from emissive surfaces;
//! never a light's direct term, which the renderer draws every frame with
//! its shadows) into lightmap pages and L1 spherical-harmonic probe grids.
//! The host hands them over in [`crate::RenderScene::baked`]:
//!
//! - [`LightmapPages`]: RGBA16F array layers of irradiance over π, linear,
//!   the convention of the environment's irradiance cube, so the shader
//!   swaps one for the other. Every baked scene brings its own pages:
//!   [`BakedLighting::meshes`] names each lightmapped mesh with the pages
//!   of the scene it was baked in and its per-vertex lightmap uvs (a second
//!   vertex stream at location 9), and each set of pages is one texture
//!   array with its own group, bound at group 3 for the batches of its
//!   meshes, so any number of scenes loaded together draw their bakes. A
//!   lightmapped mesh is drawn in every kind of lit batch ([`Family`]):
//!   the standard module's `vs_lightmap`/`fs_lightmap` (lightmap.wgsl;
//!   `vs_lightmap_compact` for compact vertices, pbr_array.wgsl's
//!   `fs_lightmap_array` for array maps, a material shader's module the
//!   same entry points with its hooks) and the physical module's
//!   `vs_physical_lightmap`/`fs_physical_lightmap`
//!   (lightmap_physical.wgsl).
//! - [`ProbeVolume`]s: one storage buffer at frame binding 10 (a volume
//!   larger than the buffer holds is resampled to fit), read by `ambient_irradiance` for
//!   every lit fragment without a lightmap: trilinear between the eight
//!   probes around it, faded in over one spacing inside the box.
//!
//! The data is renderer state that outlives frames: a render snapshot's
//! restore (`Renderer::restore_state`) keeps it, as it keeps every upload.

use glam::Vec3;
pub use og_render_data::lightmap::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Lightmap pages: `width × height` texels each, row by row from the top,
/// linear irradiance over π (alpha unused).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightmapPages {
    pub width: u32,
    pub height: u32,
    pub pages: Vec<Vec<[f32; 4]>>,
}

/// A lightmapped mesh's lightmap coordinates: per vertex its uv on the
/// page (0..1, v down) and the page.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightmapUvs {
    pub uvs: Vec<[f32; 3]>,
}

/// A box of baked probes, axis-aligned in world space: `counts` along x,
/// y and z (x fastest, 2 to 64 each), each probe the irradiance over π of
/// red, green and blue as `[c1.x, c1.y, c1.z, c0]` (`c0 + c1 · n`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProbeVolume {
    pub min: Vec3,
    pub max: Vec3,
    pub counts: [u32; 3],
    pub probes: Arc<Vec<[[f32; 4]; 3]>>,
}

/// One probe: the irradiance harmonics of red, green and blue.
pub type Probe = [[f32; 4]; 3];

/// A lightmapped mesh: the pages of the scene it was baked in and its
/// lightmap uvs on them (one per vertex of the mesh).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedMesh {
    pub pages: Arc<LightmapPages>,
    pub uvs: Arc<LightmapUvs>,
}

/// The baked lighting a scene draws with ([`crate::RenderScene::baked`]):
/// every loaded scene's bake at once.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedLighting {
    /// Lightmapped meshes, by mesh index ([`crate::MeshHandle::index`]),
    /// each with its scene's pages (meshes of one scene share the Arc: one
    /// texture array and group per distinct Arc).
    pub meshes: HashMap<usize, BakedMesh>,
    /// Every scene's light volumes; the first [`PROBE_VOLUMES`] are drawn.
    pub volumes: Vec<ProbeVolume>,
}

/// The light volumes' header: two vec4 per volume.
const VOLUME_HEADER: usize = 2 * PROBE_VOLUMES;
/// The probe buffer's header: the volumes', then the probe GI's rows
/// (gi_probes.rs, written each frame after the volumes').
const HEADER: usize = VOLUME_HEADER + crate::gi_probes::HEADER_ROWS;
/// The probe buffer's size in vec4: the header and 16384 probes of three
/// vec4.
pub(crate) const PROBE_CAPACITY: usize = HEADER + 3 * 16384;

/// The frame layout's entry for the probe buffer.
pub(crate) fn probe_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 10,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// The probe buffer's contents for `volumes` in `capacity` vec4: the
/// header (per volume its min and first probe, its max and packed counts;
/// first -1 ends the list), then the probes. A volume that does not fit
/// what is left is resampled to fewer probes; one that cannot fit two a
/// side is left out.
pub(crate) fn pack_probes(volumes: &[ProbeVolume], capacity: usize) -> Vec<[f32; 4]> {
    let mut out = vec![[0.0, 0.0, 0.0, -1.0]; VOLUME_HEADER];
    out.resize(HEADER, [0.0; 4]);
    for (i, v) in volumes.iter().take(PROBE_VOLUMES).enumerate() {
        let left = capacity.saturating_sub(out.len()) / 3;
        let Some((counts, probes)) = fit(v, left) else {
            break;
        };
        [out[2 * i], out[2 * i + 1]] = header(v.min, v.max, out.len(), counts, false);
        for p in probes {
            out.extend(p);
        }
    }
    out
}

/// A volume's two header rows: its min and first probe (in vec4), its max
/// and packed counts (7 bits a side, bit 21 on a realtime volume).
pub(crate) fn header(
    min: Vec3,
    max: Vec3,
    first: usize,
    counts: [u32; 3],
    realtime: bool,
) -> [[f32; 4]; 2] {
    let packed = counts[0] + (counts[1] << 7) + (counts[2] << 14) + ((realtime as u32) << 21);
    [
        min.extend(first as f32).to_array(),
        max.extend(packed as f32).to_array(),
    ]
}

/// A volume's probes within `room` probes: as baked, or trilinearly
/// resampled to the largest grid of its proportions that fits.
fn fit(v: &ProbeVolume, room: usize) -> Option<([u32; 3], Vec<Probe>)> {
    let c = v.counts.map(|n| n.clamp(2, 64));
    if (c[0] * c[1] * c[2]) as usize > v.probes.len() {
        return None;
    }
    let total = (c[0] * c[1] * c[2]) as usize;
    let n = fit_counts(c, room)?;
    if n == c {
        return Some((c, v.probes[..total].to_vec()));
    }
    let at = |x: u32, y: u32, z: u32| v.probes[((z * c[1] + y) * c[0] + x) as usize];
    let mut probes = Vec::with_capacity((n[0] * n[1] * n[2]) as usize);
    for z in 0..n[2] {
        for y in 0..n[1] {
            for x in 0..n[0] {
                let idx = [x, y, z];
                let g = [0, 1, 2].map(|k| idx[k] as f32 / (n[k] - 1) as f32 * (c[k] - 1) as f32);
                let g0 = [0, 1, 2].map(|k| (g[k].floor() as u32).min(c[k] - 2));
                let f = [0, 1, 2].map(|k| g[k] - g0[k] as f32);
                let mut p = [[0.0f32; 4]; 3];
                for corner in 0..8u32 {
                    let o = [corner & 1, (corner >> 1) & 1, corner >> 2];
                    let w = (0..3)
                        .map(|k| if o[k] == 1 { f[k] } else { 1.0 - f[k] })
                        .product::<f32>();
                    let q = at(g0[0] + o[0], g0[1] + o[1], g0[2] + o[2]);
                    for (pc, qc) in p.iter_mut().zip(q) {
                        for (a, b) in pc.iter_mut().zip(qc) {
                            *a += b * w;
                        }
                    }
                }
                probes.push(p);
            }
        }
    }
    Some((n, probes))
}

/// The largest grid of `c`'s proportions (2 to 64 a side) with at most
/// `room` probes: `c` itself when it fits; none when two a side do not.
pub(crate) fn fit_counts(c: [u32; 3], room: usize) -> Option<[u32; 3]> {
    let c = c.map(|n| n.clamp(2, 64));
    let total = (c[0] * c[1] * c[2]) as usize;
    if total <= room {
        return Some(c);
    }
    let scale = (room as f32 / total as f32).cbrt();
    let mut n = c.map(|k| ((k as f32 * scale).floor() as u32).clamp(2, k));
    while (n[0] * n[1] * n[2]) as usize > room {
        let k = (0..3).max_by_key(|&k| n[k])?;
        if n[k] <= 2 {
            return None;
        }
        n[k] -= 1;
    }
    Some(n)
}

/// An f32 as an IEEE half (round to nearest; overflow to infinity).
pub(crate) fn f16_bits(x: f32) -> u16 {
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32;
    let mant = b & 0x7f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if mant != 0 { 0x200 } else { 0 };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    if e <= 0 {
        if e < -10 {
            return sign;
        }
        let m = (mant | 0x80_0000) >> (1 - e);
        let round = (m >> 12) & 1;
        return sign | ((m >> 13) + round) as u16;
    }
    let half = sign as u32 | ((e as u32) << 10) | (mant >> 13);
    // Round to nearest (a carry into the exponent is the right result).
    (half + ((mant >> 12) & 1)) as u16
}

/// The GPU side of [`BakedLighting`].
pub(crate) struct Lightmaps {
    layout: wgpu::BindGroupLayout,
    /// The frame group's layout, every lightmapped pipeline's group 0.
    frame_layout: wgpu::BindGroupLayout,
    /// The pipeline layouts: frame, a family's material and effect groups,
    /// the pages (by the family's [`Groups`]).
    pipeline_layouts: HashMap<Groups, wgpu::PipelineLayout>,
    sampler: wgpu::Sampler,
    /// Each set of pages a lightmapped mesh names, by its Arc (held, so the
    /// address is not reused while it keys the group), and its group.
    pages: HashMap<usize, (Arc<LightmapPages>, wgpu::BindGroup)>,
    /// Per lightmapped mesh index: its uvs and pages (by Arc) and the uvs'
    /// buffer.
    uvs: HashMap<usize, MeshUvs>,
    /// The lit pipelines of lightmapped batches: their family and (side,
    /// blended, effects).
    pipelines: HashMap<(Family, LitKey), wgpu::RenderPipeline>,
    /// Frame binding 10, and what was last written to it: the baked
    /// volumes (their key, header and where their probes end) and the
    /// header with the realtime volumes swept.
    pub probe_buffer: wgpu::Buffer,
    probes_key: Vec<(Vec3, Vec3, [u32; 3], usize)>,
    baked_header: Vec<[f32; 4]>,
    baked_end: usize,
    header: Vec<[f32; 4]>,
    /// The buffer's size in vec4: [`PROBE_CAPACITY`], or more with the
    /// probe GI's region after it ([`Lightmaps::ensure_capacity`]).
    capacity: usize,
    /// The realtime volumes (probe_grid.rs) and what relights them.
    pub grid: crate::probe_grid::ProbeGrid,
    pub projector: crate::probe_grid::GridProjector,
}

/// A lightmapped mesh on the GPU.
struct MeshUvs {
    /// Held as the pages are.
    uvs: Arc<LightmapUvs>,
    pages: usize,
    buffer: wgpu::Buffer,
}

/// The identity of an Arc's contents.
fn key<T>(a: &Arc<T>) -> usize {
    Arc::as_ptr(a) as usize
}

/// A lightmapped pipeline's (side, blended, effects).
pub(crate) type LitKey = (crate::Faces, bool, bool);

/// The lit pipeline family a lightmapped batch draws with: each the lit
/// shading of its kind of batch with the lightmap's vertex stream and
/// pages (the entry points of lightmap.wgsl, pbr_array.wgsl and
/// lightmap_physical.wgsl), so a static mesh draws its bake whatever its
/// material.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Family {
    /// The standard lit module (a lean batch from its full vertices too).
    Standard,
    /// A lean batch from its compact vertices (compact.rs).
    Compact,
    /// An array-map material (material.rs `ArrayMaps`).
    Array,
    /// A physical material (physical.rs).
    Physical,
    /// A material shader's standard module, by its index.
    Shader(usize),
    /// A material shader's physical module.
    PhysicalShader(usize),
}

/// The bind group layouts a family's pipelines take at groups 1 and 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Groups {
    Standard,
    Array,
    Physical,
}

impl Family {
    fn groups(self) -> Groups {
        match self {
            Family::Standard | Family::Compact | Family::Shader(_) => Groups::Standard,
            Family::Array => Groups::Array,
            Family::Physical | Family::PhysicalShader(_) => Groups::Physical,
        }
    }

    /// Its vertex and fragment entry points.
    fn entries(self) -> (&'static str, &'static str) {
        match self {
            Family::Standard | Family::Shader(_) => ("vs_lightmap", "fs_lightmap"),
            Family::Compact => ("vs_lightmap_compact", "fs_lightmap"),
            Family::Array => ("vs_lightmap", "fs_lightmap_array"),
            Family::Physical | Family::PhysicalShader(_) => {
                ("vs_physical_lightmap", "fs_physical_lightmap")
            }
        }
    }
}

/// The lightmap uvs' vertex stream.
const LIGHTMAP_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: 12,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![9 => Float32x3],
};

impl Lightmaps {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_layout: &wgpu::BindGroupLayout,
    ) -> Lightmaps {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lightmap"),
            entries: &[
                crate::texture_entry(4, true, wgpu::TextureViewDimension::D2Array),
                crate::sampler_entry(5, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lightmap"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let capacity = PROBE_CAPACITY;
        let probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light volumes"),
            size: (capacity * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let empty = pack_probes(&[], capacity);
        queue.write_buffer(&probe_buffer, 0, bytemuck::cast_slice(&empty));
        let baked_header = empty[..VOLUME_HEADER].to_vec();
        Lightmaps {
            layout,
            frame_layout: frame_layout.clone(),
            pipeline_layouts: HashMap::new(),
            sampler,
            pages: HashMap::new(),
            uvs: HashMap::new(),
            pipelines: HashMap::new(),
            probe_buffer,
            probes_key: Vec::new(),
            baked_end: empty.len(),
            baked_header: baked_header.clone(),
            header: baked_header,
            capacity,
            grid: Default::default(),
            projector: crate::probe_grid::GridProjector::new(device),
        }
    }

    fn page_group(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        pages: &LightmapPages,
    ) -> wgpu::BindGroup {
        let layers = pages.pages.len().max(1) as u32;
        let (w, h) = (pages.width.max(1), pages.height.max(1));
        let texels = (w * h) as usize;
        let mut data: Vec<u16> = Vec::with_capacity(texels * 4 * layers as usize);
        for l in 0..layers as usize {
            match pages.pages.get(l) {
                Some(p) if p.len() >= texels => {
                    data.extend(p[..texels].iter().flat_map(|t| {
                        [
                            f16_bits(t[0]),
                            f16_bits(t[1]),
                            f16_bits(t[2]),
                            f16_bits(1.0),
                        ]
                    }));
                }
                _ => data.extend(std::iter::repeat_n(0u16, texels * 4)),
            }
        }
        let texture = wgpu::util::DeviceExt::create_texture_with_data(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("lightmap pages"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&data),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("lightmap pages"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        crate::volumetric::bind_at(
            device,
            "lightmap",
            layout,
            &[
                (4, wgpu::BindingResource::TextureView(&view)),
                (5, wgpu::BindingResource::Sampler(sampler)),
            ],
        )
    }

    /// Makes the probe buffer hold `vec4s` (the probe GI's region after
    /// [`PROBE_CAPACITY`]; it never shrinks). True when it was made again:
    /// its contents are written again by the next [`Lightmaps::prepare`]
    /// (the realtime volumes sweep again), and every group binding it must
    /// be made again.
    pub fn ensure_capacity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vec4s: usize,
    ) -> bool {
        if vec4s <= self.capacity {
            return false;
        }
        let capacity = vec4s.next_power_of_two().max(PROBE_CAPACITY);
        self.probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light volumes"),
            size: (capacity * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let empty = pack_probes(&[], PROBE_CAPACITY);
        queue.write_buffer(&self.probe_buffer, 0, bytemuck::cast_slice(&empty));
        self.capacity = capacity;
        self.probes_key = vec![(Vec3::NAN, Vec3::NAN, [0; 3], 0)];
        self.baked_end = empty.len();
        self.baked_header = empty[..VOLUME_HEADER].to_vec();
        self.header = self.baked_header.clone();
        self.grid = Default::default();
        true
    }

    /// Uploads what changed in `baked` since the last frame: the pages of
    /// every scene a lightmapped mesh names (each set once), the meshes'
    /// uvs; what no mesh names any more is let go. Then lays out the
    /// `realtime` volumes after the baked ones (probe_grid.rs) and answers
    /// the probes relit this frame.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        baked: &BakedLighting,
        realtime: &[crate::RealtimeLightVolume],
    ) -> Vec<crate::probe_grid::GridCapture> {
        self.uvs.retain(|mesh, m| {
            baked
                .meshes
                .get(mesh)
                .is_some_and(|b| Arc::ptr_eq(&b.uvs, &m.uvs) && key(&b.pages) == m.pages)
        });
        let named: std::collections::HashSet<usize> =
            baked.meshes.values().map(|b| key(&b.pages)).collect();
        self.pages.retain(|k, _| named.contains(k));
        for (&mesh, b) in &baked.meshes {
            let pages = key(&b.pages);
            if !self.pages.contains_key(&pages) {
                let group = Self::page_group(device, queue, &self.layout, &self.sampler, &b.pages);
                self.pages.insert(pages, (b.pages.clone(), group));
            }
            if self.uvs.contains_key(&mesh) {
                continue;
            }
            let buffer = crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("lightmap uvs"),
                    contents: bytemuck::cast_slice(if b.uvs.uvs.is_empty() {
                        &[[0.0f32; 3]]
                    } else {
                        &b.uvs.uvs[..]
                    }),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
            let uvs = b.uvs.clone();
            self.uvs.insert(mesh, MeshUvs { uvs, pages, buffer });
        }
        let probes_key: Vec<_> = baked
            .volumes
            .iter()
            .take(PROBE_VOLUMES)
            .map(|v| (v.min, v.max, v.counts, Arc::as_ptr(&v.probes) as usize))
            .collect();
        if probes_key != self.probes_key {
            let data = pack_probes(&baked.volumes, PROBE_CAPACITY);
            queue.write_buffer(&self.probe_buffer, 0, bytemuck::cast_slice(&data));
            self.probes_key = probes_key;
            self.baked_end = data.len();
            self.baked_header = data[..VOLUME_HEADER].to_vec();
            self.header = self.baked_header.clone();
        }
        let baked_volumes = self
            .baked_header
            .iter()
            .step_by(2)
            .take_while(|h| h[3] >= 0.0)
            .count();
        let (rows, captures) = self.grid.plan(realtime, self.baked_end, baked_volumes);
        let mut header = self.baked_header.clone();
        for (k, row) in rows.into_iter().enumerate() {
            let i = baked_volumes + k;
            [header[2 * i], header[2 * i + 1]] = row;
        }
        if header != self.header {
            queue.write_buffer(&self.probe_buffer, 0, bytemuck::cast_slice(&header));
            self.header = header;
        }
        self.projector.write(queue, &captures);
        captures
    }

    /// The lightmap uv stream of a mesh drawn with its lightmap.
    pub fn uvs(&self, mesh: usize) -> Option<&wgpu::Buffer> {
        self.uvs.get(&mesh).map(|m| &m.buffer)
    }

    /// The group of a lightmapped mesh's pages, and their identity (a pass
    /// binds it again only when the next batch's pages differ).
    pub fn group(&self, mesh: usize) -> Option<(usize, &wgpu::BindGroup)> {
        let m = self.uvs.get(&mesh)?;
        Some((m.pages, &self.pages.get(&m.pages)?.1))
    }

    /// Forgets the lit pipelines: the renderer's samples per pixel changed
    /// (each is made again by its next [`Self::ensure_pipeline`]).
    pub fn forget_pipelines(&mut self) {
        self.pipelines.clear();
    }

    /// Makes the lightmapped pipeline of `family` for `key` from `module`
    /// (the family's lit module), its groups 1 and 2 laid out as
    /// `material` and `fx`.
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_pipeline(
        &mut self,
        device: &wgpu::Device,
        family: Family,
        [material, fx]: [&wgpu::BindGroupLayout; 2],
        module: &wgpu::ShaderModule,
        key: LitKey,
        samples: u32,
    ) {
        if self.pipelines.contains_key(&(family, key)) {
            return;
        }
        let layout = self
            .pipeline_layouts
            .entry(family.groups())
            .or_insert_with(|| {
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("lit (lightmapped)"),
                    bind_group_layouts: &[
                        Some(&self.frame_layout),
                        Some(material),
                        Some(fx),
                        Some(&self.layout),
                    ],
                    immediate_size: 0,
                })
            });
        let (faces, blend, effects) = key;
        let constants = &[("EFFECTS", effects as u32 as f64)];
        let vertices = if family == Family::Compact {
            crate::compact::LEAN_LAYOUT
        } else {
            crate::VERTEX_LAYOUT
        };
        let buffers = [
            Some(vertices),
            Some(crate::INSTANCE_LAYOUT),
            Some(LIGHTMAP_LAYOUT),
        ];
        let (vs, fs) = family.entries();
        let p = crate::hdr_pipeline_facing(
            device,
            constants,
            faces.front(),
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "lit lightmapped",
                layout,
                module,
                vs,
                fs,
                buffers: &buffers,
                cull: faces.cull(),
                depth_write: !blend,
                depth_compare: wgpu::CompareFunction::Less,
                blend: blend.then_some(wgpu::BlendState::ALPHA_BLENDING),
                samples,
            },
        );
        self.pipelines.insert((family, key), p);
    }

    pub fn pipeline(&self, family: Family, key: LitKey) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&(family, key))
    }

    /// The pages' group layout (group 3 of every lightmapped pipeline).
    pub fn pages_layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// The order-independent variant (oit.rs) of `family`'s blended
    /// pipeline for (`side`, `effects`), over `layout` (the frame group
    /// of `mode`, the family's material and effect groups, the pages).
    #[allow(clippy::too_many_arguments)]
    pub fn oit_pipeline(
        device: &wgpu::Device,
        family: Family,
        layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
        (faces, effects): (crate::Faces, bool),
        samples: u32,
        mode: crate::oit::Oit,
        // The features left out (pipeline_features.rs).
        off: &[(&'static str, f64)],
    ) -> wgpu::RenderPipeline {
        let mut constants = vec![("EFFECTS", effects as u32 as f64)];
        constants.extend_from_slice(off);
        let vertices = if family == Family::Compact {
            crate::compact::LEAN_LAYOUT
        } else {
            crate::VERTEX_LAYOUT
        };
        let buffers = [
            Some(vertices),
            Some(crate::INSTANCE_LAYOUT),
            Some(LIGHTMAP_LAYOUT),
        ];
        let (vs, fs) = family.entries();
        crate::oit::pipeline(
            device,
            &constants,
            faces.front(),
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "lit lightmapped",
                layout,
                module,
                vs,
                fs,
                buffers: &buffers,
                cull: faces.cull(),
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            },
            mode,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(counts: [u32; 3], f: impl Fn(u32, u32, u32) -> f32) -> ProbeVolume {
        let mut probes = Vec::new();
        for z in 0..counts[2] {
            for y in 0..counts[1] {
                for x in 0..counts[0] {
                    let c = f(x, y, z);
                    probes.push([[0.0, 0.0, 0.0, c]; 3]);
                }
            }
        }
        ProbeVolume {
            min: Vec3::ZERO,
            max: Vec3::splat(4.0),
            counts,
            probes: Arc::new(probes),
        }
    }

    #[test]
    fn packs_header_and_probes() {
        let v = volume([2, 3, 4], |x, y, z| (x + 2 * y + 6 * z) as f32);
        let out = pack_probes(std::slice::from_ref(&v), 1024);
        assert_eq!(out[0][3], HEADER as f32);
        assert_eq!(out[1][3], (2 + (3 << 7) + (4 << 14)) as f32);
        assert_eq!(out[2][3], -1.0);
        // Probe (1, 2, 3) is index 23: its red c0.
        assert_eq!(out[HEADER + 23 * 3][3], 23.0);
        assert_eq!(out.len(), HEADER + 24 * 3);
    }

    #[test]
    fn resamples_a_volume_too_large_for_the_buffer() {
        // A linear field survives trilinear resampling exactly.
        let v = volume([16, 16, 16], |x, y, z| {
            x as f32 + 0.5 * y as f32 + 0.25 * z as f32
        });
        let out = pack_probes(std::slice::from_ref(&v), 1024);
        assert!(out.len() <= 1024);
        let packed = out[1][3] as u32;
        let n = [packed & 127, (packed >> 7) & 127, packed >> 14];
        assert!(n.iter().all(|&k| (2..16).contains(&k)), "{n:?}");
        // The far corner keeps its value: 15 + 7.5 + 3.75.
        let last = (n[0] * n[1] * n[2] - 1) as usize;
        assert!((out[HEADER + last * 3][3] - 26.25).abs() < 1e-3);
    }

    #[test]
    fn half_floats() {
        assert_eq!(f16_bits(1.0), 0x3c00);
        assert_eq!(f16_bits(-2.0), 0xc000);
        assert_eq!(f16_bits(0.0), 0);
        assert_eq!(f16_bits(65504.0), 0x7bff);
        assert_eq!(f16_bits(1e6), 0x7c00);
        assert_eq!(f16_bits(0.333_333_34), 0x3555);
    }
}
