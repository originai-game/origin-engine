//! Input names follow the domain declarations, not resolved runtime handles.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Video {
    /// Canonical .mp4, .webm or .mov path under the owning module's assets/
    /// or sources/, without declaration-relative or sibling fallback.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    pub source: String,
    #[schemars(range(min = 1, max = 120), extend("x-og-unit" = "Hz"))]
    pub fps: Option<u32>,
    /// Seconds retained from the beginning; omitted: 8 for a clip, whole source for a stream.
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
    pub max_seconds: Option<f64>,
    /// Width, height in pixels.
    #[schemars(inner(range(min = 2, max = 4096)), extend("x-og-unit" = "px"))]
    pub size: Option<[u32; 2]>,
    /// Frame processing; omitted color, pixel keeps nearest/lossless encoding.
    pub kind: Option<FrameKind>,
    /// Keep the source audio when present; omitted true.
    pub audio: Option<bool>,
    /// Alternate audio paths under this module's assets/ or sources/.
    /// Locale tags are unique ignoring case; ASCII letters, digits, '-' and '_'.
    #[serde(default)]
    #[schemars(extend("x-og-input" = "bytes"))]
    pub audio_locale: BTreeMap<String, String>,
    #[schemars(range(min = -30, max = 0), extend("x-og-unit" = "LUFS"))]
    pub loudness: Option<f64>,
    pub ai_generated: Option<bool>,
    pub placeholder: Option<bool>,
    pub alpha: Option<bool>,
    pub stream: Option<bool>,
    #[schemars(range(min = 0.1, max = 10), extend("x-og-unit" = "s"))]
    pub keyframe: Option<f64>,
    #[schemars(range(min = 0.5, max = 60), extend("x-og-unit" = "s"))]
    pub chunk: Option<f64>,
    #[schemars(range(max = 63))]
    pub crf: Option<u8>,
    #[schemars(range(max = 63))]
    pub crf720: Option<u8>,
    /// Zero keeps the full resolution; otherwise 64..4096 pixels.
    #[schemars(extend("anyOf" = [{"const": 0}, {"type": "integer", "minimum": 64, "maximum": 4096}, {"type": "null"}]))]
    pub poster_size: Option<u32>,
    /// Browser codecs beside AV1; omitted HEVC when available, [] disables them.
    /// All rendition controls require stream: true.
    pub renditions: Option<Vec<Rendition>>,
    pub low_rendition: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum FrameKind {
    Color,
    Pixel,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Rendition {
    Hevc,
    H264,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionAction {
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    pub source: ContentRef,
    #[schemars(length(min = 1))]
    pub clip: String,
    #[schemars(extend("x-og-ref-kinds" = ["rig"]))]
    pub rig: Option<ContentRef>,
    #[serde(default)]
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub hold_start: f32,
    /// Must be greater than holdStart (domain validation).
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    pub hold_end: f32,
    pub hand: Option<Hand>,
    /// In the head's local frame.
    pub hand_target: Option<[f32; 3]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Hand {
    Left,
    Right,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rig {
    /// Naming scheme understood by the humanoid mapper.
    #[schemars(transform = rig_scheme)]
    pub scheme: Option<String>,
    /// Left and right sole heights.
    #[schemars(inner(range(min = -10, max = 10)), extend("x-og-unit" = "m"))]
    pub foot_height: Option<[f32; 2]>,
    /// Humanoid bone name to skeleton node name; null explicitly unmaps it.
    #[serde(default)]
    #[schemars(transform = rig_bones)]
    pub bones: BTreeMap<String, Option<String>>,
}

fn rig_scheme(schema: &mut og_core::reflect::source::schemars::Schema) {
    schema.insert(
        "enum".into(),
        serde_json::json!(
            og_anim::SCHEMES
                .iter()
                .copied()
                .chain(["auto"])
                .map(serde_json::Value::from)
                .chain([serde_json::Value::Null])
                .collect::<Vec<_>>()
        ),
    );
}

fn rig_bones(schema: &mut og_core::reflect::source::schemars::Schema) {
    let value = schema.get("additionalProperties").unwrap().clone();
    schema.insert(
        "properties".into(),
        og_anim::HUMAN_BONES
            .iter()
            .map(|name| ((*name).into(), value.clone()))
            .collect::<serde_json::Map<_, _>>()
            .into(),
    );
    schema.insert("additionalProperties".into(), false.into());
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Flipbook {
    #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
    pub texture: ContentRef,
    pub filter: Option<Filter>,
    #[serde(default)]
    pub array: bool,
    /// Explicit Aseprite or TexturePacker exchange text from input.text.
    pub atlas: Option<String>,
    /// Grid and explicit/imported frames are composed by the domain validator.
    pub grid: Option<Grid>,
    /// Bottom-left fractions; a pivot may lie outside the frame.
    pub pivot: Option<[f32; 2]>,
    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub spacing: u32,
    #[serde(default)]
    pub frames: Vec<Frame>,
    #[serde(default)]
    pub clips: Vec<Clip>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Grid {
    #[schemars(inner(range(min = 1)), extend("x-og-unit" = "px"))]
    pub size: [u32; 2],
    #[schemars(range(min = 1))]
    pub columns: Option<u32>,
    #[schemars(range(min = 1))]
    pub rows: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    Nearest,
    Linear,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub name: String,
    /// Top-left x, y, width, height. Extents checked against the texture.
    #[schemars(extend("x-og-unit" = "px"))]
    pub rect: [u32; 4],
    /// Bottom-left fractions; a pivot may lie outside the frame.
    pub pivot: Option<[f32; 2]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub name: String,
    /// Frame names, resolved only by the domain validator.
    #[schemars(length(min = 1))]
    pub frames: Vec<String>,
    #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "Hz"))]
    pub fps: Option<f32>,
    /// One strictly positive duration per frame; otherwise use fps.
    #[schemars(extend("items" = {"type": "number", "format": "float", "exclusiveMinimum": 0}, "x-og-unit" = "s"))]
    pub durations: Option<Vec<f32>>,
    pub mode: Option<FlipMode>,
    #[serde(default)]
    pub events: Vec<(u32, String)>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "camelCase")]
pub enum FlipMode {
    Loop,
    Once,
    PingPong,
}

schema!(Video, MotionAction, Rig, Flipbook);
