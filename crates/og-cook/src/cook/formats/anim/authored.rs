//! Explicit model-local keys enter the same sampler as glTF channels.
use super::*;
use og_core::ogt::{Block, Entry, line::Line};

pub const CLIP_LINES: &[Line] = &[Line::new(
    "channel",
    "\"<node>\" translation | rotation | scale | weights | visibility { ... }",
    "required",
    "Repeated channels; node names must resolve uniquely. Metres, xyzw unit quaternions, scale factors, morph weights or 0/1 visibility.",
    "",
)];
pub const CHANNEL_LINES: &[Line] = &[
    Line::new(
        "interpolation",
        "step | linear | cubicSpline",
        "linear",
        "glTF interpolation; visibility requires step. Cubic keys contain in-tangent, value, out-tangent; tangents are per second.",
        "",
    ),
    Line::new(
        "key",
        "<seconds >= 0> <values...>",
        "required",
        "Strictly increasing times. Translation/scale have 3 lanes, rotation 4, weights one per morph target, visibility 1; cubic keys have three times as many values.",
        "",
    ),
];

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredClip {
    pub(super) name: String,
    channels: Vec<(String, SourceChannel)>,
}

fn fields(b: &Block, lines: &[Line]) -> Result<(), String> {
    for e in &b.entries {
        if !lines.iter().any(|f| f.key == e.key) {
            return Err(format!(
                "line {}: unknown animation field '{}'",
                e.line, e.key
            ));
        }
    }
    Ok(())
}

impl AuthoredClip {
    pub(super) fn from_input(
        d: &crate::cook::source_schemas::world::import::animation::Clip,
    ) -> Result<Self, String> {
        use crate::cook::source_schemas::world::import::animation::{ChannelPath, Interpolation};
        let fail = |s: &str| format!("clip '{}': {s}", d.name);
        let mut channels = Vec::new();
        for c in &d.channels {
            let path = match c.path {
                ChannelPath::Translation => path::TRANSLATION,
                ChannelPath::Rotation => path::ROTATION,
                ChannelPath::Scale => path::SCALE,
                ChannelPath::Weights => path::WEIGHTS,
                ChannelPath::Visibility => path::VISIBILITY,
            };
            if channels
                .iter()
                .any(|(n, channel): &(String, SourceChannel)| n == &c.node && channel.path == path)
            {
                return Err(fail("one channel per node/path"));
            }
            let interpolation = match c.interpolation {
                None | Some(Interpolation::Linear) => interpolation::LINEAR,
                Some(Interpolation::Step) => interpolation::STEP,
                Some(Interpolation::CubicSpline) => interpolation::CUBICSPLINE,
            };
            if path == path::VISIBILITY && interpolation != interpolation::STEP {
                return Err(fail("visibility requires step interpolation"));
            }
            let factor = if interpolation == interpolation::CUBICSPLINE {
                3
            } else {
                1
            };
            let lanes = match path {
                path::TRANSLATION | path::SCALE => 3,
                path::ROTATION => 4,
                path::VISIBILITY => 1,
                _ => c.keys.first().map_or(0, |k| k.values.len() / factor),
            };
            let mut times = Vec::new();
            let mut values = Vec::new();
            for key in &c.keys {
                if !key.time.is_finite()
                    || key.time < 0.0
                    || times.last().is_some_and(|t| key.time <= *t)
                {
                    return Err(fail(
                        "key times must be finite, nonnegative and strictly increasing",
                    ));
                }
                if lanes == 0
                    || key.values.len() != lanes * factor
                    || !key.values.iter().all(|v| v.is_finite())
                {
                    return Err(fail(
                        "key lane count must match its path/interpolation, with finite values",
                    ));
                }
                let start = if factor == 3 { lanes } else { 0 };
                let value = &key.values[start..start + lanes];
                if path == path::ROTATION && value.iter().map(|v| v * v).sum::<f32>() < 1e-12 {
                    return Err(fail("rotation keys need nonzero quaternions"));
                }
                if path == path::VISIBILITY && !matches!(value[0], 0.0 | 1.0) {
                    return Err(fail("visibility is 0 or 1"));
                }
                times.push(key.time);
                values.extend_from_slice(&key.values);
            }
            channels.push((
                c.node.clone(),
                SourceChannel {
                    node: 0,
                    path,
                    interpolation,
                    lanes,
                    times,
                    values,
                },
            ));
        }
        Ok(Self {
            name: d.name.clone(),
            channels,
        })
    }

    pub(super) fn parse(e: &Entry) -> Result<Self, String> {
        let fail = |s: &str| format!("line {}: animation clip: {s}", e.line);
        let name = e
            .string(0)
            .map_err(|_| fail("clip \"<name>\" { channel ... }"))?
            .to_string();
        if name.is_empty() || e.values.len() != 1 {
            return Err(fail("one nonempty clip name"));
        }
        let b = e.block.as_ref().ok_or_else(|| fail("needs channels"))?;
        fields(b, CLIP_LINES)?;
        let mut channels = Vec::new();
        for e in &b.entries {
            let fail = |s: &str| format!("line {}: clip '{name}': {s}", e.line);
            let node = e
                .string(0)
                .map_err(|_| fail("channel needs a quoted node name"))?
                .to_string();
            let path = match e.word(1).unwrap_or("") {
                "translation" => path::TRANSLATION,
                "rotation" => path::ROTATION,
                "scale" => path::SCALE,
                "weights" => path::WEIGHTS,
                "visibility" => path::VISIBILITY,
                _ => {
                    return Err(fail(
                        "channel path is translation, rotation, scale, weights or visibility",
                    ));
                }
            };
            if e.values.len() != 2
                || node.is_empty()
                || channels
                    .iter()
                    .any(|(n, c): &(String, SourceChannel)| *n == node && c.path == path)
            {
                return Err(fail(
                    "one channel per node/path, with exactly node and path arguments",
                ));
            }
            let b = e.block.as_ref().ok_or_else(|| fail("channel needs keys"))?;
            fields(b, CHANNEL_LINES)?;
            let interpolation = match b.get("interpolation").map(|e| e.word(0).unwrap_or("")) {
                None | Some("linear") => interpolation::LINEAR,
                Some("step") => interpolation::STEP,
                Some("cubicSpline") => interpolation::CUBICSPLINE,
                _ => return Err(fail("interpolation is step, linear or cubicSpline")),
            };
            if path == path::VISIBILITY && interpolation != interpolation::STEP {
                return Err(fail("visibility requires interpolation step"));
            }
            let factor = if interpolation == interpolation::CUBICSPLINE {
                3
            } else {
                1
            };
            let mut times = Vec::new();
            let mut values = Vec::new();
            let mut lanes = 0;
            for key in b.get_all("key") {
                let row: Vec<f32> = key
                    .values
                    .iter()
                    .map(|v| {
                        v.as_number()
                            .map(|v| v.as_f32())
                            .filter(|v| v.is_finite())
                            .ok_or_else(|| fail("key values must be finite numbers"))
                    })
                    .collect::<Result<_, _>>()?;
                if row.len() < 2 || row[0] < 0.0 || times.last().is_some_and(|t| row[0] <= *t) {
                    return Err(fail(
                        "key times must be nonnegative and strictly increasing",
                    ));
                }
                let n = row.len() - 1;
                let expected = match path {
                    path::TRANSLATION | path::SCALE => 3,
                    path::ROTATION => 4,
                    path::VISIBILITY => 1,
                    _ => {
                        if lanes == 0 {
                            n / factor
                        } else {
                            lanes
                        }
                    }
                };
                if expected == 0 || n != expected * factor {
                    return Err(fail("key lane count does not match its path/interpolation"));
                }
                lanes = expected;
                let value = &row[1 + if factor == 3 { lanes } else { 0 }..][..lanes];
                if path == path::ROTATION && value.iter().map(|v| v * v).sum::<f32>() < 1e-12 {
                    return Err(fail("rotation keys need nonzero quaternions"));
                }
                if path == path::VISIBILITY && !matches!(value[0], 0.0 | 1.0) {
                    return Err(fail("visibility is 0 or 1"));
                }
                times.push(row[0]);
                values.extend_from_slice(&row[1..]);
            }
            if times.is_empty() {
                return Err(fail("channel needs at least one key"));
            }
            channels.push((
                node,
                SourceChannel {
                    node: 0,
                    path,
                    interpolation,
                    lanes,
                    times,
                    values,
                },
            ));
        }
        if channels.is_empty() {
            return Err(fail("clip needs at least one channel"));
        }
        Ok(Self { name, channels })
    }

    pub(super) fn bind(
        &self,
        src: &GltfSource,
        indices: &[u32],
    ) -> Result<Vec<SourceChannel>, String> {
        self.channels
            .iter()
            .map(|(name, source)| {
                let nodes: Vec<_> = src.doc.nodes().filter(|n| n.name() == Some(name)).collect();
                if nodes.len() != 1 {
                    return Err(format!(
                        "clip '{}': node '{name}' must resolve uniquely ({} matches)",
                        self.name,
                        nodes.len()
                    ));
                }
                let node = &nodes[0];
                if source.path == path::WEIGHTS {
                    let count = node
                        .mesh()
                        .and_then(|m| m.primitives().next().map(|p| p.morph_targets().count()))
                        .unwrap_or(0);
                    if source.lanes != count {
                        return Err(format!(
                            "clip '{}': node '{name}' has {count} morph targets, not {}",
                            self.name, source.lanes
                        ));
                    }
                }
                let mut channel = source.clone();
                channel.node = indices[node.index()];
                Ok(channel)
            })
            .collect()
    }
}
