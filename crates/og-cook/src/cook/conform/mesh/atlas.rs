//! Preserve UV chart footprints and filtering gutters when reducing a model texture.
//! Charts are connected by UV edges, not positions: a geometric seam must not weld them.
use super::{GltfSource, Image, scene_primitives, uv_set};
use glam::Vec2;
use std::collections::{BTreeMap, VecDeque};

mod repack;
pub(in crate::cook) use repack::repack;

#[derive(Clone)]
pub(super) struct Atlas {
    triangles: Vec<([Vec2; 3], usize)>,
    widths: Vec<Vec2>,
}

pub(super) struct Padding {
    // For each boundary or gutter texel, an interior texel of its own chart.
    copies: Vec<(usize, usize)>,
}

impl Padding {
    pub(super) fn apply(self, image: &mut Image) {
        for (to, from) in self.copies {
            let pixel: [u8; 4] = image.rgba[from * 4..from * 4 + 4].try_into().unwrap();
            image.rgba[to * 4..to * 4 + 4].copy_from_slice(&pixel);
        }
    }
}

impl Atlas {
    pub(super) fn of(src: &GltfSource, image: usize) -> Option<Self> {
        let mut triangles = Vec::new();
        for (p, _) in scene_primitives(src) {
            let Some((set, transform)) = uv_set(src, &p.material(), image) else {
                continue;
            };
            let reader = p.reader(|b| Some(&src.buffers[b.index()].0[..]));
            let Some(uv) = reader.read_tex_coords(set) else {
                continue;
            };
            let uv: Vec<_> = uv
                .into_f32()
                .map(|v| Vec2::from(transform.apply(v)))
                .collect();
            let indices: Vec<_> = reader.read_indices().map_or_else(
                || (0..uv.len() as u32).collect(),
                |i| i.into_u32().collect(),
            );
            for t in indices.as_chunks::<3>().0 {
                let t = t.map(|i| uv[i as usize]);
                // Repeating UVs are not a packed atlas. Leave those textures' density policy alone.
                if t.iter()
                    .any(|v| !v.is_finite() || v.min_element() < 0.0 || v.max_element() > 1.0)
                {
                    return None;
                }
                if (t[1] - t[0]).perp_dot(t[2] - t[0]).abs() > 1e-12 {
                    triangles.push(t);
                }
            }
        }
        Self::charts(triangles)
    }

    fn charts(triangles: Vec<[Vec2; 3]>) -> Option<Self> {
        let mut parent: Vec<_> = (0..triangles.len()).collect();
        fn root(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }
        let mut edges = BTreeMap::new();
        for (i, t) in triangles.iter().enumerate() {
            for j in 0..3 {
                let mut edge = [
                    t[j].to_array().map(f32::to_bits),
                    t[(j + 1) % 3].to_array().map(f32::to_bits),
                ];
                edge.sort();
                if let Some(&other) = edges.get(&edge) {
                    let a = root(&mut parent, i);
                    let b = root(&mut parent, other);
                    parent[a] = b;
                } else {
                    edges.insert(edge, i);
                }
            }
        }
        let mut charts = BTreeMap::new();
        let mut bounds: Vec<(Vec2, Vec2)> = Vec::new();
        let triangles = triangles
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let r = root(&mut parent, i);
                let c = *charts.entry(r).or_insert_with(|| {
                    bounds.push((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)));
                    bounds.len() - 1
                });
                for v in t {
                    bounds[c].0 = bounds[c].0.min(v);
                    bounds[c].1 = bounds[c].1.max(v);
                }
                (t, c)
            })
            .collect();
        // A single connected UV surface has no inter-island bleed to prevent.
        (bounds.len() > 1).then(|| Self {
            triangles,
            widths: bounds.into_iter().map(|(lo, hi)| hi - lo).collect(),
        })
    }

    pub(super) fn fit(&self, target: u32, width: u32, height: u32) -> (u32, Option<Padding>) {
        let longest = width.max(height);
        let mut target = target;
        while target < longest {
            let s = target as f64 / longest as f64;
            let w = (width as f64 * s).round().max(1.0) as u32;
            let h = (height as f64 * s).round().max(1.0) as u32;
            if self
                .widths
                .iter()
                .all(|v| (v * Vec2::new(w as f32, h as f32)).min_element() >= 4.0)
                && let Some(padding) = self.padding(w, h)
            {
                return (target, Some(padding));
            }
            target = target.saturating_mul(2).min(longest);
        }
        (longest, None)
    }

    fn padding(&self, w: u32, h: u32) -> Option<Padding> {
        let (w, h) = (w as usize, h as usize);
        let mut owner = vec![usize::MAX; w * h];
        // Use only fully interior pixels as dilation seeds: their box-filter footprint
        // cannot mix another chart or unused atlas background into the gutter.
        for &(t, chart) in &self.triangles {
            let t = t.map(|v| v * Vec2::new(w as f32, h as f32));
            let lo = t[0].min(t[1]).min(t[2]).floor().max(Vec2::ZERO);
            let hi = t[0]
                .max(t[1])
                .max(t[2])
                .ceil()
                .min(Vec2::new(w as f32, h as f32));
            let sign = (t[1] - t[0]).perp_dot(t[2] - t[0]).signum();
            for y in lo.y as usize..hi.y as usize {
                for x in lo.x as usize..hi.x as usize {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let distances = [0, 1, 2].map(|i| {
                        let e = t[(i + 1) % 3] - t[i];
                        sign * e.perp_dot(p - t[i]) / e.length()
                    });
                    if distances.iter().all(|d| *d >= 0.0) {
                        let at = y * w + x;
                        if owner[at] != usize::MAX && owner[at] != chart {
                            return None;
                        }
                        owner[at] = chart;
                    }
                }
            }
        }
        let mut seeds = Vec::new();
        let mut seen = vec![false; self.widths.len()];
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                let at = y * w + x;
                let chart = owner[at];
                if chart != usize::MAX
                    && (y - 1..=y + 1)
                        .all(|ny| (x - 1..=x + 1).all(|nx| owner[ny * w + nx] == chart))
                {
                    seeds.push(at);
                    seen[chart] = true;
                }
            }
        }
        if seen.contains(&false) {
            return None;
        }
        let mut source = vec![usize::MAX; w * h];
        let mut distance = vec![u8::MAX; w * h];
        let mut queue = VecDeque::new();
        for at in seeds {
            source[at] = at;
            distance[at] = 0;
            queue.push_back(at);
        }
        while let Some(at) = queue.pop_front() {
            let chart = owner[source[at]];
            let (x, y) = (at % w, at / w);
            for (dx, dy) in [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                    continue;
                }
                let next = ny as usize * w + nx as usize;
                // Fill the rest of this chart, then two texels outside its boundary.
                let d = if owner[next] == chart {
                    0
                } else {
                    distance[at] + 1
                };
                if d > 2 {
                    continue;
                }
                if (owner[next] != usize::MAX && owner[next] != chart)
                    || (source[next] != usize::MAX && owner[source[next]] != chart)
                {
                    return None;
                }
                if d < distance[next] {
                    source[next] = source[at];
                    distance[next] = d;
                    queue.push_back(next);
                }
            }
        }
        Some(Padding {
            copies: source
                .into_iter()
                .enumerate()
                .filter(|(to, from)| *from != usize::MAX && to != from)
                .collect(),
        })
    }
}
