use glam::{Vec2, Vec3};
pub use og_render_data::material::*;

/// `struct MaterialFactors` in pbr.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialUniform {
    /// rgb: colour, a: opacity.
    pub base_color: [f32; 4],
    /// x: metalness, y: roughness, zw: normal scale.
    pub mr: [f32; 4],
    /// rgb: emissive x intensity, w: alpha cutoff (< 0: none).
    pub emissive: [f32; 4],
    /// x: normal map (0 none, 1 RGB, 2 two-channel), y: ao intensity (< 0: no
    /// map), z: environment intensity, w: mode bits (1 flat, 2 unlit, 4
    /// blended, 8 unstyled: no shadow tint or outline distance, 16 alpha to
    /// coverage, 32 single-sided: a closed surface, whose sun shadows
    /// compare past its own depth, lighting.wgsl `shadow_closed`).
    pub flags: [f32; 4],
    /// x: toon (0 off, 1 gradient map, 2 built-in bands), y: rim intensity,
    /// z: rim power, w: flash amount.
    pub toon: [f32; 4],
    /// rgb: rim colour, w: dissolve amount.
    pub rim: [f32; 4],
    /// rgb: flash colour, w: dissolve edge width.
    pub flash: [f32; 4],
    /// rgb: dissolve edge colour, w: noise scale (negative: the dissolve map).
    pub dissolve: [f32; 4],
    /// rgb: outline colour, w: outline width in pixels.
    pub outline: [f32; 4],
    /// x: clearcoat, y: clearcoat roughness, zw: leaf translucency/dome normals.
    pub coat: [f32; 4],
    /// Each slot's uv transform ([`StandardMaterial::uv_transforms`]'s
    /// order) as two rows: `[a, b, c, d]` (`u' = a u + b v`, `v' = c u +
    /// d v`) and `[tx, ty, 0, 0]`. Array materials use the base-map
    /// translation row's spare zw for their base/normal array layers;
    /// `uv[3].z` (the metalness slot's spare) holds `paintDetail`, `uv[3].w`
    /// `weathering`.
    pub uv: [[f32; 4]; 10],
    /// A material shader's parameters (`hooks.rs`).
    pub custom: [[f32; 4]; crate::MAX_SHADER_PARAMS],
    /// The parameters the frames before `stamp`'s drew, for vertex motion
    /// vectors in that frame.
    pub previous_custom: [[f32; 4]; crate::MAX_SHADER_PARAMS],
    /// x: the drawn frame `custom` was written in (`FrameUniform::drawn`):
    /// `previous_shader_params()` reads `previous_custom` in that frame
    /// only (frame.wgsl `params_moved`); yzw unused.
    pub stamp: [u32; 4],
}

/// The tail of a [`MaterialUniform`] a shader parameter change writes in
/// one call (engine s98, s100): the new parameters, the ones the frames
/// before drew and the frame they change in. Nothing needs writing on the
/// frames after, paused or not: their number is not the stamp.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ParamsChange {
    pub custom: [[f32; 4]; crate::MAX_SHADER_PARAMS],
    pub previous_custom: [[f32; 4]; crate::MAX_SHADER_PARAMS],
    pub stamp: [u32; 4],
}

// `previous_custom` directly follows `custom`, and `stamp` it, at the end
// of the uniform: a [`ParamsChange`] written at `custom`'s offset.
const _: () = assert!(
    std::mem::offset_of!(MaterialUniform, previous_custom)
        == std::mem::offset_of!(MaterialUniform, custom)
            + std::mem::size_of::<[[f32; 4]; crate::MAX_SHADER_PARAMS]>()
        && std::mem::offset_of!(MaterialUniform, stamp)
            == std::mem::offset_of!(MaterialUniform, previous_custom)
                + std::mem::size_of::<[[f32; 4]; crate::MAX_SHADER_PARAMS]>()
        && std::mem::size_of::<MaterialUniform>()
            == std::mem::offset_of!(MaterialUniform, custom) + std::mem::size_of::<ParamsChange>()
);

/// `m` with the scene's [`crate::MaterialStyle`] filled into every stylised
/// field the material leaves at its own default: toon and the ramp, the
/// rim light, the outline, flat facets. An unlit material keeps its look,
/// and so does one with `styled` 0 (its uniform also turns the frame's
/// shadow tint and outline distance off, [`uniform`]). The style's toon
/// passes by a material with a physical lobe
/// ([`StandardMaterial::has_physical_lobes`]: a clear coat, sheen,
/// transmission, iridescence, anisotropy, an `ior` other than 1.5), which
/// toon shading would drop: it keeps its physical shading and takes the
/// rest of the style. Its own `toon` still makes it toon.
pub(crate) fn styled(m: &StandardMaterial, style: &crate::MaterialStyle) -> StandardMaterial {
    let mut s = *m;
    if m.unlit != 0 || m.styled == 0 || !style.is_active() {
        return s;
    }
    if style.toon && s.toon == 0 && !m.has_physical_lobes() {
        s.toon = 1;
    }
    if s.toon != 0
        && s.gradientMap == 0
        && let Some(t) = style.gradient_map
    {
        s.gradientMap = t.slot();
    }
    if style.rim_intensity > 0.0 && s.rimIntensity <= 0.0 {
        s.rimIntensity = style.rim_intensity;
        s.rimColor = Vec3::from(style.rim_color);
        s.rimPower = style.rim_power.max(0.1);
    }
    if style.outline_width > 0.0 && s.outlineWidth <= 0.0 {
        s.outlineWidth = style.outline_width.clamp(0.0, 31.0);
        s.outlineColor = Vec3::from(style.outline_color);
    }
    if style.flat_shading && s.flatShading == 0 {
        s.flatShading = 1;
    }
    if style.wireframe && s.wireframe == 0 {
        // Over its triangles (2), not in their place: a look's vector
        // lines with the hidden ones hidden, as Battlezone and Tron drew
        // theirs. Edges alone left the scene see-through and empty (the
        // neon-vector sample: hills and sky gone, every back edge drawn).
        s.wireframe = 2;
        s.wireframeAngle = style.wireframe_angle.max(0.0);
    }
    if style.wireframe && s.wireframeColor == Vec3::ZERO {
        s.wireframeColor = Vec3::from(style.wireframe_color);
    }
    s
}

/// The uniform for `m`; `two_channel_normal` when its normal map is BC5/EAC.
pub(crate) fn uniform(m: &StandardMaterial, two_channel_normal: bool) -> MaterialUniform {
    let has_normal = m.normalMap != 0;
    let cutoff = if m.alphaTest > 0.0 { m.alphaTest } else { -1.0 };
    let mode = (m.flatShading != 0) as u32
        | ((m.unlit != 0) as u32) << 1
        | (m.is_transparent() as u32) << 2
        | ((m.styled == 0) as u32) << 3
        | ((m.alphaToCoverage != 0 && m.alphaTest > 0.0) as u32) << 4
        | ((m.side() != Side::Double) as u32) << 5;
    let scale = if has_normal { m.normalScale } else { Vec2::ONE };
    MaterialUniform {
        base_color: [m.color.x, m.color.y, m.color.z, m.opacity],
        mr: [m.metalness, m.roughness, scale.x, scale.y],
        emissive: [
            m.emissive.x * m.emissiveIntensity,
            m.emissive.y * m.emissiveIntensity,
            m.emissive.z * m.emissiveIntensity,
            cutoff,
        ],
        flags: [
            match (has_normal, two_channel_normal) {
                (false, _) => 0.0,
                (true, false) => 1.0,
                (true, true) => 2.0,
            },
            if m.aoMap != 0 { m.aoMapIntensity } else { -1.0 },
            m.envMapIntensity,
            mode as f32,
        ],
        toon: [
            match (m.toon != 0, m.gradientMap != 0) {
                (false, _) => 0.0,
                (true, true) => 1.0,
                (true, false) => 2.0,
            },
            m.rimIntensity,
            m.rimPower.max(0.1),
            m.flash.clamp(0.0, 1.0),
        ],
        rim: [
            m.rimColor.x,
            m.rimColor.y,
            m.rimColor.z,
            m.dissolve.clamp(0.0, 1.0),
        ],
        flash: [
            m.flashColor.x,
            m.flashColor.y,
            m.flashColor.z,
            m.dissolveEdgeWidth.max(0.0),
        ],
        dissolve: [
            m.dissolveEdgeColor.x,
            m.dissolveEdgeColor.y,
            m.dissolveEdgeColor.z,
            if m.dissolveMap != 0 {
                -1.0
            } else {
                m.dissolveScale.max(1e-3)
            },
        ],
        outline: [
            m.outlineColor.x,
            m.outlineColor.y,
            m.outlineColor.z,
            m.outlineWidth.clamp(0.0, 31.0),
        ],
        coat: [
            m.clearcoat.clamp(0.0, 1.0),
            m.clearcoatRoughness.clamp(0.0, 1.0),
            m.foliage.x.clamp(0.0, 1.0),
            m.foliage.y.clamp(0.0, 1.0),
        ],
        uv: {
            let mut uv = uv_rows(&m.uv_transforms());
            uv[3][2] = m.paintDetail.clamp(0.0, 1.0);
            uv[3][3] = m.weathering.clamp(0.0, 1.0);
            uv
        },
        custom: [[0.0; 4]; crate::MAX_SHADER_PARAMS],
        previous_custom: [[0.0; 4]; crate::MAX_SHADER_PARAMS],
        stamp: [0; 4],
    }
}

/// [`MaterialUniform::uv`] of `t`.
fn uv_rows(t: &[UvTransform; 5]) -> [[f32; 4]; 10] {
    let mut out = [[0.0; 4]; 10];
    for (i, t) in t.iter().enumerate() {
        let [r0, r1] = t.matrix();
        out[2 * i] = [r0[0], r0[1], r1[0], r1[1]];
        out[2 * i + 1] = [r0[2], r1[2], 0.0, 0.0];
    }
    out
}

// ── array maps ───────────────────────────────────────────────────────────

/// The array-map pipelines' WGSL: `pbr_array.wgsl` and `shadow_array.wgsl`.
const PBR_ARRAY_WGSL: &str = wgsl!("pbr_array.wgsl");
const SHADOW_ARRAY_WGSL: &str = wgsl!("shadow_array.wgsl");

/// `text` with `from` replaced by `to`, which must occur: a variant built
/// from the standard module's text fails loudly (the shader tests build
/// every module) when that text moves, never silently.
fn replace_once(text: &str, from: &str, to: &str, what: &str) -> String {
    assert!(
        text.contains(from),
        "the array-map variant cannot find {what} in the standard module"
    );
    text.replacen(from, to, 1)
}

/// Every `textureSample(<texture>, material_sampler, <coords>)` in `text`
/// with `layer` as its array index.
fn sample_layer(text: &str, texture: &str, layer: &str) -> String {
    let call = format!("textureSample({texture}, material_sampler, ");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut found = 0;
    while let Some(at) = rest.find(&call) {
        let args = at + call.len();
        let close = args + rest[args..].find(')').expect("a closing parenthesis");
        out.push_str(&rest[..close]);
        out.push_str(", ");
        out.push_str(layer);
        rest = &rest[close..];
        found += 1;
    }
    out.push_str(rest);
    assert!(found > 0, "the standard module samples {texture} nowhere");
    // `textureSampleBias(<texture>, material_sampler, <coords>, <bias>)`
    // (the material style's surface) and `textureSampleGrad(<texture>,
    // material_sampler, <coords>, <ddx>, <ddy>)` (the visibility buffer's
    // resolve, pbr_fs.wgsl `ANALYTIC`): the layer goes after the coords,
    // which hold no comma.
    for kind in ["Bias", "Grad"] {
        let call = format!("textureSample{kind}({texture}, material_sampler, ");
        let text = std::mem::take(&mut out);
        let mut rest = text.as_str();
        while let Some(at) = rest.find(&call) {
            let args = at + call.len();
            let comma = args
                + rest[args..]
                    .find(',')
                    .expect("an argument after the coords");
            out.push_str(&rest[..comma]);
            out.push_str(", ");
            out.push_str(layer);
            rest = &rest[comma..];
        }
        out.push_str(rest);
    }
    out
}

/// The standard lit module's source (`lit_source`, the built-in hooks) as
/// its array-map variant: the base colour (binding 1) and the normal map
/// (binding 4) become 2D arrays sampled at the fragment's layers, the
/// vertex outputs carry the instance's layer, flat, and the entry points
/// `vs`, `fs`, `vs_prepass` and `fs_prepass` are `pbr_array.wgsl`'s (the
/// skinned ones are unchanged: a skinned instance takes its material's
/// layers). Nothing else changes, so the pixels are the standard path's
/// where the layer holds the same image.
pub(crate) fn array_lit_source(standard: &str) -> String {
    let mut s = standard.to_string();
    for (binding, name) in [(1, "base_color_tex"), (4, "normal_tex")] {
        s = replace_once(
            &s,
            &format!("@group(1) @binding({binding}) var {name}: texture_2d<f32>;"),
            &format!("@group(1) @binding({binding}) var {name}: texture_2d_array<f32>;"),
            name,
        );
    }
    s = sample_layer(&s, "base_color_tex", "map_layer");
    s = sample_layer(&s, "normal_tex", "normal_layer");
    for out in ["VertexOut", "PrepassOut"] {
        s = replace_once(
            &s,
            &format!("struct {out} {{\n  @builtin(position) clip: vec4<f32>,\n"),
            &format!(
                "struct {out} {{\n  @builtin(position) clip: vec4<f32>,\n  // The instance's texture-array layer plus one (0: the material's).\n  @location(7) @interpolate(flat) layer: u32,\n"
            ),
            out,
        );
    }
    for (stage, signature) in [
        (
            "@vertex",
            "vs(v: VertexIn, i: InstanceIn, k: InstanceFxIn) -> VertexOut {",
        ),
        (
            "@fragment",
            "fs(in: VertexOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {",
        ),
        (
            "@fragment",
            "fs_oit(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {",
        ),
        (
            "@fragment",
            "fs_oit_list(in: VertexOut, @builtin(front_facing) front_facing: bool, @builtin(sample_mask) mask: u32) -> OitOutput {",
        ),
        (
            "@fragment",
            "fs_oit_front(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {",
        ),
        (
            "@fragment",
            "fs_oit_mark(in: VertexOut, @builtin(front_facing) front_facing: bool) {",
        ),
        (
            "@vertex",
            "vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn, k: InstanceFxIn) -> PrepassOut {",
        ),
        (
            "@fragment",
            "fs_prepass(in: PrepassOut, @builtin(front_facing) front_facing: bool) -> PrepassTargets {",
        ),
    ] {
        let (name, rest) = signature.split_once('(').expect("a signature");
        s = replace_once(
            &s,
            &format!("{stage}\nfn {signature}"),
            &format!("{stage}\nfn {name}_standard({rest}"),
            name,
        );
    }
    s.push_str(PBR_ARRAY_WGSL);
    s
}

/// The shadow module's source (`shadow_source`, the built-in hooks) as its
/// array-map variant: `vs` and `vs_lean` zero the layer an array-map
/// instance carries in its transform (`shadow_array.wgsl`).
pub(crate) fn array_shadow_source(standard: &str) -> String {
    let mut s = standard.to_string();
    for signature in [
        "vs(v: VertexIn, i: InstanceIn) -> VsOut {",
        "vs_lean(@location(0) position: vec3<f32>, i: InstanceIn) -> LeanVsOut {",
    ] {
        let (name, rest) = signature.split_once('(').expect("a signature");
        s = replace_once(
            &s,
            &format!("@vertex\nfn {signature}"),
            &format!("@vertex\nfn {name}_standard({rest}"),
            name,
        );
    }
    s.push_str(SHADOW_ARRAY_WGSL);
    s
}

/// The uniform `u` of an array-map material `m`: its normal map is the
/// array's (`two_channel`: a BC5/EAC array), and `uv[1].zw` hold `mapLayer`
/// and `normalMapLayer` (`array_layers` in pbr_array.wgsl).
pub(crate) fn array_uniform(u: &mut MaterialUniform, m: &StandardMaterial, two_channel: bool) {
    let has_normal = m.normalMapArray != 0;
    let scale = if has_normal { m.normalScale } else { Vec2::ONE };
    u.mr[2] = scale.x;
    u.mr[3] = scale.y;
    u.flags[0] = match (has_normal, two_channel) {
        (false, _) => 0.0,
        (true, false) => 1.0,
        (true, true) => 2.0,
    };
    // Exact as f32 up to 2^24 layers.
    u.uv[1][2] = m.mapLayer.min(1 << 24) as f32;
    u.uv[1][3] = m.normalMapLayer.min(1 << 24) as f32;
}

/// What an instance of an array-map material carries beside its transform
/// (`instances::GpuTransform::layer`): its layer plus one (`None`: 0, the
/// material's layers).
pub(crate) fn instance_layer_bits(layer: Option<u32>) -> f32 {
    layer.map_or(0.0, |l| l.min(1 << 24) as f32 + 1.0)
}

/// The array-map variant of the standard lit, prepass and shadow
/// pipelines, made when a scene first draws a material with array maps
/// ([`StandardMaterial::draws_array_maps`]): one material group layout
/// whose base colour and normal map are 2D arrays, the variant modules,
/// and the pipelines by the standard keys, made at first use. One lit and
/// one prepass pipeline for a typical scene, one shadow pipeline per
/// (side, dissolve).
pub(crate) struct ArrayMaps {
    /// Group 1: the standard material group with bindings 1 (base colour)
    /// and 4 (normal map) as `D2Array`.
    pub layout: wgpu::BindGroupLayout,
    /// Static and skinned, as the standard lit and prepass layouts.
    pub layouts: (wgpu::PipelineLayout, wgpu::PipelineLayout),
    pub module: wgpu::ShaderModule,
    pub shadow_module: wgpu::ShaderModule,
    /// By (side, blended, skinned, effects).
    pub lit:
        std::collections::HashMap<crate::PbrKey, crate::deferred::Deferred<wgpu::RenderPipeline>>,
    /// By (side, skinned).
    pub prepass:
        std::collections::HashMap<(Side, bool), crate::deferred::Deferred<wgpu::RenderPipeline>>,
    /// Unskinned casters by (side, dissolve) (skinned instances carry no
    /// layer: the standard shadow pipelines draw them).
    pub shadow:
        std::collections::HashMap<(Side, bool), crate::deferred::Deferred<crate::ShadowSet>>,
    /// One-layer stand-ins for a missing array: white, and the flat normal.
    pub white: wgpu::TextureView,
    pub flat_normal: wgpu::TextureView,
}

impl ArrayMaps {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        groups: [&wgpu::BindGroupLayout; 3],
        lit_source: String,
        shadow_source: String,
    ) -> ArrayMaps {
        let [frame_layout, fx_layout, palette_layout] = groups;
        let array =
            |binding| crate::texture_entry(binding, true, wgpu::TextureViewDimension::D2Array);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material (array maps)"),
            entries: &[
                crate::uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                array(1),
                crate::tex2d(2),
                crate::tex2d(3),
                array(4),
                crate::tex2d(5),
                crate::tex2d(6),
                crate::sampler_entry(7, wgpu::SamplerBindingType::Filtering),
                crate::tex2d(8),
                crate::tex2d(9),
            ],
        });
        let pipeline_layout = |label, groups: &[&wgpu::BindGroupLayout]| {
            let groups: Vec<_> = groups.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &groups,
                immediate_size: 0,
            })
        };
        let layouts = (
            pipeline_layout("pbr (array maps)", &[frame_layout, &layout, fx_layout]),
            pipeline_layout(
                "pbr skinned (array maps)",
                &[frame_layout, &layout, fx_layout, palette_layout],
            ),
        );
        let solid = |label, rgba: [u8; 4]| {
            wgpu::util::DeviceExt::create_texture_with_data(
                device,
                queue,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &rgba,
            )
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            })
        };
        ArrayMaps {
            layout,
            layouts,
            module: crate::shader(device, "pbr (array maps)", array_lit_source(&lit_source)),
            shadow_module: crate::shader(
                device,
                "shadow (array maps)",
                array_shadow_source(&shadow_source),
            ),
            lit: Default::default(),
            prepass: Default::default(),
            shadow: Default::default(),
            white: solid("array maps: white", [255; 4]),
            flat_normal: solid("array maps: flat normal", [128, 128, 255, 255]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The array-map module is the standard one with its base colour and
    /// normal map as arrays, every sample of them at the fragment's layer,
    /// the layer handed on flat, and the four entry points replaced.
    #[test]
    fn the_array_variant_samples_every_map_at_its_layer() {
        let standard = crate::lit_source(&crate::hooks::default_hooks());
        let array = array_lit_source(&standard);
        for decl in [
            "var base_color_tex: texture_2d_array<f32>;",
            "var normal_tex: texture_2d_array<f32>;",
            "@location(7) @interpolate(flat) layer: u32,",
            "fn vs_standard(",
            "fn fs_prepass_standard(",
        ] {
            assert!(array.contains(decl), "{decl}");
        }
        assert_eq!(array.matches("@interpolate(flat) layer: u32").count(), 2);
        for (texture, layer) in [
            ("base_color_tex", "map_layer"),
            ("normal_tex", "normal_layer"),
        ] {
            let samples = standard
                .matches(&format!("textureSample({texture}, "))
                .count();
            assert!(samples > 0);
            let layered = array
                .split(&format!("textureSample({texture}, material_sampler, "))
                .skip(1)
                .filter(|rest| rest.split(')').next().unwrap().ends_with(layer))
                .count();
            assert_eq!(layered, samples, "{texture}");
        }
        // The shadow variant renames its two plain vertex stages only.
        let shadow = array_shadow_source(&crate::shadow_source(&crate::hooks::default_hooks()));
        assert!(shadow.contains("fn vs_standard(") && shadow.contains("fn vs_lean_standard("));
        assert_eq!(shadow.matches("\nfn vs(").count(), 1);
    }

    /// material.wgsl's `MaterialFactors` is laid out as [`MaterialUniform`]:
    /// the same size and the parameter tail at the same offsets, so the
    /// shaders read the stamp a [`ParamsChange`] writes (engine s100).
    #[test]
    fn the_wgsl_material_is_the_rust_material() {
        let module = naga::front::wgsl::parse_str(crate::MATERIAL_WGSL).expect("material.wgsl");
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("layout");
        let (handle, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some("MaterialFactors"))
            .expect("struct MaterialFactors");
        assert_eq!(
            layouter[handle].size as usize,
            std::mem::size_of::<MaterialUniform>()
        );
        let naga::TypeInner::Struct { members, .. } = &ty.inner else {
            panic!("MaterialFactors must be a struct")
        };
        for (name, offset) in [
            ("custom", std::mem::offset_of!(MaterialUniform, custom)),
            (
                "previous_custom",
                std::mem::offset_of!(MaterialUniform, previous_custom),
            ),
            ("stamp", std::mem::offset_of!(MaterialUniform, stamp)),
        ] {
            let m = members
                .iter()
                .find(|m| m.name.as_deref() == Some(name))
                .expect(name);
            assert_eq!(m.offset as usize, offset, "{name}");
        }
    }

    #[test]
    fn array_maps_set_the_layers_and_the_normal_kind() {
        let m = StandardMaterial {
            mapArray: 3,
            mapLayer: 5,
            normalMapArray: 4,
            normalMapLayer: 2,
            normalScale: Vec2::new(0.5, 2.0),
            mapOffset: Vec2::new(0.25, -0.75),
            foliage: Vec2::new(0.3, 0.8),
            ..Default::default()
        };
        assert!(m.draws_array_maps());
        let mut u = uniform(&m, false);
        // Without a 2D normal map the standard uniform has none.
        assert_eq!((u.flags[0], u.mr[2]), (0.0, 1.0));
        array_uniform(&mut u, &m, true);
        assert_eq!(u.flags[0], 2.0);
        assert_eq!(
            [u.mr[2], u.mr[3], u.uv[1][2], u.uv[1][3]],
            [0.5, 2.0, 5.0, 2.0]
        );
        assert_eq!(u.uv[1][..2], [0.25, -0.75]);
        assert_eq!(u.coat[2..], [0.3, 0.8]);
        // Array layers must not turn ordinary surfaces into translucent leaves.
        let plain = StandardMaterial {
            foliage: Vec2::ZERO,
            ..m
        };
        let mut plain_uniform = uniform(&plain, false);
        array_uniform(&mut plain_uniform, &plain, true);
        assert_eq!(plain_uniform.coat[2..], [0.0, 0.0]);
        // A material without arrays keeps the spare UV lanes at zero.
        assert_eq!(
            uniform(&StandardMaterial::default(), false).uv[1][2..],
            [0.0, 0.0]
        );
        // Physical materials keep their own path.
        let physical = StandardMaterial { sheen: 1.0, ..m };
        assert!(!physical.draws_array_maps());
        assert_eq!(instance_layer_bits(None), 0.0);
        assert_eq!(instance_layer_bits(Some(6)), 7.0);
    }

    /// Removing a texture array forgets it in every material that samples
    /// it (and bumps their revisions), as removing a texture does.
    #[test]
    fn a_removed_array_is_forgotten_by_its_materials() {
        let mut scene = crate::RenderScene::new();
        let layer = crate::TextureSource::Rgba(crate::TextureData {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
            color_space: crate::ColorSpace::Srgb,
            mips: crate::Mips::None,
        });
        let array = scene
            .add_texture_array(crate::TextureArrayData {
                layers: vec![layer.clone(), layer],
            })
            .unwrap();
        assert_eq!(
            crate::TextureArrayHandle::from_slot(array.slot()),
            Some(array)
        );
        let h = scene.add_standard_material(StandardMaterial {
            mapArray: array.slot(),
            normalMapArray: array.slot(),
            ..Default::default()
        });
        assert!(scene.remove_texture_array(array));
        let m = scene.standard_material(h).unwrap();
        assert_eq!((m.mapArray, m.normalMapArray), (0, 0));
    }
}

#[cfg(test)]
mod style_tests {
    use super::*;

    fn toon_style() -> crate::MaterialStyle {
        crate::MaterialStyle {
            toon: true,
            rim_intensity: 0.5,
            rim_color: [1.0, 0.9, 0.8],
            rim_power: 3.0,
            outline_width: 2.0,
            flat_shading: true,
            ..Default::default()
        }
    }

    /// A plain material takes every field of the style.
    #[test]
    fn a_plain_material_takes_the_style() {
        let s = styled(&StandardMaterial::default(), &toon_style());
        assert_eq!(
            (s.toon, s.flatShading, s.outlineWidth, s.rimIntensity),
            (1, 1, 2.0, 0.5)
        );
    }

    /// A material with a physical lobe keeps its physical shading under the
    /// style's toon (toon would drop the lobe) and takes the rest; its own
    /// toon still makes it toon.
    #[test]
    fn a_physical_lobe_keeps_physical_shading_under_toon() {
        let style = toon_style();
        for m in [
            StandardMaterial {
                clearcoat: 1.0,
                ..Default::default()
            },
            StandardMaterial {
                transmission: 0.9,
                ..Default::default()
            },
            StandardMaterial {
                sheen: 0.5,
                ..Default::default()
            },
            StandardMaterial {
                ior: 1.33,
                ..Default::default()
            },
        ] {
            assert!(m.has_physical_lobes());
            let s = styled(&m, &style);
            assert_eq!(s.toon, 0, "{m:?}");
            assert_eq!(
                (s.outlineWidth, s.rimIntensity, s.flatShading),
                (2.0, 0.5, 1)
            );
        }
        let own = StandardMaterial {
            clearcoat: 1.0,
            toon: 1,
            ..Default::default()
        };
        assert_eq!(styled(&own, &style).toon, 1);
        assert!(!StandardMaterial::default().has_physical_lobes());
    }

    /// `styled` 0: the material exactly as declared, and its uniform turns
    /// the frame's shadow tint and outline distance off (mode bit 8).
    #[test]
    fn an_unstyled_material_is_untouched() {
        let m = StandardMaterial {
            styled: 0,
            ..Default::default()
        };
        assert_eq!(styled(&m, &toon_style()), m);
        assert_eq!(uniform(&m, false).flags[3] as u32 & 8, 8);
        assert_eq!(
            uniform(&StandardMaterial::default(), false).flags[3] as u32 & 8,
            0
        );
    }

    /// Mode bit 32: a single-sided material, front or back, is a closed
    /// surface whose sun shadows compare past its depth (lighting.wgsl
    /// `shadow_closed`); a double-sided one is in the shadow map itself.
    #[test]
    fn single_sided_materials_are_closed_for_shadows() {
        let closed = |side: Side| {
            let m = StandardMaterial {
                side: side as u32,
                ..Default::default()
            };
            uniform(&m, false).flags[3] as u32 & 32 != 0
        };
        assert!(closed(Side::Front));
        assert!(closed(Side::Back));
        assert!(!closed(Side::Double));
    }
}
