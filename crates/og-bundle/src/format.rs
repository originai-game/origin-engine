//! The cooked asset blobs: what each `.ogb` holds, shared by the cook (which
//! writes them) and the runtime loaders (which read them).
//!
//! Every blob's root is a reflected struct in its `meta` section (packed
//! bytes, `og_core::reflect::Layout`); every table is rows of a reflected
//! struct made only of 4-byte scalars, so a row's packed layout equals its
//! `#[repr(C)]` layout and a table can be viewed in place. The blob header's
//! `type_hash` is [`type_hash`] over the root and row schemas, its
//! `schema_version` the root schema's version: changing any row layout
//! changes the type hash and old blobs are refused.
//!
//! Models keep a second meshopt index stream, `cluster_indices`, for
//! drawing. It shares the vertices and LOD ranges of `indices` without
//! changing the simulation mesh's triangle or vertex order.
//!
//! | blob | root | sections |
//! |---|---|---|
//! | model (`og.Model`) | [`ModelMeta`] | `primitives` [`PrimitiveRow`], `lods` [`LodRow`], `clusters` [`ClusterRow`] (each level's meshlets), `meshes` [`MeshRow`], `materials` [`MaterialRow`], `textures` [`TextureRefRow`], `nodes` [`NodeRow`], `scenes` [`SceneRow`], `roots` (u32 node indices), `strings`, `vertices`, `indices`, `bvh_nodes` ([`BvhNodeRow`] rows, zlib) and `bvh_tris` (u32, zlib): each primitive's raycast BVH, og-render's `Bvh`, unless `raycast false`; `colliders` (optional, `collider true`: the primitive count, each one's u32 offset and length after the header, then og-physics' `bake_trimesh` bytes over LOD 0); `lights` [`LightRow`] (`KHR_lights_punctual`), `instances` [`InstanceRow`] (`EXT_mesh_gpu_instancing`), `variants` [`VariantRow`] and `variant_mappings` [`VariantMappingRow`] (`KHR_materials_variants`); `impostors` (optional, `impostor true`) [`ImpostorRow`] and `impostor_materials` (optional) [`ImpostorMaterialRow`]; `skin_binds` (optional) [`NodeBindRow`]; `foliage_rows` (optional) [`FoliageRow`] and `foliage` (their meshopt streams) |
//! | texture (`og.Texture`; a PNG, JPEG, KTX2, WebP, TGA, PSD or DDS source, a glTF's image, a texture array's layer) | [`TextureMeta`] | `ktx2` (a KTX2 file) |
//! | texture array (`og.TextureArray`, an `.ogt` of `type texture-array`) | [`TextureArrayMeta`] | `layers` [`ArrayLayerRow`] (each layer's `og.Texture` id, `<id>#layer<n>`), `strings` |
//! | video (`og.Video`, an `.ogt` of `type video` cooked by ffmpeg) | [`VideoMeta`] | none: a clip's frames are the texture array `<id>#frames`, a stream's the `og.VideoStream` `<id>#stream` with its poster `<id>#poster`; its sound `<id>#audio` |
//! | video stream (`og.VideoStream`, `<id>#stream` of a streamed video, fetched when it loads) | [`VideoStreamMeta`] | `config` (AV1's sequence header OBU), `units` [`VideoUnitRow`], `data` (the AV1 temporal units) |
//! | sound (`og.Sound`; a `.wav`, `.mp3`, `.ogg` (Vorbis), `.flac` or `.opus`, or a procedural preset: an `.ogt` of `type sfx` synthesised by the cook) | [`SoundMeta`] | `qoa` (`codec` QOA: a QOA file; of a sound cooked in chunks, the header and the first chunk's frames) or `opus` (`codec` Opus: an Opus sound, `crate::opus`) |
//! | sound chunk (`og.SoundChunk`, `<id>#chunk<n>` of a sound cooked in chunks) | [`SoundChunkMeta`] | `qoa` (the chunk's QOA frames) |
//! | font (`og.Font`) | [`FontMeta`] | `glyphs` [`GlyphRow`], `kerning` [`KernRow`] (sorted by left, right), `atlas.<n>` (KTX2 RGBA8, zlib), `ttf` (the font file, when shaped or `runtime`) |
//! | scene (`og.Scene`) | [`SceneMeta`] | `entities` [`EntityRow`], `refs` [`RefRow`], `components` [`ComponentRow`], `strings`, per component `c.<Type>` rows and `s.<Type>` its schema JSON; when an entity was placed from a prefab, `prefabs` [`PrefabRow`] and `prefab_refs` [`RefRow`]; when one places a sub-scene, `subscenes` [`SubsceneRow`] and `subscene_refs` [`RefRow`]. A prefab (an `.ogt` of `type prefab`) cooks to one too, flattened |
//! | colour LUT (`og.Lut`, a `.cube`) | [`LutMeta`] | `rgba` (size³ RGBA8, red fastest) |
//! | particle emitter (`og.ParticleEmitter`, an `.ogt` of `type particles`) | [`EmitterMeta`] | `text` (the validated emitter file) |
//! | UI style (`og.UiStyle`, an `.ogt` of `type style`) | [`StyleMeta`] | `text` (the validated style file) |
//! | UI document (`og.UiDocument`, an `.ogt` of `type screen` or `type component`) | [`DocumentMeta`] | `text` (the validated document) |
//! | material shader (`og.MaterialShader`, an `.ogt` of `type shader`) | [`ShaderMeta`] | `text` (the canonical shader file, code inlined) |
//! | custom post pass (`og.PostPass`, an `.ogt` of `type post`) | [`PostMeta`] | `text` (the canonical post file, code inlined) |
//! | 2D rig (`og.Skeleton2D`, an `.ogt` of `type rig2d`) | [`Rig2DMeta`] | `text` (the canonical rig: a DragonBones export converted, the texture's id and size) |
//! | flipbook sheet (`og.Flipbook`, an `.ogt` of `type flipbook`) | [`FlipbookMeta`] | `text` (the canonical sheet: frames resolved from the grid and an Aseprite or TexturePacker atlas, clips with frame indices and durations, the texture's size) |
//! | fracture (`og.Fracture`, a model's `#fracture`: a model with a `fracture` block) | [`FractureMeta`] | `asset` (`og_destruct::FractureAsset::to_bytes`: the chunks, their hierarchy, the support graph's bonds and the anchors) |
//! | animation (`og.Animation`, a glTF's `#anim`) | [`AnimMeta`] | `nodes` [`NodeRow`], `hidden` (u32 node indices hidden at rest, ascending), `skins` [`SkinRow`], `joints` (u32 node indices), `inverse_binds` (16 f32 per joint, column-major), `clips` [`ClipRow`], `channels` [`ChannelRow`], `samples` (f32), `packed` (u16: quantised keys, `crate::anim_codec`), `events` [`EventRow`], `skin_streams` [`SkinStreamRow`], `skin_data` (meshopt streams), `morphs` [`MorphRow`], `targets` [`TargetRow`], `morph_data` (meshopt streams of the moved vertices' deltas and indices), `strings` |
//! | tileset (`og.Tileset`, an `.ogt` of `type tileset`) | [`TilesetMeta`] | `text` (the canonical tileset text) |
//! | tilemap (`og.Tilemap`, an `.ogt` of `type tilemap`, a Tiled `.tmj`, an LDtk `.ldtk`) | [`TilemapMeta`] | `layers` [`MapLayerRow`], `cells` (u32 per cell, layer after layer), `objects` [`MapObjectRow`], `props` [`MapPropRow`], `tilesets` [`MapTilesetRow`], `strings`; optional `orientation` (u32 `og_tilemap::Orientation::code`, absent for orthogonal: G2-6) |
//! | behaviour (`og.Behaviour`, an `.ogt` of `type behaviour`) | [`BehaviourMeta`] | `text` (the validated behaviour file) |
//! | dialogue (`og.Dialogue`, an `.ogt` of `type dialogue`) | [`DialogueMeta`] | `text` (the validated dialogue file) |
//! | quest (`og.Quest`, an `.ogt` of `type quest`) | [`QuestMeta`] | `text` (the validated quest file) |
//! | navmesh (`og.NavMesh`, an `.ogt` of `type navmesh`) | [`NavMeshMeta`] | `columns` [`NavColumnRow`], `spans` [`NavSpanRow`] (the walkable heightfield), `tiles` [`NavTileRow`], `verts` [`NavVertRow`], `polys` [`NavPolyRow`], `poly_verts` (u32) |
//! | string table (`og.Strings`, an `.ogt` of `type strings`) | [`StringsMeta`] | `text` (the validated table) |
//! | humanoid rig (`og.Rig`, an `.ogt` of `type rig`) | [`RigMeta`] | `text` (the canonical rig: its scheme and the bones it names) |
//! | terrain (`og.Terrain`, an `.ogt` of `type terrain`) | [`TerrainMeta`] | `tiles` [`TerrainTileRow`] (og-render's tile pyramid), `heights` (each tile's `(tile + 3)²` u16 heights over the terrain's range, zlib), `control` (each tile's `(control + 2)²` RGBA8 texels, zlib), `layers` [`TerrainLayerRow`], `strings` (the layers' texture ids, the layer arrays' ids) |
//! | look (`og.Look`, an `.ogt` of `type look`) | [`LookMeta`] | `text` (the canonical, flattened look: `extends` resolved and listed, colours linear) |
//! | palette (`og.Palette`, a `.hex` or `.gpl` file) | [`PaletteMeta`] | `rgb` (`count` sRGB colours, 3 bytes each, in file order) |
//! | environment map (`og.EnvironmentMap`, an `.ogt` of `type environment`, phase 4) | [`EnvironmentMeta`] | `texels` (og-render's `EnvironmentMapData::texels`: RGB9E5 u32 LE, level after level, face after face, zlib), `irradiance` [`EnvironmentShRow`] (nine order-2 harmonics of the irradiance over pi) |
//! | streaming group pack (`og.StreamPack`, `cooked/stream/<group>.ogb`, phase 3) | [`StreamPackMeta`] | `files` [`StreamFileRow`], `names` (paths under `cooked/`), `data` (the files, each 16-aligned) |

use og_core::hash::StableHasher;
use og_core::{Reflect, Schema};

/// Changes whenever the bytes the cook writes for the same input change.
/// Part of every cook stamp, so a new cooker re-cooks everything.
pub const COOKER_VERSION: &str = concat!("og-cook ", env!("CARGO_PKG_VERSION"), " r12");

/// "None" in a u32 index field.
pub const NONE: u32 = u32::MAX;

/// The blob type hash: the layout hashes of the root and row schemas, in order.
pub fn type_hash(schemas: &[Schema]) -> u64 {
    let mut h = StableHasher::new();
    for s in schemas {
        h.write_u64(s.layout_hash());
    }
    h.finish()
}

/// The layout hash a table section of `T` rows is tagged with.
pub fn row_hash<T: Reflect>() -> u64 {
    T::schema().layout_hash()
}

/// One kind of cooked blob: its root struct and the row types it uses.
pub trait BlobKind {
    /// The root schema name, also the manifest's `type` of the output.
    const NAME: &'static str;
    fn schemas() -> Vec<Schema>;
    fn type_hash() -> u64 {
        type_hash(&Self::schemas())
    }
    fn schema_version() -> u32 {
        Self::schemas()[0].version
    }
}

/// The type hash and schema version a cooked file of manifest type `name`
/// must hold today, for every kind the cook writes. The incremental cook
/// reads a kept output's header against this, so a schema that changed
/// without a [`COOKER_VERSION`] bump still re-cooks its blobs instead of
/// leaving files the loader refuses. `None`: not a kind the cook writes
/// (such an output is never fresh, which the first cook test shows).
pub fn expected_header(name: &str) -> Option<(u64, u32)> {
    macro_rules! kinds {
        ($($k:ident),* $(,)?) => {{
            $(if name == <$k as BlobKind>::NAME {
                return Some((<$k as BlobKind>::type_hash(), <$k as BlobKind>::schema_version()));
            })*
            None
        }};
    }
    kinds!(
        Model,
        TextureArray,
        Video,
        VideoStream,
        Texture,
        Sound,
        SoundChunk,
        Font,
        Scene,
        Animation,
        Lut,
        Style,
        Document,
        Shader,
        Post,
        UiMaterial,
        Vector,
        Emitter,
        Tileset,
        Tilemap,
        Flipbook,
        Rig2D,
        Behaviour,
        Dialogue,
        Quest,
        Rig,
        Look,
        Palette,
        NavMesh,
        Strings,
        Timeline,
        Fracture,
        Lightmap,
        Terrain,
        EnvironmentMap,
        StreamPack,
    )
}

macro_rules! row {
    ($(#[$doc:meta])* $name:ident, $schema:literal, $version:literal { $($(#[$fdoc:meta])* $field:ident : $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Reflect, bytemuck::Pod, bytemuck::Zeroable)]
        #[reflect(name = $schema, version = $version)]
        pub struct $name {
            $($(#[$fdoc])* pub $field: $ty,)*
        }
    };
}

mod scene_effects;
mod world;

pub use scene_effects::*;
pub use world::*;

// ── model ────────────────────────────────────────────────────────────────

row!(
    /// The root of a model blob (a glTF file).
    ModelMeta, "og.Model", 1 {
        primitive_count: u32,
        mesh_count: u32,
        material_count: u32,
        texture_count: u32,
        node_count: u32,
        scene_count: u32,
        /// The scene `og.load.gltf` instantiates by default ([`NONE`] when the file names none and has no scenes).
        default_scene: u32,
        /// The glTF's animations and skins: not in this blob, cooked into its
        /// `#anim` blob ([`AnimMeta`]) when either is present.
        skipped_animations: u32,
        skipped_skins: u32,
        /// Bounds of the default scene's vertices with node transforms applied.
        min_x: f32, min_y: f32, min_z: f32,
        max_x: f32, max_y: f32, max_z: f32,
    }
);

/// [`PrimitiveRow::attributes`] bits.
pub mod attr {
    pub const NORMAL: u32 = 1;
    pub const UV: u32 = 2;
    pub const COLOR: u32 = 4;
    pub const TANGENT: u32 = 8;
}

row!(
    /// One glTF mesh primitive: a meshopt-encoded vertex stream and index
    /// buffer holding every LOD. The vertex stream is, per vertex and in this
    /// order: position `u16 x 4` (unorm over the bounds, `w` zero), normal
    /// `i16 x 2` (snorm octahedral), then when present uv `u16 x 2` (unorm over
    /// the uv range), colour `u8 x 4` (unorm), tangent `i16 x 4` (snorm
    /// octahedral `xy`, `z` the bitangent sign as +-32767, `w` zero).
    PrimitiveRow, "og.ModelPrimitive", 3 {
        material: u32,
        attributes: u32,
        vertex_count: u32,
        vertex_stride: u32,
        /// Byte range of the encoded vertex stream in `vertices`.
        vertex_offset: u32,
        vertex_size: u32,
        /// Byte range of the encoded indices in `indices`, and the decoded count (all LODs).
        index_offset: u32,
        index_size: u32,
        index_count: u32,
        /// The same triangles in draw-cluster order, meshopt-encoded in
        /// `cluster_indices`; decoded count and LOD ranges match `indices`.
        cluster_index_offset: u32,
        cluster_index_size: u32,
        lod_first: u32,
        lod_count: u32,
        /// Dequantisation: `p = offset + q / 65535 * scale` (also the exact local bounds' minimum and size).
        pos_offset_x: f32, pos_offset_y: f32, pos_offset_z: f32,
        pos_scale_x: f32, pos_scale_y: f32, pos_scale_z: f32,
        uv_offset_x: f32, uv_offset_y: f32,
        uv_scale_x: f32, uv_scale_y: f32,
        /// The raycast BVH over LOD 0 (og.render.raycast): a range of
        /// `bvh_nodes` and one of `bvh_tris` (counts 0: none was cooked,
        /// `raycast false`; the runtime builds it at the first raycast).
        bvh_node_first: u32,
        bvh_node_count: u32,
        bvh_tri_first: u32,
        bvh_tri_count: u32,
    }
);

row!(
    /// One node of a primitive's raycast BVH (og-render's `BvhNode`), depth
    /// first. Its bounds are quantised on the primitive's own position grid
    /// (`pos_offset` + q / 65535 * `pos_scale`), on which every decoded
    /// vertex lies, so they decode to exactly the bounds a build over the
    /// decoded mesh computes: `min_xy` packs min x (low 16 bits) and min y,
    /// `min_z_max_x` min z and max x, `max_yz` max y and max z. `first` and
    /// `count`: a leaf's range of the primitive's `bvh_tris` (relative to
    /// its first), or (`count` 0) the right child's index in the
    /// primitive's nodes.
    BvhNodeRow, "og.ModelBvhNode", 1 {
        min_xy: u32,
        min_z_max_x: u32,
        max_yz: u32,
        first: u32,
        count: u32,
    }
);

row!(
    /// One level of detail: a range of a primitive's decoded indices.
    LodRow, "og.ModelLod", 3 {
        index_first: u32,
        index_count: u32,
        /// The level's largest distance from the full mesh, both ways, in
        /// the model's units (0 for LOD 0): og-render's `lod` draws the
        /// level while it projects to less than a pixel.
        error: f32,
        /// The level's clusters: a range of `clusters`, which together
        /// cover its index range in order (og-render's `meshlet`).
        cluster_first: u32,
        cluster_count: u32,
    }
);

row!(
    /// One cluster (meshlet) of a level: at most
    /// `og_render::meshlet::CLUSTER_TRIANGLES` triangles, a contiguous
    /// range of the primitive's decoded cluster indices, with its bounding
    /// sphere and normal cone in the model's units (meshoptimizer's
    /// `meshopt_computeMeshletBounds`). The GPU culls each cluster on its
    /// own: the sphere against the frustum and the depth pyramid, the cone
    /// against the eye (every triangle faces away when
    /// `dot(normalize(center - eye), cone_axis) >= cone_cutoff + radius /
    /// distance`; a `cone_cutoff` of 1 never culls).
    ///
    /// The error fields state selection per cluster, so a cluster DAG
    /// (Nanite's) needs no other format: a cluster is drawn when its
    /// `error`, projected from the sphere `error_center`/`error_radius`,
    /// stays under a pixel and its `parent_error`, projected from the same
    /// sphere, does not (og-render's `meshlet::selected`). The cook writes
    /// the per-level chain in that form: every cluster of level k has the
    /// level's error (the largest of levels 0 to k), the next level's as
    /// its parent's (`f32::MAX` for the coarsest), the primitive's bounding
    /// sphere and `group` k, so exactly one level is drawn, the one
    /// og-render's `lod::select` picks. A DAG writes each cluster's own
    /// error and group and its parents' error and bounds.
    ClusterRow, "og.ModelCluster", 2 {
        index_first: u32,
        triangle_count: u32,
        center_x: f32, center_y: f32, center_z: f32,
        radius: f32,
        cone_x: f32, cone_y: f32, cone_z: f32,
        cone_cutoff: f32,
        error: f32,
        parent_error: f32,
        error_center_x: f32, error_center_y: f32, error_center_z: f32,
        error_radius: f32,
        /// The group it was simplified with (a DAG's), the level index here.
        group: u32,
    }
);

row!(
    /// A model's octahedral impostor (`model { impostor true }`,
    /// og-render's `impostor`): the whole model drawn as one camera-facing
    /// card from its two atlases, the last level of its chain. At most one
    /// per model (the optional `impostors` table; absent or empty: none).
    ImpostorRow, "og.ModelImpostor", 1 {
        /// The atlases' `og.Texture` ids (in `strings`): the albedo with
        /// its coverage in alpha, and the normal (in each frame's basis)
        /// with the depth towards the viewer.
        albedo_offset: u32,
        albedo_len: u32,
        normal_offset: u32,
        normal_len: u32,
        /// Frames per side of the atlases, and pixels per frame.
        grid: u32,
        frame: u32,
        /// The sphere the frames cover, in the model's space.
        center_x: f32, center_y: f32, center_z: f32,
        radius: f32,
        /// Its error in the model's units, as `LodRow::error`: an atlas
        /// texel's size on the model, so a model draws its impostor once a
        /// texel covers less than a pixel and its mesh's coarsest level
        /// would be drawn (a chain's errors only rise: the impostor
        /// replaces the mesh levels no finer than it).
        error: f32,
    }
);

row!(
    /// A model impostor's material atlas (G5, the optional
    /// `impostor_materials` table beside `impostors`; absent: none, the
    /// cards shaded by their material's factors): its `og.Texture` id (in
    /// `strings`), each frame's texel the model's roughness in red, its
    /// translucency in green and its metallic in blue.
    ImpostorMaterialRow, "og.ModelImpostorMaterial", 1 {
        material_offset: u32,
        material_len: u32,
    }
);

row!(
    /// A primitive's foliage (glTF `_OG_PIVOT` and `_OG_ANCHOR`, optional
    /// `foliage` table; absent or empty: none): its vertices' range of the
    /// `foliage` section, which holds 8 f32 per vertex (the branch pivot
    /// and stiffness, the leaf anchor and flutter, in the cooked vertex
    /// order) as one meshopt vertex stream per primitive. og-render's
    /// `meshlet::MeshFoliage` sways them on the meshlet path.
    FoliageRow, "og.ModelFoliage", 1 {
        primitive: u32,
        /// Its stream's bytes in the `foliage` section.
        stream_offset: u32,
        stream_size: u32,
        /// Bit 0: its triangles are leaf cards (`extras.ogFoliage.leaves`).
        flags: u32,
    }
);

row!(
    /// A glTF mesh: a contiguous range of primitives.
    MeshRow, "og.ModelMesh", 1 {
        primitive_first: u32,
        primitive_count: u32,
        name_offset: u32,
        name_len: u32,
    }
);

/// [`MaterialRow::alpha_mode`] values.
pub mod alpha {
    pub const OPAQUE: u32 = 0;
    pub const MASK: u32 = 1;
    pub const BLEND: u32 = 2;
}

row!(
    /// A glTF metallic-roughness material with its physical extensions
    /// (og-render's `PhysicalExtensions`). Texture fields index the model's
    /// `textures` table ([`NONE`] when absent). The last material of every
    /// model is the default one primitives without a material use.
    MaterialRow, "og.ModelMaterial", 3 {
        base_r: f32, base_g: f32, base_b: f32, base_a: f32,
        emissive_r: f32, emissive_g: f32, emissive_b: f32,
        metallic: f32,
        roughness: f32,
        normal_scale: f32,
        occlusion_strength: f32,
        alpha_mode: u32,
        alpha_cutoff: f32,
        double_sided: u32,
        base_color_texture: u32,
        metallic_roughness_texture: u32,
        normal_texture: u32,
        emissive_texture: u32,
        occlusion_texture: u32,
        name_offset: u32,
        name_len: u32,
        /// Which `KHR_materials_*` extensions the material has
        /// (`PhysicalExtensions::present` bits); the fields of the others
        /// are glTF's defaults.
        physical: u32,
        clearcoat: f32,
        clearcoat_roughness: f32,
        clearcoat_normal_scale: f32,
        sheen_r: f32, sheen_g: f32, sheen_b: f32,
        sheen_roughness: f32,
        transmission: f32,
        thickness: f32,
        /// 0: no absorption (glTF's +infinity).
        attenuation_distance: f32,
        attenuation_r: f32, attenuation_g: f32, attenuation_b: f32,
        ior: f32,
        iridescence: f32,
        iridescence_ior: f32,
        iridescence_min: f32,
        iridescence_max: f32,
        anisotropy: f32,
        anisotropy_rotation: f32,
        clearcoat_texture: u32,
        clearcoat_roughness_texture: u32,
        clearcoat_normal_texture: u32,
        sheen_color_texture: u32,
        sheen_roughness_texture: u32,
        transmission_texture: u32,
        thickness_texture: u32,
        iridescence_texture: u32,
        iridescence_thickness_texture: u32,
        anisotropy_texture: u32,
        /// 1: `KHR_materials_unlit` (drawn unlit, three.js MeshBasicMaterial).
        unlit: u32,
        /// `KHR_texture_transform` of each slot that keeps its own
        /// (og-render's `UvTransform`, offset, scale and rotation in
        /// radians; identity where the cook baked the transform into its
        /// primitives' uvs): the base colour, the metallic-roughness map
        /// (with a physical material's packed maps), the normal map, the
        /// emissive map and the occlusion map.
        map_offset_x: f32, map_offset_y: f32, map_scale_x: f32, map_scale_y: f32, map_rotation: f32,
        mr_offset_x: f32, mr_offset_y: f32, mr_scale_x: f32, mr_scale_y: f32, mr_rotation: f32,
        normal_offset_x: f32, normal_offset_y: f32, normal_scale_x: f32, normal_scale_y: f32, normal_rotation: f32,
        emissive_offset_x: f32, emissive_offset_y: f32, emissive_scale_x: f32, emissive_scale_y: f32, emissive_rotation: f32,
        occlusion_offset_x: f32, occlusion_offset_y: f32, occlusion_scale_x: f32, occlusion_scale_y: f32, occlusion_rotation: f32,
    }
);

row!(
    /// A texture a model uses: the asset id of its own texture blob.
    TextureRefRow, "og.ModelTexture", 1 {
        id_offset: u32,
        id_len: u32,
        /// 0 colour, 1 normal, 2 linear data ([`TextureMeta::kind`]).
        kind: u32,
    }
);

row!(
    /// A glTF node in topological order (a parent precedes its children).
    /// Version 2 added `visible`.
    NodeRow, "og.ModelNode", 2 {
        parent: u32,
        mesh: u32,
        name_offset: u32,
        name_len: u32,
        tx: f32, ty: f32, tz: f32,
        rx: f32, ry: f32, rz: f32, rw: f32,
        sx: f32, sy: f32, sz: f32,
        /// 1 shown, 0 hidden at rest (glTF `KHR_node_visibility` `visible:
        /// false`), and its descendants with it.
        visible: u32,
    }
);

row!(
    /// A skinned mesh node's bind pose in model space (the optional
    /// `skin_binds` table; absent: none). glTF ignores a skinned mesh
    /// node's own transform: its vertices are placed by the joints, at
    /// rest by the first joint's rest transform times its inverse bind
    /// matrix. A model drawn without its pose (and its bounds and levels
    /// of detail) uses this instead of the node's transform, which an
    /// exporter may scale arbitrarily (an `Armature` at 0.01 over vertices
    /// in metres).
    NodeBindRow, "og.ModelNodeBind", 1 {
        node: u32,
        tx: f32, ty: f32, tz: f32,
        rx: f32, ry: f32, rz: f32, rw: f32,
        sx: f32, sy: f32, sz: f32,
    }
);

row!(
    /// A glTF scene: a range of the `roots` section.
    SceneRow, "og.ModelScene", 1 {
        root_first: u32,
        root_count: u32,
        name_offset: u32,
        name_len: u32,
    }
);

/// [`LightRow::kind`] values (`KHR_lights_punctual`'s types).
pub mod light {
    pub const DIRECTIONAL: u32 = 0;
    pub const POINT: u32 = 1;
    pub const SPOT: u32 = 2;
}

row!(
    /// A `KHR_lights_punctual` light on a node (in cooked node order),
    /// shining along the node's local -Z. glTF's units: candela for point
    /// and spot lights, lux for directional ones.
    LightRow, "og.ModelLight", 1 {
        node: u32,
        /// [`light`].
        kind: u32,
        r: f32, g: f32, b: f32,
        intensity: f32,
        /// Where it reaches zero; 0: glTF's infinite range.
        range: f32,
        /// Spot cone half-angles in radians (0 and pi / 4 otherwise).
        inner_cone: f32,
        outer_cone: f32,
        name_offset: u32,
        name_len: u32,
    }
);

row!(
    /// One `EXT_mesh_gpu_instancing` instance of a node's mesh, in the
    /// node's space: the node draws its mesh once per row, never at its own
    /// transform alone. Rows of one node are consecutive, in its order.
    InstanceRow, "og.ModelInstance", 1 {
        node: u32,
        tx: f32, ty: f32, tz: f32,
        rx: f32, ry: f32, rz: f32, rw: f32,
        sx: f32, sy: f32, sz: f32,
    }
);

row!(
    /// A `KHR_materials_variants` variant: its name.
    VariantRow, "og.ModelVariant", 1 {
        name_offset: u32,
        name_len: u32,
    }
);

row!(
    /// A primitive's material under a variant (`KHR_materials_variants`
    /// mappings): a primitive without a row for a variant keeps its own.
    VariantMappingRow, "og.ModelVariantMapping", 1 {
        primitive: u32,
        variant: u32,
        material: u32,
    }
);

/// The model blob.
pub struct Model;
impl BlobKind for Model {
    const NAME: &'static str = "og.Model";
    fn schemas() -> Vec<Schema> {
        vec![
            ModelMeta::schema(),
            PrimitiveRow::schema(),
            LodRow::schema(),
            MeshRow::schema(),
            MaterialRow::schema(),
            TextureRefRow::schema(),
            NodeRow::schema(),
            SceneRow::schema(),
            BvhNodeRow::schema(),
            LightRow::schema(),
            InstanceRow::schema(),
            VariantRow::schema(),
            VariantMappingRow::schema(),
            ClusterRow::schema(),
        ]
    }
}

// ── texture ──────────────────────────────────────────────────────────────

/// [`TextureMeta::kind`] values.
pub mod kind {
    /// Colour (base colour, emissive, UI): ETC1S, sRGB.
    pub const COLOR: u32 = 0;
    /// Tangent-space normals: UASTC, linear.
    pub const NORMAL: u32 = 1;
    /// Linear data (metallic-roughness, occlusion, masks): UASTC, linear.
    pub const LINEAR: u32 = 2;
    /// A distance-field atlas: RGBA8, linear, lossless.
    pub const MSDF: u32 = 3;
    /// Pixel art (sprite sheets, tilesets): RGBA8, sRGB, lossless, no
    /// mips, sampled nearest by the runtime.
    pub const PIXEL: u32 = 4;
}

/// [`TextureMeta::codec`] values.
pub mod codec {
    pub const ETC1S: u32 = 0;
    pub const UASTC: u32 = 1;
    pub const RGBA8: u32 = 2;
}

row!(
    /// The root of a texture blob; the `ktx2` section is the KTX2 file.
    TextureMeta, "og.Texture", 1 {
        width: u32,
        height: u32,
        levels: u32,
        kind: u32,
        codec: u32,
        /// 0 sRGB, 1 linear.
        color_space: u32,
        has_alpha: u32,
        /// The KTX2 supercompression scheme (1 BasisLZ, 2 Zstandard, 3 zlib).
        supercompression: u32,
    }
);

row!(
    /// The root of a texture array blob (an `.ogt` of `type texture-array`):
    /// `layers` same-sized layers, each an `og.Texture` blob of its own
    /// (`<id>#layer<n>`, named in `layers` [`ArrayLayerRow`]), which the
    /// runtime uploads as one 2D-array texture.
    TextureArrayMeta, "og.TextureArray", 1 {
        width: u32,
        height: u32,
        layers: u32,
        /// Mip levels each layer is cooked with.
        levels: u32,
        /// [`kind`]: colour, normal, linear or pixel.
        kind: u32,
    }
);

row!(
    /// A texture array's layer: its texture's asset id in `strings`.
    ArrayLayerRow, "og.TextureArrayLayer", 1 {
        name_offset: u32,
        name_len: u32,
    }
);

/// The texture array blob.
pub struct TextureArray;
impl BlobKind for TextureArray {
    const NAME: &'static str = "og.TextureArray";
    fn schemas() -> Vec<Schema> {
        vec![TextureArrayMeta::schema(), ArrayLayerRow::schema()]
    }
}

row!(
    /// The root of a video blob (an `.ogt` of `type video`, cooked by
    /// ffmpeg). A clip (`streamed` 0): its frames are the layers of the
    /// texture array `<id>#frames`. A stream (`streamed` 1): its frames are
    /// the AV1 units of `<id>#stream` ([`VideoStreamMeta`]), decoded as it
    /// plays, and its first frame the texture `<id>#poster`. Its sound
    /// (when `audio` is 1) is the sound `<id>#audio` either way.
    VideoMeta, "og.Video", 2 {
        width: u32,
        height: u32,
        frames: u32,
        fps: u32,
        /// frames / fps.
        seconds: f32,
        /// 1 when `<id>#audio` was cooked.
        audio: u32,
        /// [`kind`]: colour or pixel.
        kind: u32,
        /// 1: streamed (`<id>#stream`, `<id>#poster`); 0: a clip (`<id>#frames`).
        streamed: u32,
        /// A stream's [`video_codec`]; 0 for a clip.
        codec: u32,
    }
);

/// The codecs a stream is cooked in.
pub mod video_codec {
    /// AV1, 8-bit 4:2:0, in temporal units of the low-overhead OBU format.
    pub const AV1: u32 = 1;
}

/// The video blob.
pub struct Video;
impl BlobKind for Video {
    const NAME: &'static str = "og.Video";
    fn schemas() -> Vec<Schema> {
        vec![VideoMeta::schema()]
    }
}

row!(
    /// The root of a video stream blob (`<id>#stream` of a streamed
    /// `type video`): the `config` section is the codec's configuration
    /// (AV1: the sequence header OBU), `units` one [`VideoUnitRow`] per
    /// frame (frame n is unit n, shown at n / fps), `data` the units' bytes.
    VideoStreamMeta, "og.VideoStream", 1 {
        /// [`video_codec`].
        codec: u32,
        width: u32,
        height: u32,
        frames: u32,
        fps: u32,
        /// Frames from one key frame to the next (the longest a seek decodes).
        key_interval: u32,
    }
);

row!(
    /// One frame's unit in a stream's `data`.
    VideoUnitRow, "og.VideoUnit", 1 {
        offset: u32,
        size: u32,
        /// 1: a key frame, where decoding may start.
        key: u32,
    }
);

/// The video stream blob.
pub struct VideoStream;
impl BlobKind for VideoStream {
    const NAME: &'static str = "og.VideoStream";
    fn schemas() -> Vec<Schema> {
        vec![VideoStreamMeta::schema(), VideoUnitRow::schema()]
    }
}

/// The texture blob.
pub struct Texture;
impl BlobKind for Texture {
    const NAME: &'static str = "og.Texture";
    fn schemas() -> Vec<Schema> {
        vec![TextureMeta::schema()]
    }
}

// ── sound ────────────────────────────────────────────────────────────────

/// `SoundMeta::codec` values.
pub mod sound_codec {
    /// QOA: the `qoa` section.
    pub const QOA: u32 = 0;
    /// Opus (`crate::opus`): the `opus` section, never chunked.
    pub const OPUS: u32 = 1;
}

row!(
    /// The root of a sound blob. With `codec` QOA the `qoa` section is a QOA
    /// file; a sound cooked in chunks (`chunk_count` > 0) holds the file's
    /// header (the whole length) and its first `chunk_frames` QOA frames
    /// only, and chunk `n` (1..chunk_count) is the blob `<id>#chunk<n>`
    /// ([`SoundChunkMeta`]). With `codec` Opus the `opus` section is an Opus
    /// sound (`crate::opus`: self-contained blocks of 0.48 s at 48 kHz),
    /// whole. Version 3 added `codec`.
    SoundMeta, "og.Sound", 3 {
        sample_rate: u32,
        channels: u32,
        /// Samples per channel.
        frames: u32,
        /// QOA frames (5120 samples per channel) per chunk; 0: not chunked.
        chunk_frames: u32,
        /// Chunks, the first included; 0: not chunked.
        chunk_count: u32,
        /// [`sound_codec`]: QOA (0) or Opus (1).
        codec: u32,
    }
);

/// The sound blob.
pub struct Sound;
impl BlobKind for Sound {
    const NAME: &'static str = "og.Sound";
    fn schemas() -> Vec<Schema> {
        vec![SoundMeta::schema()]
    }
}

row!(
    /// The root of a chunk of a sound cooked in chunks (`<id>#chunk<n>`);
    /// its `qoa` section is the chunk's QOA frames, without a file header.
    SoundChunkMeta, "og.SoundChunk", 1 {
        /// The chunk's index (1 and up: the first is in the sound's blob).
        chunk: u32,
        /// Its first QOA frame in the whole file, and how many it holds.
        first_frame: u32,
        frame_count: u32,
    }
);

/// A chunk of a sound cooked in chunks.
pub struct SoundChunk;
impl BlobKind for SoundChunk {
    const NAME: &'static str = "og.SoundChunk";
    fn schemas() -> Vec<Schema> {
        vec![SoundChunkMeta::schema()]
    }
}

// ── font ─────────────────────────────────────────────────────────────────

/// The named code point sets a font's `.ogt` takes (`font { charset … }`),
/// in the order [`charset_words`] names them.
pub const CHARSET_WORDS: [&str; 8] = [
    "latin",
    "ascii",
    "punct",
    "cjk-punct",
    "kana",
    "hangul",
    "arabic",
    "all",
];

/// The code points a charset word names.
// A charset is a list of ranges and some charsets are one range: clippy's
// single_range_in_vec_init reads `vec![a..=b]` as a mistaken `(a..=b).collect()`.
#[allow(clippy::single_range_in_vec_init)]
pub fn charset(word: &str) -> Option<Vec<std::ops::RangeInclusive<u32>>> {
    Some(match word {
        "ascii" => vec![0x20..=0x7e],
        "latin" => vec![0x20..=0x7e, 0xa0..=0xff],
        "punct" => vec![0x2000..=0x206f],
        "cjk-punct" => vec![0x3000..=0x303f, 0xff00..=0xffef],
        "kana" => vec![0x3040..=0x30ff],
        "hangul" => vec![0x1100..=0x11ff, 0x3130..=0x318f, 0xac00..=0xd7af],
        "arabic" => vec![
            0x0600..=0x06ff,
            0x0750..=0x077f,
            0xfb50..=0xfdff,
            0xfe70..=0xfeff,
        ],
        "all" => vec![0..=0x10ffff],
        _ => return None,
    })
}

/// The charset words a cooked font's glyphs show (its `.ogt`'s
/// `font { charset … }` as far as the atlas tells: a word whose ranges hold
/// at least 8 of its glyphs not already named; `latin` over `ascii` when
/// Latin-1 is there), and how many glyphs no word names (its `chars`). The
/// cooked blob keeps the glyphs, not the words: what a missing-glyph
/// warning says of the font.
pub fn charset_words(codepoints: &[u32]) -> (Vec<&'static str>, usize) {
    let within =
        |word: &str, c: u32| charset(word).is_some_and(|r| r.iter().any(|r| r.contains(&c)));
    let mut named: Vec<&'static str> = Vec::new();
    let mut left: Vec<u32> = codepoints.to_vec();
    for word in CHARSET_WORDS {
        if word == "all" || (word == "ascii" && named.contains(&"latin")) {
            continue;
        }
        let hits = left.iter().filter(|&&c| within(word, c)).count();
        let enough = if word == "latin" {
            // Latin-1's upper half, beyond ASCII.
            left.iter().filter(|&&c| (0xa0..=0xff).contains(&c)).count() >= 8
        } else {
            hits >= 8
        };
        if enough {
            named.push(word);
            left.retain(|&c| !within(word, c));
        }
    }
    (named, left.len())
}

row!(
    /// The root of a font blob: metrics in em units, the atlas pages' size.
    FontMeta, "og.Font", 1 {
        units_per_em: u32,
        /// Atlas pixels per em.
        em_px: f32,
        /// The distance range in atlas pixels (the field spans +-range/2).
        px_range: f32,
        ascender: f32,
        descender: f32,
        line_gap: f32,
        glyph_count: u32,
        page_count: u32,
        page_width: u32,
        /// Every page's height but the last's, which is cut to the rows it
        /// uses (its KTX2 holds its own size).
        page_height: u32,
        /// Code points of the source font that were not cooked (outside the charset).
        uncooked_codepoints: u32,
    }
);

row!(
    /// One glyph, sorted by code point. Plane bounds are in em relative to
    /// the pen position (y up); atlas bounds in pixels of page `page`, rows
    /// top to bottom. A glyph with no outline (space) has empty bounds.
    GlyphRow, "og.FontGlyph", 1 {
        codepoint: u32,
        page: u32,
        advance: f32,
        plane_left: f32, plane_bottom: f32, plane_right: f32, plane_top: f32,
        atlas_x: u32, atlas_y: u32, atlas_w: u32, atlas_h: u32,
    }
);

row!(
    /// One kerning pair: `adjust` em added to the advance of `left` when
    /// `right` follows it (negative pulls them together).
    KernRow, "og.FontKern", 1 {
        left: u32,
        right: u32,
        adjust: f32,
    }
);

/// The font blob.
pub struct Font;
impl BlobKind for Font {
    const NAME: &'static str = "og.Font";
    fn schemas() -> Vec<Schema> {
        vec![FontMeta::schema(), GlyphRow::schema(), KernRow::schema()]
    }
}
