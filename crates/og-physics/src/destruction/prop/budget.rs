//! The loose islands as og.Destruction's budgets weigh them.

use super::pose::{pose_of_frame, uniform};
use super::state::{Assets, State};
use std::collections::BTreeMap;

/// A loose island (a body and the chunks it carries) as the budgets weigh
/// it: when it broke loose, its volume, and its chunks of its prop.
pub struct Loose {
    pub tick: u64,
    pub volume: f32,
    pub body: u64,
    pub prop: u64,
    pub chunks: Vec<usize>,
}

/// The loose islands, the oldest first (then the smallest, then by body).
pub fn loose_islands(st: &State, assets: &Assets) -> Vec<Loose> {
    let mut loose: BTreeMap<u64, Loose> = BTreeMap::new();
    for (&k, p) in &st.props {
        let a = &assets[&p.asset];
        let scale = uniform(pose_of_frame(&p.frame).2);
        for c in 0..a.chunks.len() {
            let e = p.entities[c];
            let b = if c < a.support as usize { p.body[c] } else { e };
            if e == 0 || b == 0 {
                continue;
            }
            let l = loose.entry(b).or_insert(Loose {
                tick: p.loose[c],
                volume: 0.0,
                body: b,
                prop: k,
                chunks: Vec::new(),
            });
            l.tick = l.tick.min(p.loose[c]);
            l.volume += a.chunks[c].volume * scale * scale * scale;
            l.chunks.push(c);
        }
    }
    let mut order: Vec<Loose> = loose.into_values().collect();
    order.sort_by(|x, y| {
        (x.tick, x.volume.to_bits(), x.body).cmp(&(y.tick, y.volume.to_bits(), y.body))
    });
    order
}
