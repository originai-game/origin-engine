//! Lightmaps: every lightmapped mesh's triangles rasterised into the
//! atlas pages, each covered texel traced from its surface point over the
//! hemisphere of its normal, progressively (passes of samples, each pass
//! added to the last), then the uncovered and inside-geometry texels filled
//! from their neighbours so bilinear filtering never reads black, and a
//! light edge-aware filter over each chart.
//!
//! A texel stores the irradiance from the scene over π (the renderer
//! multiplies it by the albedo, as it does the environment's irradiance),
//! in linear RGB, with alpha 1 where a surface was traced.

use crate::atlas::Atlas;
use crate::scene::Scene;
use crate::trace::{Rng, Tracer, cosine_dir, r2};
use glam::{Vec2, Vec3};
use std::time::Instant;

/// How long and how finely a lightmap is traced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceSettings {
    /// Hemisphere samples per texel.
    pub samples: u32,
    /// Samples per progressive pass (the progress callback runs after each).
    pub pass: u32,
    /// Path vertices past the first.
    pub bounces: u32,
    /// Stop after the pass that passes this many seconds, whatever
    /// `samples` says (0: no limit). Only `samples` makes a bake repeatable.
    pub seconds: f32,
    /// The edge-aware filter after tracing.
    pub denoise: bool,
    /// Worker threads (0: the machine's).
    pub threads: usize,
    pub seed: u64,
}

impl Default for TraceSettings {
    fn default() -> Self {
        TraceSettings {
            samples: 256,
            pass: 32,
            bounces: 3,
            seconds: 0.0,
            denoise: true,
            threads: 0,
            seed: 1,
        }
    }
}

/// The traced pages: `width × height` texels each, row by row from the top.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lightmap {
    pub width: u32,
    pub height: u32,
    pub pages: Vec<Vec<[f32; 4]>>,
    /// Samples every covered texel got.
    pub samples: u32,
    /// Texels traced (covered by some triangle).
    pub texels: usize,
}

/// A covered texel: where on which surface it lies.
#[derive(Clone, Copy, Debug)]
struct Texel {
    page: u32,
    index: u32,
    position: Vec3,
    normal: Vec3,
    face: Vec3,
    /// Its centre is inside its triangle (else the nearest point of one).
    interior: bool,
}

/// Rasterises the atlas: per page, the texels its triangles cover, with
/// their world point and normals. `meshes[i]` is the scene mesh atlas
/// mesh `i` was charted from.
fn rasterise(scene: &Scene, atlas: &Atlas, meshes: &[usize]) -> Vec<Texel> {
    let (w, h) = (atlas.width as i32, atlas.height as i32);
    let pages = atlas.pages as usize;
    let mut slots: Vec<Option<Texel>> = vec![None; pages * (w * h) as usize];
    for (am, &si) in atlas.meshes.iter().zip(meshes) {
        let src = &scene.meshes[si];
        for tri in am.indices.as_chunks::<3>().0 {
            let v = [tri[0], tri[1], tri[2]].map(|i| am.vertices[i as usize]);
            let page = v[0].page;
            let uv = v.map(|x| Vec2::from(x.uv));
            let p = v.map(|x| src.positions[x.xref as usize]);
            let n = v.map(|x| src.normals.get(x.xref as usize).copied().unwrap_or(Vec3::Y));
            let face = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or(Vec3::ZERO);
            if face == Vec3::ZERO {
                continue;
            }
            let area = (uv[1] - uv[0]).perp_dot(uv[2] - uv[0]);
            if area.abs() < 1e-12 {
                continue;
            }
            let min = uv[0].min(uv[1]).min(uv[2]);
            let max = uv[0].max(uv[1]).max(uv[2]);
            let (x0, y0) = (
                (min.x - 1.0).floor().max(0.0) as i32,
                (min.y - 1.0).floor().max(0.0) as i32,
            );
            let (x1, y1) = (
                (max.x + 1.0).ceil().min(w as f32 - 1.0) as i32,
                (max.y + 1.0).ceil().min(h as f32 - 1.0) as i32,
            );
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let c = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let bary = |q: Vec2| {
                        let b1 = (q - uv[0]).perp_dot(uv[2] - uv[0]) / area;
                        let b2 = (uv[1] - uv[0]).perp_dot(q - uv[0]) / area;
                        Vec3::new(1.0 - b1 - b2, b1, b2)
                    };
                    let b = bary(c);
                    let inside = b.min_element() >= -1e-4;
                    let (b, interior) = if inside {
                        (b, true)
                    } else {
                        // The nearest point of the triangle, if within the texel.
                        let q = closest_on_triangle(c, uv);
                        if (q - c).length() > 0.75 {
                            continue;
                        }
                        // Nudged inside, off the edge.
                        let cen = (uv[0] + uv[1] + uv[2]) / 3.0;
                        (bary(q + (cen - q) * 0.01), false)
                    };
                    let slot = &mut slots[page as usize * (w * h) as usize + (y * w + x) as usize];
                    if slot.is_some_and(|s| s.interior || !interior) {
                        continue;
                    }
                    let position = p[0] * b.x + p[1] * b.y + p[2] * b.z;
                    let mut normal = (n[0] * b.x + n[1] * b.y + n[2] * b.z).normalize_or(face);
                    if normal.dot(face) < 0.0 {
                        normal = face;
                    }
                    *slot = Some(Texel {
                        page,
                        index: (y * w + x) as u32,
                        position,
                        normal,
                        face,
                        interior,
                    });
                }
            }
        }
    }
    slots.into_iter().flatten().collect()
}

fn closest_on_triangle(p: Vec2, t: [Vec2; 3]) -> Vec2 {
    let seg = |a: Vec2, b: Vec2| {
        let ab = b - a;
        let s = ((p - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        a + ab * s
    };
    let cands = [seg(t[0], t[1]), seg(t[1], t[2]), seg(t[2], t[0])];
    cands
        .into_iter()
        .min_by(|a, b| {
            (*a - p)
                .length_squared()
                .total_cmp(&(*b - p).length_squared())
        })
        .unwrap_or(p)
}

/// Bakes the lightmap of `atlas` (charted from `meshes`, indices into the
/// scene's meshes). `progress` hears the samples done, the total and the
/// seconds so far after each pass.
pub fn bake(
    scene: &Scene,
    atlas: &Atlas,
    meshes: &[usize],
    settings: &TraceSettings,
    progress: &mut dyn FnMut(u32, u32, f32),
) -> Lightmap {
    let started = Instant::now();
    let tracer = Tracer::new(scene, settings.bounces);
    let texels = rasterise(scene, atlas, meshes);
    let n = texels.len();
    let threads = if settings.threads == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        settings.threads
    }
    .max(1);
    // Per texel: the radiance sum, the samples that started inside geometry.
    let mut sums = vec![Vec3::ZERO; n];
    let mut inside = vec![0u32; n];
    let total = settings.samples.max(1);
    let pass = settings.pass.clamp(1, total);
    let mut done = 0;
    while done < total {
        let count = pass.min(total - done);
        let chunk = n.div_ceil(threads).max(1);
        std::thread::scope(|s| {
            for ((ts, sum), ins) in texels
                .chunks(chunk)
                .zip(sums.chunks_mut(chunk))
                .zip(inside.chunks_mut(chunk))
            {
                let tracer = &tracer;
                s.spawn(move || {
                    for ((t, sum), ins) in ts.iter().zip(sum.iter_mut()).zip(ins.iter_mut()) {
                        let seed = settings.seed ^ ((t.page as u64) << 40) ^ t.index as u64;
                        let mut rng = Rng::new(
                            seed.wrapping_mul(0x2545_F491_4F6C_DD1D)
                                .wrapping_add(done as u64),
                        );
                        let mut shift = Rng::new(seed);
                        let offset = Vec2::new(shift.f32(), shift.f32());
                        let origin = t.position + t.face * tracer.epsilon * 4.0;
                        for i in done..done + count {
                            let mut d = cosine_dir(t.normal, r2(i, offset));
                            if d.dot(t.face) <= 0.0 {
                                d = (d - t.face * d.dot(t.face) * 1.01).normalize_or(t.face);
                            }
                            let (l, back) = tracer.radiance(origin, d, &mut rng);
                            *sum += l;
                            *ins += back as u32;
                        }
                    }
                });
            }
        });
        done += count;
        let secs = started.elapsed().as_secs_f32();
        progress(done, total, secs);
        if settings.seconds > 0.0 && secs >= settings.seconds {
            break;
        }
    }
    let (w, h) = (atlas.width as usize, atlas.height as usize);
    let mut pages = vec![vec![[0.0f32; 4]; w * h]; atlas.pages as usize];
    // Normals per texel for the filter, and which texels are valid.
    let mut normals = vec![vec![Vec3::ZERO; w * h]; atlas.pages as usize];
    for (i, t) in texels.iter().enumerate() {
        // A texel whose samples start inside geometry (behind a wall that
        // meets its surface) would show the inside's darkness: dropped,
        // and filled from its neighbours below.
        if inside[i] * 4 > done {
            continue;
        }
        let e = sums[i] / done.max(1) as f32;
        pages[t.page as usize][t.index as usize] = [e.x, e.y, e.z, 1.0];
        normals[t.page as usize][t.index as usize] = t.normal;
    }
    if settings.denoise {
        for (page, nrm) in pages.iter_mut().zip(&normals) {
            *page = filter(page, nrm, w, h);
        }
    }
    for page in &mut pages {
        dilate(page, w, h, 8);
    }
    Lightmap {
        width: atlas.width,
        height: atlas.height,
        pages,
        samples: done,
        texels: n,
    }
}

/// One 3 × 3 pass weighting neighbours by how alike their normals are
/// (charts meeting at an edge keep their own light).
fn filter(page: &[[f32; 4]], normals: &[Vec3], w: usize, h: usize) -> Vec<[f32; 4]> {
    let mut out = page.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if page[i][3] <= 0.0 {
                continue;
            }
            let n0 = normals[i];
            let mut sum = Vec3::ZERO;
            let mut weight = 0.0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                        continue;
                    }
                    let j = yy as usize * w + xx as usize;
                    if page[j][3] <= 0.0 {
                        continue;
                    }
                    let similar = n0.dot(normals[j]);
                    if similar < 0.9 {
                        continue;
                    }
                    let k = if dx == 0 && dy == 0 {
                        4.0
                    } else if dx == 0 || dy == 0 {
                        2.0
                    } else {
                        1.0
                    };
                    sum += Vec3::new(page[j][0], page[j][1], page[j][2]) * k;
                    weight += k;
                }
            }
            let c = sum / weight;
            out[i] = [c.x, c.y, c.z, 1.0];
        }
    }
    out
}

/// Fills empty texels (alpha 0) next to filled ones with their mean, ring
/// by ring, `rings` times; the filled ones keep alpha 1, the dilated ones
/// get 0.5 (still lit, never traced).
pub fn dilate(page: &mut [[f32; 4]], w: usize, h: usize, rings: u32) {
    for _ in 0..rings {
        let src = page.to_vec();
        let mut grew = false;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if src[i][3] > 0.0 {
                    continue;
                }
                let mut sum = [0.0f32; 3];
                let mut count = 0.0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                        if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                            continue;
                        }
                        let j = yy as usize * w + xx as usize;
                        if src[j][3] > 0.0 {
                            for k in 0..3 {
                                sum[k] += src[j][k];
                            }
                            count += 1.0;
                        }
                    }
                }
                if count > 0.0 {
                    page[i] = [sum[0] / count, sum[1] / count, sum[2] / count, 0.5];
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::{PackSettings, UnwrapMesh, unwrap};
    use crate::scene::{Material, Mesh};

    /// A closed box (normals inwards) of `size` metres, one material per
    /// wall: [floor, ceiling, left, right, back, front].
    pub(crate) fn room(size: f32, materials: [usize; 6]) -> Vec<Mesh> {
        let s = size * 0.5;
        // (centre, half_x, half_y): half_x × half_y points into the room.
        let walls = [
            (Vec3::new(0.0, -s, 0.0), Vec3::X * s, Vec3::NEG_Z * s),
            (Vec3::new(0.0, s, 0.0), Vec3::X * s, Vec3::Z * s),
            (Vec3::new(-s, 0.0, 0.0), Vec3::NEG_Z * s, Vec3::Y * s),
            (Vec3::new(s, 0.0, 0.0), Vec3::Z * s, Vec3::Y * s),
            (Vec3::new(0.0, 0.0, -s), Vec3::X * s, Vec3::Y * s),
            (Vec3::new(0.0, 0.0, s), Vec3::NEG_X * s, Vec3::Y * s),
        ];
        walls
            .iter()
            .zip(materials)
            .map(|(&(c, hx, hy), m)| {
                let n = hx.cross(hy).normalize();
                Mesh {
                    positions: vec![c - hx - hy, c + hx - hy, c + hx + hy, c - hx + hy],
                    normals: vec![n; 4],
                    indices: vec![0, 1, 2, 0, 2, 3],
                    material: m,
                    lightmapped: true,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn atlas_of(scene: &Scene) -> (Atlas, Vec<usize>) {
        let picked: Vec<usize> = (0..scene.meshes.len())
            .filter(|&i| scene.meshes[i].lightmapped)
            .collect();
        let unwrap_meshes: Vec<UnwrapMesh> = picked
            .iter()
            .map(|&i| {
                let m = &scene.meshes[i];
                UnwrapMesh {
                    positions: m.positions.iter().map(|p| p.to_array()).collect(),
                    normals: m.normals.iter().map(|p| p.to_array()).collect(),
                    indices: m.indices.clone(),
                }
            })
            .collect();
        let atlas = unwrap(
            &unwrap_meshes,
            &PackSettings {
                texels_per_unit: 8.0,
                ..Default::default()
            },
        )
        .unwrap();
        (atlas, picked)
    }

    /// The mean of a wall's traced texels.
    fn wall_mean(map: &Lightmap, atlas: &Atlas, wall: usize) -> Vec3 {
        let m = &atlas.meshes[wall];
        let mut sum = Vec3::ZERO;
        let mut n = 0.0;
        let (w, _) = (map.width as usize, map.height as usize);
        // Sample at the centroid of each of its triangles.
        for t in m.indices.as_chunks::<3>().0 {
            let c = t.iter().fold(Vec2::ZERO, |a, &i| {
                a + Vec2::from(m.vertices[i as usize].uv)
            }) / 3.0;
            let page = m.vertices[t[0] as usize].page as usize;
            let texel = map.pages[page][c.y as usize * w + c.x as usize];
            sum += Vec3::new(texel[0], texel[1], texel[2]);
            n += 1.0;
        }
        sum / n
    }

    /// A closed white furnace: every wall glows with radiance 1, so every
    /// texel's irradiance over π is 1 (whatever the albedo of a non-emitting
    /// box would bounce, an emitting one returns its emission).
    #[test]
    fn a_furnace_box_bakes_to_its_emission() {
        let scene = Scene {
            meshes: room(2.0, [0; 6]),
            materials: vec![Material {
                base_color: Vec3::ZERO,
                emissive: Vec3::ONE,
                ..Default::default()
            }],
            ..Default::default()
        };
        let (atlas, picked) = atlas_of(&scene);
        let map = bake(
            &scene,
            &atlas,
            &picked,
            &TraceSettings {
                samples: 16,
                ..Default::default()
            },
            &mut |_, _, _| {},
        );
        for wall in 0..6 {
            let m = wall_mean(&map, &atlas, wall);
            assert!(
                (m - Vec3::ONE).abs().max_element() < 1e-3,
                "wall {wall}: {m}"
            );
        }
    }

    /// The Cornell box's point: the white floor next to a red wall and a
    /// green wall, lit from a hole in the ceiling (a rect light), takes red
    /// near the red wall and green near the green one.
    #[test]
    fn coloured_walls_bleed_onto_the_floor() {
        let mats = vec![
            Material {
                base_color: Vec3::splat(0.75),
                ..Default::default()
            },
            Material {
                base_color: Vec3::new(0.7, 0.05, 0.05),
                ..Default::default()
            },
            Material {
                base_color: Vec3::new(0.05, 0.7, 0.05),
                ..Default::default()
            },
        ];
        // floor, ceiling white; left red; right green; back, front white.
        let scene = Scene {
            meshes: room(2.0, [0, 0, 1, 2, 0, 0]),
            materials: mats,
            lights: vec![Light::Rect {
                position: Vec3::new(0.0, 0.99, 0.0),
                // half_x × half_y = +y: it shines down, from its -Z.
                half_x: Vec3::X * 0.3,
                half_y: Vec3::NEG_Z * 0.3,
                radiance: Vec3::splat(20.0),
                two_sided: false,
            }],
            ..Default::default()
        };
        let (atlas, picked) = atlas_of(&scene);
        let map = bake(
            &scene,
            &atlas,
            &picked,
            &TraceSettings {
                samples: 64,
                ..Default::default()
            },
            &mut |_, _, _| {},
        );
        // The floor's texels near x = -0.8 (red side) and x = +0.8 (green side).
        let floor = &atlas.meshes[0];
        let src = &scene.meshes[0];
        let w = map.width as usize;
        let near = |side: f32| {
            let mut sum = Vec3::ZERO;
            let mut n = 0.0;
            for t in floor.indices.as_chunks::<3>().0 {
                for &i in t {
                    let v = floor.vertices[i as usize];
                    let p = src.positions[v.xref as usize];
                    if (p.x - side).abs() < 0.05 {
                        // The texel just inside the chart from this corner.
                        let c = t.iter().fold(Vec2::ZERO, |a, &j| {
                            a + Vec2::from(floor.vertices[j as usize].uv)
                        }) / 3.0;
                        let q = Vec2::from(v.uv) + (c - Vec2::from(v.uv)).normalize() * 2.0;
                        let texel = map.pages[v.page as usize][q.y as usize * w + q.x as usize];
                        sum += Vec3::new(texel[0], texel[1], texel[2]);
                        n += 1.0;
                    }
                }
            }
            sum / n
        };
        let red_side = near(-1.0);
        let green_side = near(1.0);
        assert!(
            red_side.x > red_side.y * 1.3,
            "red bleeds near the red wall: {red_side}"
        );
        assert!(
            green_side.y > green_side.x * 1.3,
            "green near the green wall: {green_side}"
        );
    }

    #[test]
    fn dilation_fills_the_gutter_from_its_neighbours() {
        let mut page = vec![[0.0f32; 4]; 9];
        page[4] = [1.0, 2.0, 3.0, 1.0];
        dilate(&mut page, 3, 3, 1);
        assert!(page.iter().all(|t| t[3] > 0.0));
        assert_eq!(page[0], [1.0, 2.0, 3.0, 0.5]);
        assert_eq!(page[4], [1.0, 2.0, 3.0, 1.0]);
    }

    use crate::scene::Light;
}
