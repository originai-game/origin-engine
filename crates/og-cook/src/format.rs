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
//! | blob | root | sections |
//! |---|---|---|
//! | model (`og.Model`) | [`ModelMeta`] | `primitives` [`PrimitiveRow`], `lods` [`LodRow`], `meshes` [`MeshRow`], `materials` [`MaterialRow`], `textures` [`TextureRefRow`], `nodes` [`NodeRow`], `scenes` [`SceneRow`], `roots` (u32 node indices), `strings`, `vertices`, `indices` |
//! | texture (`og.Texture`) | [`TextureMeta`] | `ktx2` (a KTX2 file) |
//! | sound (`og.Sound`; a `.wav`, or a procedural preset: an `.ogt` of `type sfx` synthesised by the cook) | [`SoundMeta`] | `qoa` (a QOA file) |
//! | font (`og.Font`) | [`FontMeta`] | `glyphs` [`GlyphRow`], `atlas.<n>` (KTX2 RGBA8, zlib) |
//! | scene (`og.Scene`) | [`SceneMeta`] | `entities` [`EntityRow`], `refs` [`RefRow`], `components` [`ComponentRow`], `strings`, per component `c.<Type>` rows and `s.<Type>` its schema JSON |
//! | colour LUT (`og.Lut`, a `.cube`) | [`LutMeta`] | `rgba` (size³ RGBA8, red fastest) |
//! | particle emitter (`og.ParticleEmitter`, an `.ogt` of `type particles`) | [`EmitterMeta`] | `text` (the validated emitter file) |
//! | UI skin (`og.UiSkin`, an `.ogt` of `type skin`) | [`SkinMeta`] | `text` (the validated skin file) |
//! | material shader (`og.MaterialShader`, an `.ogt` of `type shader`) | [`ShaderMeta`] | `text` (the canonical shader file, code inlined) |
//! | flipbook sheet (`og.Flipbook`, an `.ogt` of `type flipbook`) | [`FlipbookMeta`] | `text` (the canonical sheet: frames resolved from the grid and an Aseprite or TexturePacker atlas, clips with frame indices and durations, the texture's size) |
//! | animation (`og.Animation`, a glTF's `#anim`) | [`AnimMeta`] | `nodes` [`NodeRow`], `skins` [`SkinRow`], `joints` (u32 node indices), `inverse_binds` (16 f32 per joint, column-major), `clips` [`ClipRow`], `channels` [`ChannelRow`], `samples` (f32), `events` [`EventRow`], `skin_streams` [`SkinStreamRow`], `skin_data` (meshopt streams), `morphs` [`MorphRow`], `targets` [`TargetRow`], `morph_data` (meshopt streams), `strings` |
//! | tileset (`og.Tileset`, an `.ogt` of `type tileset`) | [`TilesetMeta`] | `text` (the canonical tileset text) |
//! | tilemap (`og.Tilemap`, an `.ogt` of `type tilemap`, a Tiled `.tmj`, an LDtk `.ldtk`) | [`TilemapMeta`] | `layers` [`MapLayerRow`], `cells` (u32 per cell, layer after layer), `objects` [`MapObjectRow`], `props` [`MapPropRow`], `tilesets` [`MapTilesetRow`], `strings` |
//! | behaviour (`og.Behaviour`, an `.ogt` of `type behaviour`) | [`BehaviourMeta`] | `text` (the validated behaviour file) |
//! | navmesh (`og.NavMesh`, an `.ogt` of `type navmesh`) | [`NavMeshMeta`] | `columns` [`NavColumnRow`], `spans` [`NavSpanRow`] (the walkable heightfield), `tiles` [`NavTileRow`], `verts` [`NavVertRow`], `polys` [`NavPolyRow`], `poly_verts` (u32) |
//! | string table (`og.Strings`, an `.ogt` of `type strings`) | [`StringsMeta`] | `text` (the validated table) |
//! | humanoid rig (`og.Rig`, an `.ogt` of `type rig`) | [`RigMeta`] | `text` (the canonical rig: its scheme and the bones it names) |
//! | terrain (`og.Terrain`, an `.ogt` of `type terrain`) | [`TerrainMeta`] | `tiles` [`TerrainTileRow`] (og-render's tile pyramid), `heights` (each tile's `(tile + 3)²` u16 heights over the terrain's range, zlib), `control` (each tile's `(control + 2)²` RGBA8 texels, zlib), `layers` [`TerrainLayerRow`], `strings` (the layers' texture ids) |
//! | environment map (`og.EnvironmentMap`, an `.ogt` of `type environment`, phase 4) | [`EnvironmentMeta`] | `texels` (og-render's `EnvironmentMapData::texels`: RGB9E5 u32 LE, level after level, face after face, zlib), `irradiance` [`EnvironmentShRow`] (nine order-2 harmonics of the irradiance over pi) |
//! | streaming group pack (`og.StreamPack`, `cooked/stream/<group>.ogb`, phase 3) | [`StreamPackMeta`] | `files` [`StreamFileRow`], `names` (paths under `cooked/`), `data` (the files, each 16-aligned) |

use og_core::hash::StableHasher;
use og_core::{Reflect, Schema};

/// Changes whenever the bytes the cook writes for the same input change.
/// Part of every cook stamp, so a new cooker re-cooks everything.
pub const COOKER_VERSION: &str = concat!("og-cook ", env!("CARGO_PKG_VERSION"), " r2");

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
    PrimitiveRow, "og.ModelPrimitive", 1 {
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
        lod_first: u32,
        lod_count: u32,
        /// Dequantisation: `p = offset + q / 65535 * scale` (also the exact local bounds' minimum and size).
        pos_offset_x: f32, pos_offset_y: f32, pos_offset_z: f32,
        pos_scale_x: f32, pos_scale_y: f32, pos_scale_z: f32,
        uv_offset_x: f32, uv_offset_y: f32,
        uv_scale_x: f32, uv_scale_y: f32,
    }
);

row!(
    /// One level of detail: a range of a primitive's decoded indices.
    LodRow, "og.ModelLod", 1 {
        index_first: u32,
        index_count: u32,
        /// The simplifier's error relative to the primitive's extent (0 for LOD 0).
        error: f32,
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
    /// A glTF metallic-roughness material. Texture fields index the model's
    /// `textures` table ([`NONE`] when absent). The last material of every
    /// model is the default one primitives without a material use.
    MaterialRow, "og.ModelMaterial", 1 {
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
    NodeRow, "og.ModelNode", 1 {
        parent: u32,
        mesh: u32,
        name_offset: u32,
        name_len: u32,
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

/// The texture blob.
pub struct Texture;
impl BlobKind for Texture {
    const NAME: &'static str = "og.Texture";
    fn schemas() -> Vec<Schema> {
        vec![TextureMeta::schema()]
    }
}

// ── sound ────────────────────────────────────────────────────────────────

row!(
    /// The root of a sound blob; the `qoa` section is a QOA file.
    SoundMeta, "og.Sound", 1 {
        sample_rate: u32,
        channels: u32,
        /// Samples per channel.
        frames: u32,
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

// ── font ─────────────────────────────────────────────────────────────────

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

/// The font blob.
pub struct Font;
impl BlobKind for Font {
    const NAME: &'static str = "og.Font";
    fn schemas() -> Vec<Schema> {
        vec![FontMeta::schema(), GlyphRow::schema()]
    }
}

// ── scene ────────────────────────────────────────────────────────────────

row!(
    /// The root of a scene blob.
    SceneMeta, "og.Scene", 1 {
        entity_count: u32,
        component_count: u32,
        ref_count: u32,
        name_offset: u32,
        name_len: u32,
    }
);

row!(
    /// One entity in document order (a parent precedes its children).
    EntityRow, "og.SceneEntity", 1 {
        name_offset: u32,
        name_len: u32,
        parent: u32,
        /// The `refs` entry this entity instantiates (a model), or [`NONE`].
        instance: u32,
    }
);

row!(
    /// An asset the scene references, by asset id.
    RefRow, "og.SceneRef", 1 {
        id_offset: u32,
        id_len: u32,
    }
);

row!(
    /// One component table: `c.<name>` holds rows of `u32` entity index then
    /// the packed fields of `s.<name>` (the schema JSON).
    ComponentRow, "og.SceneComponent", 1 {
        name_offset: u32,
        name_len: u32,
        layout_hash_lo: u32,
        layout_hash_hi: u32,
        rows: u32,
        /// 1 when the cook had no schema for the type and inferred one from the text.
        inferred: u32,
    }
);

/// The scene blob.
pub struct Scene;
impl BlobKind for Scene {
    const NAME: &'static str = "og.Scene";
    fn schemas() -> Vec<Schema> {
        vec![
            SceneMeta::schema(),
            EntityRow::schema(),
            RefRow::schema(),
            ComponentRow::schema(),
        ]
    }
}

/// Reads `len` bytes at `offset` of a string pool.
pub fn pool_str(pool: &[u8], offset: u32, len: u32) -> Option<&str> {
    let start = offset as usize;
    pool.get(start..start.checked_add(len as usize)?)
        .and_then(|b| std::str::from_utf8(b).ok())
}

// ── animation ────────────────────────────────────────────────────────────

row!(
    /// The root of an animation blob (`<model>#anim`): a glTF's skeleton
    /// (every node, in the model blob's order), skins, clips resampled at a
    /// fixed rate, the skinned primitives' joint streams and the morph
    /// targets, each primitive indexed as in the model blob.
    AnimMeta, "og.Animation", 1 {
        node_count: u32,
        skin_count: u32,
        joint_count: u32,
        clip_count: u32,
        channel_count: u32,
        event_count: u32,
        skin_stream_count: u32,
        morph_count: u32,
    }
);

row!(
    /// A glTF skin: a range of `joints` (and of `inverse_binds`, 16 floats each).
    SkinRow, "og.AnimSkin", 1 {
        joint_first: u32,
        joint_count: u32,
        /// glTF `skin.skeleton` as a node index, or [`NONE`].
        skeleton: u32,
        name_offset: u32,
        name_len: u32,
    }
);

row!(
    /// A clip: `frames` samples per channel at `rate` Hz from 0 to `duration`
    /// (the last at `duration`); `source_rate` is 1 / the source's smallest key interval.
    ClipRow, "og.AnimClip", 1 {
        name_offset: u32,
        name_len: u32,
        duration: f32,
        rate: f32,
        source_rate: f32,
        frames: u32,
        channel_first: u32,
        channel_count: u32,
        event_first: u32,
        event_count: u32,
    }
);

/// [`ChannelRow::path`] values (`og_anim::ChannelPath::code`).
pub mod path {
    pub const TRANSLATION: u32 = 0;
    pub const ROTATION: u32 = 1;
    pub const SCALE: u32 = 2;
    pub const WEIGHTS: u32 = 3;
}

/// [`ChannelRow::interpolation`] values: the source sampler's.
pub mod interpolation {
    pub const LINEAR: u32 = 0;
    pub const STEP: u32 = 1;
    pub const CUBICSPLINE: u32 = 2;
}

row!(
    /// One channel: `frames × lanes` floats of `samples` from `sample_offset`.
    ChannelRow, "og.AnimChannel", 1 {
        node: u32,
        path: u32,
        /// The source sampler; STEP channels hold each sample, the others interpolate.
        interpolation: u32,
        lanes: u32,
        sample_offset: u32,
    }
);

row!(
    /// A named marker of clip `clip` (from the glTF animation's
    /// `extras."og.events"` or the import settings).
    EventRow, "og.AnimEvent", 1 {
        clip: u32,
        time: f32,
        name_offset: u32,
        name_len: u32,
    }
);

row!(
    /// The skin attributes of one skinned model primitive, per cooked vertex
    /// (the model blob's order): joints `u16 x 4`, weights `u16 x 4` (unorm,
    /// summing to 65535), meshopt-encoded at stride 16 in `skin_data`.
    SkinStreamRow, "og.AnimSkinStream", 1 {
        primitive: u32,
        mesh: u32,
        vertex_count: u32,
        data_offset: u32,
        data_size: u32,
    }
);

row!(
    /// The morph targets of one model primitive: per target (in glTF order),
    /// per cooked vertex, the position delta `f32 x 3` then, when
    /// `normals` is 1, the normal delta `f32 x 3`; one meshopt stream of
    /// `target_count × vertex_count` elements in `morph_data`.
    MorphRow, "og.AnimMorph", 1 {
        primitive: u32,
        mesh: u32,
        vertex_count: u32,
        target_first: u32,
        target_count: u32,
        normals: u32,
        data_offset: u32,
        data_size: u32,
    }
);

row!(
    /// A morph target's name (glTF `mesh.extras.targetNames`; empty when unnamed)
    /// and its default weight (`mesh.weights`).
    TargetRow, "og.AnimTarget", 1 {
        name_offset: u32,
        name_len: u32,
        default_weight: f32,
    }
);

/// The animation blob.
pub struct Animation;
impl BlobKind for Animation {
    const NAME: &'static str = "og.Animation";
    fn schemas() -> Vec<Schema> {
        vec![
            AnimMeta::schema(),
            NodeRow::schema(),
            SkinRow::schema(),
            ClipRow::schema(),
            ChannelRow::schema(),
            EventRow::schema(),
            SkinStreamRow::schema(),
            MorphRow::schema(),
            TargetRow::schema(),
        ]
    }
}

// ── effects ──────────────────────────────────────────────────────────────

row!(
    /// The root of a colour lookup table blob (a `.cube` file).
    LutMeta, "og.Lut", 1 {
        /// The table's side (og-render's LUT_SIZE, 32).
        size: u32,
    }
);

/// The colour LUT blob: `rgba` holds size³ RGBA8 texels, red fastest.
pub struct Lut;
impl BlobKind for Lut {
    const NAME: &'static str = "og.Lut";
    fn schemas() -> Vec<Schema> {
        vec![LutMeta::schema()]
    }
}

row!(
    /// The root of a particle emitter blob (an `.ogt` of `type particles`).
    EmitterMeta, "og.ParticleEmitter", 1 {
        /// Bytes of the validated emitter text in `text`.
        text_bytes: u32,
    }
);

row!(
    /// The root of a UI skin blob (an `.ogt` of `type skin`).
    SkinMeta, "og.UiSkin", 1 {
        /// Bytes of the validated skin text in `text`.
        text_bytes: u32,
    }
);

/// The UI skin blob: `text` holds the skin file the cook validated
/// (og-ui's `parse_skin`), parsed again at load with the bundle's assets.
pub struct Skin;
impl BlobKind for Skin {
    const NAME: &'static str = "og.UiSkin";
    fn schemas() -> Vec<Schema> {
        vec![SkinMeta::schema()]
    }
}

row!(
    /// The root of a material shader blob (an `.ogt` of `type shader`).
    ShaderMeta, "og.MaterialShader", 1 {
        /// Bytes of the canonical shader text (code inlined) in `text`.
        text_bytes: u32,
    }
);

/// The material shader blob: `text` holds the shader file the cook
/// compiled for every back end (`crate::shader`), its code inlined.
pub struct Shader;
impl BlobKind for Shader {
    const NAME: &'static str = "og.MaterialShader";
    fn schemas() -> Vec<Schema> {
        vec![ShaderMeta::schema()]
    }
}

/// The particle emitter blob: `text` holds the emitter file the cook
/// validated (`crate::fx`), parsed again at load with the bundle's textures.
pub struct Emitter;
impl BlobKind for Emitter {
    const NAME: &'static str = "og.ParticleEmitter";
    fn schemas() -> Vec<Schema> {
        vec![EmitterMeta::schema()]
    }
}

// ── tilemaps ─────────────────────────────────────────────────────────────

row!(
    /// The root of a tileset blob (an `.ogt` of `type tileset`).
    TilesetMeta, "og.Tileset", 1 {
        /// Bytes of the canonical tileset text in `text`.
        text_bytes: u32,
    }
);

/// The tileset blob: `text` holds the canonical tileset text
/// (`og_tilemap::Tileset::to_text`), parsed again at load.
pub struct Tileset;
impl BlobKind for Tileset {
    const NAME: &'static str = "og.Tileset";
    fn schemas() -> Vec<Schema> {
        vec![TilesetMeta::schema()]
    }
}

row!(
    /// The root of a tilemap blob (an `.ogt` of `type tilemap`, a Tiled
    /// `.tmj` or an LDtk `.ldtk`).
    TilemapMeta, "og.Tilemap", 1 {
        width: u32,
        height: u32,
        tile_width_px: u32,
        tile_height_px: u32,
        /// World units per tile width.
        unit: f32,
        layer_count: u32,
        object_count: u32,
        tileset_count: u32,
        prop_count: u32,
    }
);

/// [`MapLayerRow::flags`] bits.
pub mod layer_flag {
    pub const VISIBLE: u32 = 1;
    pub const COLLISION: u32 = 2;
    pub const REPEAT_X: u32 = 4;
    pub const REPEAT_Y: u32 = 8;
    /// A terrain layer: `terrain_*` names its terrain.
    pub const TERRAIN: u32 = 16;
}

row!(
    /// One layer: its `width × height` cells start at `cell_offset` (in
    /// cells) of the `cells` section (u32 little-endian per cell, the
    /// `og_tilemap::Layer::cells` encoding).
    MapLayerRow, "og.TilemapLayer", 1 {
        name_offset: u32,
        name_len: u32,
        tileset: u32,
        terrain_offset: u32,
        terrain_len: u32,
        parallax_x: f32,
        parallax_y: f32,
        offset_x: f32,
        offset_y: f32,
        opacity: f32,
        flags: u32,
        z: i32,
        cell_offset: u32,
    }
);

row!(
    /// A map object in map-local world units (top-left corner, y up), with
    /// `prop_count` rows of `props` from `prop_first`.
    MapObjectRow, "og.TilemapObject", 1 {
        name_offset: u32,
        name_len: u32,
        kind_offset: u32,
        kind_len: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        prop_first: u32,
        prop_count: u32,
    }
);

row!(
    /// A key and value of an object's properties.
    MapPropRow, "og.TilemapProp", 1 {
        key_offset: u32,
        key_len: u32,
        value_offset: u32,
        value_len: u32,
    }
);

row!(
    /// A map's tileset: `embedded` 0, the asset id of a tileset blob in
    /// `strings`; 1, the canonical text of a tileset an importer made.
    MapTilesetRow, "og.TilemapTileset", 1 {
        embedded: u32,
        offset: u32,
        len: u32,
    }
);

/// The tilemap blob.
pub struct Tilemap;
impl BlobKind for Tilemap {
    const NAME: &'static str = "og.Tilemap";
    fn schemas() -> Vec<Schema> {
        vec![
            TilemapMeta::schema(),
            MapLayerRow::schema(),
            MapObjectRow::schema(),
            MapPropRow::schema(),
            MapTilesetRow::schema(),
        ]
    }
}

// ── flipbooks ────────────────────────────────────────────────────────────

row!(
    /// The root of a flipbook sheet blob (an `.ogt` of `type flipbook`).
    FlipbookMeta, "og.Flipbook", 1 {
        /// Bytes of the canonical sheet text in `text`.
        text_bytes: u32,
    }
);

/// The flipbook sheet blob: `text` holds the canonical sheet
/// (`og_anim::FlipbookSheet::to_text`), parsed again at load.
pub struct Flipbook;
impl BlobKind for Flipbook {
    const NAME: &'static str = "og.Flipbook";
    fn schemas() -> Vec<Schema> {
        vec![FlipbookMeta::schema()]
    }
}

// ── E6: behaviours and navmeshes ─────────────────────────────────────────

row!(
    /// The root of a behaviour blob (an `.ogt` of `type behaviour`).
    BehaviourMeta, "og.Behaviour", 1 {
        /// Bytes of the validated behaviour text in `text`.
        text_bytes: u32,
    }
);

/// The behaviour blob: `text` holds the behaviour file the cook validated
/// (og-ai's `Behaviour::parse`), parsed again at load.
pub struct Behaviour;
impl BlobKind for Behaviour {
    const NAME: &'static str = "og.Behaviour";
    fn schemas() -> Vec<Schema> {
        vec![BehaviourMeta::schema()]
    }
}

row!(
    /// The root of a rig blob (an `.ogt` of `type rig`).
    RigMeta, "og.Rig", 1 {
        /// Bytes of the canonical rig text in `text`.
        text_bytes: u32,
    }
);

/// The rig blob: `text` holds the canonical humanoid map
/// (`og_anim::RigAsset::to_text`), parsed again at load.
pub struct Rig;
impl BlobKind for Rig {
    const NAME: &'static str = "og.Rig";
    fn schemas() -> Vec<Schema> {
        vec![RigMeta::schema()]
    }
}

row!(
    /// The root of a navmesh blob (an `.ogt` of `type navmesh`): the
    /// heightfield's grid and the agent it was baked for.
    NavMeshMeta, "og.NavMesh", 1 {
        cell_size: f32,
        cell_height: f32,
        agent_height: f32,
        agent_radius: f32,
        agent_climb: f32,
        max_slope: f32,
        tile_size: u32,
        max_error: f32,
        origin_x: f32,
        origin_y: f32,
        origin_z: f32,
        width: u32,
        depth: u32,
        tiles: u32,
    }
);

row!(
    /// One column of the heightfield: its first span and span count.
    NavColumnRow, "og.NavMesh.column", 1 {
        first: u32,
        count: u32,
    }
);

row!(
    /// One walkable floor: its height and the headroom above it, in cell heights.
    NavSpanRow, "og.NavMesh.span", 1 {
        floor: u32,
        clear: u32,
    }
);

row!(
    /// One tile: its vertices and polygons, as ranges of `verts` and `polys`.
    NavTileRow, "og.NavMesh.tile", 1 {
        vert_first: u32,
        vert_count: u32,
        poly_first: u32,
        poly_count: u32,
    }
);

row!(
    /// One tile vertex: world position and lattice coordinates.
    NavVertRow, "og.NavMesh.vert", 1 {
        x: f32,
        y: f32,
        z: f32,
        lx: i32,
        lz: i32,
    }
);

row!(
    /// One polygon: its vertex indices as a range of `poly_verts` (u32,
    /// into its tile's vertices), and its region.
    NavPolyRow, "og.NavMesh.poly", 1 {
        index_first: u32,
        index_count: u32,
        region: u32,
    }
);

/// The navmesh blob: `meta`, `columns` and `spans` (the walkable
/// heightfield, so obstacles can rebuild tiles at runtime), and the tiles
/// the cook built: `tiles`, `verts`, `polys`, `poly_verts` (u32). Links are
/// made at load.
pub struct NavMesh;
impl BlobKind for NavMesh {
    const NAME: &'static str = "og.NavMesh";
    fn schemas() -> Vec<Schema> {
        vec![
            NavMeshMeta::schema(),
            NavColumnRow::schema(),
            NavSpanRow::schema(),
            NavTileRow::schema(),
            NavVertRow::schema(),
            NavPolyRow::schema(),
        ]
    }
}

// ── E6: string tables ────────────────────────────────────────────────────

row!(
    /// The root of a string table blob (an `.ogt` of `type strings`).
    StringsMeta, "og.Strings", 1 {
        /// Bytes of the validated table text in `text`.
        text_bytes: u32,
        /// Its entries.
        strings: u32,
    }
);

/// The string table blob: `text` holds the table the cook validated
/// (og-i18n's `StringTable::parse`), parsed again at load.
pub struct Strings;
impl BlobKind for Strings {
    const NAME: &'static str = "og.Strings";
    fn schemas() -> Vec<Schema> {
        vec![StringsMeta::schema()]
    }
}

// ── phase 2: terrain ─────────────────────────────────────────────────────

row!(
    /// The root of a terrain blob (an `.ogt` of `type terrain`).
    TerrainMeta, "og.Terrain", 2 {
        /// Extent along x and z in metres, centred on the terrain's origin.
        size_x: f32,
        size_z: f32,
        /// Height samples along x and z.
        columns: u32,
        rows: u32,
        /// og-render's TerrainLayout: quads per tile, control texels per
        /// tile (0: none) and pyramid levels.
        tile: u32,
        control: u32,
        levels: u32,
        /// The lowest and highest height (metres above the origin): every
        /// tile's 16-bit heights span this range.
        height_min: f32,
        height_max: f32,
        layers: u32,
        /// The control map's size (0: none).
        control_width: u32,
        control_height: u32,
        height_blend: f32,
        /// The clipmap's finest spacing (0: the samples') and quads per side (0: 64).
        lod_spacing: f32,
        lod_ring: u32,
        /// 1: casts shadows; 1: the game gets a collider.
        cast_shadows: u32,
        collision: u32,
    }
);

row!(
    /// One tile of the pyramid (in TerrainLayout's index order): its
    /// level and place, the range of its heights, its zlib'd 16-bit
    /// heights in `heights` and its zlib'd control texels in `control`.
    TerrainTileRow, "og.Terrain.tile", 2 {
        index: u32,
        level: u32,
        x: u32,
        z: u32,
        min: f32,
        max: f32,
        offset: u32,
        bytes: u32,
        control_offset: u32,
        control_bytes: u32,
    }
);

row!(
    /// One layer: its textures (ranges of `strings`; [`NONE`] for none)
    /// and og-render's `TerrainLayer` fields.
    TerrainLayerRow, "og.Terrain.layer", 1 {
        albedo_at: u32,
        albedo_len: u32,
        normal_at: u32,
        normal_len: u32,
        r: f32,
        g: f32,
        b: f32,
        roughness: f32,
        tile: f32,
        triplanar: u32,
        normal_strength: f32,
        slope_min: f32,
        slope_max: f32,
        slope_fade: f32,
        altitude_min: f32,
        altitude_max: f32,
        altitude_fade: f32,
    }
);

/// The terrain blob: `meta`, `tiles` with their zlib'd heights in
/// `heights` and control texels in `control`, `layers` naming textures in
/// `strings`.
pub struct Terrain;
impl BlobKind for Terrain {
    const NAME: &'static str = "og.Terrain";
    fn schemas() -> Vec<Schema> {
        vec![
            TerrainMeta::schema(),
            TerrainTileRow::schema(),
            TerrainLayerRow::schema(),
        ]
    }
}

// ── phase 4: environment maps ────────────────────────────────────────────

row!(
    /// The root of an environment map blob (an `.ogt` of `type environment`):
    /// og-render's `EnvironmentMapData` and what it was baked from.
    EnvironmentMeta, "og.EnvironmentMap", 1 {
        /// Level 0's face size (a power of two) and the level count
        /// (og-render's `ENVIRONMENT_MAP_LEVELS`; level m is roughness
        /// m / (levels - 1)).
        size: u32,
        levels: u32,
        /// RGB9E5 texels in `texels` once inflated.
        texel_count: u32,
        /// The equirectangular source's size.
        source_width: u32,
        source_height: u32,
        /// The intensity and the turn about +y (degrees) baked in.
        intensity: f32,
        rotation: f32,
        /// GGX samples per texel of each prefiltered level.
        samples: u32,
    }
);

row!(
    /// One harmonic coefficient of the irradiance over pi (og-render's
    /// `EnvironmentMapData::irradiance`), nine in order.
    EnvironmentShRow, "og.EnvironmentMap.sh", 1 {
        r: f32,
        g: f32,
        b: f32,
    }
);

/// The environment map blob: `meta`, `texels` (zlib'd RGB9E5) and `irradiance`.
pub struct EnvironmentMap;
impl BlobKind for EnvironmentMap {
    const NAME: &'static str = "og.EnvironmentMap";
    fn schemas() -> Vec<Schema> {
        vec![EnvironmentMeta::schema(), EnvironmentShRow::schema()]
    }
}

// ── phase 3: streaming groups ────────────────────────────────────────────

row!(
    /// The root of a streaming group's pack (`cooked/stream/<group>.ogb`):
    /// the cooked files of the group's assets, one download.
    StreamPackMeta, "og.StreamPack", 1 {
        files: u32,
    }
);

row!(
    /// One file of a pack: its path under `cooked/` (a range of `names`)
    /// and its bytes (a range of `data`, 16-aligned).
    StreamFileRow, "og.StreamPack.file", 1 {
        name_at: u32,
        name_len: u32,
        data_at: u32,
        data_len: u32,
    }
);

/// A streaming group's pack: `meta`, `files` naming ranges of `names` and
/// `data`.
pub struct StreamPack;
impl BlobKind for StreamPack {
    const NAME: &'static str = "og.StreamPack";
    fn schemas() -> Vec<Schema> {
        vec![StreamPackMeta::schema(), StreamFileRow::schema()]
    }
}
