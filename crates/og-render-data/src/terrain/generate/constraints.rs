//! Authored centre lines use the automatic river carver's parabolic
//! cross-section, projected onto polyline segments rather than flow cells.
//! Preservation projects the final result back toward the input surface;
//! it is a shape constraint, not an impermeable erosion simulation.
use super::Generate;

#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    pub points: Vec<[f32; 3]>,
    pub width: f32,
    pub depth: f32,
    pub river: bool,
}

pub(super) fn check(g: &Generate) -> Result<(), String> {
    if !g.preserve.is_empty()
        && (g.preserve.len() as u64 != u64::from(g.columns) * u64::from(g.rows)
            || g.preserve
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)))
    {
        return Err("preserve needs columns × rows finite weights in 0..1".into());
    }
    if g.preserve_border > 4097 {
        return Err("preserveBorder must be 0..4097 samples".into());
    }
    for c in &g.channels {
        if c.points.len() < 2
            || c.points.len() > 65_536
            || c.points.iter().flatten().any(|v| !v.is_finite())
            || !c.width.is_finite()
            || !(0.001..=1e6).contains(&c.width)
            || !c.depth.is_finite()
            || !(0.0..=1e6).contains(&c.depth)
        {
            return Err("channel needs 2..65536 finite xyz points, width 0.001..1000000 and depth 0..1000000 metres".into());
        }
        for p in c.points.windows(2) {
            let dx = p[1][0] - p[0][0];
            let dz = p[1][2] - p[0][2];
            if !dx.is_finite()
                || !dz.is_finite()
                || !(p[1][1] - p[0][1]).is_finite()
                || !(p[0][1] - c.depth).is_finite()
                || !(p[1][1] - c.depth).is_finite()
                || dx * dx + dz * dz <= 0.0
                || !(dx * dx + dz * dz).is_finite()
                || (c.river && p[1][1] > p[0][1])
            {
                return Err("channel segments need distinct xz endpoints; river elevations must not rise downstream".into());
            }
        }
    }
    Ok(())
}

pub(super) fn carve(g: &Generate, size: [f32; 2], heights: &mut [f32]) -> f32 {
    let cols = g.columns as usize;
    let cell = [
        size[0] / (g.columns - 1) as f32,
        size[1] / (g.rows - 1) as f32,
    ];
    let ground = if g.channels.is_empty() {
        Vec::new()
    } else {
        heights.to_vec()
    };
    let mut river_length = 0.0;
    for channel in &g.channels {
        let half = channel.width * 0.5;
        for p in channel.points.windows(2) {
            let [a, b] = [p[0], p[1]];
            let [dx, dz] = [b[0] - a[0], b[2] - a[2]];
            let length2 = dx * dx + dz * dz;
            if channel.river {
                river_length += length2.sqrt();
            }
            let c0 = ((a[0].min(b[0]) - half + size[0] * 0.5) / cell[0])
                .ceil()
                .max(0.0) as usize;
            let c1 = ((a[0].max(b[0]) + half + size[0] * 0.5) / cell[0])
                .floor()
                .min((g.columns - 1) as f32);
            let r0 = ((a[2].min(b[2]) - half + size[1] * 0.5) / cell[1])
                .ceil()
                .max(0.0) as usize;
            let r1 = ((a[2].max(b[2]) + half + size[1] * 0.5) / cell[1])
                .floor()
                .min((g.rows - 1) as f32);
            if c1 < 0.0 || r1 < 0.0 {
                continue;
            }
            for r in r0..=r1 as usize {
                for c in c0..=c1 as usize {
                    let x = c as f32 * cell[0] - size[0] * 0.5 - a[0];
                    let z = r as f32 * cell[1] - size[1] * 0.5 - a[2];
                    let t = ((x * dx + z * dz) / length2).clamp(0.0, 1.0);
                    let distance2 = (x - t * dx) * (x - t * dx) + (z - t * dz) * (z - t * dz);
                    if distance2 > half * half {
                        continue;
                    }
                    let k = r * cols + c;
                    let bed = a[1] + t * (b[1] - a[1]) - channel.depth;
                    let carved = bed + (ground[k] - bed).max(0.0) * (distance2 / (half * half));
                    heights[k] = heights[k].min(carved);
                }
            }
        }
    }
    river_length
}

pub(super) fn preserve(g: &Generate, original: &[f32], heights: &mut [f32]) {
    for (k, h) in heights.iter_mut().enumerate() {
        let [c, r] = [k as u32 % g.columns, k as u32 / g.columns];
        let border = c.min(g.columns - 1 - c).min(r).min(g.rows - 1 - r) < g.preserve_border;
        let w = if border {
            1.0
        } else {
            g.preserve.get(k).copied().unwrap_or(0.0)
        };
        // Exact endpoints matter: a fully protected sample keeps its bits.
        if w == 1.0 {
            *h = original[k];
        } else if w != 0.0 {
            *h = (f64::from(*h) * (1.0 - f64::from(w)) + f64::from(original[k]) * f64::from(w))
                as f32;
        }
    }
}
