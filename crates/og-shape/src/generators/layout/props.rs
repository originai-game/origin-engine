//! Sites: where props stand along each block's pavement.

use super::geom::{EPS, edge, grown};
use super::{Block, PropDesc, Site, Stream, Street, key};

/// Every prop's sites, block by block, edge by edge.
pub(super) fn sites(
    blocks: &[Block],
    streets: &[Street],
    props: &[PropDesc],
    seed: u64,
) -> Result<Vec<Site>, String> {
    for p in props {
        if let Some(name) = &p.street
            && !streets.iter().any(|s| &s.name == name)
        {
            let named: Vec<&str> = streets
                .iter()
                .filter(|s| !s.name.is_empty())
                .map(|s| s.name.as_str())
                .collect();
            return Err(format!(
                "prop `{}` names street `{name}`, which the plan does not have; its streets: {}",
                p.name,
                if named.is_empty() {
                    "none named".into()
                } else {
                    named.join(", ")
                }
            ));
        }
    }
    let mut out = Vec::new();
    for (bi, block) in blocks.iter().enumerate() {
        let (pts, tags) = (&block.poly.points, &block.poly.streets);
        for (k, p) in props.iter().enumerate() {
            let takes = |s: Option<usize>| {
                s.is_some_and(|s| p.street.as_ref().is_none_or(|n| &streets[s].name == n))
            };
            // The line props stand on: `kerb` in from the roadway's edge
            // (never past the block's own edge).
            let out_by: Vec<f64> = tags
                .iter()
                .map(|t| t.map_or(0.0, |s| (streets[s].sidewalk - p.kerb).max(0.0)))
                .collect();
            let Some(line) = grown(pts, &out_by) else {
                continue;
            };
            let m = line.len();
            for j in 0..m {
                let i = line.streets[j].expect("grown tags every edge");
                if !takes(tags[i]) {
                    continue;
                }
                let street = tags[i].expect("taken");
                let (a, b) = (line.points[j], line.points[(j + 1) % m]);
                let (e, inn) = edge(pts, i);
                let len = a.distance(b);
                if p.every > 0.0 && len > EPS {
                    let count = (len / p.every).floor() as usize;
                    let margin = (len - p.every * count as f64) / 2.0;
                    for c in 0..count {
                        let at = margin + p.every * (c as f64 + 0.5);
                        let place = a + e * at;
                        let mut s = Stream::new(key(seed, &format!("prop {}", p.name), place));
                        let slide = s.range(-p.jitter, p.jitter).clamp(-at, len - at);
                        out.push(Site {
                            prop: k,
                            block: bi,
                            street,
                            position: place + e * slide,
                            facing: -inn,
                            corner: false,
                        });
                    }
                }
                let next = line.streets[(j + 1) % m].expect("grown tags every edge");
                if p.corners && next != i && takes(tags[next]) {
                    let (_, inn2) = edge(pts, next);
                    out.push(Site {
                        prop: k,
                        block: bi,
                        street,
                        position: b,
                        facing: -(inn + inn2).normalize_or(inn),
                        corner: true,
                    });
                }
            }
        }
    }
    Ok(out)
}
