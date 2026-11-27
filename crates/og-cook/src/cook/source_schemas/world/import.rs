//! Explicit external-file imports, not sidecars or directory match rules.
//! Source bytes/text are captured by the author input boundary at kind cutover.
//! Video and other domain-specific imports already have their own file schemas.
use super::{
    super::ContentRef,
    model::{Conform, process::Process},
};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

macro_rules! input {
    ($(#[$doc:meta])* $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$attr])* pub $field: $ty,)* }
    };
}
macro_rules! vocabulary {
    ($name:ident { $($variant:ident => $value:literal),* $(,)? }) => {
        #[derive(Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        pub enum $name { $(#[serde(rename = $value)] $variant,)* }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self { $(Self::$variant => $value,)* })
            }
        }
    };
}

pub mod animation;
pub mod rive;

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ImportInput {
    Model(Box<Model>),
    Texture(Texture),
    Sound(Sound),
    Font(Font),
    Tilemap(Tilemap),
    Rive(rive::Rive),
}

input!(Model {
    /// GLB/glTF/OBJ/FBX/VOX input. External buffers and images are discovered by
    /// the importer and must enter the closed input/dependency graph as well.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: String,
    #[serde(flatten)]
    options: super::model::ModelOptions,
    process: Option<Process>,
    fracture: Option<super::model::process::Fracture>,
    /// Source forward axis rotated to +Z; must be perpendicular to up.
    front: Option<Axis>,
    up: Option<Axis>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m/voxel"))]
    vox_scale: Option<f32>,
    vox_origin: Option<VoxOrigin>,
    #[schemars(range(min = 1, max = 120), extend("x-og-unit" = "Hz"))]
    vox_fps: Option<f32>,
    #[serde(default)]
    vox_clips: Vec<VoxClip>,
    conform: Option<Conform>,
});

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Collider {
    Enabled(bool),
    Method(ColliderMethod),
}
vocabulary!(ColliderMethod { Convex => "convex", Vhacd => "vhacd" });
vocabulary!(Axis { X => "+x", NegX => "-x", Y => "+y", NegY => "-y", Z => "+z", NegZ => "-z" });
vocabulary!(VoxOrigin { Center => "center", Bottom => "bottom" });
input!(VoxClip {
    #[schemars(length(min = 1))]
    name: String,
    /// Inclusive frame indices; first <= last and source bounds are domain checks.
    first: u32,
    last: u32,
});

input!(Texture {
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: String,
    /// Distinct from the root import kind discriminant.
    usage: Option<TextureUsage>,
    /// Pixel textures have no mip chain and cannot stream (domain validation).
    stream: Option<bool>,
    normal: Option<Normal>,
    sway: Option<Sway>,
    conform: Option<TextureConform>,
});
vocabulary!(TextureUsage { Color => "color", Normal => "normal", Linear => "linear", Single => "single", Pixel => "pixel" });

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum TextureConform {
    Off,
    Sprite {
        #[schemars(range(min = 1, max = 4096), extend("x-og-unit" = "px"))]
        size: Option<u32>,
    },
    Ui {
        #[schemars(range(min = 1, max = 4096), extend("x-og-unit" = "px"))]
        size: Option<u32>,
    },
    /// Use the role reported by consumers, with an explicit logical size.
    Auto {
        #[schemars(range(min = 1, max = 4096), extend("x-og-unit" = "px"))]
        size: Option<u32>,
    },
}

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Normal {
    Auto {},
    Image {
        #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
        source: String,
    },
}
input!(Sway {
    kind: SwayKind,
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    parts: Option<String>,
});
vocabulary!(SwayKind { Clumps => "clumps", Tiers => "tiers" });

input!(Sound {
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: String,
    /// QOA only. Absent selects automatic streaming above 1 MiB; Opus streams
    /// its own codec blocks rather than QOA chunks.
    stream: Option<bool>,
    #[schemars(range(min = 1, max = 60), extend("x-og-unit" = "s"))]
    chunk: Option<f32>,
    codec: Option<Codec>,
    /// Rounded to whole kb/s; absent selects the mono/stereo codec default.
    #[schemars(range(min = 16, max = 256), extend("x-og-unit" = "kb/s"))]
    bitrate: Option<f32>,
});
vocabulary!(Codec { Qoa => "qoa", Opus => "opus" });
input!(Font {
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: String,
    /// Omitted: outline font. Pixel grid changes only this font's raster mode.
    #[schemars(range(min = 4, max = 64), extend("x-og-unit" = "px/em"))]
    pixel: Option<u32>,
});
input!(Tilemap {
    /// Explicit Tiled/LDtk exchange text; never an engine author DSL.
    #[schemars(length(min = 1), extend("x-og-input" = "text"))]
    source: String,
    /// World units per tile width.
    unit: Option<f32>,
    level: Option<String>,
    #[serde(default)]
    solid: Vec<i64>,
    #[schemars(extend("x-og-ref-kinds" = ["tileset"]))]
    tileset: Option<ContentRef>,
    /// External TSX/TSJ or LDtk image paths as written in the map to tileset refs.
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["tileset"]))]
    tilesets: BTreeMap<String, ContentRef>,
    /// Embedded Tiled/LDtk image paths as written in the map to texture refs.
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    textures: BTreeMap<String, ContentRef>,
});

impl ImportInput {
    pub fn schema() -> &'static InputSchema<Self> {
        static SCHEMA: OnceLock<InputSchema<ImportInput>> = OnceLock::new();
        SCHEMA.get_or_init(|| InputSchema::new().expect("import input schema"))
    }
}
