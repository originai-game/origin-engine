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
//!   swaps one for the other ([`BakedLighting::meshes`] names the meshes
//!   drawn with them and their per-vertex lightmap uvs, a second vertex
//!   stream at location 9). A lightmapped mesh is drawn by the standard lit
//!   pipeline's `vs_lightmap`/`fs_lightmap` (lightmap.wgsl) with the pages
//!   at group 3; physical, array-map and material-shader batches keep the
//!   probes or the sky.
//! - [`ProbeVolume`]s: one buffer at frame binding 10 (storage on the full
//!   tier, a 16 KiB uniform on WebGL2, where a volume larger than the
//!   uniform holds is resampled to fit), read by `ambient_irradiance` for
//!   every lit fragment without a lightmap: trilinear between the eight
//!   probes around it, faded in over one spacing inside the box.
//!
//! The data is renderer state that outlives frames: a render snapshot's
//! restore (`Renderer::restore_state`) keeps it, as it keeps every upload.

use crate::clusters::LightTables;
use crate::material::Side;
use glam::Vec3;
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

/// The baked lighting a scene draws with ([`crate::RenderScene::baked`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedLighting {
    pub pages: Option<Arc<LightmapPages>>,
    /// Lightmapped meshes, by mesh index ([`crate::MeshHandle::index`]):
    /// one entry per vertex of the mesh.
    pub meshes: HashMap<usize, Arc<LightmapUvs>>,
    /// The first [`PROBE_VOLUMES`] are drawn.
    pub volumes: Vec<ProbeVolume>,
}

/// The most light volumes a frame samples.
pub const PROBE_VOLUMES: usize = 8;
/// The probe buffer's header: two vec4 per volume.
const HEADER: usize = 2 * PROBE_VOLUMES;
/// The probe buffer's size in vec4 on each tier.
fn probe_capacity(tables: LightTables) -> usize {
    match tables {
        // 16384 probes of three vec4.
        LightTables::Storage => HEADER + 3 * 16384,
        // One 16 KiB uniform.
        LightTables::Uniform => 1024,
    }
}

/// The WGSL of the probe buffer and the ambient-light hook (frame binding
/// 10), placed after lighting.wgsl in the lit shaders.
pub(crate) fn probe_wgsl(tables: LightTables) -> String {
    let space = match tables {
        LightTables::Storage => "storage, read",
        LightTables::Uniform => "uniform",
    };
    format!(
        "{}\n@group(0) @binding(10) var<{space}> probe_data: array<vec4<f32>, {n}>;\n{}",
        format_args!("const PROBE_VOLUMES: u32 = {PROBE_VOLUMES}u;"),
        AMBIENT_WGSL,
        n = probe_capacity(tables),
    )
}

const AMBIENT_WGSL: &str = r#"
// The baked lightmap texel of this fragment (fs_lightmap sets it; a: 1),
// irradiance over π.
var<private> baked_lightmap: vec4<f32>;

fn probe_irradiance(at: u32, n: vec3<f32>) -> vec3<f32> {
  let r = probe_data[at];
  let g = probe_data[at + 1u];
  let b = probe_data[at + 2u];
  return max(vec3<f32>(r.w + dot(r.xyz, n), g.w + dot(g.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0));
}

// The light volumes' irradiance over π at p for normal n (rgb) and how
// much of it to take (w: 0 outside every volume, 1 a spacing inside one).
fn baked_volume(p: vec3<f32>, n: vec3<f32>) -> vec4<f32> {
  var sum = vec3<f32>(0.0);
  var weight = 0.0;
  for (var i = 0u; i < PROBE_VOLUMES; i++) {
    let h0 = probe_data[2u * i];
    let h1 = probe_data[2u * i + 1u];
    if (h0.w < 0.0) { break; }
    let packed = u32(h1.w);
    let c = vec3<u32>(packed & 127u, (packed >> 7u) & 127u, packed >> 14u);
    let cmax = vec3<f32>(c - vec3<u32>(1u));
    let size = max(h1.xyz - h0.xyz, vec3<f32>(1e-4));
    let d = min(p - h0.xyz, h1.xyz - p) / (size / cmax);
    let w = clamp(min(d.x, min(d.y, d.z)), 0.0, 1.0);
    if (w <= 0.0) { continue; }
    let g = clamp((p - h0.xyz) / size * cmax, vec3<f32>(0.0), cmax);
    let g0 = min(vec3<u32>(floor(g)), c - vec3<u32>(2u));
    let f = g - vec3<f32>(g0);
    var irr = vec3<f32>(0.0);
    for (var k = 0u; k < 8u; k++) {
      let o = vec3<u32>(k & 1u, (k >> 1u) & 1u, k >> 2u);
      let q = g0 + o;
      let wk = select(1.0 - f.x, f.x, o.x == 1u) * select(1.0 - f.y, f.y, o.y == 1u) * select(1.0 - f.z, f.z, o.z == 1u);
      let idx = (q.z * c.y + q.y) * c.x + q.x;
      irr += probe_irradiance(u32(h0.w) + idx * 3u, n) * wk;
    }
    sum += irr * w;
    weight += w;
  }
  if (weight <= 0.0) { return vec4<f32>(0.0); }
  return vec4<f32>(sum / weight, min(weight, 1.0));
}

// The diffuse ambient light of a lit fragment, in the environment
// irradiance's units (the caller scales it by the environment intensity,
// which baked light does not take): its lightmap texel, else the light
// volumes around it blended over `env`, else `env`.
fn ambient_irradiance(env: vec3<f32>, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  let scale = 1.0 / max(frame.params.x, 1e-4);
  if (baked_lightmap.a > 0.5) { return baked_lightmap.rgb * scale; }
  let b = baked_volume(p, n);
  return mix(env, b.rgb * scale, b.w);
}
"#;

/// The frame layout's entry for the probe buffer.
pub(crate) fn probe_layout_entry(tables: LightTables) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 10,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: match tables {
                LightTables::Storage => wgpu::BufferBindingType::Storage { read_only: true },
                LightTables::Uniform => wgpu::BufferBindingType::Uniform,
            },
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
    let mut out = vec![[0.0, 0.0, 0.0, -1.0]; HEADER];
    for (i, v) in volumes.iter().take(PROBE_VOLUMES).enumerate() {
        let left = capacity.saturating_sub(out.len()) / 3;
        let Some((counts, probes)) = fit(v, left) else {
            break;
        };
        let first = out.len();
        out[2 * i] = v.min.extend(first as f32).to_array();
        let packed = counts[0] + (counts[1] << 7) + (counts[2] << 14);
        out[2 * i + 1] = v.max.extend(packed as f32).to_array();
        for p in probes {
            out.extend(p);
        }
    }
    out
}

/// A volume's probes within `room` probes: as baked, or trilinearly
/// resampled to the largest grid of its proportions that fits.
fn fit(v: &ProbeVolume, room: usize) -> Option<([u32; 3], Vec<Probe>)> {
    let c = v.counts.map(|n| n.clamp(2, 64));
    if (c[0] * c[1] * c[2]) as usize > v.probes.len() {
        return None;
    }
    let total = (c[0] * c[1] * c[2]) as usize;
    if total <= room {
        return Some((c, v.probes[..total].to_vec()));
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
    tables: LightTables,
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    /// The pages bound (by the scene's Arc) and their group.
    pages: Option<(*const LightmapPages, wgpu::BindGroup)>,
    /// A one-texel page for frames whose lightmapped meshes outlived theirs.
    empty: wgpu::BindGroup,
    /// Per lightmapped mesh index: its uvs (by Arc) and their buffer.
    uvs: HashMap<usize, (*const LightmapUvs, wgpu::Buffer)>,
    /// The lit pipelines of lightmapped batches: (side, blended, effects).
    pipelines: HashMap<(Side, bool, bool), wgpu::RenderPipeline>,
    /// Frame binding 10, and what was last written to it.
    pub probe_buffer: wgpu::Buffer,
    probes_key: Vec<(Vec3, Vec3, [u32; 3], *const Vec<Probe>)>,
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
        groups: [&wgpu::BindGroupLayout; 3],
        tables: LightTables,
    ) -> Lightmaps {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lightmap"),
            entries: &[
                crate::texture_entry(4, true, wgpu::TextureViewDimension::D2Array),
                crate::sampler_entry(5, wgpu::SamplerBindingType::Filtering),
            ],
        });
        let [frame, material, fx] = groups;
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr (lightmapped)"),
            bind_group_layouts: &[Some(frame), Some(material), Some(fx), Some(&layout)],
            immediate_size: 0,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lightmap"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let empty_pages = LightmapPages {
            width: 1,
            height: 1,
            pages: vec![vec![[0.0; 4]]],
        };
        let empty = Self::page_group(device, queue, &layout, &sampler, &empty_pages);
        let capacity = probe_capacity(tables);
        let probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light volumes"),
            size: (capacity * 16) as u64,
            usage: match tables {
                LightTables::Storage => wgpu::BufferUsages::STORAGE,
                LightTables::Uniform => wgpu::BufferUsages::UNIFORM,
            } | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &probe_buffer,
            0,
            bytemuck::cast_slice(&pack_probes(&[], capacity)),
        );
        Lightmaps {
            tables,
            layout,
            pipeline_layout,
            sampler,
            pages: None,
            empty,
            uvs: HashMap::new(),
            pipelines: HashMap::new(),
            probe_buffer,
            probes_key: Vec::new(),
        }
    }

    fn page_group(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        pages: &LightmapPages,
    ) -> wgpu::BindGroup {
        let layers = crate::gl_safe_array_layers(pages.pages.len().max(1) as u32);
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

    /// Uploads what changed in `baked` since the last frame.
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, baked: &BakedLighting) {
        let key = baked.pages.as_ref().map(Arc::as_ptr);
        if self.pages.as_ref().map(|(k, _)| *k) != key {
            self.pages = baked.pages.as_ref().map(|p| {
                (
                    Arc::as_ptr(p),
                    Self::page_group(device, queue, &self.layout, &self.sampler, p),
                )
            });
        }
        self.uvs
            .retain(|mesh, (k, _)| baked.meshes.get(mesh).is_some_and(|u| Arc::as_ptr(u) == *k));
        for (&mesh, uvs) in &baked.meshes {
            if self.uvs.contains_key(&mesh) {
                continue;
            }
            let buffer = wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("lightmap uvs"),
                    contents: bytemuck::cast_slice(if uvs.uvs.is_empty() {
                        &[[0.0f32; 3]]
                    } else {
                        &uvs.uvs[..]
                    }),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            );
            self.uvs.insert(mesh, (Arc::as_ptr(uvs), buffer));
        }
        let probes_key: Vec<_> = baked
            .volumes
            .iter()
            .take(PROBE_VOLUMES)
            .map(|v| (v.min, v.max, v.counts, Arc::as_ptr(&v.probes)))
            .collect();
        if probes_key != self.probes_key {
            let data = pack_probes(&baked.volumes, probe_capacity(self.tables));
            queue.write_buffer(&self.probe_buffer, 0, bytemuck::cast_slice(&data));
            self.probes_key = probes_key;
        }
    }

    /// The lightmap uv stream of a mesh drawn with its lightmap.
    pub fn uvs(&self, mesh: usize) -> Option<&wgpu::Buffer> {
        self.uvs.get(&mesh).map(|(_, b)| b)
    }

    /// The pages' group (a black page when the scene has none).
    pub fn group(&self) -> &wgpu::BindGroup {
        self.pages.as_ref().map_or(&self.empty, |(_, g)| g)
    }

    /// Makes the lightmapped pipeline for (side, blended, effects).
    pub fn ensure_pipeline(
        &mut self,
        device: &wgpu::Device,
        module: &wgpu::ShaderModule,
        key: (Side, bool, bool),
        samples: u32,
    ) {
        if self.pipelines.contains_key(&key) {
            return;
        }
        let (side, blend, effects) = key;
        let constants = [("EFFECTS", effects as u32 as f64)];
        let buffers = [
            Some(crate::VERTEX_LAYOUT),
            Some(crate::INSTANCE_LAYOUT),
            Some(LIGHTMAP_LAYOUT),
        ];
        let p = crate::hdr_pipeline_with(
            device,
            &constants,
            crate::HdrPipeline {
                label: "pbr lightmapped",
                layout: &self.pipeline_layout,
                module,
                vs: "vs_lightmap",
                fs: "fs_lightmap",
                buffers: &buffers,
                cull: crate::cull_mode(side),
                depth_write: !blend,
                depth_compare: wgpu::CompareFunction::Less,
                blend: blend.then_some(wgpu::BlendState::ALPHA_BLENDING),
                samples,
            },
        );
        self.pipelines.insert(key, p);
    }

    pub fn pipeline(&self, key: (Side, bool, bool)) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&key)
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
    fn resamples_a_volume_too_large_for_the_uniform() {
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
