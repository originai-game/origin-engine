//! Camera planning (trompe's `planner.ts`): sample where a player can stand,
//! cast cheap low-resolution probes from candidate cameras there, then keep
//! greedily the cameras that capture the most texture well (a set cover over
//! uv cells), ordered so each view overlaps what is already painted. Roofs
//! get close drone-style cameras of their own, a ring of aerial cameras picks
//! up what players never see, and [`hole_candidates`] feeds the last pass.

use super::semantics::{is_roof, roof_labels};
use super::{GBuffer, Level, ViewKind, ViewSpec};
use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct PlanOptions {
    pub eye_height: f32,
    /// Minimum distance between player sample points (m).
    pub spacing: f32,
    pub fov: f32,
    pub yaws: u32,
    pub pitch_deg: f32,
    pub max_views: usize,
    pub target_coverage: f32,
    pub probe_size: u32,
    /// Coverage is counted in a `grid × grid` division of uv space (the
    /// CLI uses one cell per 16 × 16 texels of the atlas, 256 to 1024).
    pub grid: u32,
    pub good_ratio: f32,
    pub good_abs: f32,
    /// Cameras per aerial ring (0: none); up to twice this are used.
    pub aerial: usize,
    /// Most close cameras over the roofs (0: none).
    pub roofs: usize,
    pub roof_distance: f32,
    pub seed: u32,
    pub threads: usize,
}

impl Default for PlanOptions {
    fn default() -> Self {
        PlanOptions {
            eye_height: 1.65,
            spacing: 4.0,
            fov: 90.0,
            yaws: 8,
            pitch_deg: 6.0,
            max_views: 36,
            target_coverage: 0.97,
            probe_size: 96,
            grid: 256,
            good_ratio: 0.5,
            good_abs: 0.12,
            aerial: 0,
            roofs: 0,
            roof_distance: 15.0,
            seed: 7,
            threads: 0,
        }
    }
}

/// The chosen cameras in paint order, with what the planner expects.
pub struct Plan {
    pub points: Vec<Vec3>,
    pub views: Vec<ViewSpec>,
    /// Predicted share of reachable cells captured well after each view.
    pub predicted: Vec<f32>,
    pub reachable_cells: usize,
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        (t ^ (t >> 14)) as f32 / 4_294_967_296.0
    }
}

fn hit_dist(level: &Level, o: Vec3, d: Vec3) -> Option<(f32, Vec3)> {
    level
        .hit(o, d, f32::MAX)
        .map(|h| (h.t, level.face_normal(h.triangle as usize)))
}

/// Eye points on walkable floors with headroom, enclosed, thinned to
/// `spacing`, in the main areas a player can walk between.
pub fn player_points(level: &Level, o: &PlanOptions) -> Vec<Vec3> {
    let mut rng = Rng(o.seed);
    let per_area = 4.0 / (o.spacing * o.spacing);
    let mut cand = Vec::new();
    for t in 0..level.triangles() {
        let n = level.face_normal(t);
        if n.y < 0.7 {
            continue;
        }
        let [a, b, c] = level.corners(t).map(|i| level.positions[i as usize]);
        let area = 0.5 * (b - a).cross(c - a).length();
        let k = area * per_area;
        let count = k.floor() as usize + (rng.next() < k.fract()) as usize;
        for _ in 0..count {
            let (mut r1, mut r2) = (rng.next(), rng.next());
            if r1 + r2 > 1.0 {
                (r1, r2) = (1.0 - r1, 1.0 - r2);
            }
            cand.push(a + (b - a) * r1 + (c - a) * r2);
        }
    }
    let mut ok = Vec::new();
    for p in cand {
        if hit_dist(level, p + Vec3::Y * 0.05, Vec3::Y).is_some_and(|(d, _)| d < o.eye_height + 0.3)
        {
            continue;
        }
        let eye = p + Vec3::Y * o.eye_height;
        match hit_dist(level, eye, Vec3::NEG_Y) {
            Some((d, _)) if (d - o.eye_height).abs() <= 0.1 => ok.push(eye),
            _ => {}
        }
    }
    // Enclosed: most horizontal directions meet a wall within 30 m from its front.
    let mut inside = vec![false; ok.len()];
    let hits: Vec<usize> = ok
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let (mut n, mut back) = (0, 0);
            for k in 0..8 {
                let a = k as f32 / 8.0 * std::f32::consts::TAU;
                let d = Vec3::new(a.sin(), 0.0, a.cos());
                if let Some((t, face)) = hit_dist(level, p, d) {
                    if t >= 30.0 {
                        continue;
                    }
                    if face.dot(d) > 0.0 { back += 1 } else { n += 1 }
                }
            }
            inside[i] = back >= 3;
            n
        })
        .collect();
    let mut open = ok.clone();
    for min in [5usize, 3, 0] {
        let keep: Vec<Vec3> = (0..ok.len())
            .filter(|&i| !inside[i] && hits[i] >= min)
            .map(|i| ok[i])
            .collect();
        if keep.len() >= 3.max(ok.len() / 4) || min == 0 {
            open = keep;
            break;
        }
    }
    // Thinning in a shuffled order.
    let mut order: Vec<usize> = (0..open.len()).collect();
    for i in (1..order.len()).rev() {
        let j = (rng.next() * (i + 1) as f32) as usize;
        order.swap(i, j.min(i));
    }
    let mut kept: Vec<Vec3> = Vec::new();
    for i in order {
        let p = open[i];
        if kept.iter().any(|q| {
            ((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt() < o.spacing
                && (q.y - p.y).abs() < 2.0
        }) {
            continue;
        }
        kept.push(p);
    }
    main_areas(level, &kept, o)
}

/// Following the floor in 0.4 m steps: no step up over 0.65 m, no drop over
/// 1.5 m, and a clear line of sight at eye height.
fn walkable(level: &Level, a: Vec3, b: Vec3, eye: f32) -> bool {
    let d = a.distance(b);
    if level.hit(a, (b - a) / d.max(1e-6), d - 0.05).is_some() {
        return false;
    }
    let steps = (((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt() / 0.4)
        .ceil()
        .max(1.0) as usize;
    let mut prev = a.y - eye;
    let top = a.y.max(b.y) + 0.2;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let o = Vec3::new(a.x + (b.x - a.x) * t, top, a.z + (b.z - a.z) * t);
        let Some(h) = level.hit(o, Vec3::NEG_Y, f32::MAX) else {
            return false;
        };
        let floor = h.position.y;
        if floor - prev > 0.65 || prev - floor > 1.5 {
            return false;
        }
        prev = floor;
    }
    (prev - (b.y - eye)).abs() < 0.3
}

fn main_areas(level: &Level, pts: &[Vec3], o: &PlanOptions) -> Vec<Vec3> {
    let mut parent: Vec<usize> = (0..pts.len()).collect();
    fn find(p: &mut [usize], i: usize) -> usize {
        let mut r = i;
        while p[r] != r {
            r = p[r];
        }
        let mut j = i;
        while p[j] != r {
            let n = p[j];
            p[j] = r;
            j = n;
        }
        r
    }
    let reach = o.spacing * 2.5;
    for i in 0..pts.len() {
        for j in i + 1..pts.len() {
            let (a, b) = (pts[i], pts[j]);
            if a.distance(b) > reach || (a.y - b.y).abs() > 3.0 {
                continue;
            }
            let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
            if ri != rj && walkable(level, a, b, o.eye_height) {
                parent[ri] = rj;
            }
        }
    }
    let roots: Vec<usize> = (0..pts.len()).map(|i| find(&mut parent, i)).collect();
    let mut size = std::collections::HashMap::new();
    for &r in &roots {
        *size.entry(r).or_insert(0usize) += 1;
    }
    let best = size.values().copied().max().unwrap_or(0);
    let min = 2.max((0.3 * best as f32).ceil() as usize);
    (0..pts.len())
        .filter(|&i| size[&roots[i]] >= min)
        .map(|i| pts[i])
        .collect()
}

pub fn ground_candidates(points: &[Vec3], o: &PlanOptions) -> Vec<ViewSpec> {
    let mut rng = Rng(o.seed.wrapping_add(1));
    let mut out = Vec::new();
    for &p in points {
        let offset = rng.next() * std::f32::consts::TAU / o.yaws as f32;
        for k in 0..o.yaws {
            out.push(ViewSpec {
                position: p,
                yaw: offset + std::f32::consts::TAU * k as f32 / o.yaws as f32,
                pitch: o.pitch_deg.to_radians(),
                fov: o.fov,
                kind: ViewKind::Ground,
            });
        }
    }
    out
}

/// A high ring looking down into the level and a low ring outside it
/// facing the outer walls.
pub fn aerial_candidates(level: &Level, ring: usize, kind: ViewKind) -> Vec<ViewSpec> {
    let (min, max) = (level.min, level.max);
    let c = (min + max) * 0.5;
    let size = max - min;
    let r = 0.5 * (size.x * size.x + size.z * size.z).sqrt();
    let mut out = Vec::new();
    for k in 0..ring {
        let a = std::f32::consts::TAU * k as f32 / ring as f32 + 0.3;
        let eye = Vec3::new(
            c.x + a.sin() * r * 0.9,
            max.y + r * 0.55,
            c.z + a.cos() * r * 0.9,
        );
        out.push(ViewSpec::looking(
            eye,
            Vec3::new(c.x, min.y, c.z),
            70.0,
            kind,
        ));
    }
    let low = min.y + 12f32.min(0.45 * size.y);
    for k in 0..2 * ring {
        let a = std::f32::consts::TAU * k as f32 / (2 * ring) as f32;
        let eye = Vec3::new(c.x + a.sin() * r * 1.05, low, c.z + a.cos() * r * 1.05);
        out.push(ViewSpec::looking(
            eye,
            Vec3::new(c.x, min.y + 3.0, c.z),
            70.0,
            kind,
        ));
    }
    out
}

/// Close cameras over the roofs: back from points spread over the roof
/// surfaces, `roof_distance` away and 50° down, facing the slope.
pub fn roof_candidates(level: &Level, o: &PlanOptions) -> Vec<ViewSpec> {
    let roofs = roof_labels(&level.labels);
    let mut rng = Rng(o.seed.wrapping_add(2));
    let spacing = o.roof_distance * 0.55;
    let mut pts: Vec<(Vec3, Vec3)> = Vec::new();
    for t in 0..level.triangles() {
        let n = level.face_normal(t);
        let [a, b, c] = level.corners(t).map(|i| level.positions[i as usize]);
        let centre = (a + b + c) / 3.0;
        if !is_roof(
            roofs.as_deref(),
            level.triangle_label[t] as usize,
            n.y,
            centre.y,
            level.min.y,
        ) {
            continue;
        }
        let k = 4.0 * 0.5 * (b - a).cross(c - a).length() / (spacing * spacing);
        let count = k.floor() as usize + (rng.next() < k.fract()) as usize;
        for _ in 0..count {
            let (mut r1, mut r2) = (rng.next(), rng.next());
            if r1 + r2 > 1.0 {
                (r1, r2) = (1.0 - r1, 1.0 - r2);
            }
            let p = a + (b - a) * r1 + (c - a) * r2;
            if pts.iter().all(|q| q.0.distance(p) >= spacing) {
                pts.push((p, n));
            }
        }
    }
    let pitch = (-50f32).to_radians();
    let mut out = Vec::new();
    for (p, n) in pts {
        let h = (n.x * n.x + n.z * n.z).sqrt();
        let base = n.x.atan2(n.z);
        let yaws: Vec<f32> = if h < 0.2 {
            [0.0, 0.5, 1.0, 1.5]
                .iter()
                .map(|f| base + f * std::f32::consts::PI)
                .collect()
        } else {
            vec![base, base - 0.7, base + 0.7]
        };
        for yaw in yaws {
            let dir = Vec3::new(
                -yaw.sin() * pitch.cos(),
                pitch.sin(),
                -yaw.cos() * pitch.cos(),
            );
            if level.hit(p + n * 0.05, -dir, o.roof_distance).is_some() {
                continue;
            }
            out.push(ViewSpec {
                position: p - dir * o.roof_distance,
                yaw,
                pitch,
                fov: 60.0,
                kind: ViewKind::Roof,
            });
        }
    }
    out
}

/// Candidates for the hole pass: 5 m above where players stand looking
/// down, and the high ring.
pub fn hole_candidates(level: &Level, points: &[Vec3], ring: usize) -> Vec<ViewSpec> {
    let mut out = Vec::new();
    let pitch = (-55f32).to_radians();
    for &p in points {
        if level.hit(p, Vec3::Y, 5.5).is_some() {
            continue;
        }
        for k in 0..4 {
            out.push(ViewSpec {
                position: p + Vec3::Y * 5.0,
                yaw: k as f32 * std::f32::consts::FRAC_PI_2 + 0.4,
                pitch,
                fov: 75.0,
                kind: ViewKind::Fill,
            });
        }
    }
    out.extend(
        aerial_candidates(level, ring.max(8), ViewKind::Fill)
            .into_iter()
            .take(ring.max(8)),
    );
    out
}

/// The uv cells a camera sees, each with the best quality seen.
pub struct Cells {
    pub cells: Vec<u32>,
    pub q: Vec<f32>,
}

fn probe(
    level: &Level,
    specs: &[ViewSpec],
    o: &PlanOptions,
    only: Option<&dyn Fn(u16, f32, f32) -> bool>,
) -> Vec<Cells> {
    let g = o.grid as usize;
    let mut scratch = vec![0f32; g * g];
    specs
        .iter()
        .map(|&s| {
            let gb = GBuffer::cast(level, s, o.probe_size, o.threads);
            let mut touched = Vec::new();
            for i in 0..gb.hits.len() {
                let Some(smp) = gb.sample(i) else { continue };
                if smp.quality <= 0.0 {
                    continue;
                }
                if only.is_some_and(|f| !f(smp.hit.label, smp.hit.normal.y, smp.hit.position.y)) {
                    continue;
                }
                let cx = ((smp.hit.uv.x * g as f32) as usize).min(g - 1);
                let cy = ((smp.hit.uv.y * g as f32) as usize).min(g - 1);
                let c = cy * g + cx;
                if scratch[c] == 0.0 {
                    touched.push(c as u32);
                }
                scratch[c] = scratch[c].max(smp.quality);
            }
            let q = touched
                .iter()
                .map(|&c| std::mem::take(&mut scratch[c as usize]))
                .collect();
            Cells { cells: touched, q }
        })
        .collect()
}

/// Lazy greedy set cover over cells, then an order where every view
/// overlaps what earlier ones painted. `have` holds cells already captured
/// well (shared across passes); `skip` cells belong to another pass.
fn select(
    lists: &[Cells],
    o: &PlanOptions,
    max_views: usize,
    target: f32,
    have: &mut [u8],
    skip: Option<&[u8]>,
    aerial: bool,
) -> Vec<usize> {
    let g2 = (o.grid * o.grid) as usize;
    let mut best = vec![0f32; g2];
    for l in lists {
        for (c, q) in l.cells.iter().zip(&l.q) {
            best[*c as usize] = best[*c as usize].max(*q);
        }
    }
    if let Some(s) = skip {
        for c in 0..g2 {
            if s[c] != 0 {
                best[c] = 0.0;
            }
        }
    }
    let reachable = best.iter().filter(|&&q| q > 0.0).count();
    if reachable == 0 {
        return Vec::new();
    }
    let good = |c: u32, q: f32| {
        best[c as usize] > 0.0 && q >= (o.good_ratio * best[c as usize]).min(o.good_abs)
    };
    let mut covered = (0..g2).filter(|&c| best[c] > 0.0 && have[c] != 0).count();
    let gain = |k: usize, have: &[u8]| {
        let l = &lists[k];
        let g = (0..l.cells.len())
            .filter(|&i| have[l.cells[i] as usize] == 0 && good(l.cells[i], l.q[i]))
            .count();
        if aerial { g / 2 } else { g }
    };
    let mut heap: Vec<(usize, usize)> = (0..lists.len()).map(|k| (gain(k, have), k)).collect();
    heap.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut chosen = Vec::new();
    // A view must newly cover a few cells, however large the level: a floor
    // relative to everything reachable stops a town's plan after a handful
    // of street views. `max_views` and `target` bound the rest.
    let floor = 8;
    while chosen.len() < max_views
        && !heap.is_empty()
        && (covered as f32) / (reachable as f32) < target
    {
        let (_, k) = heap.remove(0);
        let g = gain(k, have);
        if heap.first().is_some_and(|h| g < h.0) {
            let at = heap.partition_point(|h| h.0 > g);
            heap.insert(at, (g, k));
            continue;
        }
        if g < floor {
            break;
        }
        chosen.push(k);
        let l = &lists[k];
        for i in 0..l.cells.len() {
            let c = l.cells[i] as usize;
            if have[c] == 0 && good(l.cells[i], l.q[i]) {
                have[c] = 1;
                if best[c] > 0.0 {
                    covered += 1;
                }
            }
        }
    }
    if aerial || chosen.len() < 2 {
        return chosen;
    }
    // Paint order: biggest first, then always the one overlapping the painted most.
    let mut painted = vec![0u8; g2];
    let mut order = vec![chosen[0]];
    for &c in &lists[chosen[0]].cells {
        painted[c as usize] = 1;
    }
    let mut rest: Vec<usize> = chosen[1..].to_vec();
    while !rest.is_empty() {
        let (mut bi, mut bs) = (0, -1.0f32);
        for (ri, &k) in rest.iter().enumerate() {
            let l = &lists[k];
            let ov = l
                .cells
                .iter()
                .filter(|&&c| painted[c as usize] != 0)
                .count() as f32
                / l.cells.len().max(1) as f32;
            let rank =
                chosen.iter().position(|&c| c == k).unwrap_or(0) as f32 / chosen.len() as f32;
            let score = if ov > 0.08 {
                1.0 + ov * 0.1 + (1.0 - rank)
            } else {
                ov
            };
            if score > bs {
                (bi, bs) = (ri, score);
            }
        }
        let k = rest.remove(bi);
        for &c in &lists[k].cells {
            painted[c as usize] = 1;
        }
        order.push(k);
    }
    order
}

pub fn plan(level: &Level, o: &PlanOptions) -> Result<Plan, String> {
    let points = player_points(level, o);
    if points.is_empty() {
        return Err("found nowhere to stand: is the level Y-up with walkable floors?".into());
    }
    let g2 = (o.grid * o.grid) as usize;
    let ground = ground_candidates(&points, o);
    let ground_cells = probe(level, &ground, o, None);
    let roofs = roof_labels(&level.labels);
    let ground_y = level.min.y;
    let roof_test =
        |l: u16, ny: f32, y: f32| is_roof(roofs.as_deref(), l as usize, ny, y, ground_y);
    let roof = if o.roofs > 0 {
        roof_candidates(level, o)
    } else {
        Vec::new()
    };
    let roof_cells = probe(level, &roof, o, Some(&roof_test));
    let air = if o.aerial > 0 {
        aerial_candidates(level, o.aerial, ViewKind::Aerial)
    } else {
        Vec::new()
    };
    let air_cells = probe(level, &air, o, None);
    let roof_mask = (!roof.is_empty()).then(|| {
        let mut m = vec![0u8; g2];
        for l in &roof_cells {
            for &c in &l.cells {
                m[c as usize] = 1;
            }
        }
        m
    });
    let mut have = vec![0u8; g2];
    let gsel = select(
        &ground_cells,
        o,
        o.max_views,
        o.target_coverage,
        &mut have,
        roof_mask.as_deref(),
        false,
    );
    let mut roof_have = vec![0u8; g2];
    let rsel = select(&roof_cells, o, o.roofs, 0.98, &mut roof_have, None, true);
    let asel = select(
        &air_cells,
        o,
        2 * o.aerial,
        o.target_coverage,
        &mut have,
        roof_mask.as_deref(),
        true,
    );
    let picked: Vec<(ViewSpec, &Cells)> = gsel
        .iter()
        .map(|&k| (ground[k], &ground_cells[k]))
        .chain(rsel.iter().map(|&k| (roof[k], &roof_cells[k])))
        .chain(asel.iter().map(|&k| (air[k], &air_cells[k])))
        .collect();
    // Predicted coverage over everything any candidate sees well.
    let mut q_all = vec![0f32; g2];
    for l in ground_cells.iter().chain(&roof_cells).chain(&air_cells) {
        for (c, q) in l.cells.iter().zip(&l.q) {
            q_all[*c as usize] = q_all[*c as usize].max(*q);
        }
    }
    let reachable = q_all.iter().filter(|&&q| q > 0.0).count();
    let mut cov = vec![false; g2];
    let mut n = 0usize;
    let predicted = picked
        .iter()
        .map(|(_, l)| {
            for (c, q) in l.cells.iter().zip(&l.q) {
                let c = *c as usize;
                if !cov[c] && *q >= (o.good_ratio * q_all[c]).min(o.good_abs) {
                    cov[c] = true;
                    n += 1;
                }
            }
            n as f32 / reachable.max(1) as f32
        })
        .collect();
    Ok(Plan {
        points,
        views: picked.into_iter().map(|p| p.0).collect(),
        predicted,
        reachable_cells: reachable,
    })
}
