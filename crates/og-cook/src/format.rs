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
//! | sound (`og.Sound`) | [`SoundMeta`] | `qoa` (a QOA file) |
//! | font (`og.Font`) | [`FontMeta`] | `glyphs` [`GlyphRow`], `atlas.<n>` (KTX2 RGBA8, zlib) |
//! | scene (`og.Scene`) | [`SceneMeta`] | `entities` [`EntityRow`], `refs` [`RefRow`], `components` [`ComponentRow`], `strings`, per component `c.<Type>` rows and `s.<Type>` its schema JSON |
//! | animation (`og.Animation`, a glTF's `#anim`) | [`AnimMeta`] | `nodes` [`NodeRow`], `skins` [`SkinRow`], `joints` (u32 node indices), `inverse_binds` (16 f32 per joint, column-major), `clips` [`ClipRow`], `channels` [`ChannelRow`], `samples` (f32), `events` [`EventRow`], `skin_streams` [`SkinStreamRow`], `skin_data` (meshopt streams), `morphs` [`MorphRow`], `targets` [`TargetRow`], `morph_data` (meshopt streams), `strings` |

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
