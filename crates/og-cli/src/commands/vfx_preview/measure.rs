//! Step 2 of `og vfx preview`: the box each effect reaches, measured from
//! frames of it alone (the steps are in the parent module's doc).

use super::{
    Bounds, CHUNK, CORE_TRIM, Effect, MEASURE_HALF, MEASURE_HALF_MAX, MEASURE_PX, Measured,
    SAMPLES, Stage, THRESHOLD, TICK, Timing, open,
};
use image::RgbaImage;
use std::path::Path;

/// Where an effect showed in one measuring view, over every sample: how
/// many of its pixels fell in each column and each row.
struct Seen {
    cols: Vec<u64>,
    rows: Vec<u64>,
}

impl Seen {
    fn new() -> Seen {
        Seen {
            cols: vec![0; MEASURE_PX as usize],
            rows: vec![0; MEASURE_PX as usize],
        }
    }

    /// Counts the pixels of `frame` that differ from `empty`, and says
    /// whether any did.
    fn add(&mut self, empty: &RgbaImage, frame: &RgbaImage) -> bool {
        let mut any = false;
        for (x, y, p) in frame.enumerate_pixels() {
            let q = empty.get_pixel(x, y);
            if (0..3).any(|c| (i16::from(p[c]) - i16::from(q[c])).abs() > THRESHOLD) {
                self.cols[x as usize] += 1;
                self.rows[y as usize] += 1;
                any = true;
            }
        }
        any
    }

    /// The pixel box (x0, y0, x1, y1, inclusive) holding all it showed but
    /// `trim` of its pixels at each end of each axis (0: everything), or
    /// None when it showed nothing.
    fn pixels(&self, trim: f64) -> Option<[u32; 4]> {
        let span = |counts: &[u64]| -> Option<(u32, u32)> {
            let total: u64 = counts.iter().sum();
            if total == 0 {
                return None;
            }
            let cut = (total as f64 * trim) as u64;
            let mut sum = 0;
            let lo = counts.iter().position(|&c| {
                sum += c;
                sum > cut
            })?;
            let mut sum = 0;
            let hi = counts.iter().rposition(|&c| {
                sum += c;
                sum > cut
            })?;
            Some((lo as u32, hi as u32))
        };
        let (x0, x1) = span(&self.cols)?;
        let (y0, y1) = span(&self.rows)?;
        Some([x0, y0, x1, y1])
    }

    /// The share of its pixels in the view's lower half.
    fn below(&self) -> f64 {
        let total: u64 = self.rows.iter().sum();
        let below: u64 = self.rows[self.rows.len() / 2..].iter().sum();
        if total == 0 {
            0.0
        } else {
            below as f64 / total as f64
        }
    }
}

/// The measuring views for a half side of `half` metres at the origin:
/// from the front, level with the ground (x right, y up, the origin at the
/// centre), and from above (x right, z down the image).
fn measure_views(half: f64) -> [String; 2] {
    let d = half * 40.0;
    let fov = 2.0 * (half / d).atan().to_degrees();
    [
        format!("eye:0:0:{d}:0:0:0:{fov}"),
        format!("eye:0:{d}:{}:0:0:0:{fov}", d * 1e-3),
    ]
}

/// The box in metres of the pixel boxes seen from the front and from above
/// with a half side of `half`, and whether either reached its view's edge.
fn metres(front: Option<[u32; 4]>, top: Option<[u32; 4]>, half: f64) -> Option<(Bounds, bool)> {
    let s = f64::from(MEASURE_PX);
    // A pixel column's or row's place across the view, -half to half.
    let at = |px: u32, edge: f64| ((f64::from(px) + edge) / s - 0.5) * 2.0 * half;
    let edge =
        |b: [u32; 4]| b[0] == 0 || b[1] == 0 || b[2] == MEASURE_PX - 1 || b[3] == MEASURE_PX - 1;
    let mut x: Option<(f64, f64)> = None;
    let mut widen = |lo: f64, hi: f64| {
        x = Some(x.map_or((lo, hi), |(a, b): (f64, f64)| (a.min(lo), b.max(hi))));
    };
    let mut y = (0.0, 0.0);
    let mut z = (0.0, 0.0);
    if let Some(f) = front {
        widen(at(f[0], 0.0), at(f[2], 1.0));
        y = (-at(f[3], 1.0), -at(f[1], 0.0));
    }
    if let Some(t) = top {
        widen(at(t[0], 0.0), at(t[2], 1.0));
        z = (at(t[1], 0.0), at(t[3], 1.0));
    }
    let (x0, x1) = x?;
    // Seen from above alone (a decal, a ground ring): flat on the ground.
    // Seen from the front alone: under the ground, its depth unknown.
    Some((
        Bounds {
            min: [x0, y.0, z.0],
            max: [x1, y.1, z.1],
            clipped: false,
        },
        front.is_some_and(edge) || top.is_some_and(edge),
    ))
}

/// Step 2: each effect's measured bounds (None for one that draws nothing
/// that shows over black).
pub(super) fn bounds(
    stage: &Stage,
    bundle: &Path,
    effects: &[Effect],
    timing: &[Timing],
) -> Result<Vec<Option<Measured>>, String> {
    let mut out: Vec<Option<Measured>> = vec![None; effects.len()];
    // The half side each effect was last measured with.
    let mut halves = vec![MEASURE_HALF; effects.len()];
    let mut todo: Vec<usize> = (0..effects.len()).collect();
    let mut half = MEASURE_HALF;
    while !todo.is_empty() {
        let views = measure_views(half);
        let mut again = Vec::new();
        for chunk in todo.chunks(CHUNK) {
            let mut commands = vec!["0:stage:measure".to_string()];
            for (v, name) in views.iter().zip(["f", "t"]) {
                commands.push(format!("0.3:{v}"));
                commands.push(format!("0.3:screenshot:empty-{name}.png"));
            }
            let mut t = 0.5;
            for &i in chunk {
                commands.push(format!("{t:.3}:play:{}", effects[i].id));
                for (k, f) in SAMPLES.iter().enumerate() {
                    let at = t + (f * timing[i].show).max(1.0 / TICK);
                    for (v, name) in views.iter().zip(["f", "t"]) {
                        commands.push(format!("{at:.3}:{v}"));
                        commands.push(format!("{at:.3}:screenshot:m{i}-{name}{k}.png"));
                    }
                }
                t += timing[i].show;
                commands.push(format!("{t:.3}:clear"));
                t += 0.2;
            }
            let (_, dir) = stage.run(bundle, (MEASURE_PX, MEASURE_PX), commands, t + 0.1, &[])?;
            let empty = [
                open(&dir.join("empty-f.png"))?,
                open(&dir.join("empty-t.png"))?,
            ];
            for &i in chunk {
                let mut seen = [Seen::new(), Seen::new()];
                // The first sample after the last that showed anything.
                let mut busy = 0;
                for k in 0..SAMPLES.len() {
                    for (v, name) in ["f", "t"].iter().enumerate() {
                        let frame = open(&dir.join(format!("m{i}-{name}{k}.png")))?;
                        if seen[v].add(&empty[v], &frame) {
                            busy = k + 1;
                        }
                    }
                }
                let busy = SAMPLES.get(busy).copied().unwrap_or(1.0);
                let reach = metres(seen[0].pixels(0.0), seen[1].pixels(0.0), half);
                let core = metres(seen[0].pixels(CORE_TRIM), seen[1].pixels(CORE_TRIM), half);
                halves[i] = half;
                match reach.zip(core) {
                    None => out[i] = None,
                    Some(((b, edge), (core, _))) => {
                        let widest = half * 2.0 > MEASURE_HALF_MAX;
                        out[i] = Some(Measured {
                            reach: Bounds {
                                clipped: edge && widest,
                                ..b
                            },
                            core,
                            below: seen[0].below(),
                            busy,
                            marks: timing[i].elements.iter().any(|e| e == "decal"),
                        });
                        if edge && !widest {
                            again.push(i);
                        }
                    }
                }
            }
            stage.done(&dir);
        }
        todo = again;
        half *= 2.0;
    }
    depth(stage, bundle, effects, timing, &halves, &mut out)?;
    let flat: Vec<usize> = (0..effects.len())
        .filter(|&i| out[i].is_none() && timing[i].elements.iter().any(|e| e == "decal"))
        .collect();
    marks(stage, bundle, effects, timing, flat, &mut out)?;
    Ok(out)
}

/// The depth of the effects anchored at their centre: from above, what one
/// draws below its origin is under the ground, so each is seen from above
/// once more raised to stand on it, as the shots play it, and its box takes
/// that view's x and z.
fn depth(
    stage: &Stage,
    bundle: &Path,
    effects: &[Effect],
    timing: &[Timing],
    halves: &[f64],
    out: &mut [Option<Measured>],
) -> Result<(), String> {
    let raised: Vec<usize> = (0..effects.len())
        .filter(|&i| out[i].is_some_and(|m| m.lift() > 0.0))
        .collect();
    let mut sizes: Vec<f64> = raised.iter().map(|&i| halves[i]).collect();
    sizes.sort_by(f64::total_cmp);
    sizes.dedup();
    for half in sizes {
        let [_, top] = measure_views(half);
        let of: Vec<usize> = raised
            .iter()
            .copied()
            .filter(|&i| halves[i] == half)
            .collect();
        for chunk in of.chunks(CHUNK) {
            let mut commands = vec![
                "0:stage:measure".to_string(),
                format!("0.3:{top}"),
                "0.3:screenshot:empty.png".to_string(),
            ];
            let mut t = 0.5;
            for &i in chunk {
                let lift = out[i].map_or(0.0, |m| m.lift());
                commands.push(format!("{t:.3}:lift:{lift:.4}"));
                commands.push(format!("{t:.3}:play:{}", effects[i].id));
                for (k, f) in SAMPLES.iter().enumerate() {
                    let at = t + (f * timing[i].show).max(1.0 / TICK);
                    commands.push(format!("{at:.3}:screenshot:m{i}-{k}.png"));
                }
                t += timing[i].show;
                commands.push(format!("{t:.3}:clear"));
                t += 0.2;
            }
            let (_, dir) = stage.run(bundle, (MEASURE_PX, MEASURE_PX), commands, t + 0.1, &[])?;
            let empty = open(&dir.join("empty.png"))?;
            for &i in chunk {
                let mut seen = Seen::new();
                for k in 0..SAMPLES.len() {
                    seen.add(&empty, &open(&dir.join(format!("m{i}-{k}.png")))?);
                }
                let reach = metres(None, seen.pixels(0.0), half);
                let core = metres(None, seen.pixels(CORE_TRIM), half);
                if let (Some(m), Some(((reach, _), (core, _)))) = (out[i].as_mut(), reach.zip(core))
                {
                    for (b, seen) in [(&mut m.reach, reach), (&mut m.core, core)] {
                        for axis in [0, 2] {
                            b.min[axis] = b.min[axis].min(seen.min[axis]);
                            b.max[axis] = b.max[axis].max(seen.max[axis]);
                        }
                    }
                }
            }
            stage.done(&dir);
        }
    }
    Ok(())
}

/// The flat boxes of effects that drew nothing over black and have a decal
/// (`todo`): a decal lands on lit surfaces alone, so each is seen from above
/// on a ground lit from straight overhead (stage.ts `stage:marks`), where
/// what it left differs from the empty ground.
fn marks(
    stage: &Stage,
    bundle: &Path,
    effects: &[Effect],
    timing: &[Timing],
    mut todo: Vec<usize>,
    out: &mut [Option<Measured>],
) -> Result<(), String> {
    let mut half = MEASURE_HALF;
    while !todo.is_empty() {
        let [_, top] = measure_views(half);
        let mut again = Vec::new();
        for chunk in todo.chunks(CHUNK) {
            let mut commands = vec![
                "0:stage:marks".to_string(),
                format!("0.3:{top}"),
                "0.3:screenshot:empty.png".to_string(),
            ];
            let mut t = 0.5;
            for &i in chunk {
                commands.push(format!("{t:.3}:play:{}", effects[i].id));
                for (k, f) in SAMPLES.iter().enumerate() {
                    let at = t + (f * timing[i].show).max(1.0 / TICK);
                    commands.push(format!("{at:.3}:screenshot:m{i}-{k}.png"));
                }
                t += timing[i].show;
                commands.push(format!("{t:.3}:clear"));
                t += 0.2;
            }
            let (_, dir) = stage.run(bundle, (MEASURE_PX, MEASURE_PX), commands, t + 0.1, &[])?;
            let empty = open(&dir.join("empty.png"))?;
            for &i in chunk {
                let mut seen = Seen::new();
                for k in 0..SAMPLES.len() {
                    seen.add(&empty, &open(&dir.join(format!("m{i}-{k}.png")))?);
                }
                let reach = metres(None, seen.pixels(0.0), half);
                let core = metres(None, seen.pixels(CORE_TRIM), half);
                if let Some(((b, edge), (core, _))) = reach.zip(core) {
                    let widest = half * 2.0 > MEASURE_HALF_MAX;
                    out[i] = Some(Measured {
                        reach: Bounds {
                            clipped: edge && widest,
                            ..b
                        },
                        core,
                        below: 0.0,
                        busy: 1.0,
                        marks: true,
                    });
                    if edge && !widest {
                        again.push(i);
                    }
                }
            }
            stage.done(&dir);
        }
        todo = again;
        half *= 2.0;
    }
    Ok(())
}
