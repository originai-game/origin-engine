//! Native `Tier::Full` bindless material table (`Caps::BINDLESS_TEXTURES |
//! Caps::VERTEX_STORAGE`, `shaders/pbr_bindless.wgsl`):
//! stores all standard opaque material uniforms + texture slot indices in one
//! storage buffer paired with a `binding_array<texture_2d<f32>, 1024>`, and a
//! per-instance `u32` material ID table read in `vs_bindless`. Standard opaque
//! batches bind ONE shared Group 1 across the entire pass and coalesce
//! multi-material draw runs through `multi_draw_indexed_indirect`.

use crate::material::MaterialUniform;
use crate::{Batch, Faces, GpuTexture};
use std::collections::HashMap;
use std::num::NonZeroU32;

/// Fixed capacity of `bindless_textures` in `pbr_bindless.wgsl` (slots 0 and 1
/// are `default_white` and `default_normal`; slots `2..1024` mirror
/// `cache.textures[0..1022]`).
pub(crate) const MAX_BINDLESS_TEXTURES: u32 = 1024;

const PBR_BINDLESS_WGSL: &str = wgsl!("pbr_bindless.wgsl");

/// GPU record matching `struct BindlessMaterial` in `bindless_lit_source`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BindlessMaterialGpu {
    pub factors: MaterialUniform,
    /// `[base, metalness, roughness, normal]` texture array indices.
    pub tex0: [u32; 4],
    /// `[emissive, ao, gradient, dissolve]` texture array indices.
    pub tex1: [u32; 4],
}

fn replace_once(text: &str, from: &str, to: &str, what: &str) -> String {
    assert!(
        text.contains(from),
        "bindless lit variant cannot find {what} in standard module"
    );
    text.replacen(from, to, 1)
}

/// Removes every `@vertex` / `@fragment` entry-point function from `src` so
/// the bindless module compiles only `vs_bindless` and `fs_bindless`.
fn strip_entry_points(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "@vertex" || trimmed == "@fragment" {
            i += 1;
            let mut depth = 0i32;
            let mut opened = false;
            while i < lines.len() {
                for ch in lines[i].chars() {
                    if ch == '{' {
                        depth += 1;
                        opened = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                i += 1;
                if opened && depth <= 0 {
                    break;
                }
            }
            continue;
        }
        out.push_str(lines[i]);
        out.push('\n');
        i += 1;
    }
    out
}

/// Removes a top-level WGSL function starting with `prefix` (e.g. `fn prepass_targets(`).
fn strip_fn(src: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_start().starts_with(prefix) {
            let mut depth = 0i32;
            let mut opened = false;
            while i < lines.len() {
                for ch in lines[i].chars() {
                    if ch == '{' {
                        depth += 1;
                        opened = true;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                i += 1;
                if opened && depth <= 0 {
                    break;
                }
            }
            continue;
        }
        out.push_str(lines[i]);
        out.push('\n');
        i += 1;
    }
    out
}

fn replace_ident(src: &str, ident: &str, replacement: &str) -> String {
    let is_id = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut pos = 0;
    while let Some(rel) = src[pos..].find(ident) {
        let start = pos + rel;
        let end = start + ident.len();
        let before_ok = start == 0 || !is_id(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_id(bytes[end]);
        out.push_str(&src[pos..start]);
        if before_ok && after_ok {
            out.push_str(replacement);
        } else {
            out.push_str(ident);
        }
        pos = end;
    }
    out.push_str(&src[pos..]);
    out
}

/// Transforms the standard lit module (`lit_source_with`) into its native
/// `Tier::Full` bindless material variant (`Caps::BINDLESS_TEXTURES |
/// Caps::VERTEX_STORAGE`), preserving bit-identical PBR shading via
/// `lit_fragment`.
pub(crate) fn bindless_lit_source(standard: &str) -> String {
    let mut s = format!(
        "enable wgpu_binding_array;\n{}",
        strip_entry_points(standard)
    );
    // Strip unused non-bindless helpers (OIT alpha mark, prepass, legacy sampler).
    for dead in [
        "fn material_sample(",
        "fn lit_alpha_mark(",
        "fn lightmapped_alpha_mark(",
        "fn prepass_targets(",
    ] {
        s = strip_fn(&s, dead);
    }
    s = replace_once(
        &s,
        "@group(1) @binding(0) var<uniform> material: MaterialFactors;",
        &format!(
            "struct BindlessMaterial {{\n\
               factors: MaterialFactors,\n\
               tex0: vec4<u32>,\n\
               tex1: vec4<u32>,\n\
             }};\n\
             @group(1) @binding(0) var<storage, read> bindless_materials: array<BindlessMaterial>;\n\
             @group(1) @binding(1) var bindless_textures: binding_array<texture_2d<f32>, {MAX_BINDLESS_TEXTURES}>;\n\
             @group(1) @binding(2) var material_sampler: sampler;\n\
             @group(1) @binding(3) var<storage, read> bindless_instance_materials: array<u32>;\n\
             var<private> material: MaterialFactors;\n\
             var<private> bindless_tex0: vec4<u32>;\n\
             var<private> bindless_tex1: vec4<u32>;"
        ),
        "material uniform",
    );
    for decl in [
        "@group(1) @binding(1) var base_color_tex: texture_2d<f32>;",
        "@group(1) @binding(2) var metalness_tex: texture_2d<f32>;",
        "@group(1) @binding(3) var roughness_tex: texture_2d<f32>;",
        "@group(1) @binding(4) var normal_tex: texture_2d<f32>;",
        "@group(1) @binding(5) var emissive_tex: texture_2d<f32>;",
        "@group(1) @binding(6) var ao_tex: texture_2d<f32>;",
        "@group(1) @binding(7) var material_sampler: sampler;",
        "@group(1) @binding(8) var gradient_tex: texture_2d<f32>;",
        "@group(1) @binding(9) var dissolve_tex: texture_2d<f32>;",
    ] {
        s = replace_once(&s, decl, "", decl);
    }
    s = replace_once(
        &s,
        "@group(1) @binding(10) var<uniform> stochastic_lookup: StochasticLookup;",
        "var<private> stochastic_lookup: StochasticLookup;",
        "stochastic_lookup",
    );
    // Replace material_sample(...) calls (which cannot take a binding_array element as a texture_2d parameter).
    for (from, to) in [
        (
            "material_sample(metalness_tex, UV_METALNESS, uv, 1u)",
            "bindless_sample(bindless_tex0.y, UV_METALNESS, uv)",
        ),
        (
            "material_sample(roughness_tex, UV_METALNESS, uv, 2u)",
            "bindless_sample(bindless_tex0.z, UV_METALNESS, uv)",
        ),
        (
            "material_sample(emissive_tex, UV_EMISSIVE, uv, 4u)",
            "bindless_sample(bindless_tex1.x, UV_EMISSIVE, uv)",
        ),
        (
            "material_sample(ao_tex, UV_AO, uv, 5u)",
            "bindless_sample(bindless_tex1.y, UV_AO, uv)",
        ),
    ] {
        assert!(
            s.contains(from),
            "bindless lit variant cannot find `{from}`"
        );
        s = s.replace(from, to);
    }
    // Replace whole-identifier texture references (base_color_tex, normal_tex, gradient_tex, dissolve_tex).
    s = replace_ident(&s, "base_color_tex", "bindless_textures[bindless_tex0.x]");
    s = replace_ident(&s, "normal_tex", "bindless_textures[bindless_tex0.w]");
    s = replace_ident(&s, "gradient_tex", "bindless_textures[bindless_tex1.z]");
    s = replace_ident(&s, "dissolve_tex", "bindless_textures[bindless_tex1.w]");
    // Bindless always runs with ANALYTIC = true and DISCARD = false (every
    // derivative is evaluated in uniform control flow at the top of fs_bindless
    // before per-instance material lookup). Remove unreachable non-analytic
    // derivative fallbacks inside material-dependent branches.
    for (from, to, what) in [
        (
            "    toon_aa = clamp(fwidthFine(n_dot_l), 1e-4, 0.25) * px;",
            "    toon_aa = 0.01 * px;",
            "toon_edges fallback",
        ),
        (
            "fn world_dx(p: vec3<f32>) -> vec3<f32> {\n  if (ANALYTIC) { return analytic_dx; }\n  return dpdxFine(p);\n}",
            "fn world_dx(p: vec3<f32>) -> vec3<f32> {\n  return analytic_dx;\n}",
            "world_dx",
        ),
        (
            "fn world_dy(p: vec3<f32>) -> vec3<f32> {\n  if (ANALYTIC) { return analytic_dy; }\n  return dpdyFine(p);\n}",
            "fn world_dy(p: vec3<f32>) -> vec3<f32> {\n  return analytic_dy;\n}",
            "world_dy",
        ),
        (
            "base_sample = base_texel(base_uv, dpdxFine(base_uv), dpdyFine(base_uv));",
            "base_sample = vec4<f32>(1.0);",
            "base_sample fallback",
        ),
        (
            "coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);",
            "coverage = 1.0;",
            "a2c coverage fallback",
        ),
        (
            "normal_sample = normal_texel(normal_uv, dpdxFine(normal_uv), dpdyFine(normal_uv));",
            "normal_sample = vec4<f32>(0.5, 0.5, 1.0, 1.0);",
            "normal_sample fallback",
        ),
        (
            "coarse_sample = textureSampleBias(bindless_textures[bindless_tex0.x], material_sampler, base_uv, 6.0);",
            "coarse_sample = base_sample;",
            "coarse_sample fallback",
        ),
        (
            "footprint = max(length(dpdxFine(in.local_pos)), length(dpdyFine(in.local_pos)));",
            "footprint = 0.0;",
            "local_pos footprint fallback",
        ),
        (
            "dh = vec2<f32>(dpdxFine(hm), dpdyFine(hm));",
            "dh = vec2<f32>(0.0);",
            "material_surface dh fallback",
        ),
        (
            "weathered = weather_surface_at(s, n, geo_n, in.world_pos, weather_cavity(uv), material.uv[3].w, select(WEATHER_STANDS, WEATHER_MOVES, in.moves != 0u));",
            "weathered = Weathered();",
            "weather_surface_at fallback",
        ),
        (
            "s.roughness = specular_aa(n, s.roughness, dpdxFine(n), dpdyFine(n));",
            "s.roughness = s.roughness;",
            "specular_aa fallback",
        ),
    ] {
        s = replace_once(&s, from, to, what);
    }
    // Add flat mat_id at free location 7 of VertexOut.
    s = replace_once(
        &s,
        "struct VertexOut {\n  @builtin(position) clip: vec4<f32>,\n",
        "struct VertexOut {\n  @builtin(position) clip: vec4<f32>,\n  @location(7) @interpolate(flat, either) mat_id: u32,\n",
        "VertexOut",
    );
    s.push_str(PBR_BINDLESS_WGSL);
    s
}

/// Shared bindless material table, texture binding array, instance material ID
/// buffer, and lazily built `(Faces, effects)` opaque lit pipelines.
pub(crate) struct BindlessTable {
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    module: crate::deferred::Module,
    pipelines: HashMap<(Faces, bool), wgpu::RenderPipeline>,
    materials_buf: wgpu::Buffer,
    instance_mats_buf: wgpu::Buffer,
    instance_mats_scratch: Vec<u32>,
    instance_mats_key: Option<(u64, usize)>,
    bind_group: Option<wgpu::BindGroup>,
    dirty_group: bool,
}

impl BindlessTable {
    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        fx_layout: &wgpu::BindGroupLayout,
        bindless_wgsl: String,
    ) -> BindlessTable {
        let storage_ro =
            |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pbr bindless"),
            entries: &[
                storage_ro(0, wgpu::ShaderStages::FRAGMENT),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(MAX_BINDLESS_TEXTURES),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                storage_ro(3, wgpu::ShaderStages::VERTEX),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr bindless"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout), Some(fx_layout)],
            immediate_size: 0,
        });
        let module = crate::deferred::module(device, "pbr bindless", bindless_wgsl);
        let make_buf = |label: &str, size: u64| {
            crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size: size.max(16),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            )
        };
        BindlessTable {
            layout,
            pipeline_layout,
            module,
            pipelines: HashMap::new(),
            materials_buf: make_buf(
                "bindless materials",
                std::mem::size_of::<BindlessMaterialGpu>() as u64,
            ),
            instance_mats_buf: make_buf("bindless instance materials", 64),
            instance_mats_scratch: Vec::new(),
            instance_mats_key: None,
            bind_group: None,
            dirty_group: true,
        }
    }

    pub fn clear_pipelines(&mut self) {
        self.pipelines.clear();
    }

    pub fn mark_dirty(&mut self) {
        self.dirty_group = true;
    }

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    pub fn pipeline(&self, faces: Faces, effects: bool) -> Option<&wgpu::RenderPipeline> {
        self.pipelines.get(&(faces, effects))
    }

    /// Uploads the flat `BindlessMaterialGpu` table when materials changed.
    pub fn sync_materials(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        records: &[BindlessMaterialGpu],
    ) {
        if records.is_empty() {
            return;
        }
        let bytes = std::mem::size_of_val(records) as u64;
        if bytes > self.materials_buf.size() {
            self.materials_buf = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("bindless materials"),
                    size: bytes.next_power_of_two().max(bytes),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );
            self.dirty_group = true;
        }
        crate::traffic::write_buffer(queue, &self.materials_buf, 0, bytemuck::cast_slice(records));
    }

    /// Updates the per-sorted-instance material ID table (`bindless_instance_materials`).
    pub fn sync_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        serial: u64,
        instance_count: usize,
        batches: &[Batch],
    ) {
        let key = (serial, instance_count);
        if self.instance_mats_key == Some(key) {
            return;
        }
        self.instance_mats_scratch.clear();
        self.instance_mats_scratch.resize(instance_count.max(1), 0);
        for b in batches {
            let start = b.first as usize;
            let end = (start + b.count as usize).min(self.instance_mats_scratch.len());
            if start < end {
                self.instance_mats_scratch[start..end].fill(b.material as u32);
            }
        }
        let bytes = (self.instance_mats_scratch.len() * 4) as u64;
        if bytes > self.instance_mats_buf.size() {
            self.instance_mats_buf = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("bindless instance materials"),
                    size: bytes.next_power_of_two().max(bytes),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );
            self.dirty_group = true;
        }
        crate::traffic::write_buffer(
            queue,
            &self.instance_mats_buf,
            0,
            bytemuck::cast_slice(&self.instance_mats_scratch),
        );
        self.instance_mats_key = Some(key);
    }

    /// Rebuilds the shared Group 1 texture binding array when textures or buffers changed.
    pub fn ensure_bind_group(
        &mut self,
        device: &wgpu::Device,
        default_white: &wgpu::TextureView,
        default_normal: &wgpu::TextureView,
        textures: &[GpuTexture],
        sampler: &wgpu::Sampler,
    ) {
        if !self.dirty_group && self.bind_group.is_some() {
            return;
        }
        let cap = MAX_BINDLESS_TEXTURES as usize;
        let mut views: Vec<&wgpu::TextureView> = Vec::with_capacity(cap);
        views.push(default_white);
        views.push(default_normal);
        for t in textures.iter().take(cap - 2) {
            views.push(&t.view);
        }
        views.resize(cap, default_white);
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pbr bindless"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.materials_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureViewArray(&views),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.instance_mats_buf.as_entire_binding(),
                },
            ],
        }));
        self.dirty_group = false;
    }

    /// Lazily builds bindless opaque lit pipelines for the `(faces, effects)` seen in `batches`.
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        batches: &[Batch],
        materials: &[crate::GpuMaterial],
        samples: u32,
    ) {
        let buffers = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        for b in batches {
            if b.transparent
                || b.physical
                || b.array
                || b.compact
                || b.shader.is_some()
                || b.dissolve
                || b.reveal
                || !materials.get(b.material).is_some_and(|m| m.bindless_ok)
            {
                continue;
            }
            let key = (b.faces, b.effects);
            if self.pipelines.contains_key(&key) {
                continue;
            }
            let pipe = crate::hdr_pipeline_facing(
                device,
                &[
                    ("EFFECTS", b.effects as u32 as f64),
                    ("ANALYTIC", 1.0),
                    ("DISCARD", 0.0),
                ],
                b.faces.front(),
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label: "pbr bindless",
                    layout: &self.pipeline_layout,
                    module: self.module.get(),
                    vs: "vs_bindless",
                    fs: "fs_bindless",
                    buffers: &buffers,
                    cull: b.faces.cull(),
                    depth_write: true,
                    depth_compare: crate::LIT_DEPTH_COMPARE,
                    blend: None,
                    samples,
                },
            );
            self.pipelines.insert(key, pipe);
        }
    }
}
