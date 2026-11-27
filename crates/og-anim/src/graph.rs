//! Native animation graphs: blend trees and layered state machines.
use crate::asset::{NONE, SkeletonAsset};
use crate::schema::GRAPH_PARAMS;
use og_core::glam::Vec2;
use og_core::{Error, ErrorKind, Result};

mod cooked;
mod input;
mod source;
mod validate;
pub use input::*;
pub use source::{ClipBinding, GraphSource};

#[derive(Clone, Debug, PartialEq)]
pub enum Motion {
    Clip {
        clip: u64,
        speed: f32,
        looping: bool,
        mirror: bool,
    },
    /// Children sorted by threshold; clamp at the ends.
    Blend1D {
        param: usize,
        children: Vec<(f32, usize)>,
    },
    /// Freeform Cartesian gradient-band interpolation.
    Blend2D {
        x: usize,
        y: usize,
        children: Vec<(Vec2, usize)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionNode {
    pub name: String,
    pub motion: Motion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[cfg_attr(
    feature = "source",
    derive(og_core::reflect::source::schemars::JsonSchema)
)]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "lowercase")]
pub enum LayerMode {
    Override,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[cfg_attr(
    feature = "source",
    derive(og_core::reflect::source::schemars::JsonSchema)
)]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Condition {
    pub param: usize,
    pub op: Op,
    pub value: f32,
}

impl Condition {
    pub fn holds(&self, params: &[f32; GRAPH_PARAMS]) -> bool {
        let v = params[self.param];
        match self.op {
            Op::Gt => v > self.value,
            Op::Ge => v >= self.value,
            Op::Lt => v < self.value,
            Op::Le => v <= self.value,
            Op::Eq => v == self.value,
            Op::Ne => v != self.value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[cfg_attr(
    feature = "source",
    derive(og_core::reflect::source::schemars::JsonSchema)
)]
#[cfg_attr(
    feature = "source",
    schemars(crate = "og_core::reflect::source::schemars")
)]
#[serde(rename_all = "lowercase")]
pub enum Interrupt {
    #[default]
    Any,
    None,
    Source,
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// None: any state except the target.
    pub from: Option<usize>,
    pub to: usize,
    pub conditions: Vec<Condition>,
    /// Minimum source phase, in cycles.
    pub exit: Option<f32>,
    pub fade: f32,
    pub interrupt: Interrupt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    pub motion: usize,
    pub speed: f32,
    /// Index into GraphAsset.sync_groups, in first-use order.
    pub sync: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaskEntry {
    pub node: String,
    pub weight: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    pub mode: LayerMode,
    pub weight: f32,
    pub weight_param: Option<usize>,
    pub states: Vec<State>,
    pub entry: usize,
    /// Priority order; first eligible transition fires.
    pub transitions: Vec<Transition>,
    pub mask: Vec<MaskEntry>,
}

impl Layer {
    pub fn weight(&self, params: &[f32; GRAPH_PARAMS]) -> f32 {
        match self.weight_param {
            Some(p) => params[p].clamp(0.0, 1.0),
            None => self.weight,
        }
    }

    /// Nearest named ancestor's weight; nodes outside every mask get zero.
    pub fn node_weights(&self, skeleton: &SkeletonAsset) -> Option<Vec<f32>> {
        if self.mask.is_empty() {
            return None;
        }
        let mut named = vec![None; skeleton.nodes.len()];
        for m in &self.mask {
            if let Some(n) = skeleton.node(&m.node) {
                named[n as usize] = Some(m.weight);
            }
        }
        let mut out = vec![0.0f32; skeleton.nodes.len()];
        for (i, n) in skeleton.nodes.iter().enumerate() {
            out[i] = match (named[i], n.parent) {
                (Some(w), _) => w,
                (None, NONE) => 0.0,
                (None, p) => out[p as usize],
            };
        }
        Some(out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphAsset {
    pub params: Vec<String>,
    pub nodes: Vec<MotionNode>,
    pub layers: Vec<Layer>,
    pub sync_groups: Vec<String>,
}

impl GraphAsset {
    pub fn param(&self, name: &str) -> Option<usize> {
        self.params.iter().position(|p| p == name)
    }

    pub fn clips(&self) -> Vec<u64> {
        let mut out: Vec<u64> = self
            .nodes
            .iter()
            .filter_map(|n| match n.motion {
                Motion::Clip { clip, .. } => Some(clip),
                _ => None,
            })
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Clip leaves and weights in tree order, summing to `weight`.
    pub fn leaves(
        &self,
        node: usize,
        params: &[f32; GRAPH_PARAMS],
        weight: f32,
        out: &mut Vec<(usize, f32)>,
    ) {
        if weight <= 0.0 {
            return;
        }
        match &self.nodes[node].motion {
            Motion::Clip { .. } => out.push((node, weight)),
            Motion::Blend1D { param, children } => {
                let v = params[*param];
                let n = children.len();
                if n == 1 || v <= children[0].0 {
                    self.leaves(children[0].1, params, weight, out);
                } else if v >= children[n - 1].0 {
                    self.leaves(children[n - 1].1, params, weight, out);
                } else {
                    let i = children
                        .windows(2)
                        .position(|w| v < w[1].0)
                        .unwrap_or(n - 2);
                    let (a, b) = (children[i], children[i + 1]);
                    let span = b.0 - a.0;
                    let t = if span > 0.0 { (v - a.0) / span } else { 1.0 };
                    self.leaves(a.1, params, weight * (1.0 - t), out);
                    self.leaves(b.1, params, weight * t, out);
                }
            }
            Motion::Blend2D { x, y, children } => {
                let p = Vec2::new(params[*x], params[*y]);
                let mut ws: Vec<f32> = Vec::with_capacity(children.len());
                for (i, (pi, _)) in children.iter().enumerate() {
                    let mut w = 1.0f32;
                    for (j, (pj, _)) in children.iter().enumerate() {
                        if i == j {
                            continue;
                        }
                        let d = *pj - *pi;
                        let len2 = d.length_squared();
                        if len2 <= 0.0 {
                            continue;
                        }
                        let h = 1.0 - (p - *pi).dot(d) / len2;
                        w = w.min(h.max(0.0));
                    }
                    ws.push(w);
                }
                let total: f32 = ws.iter().sum();
                if total <= 0.0 {
                    let near = children
                        .iter()
                        .enumerate()
                        .min_by(|a, b| {
                            (p - a.1.0)
                                .length_squared()
                                .total_cmp(&(p - b.1.0).length_squared())
                        })
                        .map_or(0, |c| c.0);
                    self.leaves(children[near].1, params, weight, out);
                    return;
                }
                for (i, (_, c)) in children.iter().enumerate() {
                    self.leaves(*c, params, weight * ws[i] / total, out);
                }
            }
        }
    }
}
