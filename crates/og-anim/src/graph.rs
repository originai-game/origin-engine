//! The animation graph asset: an `.ogt` document of type `animgraph`
//! (grammar in the crate README). Motions (clips and 1D/2D blends) form a
//! tree; layers run state machines over them; transitions fire on
//! conditions over the graph's parameters and cross-fade.

use crate::asset::{NONE, SkeletonAsset};
use crate::schema::GRAPH_PARAMS;
use og_core::glam::Vec2;
use og_core::ogt::{Block, Document, Entry, Value};
use og_core::{Error, ErrorKind, Result};

/// One motion of the tree.
#[derive(Clone, Debug, PartialEq)]
pub enum Motion {
    /// A clip asset. `speed` scales its duration; it is sampled at the
    /// state's phase (so blended clips stay in step).
    Clip {
        clip: u64,
        speed: f32,
        looping: bool,
    },
    /// Children at thresholds of one parameter, sorted by threshold; the two
    /// around the parameter's value blend linearly (clamped at the ends).
    Blend1D {
        param: usize,
        children: Vec<(f32, usize)>,
    },
    /// Children at points of two parameters, weighted by gradient-band
    /// interpolation (freeform Cartesian).
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

/// How a layer combines with the layers below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerMode {
    /// Replaces the pose below, by the layer's weight.
    Override,
    /// Adds its motions' difference from their first frame, by the weight.
    Additive,
}

/// A comparison of a parameter with a number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// What may interrupt a transition while it cross-fades.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interrupt {
    /// Any transition that could fire from the new state, `any` ones
    /// included (the default).
    #[default]
    Any,
    /// Nothing: the fade completes first.
    None,
    /// Only transitions out of the state being left (to a third state).
    Source,
    /// Only transitions out of the new state itself, not `any` ones.
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    /// The source state, or `None` for `any` (every state but the target).
    pub from: Option<usize>,
    pub to: usize,
    /// All must hold.
    pub conditions: Vec<Condition>,
    /// When set, the source state's phase (cycles since it was entered) must
    /// have reached this.
    pub exit: Option<f32>,
    /// Cross-fade seconds.
    pub fade: f32,
    /// What may interrupt it during its fade.
    pub interrupt: Interrupt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub name: String,
    /// A [`MotionNode`] index.
    pub motion: usize,
    /// A playback-rate multiplier.
    pub speed: f32,
    /// The sync group (an index into [`GraphAsset::sync_groups`]): every
    /// layer's current state in one group plays at the phase of the first
    /// layer (file order) whose current state is in it, and a transition
    /// between two states of one group keeps the phase's fraction.
    pub sync: Option<usize>,
}

/// One line of a layer's mask: the node named (by name, resolved against
/// the skeleton the graph plays on) and every node under it take `weight`,
/// down to the next node another line names.
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
    /// When set, the weight is this parameter (clamped to 0..=1).
    pub weight_param: Option<usize>,
    pub states: Vec<State>,
    pub entry: usize,
    /// In file order; the first whose conditions hold fires.
    pub transitions: Vec<Transition>,
    /// Per-joint weights (`mask "<node>" [weight]`); empty: every node 1.
    pub mask: Vec<MaskEntry>,
}

impl Layer {
    pub fn weight(&self, params: &[f32; GRAPH_PARAMS]) -> f32 {
        match self.weight_param {
            Some(p) => params[p].clamp(0.0, 1.0),
            None => self.weight,
        }
    }

    /// The mask's weight of every node of `skeleton` (`None` without a
    /// mask): the weight of the nearest of the node and its ancestors a
    /// `mask` line names, 0 for nodes under none. A name the skeleton lacks
    /// masks nothing.
    pub fn node_weights(&self, skeleton: &SkeletonAsset) -> Option<Vec<f32>> {
        if self.mask.is_empty() {
            return None;
        }
        let mut named: Vec<Option<f32>> = vec![None; skeleton.nodes.len()];
        for m in &self.mask {
            if let Some(n) = skeleton.node(&m.node) {
                named[n as usize] = Some(m.weight);
            }
        }
        // Nodes are in topological order: a parent's weight is known first.
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

/// A parsed `animgraph`.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphAsset {
    /// Parameter names; parameter `i` is `og.AnimationGraph.p<i>`.
    pub params: Vec<String>,
    pub nodes: Vec<MotionNode>,
    /// At least one; layer 0 is the base and the only one root motion and
    /// `og.AnimationGraph.state` come from.
    pub layers: Vec<Layer>,
    /// The sync groups' names, in first-use order ([`State::sync`]).
    pub sync_groups: Vec<String>,
}

fn err(e: &Entry, reason: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {}: {reason}", e.line))
}

fn number(e: &Entry, i: usize) -> Result<f32> {
    e.f32(i).map_err(|x| err(e, x.reason()))
}

fn word(e: &Entry, i: usize) -> Result<&str> {
    e.word(i).map_err(|x| err(e, x.reason()))
}

fn arity(e: &Entry, n: usize) -> Result<()> {
    if e.values.len() != n {
        return Err(err(
            e,
            format!("'{}' takes {n} value(s), got {}", e.key, e.values.len()),
        ));
    }
    Ok(())
}

impl GraphAsset {
    /// Parses `animgraph` text. `resolve` turns a clip reference (the string
    /// after a `clip` node's name) into a registered clip handle.
    pub fn parse(text: &str, resolve: &dyn Fn(&str) -> Option<u64>) -> Result<GraphAsset> {
        let doc = Document::parse(text)?;
        GraphAsset::from_document(&doc, resolve)
    }

    pub fn from_document(
        doc: &Document,
        resolve: &dyn Fn(&str) -> Option<u64>,
    ) -> Result<GraphAsset> {
        if doc.ty != "animgraph" || doc.version != 1 {
            return Err(Error::new(
                ErrorKind::Parse,
                format!(
                    "an animation graph is `type animgraph` `version 1`, not `type {}` `version {}`",
                    doc.ty, doc.version
                ),
            ));
        }
        let body = &doc.body;
        let mut params: Vec<String> = Vec::new();
        for e in body.get_all("param") {
            arity(e, 1)?;
            let name = word(e, 0)?.to_string();
            if params.contains(&name) {
                return Err(err(e, format!("parameter '{name}' is declared twice")));
            }
            params.push(name);
        }
        if params.len() > GRAPH_PARAMS {
            return Err(Error::new(
                ErrorKind::Parse,
                format!(
                    "{} parameters; og.AnimationGraph has {GRAPH_PARAMS} (p0…p7)",
                    params.len()
                ),
            ));
        }
        let param = |e: &Entry, name: &str| -> Result<usize> {
            params.iter().position(|p| p == name).ok_or_else(|| {
                err(
                    e,
                    format!("unknown parameter '{name}' (declare it with `param {name}`)"),
                )
            })
        };

        // Node names first, so children may refer forward.
        let node_entries: Vec<&Entry> = body
            .entries
            .iter()
            .filter(|e| matches!(e.key.as_str(), "clip" | "blend1d" | "blend2d"))
            .collect();
        let mut names: Vec<String> = Vec::new();
        for e in &node_entries {
            let name = word(e, 0)?.to_string();
            if names.contains(&name) {
                return Err(err(e, format!("motion '{name}' is defined twice")));
            }
            names.push(name);
        }
        let node_of = |e: &Entry, name: &str| -> Result<usize> {
            names
                .iter()
                .position(|n| n == name)
                .ok_or_else(|| err(e, format!("unknown motion '{name}'")))
        };
        let mut nodes = Vec::new();
        for e in &node_entries {
            let name = word(e, 0)?.to_string();
            let empty = Block::new();
            let block = e.block.as_ref().unwrap_or(&empty);
            let motion = match e.key.as_str() {
                "clip" => {
                    arity(e, 2)?;
                    let reference = e.string(1).map_err(|x| err(e, x.reason()))?;
                    let clip = resolve(reference)
                        .ok_or_else(|| err(e, format!("clip '{reference}' is not loaded")))?;
                    let mut speed = 1.0;
                    let mut looping = true;
                    for b in &block.entries {
                        match b.key.as_str() {
                            "speed" => {
                                arity(b, 1)?;
                                speed = number(b, 0)?;
                                if !(speed > 0.0 && speed.is_finite()) {
                                    return Err(err(b, "a clip's speed must be positive"));
                                }
                            }
                            "loop" => {
                                arity(b, 1)?;
                                looping = b.values[0]
                                    .as_bool()
                                    .ok_or_else(|| err(b, "loop takes true or false"))?;
                            }
                            k => {
                                return Err(err(
                                    b,
                                    format!("unknown clip setting '{k}' (speed, loop)"),
                                ));
                            }
                        }
                    }
                    Motion::Clip {
                        clip,
                        speed,
                        looping,
                    }
                }
                "blend1d" => {
                    arity(e, 2)?;
                    let p = param(e, word(e, 1)?)?;
                    let mut children = Vec::new();
                    for b in &block.entries {
                        if b.key != "child" {
                            return Err(err(
                                b,
                                format!(
                                    "a blend1d holds `child <motion> <threshold>` lines, not '{}'",
                                    b.key
                                ),
                            ));
                        }
                        arity(b, 2)?;
                        children.push((number(b, 1)?, node_of(b, word(b, 0)?)?));
                    }
                    if children.is_empty() {
                        return Err(err(e, "a blend1d needs at least one child"));
                    }
                    children.sort_by(|a, b| a.0.total_cmp(&b.0));
                    Motion::Blend1D { param: p, children }
                }
                _ => {
                    arity(e, 3)?;
                    let x = param(e, word(e, 1)?)?;
                    let y = param(e, word(e, 2)?)?;
                    let mut children = Vec::new();
                    for b in &block.entries {
                        if b.key != "child" {
                            return Err(err(
                                b,
                                format!(
                                    "a blend2d holds `child <motion> <x> <y>` lines, not '{}'",
                                    b.key
                                ),
                            ));
                        }
                        arity(b, 3)?;
                        children.push((
                            Vec2::new(number(b, 1)?, number(b, 2)?),
                            node_of(b, word(b, 0)?)?,
                        ));
                    }
                    if children.is_empty() {
                        return Err(err(e, "a blend2d needs at least one child"));
                    }
                    Motion::Blend2D { x, y, children }
                }
            };
            nodes.push(MotionNode { name, motion });
        }
        // The tree must be acyclic.
        fn visit(
            nodes: &[MotionNode],
            i: usize,
            on_path: &mut Vec<bool>,
        ) -> std::result::Result<(), String> {
            if on_path[i] {
                return Err(nodes[i].name.clone());
            }
            on_path[i] = true;
            let children: Vec<usize> = match &nodes[i].motion {
                Motion::Clip { .. } => Vec::new(),
                Motion::Blend1D { children, .. } => children.iter().map(|c| c.1).collect(),
                Motion::Blend2D { children, .. } => children.iter().map(|c| c.1).collect(),
            };
            for c in children {
                visit(nodes, c, on_path)?;
            }
            on_path[i] = false;
            Ok(())
        }
        for i in 0..nodes.len() {
            visit(&nodes, i, &mut vec![false; nodes.len()]).map_err(|n| {
                Error::new(ErrorKind::Parse, format!("motion '{n}' contains itself"))
            })?;
        }

        let mut layers = Vec::new();
        let mut sync_groups: Vec<String> = Vec::new();
        for e in body.get_all("layer") {
            arity(e, 1)?;
            let name = word(e, 0)?.to_string();
            let empty = Block::new();
            let block = e.block.as_ref().unwrap_or(&empty);
            let mut layer = Layer {
                name,
                mode: LayerMode::Override,
                weight: 1.0,
                weight_param: None,
                states: Vec::new(),
                entry: 0,
                transitions: Vec::new(),
                mask: Vec::new(),
            };
            for b in block.get_all("state") {
                arity(b, 2)?;
                let sname = word(b, 0)?.to_string();
                if layer.states.iter().any(|s| s.name == sname) {
                    return Err(err(b, format!("state '{sname}' is defined twice")));
                }
                let mut speed = 1.0;
                let mut sync = None;
                if let Some(sb) = &b.block {
                    for s in &sb.entries {
                        match s.key.as_str() {
                            "speed" => {
                                arity(s, 1)?;
                                speed = number(s, 0)?;
                                if !(speed >= 0.0 && speed.is_finite()) {
                                    return Err(err(s, "a state's speed must be 0 or more"));
                                }
                            }
                            "sync" => {
                                arity(s, 1)?;
                                let g = word(s, 0)?;
                                sync = Some(match sync_groups.iter().position(|x| x == g) {
                                    Some(i) => i,
                                    None => {
                                        sync_groups.push(g.to_string());
                                        sync_groups.len() - 1
                                    }
                                });
                            }
                            k => {
                                return Err(err(
                                    s,
                                    format!("unknown state setting '{k}' (speed, sync)"),
                                ));
                            }
                        }
                    }
                }
                layer.states.push(State {
                    name: sname,
                    motion: node_of(b, word(b, 1)?)?,
                    speed,
                    sync,
                });
            }
            if layer.states.is_empty() {
                return Err(err(e, format!("layer '{}' has no states", layer.name)));
            }
            let state_of = |b: &Entry, n: &str| -> Result<usize> {
                layer
                    .states
                    .iter()
                    .position(|s| s.name == n)
                    .ok_or_else(|| err(b, format!("unknown state '{n}'")))
            };
            let mut entry = None;
            let mut transitions = Vec::new();
            let mut weight = None;
            let mut weight_param = None;
            let mut mode = LayerMode::Override;
            let mut mask: Vec<MaskEntry> = Vec::new();
            for b in &block.entries {
                match b.key.as_str() {
                    "state" => {}
                    "mask" => {
                        if b.values.is_empty() || b.values.len() > 2 {
                            return Err(err(
                                b,
                                "`mask \"<node>\" [weight]` takes a node name and a weight 0..1",
                            ));
                        }
                        let node = b.string(0).map_err(|x| err(b, x.reason()))?.to_string();
                        let weight = if b.values.len() == 2 {
                            number(b, 1)?
                        } else {
                            1.0
                        };
                        if !(0.0..=1.0).contains(&weight) {
                            return Err(err(b, "a mask weight is 0 to 1"));
                        }
                        if mask.iter().any(|m| m.node == node) {
                            return Err(err(b, format!("node '{node}' is masked twice")));
                        }
                        mask.push(MaskEntry { node, weight });
                    }
                    "entry" => {
                        arity(b, 1)?;
                        entry = Some(state_of(b, word(b, 0)?)?);
                    }
                    "mode" => {
                        arity(b, 1)?;
                        mode = match word(b, 0)? {
                            "override" => LayerMode::Override,
                            "additive" => LayerMode::Additive,
                            w => {
                                return Err(err(
                                    b,
                                    format!("layer mode '{w}' (override or additive)"),
                                ));
                            }
                        };
                    }
                    "weight" => {
                        arity(b, 1)?;
                        match &b.values[0] {
                            Value::Word(w) => weight_param = Some(param(b, w)?),
                            _ => weight = Some(number(b, 0)?.clamp(0.0, 1.0)),
                        }
                    }
                    "transition" => {
                        arity(b, 2)?;
                        let from = match word(b, 0)? {
                            "any" => None,
                            s => Some(state_of(b, s)?),
                        };
                        let to = state_of(b, word(b, 1)?)?;
                        let mut t = Transition {
                            from,
                            to,
                            conditions: Vec::new(),
                            exit: None,
                            fade: 0.2,
                            interrupt: Interrupt::Any,
                        };
                        if let Some(tb) = &b.block {
                            for c in &tb.entries {
                                match c.key.as_str() {
                                    "when" => {
                                        arity(c, 3)?;
                                        let op = match word(c, 1)? {
                                            "gt" => Op::Gt,
                                            "ge" => Op::Ge,
                                            "lt" => Op::Lt,
                                            "le" => Op::Le,
                                            "eq" => Op::Eq,
                                            "ne" => Op::Ne,
                                            o => {
                                                return Err(err(
                                                    c,
                                                    format!(
                                                        "comparison '{o}' (gt, ge, lt, le, eq, ne)"
                                                    ),
                                                ));
                                            }
                                        };
                                        t.conditions.push(Condition {
                                            param: param(c, word(c, 0)?)?,
                                            op,
                                            value: number(c, 2)?,
                                        });
                                    }
                                    "exit" => {
                                        arity(c, 1)?;
                                        t.exit = Some(number(c, 0)?);
                                    }
                                    "fade" => {
                                        arity(c, 1)?;
                                        t.fade = number(c, 0)?.max(0.0);
                                    }
                                    "interrupt" => {
                                        arity(c, 1)?;
                                        t.interrupt = match word(c, 0)? {
                                            "any" => Interrupt::Any,
                                            "none" => Interrupt::None,
                                            "source" => Interrupt::Source,
                                            "next" => Interrupt::Next,
                                            w => {
                                                return Err(err(
                                                    c,
                                                    format!(
                                                        "interrupt '{w}' (any, none, source, next)"
                                                    ),
                                                ));
                                            }
                                        };
                                    }
                                    k => {
                                        return Err(err(
                                            c,
                                            format!(
                                                "unknown transition setting '{k}' (when, exit, fade, interrupt)"
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                        if t.conditions.is_empty() && t.exit.is_none() {
                            return Err(err(b, "a transition needs a `when` or an `exit`"));
                        }
                        transitions.push(t);
                    }
                    k => {
                        return Err(err(
                            b,
                            format!(
                                "unknown layer setting '{k}' (state, entry, mode, weight, mask, transition)"
                            ),
                        ));
                    }
                }
            }
            layer.entry = entry.unwrap_or(0);
            layer.transitions = transitions;
            layer.mode = mode;
            layer.weight = weight.unwrap_or(1.0);
            layer.weight_param = weight_param;
            layer.mask = mask;
            layers.push(layer);
        }
        if layers.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "an animation graph needs at least one `layer`",
            ));
        }
        for e in &body.entries {
            if !matches!(
                e.key.as_str(),
                "param" | "clip" | "blend1d" | "blend2d" | "layer"
            ) {
                return Err(err(
                    e,
                    format!(
                        "unknown graph entry '{}' (param, clip, blend1d, blend2d, layer)",
                        e.key
                    ),
                ));
            }
        }
        Ok(GraphAsset {
            params,
            nodes,
            layers,
            sync_groups,
        })
    }

    /// The slot of a parameter by name.
    pub fn param(&self, name: &str) -> Option<usize> {
        self.params.iter().position(|p| p == name)
    }

    /// The clip handles the graph plays.
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

    /// The clip leaves under `node` with their weights (summing to `weight`),
    /// in tree order.
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
                    // Outside every band: the nearest child.
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
