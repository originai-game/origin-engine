//! Timeline tracks and keys are discriminated data. Reflected field tracks
//! name the existing record schemas; no renderer field table is copied here.
use super::*;
use crate::cook::source_schemas::reflected::{RecordMarker, Reflected};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    pub name: Option<String>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
    pub duration: Option<f32>,
    #[serde(default)]
    pub paths: BTreeMap<String, Path>,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Path {
    #[schemars(extend("x-og-unit" = "m"))]
    pub points: Vec<[f32; 3]>,
    pub closed: Option<bool>,
    pub kind: Option<PathKind>,
    pub tension: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum PathKind {
    CatmullRom,
    Bezier,
    Linear,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Track {
    Camera {
        rail: Option<String>,
        follow: Option<String>,
        keys: Vec<Key<Camera>>,
        #[serde(rename = "blendIn")]
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        blend_in: Option<f32>,
        #[serde(rename = "blendOut")]
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        blend_out: Option<f32>,
    },
    Transform {
        entity: String,
        keys: Vec<Key<Transform>>,
    },
    Animation {
        entity: String,
        #[schemars(extend("x-og-ref-kinds" = ["model"]))]
        model: Option<ContentRef>,
        keys: Vec<Key<Animation>>,
    },
    Audio {
        keys: Vec<Key<Audio>>,
    },
    Music {
        keys: Vec<Key<Music>>,
    },
    Caption {
        #[serde(default)]
        keys: Vec<Key<Caption>>,
        #[serde(default)]
        imports: Vec<CaptionImport>,
        locales: Option<BTreeMap<String, CaptionCues>>,
    },
    Event {
        keys: Vec<Key<Event>>,
    },
    Transition {
        keys: Vec<Key<Transition>>,
    },
    Marker {
        keys: Vec<Key<Marker>>,
    },
    Property {
        entity: Option<String>,
        field: String,
        keys: Vec<Key<FieldValue>>,
    },
    Light {
        entity: String,
        keys: Vec<Key<Light>>,
    },
    Post {
        keys: Vec<Key<Reflected<PostEffects>>>,
    },
    Visibility {
        entity: String,
        keys: Vec<Key<bool>>,
    },
    Blendshape {
        entity: String,
        keys: Vec<Key<Reflected<Blendshapes>>>,
    },
    Material {
        entity: String,
        index: Option<u32>,
        keys: Vec<Key<Material>>,
    },
    Timeline {
        keys: Vec<Key<Nested>>,
    },
    Shot {
        keys: Vec<Key<Shot>>,
    },
    Vfx {
        keys: Vec<Key<Vfx>>,
    },
    Video {
        entity: Option<String>,
        layer: Option<VideoLayer>,
        keys: Vec<Key<Video>>,
    },
    Hotspot {
        keys: Vec<Key<Hotspot>>,
    },
    Scene {
        keys: Vec<Key<Scene>>,
    },
}

macro_rules! record {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Debug)]
        pub struct $name;
        impl RecordMarker for $name {
            const NAME: &'static str = concat!("og.", stringify!($name));
        }
    )+};
}
record!(
    DirectionalLight,
    PointLight,
    SpotLight,
    RectLight,
    Light2D,
    PostEffects
);

#[derive(Debug)]
pub struct Blendshapes;
impl RecordMarker for Blendshapes {
    const NAME: &'static str = "og.Blendshapes";
    // Playback drives weights and sets source to component itself.
    const EXCLUDED: &'static [&'static str] = &["source", "mesh"];
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Light {
    Directional(Reflected<DirectionalLight>),
    Point(Reflected<PointLight>),
    Spot(Reflected<SpotLight>),
    Rect(Reflected<RectLight>),
    Planar(Reflected<Light2D>),
}

#[derive(Debug)]
pub struct StandardMaterial;
impl RecordMarker for StandardMaterial {
    const NAME: &'static str = "og.StandardMaterial";
    // UV controls have their own typed fields on Material; never accept two
    // competing values for the same control in a single key.
    const EXCLUDED: &'static [&'static str] = &["mapOffset", "mapRepeat", "mapRotation"];

    fn layout() -> Option<og_core::Schema> {
        let mut layout = <og_render_data::StandardMaterial as og_core::Reflect>::schema();
        layout
            .fields
            .retain(|f| og_timeline::MATERIAL_FIELDS.contains(&f.name.as_str()));
        Some(layout)
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Material {
    /// Only reflected fields actually driven by timeline presentation.
    pub standard: Option<Reflected<StandardMaterial>>,
    pub map_offset: Option<[f32; 2]>,
    pub map_repeat: Option<[f32; 2]>,
    #[schemars(extend("x-og-unit" = "rad"))]
    pub map_rotation: Option<f32>,
    /// Explicit shader uniforms, not an escape hatch for misspelled standard
    /// fields. Matching the target shader's names/dimensions is a graph check.
    #[serde(default)]
    #[schemars(transform = shader_parameters)]
    pub params: BTreeMap<String, Uniform>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum Uniform {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
}

fn shader_parameters(schema: &mut og_core::reflect::source::schemars::Schema) {
    // Closed reserved properties also produce optional-never TS fields. The
    // remaining index signature keeps only the renderer's numeric dimensions.
    schema.insert(
        "properties".into(),
        og_timeline::MATERIAL_FIELDS
            .iter()
            .map(|name| ((*name).into(), serde_json::Value::Bool(false)))
            .collect::<serde_json::Map<_, _>>()
            .into(),
    );
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Key<T> {
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub time: f32,
    /// Shared easing syntax, including cubicBezier, steps and points.
    pub ease: Option<String>,
    pub value: T,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum FieldValue {
    Number(f32),
    Bool(bool),
    Word(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Camera {
    #[schemars(extend("x-og-unit" = "m"))]
    pub position: Option<[f32; 3]>,
    #[schemars(extend("x-og-unit" = "m"))]
    pub target: Option<[f32; 3]>,
    pub at: Option<f32>,
    #[schemars(range(min = 1, max = 179), extend("x-og-unit" = "deg"))]
    pub fov: Option<f32>,
    #[schemars(extend("x-og-unit" = "deg"))]
    pub roll: Option<f32>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
    pub focus: Option<f32>,
    pub focus_on: Option<String>,
    #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
    pub focus_range: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Transform {
    #[schemars(extend("x-og-unit" = "m"))]
    pub position: Option<[f32; 3]>,
    /// Euler XYZ degrees; exclusive with quaternion (domain check).
    #[schemars(extend("x-og-unit" = "deg"))]
    pub rotation: Option<[f32; 3]>,
    pub quaternion: Option<[f32; 4]>,
    pub scale: Option<[f32; 3]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Animation {
    pub clip: String,
    pub speed: Option<f32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub blend_in: Option<f32>,
    #[serde(rename = "loop")]
    pub looping: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Audio {
    #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
    pub sound: ContentRef,
    #[schemars(range(min = 0))]
    pub volume: Option<f32>,
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub playback_rate: Option<f32>,
    pub bus: Option<Bus>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Music {
    #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
    pub sound: ContentRef,
    #[schemars(range(min = 0))]
    pub volume: Option<f32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub fade_in: Option<f32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub fade_out: Option<f32>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub end: Option<f32>,
    #[serde(rename = "loop")]
    pub looping: Option<bool>,
    pub bus: Option<Bus>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Bus {
    Master,
    Music,
    Sfx,
    Ui,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct CaptionImport {
    /// Explicit SRT, VTT or JSON cue file; cook captures it as a dependency.
    #[schemars(length(min = 1), extend("x-og-input" = "text"))]
    pub source: String,
    #[schemars(extend("x-og-unit" = "s"))]
    pub offset: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct CaptionCues {
    #[serde(default)]
    pub keys: Vec<Key<Caption>>,
    #[serde(default)]
    pub imports: Vec<CaptionImport>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Caption {
    pub end: Option<f32>,
    pub text: String,
    pub speaker: Option<String>,
    pub position: Option<CaptionPosition>,
    #[serde(default)]
    pub spans: Vec<Span>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum CaptionPosition {
    Top,
    Bottom,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub name: String,
    pub position: u32,
    pub length: u32,
    pub color: Option<String>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub action: String,
    pub retroactive: Option<bool>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Marker {
    #[schemars(length(min = 1, max = 63))]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub kind: Option<TransitionKind>,
    pub mode: Option<TransitionMode>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub duration: Option<f32>,
    pub color: Option<[f32; 3]>,
    pub from: Option<Direction>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum TransitionKind {
    Cut,
    Fade,
    Wipe,
    Dissolve,
    Iris,
    Slide,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum TransitionMode {
    Through,
    In,
    Out,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Nested {
    #[schemars(extend("x-og-ref-kinds" = ["timeline"]))]
    pub play: ContentRef,
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub speed: Option<f32>,
    #[serde(rename = "loop")]
    pub looping: Option<bool>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Shot {
    pub camera: Option<String>,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub blend: Option<f32>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Vfx {
    #[schemars(extend("x-og-ref-kinds" = ["vfx"]))]
    pub effect: ContentRef,
    pub at: Option<String>,
    pub position: Option<[f32; 3]>,
    #[schemars(extend("exclusiveMinimum" = 0))]
    pub scale: Option<f32>,
    pub seed: Option<u32>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Video {
    #[schemars(extend("x-og-ref-kinds" = ["video"]))]
    pub video: ContentRef,
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub from: Option<f32>,
    #[schemars(range(min = 0, max = 4))]
    pub speed: Option<f32>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum VideoLayer {
    Over,
    Under,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Hotspot {
    #[schemars(length(min = 1, max = 63))]
    pub id: String,
    pub rect: Option<[f32; 4]>,
    pub shape: Option<HotspotShape>,
    pub kind: Option<HotspotKind>,
    pub label: Option<String>,
    pub off: Option<bool>,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum HotspotShape {
    Rect,
    Circle,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum HotspotKind {
    Choice,
    Inspect,
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// Null clears the scene at this key.
    #[schemars(extend("x-og-ref-kinds" = ["scene"]))]
    pub scene: Option<ContentRef>,
}

schema!(Timeline);
