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
//! Texels no triangle covers copy the nearest covered texel outward,
//! with the same donor for every map. No unused page area is filled with
//! an unrelated mean colour or neutral normal.
//!
//! One thread, in a fixed order: the same input gives the same pages.

use crate::bvh::Bvh;
use crate::scene::{Image, Mesh};
use glam::{Mat3, Vec2, Vec3};

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
    /// Tangent-space normal and glTF's normal scale.
    pub normal_map: Option<usize>,
    pub normal_scale: f32,
    /// Occlusion is the red channel, mixed with white by its strength.
    pub occlusion_map: Option<usize>,
    pub occlusion_strength: f32,
    /// Base colour factor's alpha.
    pub opacity: f32,
    /// Affine UV transforms: base, metallic/roughness, normal, emissive,
    /// occlusion. Each map keeps its own KHR_texture_transform.
    pub uv_transforms: [Mat3; 5],
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
            normal_map: None,
            normal_scale: 1.0,
            occlusion_map: None,
            occlusion_strength: 1.0,
            opacity: 1.0,
            uv_transforms: [Mat3::IDENTITY; 5],
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
    /// MikkTSpace tangents for the new UVs (empty derives face frames).
    pub tangents: &'a [glam::Vec4],
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
/// alpha is 1 where a triangle covers it, 0.5 within eight dilation steps
/// and 0.25 farther into the unused page area.
#[derive(Clone, Debug, PartialEq)]
pub struct Pages {
    pub width: u32,
    pub height: u32,
    /// Linear base colour.
    pub base: Vec<[f32; 4]>,
    /// Roughness in red, metalness in green, occlusion in blue.
    pub surface: Vec<[f32; 4]>,
    /// Linear emitted radiance.
    pub emissive: Vec<[f32; 4]>,
    /// Tangent-space normal, encoded as 0..1 RGB.
    pub normal: Vec<[f32; 4]>,
    /// Opacity in red. Kept separately because page alpha records coverage,
    /// including dilation, rather than material transparency.
    pub opacity: Vec<[f32; 4]>,
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
    normal: Vec3,
    occlusion: f32,
    opacity: f32,
}

/// A triangle's UV frame, orthonormal at the shading normal. Mirrored UVs
/// keep the bitangent sign; degenerate UVs use a geometric frame.
fn frame(p: [Vec3; 3], uv: [Vec2; 3], n: Vec3, tangent: glam::Vec4) -> Mat3 {
    if tangent.truncate().length_squared() > 1e-20 {
        let t = tangent.truncate();
        let t = (t - n * n.dot(t)).normalize_or(n.any_orthonormal_vector());
        return Mat3::from_cols(t, n.cross(t) * tangent.w.signum(), n);
    }
    // The engine's MikkTSpace convention has its bitangent up the image.
    let uv = uv.map(|u| Vec2::new(u.x, -u.y));
    let (e1, e2) = (p[1] - p[0], p[2] - p[0]);
    let (u1, u2) = (uv[1] - uv[0], uv[2] - uv[0]);
    let det = u1.perp_dot(u2);
    let (t, b) = if det.abs() > 1e-20 {
        ((e1 * u2.y - e2 * u1.y) / det, (e2 * u1.x - e1 * u2.x) / det)
    } else {
        let t = n.any_orthonormal_vector();
        (t, n.cross(t))
    };
    let t = (t - n * n.dot(t)).normalize_or(n.any_orthonormal_vector());
    Mat3::from_cols(t, n.cross(t) * n.cross(t).dot(b).signum(), n)
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
        let read = |map: Option<usize>, slot: usize| -> Option<[f32; 4]> {
            let image = self.source.images.get(map?)?;
            let transform = surface.uv_transforms[slot];
            let rate = rate * transform.determinant().abs().sqrt();
            let level = (rate * image.side() as f32 * self.step).max(1.0).log2();
            Some(image.sample((transform * uv.extend(1.0)).truncate(), level))
        };
        let face = (c[1] - c[0]).cross(c[2] - c[0]).normalize_or(Vec3::Z);
        let normal = idx
            .iter()
            .zip(w)
            .map(|(&i, w)| mesh.normals.get(i).copied().unwrap_or(face) * w)
            .sum::<Vec3>()
            .normalize_or(face);
        let mut out = Shown {
            base: surface.base_color * color,
            roughness: surface.roughness,
            metallic: surface.metallic,
            emissive: surface.emissive,
            normal,
            occlusion: 1.0,
            opacity: surface.opacity
                * idx
                    .iter()
                    .zip(w)
                    .map(|(&i, w)| mesh.opacity.get(i).copied().unwrap_or(1.0) * w)
                    .sum::<f32>(),
        };
        if let Some(t) = read(surface.base_map, 0) {
            out.base *= Vec3::new(t[0], t[1], t[2]);
            out.opacity *= t[3];
        }
        if let Some(t) = read(surface.metallic_roughness_map, 1) {
            out.roughness *= t[1];
            out.metallic *= t[2];
        }
        if let Some(t) = read(surface.normal_map, 2) {
            let mapped = Vec3::new(
                (t[0] * 2.0 - 1.0) * surface.normal_scale,
                (t[1] * 2.0 - 1.0) * surface.normal_scale,
                t[2] * 2.0 - 1.0,
            );
            let tangent = idx
                .iter()
                .zip(w)
                .map(|(&i, w)| mesh.tangents.get(i).copied().unwrap_or(glam::Vec4::ZERO) * w)
                .sum();
            out.normal = (frame(*c, uvs, normal, tangent) * mapped).normalize_or(normal);
        }
        if let Some(t) = read(surface.occlusion_map, 4) {
            out.occlusion = 1.0 + surface.occlusion_strength * (t[0] - 1.0);
        }
        if out.emissive != Vec3::ZERO
            && let Some(t) = read(surface.emissive_map, 3)
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
        t.normal += s.normal;
        t.occlusion += s.occlusion;
        t.opacity += s.opacity;
        counts[i] += 1;
    }
    struct Tri {
        uv: [Vec2; 3],
        p: [Vec3; 3],
        n: [Vec3; 3],
        t: [glam::Vec4; 3],
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
                t: i.map(|i| target.tangents.get(i).copied().unwrap_or(glam::Vec4::ZERO)),
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
        (p, n, t.t[0] * b.x + t.t[1] * b.y + t.t[2] * b.z)
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
                        let (p, n, tangent) = point(t, b);
                        if let Some(mut s) = reader.at(p, n) {
                            s.normal = frame(t.p, t.uv, n, tangent).transpose() * s.normal;
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
                let (p, n, tangent) = point(t, bary(t, q + (centre - q) * 0.01));
                if let Some(mut s) = reader.at(p, n) {
                    s.normal = frame(t.p, t.uv, n, tangent).transpose() * s.normal;
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
        normal: vec![[0.0; 4]; w * h],
        opacity: vec![[0.0; 4]; w * h],
        covered: 0,
    };
    for (i, (s, &n)) in sums.iter().zip(&counts).enumerate() {
        if n == 0 {
            continue;
        }
        let k = 1.0 / n as f32;
        let (b, e) = (s.base * k, s.emissive * k);
        pages.base[i] = [b.x, b.y, b.z, 1.0];
        pages.surface[i] = [s.roughness * k, s.metallic * k, s.occlusion * k, 1.0];
        pages.emissive[i] = [e.x, e.y, e.z, 1.0];
        let normal = s.normal.normalize_or(Vec3::Z) * 0.5 + Vec3::splat(0.5);
        pages.normal[i] = [normal.x, normal.y, normal.z, 1.0];
        pages.opacity[i] = [s.opacity * k, 0.0, 0.0, 1.0];
        pages.covered += 1;
    }
    // One multi-source flood for every map: no additional pack space and
    // linear work even when most of the page is unused. Stable row-major
    // seeds resolve ties identically on every host.
    let mut distance = vec![usize::MAX; w * h];
    let mut queue = std::collections::VecDeque::new();
    for (i, &n) in counts.iter().enumerate() {
        if n > 0 {
            distance[i] = 0;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % w, i / w);
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (xx, yy) = (x as isize + dx, y as isize + dy);
            if xx < 0 || yy < 0 || xx >= w as isize || yy >= h as isize {
                continue;
            }
            let j = yy as usize * w + xx as usize;
            if distance[j] != usize::MAX {
                continue;
            }
            distance[j] = distance[i] + 1;
            for page in [
                &mut pages.base,
                &mut pages.surface,
                &mut pages.emissive,
                &mut pages.normal,
                &mut pages.opacity,
            ] {
                page[j] = page[i];
                page[j][3] = if distance[j] <= 8 { 0.5 } else { 0.25 };
            }
            queue.push_back(j);
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
