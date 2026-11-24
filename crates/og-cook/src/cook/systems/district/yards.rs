//! Shared Poisson sites for a yard's weighted props. Clipping the sites,
//! rather than their models, leaves the authored margin around each one.

use glam::DVec2;
use og_core::{Rng, procgen};
use og_shape::generators::layout::Yard;

use super::parse::Yards;

pub(super) struct Site {
    pub prop: usize,
    pub position: DVec2,
    pub turn: f64,
}

pub(super) fn sites(yard: &Yard, rules: &Yards) -> Result<Vec<Site>, String> {
    if rules.max == 0 {
        return Ok(Vec::new());
    }
    let points = &yard.footprint;
    let (lo, hi) = points.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    );
    let seed = (yard.seed ^ (yard.seed >> 32)) as u32;
    let candidates = procgen::poisson_disk(
        seed,
        (hi.x - lo.x) as f32,
        (hi.y - lo.y) as f32,
        rules.spacing as f32,
        rules.attempts,
        rules.max,
    )
    .map_err(|e| format!("yard {}: {}", yard.block, e.reason()))?;
    let mut rng = Rng::new(seed);
    let total: f64 = rules.props.iter().map(|p| p.weight).sum();
    let mut out = Vec::new();
    for point in candidates {
        let position = lo + DVec2::new(f64::from(point[0]), f64::from(point[1]));
        if !(0..points.len()).all(|i| {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            (b - a).perp_dot(position - a) >= rules.margin * a.distance(b)
        }) {
            continue;
        }
        let mut weight = rng.range(0.0, total);
        let prop = rules
            .props
            .iter()
            .position(|p| {
                weight -= p.weight;
                weight < 0.0
            })
            .unwrap_or(rules.props.len() - 1);
        out.push(Site {
            prop,
            position,
            turn: rng.range(rules.turn[0], rules.turn[1]).to_radians(),
        });
    }
    Ok(out)
}
