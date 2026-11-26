//! Terrain source data and ordered processing, without embedded author scripts.
use super::{ContentRef, InputSchema, JsonSchema, OnceLock, Reflected};
use crate::cook::{generator::Heightfield, source_schemas::world::geometry::Geometry};
use serde::Deserialize;

/// A terrain's source, processing and material declarations. Sample counts,
/// ascending bands, channels, layer indices and projected edits remain domain
/// validation; this does not run or replace the existing terrain cooker.
#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerrainInput {
    pub source: TerrainSource,
    /// Shared seed for the noise source and seeded processing stages.
    pub seed: Option<u32>,
    /// Positive world extents along X and Z, in metres.
    #[schemars(extend("items" = {"type": "number", "format": "float", "exclusiveMinimum": 0}, "x-og-unit" = "m"))]
    pub size: Option<[f32; 2]>,
    pub process: Option<TerrainProcess>,
    /// RGBA control images, one per four layers. Exclusive with biome rules.
    #[schemars(length(max = 4))]
    pub control: Option<Vec<String>>,
    /// Raw hole image relative to the owning module: black/transparent cuts cells.
    pub holes: Option<String>,
    #[schemars(range(min = 16, max = 1024), extend("x-og-unit" = "quads"))]
    pub tile: Option<u32>,
    #[schemars(range(min = 0))]
    pub height_blend: Option<f32>,
    pub cast_shadows: Option<bool>,
    pub collision: Option<bool>,
    pub lod: Option<TerrainLod>,
    #[schemars(length(max = 16))]
    pub layers: Option<Vec<TerrainLayerInput>>,
    #[schemars(extend("x-og-ref-kinds" = ["texture-array"]))]
    pub albedo_array: Option<ContentRef>,
    #[schemars(extend("x-og-ref-kinds" = ["texture-array"]))]
    pub normal_array: Option<ContentRef>,
    /// Brushes and stamps in their written order, after generation/processing.
    pub edits: Option<Vec<TerrainEdit>>,
}

impl TerrainInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<TerrainInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("terrain input schema"))
    }
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TerrainSource {
    Image {
        /// PNG/EXR source relative to the owning module; first channel is height.
        source: String,
        /// Metres corresponding to image intensities zero and one.
        #[schemars(extend("x-og-unit" = "m"))]
        height: Option<[f32; 2]>,
    },
    /// An already evaluated TS heightfield, never a script path or text program.
    Heightfield {
        /// The shared heightfield's domain validator checks rows × columns and finiteness.
        field: Heightfield,
    },
    Noise {
        #[schemars(range(min = 2, max = 4097))]
        columns: Option<u32>,
        #[schemars(range(min = 2, max = 4097))]
        rows: Option<u32>,
        #[schemars(extend("x-og-unit" = "m"))]
        height: Option<[f32; 2]>,
        noise: Option<TerrainNoise>,
        mask: Option<TerrainMask>,
    },
}

input! {
    TerrainNoise {
        #[schemars(range(min = 1, max = 12))]
        octaves: Option<u32>,
        /// Noise features across the longer world extent.
        #[schemars(range(min = 0.001, max = 10000))]
        frequency: Option<f32>,
        #[schemars(range(min = 1, max = 8))]
        lacunarity: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        gain: Option<f32>,
        #[schemars(range(min = 0, max = 100))]
        warp: Option<f32>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TerrainMask {
    None,
    Island {
        #[schemars(range(max = 1), extend("exclusiveMinimum" = 0))]
        falloff: Option<f32>,
    },
    Ridge {
        #[schemars(range(min = 0, max = 1))]
        strength: Option<f32>,
    },
}
input! {
    /// Processing runs in domain order: hydraulic, thermal, terraces, shore,
    /// drainage, authored channels and biome controls. It never evaluates TS.
    TerrainProcess {
        erosion: Option<HydraulicInput>,
        thermal: Option<ThermalInput>,
        terraces: Option<TerracesInput>,
        #[schemars(extend("x-og-unit" = "m"))]
        sea: Option<f32>,
        #[schemars(range(min = 0, max = 1_000_000), extend("x-og-unit" = "m"))]
        shore: Option<f32>,
        rivers: Option<RiversInput>,
        channels: Option<Vec<TerrainChannel>>,
        #[schemars(extend("x-og-ref-kinds" = ["field"]))]
        preserve: Option<ContentRef>,
        #[schemars(range(max = 4097), extend("x-og-unit" = "samples"))]
        preserve_border: Option<u32>,
        moisture: Option<MoistureInput>,
        biomes: Option<Vec<BiomeInput>>,
    }
}
input! {
    HydraulicInput {
        #[schemars(range(max = 10_000_000))]
        drops: Option<u32>,
        #[schemars(range(min = 1, max = 1000), extend("x-og-unit" = "steps"))]
        lifetime: Option<u32>,
        #[schemars(range(min = 1, max = 16), extend("x-og-unit" = "samples"))]
        radius: Option<u32>,
        #[schemars(range(min = 0, max = 1))]
        inertia: Option<f32>,
        #[schemars(range(min = 0, max = 1000))]
        capacity: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        deposition: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        erosion: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        evaporation: Option<f32>,
        #[schemars(range(min = 0, max = 1000))]
        gravity: Option<f32>,
    }
}
input! {
    ThermalInput {
        #[schemars(range(min = 0, max = 89), extend("x-og-unit" = "degrees"))]
        talus: Option<f32>,
        #[schemars(range(max = 10000))]
        iterations: Option<u32>,
        #[schemars(range(min = 0, max = 1))]
        rate: Option<f32>,
    }
}
input! {
    TerracesInput {
        #[schemars(range(min = 0.001, max = 1_000_000), extend("x-og-unit" = "m"))]
        step: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        blend: Option<f32>,
    }
}
input! {
    RiversInput {
        #[schemars(range(min = 1))]
        threshold: Option<u32>,
        #[schemars(range(min = 0, max = 1_000_000), extend("x-og-unit" = "m"))]
        depth: Option<f32>,
        #[schemars(range(min = 0, max = 1_000_000), extend("x-og-unit" = "m"))]
        width: Option<f32>,
    }
}
input! {
    TerrainChannel {
        /// Consecutive XZ points must differ; river Y must not rise downstream.
        #[schemars(length(min = 2), extend("x-og-unit" = "m"))]
        points: Vec<[f32; 3]>,
        #[schemars(range(min = 0.001, max = 1_000_000), extend("x-og-unit" = "m"))]
        width: f32,
        #[schemars(range(min = 0, max = 1_000_000), extend("x-og-unit" = "m"))]
        depth: Option<f32>,
        river: Option<bool>,
    }
}
input! {
    MoistureInput {
        #[schemars(range(min = 0, max = 1_000_000))]
        flow: Option<f32>,
        #[schemars(range(min = 0.001, max = 1_000_000_000))]
        scale: Option<f32>,
        #[schemars(range(min = 0, max = 1_000_000))]
        noise: Option<f32>,
        #[schemars(range(min = 0.001, max = 10000))]
        frequency: Option<f32>,
    }
}
input! {
    BiomeInput {
        #[schemars(range(max = 15))]
        layer: u32,
        #[schemars(range(min = 0, max = 1))]
        weight: Option<f32>,
        #[schemars(extend("x-og-unit" = "m"))]
        altitude: Option<[f32; 2]>,
        #[schemars(range(min = 0, max = 1_000_000), extend("x-og-unit" = "m"))]
        altitude_fade: Option<f32>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        slope: Option<[f32; 2]>,
        #[schemars(range(min = 0, max = 90), extend("x-og-unit" = "degrees"))]
        slope_fade: Option<f32>,
        moisture: Option<[f32; 2]>,
        #[schemars(range(min = 0, max = 1))]
        moisture_fade: Option<f32>,
    }
}
input! {
    TerrainLod {
        /// Zero uses the source sample spacing.
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        spacing: Option<f32>,
        #[schemars(extend("x-og-unit" = "quads"))]
        ring: Option<u32>,
    }
}
input! {
    TerrainLayerInput {
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        albedo: Option<ContentRef>,
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        normal: Option<ContentRef>,
        albedo_layer: Option<u32>,
        normal_layer: Option<u32>,
        color: Option<[f32; 3]>,
        #[schemars(range(min = 0, max = 1))]
        roughness: Option<f32>,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        tile: Option<f32>,
        triplanar: Option<bool>,
        #[schemars(range(min = 0, max = 4))]
        normal_strength: Option<f32>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        slope: Option<[f32; 2]>,
        #[schemars(range(min = 0), extend("x-og-unit" = "degrees"))]
        slope_fade: Option<f32>,
        #[schemars(extend("x-og-unit" = "m"))]
        altitude: Option<[f32; 2]>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        altitude_fade: Option<f32>,
        #[schemars(range(min = 0, max = 90), extend("x-og-unit" = "degrees"))]
        project_slope: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        anti_tiling: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        height_contrast: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        macro_variation: Option<f32>,
        #[schemars(range(min = 0, max = 4), extend("x-og-unit" = "m"))]
        displacement: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        cell_variation: Option<f32>,
    }
}

pub struct Transform;
impl super::super::reflected::RecordMarker for Transform {
    const NAME: &'static str = "Transform";
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TerrainEdit {
    Brush {
        mode: BrushMode,
        #[schemars(extend("x-og-unit" = "m"))]
        center: [f32; 3],
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        radius: f32,
        /// Raise/lower use metres; other brush modes clamp strength to one.
        #[schemars(range(min = 0))]
        strength: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        falloff: Option<f32>,
        #[schemars(range(max = 15))]
        layer: Option<u32>,
    },
    Stamp {
        geometry: Geometry,
        /// Existing Transform reflection supplies placement, never a copied field table.
        transform: Option<Reflected<Transform>>,
        mode: Option<StampMode>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        depth: Option<f32>,
    },
}
words!(BrushMode {
    Raise,
    Lower,
    Flatten,
    Smooth,
    Paint,
    Hole,
    Fill
});
words!(StampMode { Carve, Hole });
