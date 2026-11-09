use super::*;

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

row!(
    /// Where one entity came from (the optional `prefabs` table, one row
    /// per entity in entity order, present when any entity was placed
    /// from a prefab): `prefab` is its `prefab_refs` entry (the prefab
    /// placed, the outermost for a nested one) or [`NONE`] for an
    /// authored entity, `node` its index in that prefab's own table and
    /// `root` the entity index of its instance's root.
    PrefabRow, "og.ScenePrefab", 1 {
        prefab: u32,
        node: u32,
        root: u32,
    }
);

row!(
    /// A sub-scene placed under an entity (the optional `subscenes` table,
    /// one row per entity with a `scene "<id>"` line, in entity order):
    /// `entity` is the entity's index and `scene` its `subscene_refs`
    /// entry, the placed scene's asset id. The placed scene stays its own
    /// blob (it is not flattened): the runtime spawns it under the entity,
    /// with it or, for a streaming cell (`og.StreamCell`), when the cell
    /// loads.
    SubsceneRow, "og.SceneSubscene", 1 {
        entity: u32,
        scene: u32,
    }
);

row!(
    /// Where an entity is declared (the optional `lines` table, one row
    /// per entity in entity order, written outside a publish cook): the
    /// 1-based line of its `entity` in the scene's text, or for a node an
    /// instance placed from a prefab the instance's own line; 0 for none.
    /// What the probe answers as a scene entity's source
    /// (`scenes/main.ogt:12`) on every host, with no source text to read.
    SceneLineRow, "og.SceneLine", 1 {
        line: u32,
    }
);

/// The scene blob. A prefab's blob is one too (manifest type
/// `og.Scene`), its entities all linked to it.
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
    AnimMeta, "og.Animation", 2 {
        node_count: u32,
        skin_count: u32,
        joint_count: u32,
        clip_count: u32,
        channel_count: u32,
        event_count: u32,
        skin_stream_count: u32,
        morph_count: u32,
        /// Nodes hidden at rest (`hidden`: glTF `KHR_node_visibility`).
        hidden_count: u32,
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
    /// One lane, 0 hidden or 1 visible, STEP (`OG_animation_visibility`).
    pub const VISIBILITY: u32 = 4;
}

/// [`ChannelRow::interpolation`] values: the source sampler's.
pub mod interpolation {
    pub const LINEAR: u32 = 0;
    pub const STEP: u32 = 1;
    pub const CUBICSPLINE: u32 = 2;
}

row!(
    /// One channel of a clip's `frames` samples, held as its `codec` says
    /// (`crate::anim_codec`): `frames × lanes` floats of `samples` from
    /// `sample_offset` (f32), one sample there (constant), or `keys`
    /// quantised keys in `packed` from `packed_offset` with their per-lane
    /// range in `samples` (quantised; the loader rebuilds every frame).
    ChannelRow, "og.AnimChannel", 2 {
        node: u32,
        path: u32,
        /// The source sampler; STEP channels hold each sample, the others interpolate.
        interpolation: u32,
        lanes: u32,
        sample_offset: u32,
        /// `crate::anim_codec::codec`: 0 f32, 1 constant, 2 quantised.
        codec: u32,
        /// Kept keys: `frames` for f32, 1 for constant.
        keys: u32,
        /// Quantised: the first u16 of `packed`.
        packed_offset: u32,
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
    /// The morph targets of one model primitive, sparse: per target (in
    /// glTF order) only the cooked vertices it moves (a position or normal
    /// delta with any bit set), each the position delta `f32 x 3` then,
    /// when `normals` is 1, the normal delta `f32 x 3`; one meshopt stream
    /// of every target's moved vertices in `morph_data`
    /// (`data_offset`/`data_size`, `moved` elements), and their vertex
    /// indices (u32, ascending per target) as one meshopt stream at stride
    /// 4 (`index_offset`/`index_size`). A vertex a target does not list
    /// has a zero delta.
    MorphRow, "og.AnimMorph", 2 {
        primitive: u32,
        mesh: u32,
        vertex_count: u32,
        target_first: u32,
        target_count: u32,
        normals: u32,
        data_offset: u32,
        data_size: u32,
        /// Moved vertices over every target (the streams' element count).
        moved: u32,
        index_offset: u32,
        index_size: u32,
    }
);

row!(
    /// A morph target's name (glTF `mesh.extras.targetNames`; empty when
    /// unnamed), its default weight (`mesh.weights`) and its range of its
    /// morph's moved vertices (`moved_first`, `moved_count`).
    TargetRow, "og.AnimTarget", 2 {
        name_offset: u32,
        name_len: u32,
        default_weight: f32,
        moved_first: u32,
        moved_count: u32,
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
    /// The root of an effect material blob (an `.ogt` of `type effect`).
    EffectMeta, "og.EffectMaterial", 1 {
        /// Bytes of the validated effect text in `text`.
        text_bytes: u32,
    }
);

row!(
    /// The root of a composed effect blob (an `.ogt` of `type vfx`).
    VfxMeta, "og.Vfx", 1 {
        /// Bytes of the validated vfx text in `text`.
        text_bytes: u32,
    }
);

row!(
    /// The root of a CSS pack blob, named by its directory.
    StyleMeta, "og.UiStyle", 2 {
        /// Bytes of the JSON array of (CSS asset ID, source text) pairs.
        text_bytes: u32,
    }
);

/// The CSS pack blob: `text` holds source pairs validated with the same
/// CSS parser the runtime uses. There is no author-facing manifest.
pub struct Style;
impl BlobKind for Style {
    const NAME: &'static str = "og.UiStyle";
    fn schemas() -> Vec<Schema> {
        vec![StyleMeta::schema()]
    }
}

row!(
    /// The root of a scriptless Svelte UI document blob (`.svelte`).
    DocumentMeta, "og.UiDocument", 2 {
        /// Bytes of the validated document text in `text`.
        text_bytes: u32,
    }
);

/// The UI document blob: `text` holds the document the cook validated
/// against og-ui's reflected props (`og_ui::UiDocument::parse`), parsed
/// again at load (docs/ui.md §2.1).
pub struct Document;
impl BlobKind for Document {
    const NAME: &'static str = "og.UiDocument";
    fn schemas() -> Vec<Schema> {
        vec![DocumentMeta::schema()]
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

row!(
    /// The root of a custom post pass blob (an `.ogt` of `type post`).
    PostMeta, "og.PostPass", 1 {
        /// Bytes of the canonical post file (code inlined) in `text`.
        text_bytes: u32,
    }
);

/// The custom post pass blob: `text` holds the post file the cook compiled
/// for every back end (`crate::shader`), its code inlined.
pub struct Post;
impl BlobKind for Post {
    const NAME: &'static str = "og.PostPass";
    fn schemas() -> Vec<Schema> {
        vec![PostMeta::schema()]
    }
}

row!(
    /// The root of a custom UI material blob (an `.ogt` of `type ui-material`).
    UiMaterialMeta, "og.UiMaterial", 1 {
        /// Bytes of the canonical UI material file (code inlined) in `text`.
        text_bytes: u32,
    }
);

/// The custom UI material blob (`docs/ui.md` §3.3): `text` holds the
/// material file the cook compiled for every back end (`crate::shader`),
/// its code inlined.
pub struct UiMaterial;
impl BlobKind for UiMaterial {
    const NAME: &'static str = "og.UiMaterial";
    fn schemas() -> Vec<Schema> {
        vec![UiMaterialMeta::schema()]
    }
}

row!(
    /// The root of a vector blob (an `.ogt` of `type vector`, `docs/ui.md`
    /// §5.1): its view box and counts.
    VectorMeta, "og.Vector", 1 {
        x: f32, y: f32, width: f32, height: f32,
        /// `og_render::UiVectorVertex`es in `vertices` (24 bytes each).
        vertex_count: u32,
        /// `u32` triangle indices in `indices`.
        index_count: u32,
        paint_count: u32,
    }
);

row!(
    /// A vector's paint: `kind` 0 solid (`r g b a`, linear, straight
    /// alpha), 1 linear, 2 radial, 3 conic, 4 diamond gradient (`from`,
    /// `to` in view-box units, its stops at `first_stop` in `stops`);
    /// `spread` 0 pad, 1 repeat, 2 reflect.
    VectorPaintRow, "og.VectorPaint", 1 {
        kind: u32,
        spread: u32,
        dither: u32,
        first_stop: u32,
        stop_count: u32,
        r: f32, g: f32, b: f32, a: f32,
        from_x: f32, from_y: f32, to_x: f32, to_y: f32,
    }
);

row!(
    /// A gradient stop of a vector's paint (linear RGBA, straight alpha).
    VectorStopRow, "og.VectorStop", 1 {
        offset: f32,
        r: f32, g: f32, b: f32, a: f32,
    }
);

/// The vector blob: `meta`, `paints`, `stops`, and raw `vertices` and
/// `indices`, the triangles og-render's UI pass draws.
pub struct Vector;
impl BlobKind for Vector {
    const NAME: &'static str = "og.Vector";
    fn schemas() -> Vec<Schema> {
        vec![
            VectorMeta::schema(),
            VectorPaintRow::schema(),
            VectorStopRow::schema(),
        ]
    }
}

row!(
    /// The root of a Rive artboard blob (`docs/ui.md` §3.3): the bytes of
    /// the converted artboard in `artboard` (og-ui's `rive::encode`).
    RiveArtboardMeta, "og.RiveArtboard", 1 {
        bytes: u32,
    }
);

/// A Rive artboard converted at cook time (an artboard of a `.riv`, id
/// `<file>.riv#<artboard>`): `meta` and the raw `artboard` bytes, which
/// og-ui's `rive::decode` reads and checks.
pub struct RiveArtboard;
impl BlobKind for RiveArtboard {
    const NAME: &'static str = "og.RiveArtboard";
    fn schemas() -> Vec<Schema> {
        vec![RiveArtboardMeta::schema()]
    }
}

/// The effect material blob: `text` holds the effect file the cook
/// validated (`crate::effect`), parsed again at load with the bundle's
/// textures and shaders.
pub struct Effect;
impl BlobKind for Effect {
    const NAME: &'static str = "og.EffectMaterial";
    fn schemas() -> Vec<Schema> {
        vec![EffectMeta::schema()]
    }
}

/// The composed effect blob: `text` holds the vfx file the cook validated
/// (`crate::vfx`), parsed again at load.
pub struct Vfx;
impl BlobKind for Vfx {
    const NAME: &'static str = "og.Vfx";
    fn schemas() -> Vec<Schema> {
        vec![VfxMeta::schema()]
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
