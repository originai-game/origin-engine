//! Explicit model-local keys enter the same sampler as glTF channels.
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredClip {
    pub(super) name: String,
    channels: Vec<(String, SourceChannel)>,
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
