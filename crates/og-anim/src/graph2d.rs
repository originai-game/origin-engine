//! Animation graphs on 2D targets: an `animgraph` whose `clip` lines name
//! clips of the sheet or rig it plays on, by name, so one graph serves
//! every sheet (or rig) with those clip names. og.flipbook and
//! og.skeleton2d run it on their entities that carry an og.AnimationGraph
//! of such a graph: the same state machines, parameters, transitions,
//! interrupts, sync groups and layers og.anim runs on skeletons, played
//! the way each target can play them.
//!
//! - A flipbook shows one clip: the heaviest leaf of the layers (an upper
//!   override layer at weight 1 hides the layers under it; at 0.5 or more
//!   it wins over them). Transitions cut, never fade (their `fade` is not
//!   used), and the cut waits for the frame on screen to end, so a frame
//!   is never shortened; a cut within one state (a blend's leaf) or
//!   between states of one sync group keeps the phase.
//! - A 2D skeleton blends two clips: the two heaviest leaves, their
//!   weights renormalised, written into Skeleton2D.clip and blendClip with
//!   their times and `blend`; transitions cross-fade over their `fade`.
//!
//! Additive layers and masks are skeleton-only: a 2D target ignores an
//! additive layer and plays a masked one whole. Machine state (each
//! layer's state, phase and fade) is world state, in the target system's
//! state block.

use crate::asset::NONE;
use crate::codec::{Reader, Writer};
use crate::graph::{GraphAsset, Interrupt, LayerMode, Motion};
use crate::machine::{LayerState, Phase};
use crate::schema::AnimationGraph;
use og_core::Result;
use std::cell::RefCell;

/// A graph for a 2D target: its asset (clip ids are `1 + ` an index into
/// `clips`) and the clip names its `clip` lines give.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteGraph {
    pub asset: GraphAsset,
    pub clips: Vec<String>,
}

impl SpriteGraph {
    /// Parses `animgraph` text whose `clip` lines name the target's clips.
    pub fn parse(text: &str) -> Result<SpriteGraph> {
        let names = RefCell::new(Vec::<String>::new());
        let asset = GraphAsset::parse(text, &|name| {
            let mut v = names.borrow_mut();
            let i = match v.iter().position(|n| n == name) {
                Some(i) => i,
                None => {
                    v.push(name.to_string());
                    v.len() - 1
                }
            };
            Some(i as u64 + 1)
        })?;
        Ok(SpriteGraph {
            asset,
            clips: names.into_inner(),
        })
    }

    /// The clip name of a clip id of the graph.
    pub fn name(&self, clip: u64) -> &str {
        &self.clips[(clip - 1) as usize]
    }
}

/// A clip leaf the graph plays this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Played {
    /// The graph's clip id ([`SpriteGraph::name`]).
    pub clip: u64,
    /// Its share of what the target shows: the weights of a tick sum to 1.
    pub weight: f32,
    pub layer: u32,
    pub state: u32,
    /// The state's phase before and after this tick, and its wraps.
    pub before: Phase,
    pub after: Phase,
    pub wraps: i32,
    pub looping: bool,
    /// The playback rate: the graph's speed by the state's and the clip's.
    pub rate: f32,
    /// The state's sync group.
    pub sync: Option<usize>,
}

/// One entity's graph: its layers' state machines.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Machine {
    pub asset: u64,
    pub layers: Vec<LayerState>,
}

impl Machine {
    /// Steps the machine of graph `handle` by `dt` (scaled) seconds: the
    /// transitions, the fades (`fades` false: a transition completes at
    /// once) and the phases, with each clip's seconds from `duration`
    /// (`None`: the target has no such clip; it plays nothing). Writes the
    /// component's `state` and `stateTime`; returns the leaves played, by
    /// layer, the fading-out state's before the current's.
    pub(crate) fn step(
        &mut self,
        handle: u64,
        graph: &SpriteGraph,
        comp: &mut AnimationGraph,
        dt: f32,
        fades: bool,
        duration: &dyn Fn(u64) -> Option<f32>,
    ) -> Vec<Played> {
        let g = &graph.asset;
        if self.asset != handle || self.layers.len() != g.layers.len() {
            self.asset = handle;
            self.layers = g.layers.iter().map(LayerState::entry).collect();
        }
        let params = comp.params();
        let speed = comp.speed.max(0.0);
        let mut leaders: Vec<Option<(Phase, Phase, i32)>> = vec![None; g.sync_groups.len()];
        let mut out: Vec<Played> = Vec::new();
        for (li, layer) in g.layers.iter().enumerate() {
            let ls = &mut self.layers[li];
            ls.transition(layer, &params);
            if !fades {
                ls.settle(1.0);
            }
            let alpha = ls.fade_alpha(dt);
            let mut mine: Vec<Played> = Vec::new();
            let mut run =
                |state: usize, phase: &mut Phase, w: f32, follow: Option<(Phase, Phase, i32)>| {
                    let st = &layer.states[state];
                    let mut leaves = Vec::new();
                    g.leaves(st.motion, &params, 1.0, &mut leaves);
                    let mut total = 0.0f32;
                    for &(n, lw) in &leaves {
                        if let Motion::Clip {
                            clip, speed: cs, ..
                        } = g.nodes[n].motion
                        {
                            total += lw * duration(clip).unwrap_or(0.0) / cs;
                        }
                    }
                    let (before, wraps) = match follow {
                        Some((b, a, k)) => {
                            *phase = a;
                            (b, k)
                        }
                        None => {
                            let before = *phase;
                            let wraps = if total > 0.0 {
                                phase.advance(dt * speed * st.speed / total)
                            } else {
                                0
                            };
                            (before, wraps)
                        }
                    };
                    for &(n, lw) in &leaves {
                        let Motion::Clip {
                            clip,
                            speed: cs,
                            looping,
                            ..
                        } = g.nodes[n].motion
                        else {
                            continue;
                        };
                        if duration(clip).is_none() || lw * w <= 0.0 {
                            continue;
                        }
                        mine.push(Played {
                            clip,
                            weight: lw * w,
                            layer: li as u32,
                            state: state as u32,
                            before,
                            after: *phase,
                            wraps,
                            looping,
                            rate: speed * st.speed * cs,
                            sync: st.sync,
                        });
                    }
                    (before, wraps)
                };
            if ls.prev != NONE && alpha < 1.0 {
                let (prev, mut prev_phase) = (ls.prev as usize, ls.prev_phase);
                run(prev, &mut prev_phase, 1.0 - alpha, None);
                ls.prev_phase = prev_phase;
            }
            let (cur, mut cur_phase) = (ls.state as usize, ls.phase);
            let group = layer.states[cur].sync;
            let follow = group.and_then(|k| leaders[k]);
            let (before, wraps) = run(cur, &mut cur_phase, alpha, follow);
            if let Some(k) = group
                && leaders[k].is_none()
            {
                leaders[k] = Some((before, cur_phase, wraps));
            }
            ls.phase = cur_phase;
            ls.time += dt * speed;
            ls.settle(alpha);
            // An override layer covers the layers under it by its weight.
            if layer.mode == LayerMode::Additive {
                continue;
            }
            let lw = if li == 0 { 1.0 } else { layer.weight(&params) };
            for p in &mut out {
                p.weight *= 1.0 - lw;
            }
            for mut p in mine {
                p.weight *= lw;
                out.push(p);
            }
        }
        if let Some(l) = self.layers.first() {
            comp.state = l.state;
            comp.state_time = l.time;
        }
        out.retain(|p| p.weight > 0.0);
        let sum: f32 = out.iter().map(|p| p.weight).sum();
        if sum > 0.0 {
            for p in &mut out {
                p.weight /= sum;
            }
        }
        out
    }
}

/// The heaviest leaf; of equal ones, the last (an upper layer, the
/// current state over the one fading out).
pub(crate) fn heaviest(played: &[Played]) -> Option<Played> {
    played
        .iter()
        .copied()
        .fold(None, |best: Option<Played>, p| match best {
            Some(b) if b.weight > p.weight => Some(b),
            _ => Some(p),
        })
}

fn guard_code(g: Interrupt) -> u8 {
    match g {
        Interrupt::Any => 0,
        Interrupt::None => 1,
        Interrupt::Source => 2,
        Interrupt::Next => 3,
    }
}

fn guard_of(c: u8) -> Interrupt {
    match c {
        1 => Interrupt::None,
        2 => Interrupt::Source,
        3 => Interrupt::Next,
        _ => Interrupt::Any,
    }
}

impl Machine {
    pub(crate) fn save(&self, w: &mut Writer) {
        w.u64(self.asset);
        w.u32(self.layers.len() as u32);
        for l in &self.layers {
            w.u32(l.state);
            w.u32(l.phase.cycles);
            w.f32(l.phase.frac);
            w.f32(l.time);
            w.u32(l.prev);
            w.u32(l.prev_phase.cycles);
            w.f32(l.prev_phase.frac);
            w.f32(l.fade);
            w.f32(l.fade_duration);
            w.u8(guard_code(l.guard));
        }
    }

    pub(crate) fn load(r: &mut Reader) -> Result<Machine> {
        let asset = r.u64()?;
        let n = r.count(37)?;
        let mut layers = Vec::with_capacity(n);
        for _ in 0..n {
            layers.push(LayerState {
                state: r.u32()?,
                phase: Phase {
                    cycles: r.u32()?,
                    frac: r.f32()?,
                },
                time: r.f32()?,
                prev: r.u32()?,
                prev_phase: Phase {
                    cycles: r.u32()?,
                    frac: r.f32()?,
                },
                fade: r.f32()?,
                fade_duration: r.f32()?,
                guard: guard_of(r.u8()?),
            });
        }
        Ok(Machine { asset, layers })
    }
}
