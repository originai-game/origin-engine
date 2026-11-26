//! glTF animation import options. Lane count, strictly increasing keys, unique
//! node/path pairs, visibility interpolation and quaternion norms are validated
//! by the animation domain against the source skeleton and morph targets.
use super::*;

input!(Animation {
    #[schemars(range(min = 1, max = 120), extend("x-og-unit" = "Hz"))]
    rate: Option<f32>,
    #[serde(default)]
    events: Vec<Event>,
    #[schemars(extend("x-og-ref-kinds" = ["rig"]))]
    rig: Option<ContentRef>,
    #[serde(default)]
    #[schemars(extend("x-og-ref-kinds" = ["model"]))]
    libraries: Vec<ContentRef>,
    foot_ik: Option<bool>,
    #[serde(default)]
    mirrors: Vec<Mirror>,
    compress: Option<bool>,
    #[schemars(range(max = 0.1), extend("exclusiveMinimum" = 0))]
    tolerance: Option<f32>,
    #[serde(default)]
    clips: Vec<Clip>,
});
input!(Event {
    clip: String,
    marker: String,
    #[schemars(extend("x-og-unit" = "s"))]
    time: f32,
});
input!(Mirror { clip: String, name: Option<String> });
input!(Clip {
    #[schemars(length(min = 1))]
    name: String,
    #[schemars(length(min = 1))]
    channels: Vec<Channel>,
});
input!(Channel {
    #[schemars(length(min = 1))]
    node: String,
    path: ChannelPath,
    interpolation: Option<Interpolation>,
    #[schemars(length(min = 1))]
    keys: Vec<Key>,
});
vocabulary!(ChannelPath { Translation => "translation", Rotation => "rotation", Scale => "scale", Weights => "weights", Visibility => "visibility" });
vocabulary!(Interpolation { Step => "step", Linear => "linear", CubicSpline => "cubicSpline" });
input!(Key {
    #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
    time: f32,
    /// Metres, xyzw unit quaternion, scale factors, morph weights or 0/1.
    /// Cubic spline keys contain in-tangent, value, out-tangent (per second).
    #[schemars(length(min = 1))]
    values: Vec<f32>,
});
