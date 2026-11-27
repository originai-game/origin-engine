//! Named graph input shared by author declarations and runtime construction.
use super::*;
#[cfg(feature = "source")]
use og_core::reflect::source::schemars::JsonSchema;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct GraphDefinition {
    #[serde(default)]
    #[cfg_attr(feature = "source", schemars(length(max = 8)))]
    pub params: Vec<String>,
    pub motions: BTreeMap<String, MotionInput>,
    #[cfg_attr(feature = "source", schemars(length(min = 1)))]
    pub layers: Vec<LayerInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum MotionInput {
    Clip {
        clip: String,
        speed: Option<f32>,
        looping: Option<bool>,
        mirror: Option<bool>,
    },
    Blend1d {
        param: String,
        #[cfg_attr(feature = "source", schemars(length(min = 1)))]
        children: Vec<ThresholdInput>,
    },
    Blend2d {
        x: String,
        y: String,
        #[cfg_attr(feature = "source", schemars(length(min = 1)))]
        children: Vec<PointInput>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct ThresholdInput {
    pub motion: String,
    pub threshold: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct PointInput {
    pub motion: String,
    pub point: [f32; 2],
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LayerInput {
    pub name: String,
    pub mode: Option<LayerMode>,
    #[cfg_attr(feature = "source", schemars(range(min = 0, max = 1)))]
    pub weight: Option<f32>,
    pub weight_param: Option<String>,
    #[cfg_attr(feature = "source", schemars(length(min = 1)))]
    pub states: Vec<StateInput>,
    pub entry: String,
    #[serde(default)]
    pub transitions: Vec<TransitionInput>,
    #[serde(default)]
    pub mask: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct StateInput {
    pub name: String,
    pub motion: String,
    #[cfg_attr(feature = "source", schemars(range(min = 0)))]
    pub speed: Option<f32>,
    pub sync: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct TransitionInput {
    pub from: Option<String>,
    pub to: String,
    #[serde(default)]
    pub conditions: Vec<ConditionInput>,
    #[cfg_attr(feature = "source", schemars(range(min = 0)))]
    pub exit: Option<f32>,
    #[cfg_attr(feature = "source", schemars(range(min = 0)))]
    pub fade: Option<f32>,
    pub interrupt: Option<Interrupt>,
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(feature = "source", derive(JsonSchema))]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(deny_unknown_fields)]
pub struct ConditionInput {
    pub param: String,
    pub op: Op,
    pub value: f32,
}

impl GraphDefinition {
    /// Resolve names once; the resulting graph only uses native indices.
    pub fn build(&self, resolve: &dyn Fn(&str) -> Option<u64>) -> Result<GraphAsset> {
        fn index(names: &[&str], name: &str, kind: &str) -> Result<usize> {
            names
                .iter()
                .position(|n| *n == name)
                .ok_or_else(|| Error::new(ErrorKind::Parse, format!("unknown {kind} '{name}'")))
        }
        let params = self.params.iter().map(String::as_str).collect::<Vec<_>>();
        let names = self.motions.keys().map(String::as_str).collect::<Vec<_>>();
        let mut nodes = Vec::new();
        for (name, input) in &self.motions {
            let motion = match input {
                MotionInput::Clip {
                    clip,
                    speed,
                    looping,
                    mirror,
                } => Motion::Clip {
                    clip: resolve(clip).ok_or_else(|| {
                        Error::new(ErrorKind::Parse, format!("clip '{clip}' is not loaded"))
                    })?,
                    speed: speed.unwrap_or(1.0),
                    looping: looping.unwrap_or(true),
                    mirror: mirror.unwrap_or(false),
                },
                MotionInput::Blend1d { param, children } => {
                    let mut children = children
                        .iter()
                        .map(|c| Ok((c.threshold, index(&names, &c.motion, "motion")?)))
                        .collect::<Result<Vec<_>>>()?;
                    og_core::sort::sort_by(&mut children, |a, b| a.0.total_cmp(&b.0));
                    Motion::Blend1D {
                        param: index(&params, param, "parameter")?,
                        children,
                    }
                }
                MotionInput::Blend2d { x, y, children } => Motion::Blend2D {
                    x: index(&params, x, "parameter")?,
                    y: index(&params, y, "parameter")?,
                    children: children
                        .iter()
                        .map(|c| {
                            Ok((
                                Vec2::from_array(c.point),
                                index(&names, &c.motion, "motion")?,
                            ))
                        })
                        .collect::<Result<_>>()?,
                },
            };
            nodes.push(MotionNode {
                name: name.clone(),
                motion,
            });
        }
        let mut sync_groups = Vec::<String>::new();
        let mut layers = Vec::new();
        for l in &self.layers {
            if l.weight.is_some() && l.weight_param.is_some() {
                return Err(Error::new(
                    ErrorKind::Parse,
                    "layer weight and weightParam are exclusive",
                ));
            }
            let mut states = Vec::new();
            for s in &l.states {
                let sync = s
                    .sync
                    .as_ref()
                    .map(|g| match sync_groups.iter().position(|n| n == g) {
                        Some(i) => i,
                        None => {
                            sync_groups.push(g.clone());
                            sync_groups.len() - 1
                        }
                    });
                states.push(State {
                    name: s.name.clone(),
                    motion: index(&names, &s.motion, "motion")?,
                    speed: s.speed.unwrap_or(1.0),
                    sync,
                });
            }
            let state_names = l.states.iter().map(|s| s.name.as_str()).collect::<Vec<_>>();
            let transitions = l
                .transitions
                .iter()
                .map(|t| {
                    Ok(Transition {
                        from: t
                            .from
                            .as_ref()
                            .map(|s| index(&state_names, s, "state"))
                            .transpose()?,
                        to: index(&state_names, &t.to, "state")?,
                        conditions: t
                            .conditions
                            .iter()
                            .map(|c| {
                                Ok(Condition {
                                    param: index(&params, &c.param, "parameter")?,
                                    op: c.op,
                                    value: c.value,
                                })
                            })
                            .collect::<Result<_>>()?,
                        exit: t.exit,
                        fade: t.fade.unwrap_or(0.2),
                        interrupt: t.interrupt.unwrap_or_default(),
                    })
                })
                .collect::<Result<_>>()?;
            layers.push(Layer {
                name: l.name.clone(),
                mode: l.mode.unwrap_or(LayerMode::Override),
                weight: l.weight.unwrap_or(1.0),
                weight_param: l
                    .weight_param
                    .as_ref()
                    .map(|p| index(&params, p, "parameter"))
                    .transpose()?,
                states,
                entry: index(&state_names, &l.entry, "state")?,
                transitions,
                mask: l
                    .mask
                    .iter()
                    .map(|(node, weight)| MaskEntry {
                        node: node.clone(),
                        weight: *weight,
                    })
                    .collect(),
            });
        }
        let graph = GraphAsset {
            params: self.params.clone(),
            nodes,
            layers,
            sync_groups,
        };
        graph.validate()?;
        Ok(graph)
    }
}
