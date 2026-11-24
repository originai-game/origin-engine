//! Octahedral impostors (`model { impostor true }`, og-render's
//! `impostor.rs`): the model's default scene rendered on the CPU from
//! `grid × grid` directions over the upper hemisphere (hemi-octahedral;
//! with `impostorBelow true` the lower one's too, in the atlases' right
//! half: [`ImpostorBake::below`]),
//! each frame a square of two atlases: the albedo with its coverage in alpha (sRGB), and the
//! normal in the frame's basis with the depth towards the viewer (linear,
//! `n · 0.5 + 0.5`, `0.5 + 0.5 × depth / radius`).
//!
//! A third atlas (G5, linear) holds each texel's roughness (red),
//! translucency (green) and metallic (blue), so foliage cards are lit
//! through from behind and keep their materials' gloss; its alpha is how
//! much of the sun reaches the texel past the model's faces near it, over
//! a spread of sun directions ([`near_shade`]). A mesh's leaves shade each
//! other at a scale the card's own shadow (its depth atlas) cannot
//! resolve; without that share the card stands lighter than its mesh.
//!
//! Each frame is drawn at four times its size along each side (16 samples
//! a texel) with a depth buffer and box filtered down, so its silhouette
//! and thin leaves are antialiased; texels nothing covers then take their
//! nearest covered neighbour's colour, normal and material (the albedo's
//! alpha stays 0), so neither the atlas's filtering nor its mips darken
//! the edges. The normal is the texel's mean, not normalised: its length
//! says how far its faces agree. The
//! albedo is the material's base colour factor times its base colour
//! texture times the vertex colour: lighting is the renderer's. Textures
//! are read bilinearly from the mip level of the triangle's texel
//! footprint on the frame's fine pixels (trilinear between two levels),
//! so a dense leaf texture shrunk into a frame does not alias.
//!
//! A model with translucent materials (foliage: glTF
//! `KHR_materials_diffuse_transmission` or `KHR_materials_transmission`,
//! or double-sided alpha-masked leaf cards) takes 12 × 12 frames unless
//! `impostorGrid` says otherwise; others 8 × 8 (Brucks, *Octahedral
//! Impostors*, 2018: more frames where the silhouette changes fastest).

use super::GltfSource;
use crate::cook::texture::Image;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use og_render_data::impostor::{
    CardTrim, blend_frames, frame_basis, frame_direction, hemi_oct_decode, march,
};

/// How a model's impostor is baked: frames per side (0: by the model,
/// [`ImpostorBake::resolved`]), pixels per frame, and whether the lower
/// hemisphere is baked too (`impostorBelow`: the atlases twice as wide,
/// for a model seen from below its horizon).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImpostorBake {
    pub grid: u32,
    pub frame: u32,
    pub below: bool,
}

impl Default for ImpostorBake {
    fn default() -> Self {
        ImpostorBake {
            grid: 0,
            frame: 128,
            below: false,
        }
    }
}

impl ImpostorBake {
    /// Frames across the atlases: the lower hemisphere's stand right of
    /// the upper one's.
    pub fn columns(self) -> u32 {
        self.grid * if self.below { 2 } else { 1 }
    }
}

/// Frames per side for a model without and with translucent materials,
/// where `impostorGrid` does not say.
const GRID: u32 = 8;
const FOLIAGE_GRID: u32 = 12;
/// Samples per texel along each side of a frame.
const SUPERSAMPLE: u32 = 4;

impl ImpostorBake {
    /// The bake with its grid decided: as asked, else 12 frames a side for
    /// foliage (a translucent material, [`translucency`]) and 8 for
    /// anything else.
    pub fn resolved(self, src: &GltfSource) -> ImpostorBake {
        if self.grid != 0 {
            return self;
        }
        let foliage = src.doc.materials().any(|m| translucency(&m) > 0.0);
        ImpostorBake {
            grid: if foliage { FOLIAGE_GRID } else { GRID },
            ..self
        }
    }
}

/// How much light a material lets through to its far side (0..1): glTF
/// `KHR_materials_diffuse_transmission`'s factor, else half of
/// `KHR_materials_transmission`'s, else 0.5 for a double-sided alpha-
/// masked material (a leaf card), else 0.
pub fn translucency(m: &gltf::Material) -> f32 {
    if let Some(f) = diffuse_transmission(m) {
        return f;
    }
    if let Some(f) = m
        .extension_value("KHR_materials_transmission")
        .map(|v| v.get("transmissionFactor").and_then(|f| f.as_f64()))
    {
        return (0.5 * f.unwrap_or(0.0) as f32).clamp(0.0, 1.0);
    }
    if m.alpha_mode() == gltf::material::AlphaMode::Mask && m.double_sided() {
        return 0.5;
    }
    0.0
}

/// glTF `KHR_materials_diffuse_transmission`'s factor (0..1) where the
/// material states one: a thin leaf's translucency, which the cooked
/// material draws with.
pub fn diffuse_transmission(m: &gltf::Material) -> Option<f32> {
    let v = m.extension_value("KHR_materials_diffuse_transmission")?;
    let f = v.get("diffuseTransmissionFactor").and_then(|f| f.as_f64());
    Some((f.unwrap_or(0.0) as f32).clamp(0.0, 1.0))
}

/// How far a material's normals turn to its canopy's dome (0..1): its
/// glTF `extras` `{ "ogFoliage": { "domeNormals": 0.7 } }`, else 0.
pub fn dome_normals(m: &gltf::Material) -> f32 {
    m.extras()
        .as_ref()
        .and_then(|e| serde_json::from_str::<serde_json::Value>(e.get()).ok())
        .and_then(|v| v["ogFoliage"]["domeNormals"].as_f64())
        .map_or(0.0, |d| (d as f32).clamp(0.0, 1.0))
}

/// A baked impostor: the three atlases and the sphere they cover.
pub struct Baked {
    pub albedo: Image,
    pub normal: Image,
    /// Roughness, translucency and metallic (linear), coverage in alpha.
    pub material: Image,
    pub center: Vec3,
    pub radius: f32,
    /// The card's outline: where any view's frames can draw ([`card_trim`]).
    pub trim: CardTrim,
}

/// A texture decoded once, linear, with its box-filtered mips.
struct Mips {
    levels: Vec<(u32, u32, Vec<Vec4>)>,
}

impl Mips {
    /// `srgb`: the colour channels are sRGB-encoded (a base colour).
    fn new(img: &Image, srgb: bool) -> Mips {
        let decode = |v: u8, colour: bool| {
            let x = f32::from(v) / 255.0;
            if colour && srgb { srgb_to_linear(x) } else { x }
        };
        let base: Vec<Vec4> = img
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| {
                Vec4::new(
                    decode(p[0], true),
                    decode(p[1], true),
                    decode(p[2], true),
                    decode(p[3], false),
                )
            })
            .collect();
        let mut levels = vec![(img.width.max(1), img.height.max(1), base)];
        while let Some((w, h, px)) = levels.last().filter(|l| l.0 > 1 || l.1 > 1) {
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let at = |x: u32, y: u32| px[(y.min(h - 1) * w + x.min(w - 1)) as usize];
            let next = (0..nh)
                .flat_map(|y| (0..nw).map(move |x| (x, y)))
                .map(|(x, y)| {
                    (at(2 * x, 2 * y)
                        + at(2 * x + 1, 2 * y)
                        + at(2 * x, 2 * y + 1)
                        + at(2 * x + 1, 2 * y + 1))
                        * 0.25
                })
                .collect();
            levels.push((nw, nh, next));
        }
        Mips { levels }
    }

    fn bilinear(&self, level: usize, uv: Vec2) -> Vec4 {
        let (w, h, px) = &self.levels[level.min(self.levels.len() - 1)];
        let (w, h) = (*w as i64, *h as i64);
        let p = Vec2::new(uv.x * w as f32 - 0.5, uv.y * h as f32 - 0.5);
        let i = p.floor();
        let f = p - i;
        let at = |dx: i64, dy: i64| {
            let x = (i.x as i64 + dx).rem_euclid(w);
            let y = (i.y as i64 + dy).rem_euclid(h);
            px[(y * w + x) as usize]
        };
        let top = at(0, 0).lerp(at(1, 0), f.x);
        let bottom = at(0, 1).lerp(at(1, 1), f.x);
        top.lerp(bottom, f.y)
    }

    /// Trilinear at level of detail `lod` (texels per fine pixel, log2).
    fn sample(&self, uv: Vec2, lod: f32) -> Vec4 {
        let lod = lod.clamp(0.0, (self.levels.len() - 1) as f32);
        let lo = lod.floor() as usize;
        self.bilinear(lo, uv)
            .lerp(self.bilinear(lo + 1, uv), lod - lo as f32)
    }

    /// The level of detail for a triangle covering `uv_area` of the
    /// texture (in uv units) over `px_area` fine pixels.
    fn lod(&self, uv_area: f32, px_area: f32) -> f32 {
        let (w, h, _) = &self.levels[0];
        let texels = uv_area.abs() * (*w as f32) * (*h as f32);
        0.5 * (texels / px_area.abs().max(1e-12)).max(1e-12).log2()
    }
}

struct Tri {
    p: [Vec3; 3],
    n: [Vec3; 3],
    uv: [Vec2; 3],
    color: [Vec4; 3],
    /// The material's base colour factor and its texture (an image index).
    base: Vec4,
    texture: Option<usize>,
    /// Its roughness and metallic factors and their texture (green and
    /// blue), and its translucency.
    roughness: f32,
    metallic: f32,
    mr_texture: Option<usize>,
    translucency: f32,
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// The model's canopy centre in its own units, as the cooked model has it
/// (og-bundle's `CookedModel::mesh_foliage`): the centre of the box of its
/// leaf cards' anchors, of every anchor without leaf cards.
fn canopy_center(src: &GltfSource) -> Vec3 {
    let foliage: Vec<(bool, Vec<[f32; 8]>)> = src
        .doc
        .meshes()
        .flat_map(|m| m.primitives())
        .filter_map(|p| {
            let count = p.get(&gltf::Semantic::Positions)?.count();
            let f = super::primitive::read_foliage(src, &p, count).ok()??;
            Some((super::primitive::leaf_cards(&p), f))
        })
        .collect();
    let any_leaves = foliage.iter().any(|f| f.0);
    let (lo, hi) = foliage
        .iter()
        .filter(|f| f.0 || !any_leaves)
        .flat_map(|f| f.1.iter().map(|v| Vec3::new(v[4], v[5], v[6])))
        .fold(
            (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
            |(lo, hi), a| (lo.min(a), hi.max(a)),
        );
    (lo + hi) * 0.5
}

/// og-render's `foliage_normal` (foliage.wgsl): a leaf's normal `n`
/// turned by `strength` to its canopy's dome, out from `center` through
/// its `anchor` and up.
fn dome_normal(n: Vec3, anchor: Vec3, center: Vec3, strength: f32) -> Vec3 {
    if strength <= 0.0 {
        return n;
    }
    let dome = (anchor - center)
        .try_normalize()
        .map_or(Vec3::Y, |r| (r + Vec3::new(0.0, 0.45, 0.0)).normalize());
    n.lerp(dome, strength.clamp(0.0, 1.0)).normalize_or(dome)
}

/// The default scene's triangles in the model's space, as the rest pose
/// draws them (hidden nodes and their descendants left out, an instanced
/// node at each of its instances).
fn triangles(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
) -> Vec<Tri> {
    let mut out = Vec::new();
    let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) else {
        return out;
    };
    let canopy = canopy_center(src);
    // A skinned mesh is drawn at its bind pose (model.rs `skin_binds`).
    let binds = super::skin_binds(src);
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        if node
            .extension_value("KHR_node_visibility")
            .and_then(|v| v.get("visible")?.as_bool())
            == Some(false)
        {
            continue;
        }
        let (t, r, s) = node.transform().decomposed();
        let world = parent
            * Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
        let at: Vec<Mat4> = match (binds.get(&node.index()), instanced.get(&node.index())) {
            (Some(b), _) => vec![*b],
            (None, Some(i)) => i.iter().map(|m| world * *m).collect(),
            (None, None) => vec![world],
        };
        if let Some(mesh) = node.mesh() {
            for prim in mesh
                .primitives()
                .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
            {
                let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
                let Some(pos) = reader
                    .read_positions()
                    .map(|p| p.map(Vec3::from).collect::<Vec<_>>())
                else {
                    continue;
                };
                let normals: Option<Vec<Vec3>> =
                    reader.read_normals().map(|n| n.map(Vec3::from).collect());
                let uvs: Option<Vec<Vec2>> = reader
                    .read_tex_coords(0)
                    .map(|t| t.into_f32().map(Vec2::from).collect());
                let colors: Option<Vec<Vec4>> = reader
                    .read_colors(0)
                    .map(|c| c.into_rgba_f32().map(Vec4::from).collect());
                let index: Vec<u32> = match reader.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..pos.len() as u32).collect(),
                };
                let m = prim.material();
                let pbr = m.pbr_metallic_roughness();
                let base = Vec4::from(pbr.base_color_factor());
                let texture = pbr
                    .base_color_texture()
                    .filter(|t| t.tex_coord() == 0)
                    .map(|t| t.texture().source().index());
                let mr_texture = pbr
                    .metallic_roughness_texture()
                    .filter(|t| t.tex_coord() == 0)
                    .map(|t| t.texture().source().index());
                let (roughness, metallic) = (pbr.roughness_factor(), pbr.metallic_factor());
                let translucent = translucency(&m);
                // A leaf's normals turn to its canopy's dome before they
                // are placed, as the mesh's are drawn.
                let dome = dome_normals(&m);
                let anchors = (dome > 0.0)
                    .then(|| super::primitive::read_foliage(src, &prim, pos.len()).ok()?)
                    .flatten();
                for xf in &at {
                    let normal_m =
                        Mat4::from_mat3(glam::Mat3::from_mat4(*xf).inverse().transpose());
                    for k in index.as_chunks::<3>().0 {
                        let v = |j: usize| k[j] as usize;
                        if (0..3).any(|j| v(j) >= pos.len()) {
                            continue;
                        }
                        let p = [0, 1, 2].map(|j| xf.transform_point3(pos[v(j)]));
                        let flat = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
                        let n = [0, 1, 2].map(|j| {
                            normals
                                .as_ref()
                                .and_then(|n| n.get(v(j)))
                                .map(|n| match &anchors {
                                    Some(a) => {
                                        let a = a[v(j)];
                                        dome_normal(*n, Vec3::new(a[4], a[5], a[6]), canopy, dome)
                                    }
                                    None => *n,
                                })
                                .map(|n| normal_m.transform_vector3(n).normalize_or(flat))
                                .unwrap_or(flat)
                        });
                        let uv = [0, 1, 2].map(|j| {
                            uvs.as_ref()
                                .and_then(|u| u.get(v(j)))
                                .copied()
                                .unwrap_or_default()
                        });
                        let color = [0, 1, 2].map(|j| {
                            colors
                                .as_ref()
                                .and_then(|c| c.get(v(j)))
                                .copied()
                                .unwrap_or(Vec4::ONE)
                        });
                        out.push(Tri {
                            p,
                            n,
                            uv,
                            color,
                            base,
                            texture,
                            roughness,
                            metallic,
                            mr_texture,
                            translucency: translucent,
                        });
                    }
                }
            }
        }
        for c in node.children() {
            stack.push((c, world));
        }
    }
    out
}

/// A frame's fine pixels: depth towards the viewer (radii from the centre,
/// `f32::MIN` where nothing covers), albedo, normal in the frame's basis
/// and material.
struct Fine {
    depth: Vec<f32>,
    col: Vec<Vec4>,
    nrm: Vec<Vec3>,
    mat: Vec<Vec3>,
}

/// The band (radii) over which the card's own shadow takes over from the
/// baked one: og-render's impostor.wgsl `imp_frame_shadow` shades a point
/// by what stands this far ahead of it, the bake by what stands nearer.
const SHADE_BAND: [f32; 2] = [0.08, 0.25];

/// How far a face `ahead` of a point towards the sun (radii) shades it in
/// the bake: fully past three depth steps of `step` (a face does not shade
/// itself across a fine pixel), fading out over [`SHADE_BAND`].
fn near_shade(ahead: f32, step: f32) -> f32 {
    let smooth = |a: f32, b: f32, x: f32| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    smooth(3.0 * step, 6.0 * step, ahead) * (1.0 - smooth(SHADE_BAND[0], SHADE_BAND[1], ahead))
}

/// Bakes the impostor of `src`'s default scene into the sphere about its
/// bounds `(min, max)` (`b` resolved: [`ImpostorBake::resolved`]).
pub fn bake(
    src: &GltfSource,
    instanced: &std::collections::BTreeMap<usize, Vec<Mat4>>,
    (min, max): (Vec3, Vec3),
    b: ImpostorBake,
) -> Result<Baked, String> {
    let b = b.resolved(src);
    if b.columns() * b.frame > 4096 {
        return Err(format!(
            "an impostor of {} × {} frames of {} px makes an atlas {} pixels wide, past 4096: lower impostorSize or impostorGrid{}",
            b.columns(),
            b.grid,
            b.frame,
            b.columns() * b.frame,
            if b.below {
                " (impostorBelow doubles the frames across)"
            } else {
                ""
            }
        ));
    }
    let center = (min + max) * 0.5;
    let radius = ((max - min) * 0.5).length().max(1e-3);
    let tris = triangles(src, instanced);
    // The textures, decoded once with their mips: base colours sRGB,
    // metallic-roughness linear.
    let mut images: std::collections::BTreeMap<(usize, bool), Mips> =
        std::collections::BTreeMap::new();
    for t in &tris {
        for (index, srgb) in [(t.texture, true), (t.mr_texture, false)] {
            if let Some(i) = index
                && !images.contains_key(&(i, srgb))
                && let Some(bytes) = src.images.get(i)
            {
                images.insert(
                    (i, srgb),
                    Mips::new(&crate::cook::texture::decode(bytes)?, srgb),
                );
            }
        }
    }
    let grid = b.grid;
    // The atlases' width (their rows' stride) and height.
    let (size, height) = (b.columns() * b.frame, grid * b.frame);
    let mut albedo = vec![0u8; (size * height * 4) as usize];
    let mut normal = vec![0u8; (size * height * 4) as usize];
    let mut material = vec![0u8; (size * height * 4) as usize];
    let ss = SUPERSAMPLE;
    let fine = b.frame * ss;
    // The model drawn from `d` (from the centre towards the viewer).
    let raster = |d: Vec3| {
        let (right, up) = frame_basis(d);
        // Per fine pixel: depth, albedo (linear rgba), normal and
        // material (roughness, translucency, metallic).
        let n_px = (fine * fine) as usize;
        let mut depth = vec![f32::MIN; n_px];
        let mut col = vec![Vec4::ZERO; n_px];
        let mut nrm = vec![Vec3::ZERO; n_px];
        let mut mat = vec![Vec3::ZERO; n_px];
        let to_px = |p: Vec3| {
            let q = p - center;
            let x = q.dot(right) / radius;
            let y = q.dot(up) / radius;
            Vec3::new(
                (x * 0.5 + 0.5) * fine as f32,
                (0.5 - y * 0.5) * fine as f32,
                q.dot(d) / radius,
            )
        };
        for t in &tris {
            let s = t.p.map(to_px);
            let area =
                (s[1].x - s[0].x) * (s[2].y - s[0].y) - (s[1].y - s[0].y) * (s[2].x - s[0].x);
            if area.abs() < 1e-12 {
                continue;
            }
            // The texture's footprint: the triangle's uv area over its
            // fine pixels (affine, one level of detail a triangle).
            let uv_area = (t.uv[1].x - t.uv[0].x) * (t.uv[2].y - t.uv[0].y)
                - (t.uv[1].y - t.uv[0].y) * (t.uv[2].x - t.uv[0].x);
            let sample = |index: Option<usize>, srgb: bool, uv: Vec2| {
                index
                    .and_then(|i| images.get(&(i, srgb)))
                    .map_or(Vec4::ONE, |m| {
                        m.sample(uv - uv.floor(), m.lod(uv_area, area))
                    })
            };
            let lo = s[0].min(s[1]).min(s[2]);
            let hi = s[0].max(s[1]).max(s[2]);
            let (x0, y0) = (lo.x.floor().max(0.0) as u32, lo.y.floor().max(0.0) as u32);
            let (x1, y1) = (
                (hi.x.ceil() as i64).clamp(0, fine as i64) as u32,
                (hi.y.ceil() as i64).clamp(0, fine as i64) as u32,
            );
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let e =
                        |a: Vec3, b: Vec3| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
                    let w = Vec3::new(e(s[1], s[2]), e(s[2], s[0]), e(s[0], s[1])) / area;
                    if w.min_element() < 0.0 {
                        continue;
                    }
                    let z = w.x * s[0].z + w.y * s[1].z + w.z * s[2].z;
                    let at = (y * fine + x) as usize;
                    if z <= depth[at] {
                        continue;
                    }
                    let uv = t.uv[0] * w.x + t.uv[1] * w.y + t.uv[2] * w.z;
                    let vc = t.color[0] * w.x + t.color[1] * w.y + t.color[2] * w.z;
                    let c = t.base * sample(t.texture, true, uv) * vc;
                    // A cut-out texel (alpha under half) leaves it uncovered.
                    if c.w < 0.5 {
                        continue;
                    }
                    depth[at] = z;
                    col[at] = c.truncate().extend(1.0);
                    let mr = sample(t.mr_texture, false, uv);
                    mat[at] = Vec3::new(
                        (t.roughness * mr.y).clamp(0.0, 1.0),
                        t.translucency,
                        (t.metallic * mr.z).clamp(0.0, 1.0),
                    );
                    let mut n = (t.n[0] * w.x + t.n[1] * w.y + t.n[2] * w.z).normalize_or(d);
                    // The side seen: a back face shows its normal turned to the viewer.
                    if n.dot(d) < 0.0 {
                        n = -n;
                    }
                    nrm[at] = Vec3::new(n.dot(right), n.dot(up), n.dot(d));
                }
            }
        }
        Fine {
            depth,
            col,
            nrm,
            mat,
        }
    };
    // The model's depth from a spread of sun directions: every third
    // frame's.
    let suns: Vec<(Vec3, Vec<f32>)> = (1..grid)
        .step_by(3)
        .flat_map(|j| {
            (1..grid)
                .step_by(3)
                .map(move |i| frame_direction(i, j, grid))
        })
        .map(|d| (d, raster(d).depth))
        .collect();
    // Per frame (row by row), its covered texels for the card's outline.
    let mut covers = Vec::with_capacity((grid * b.columns()) as usize);
    for j in 0..grid {
        for i in 0..b.columns() {
            let d = frame_direction(i, j, grid);
            let (right, up) = frame_basis(d);
            let Fine {
                depth,
                col,
                nrm,
                mat,
            } = raster(d);
            // How much of `suns` reaches the fine pixel (x, y), whose normal
            // in the frame's basis is `n`, past the model's faces near it
            // ([`near_shade`]), and how much would in the open: each sun
            // by its cosine on the face (either side of a leaf, which
            // light passes through), so one grazing it counts for nothing.
            let sunlit = |x: u32, y: u32, z: f32, n: Vec3, leaf: bool| {
                let at = |v: u32| (v as f32 + 0.5) / fine as f32 * 2.0 - 1.0;
                let q = right * at(x) - up * at(y) + d * z;
                let n = right * n.x + up * n.y + d * n.z;
                suns.iter().fold(Vec2::ZERO, |sum, (sun, front)| {
                    let cos = n.dot(*sun);
                    let cos = if leaf { cos.abs() } else { cos.max(0.0) };
                    let (r, u) = frame_basis(*sun);
                    let px = (Vec2::new(q.dot(r), -q.dot(u)) * 0.5 + 0.5) * fine as f32;
                    let shade = if px.min_element() < 0.0 || px.max_element() >= fine as f32 {
                        0.0
                    } else {
                        let ahead =
                            front[px.y as usize * fine as usize + px.x as usize] - q.dot(*sun);
                        near_shade(ahead, 2.0 / fine as f32)
                    };
                    sum + Vec2::new(1.0 - shade, 1.0) * cos
                })
            };
            // Box filter down to the frame's pixels and write them.
            let (ox, oy) = (i * b.frame, j * b.frame);
            for y in 0..b.frame {
                for x in 0..b.frame {
                    let (mut c, mut n, mut m, mut z, mut k) =
                        (Vec4::ZERO, Vec3::ZERO, Vec3::ZERO, 0.0f32, 0.0f32);
                    let mut lit = Vec2::ZERO;
                    for sy in 0..ss {
                        for sx in 0..ss {
                            let at = ((y * ss + sy) * fine + x * ss + sx) as usize;
                            if depth[at] > f32::MIN {
                                c += col[at];
                                n += nrm[at];
                                m += mat[at];
                                z += depth[at];
                                k += 1.0;
                                // One sample in four, a rotated grid.
                                if (sx + 2 * sy) % 4 == 0 {
                                    lit += sunlit(
                                        x * ss + sx,
                                        y * ss + sy,
                                        depth[at],
                                        nrm[at],
                                        mat[at].y > 0.0,
                                    );
                                }
                            }
                        }
                    }
                    let at = (((oy + y) * size + ox + x) * 4) as usize;
                    if k == 0.0 {
                        continue;
                    }
                    let a = k / (ss * ss) as f32;
                    let c = c / k;
                    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    albedo[at] = q(linear_to_srgb(c.x));
                    albedo[at + 1] = q(linear_to_srgb(c.y));
                    albedo[at + 2] = q(linear_to_srgb(c.z));
                    albedo[at + 3] = q(a);
                    // The mean, not normalised: its length says how far
                    // the texel's normals agree (impostor.wgsl imp_direct).
                    let n = n / k * 0.5 + 0.5;
                    let z = (z / k).clamp(-1.0, 1.0) * 0.5 + 0.5;
                    for (o, v) in [n.x, n.y, n.z, z].into_iter().enumerate() {
                        normal[at + o] = q(v);
                    }
                    let m = m / k;
                    // A texel no sun faces takes the open sky's.
                    let lit = if lit.y > 1e-4 { lit.x / lit.y } else { 1.0 };
                    for (o, v) in [m.x, m.y, m.z, lit].into_iter().enumerate() {
                        material[at + o] = q(v);
                    }
                }
            }
            dilate(
                [&mut albedo, &mut normal, &mut material],
                size,
                ox,
                oy,
                b.frame,
            );
            covers.push(frame_cover(&albedo, &normal, size, (ox, oy), b.frame));
        }
    }
    let trim = card_trim(&covers, b);
    let image = |rgba| Image {
        width: size,
        height,
        rgba,
    };
    Ok(Baked {
        albedo: image(albedo),
        normal: image(normal),
        material: image(material),
        center,
        radius,
        trim,
    })
}

/// The rule [`card_trim`] bounds a card by beside the values it reads,
/// which [`trim_stamp`] hashes: bumped only when its code changes.
const TRIM_VERSION: u32 = 1;

/// What a cooked outline depends on, in the model blob's stamp: the
/// rule's version, its own values and the shader's march
/// (og-render-data's `impostor::march`), so a change to any cooks the
/// outlines again.
pub fn trim_stamp() -> Vec<u8> {
    let mut b = TRIM_VERSION.to_le_bytes().to_vec();
    b.extend((TRIM_WINDOW as u32).to_le_bytes());
    b.extend(TRIM_VIEWS.to_le_bytes());
    b.extend(TRIM_PAD_TEXELS.to_le_bytes());
    b.extend((TRIM_BINS as u32).to_le_bytes());
    b.extend(og_render_data::impostor::march_stamp());
    b
}

/// Texels about a covered texel whose depths the card's parallax steps may
/// read for it: three texels (its bilinear footprint and a step's spread)
/// of the coarse level the fixed point reads (`march::COARSE_LEVELS`), in
/// the frame's own texels.
const TRIM_WINDOW: usize = 3 << march::COARSE_LEVELS;
/// Views sampled along each side of a grid cell for the outline.
const TRIM_VIEWS: u32 = 4;
/// Atlas texels the outline keeps past every covered texel: the bilinear
/// footprint and coverage spread of the levels a card reads before it is
/// a few pixels wide (level 2), where one more texel is under a pixel.
const TRIM_PAD_TEXELS: f32 = 4.0;
/// Rows of the outline's height profile.
const TRIM_BINS: usize = 256;

/// A frame's covered texels for [`card_trim`]: each texel's point in the
/// frame's square (`[-1, 1]²`, y up) and the range of depths (radii,
/// towards the viewer) the card's parallax steps may land it at: its own,
/// back to `march::THICK` behind it, and its neighbours' (`TRIM_WINDOW`),
/// as the atlas stores them (dilated out from the silhouette).
fn frame_cover(
    albedo: &[u8],
    normal: &[u8],
    size: u32,
    (ox, oy): (u32, u32),
    frame: u32,
) -> Vec<[f32; 4]> {
    let f = frame as usize;
    let at = |x: usize, y: usize| ((oy as usize + y) * size as usize + ox as usize + x) * 4;
    // A dilated texel's depth (its alpha 0) counts, an empty one's not.
    let depth: Vec<Option<f32>> = (0..f * f)
        .map(|k| {
            let p = at(k % f, k / f);
            (normal[p..p + 4] != [0; 4]).then(|| f32::from(normal[p + 3]) / 255.0 * 2.0 - 1.0)
        })
        .collect();
    // The window's lowest and highest depth: a row pass, then a column one.
    let window = |pick: fn(f32, f32) -> f32| {
        let mut rows = vec![None::<f32>; f * f];
        for y in 0..f {
            for x in 0..f {
                let (a, b) = (x.saturating_sub(TRIM_WINDOW), (x + TRIM_WINDOW).min(f - 1));
                rows[y * f + x] = (a..=b).filter_map(|k| depth[y * f + k]).reduce(pick);
            }
        }
        let mut out = vec![None::<f32>; f * f];
        for y in 0..f {
            for x in 0..f {
                let (a, b) = (y.saturating_sub(TRIM_WINDOW), (y + TRIM_WINDOW).min(f - 1));
                out[y * f + x] = (a..=b).filter_map(|k| rows[k * f + x]).reduce(pick);
            }
        }
        out
    };
    let (lo, hi) = (window(f32::min), window(f32::max));
    let unit = |v: usize| (v as f32 + 0.5) / frame as f32 * 2.0 - 1.0;
    (0..f * f)
        .filter(|&k| albedo[at(k % f, k / f) + 3] > 0)
        .map(|k| {
            let z = depth[k].unwrap_or(0.0);
            [
                unit(k % f),
                -unit(k / f),
                lo[k].unwrap_or(z).min(z - march::THICK),
                hi[k].unwrap_or(z).max(z),
            ]
        })
        .collect()
}

/// The card's outline ([`CardTrim`]): over views sampled across the grid
/// (`TRIM_VIEWS` a cell side, the lower hemisphere's too with `below`;
/// without, a view from below is the horizon's above it, impostor_common.wgsl
/// `imp_view`), each of the three frames the card blends for it
/// ([`blend_frames`]) draws its covered texels (`covers`, per frame row by
/// row) where the eye's ray through the card's point meets them: the
/// texel's point at each depth its parallax may land it at, seen along the
/// view in the view's basis, and where the frame projects without
/// parallax (a coarse shadow card). The outline holds every such point,
/// padded by `TRIM_PAD_TEXELS` and by how far a point moves between
/// sampled views: the card then never cuts a texel a frame would draw.
fn card_trim(covers: &[Vec<[f32; 4]>], b: ImpostorBake) -> CardTrim {
    let grid = b.grid.max(2);
    let n = (grid - 1) * TRIM_VIEWS;
    let mut views = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let d = hemi_oct_decode(Vec2::new(i as f32, j as f32) / n as f32 * 2.0 - 1.0);
            views.push(d);
            if b.below && d.y > 0.0 {
                views.push(Vec3::new(d.x, -d.y, d.z));
            }
        }
    }
    // The widest step between neighbouring samples (the grid's corners
    // are its coarsest): a point a radius out moves by as much.
    let step = (0..n)
        .flat_map(|j| (0..n).map(move |i| (i, j)))
        .map(|(i, j)| {
            let d = |i: u32, j: u32| {
                hemi_oct_decode(Vec2::new(i as f32, j as f32) / n as f32 * 2.0 - 1.0)
            };
            d(i, j).angle_between(d(i + 1, j + 1))
        })
        .fold(0.0f32, f32::max);
    // Per height bin, the widest |x| a point in it reaches.
    let mut width = [f32::NEG_INFINITY; TRIM_BINS];
    let bin = |y: f32| (((y * 0.5 + 0.5) * TRIM_BINS as f32) as usize).min(TRIM_BINS - 1);
    let mut put = |p: Vec2| {
        if p.y.abs() <= 1.0 + 1e-3 {
            let k = bin(p.y.clamp(-1.0, 1.0));
            width[k] = width[k].max(p.x.abs());
        } else {
            // Past the square: the card's square is the most it covers.
            let k = if p.y > 0.0 { TRIM_BINS - 1 } else { 0 };
            width[k] = width[k].max(p.x.abs());
        }
    };
    let columns = b.columns();
    for v in views {
        let (vr, vu) = frame_basis(v);
        for ((ci, cj), _) in blend_frames(v, grid, b.below) {
            let Some(cover) = covers.get((cj * columns + ci) as usize) else {
                continue;
            };
            let d = frame_direction(ci, cj, grid);
            let (r, u) = frame_basis(d);
            // A frame point (x, y, z) seen along v: x r + y u + z d in
            // v's basis.
            let (rx, ux, dx) = (r.dot(vr), u.dot(vr), d.dot(vr));
            let (ry, uy, dy) = (r.dot(vu), u.dot(vu), d.dot(vu));
            // Without parallax the card's point projects onto the frame's
            // point: solve (p·r, p·u) = (x, y) for p = a vr + b vu.
            let det = rx * uy - ux * ry;
            for &[x, y, lo, hi] in cover {
                for z in [lo, hi] {
                    put(Vec2::new(
                        x * rx + y * ux + z * dx,
                        x * ry + y * uy + z * dy,
                    ));
                }
                if det.abs() > 1e-3 {
                    put(Vec2::new((x * uy - y * ux) / det, (y * rx - x * ry) / det));
                }
            }
        }
    }
    let Some(first) = width.iter().position(|w| w.is_finite()) else {
        return CardTrim::FULL;
    };
    let last = width.iter().rposition(|w| w.is_finite()).expect("one is");
    let pad = TRIM_PAD_TEXELS * 2.0 / b.frame as f32 + 3f32.sqrt() * step * 0.5;
    let edge = |k: usize| k as f32 / TRIM_BINS as f32 * 2.0 - 1.0;
    let y = [
        (edge(first) - pad).max(-1.0),
        (edge(last + 1) + pad).min(1.0),
    ];
    // Each strip as wide as the points within `pad` of its rows; each
    // boundary as wide as the strips either side.
    let rows = og_render_data::impostor::CARD_ROWS;
    let strip = |s: usize| {
        let (bottom, top) = (
            y[0] + (y[1] - y[0]) * s as f32 / rows as f32 - pad,
            y[0] + (y[1] - y[0]) * (s + 1) as f32 / rows as f32 + pad,
        );
        (bin(bottom.max(-1.0))..=bin(top.min(1.0)))
            .map(|k| width[k])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let strips: Vec<f32> = (0..rows).map(strip).collect();
    let widths = std::array::from_fn(|k| {
        let w = strips[k.saturating_sub(1)].max(strips[k.min(rows - 1)]);
        if w.is_finite() {
            (w + pad).min(1.0)
        } else {
            0.0
        }
    });
    CardTrim { widths, y }
}

/// Gives a frame's uncovered texels their nearest covered neighbour's
/// colour, normal and material (the albedo's alpha kept 0), a few texels
/// out from the silhouette.
fn dilate([albedo, normal, material]: [&mut [u8]; 3], size: u32, ox: u32, oy: u32, frame: u32) {
    let mut filled: Vec<bool> = (0..frame * frame)
        .map(|k| albedo[((((oy + k / frame) * size) + ox + k % frame) * 4 + 3) as usize] > 0)
        .collect();
    if !filled.iter().any(|f| *f) {
        return;
    }
    for _ in 0..8 {
        let before = filled.clone();
        for y in 0..frame {
            for x in 0..frame {
                if before[(y * frame + x) as usize] {
                    continue;
                }
                let from = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .map(|(dx, dy)| (x as i32 + dx, y as i32 + dy))
                    .find(|&(nx, ny)| {
                        nx >= 0
                            && ny >= 0
                            && (nx as u32) < frame
                            && (ny as u32) < frame
                            && before[(ny as u32 * frame + nx as u32) as usize]
                    });
                if let Some((nx, ny)) = from {
                    let src = ((((oy + ny as u32) * size) + ox + nx as u32) * 4) as usize;
                    let dst = ((((oy + y) * size) + ox + x) * 4) as usize;
                    for o in 0..3 {
                        albedo[dst + o] = albedo[src + o];
                    }
                    for o in 0..4 {
                        normal[dst + o] = normal[src + o];
                        material[dst + o] = material[src + o];
                    }
                    filled[(y * frame + x) as usize] = true;
                }
            }
        }
    }
}
