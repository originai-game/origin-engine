use super::*;

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
    /// Kept out of 2D lighting (`og_tilemap::Layer::lit` false). A clear
    /// bit is lit, so blobs cooked before it light their layers.
    pub const UNLIT: u32 = 32;
    /// One collider per solid cell off the orthogonal grid
    /// (`og_tilemap::Layer::cell_colliders`, G2-6). A clear bit merges
    /// them, so blobs cooked before it keep their bytes.
    pub const CELL_COLLIDERS: u32 = 64;
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
        /// `og_tilemap::Layer::collider_layers` (0: the og.Tilemap's).
        collider_layers: u32,
        /// `og_tilemap::Layer::collider_mask` (0: every layer).
        collider_mask: u32,
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

// ── 2D rigs ──────────────────────────────────────────────────────────────

row!(
    /// The root of a 2D rig blob (an `.ogt` of `type rig2d`).
    Rig2DMeta, "og.Rig2D", 1 {
        /// Bytes of the canonical rig text in `text`.
        text_bytes: u32,
    }
);

/// The 2D rig blob: `text` holds the canonical rig
/// (`og_anim::Rig2dAsset::to_text`, a DragonBones export already
/// converted), parsed again at load.
pub struct Rig2D;
impl BlobKind for Rig2D {
    const NAME: &'static str = "og.Rig2D";
    fn schemas() -> Vec<Schema> {
        vec![Rig2DMeta::schema()]
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
    /// The root of a dialogue blob (an `.ogt` of `type dialogue`, G2-8).
    DialogueMeta, "og.Dialogue", 1 {
        /// Bytes of the validated dialogue text in `text`.
        text_bytes: u32,
    }
);

/// The dialogue blob: `text` holds the dialogue file the cook validated
/// (og-story's `Dialogue::parse`), parsed again at load.
pub struct Dialogue;
impl BlobKind for Dialogue {
    const NAME: &'static str = "og.Dialogue";
    fn schemas() -> Vec<Schema> {
        vec![DialogueMeta::schema()]
    }
}

row!(
    /// The root of a quest blob (an `.ogt` of `type quest`, G2-8).
    QuestMeta, "og.Quest", 1 {
        /// Bytes of the validated quest text in `text`.
        text_bytes: u32,
    }
);

/// The quest blob: `text` holds the quest file the cook validated
/// (og-story's `Quest::parse`), parsed again at load.
pub struct Quest;
impl BlobKind for Quest {
    const NAME: &'static str = "og.Quest";
    fn schemas() -> Vec<Schema> {
        vec![QuestMeta::schema()]
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
    /// The root of a look blob (an `.ogt` of `type look`).
    LookMeta, "og.Look", 1 {
        /// Bytes of the canonical look text in `text`.
        text_bytes: u32,
    }
);

/// The look blob: `text` holds the look the cook flattened and checked
/// against the resources' reflected schema (`crate::look::LookFile`),
/// parsed again at load, where the assets it names are loaded.
pub struct Look;
impl BlobKind for Look {
    const NAME: &'static str = "og.Look";
    fn schemas() -> Vec<Schema> {
        vec![LookMeta::schema()]
    }
}

row!(
    /// The root of a palette blob (a `.hex` or `.gpl` file).
    PaletteMeta, "og.Palette", 1 {
        /// Colours in `rgb`.
        count: u32,
    }
);

/// The palette blob: `rgb` holds `count` sRGB colours, three bytes each.
pub struct Palette;
impl BlobKind for Palette {
    const NAME: &'static str = "og.Palette";
    fn schemas() -> Vec<Schema> {
        vec![PaletteMeta::schema()]
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

// ── depth S2: timelines ──────────────────────────────────────────────────

row!(
    /// The root of a timeline blob (an `.ogt` of `type timeline`).
    TimelineMeta, "og.TimelineAsset", 1 {
        /// Bytes of the canonical timeline text in `text`.
        text_bytes: u32,
        /// Its tracks.
        tracks: u32,
        /// Its duration in seconds.
        duration: f32,
    }
);

/// The timeline blob: `text` holds the canonical timeline
/// (`og_timeline::Timeline::to_text`, its caption files imported as keys),
/// parsed again at load.
pub struct Timeline;
impl BlobKind for Timeline {
    const NAME: &'static str = "og.TimelineAsset";
    fn schemas() -> Vec<Schema> {
        vec![TimelineMeta::schema()]
    }
}

// ── depth S7: baked lighting ─────────────────────────────────────────────

row!(
    /// The root of a lightmap blob (an `.ogt` of `type lightmap`).
    LightmapMeta, "og.LightmapAsset", 1 {
        /// Bytes of the canonical lightmap text in `text`.
        text_bytes: u32,
        /// Bytes of the pages' KTX2 file in `pages`.
        pages_bytes: u32,
        /// Static model entities drawn with it.
        models: u32,
        /// Lightmap pages (array layers).
        pages: u32,
        /// The pages' side in texels.
        texels: u32,
        /// Light volumes and their probes.
        volumes: u32,
        probes: u32,
    }
);

/// The lightmap blob: `text` holds the canonical lightmap file
/// (`crate::lightmap::LightmapFile::to_text`), `pages` its pages (KTX2,
/// RGBA16F array layers).
pub struct Lightmap;
impl BlobKind for Lightmap {
    const NAME: &'static str = "og.Lightmap";
    fn schemas() -> Vec<Schema> {
        vec![LightmapMeta::schema()]
    }
}

// ── phase 2: terrain ─────────────────────────────────────────────────────

row!(
    /// The root of a terrain blob (an `.ogt` of `type terrain`).
    TerrainMeta, "og.Terrain", 6 {
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
        /// The texture arrays the layers' albedos and normal maps come
        /// from (`layers`, `layerNormals`: asset ids, ranges of `strings`;
        /// [`NONE`] for none).
        albedo_array_at: u32,
        albedo_array_len: u32,
        normal_array_at: u32,
        normal_array_len: u32,
        /// 1: the field was made by `generate`, not read from a heightmap.
        generated: u32,
        /// 1: a generated terrain's `sea`, at `sea_level` metres above the
        /// origin (og.terrain.load's TerrainInfo.seaLevel).
        sea: u32,
        sea_level: f32,
        /// A generated terrain's rivers, in metres along their samples.
        river_length: f32,
        /// Cells cut out as holes (0: none). With holes, each tile
        /// height's lowest bit is its sample's cell's hole flag on the
        /// tile's level (og-render's `mark_holes`).
        holes: u32,
        /// The control map's pages of four channels (one per four
        /// layers; each tile's control texels hold them one after the
        /// other).
        control_pages: u32,
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
    /// One layer: its textures (ranges of `strings`; [`NONE`] for none),
    /// or with the terrain's arrays its layers of them, and og-render's
    /// `TerrainLayer` fields.
    TerrainLayerRow, "og.Terrain.layer", 2 {
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
        /// With the terrain's arrays: this layer's layer of each ([`NONE`]: none).
        albedo_layer: u32,
        normal_layer: u32,
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
        /// GGX samples per texel of each prefiltered level (0: a KTX2's
        /// own prefiltered levels, `prefiltered true`, taken as they are).
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
