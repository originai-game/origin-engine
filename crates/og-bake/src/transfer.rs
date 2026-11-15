//! Surface transfer: what a detailed surface shows, drawn into the texture
//! pages of a simpler mesh that stands in for it (og-cook's HLOD proxies,
//! docs/design.md §10.6).
//!
//! The simpler mesh (the [`Target`]) is charted already: every vertex has
//! its place on one page, in texels. Each texel its triangles cover is
//! asked what the detailed surface (the [`Source`]) shows there: a ray
//! from `reach` outside the target's surface, along its normal inwards,
//! takes the first source triangle it meets from the front, which is the
//! one a viewer outside sees; where it meets none within `2 × reach`, the
//! nearest point of the source is taken. That point's base colour, roughness, metalness
//! and emission are read from its material (factor × map × vertex colour)
//! and averaged over `samples × samples` points a texel.
//!
//! A source map is read at the level of its box-filtered chain ([`Mips`])
//! whose texels are as large as the step between two of those points, so a
//! brick wall drawn at four texels a metre is its mean colour, not noise.
//!
//! Texels no triangle covers take their covered neighbours' values
//! ([`crate::lightmap::dilate`]), then the pages' mean, so neither
//! filtering nor a mip level reads a value that was never written.
//!
//! One thread, in a fixed order: the same input gives the same pages.

use crate::bvh::Bvh;
use crate::scene::{Image, Mesh};
use glam::{Vec2, Vec3};

/// An image and its box-filtered levels down to one texel.
#[derive(Clone, Debug, PartialEq)]
pub struct Mips {
    levels: Vec<Image>,
}

impl Mips {
    /// The chain under `image`: each level half the one before along each
    /// side (at least one texel), each texel the mean of the up to four
    /// it covers.
    pub fn new(image: Image) -> Mips {
        let mut levels = vec![image];
        loop {
            let last = levels.last().expect("one level");
            if last.width <= 1 && last.height <= 1 || last.texels.is_empty() {
                break;
            }
            let (w, h) = ((last.width / 2).max(1), (last.height / 2).max(1));
            let (sx, sy) = (last.width / w, last.height / h);
            let mut texels = Vec::with_capacity((w * h) as usize);
            for y in 0..h {
                for x in 0..w {
                    let mut sum = [0.0f32; 4];
                    let mut n = 0.0;
                    for dy in 0..sy.min(2) {
                        for dx in 0..sx.min(2) {
                            let (xx, yy) = (x * sx + dx, y * sy + dy);
                            let t = last.texels[(yy * last.width + xx) as usize];
                            for k in 0..4 {
                                sum[k] += t[k];
                            }
                            n += 1.0;
                        }
                    }
                    texels.push(sum.map(|s| s / n));
                }
            }
            levels.push(Image {
                width: w,
                height: h,
                texels,
            });
        }
        Mips { levels }
    }

    /// The longest side of the full image, in texels.
    pub fn side(&self) -> u32 {
        self.levels[0].width.max(self.levels[0].height)
    }

    /// The image at `uv`, `level` levels down (0 the full image; between
    /// two levels, their blend), bilinear and repeating.
    pub fn sample(&self, uv: Vec2, level: f32) -> [f32; 4] {
        let top = (self.levels.len() - 1) as f32;
        let level = if level.is_finite() {
            level.clamp(0.0, top)
        } else {
            0.0
        };
        let lo = level.floor() as usize;
        let f = level - lo as f32;
        let a = self.levels[lo].sample(uv);
        if f <= 0.0 || lo + 1 >= self.levels.len() {
            return a;
        }
        let b = self.levels[lo + 1].sample(uv);
        std::array::from_fn(|k| a[k] + (b[k] - a[k]) * f)
    }
}

/// What a source surface shows: its factors and maps (indices into
/// [`Source::images`]). The maps are glTF's: the base colour linear here
/// (decoded on load), roughness in the green and metalness in the blue
/// channel of the one map, emission linear.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    pub base_color: Vec3,
    pub base_map: Option<usize>,
    pub roughness: f32,
    pub metallic: f32,
    pub metallic_roughness_map: Option<usize>,
    pub emissive: Vec3,
    pub emissive_map: Option<usize>,
}

impl Default for Surface {
    fn default() -> Self {
        Surface {
            base_color: Vec3::ONE,
            base_map: None,
            roughness: 1.0,
            metallic: 0.0,
            metallic_roughness_map: None,
            emissive: Vec3::ZERO,
            emissive_map: None,
        }
    }
}

/// The detailed surface: meshes in the target's space ([`Mesh::material`]
/// an index into `surfaces`; `lightmapped` is not read).
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub meshes: &'a [Mesh],
    pub surfaces: &'a [Surface],
    pub images: &'a [&'a Mips],
}

/// The mesh the pages are for: its vertices' positions, normals and places
/// on the page in texels, and its triangles.
#[derive(Clone, Copy)]
pub struct Target<'a> {
    pub positions: &'a [Vec3],
    pub normals: &'a [Vec3],
    pub uvs: &'a [Vec2],
    pub indices: &'a [u32],
}

/// How the pages are drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferSettings {
    /// The page's size in texels.
    pub width: u32,
    pub height: u32,
    /// How far the source's surface may stand off the target's (metres):
    /// the rays start this far outside it.
    pub reach: f32,
    /// Points asked per texel along each side (1 to 8).
    pub samples: u32,
    /// The length of surface one texel covers (metres): the source maps
    /// are read at the level that matches it.
    pub texel: f32,
}

/// The pages: `width × height` texels each, rows top to bottom. A texel's
/// alpha is 1 where a triangle covers it, 0.5 where it was filled from its
/// neighbours and 0.25 where from the pages' mean.
#[derive(Clone, Debug, PartialEq)]
pub struct Pages {
    pub width: u32,
    pub height: u32,
    /// Linear base colour.
    pub base: Vec<[f32; 4]>,
    /// Roughness in red, metalness in green.
    pub surface: Vec<[f32; 4]>,
    /// Linear emitted radiance.
    pub emissive: Vec<[f32; 4]>,
    /// How many texels a triangle covers.
    pub covered: usize,
}

/// What one point of the source shows.
#[derive(Clone, Copy, Default)]
struct Shown {
    base: Vec3,
    roughness: f32,
    metallic: f32,
    emissive: Vec3,
}

struct Reader<'a> {
    source: Source<'a>,
    bvh: Bvh,
    /// Per triangle of `bvh`: its mesh and its first index there.
    owner: Vec<(u32, u32)>,
    reach: f32,
    /// The step between two asked points, in metres.
    step: f32,
}

impl Reader<'_> {
    fn new<'a>(source: Source<'a>, reach: f32, step: f32) -> Reader<'a> {
        let mut corners = Vec::new();
        let mut owner = Vec::new();
        for (mi, m) in source.meshes.iter().enumerate() {
            for (ti, t) in m.indices.as_chunks::<3>().0.iter().enumerate() {
                let c = t.map(|i| m.positions.get(i as usize).copied().unwrap_or(Vec3::NAN));
                corners.push(c);
                owner.push((mi as u32, ti as u32 * 3));
            }
        }
        Reader {
            source,
            bvh: Bvh::new(corners),
            owner,
            reach,
            step,
        }
    }

    /// What the source shows at `p` of the target, whose normal is `n`.
    fn at(&self, p: Vec3, n: Vec3) -> Option<Shown> {
        // A surface met from behind is not the one a viewer outside sees
        // (the far wall of the house across a lane narrower than `reach`,
        // where the ray starts): the ray goes on past it.
        let mut origin = p + n * self.reach;
        let mut left = self.reach * 2.0;
        for _ in 0..8 {
            match self.bvh.closest(origin, -n, left) {
                Some(h) if h.back => {
                    let step = h.t + self.reach * 1e-4;
                    origin -= n * step;
                    left -= step;
                    if left <= 0.0 {
                        break;
                    }
                }
                Some(h) => return Some(self.shown(h.triangle, h.u, h.v)),
                None => break,
            }
        }
        let h = self.bvh.nearest(p, f32::INFINITY)?;
        Some(self.shown(h.triangle, h.u, h.v))
    }

    fn shown(&self, triangle: u32, u: f32, v: f32) -> Shown {
        let (mi, first) = self.owner[triangle as usize];
        let mesh = &self.source.meshes[mi as usize];
        let idx: [usize; 3] = std::array::from_fn(|k| mesh.indices[first as usize + k] as usize);
        let w = [1.0 - u - v, u, v];
        let surface = self
            .source
            .surfaces
            .get(mesh.material)
            .cloned()
            .unwrap_or_default();
        let uvs = idx.map(|i| mesh.uvs.get(i).copied().unwrap_or(Vec2::ZERO));
        let uv = uvs[0] * w[0] + uvs[1] * w[1] + uvs[2] * w[2];
        let color = if mesh.colors.is_empty() {
            Vec3::ONE
        } else {
            let c = idx.map(|i| mesh.colors.get(i).copied().unwrap_or(Vec3::ONE));
            c[0] * w[0] + c[1] * w[1] + c[2] * w[2]
        };
        // How much of the map a metre of this triangle crosses.
        let c = self.bvh.corners(triangle);
        let world = (c[1] - c[0]).cross(c[2] - c[0]).length();
        let mapped = (uvs[1] - uvs[0]).perp_dot(uvs[2] - uvs[0]).abs();
        let rate = if world > 1e-20 {
            (mapped / world).sqrt()
        } else {
            0.0
        };
        let read = |map: Option<usize>| -> Option<[f32; 4]> {
            let image = self.source.images.get(map?)?;
            let level = (rate * image.side() as f32 * self.step).max(1.0).log2();
            Some(image.sample(uv, level))
        };
        let mut out = Shown {
            base: surface.base_color * color,
            roughness: surface.roughness,
            metallic: surface.metallic,
            emissive: surface.emissive,
        };
        if let Some(t) = read(surface.base_map) {
            out.base *= Vec3::new(t[0], t[1], t[2]);
        }
        if let Some(t) = read(surface.metallic_roughness_map) {
            out.roughness *= t[1];
            out.metallic *= t[2];
        }
        if out.emissive != Vec3::ZERO
            && let Some(t) = read(surface.emissive_map)
        {
            out.emissive *= Vec3::new(t[0], t[1], t[2]);
        }
        out
    }
}

/// Draws the pages of `target` from `source`.
pub fn transfer(source: Source, target: Target, settings: &TransferSettings) -> Pages {
    let (w, h) = (
        settings.width.max(1) as usize,
        settings.height.max(1) as usize,
    );
    let samples = settings.samples.clamp(1, 8) as usize;
    let reader = Reader::new(
        source,
        settings.reach.max(1e-4),
        settings.texel.max(1e-6) / samples as f32,
    );
    // Per texel: the sum of what its points showed, and how many.
    let mut sums = vec![Shown::default(); w * h];
    let mut counts = vec![0u32; w * h];
    fn add(sums: &mut [Shown], counts: &mut [u32], i: usize, s: Shown) {
        let t = &mut sums[i];
        t.base += s.base;
        t.roughness += s.roughness;
        t.metallic += s.metallic;
        t.emissive += s.emissive;
        counts[i] += 1;
    }
    struct Tri {
        uv: [Vec2; 3],
        p: [Vec3; 3],
        n: [Vec3; 3],
        face: Vec3,
        area: f32,
    }
    let tris: Vec<Tri> = target
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|t| {
            let i = t.map(|i| i as usize);
            let uv = i.map(|i| target.uvs.get(i).copied().unwrap_or(Vec2::ZERO));
            let p = i.map(|i| target.positions.get(i).copied().unwrap_or(Vec3::ZERO));
            let face = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or(Vec3::ZERO);
            let area = (uv[1] - uv[0]).perp_dot(uv[2] - uv[0]);
            (face != Vec3::ZERO && area.abs() > 1e-12).then(|| Tri {
                uv,
                p,
                n: i.map(|i| target.normals.get(i).copied().unwrap_or(face)),
                face,
                area,
            })
        })
        .collect();
    let bary = |t: &Tri, q: Vec2| {
        let b1 = (q - t.uv[0]).perp_dot(t.uv[2] - t.uv[0]) / t.area;
        let b2 = (t.uv[1] - t.uv[0]).perp_dot(q - t.uv[0]) / t.area;
        Vec3::new(1.0 - b1 - b2, b1, b2)
    };
    let point = |t: &Tri, b: Vec3| {
        let p = t.p[0] * b.x + t.p[1] * b.y + t.p[2] * b.z;
        let mut n = (t.n[0] * b.x + t.n[1] * b.y + t.n[2] * b.z).normalize_or(t.face);
        if n.dot(t.face) < 0.0 {
            n = t.face;
        }
        (p, n)
    };
    let bounds = |t: &Tri, margin: f32| {
        let min = t.uv[0].min(t.uv[1]).min(t.uv[2]);
        let max = t.uv[0].max(t.uv[1]).max(t.uv[2]);
        (
            (min.x - margin).floor().max(0.0) as usize,
            (min.y - margin).floor().max(0.0) as usize,
            ((max.x + margin).ceil().max(0.0) as usize).min(w - 1),
            ((max.y + margin).ceil().max(0.0) as usize).min(h - 1),
        )
    };
    // The points inside each triangle.
    for t in &tris {
        let (x0, y0, x1, y1) = bounds(t, 0.0);
        for y in y0..=y1 {
            for x in x0..=x1 {
                for sy in 0..samples {
                    for sx in 0..samples {
                        let q = Vec2::new(
                            x as f32 + (sx as f32 + 0.5) / samples as f32,
                            y as f32 + (sy as f32 + 0.5) / samples as f32,
                        );
                        let b = bary(t, q);
                        if b.min_element() < -1e-4 {
                            continue;
                        }
                        let (p, n) = point(t, b);
                        if let Some(s) = reader.at(p, n) {
                            add(&mut sums, &mut counts, y * w + x, s);
                        }
                    }
                }
            }
        }
    }
    // A texel a triangle only grazes (none of its points inside): the
    // triangle's nearest point, when within three quarters of a texel.
    for t in &tris {
        let (x0, y0, x1, y1) = bounds(t, 1.0);
        for y in y0..=y1 {
            for x in x0..=x1 {
                if counts[y * w + x] > 0 {
                    continue;
                }
                let c = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let q = nearest_2d(c, t.uv);
                if (q - c).length() > 0.75 {
                    continue;
                }
                let centre = (t.uv[0] + t.uv[1] + t.uv[2]) / 3.0;
                let (p, n) = point(t, bary(t, q + (centre - q) * 0.01));
                if let Some(s) = reader.at(p, n) {
                    add(&mut sums, &mut counts, y * w + x, s);
                }
            }
        }
    }
    let mut pages = Pages {
        width: w as u32,
        height: h as u32,
        base: vec![[0.0; 4]; w * h],
        surface: vec![[0.0; 4]; w * h],
        emissive: vec![[0.0; 4]; w * h],
        covered: 0,
    };
    let mut mean = Shown::default();
    for (i, (s, &n)) in sums.iter().zip(&counts).enumerate() {
        if n == 0 {
            continue;
        }
        let k = 1.0 / n as f32;
        let (b, e) = (s.base * k, s.emissive * k);
        pages.base[i] = [b.x, b.y, b.z, 1.0];
        pages.surface[i] = [s.roughness * k, s.metallic * k, 0.0, 1.0];
        pages.emissive[i] = [e.x, e.y, e.z, 1.0];
        pages.covered += 1;
        mean.base += b;
        mean.roughness += s.roughness * k;
        mean.metallic += s.metallic * k;
        mean.emissive += e;
    }
    let k = 1.0 / pages.covered.max(1) as f32;
    let fill = [
        [mean.base.x * k, mean.base.y * k, mean.base.z * k, 0.25],
        [mean.roughness * k, mean.metallic * k, 0.0, 0.25],
        [
            mean.emissive.x * k,
            mean.emissive.y * k,
            mean.emissive.z * k,
            0.25,
        ],
    ];
    for (page, fill) in [&mut pages.base, &mut pages.surface, &mut pages.emissive]
        .into_iter()
        .zip(fill)
    {
        crate::lightmap::dilate(page, w, h, 8);
        for t in page.iter_mut().filter(|t| t[3] <= 0.0) {
            *t = fill;
        }
    }
    pages
}

/// The point of a triangle of the page nearest `p`, on its border.
fn nearest_2d(p: Vec2, t: [Vec2; 3]) -> Vec2 {
    let seg = |a: Vec2, b: Vec2| {
        let ab = b - a;
        let s = ((p - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        a + ab * s
    };
    let mut best = seg(t[0], t[1]);
    for q in [seg(t[1], t[2]), seg(t[2], t[0])] {
        if (q - p).length_squared() < (best - p).length_squared() {
            best = q;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(z: f32, size: f32, material: usize) -> Mesh {
        Mesh {
            positions: vec![
                Vec3::new(0.0, 0.0, z),
                Vec3::new(size, 0.0, z),
                Vec3::new(size, size, z),
                Vec3::new(0.0, size, z),
            ],
            normals: vec![Vec3::Z; 4],
            uvs: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
            colors: Vec::new(),
            indices: vec![0, 1, 2, 0, 2, 3],
            material,
            lightmapped: false,
        }
    }

    fn settings(side: u32) -> TransferSettings {
        TransferSettings {
            width: side,
            height: side,
            reach: 0.5,
            samples: 2,
            texel: 4.0 / side as f32,
        }
    }

    /// The target's page over a 4 m quad at z 0, `side` texels across.
    fn page(side: u32) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec2>, Vec<u32>) {
        let m = quad(0.0, 4.0, 0);
        let uvs = m.uvs.iter().map(|u| *u * side as f32).collect();
        (m.positions, m.normals, uvs, m.indices)
    }

    #[test]
    fn a_texel_takes_the_surface_seen_from_outside_not_the_one_behind_it() {
        // A red wall 0.1 m in front of the target and a green one 0.1 m
        // behind: both as near, and the red is what a viewer sees.
        let meshes = [quad(0.1, 4.0, 0), quad(-0.1, 4.0, 1)];
        let surfaces = [
            Surface {
                base_color: Vec3::X,
                roughness: 0.25,
                ..Default::default()
            },
            Surface {
                base_color: Vec3::Y,
                metallic: 1.0,
                ..Default::default()
            },
        ];
        let (p, n, uv, i) = page(8);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &settings(8),
        );
        assert_eq!(pages.covered, 64);
        for (b, s) in pages.base.iter().zip(&pages.surface) {
            assert_eq!(*b, [1.0, 0.0, 0.0, 1.0]);
            assert_eq!([s[0], s[1]], [0.25, 0.0]);
        }
    }

    #[test]
    fn a_surface_out_of_the_rays_reach_is_still_found_as_the_nearest() {
        let meshes = [quad(-3.0, 4.0, 0)];
        let surfaces = [Surface {
            base_color: Vec3::new(0.2, 0.4, 0.6),
            emissive: Vec3::new(2.0, 0.0, 0.0),
            ..Default::default()
        }];
        let (p, n, uv, i) = page(4);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &settings(4),
        );
        let near = |t: [f32; 4], want: [f32; 3]| (0..3).all(|k| (t[k] - want[k]).abs() < 1e-5);
        assert!(near(pages.base[5], [0.2, 0.4, 0.6]), "{:?}", pages.base[5]);
        assert!(near(pages.emissive[5], [2.0, 0.0, 0.0]));
    }

    /// A checker of one-texel black and white squares, `side` across.
    fn checker(side: u32) -> Mips {
        Mips::new(Image {
            width: side,
            height: side,
            texels: (0..side * side)
                .map(|i| {
                    let v = ((i % side + i / side) % 2) as f32;
                    [v, v, v, 1.0]
                })
                .collect(),
        })
    }

    #[test]
    fn a_map_finer_than_the_page_is_read_as_its_mean_and_a_coarser_one_as_it_is() {
        // 64 checker squares across 4 m on a page of 4 texels: every texel
        // holds 256 squares and reads their mean, mid grey. Read at the
        // full image, 2 × 2 points a texel would land on squares of one
        // colour or the other.
        let fine = checker(64);
        let meshes = [quad(0.0, 4.0, 0)];
        let surfaces = [Surface {
            base_map: Some(0),
            ..Default::default()
        }];
        let (p, n, uv, i) = page(4);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[&fine],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &settings(4),
        );
        for t in &pages.base {
            assert!((t[0] - 0.5).abs() < 0.02, "{t:?}");
        }
        // 4 squares across a page of 16 texels: the image is coarser than
        // the page and is read as it is, so its black and its white both
        // reach the page (bilinear between them at the squares' edges).
        let coarse = checker(4);
        let (p, n, uv, i) = page(16);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[&coarse],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &settings(16),
        );
        let (lo, hi) = pages
            .base
            .iter()
            .fold((1.0f32, 0.0f32), |(lo, hi), t| (lo.min(t[0]), hi.max(t[0])));
        assert!(lo < 0.25 && hi > 0.75, "{lo} to {hi}");
    }

    #[test]
    fn texels_no_triangle_covers_take_their_neighbours_then_the_mean() {
        // The target's page is 32 texels, its one quad on the first 4.
        let meshes = [quad(0.0, 4.0, 0)];
        let surfaces = [Surface {
            base_color: Vec3::new(0.5, 0.25, 0.125),
            ..Default::default()
        }];
        let (p, n, uv, i) = page(4);
        let mut s = settings(4);
        (s.width, s.height) = (32, 32);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &s,
        );
        // The 16 texels inside it and the 9 its border comes within three
        // quarters of a texel of.
        assert_eq!(pages.covered, 25);
        let alphas: std::collections::BTreeSet<u32> =
            pages.base.iter().map(|t| (t[3] * 4.0) as u32).collect();
        assert_eq!(alphas.into_iter().collect::<Vec<_>>(), [1, 2, 4]);
        for t in &pages.base {
            assert!((Vec3::new(t[0], t[1], t[2]) - Vec3::new(0.5, 0.25, 0.125)).length() < 1e-5);
        }
    }

    #[test]
    fn a_wall_met_from_behind_is_passed_for_the_one_facing_out() {
        // The target's wall is green and faces +z. A blue wall 0.3 m in
        // front of it faces it (a house across a lane narrower than the
        // reach): the ray starts behind the blue wall and meets its back
        // first.
        let mut across = quad(0.3, 4.0, 0);
        across.indices = vec![0, 2, 1, 0, 3, 2];
        let meshes = [across, quad(0.0, 4.0, 1)];
        let surfaces = [
            Surface {
                base_color: Vec3::Z,
                ..Default::default()
            },
            Surface {
                base_color: Vec3::Y,
                ..Default::default()
            },
        ];
        let (p, n, uv, i) = page(4);
        let pages = transfer(
            Source {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &[],
            },
            Target {
                positions: &p,
                normals: &n,
                uvs: &uv,
                indices: &i,
            },
            &settings(4),
        );
        for t in &pages.base {
            assert_eq!(*t, [0.0, 1.0, 0.0, 1.0]);
        }
    }
}
