//! GPU resource records, batches and shadow maps.

use super::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShadowPassUniform {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) light: [f32; 4],
    /// x: the scene clock in seconds (a material shader's `h.time`), yzw:
    /// the scene's origin (`h.origin`, `origin.rs`).
    pub(crate) time: [f32; 4],
    /// x: 1 for a paraboloid hemisphere, y: +1 the one below the light, -1
    /// above, z: the map's side, w: the projection's scale (shadow_pass.wgsl).
    pub(crate) paraboloid: [f32; 4],
    /// Shadows draw only the current deformation; motion history lives in Frame.
    pub(crate) foliage_wind: crate::foliage::FoliageWind,
    pub(crate) pad: [f32; 8],
}

#[cfg(test)]
mod foliage_layout_tests {
    use super::*;

    #[test]
    fn shadow_wind_layout_matches_rust_and_dynamic_stride() {
        let module =
            naga::front::wgsl::parse_str(include_str!("../../shaders/shadow_pass.wgsl")).unwrap();
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).unwrap();
        let (handle, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some("ShadowPass"))
            .unwrap();
        let naga::TypeInner::Struct { members, .. } = &ty.inner else {
            panic!("ShadowPass must be a struct")
        };
        for (name, offset) in [
            (
                "view_proj",
                std::mem::offset_of!(ShadowPassUniform, view_proj),
            ),
            ("light", std::mem::offset_of!(ShadowPassUniform, light)),
            ("time", std::mem::offset_of!(ShadowPassUniform, time)),
            (
                "paraboloid",
                std::mem::offset_of!(ShadowPassUniform, paraboloid),
            ),
            (
                "foliage_wind",
                std::mem::offset_of!(ShadowPassUniform, foliage_wind),
            ),
        ] {
            let member = members
                .iter()
                .find(|m| m.name.as_deref() == Some(name))
                .unwrap();
            assert_eq!(member.offset as usize, offset, "{name}");
        }
        assert_eq!(
            layouter[handle].size as usize,
            std::mem::offset_of!(ShadowPassUniform, pad)
        );
        assert_eq!(
            std::mem::size_of::<ShadowPassUniform>() as u64,
            SHADOW_SLOT_STRIDE
        );
        assert_eq!(SHADOW_SLOT_STRIDE % 256, 0);
    }
}

/// An instance in the instance buffers: its columns' xyz, its
/// texture-array layer and its look ([`instances::GpuTransform`]).
pub(crate) const INSTANCE_STRIDE: u64 = std::mem::size_of::<instances::GpuTransform>() as u64;

pub(crate) const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4, 15 => Float32x4],
};

pub(crate) const INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32x4],
};

/// One attribute of [`Vertex`] at its offset in the shared vertex buffer.
pub(crate) const fn vertex_attribute(
    format: wgpu::VertexFormat,
    offset: usize,
    location: u32,
) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format,
        offset: offset as u64,
        shader_location: location,
    }
}

const POSITION_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x3,
    std::mem::offset_of!(Vertex, position),
    0,
);
const NORMAL_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x3,
    std::mem::offset_of!(Vertex, normal),
    1,
);
const COLOR_ATTRIBUTE: wgpu::VertexAttribute = vertex_attribute(
    wgpu::VertexFormat::Float32x4,
    std::mem::offset_of!(Vertex, color),
    3,
);

/// The shared vertex buffer as the lean lit stage reads it
/// (`pbr_lean.wgsl` `vs_lean`): position, normal and colour.
pub(crate) const LEAN_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE, COLOR_ATTRIBUTE],
};

/// As the lean prepass reads it (`vs_prepass_lean`): position and normal.
pub(crate) const POSITION_NORMAL_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[POSITION_ATTRIBUTE, NORMAL_ATTRIBUTE],
    };

/// The water's mesh surfaces (`water.wgsl` `vs_mesh`): position, normal, uv.
pub(crate) const POSITION_NORMAL_UV_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            POSITION_ATTRIBUTE,
            NORMAL_ATTRIBUTE,
            vertex_attribute(
                wgpu::VertexFormat::Float32x2,
                std::mem::offset_of!(Vertex, uv),
                2,
            ),
        ],
    };

/// As the lean shadow caster reads it (`shadow.wgsl` `vs_lean`): position.
pub(crate) const POSITION_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[POSITION_ATTRIBUTE],
};

pub(crate) struct GpuMesh {
    /// The f32 vertices ([`Vertex`]); a 16-byte placeholder while `full`
    /// is false.
    pub(crate) vertices: wgpu::Buffer,
    /// Whether `vertices` holds the mesh: false for a compact mesh no
    /// pipeline that reads [`Vertex`] has drawn yet ([`Renderer::ensure_full`]).
    pub(crate) full: bool,
    /// The compact vertices ([`VertexEncoding::Compact`], `compact.rs`)
    /// and how to undo their positions' quantisation.
    pub(crate) compact: Option<(wgpu::Buffer, compact::Dequant)>,
    pub(crate) indices: wgpu::Buffer,
    pub(crate) index_count: u32,
    /// The scene's revision of the mesh this holds (`RenderScene::set_mesh`).
    pub(crate) revision: u64,
    /// Local-space bounds as (center, half extent).
    pub(crate) bounds: Option<(Vec3, Vec3)>,
    pub(crate) foliage: Option<FoliageMesh>,
}

/// Attachment bounds cached by the scene's immutable foliage allocation.
pub(crate) struct FoliageMesh {
    pub(crate) source: std::sync::Arc<crate::meshlet::MeshFoliage>,
    pub(crate) rest: Option<(Vec3, Vec3)>,
    pub(crate) deformation: crate::foliage::FoliageBounds,
}

impl GpuMesh {
    /// What a draw binds: the compact vertices for a [`Batch::compact`]
    /// draw, else the f32 ones.
    pub(crate) fn vertex_buffer(&self, compact: bool) -> &wgpu::Buffer {
        match &self.compact {
            Some((buffer, _)) if compact => buffer,
            _ => {
                debug_assert!(self.full, "a compact mesh's f32 vertices were not uploaded");
                &self.vertices
            }
        }
    }
}

pub(crate) struct GpuMaterial {
    pub(crate) revision: u64,
    /// Its standard uniform (the group's binding 0) and the shader
    /// parameters it holds: a parameter change is written into it in place.
    pub(crate) factors: wgpu::Buffer,
    pub(crate) params: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
    pub(crate) previous_params: Option<[[f32; 4]; MAX_SHADER_PARAMS]>,
    /// The standard group: every pass but a physical material's lit one.
    pub(crate) bind_group: wgpu::BindGroup,
    /// A physical material's group for its lit pass (`physical.rs`).
    pub(crate) physical: Option<wgpu::BindGroup>,
    /// An array-map material's group for its lit and prepass passes
    /// ([`material::ArrayMaps`]); its shadows bind the standard group.
    pub(crate) array: Option<wgpu::BindGroup>,
    pub(crate) side: Side,
    /// Drawn in the sorted pass, back to front: blended or transmissive.
    pub(crate) transparent: bool,
    /// Blended (no depth write).
    pub(crate) blend: bool,
    /// Refracts the copy of the opaque scene.
    pub(crate) transmissive: bool,
    /// Discards pixels (shadows draw it with the material).
    pub(crate) dissolve: bool,
    /// Has its own outline (turns the outline pass on).
    pub(crate) outline: bool,
    /// Uses toon, rim, flash or dissolve.
    pub(crate) effects: bool,
    /// Its material shader (the scene's index).
    pub(crate) shader: Option<usize>,
    /// Writes exactly the depth of its triangles (opaque, no alpha cut,
    /// dissolve or shader): GPU culling may draw it as an occluder.
    pub(crate) occluder: bool,
    /// Plain: an occluder with no texture, no effect (toon, rim, flash,
    /// outline) and not physical. Its unskinned instances draw through the
    /// lean lit and prepass pipelines (`pbr_lean.wgsl`): the same pixels
    /// with fewer attributes read, fewer varyings and (without screen
    /// derivatives) no discard.
    pub(crate) lean: bool,
    /// Flat-shaded: its normals come from screen derivatives.
    pub(crate) flat: bool,
    /// Drawn as its triangles' edges in this colour (`lines.rs`), not as triangles.
    pub(crate) wireframe: Option<[f32; 4]>,
}

pub(crate) struct GpuTexture {
    pub(crate) view: wgpu::TextureView,
    pub(crate) format: TextureFormat,
    /// Bytes allocated, every level.
    pub(crate) bytes: u64,
    /// A render target's ([`TextureSource::Target`]): counted with the
    /// render targets in [`crate::GpuMemory`].
    pub(crate) target: bool,
}

/// What an RGBA8 texture allocates on the GPU, every level.
pub(crate) fn rgba_texture_bytes(texture: &TextureData) -> u64 {
    match texture.mips {
        Mips::Generate => (0..mips::mip_count(texture.width, texture.height))
            .map(|l| TextureFormat::Rgba8.level_bytes(texture.width, texture.height, l) as u64)
            .sum(),
        Mips::None => texture.rgba.len() as u64,
    }
}

pub(crate) struct GpuFont {
    pub(crate) layout: FontLayout,
    pub(crate) bind_group: wgpu::BindGroup,
    /// The scene's revision of the font this was built from.
    pub(crate) revision: u64,
}

/// GPU copies of one scene's resources, indexed like the scene's handles.
#[derive(Default)]
pub(crate) struct SceneCache {
    /// The scene's palettes (`add_palette`) as 256 x 1 textures (`None`:
    /// removed), and the scene's revision of each when it was uploaded.
    pub(crate) palettes: Vec<Option<wgpu::TextureView>>,
    pub(crate) palette_revisions: Vec<u64>,
    /// The material style the materials were uploaded with.
    pub(crate) material_style: MaterialStyle,
    pub(crate) scene_id: u64,
    pub(crate) meshes: Vec<GpuMesh>,
    /// The lit shader with each of the scene's material shaders.
    pub(crate) shaders: Vec<wgpu::ShaderModule>,
    /// The shadow shader with each of them.
    pub(crate) shadow_shaders: Vec<wgpu::ShaderModule>,
    /// Their lit pipelines by (shader, side, blended, effects, skinned).
    pub(crate) hook_pipelines: HashMap<(usize, Side, bool, bool, bool), wgpu::RenderPipeline>,
    /// Their shadow and prepass pipelines by (shader, side, skinned).
    pub(crate) hook_shadow_pipelines: HashMap<(usize, Side, bool), wgpu::RenderPipeline>,
    pub(crate) hook_prepass_pipelines: HashMap<(usize, Side, bool), wgpu::RenderPipeline>,
    /// The physical lit shader with a material shader, made when a physical
    /// material first uses it, and its pipelines by (shader, key).
    pub(crate) physical_shaders: HashMap<usize, wgpu::ShaderModule>,
    pub(crate) physical_hook_pipelines: HashMap<(usize, PbrKey), wgpu::RenderPipeline>,
    /// Physical materials' packed maps, shared by materials with the same maps.
    pub(crate) packs: HashMap<physical::PackKey, wgpu::TextureView>,
    pub(crate) textures: Vec<GpuTexture>,
    /// The scene's revision of each texture when it was uploaded.
    pub(crate) texture_revisions: Vec<u64>,
    /// Each stream texture's frame serial when it was last written
    /// ([`RenderScene::write_stream_texture`]), by texture index.
    pub(crate) stream_serials: std::collections::BTreeMap<usize, u64>,
    pub(crate) materials: Vec<GpuMaterial>,
    /// Sprite bind groups per texture, made on first use.
    pub(crate) sprite_groups: Vec<Option<wgpu::BindGroup>>,
    /// The filter each texture's sprite group was made with.
    pub(crate) filters: Vec<TextureFilter>,
    /// The same with a repeating sampler (trails, ribbons, beams).
    pub(crate) ribbon_groups: Vec<Option<wgpu::BindGroup>>,
    pub(crate) fonts: Vec<GpuFont>,
    /// Joints and weights of skinned (or morphed) meshes, by mesh.
    pub(crate) skins: Vec<Option<wgpu::Buffer>>,
    /// Each skinned mesh's joint boxes (`skin::joint_boxes`): its posed
    /// bounds, which pick its instances' reflection probes. By mesh.
    pub(crate) joint_boxes: Vec<Vec<skin::JointBox>>,
    /// Per mesh: (first delta, targets) in `morph_data`.
    pub(crate) morph_ranges: Vec<(u32, u32)>,
    /// Every mesh's morph deltas: per vertex, per target, position then normal.
    pub(crate) morph_data: Vec<[f32; 4]>,
    /// Whether `morph_data` changed since it was uploaded.
    pub(crate) morphs_dirty: bool,
    /// Colour LUTs as 3D textures with the bytes each allocates (`None`:
    /// removed), and the scene's revision of each when it was uploaded.
    pub(crate) luts: Vec<Option<(wgpu::TextureView, u64)>>,
    pub(crate) lut_revisions: Vec<u64>,
    /// Environment maps as cube textures (kept for their views; `None`:
    /// removed), and the scene's revision of each when it was uploaded.
    pub(crate) environment_maps: Vec<Option<GpuEnvironmentMap>>,
    pub(crate) environment_map_revisions: Vec<u64>,
    /// Texture arrays, each one 2D-array texture (`None`: removed), and
    /// the scene's revision of each when it was uploaded.
    pub(crate) texture_arrays: Vec<Option<texture_array::GpuTextureArray>>,
    pub(crate) texture_array_revisions: Vec<u64>,
    /// Array sprite bind groups per texture array, made on first use, with
    /// the filter each was made with.
    pub(crate) sprite_array_groups: Vec<Option<(wgpu::BindGroup, TextureFilter)>>,
}

/// An environment map's cube on the GPU.
pub(crate) struct GpuEnvironmentMap {
    /// Held for the view.
    pub(crate) _texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    /// Bytes allocated, every level of every face.
    pub(crate) bytes: u64,
}

/// What a palette's 256 x 1 RGBA8 texture allocates.
pub(crate) const PALETTE_BYTES: u64 = MAX_PALETTE_COLORS as u64 * 4;

/// What the image-based light was last generated from: the sky, or an
/// environment map of a scene (its id and index), turned and with its
/// background scaled (as bits, so a key compares exactly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EnvironmentKey {
    Sky(ProceduralSky),
    /// The per-pixel sky's (`Background::Procedural`): made once, then
    /// kept up to date by the live sky's capture (`sky_capture.rs`), not
    /// remade when the sky changes.
    Live,
    Map {
        scene: u64,
        map: usize,
        /// The scene's revision of the map's slot (a slot refilled after a
        /// removal is another map).
        revision: u64,
        rotation: u32,
        background: u32,
        /// The map blended towards (its index and revision) and the drawn
        /// weight's bits.
        blend: Option<(usize, u64, u32)>,
    },
}

/// One (mesh, material) run of the sorted instances.
pub(crate) struct Batch {
    pub(crate) mesh: usize,
    pub(crate) material: usize,
    pub(crate) first: u32,
    pub(crate) count: u32,
    pub(crate) side: Side,
    /// In the sorted pass (blended or transmissive).
    pub(crate) transparent: bool,
    /// Blended: the pipeline blends and writes no depth.
    pub(crate) blend: bool,
    /// Drawn with the physical lit pipelines.
    pub(crate) physical: bool,
    /// Refracts the scene copy.
    pub(crate) transmissive: bool,
    pub(crate) dissolve: bool,
    /// Needs the effect shader (its material's or the frame's effects).
    pub(crate) effects: bool,
    /// Its material's shader.
    pub(crate) shader: Option<usize>,
    /// Drawn with the lean pipelines ([`GpuMaterial::lean`]; never skinned).
    pub(crate) lean: bool,
    /// Its fragments take screen derivatives (flat shading, or the frame's
    /// decals): its lean pipelines keep the discard (`DISCARD` in
    /// pbr_fs.wgsl), so the derivatives are the full pipeline's.
    pub(crate) derivatives: bool,
    /// Drawn from its mesh's compact vertices (lean, the mesh
    /// [`VertexEncoding::Compact`]): the instance transforms include the
    /// dequantisation (`compact.rs`).
    pub(crate) compact: bool,
    /// Drawn with the array-map pipelines ([`GpuMaterial::array`]): its
    /// instances carry their layer in the transform.
    pub(crate) array: bool,
    /// Carries cooked foliage attachments; its auxiliary instance stream
    /// supplies sway even when motion effects are disabled.
    pub(crate) foliage: bool,
    /// Drawn into the shadow maps ([`Instance::cast_shadows`],
    /// [`SkinnedInstance::cast_shadows`]).
    pub(crate) cast: bool,
    /// Its instances' three.js layer mask (`views.rs`).
    pub(crate) layers: u32,
    /// The reflection probe its instances sample (`probe_select.rs`).
    pub(crate) probe: u8,
}

/// The effect bind groups and what they were made from.
pub(crate) struct FxGroups {
    pub(crate) key: (u64, u64, Option<TextureHandle>, u32, bool, bool, bool),
    pub(crate) fx: wgpu::BindGroup,
    /// `fx` for the lightmapped pipelines (`Renderer::fx_lean_group`).
    pub(crate) fx_lean: Option<wgpu::BindGroup>,
    pub(crate) sprite: wgpu::BindGroup,
    pub(crate) glow: wgpu::BindGroup,
}

/// The skinned instances of a frame: batches over `data` (every instance,
/// for shadows), each entry's widened world bounds, and the joint matrices.
pub(crate) struct SkinnedFrame {
    pub(crate) batches: Vec<Batch>,
    pub(crate) data: Vec<skin::SkinnedInstanceGpu>,
    pub(crate) spheres: Vec<(Vec3, f32)>,
    /// The palette buffer's current section: the joints, then the weight
    /// and deformed vertex slots (`skin.rs`, its layout).
    pub(crate) matrices: Vec<[[f32; 4]; 4]>,
    /// The previous frame's weight and deformed vertex slots, in the
    /// current section's order.
    pub(crate) previous_slots: Vec<[[f32; 4]; 4]>,
    /// Whether a baked instance moved or played on since the frame before
    /// (TAA's motion vectors then need the prepass).
    pub(crate) baked_moved: bool,
}

/// The lit pipelines: (side, blended, skinned, effects). Without effects the
/// shader's `EFFECTS` override is false: the toon, rim, flash, dissolve,
/// decal and AO code is compiled out (the features-off cost). The physical
/// pipelines are keyed the same (`physical.rs`).
pub(crate) type PbrKey = (Side, bool, bool, bool);
/// The shadow pipelines: (side, skinned, dissolving).
pub(crate) type ShadowKey = (Side, bool, bool);
/// What a lit pass's draws switch pipelines on: (side, blended, effects,
/// material shader, physical, lean, derivatives, compact, array).
pub(crate) type LitDrawKey = (
    Side,
    bool,
    bool,
    Option<usize>,
    bool,
    bool,
    bool,
    bool,
    bool,
);
/// The lean lit pipelines: (side, effects, derivatives, compact); `effects`
/// as in [`PbrKey`] (a plain material has none of its own: the frame's
/// decals or AO), `derivatives` as [`Batch::derivatives`], `compact`: the
/// vertices are [`VertexEncoding::Compact`]'s.
pub(crate) type LeanKey = (Side, bool, bool, bool);

/// What a frame's CPU culls share ([`Renderer::cull`]), kept between
/// frames so no pass allocates it: the in-view test of each sorted
/// instance, and per batch whether a list draws it whole from the sorted
/// instances. Those are the only sorted records a draw reads (every other
/// draw reads copies appended after them), so the only ones uploaded.
#[derive(Default)]
pub(crate) struct CullScratch {
    pub(crate) inside: Vec<bool>,
    pub(crate) whole: Vec<bool>,
}

/// What one pass draws: (batch, first instance, count) ranges in the instance buffer.
#[derive(Default)]
pub(crate) struct DrawList {
    pub(crate) ranges: Vec<(u32, u32, u32)>,
    pub(crate) instances: u32,
    pub(crate) triangles: u64,
    /// Triangles of the instances inside the planes ([`Renderer::cull`];
    /// `triangles` counts whole batches drawn past 90 % in view).
    pub(crate) in_view_triangles: u64,
    /// GPU culling's pass: the ranges are in its `instances` and each draw's
    /// count is in its `args` (`instances` and `triangles` are 0 here).
    pub(crate) indirect: Option<u32>,
    /// A transparent list's billboards drawn before each range: how many of
    /// the pass's ([`sprites::SpriteFrame::billboards`]) are farther than
    /// its first instance ([`Renderer::sort_transparent`]); empty in the
    /// other lists.
    pub(crate) before: Vec<u32>,
}

/// GPU culling's list for pass `g`: every `include`d batch, drawn indirect
/// from the pass's region.
pub(crate) fn indirect_list(
    batches: &[Batch],
    n: u32,
    g: u32,
    include: impl Fn(&Batch) -> bool,
) -> DrawList {
    DrawList {
        ranges: batches
            .iter()
            .enumerate()
            .filter(|(_, b)| include(b))
            .map(|(i, b)| (i as u32, (g + 1) * n + b.first, b.count))
            .collect(),
        indirect: Some(g),
        ..DrawList::default()
    }
}

/// A shadow map: one R32Float array the lit shaders read, a view per layer,
/// and the depth buffer its passes test against.
pub(crate) struct ShadowMap {
    /// The array itself (area.rs names it to know when it was made anew).
    pub(crate) texture: wgpu::Texture,
    pub(crate) array_view: wgpu::TextureView,
    pub(crate) layers: Vec<wgpu::TextureView>,
    pub(crate) depth: wgpu::TextureView,
    /// The depth buffer itself (the static shadow cache copies it).
    pub(crate) depth_texture: wgpu::Texture,
}

/// One shadow layer's static casters, kept while dynamic casters are drawn
/// over the live layer (`shadow_cache.rs`): its shadow value and depth.
pub(crate) struct CachedShadow {
    pub(crate) size: u32,
    pub(crate) color: wgpu::Texture,
    pub(crate) depth: wgpu::Texture,
}

impl CachedShadow {
    pub(crate) fn new(device: &wgpu::Device, size: u32) -> CachedShadow {
        let usage = wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST;
        CachedShadow {
            size,
            color: target_texture(
                device,
                "cached shadow",
                size,
                size,
                wgpu::TextureFormat::R32Float,
                usage,
            ),
            depth: target_texture(
                device,
                "cached shadow depth",
                size,
                size,
                DEPTH_FORMAT,
                usage,
            ),
        }
    }
}

impl ShadowMap {
    pub(crate) fn new(device: &wgpu::Device, label: &str, size: u32, layers: u32) -> ShadowMap {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let depth_texture = target_texture(
            device,
            label,
            size,
            size,
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
        );
        ShadowMap {
            array_view: texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            }),
            layers: (0..layers)
                .map(|layer| {
                    texture.create_view(&wgpu::TextureViewDescriptor {
                        label: Some(label),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        base_array_layer: layer,
                        array_layer_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect(),
            depth: depth_texture.create_view(&Default::default()),
            depth_texture,
            texture,
        }
    }
}
