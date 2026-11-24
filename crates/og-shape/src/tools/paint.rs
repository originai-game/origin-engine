//! Procedural surfaces: a material's [`Surface`] cooked into texture maps
//! on a fresh atlas, so a model built from a program carries a patterned
//! base colour, roughness and metalness, a normal map and emission
//! without a painted image.
//!
//! Entry: [`paint_scene`]. When a part's material has a surface, every
//! part without an image map is unwrapped anew and the parts are packed
//! into one atlas at one texel density (the largest surface's
//! `density`). For every texel a part covers:
//!
//! 1. **The surface point.** Its position (metres, model axes, Z up), the
//!    frame a viewer shades it in (normal, MikkTSpace tangent, as
//!    [`crate::tools::bake`] writes it), and its curvature: the
//!    surface's bend within a smooth surface (a vertex's mean bend over
//!    its neighbours, interpolated) and its distance to the nearest sharp
//!    edge (between two surfaces), convex or concave.
//! 2. **The layers**, in order, over the material's own colour,
//!    roughness, metalness and emission: each [`Pattern`] gives a mask
//!    (0..1) and a tone; the mask, times `amount`, mixes in the layer's
//!    colour, roughness, metalness and emission and raises the height by
//!    `relief`, and `jitter` varies the brightness beneath by the tone.
//! 3. **The normal**: the height's slope along the tangent and the
//!    bitangent (the height again half a texel either way), the normal
//!    tilted by it, written in the texel's frame as the bake writes it.
//! 4. **Dilation** past the charts, as the bake does.
//!
//! The maps take the parts' factors (each part's material may differ);
//! the material written is neutral and shared by every painted part.
//!
//! Arithmetic: `+ − × /`, `floor` and `sqrt` in a fixed order, the noise
//! of [`crate::edit::displace`], the sRGB curve of
//! [`crate::tools::sampler`]: the PNG bytes are the same on every host.

use std::collections::BTreeMap;
use std::fmt;

use crate::edit::displace::{corner, craters, gradient, unit as hash_unit};
use crate::math::{DVec2, DVec3};
use crate::mesh::Solid;
use crate::tools::bake::{BakedMap, data_byte, dilate, dilation, frame_at, rasterize, srgb_byte};
use crate::tools::export::{ColorSpace, Material, Scene};
use crate::tools::image::{Codec, Texture};
use crate::tools::preview::Image;
use crate::tools::sampler::{linear_table, to_linear};
use crate::tools::surface::{CornerFrame, corner_frames};
use crate::tools::{Atlas, Unwrap};

/// What a layer draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pattern {
    /// Fractal noise, mottling (`scale` its largest blotch); with a
    /// `threshold`, patches (frost, grime, lichen).
    #[default]
    Noise,
    /// Ridged fractal noise: crests and cracks.
    Ridged,
    /// Bowls with raised rims (`scale` the lattice): pocked rock, a moon.
    /// The mask is the bowls; `relief` raises the rims and sinks the
    /// bowls.
    Craters,
    /// Bands across `axis` every `scale`, each its own tone, warped by
    /// noise, `width` the blend between bands: sediment, ice layers.
    Strata,
    /// Lines across `axis` every `scale`, `width` wide: trim, hazard and
    /// light lines.
    Stripes,
    /// Seams of a grid of panels `scale` wide and `cell` tall (`width`
    /// the seam), projected along the normal's main axis; the tone is the
    /// panel's own.
    Panels,
    /// Domes `width` across, `scale` apart, along the edges of panels
    /// `cell` wide, `inset` in from them.
    Rivets,
    /// Thin branching lines where noise crosses zero (`width` their
    /// width): ice veins, cracks, circuitry glow.
    Veins,
    /// Convex edges and bends: sharp edges within `width`, and the
    /// surface's bend: worn paint, chipped rims, frost on crests.
    Edges,
    /// Concave edges and hollows, the same way: dirt, soot, shadowed
    /// seams.
    Cavities,
    /// Where the surface faces `axis`: 1 facing it, 0 facing away (shaped
    /// by `threshold`): frost and dust on tops, dark undersides, a value
    /// step per face of a faceted part.
    Facing,
    /// Along `axis` across every part painted with the material, 0 at its
    /// lowest point and 1 at its highest (shaped by `threshold`): a dark
    /// foot and a pale crown, soot rising up a tower.
    Gradient,
}

impl Pattern {
    /// The pattern a name gives.
    pub fn parse(s: &str) -> Option<Pattern> {
        Some(match s {
            "noise" => Pattern::Noise,
            "ridged" => Pattern::Ridged,
            "craters" => Pattern::Craters,
            "strata" => Pattern::Strata,
            "stripes" => Pattern::Stripes,
            "panels" => Pattern::Panels,
            "rivets" => Pattern::Rivets,
            "veins" => Pattern::Veins,
            "edges" => Pattern::Edges,
            "cavities" => Pattern::Cavities,
            "facing" => Pattern::Facing,
            "gradient" => Pattern::Gradient,
            _ => return None,
        })
    }

    /// The names [`Pattern::parse`] takes.
    pub const NAMES: &str = "noise, ridged, craters, strata, stripes, panels, rivets, veins, edges, cavities, facing or gradient";
}

/// One layer of a [`Surface`]. Lengths are metres.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    /// What it draws.
    pub pattern: Pattern,
    /// Its size: the noise's largest feature, the band or stripe period,
    /// the panel width, the rivet spacing, the veins' cell.
    pub scale: f64,
    /// Line, seam, rivet, vein and band-blend width; for edges and
    /// cavities, how far the wear reaches.
    pub width: f64,
    /// Panels: their height (0 is `scale`); rivets: the panel width they
    /// line (0 is four `scale`).
    pub cell: f64,
    /// Rivets: from the panel edge (0 is the rivet's width).
    pub inset: f64,
    /// Strata and stripes run across this direction, facing and gradient
    /// along it (model axes, Z up).
    pub axis: DVec3,
    /// Noise, ridged and veins: octaves; 0 is four, at most eight.
    pub octaves: u32,
    /// Noise, ridged, facing and gradient: the value (0..1) above which the mask is 1, with
    /// `softness` either side; `None` is the value itself.
    pub threshold: Option<f64>,
    /// The blend either side of `threshold`; 0 is 0.05.
    pub softness: f64,
    /// Picks another field of the same pattern.
    pub seed: u32,
    /// The mask's strength, 0..1.
    pub amount: f64,
    /// Breaks the mask up by noise of `width` × 4 (edges, cavities,
    /// veins…), 0..1.
    pub breakup: f64,
    /// sRGB colour mixed in by the mask.
    pub color: Option<[f64; 3]>,
    /// Brightness varied by the pattern's tone over the whole layer, 0..1
    /// (each panel, band or blotch a little lighter or darker).
    pub jitter: f64,
    /// Roughness mixed in by the mask.
    pub roughness: Option<f64>,
    /// Metalness mixed in by the mask.
    pub metalness: Option<f64>,
    /// sRGB emission mixed in by the mask.
    pub emissive: Option<[f64; 3]>,
    /// The emission's strength; above 1 it blooms.
    pub glow: f64,
    /// Height added by the mask, metres (negative cuts a groove).
    pub relief: f64,
}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            pattern: Pattern::Noise,
            scale: 0.0,
            width: 0.0,
            cell: 0.0,
            inset: 0.0,
            axis: DVec3::Z,
            octaves: 0,
            threshold: None,
            softness: 0.0,
            seed: 0,
            amount: 1.0,
            breakup: 0.0,
            color: None,
            jitter: 0.0,
            roughness: None,
            metalness: None,
            emissive: None,
            glow: 1.0,
            relief: 0.0,
        }
    }
}

/// A material's procedural surface: layers over its own factors, cooked
/// at `density` texels per metre.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    /// Texels per metre on the surface; 0 is 256.
    pub density: f64,
    /// Added to every layer's seed.
    pub seed: u32,
    /// Drawn in order, each over the ones before.
    pub layers: Vec<Layer>,
    /// The normal map's strength; 0 is 1.
    pub normal_scale: f64,
}

impl Surface {
    /// Why the surface cannot be painted, if it cannot.
    pub fn check(&self) -> Result<(), String> {
        let pos = |v: f64| v.is_finite() && v > 0.0;
        if !(self.density == 0.0 || pos(self.density)) {
            return Err("density: texels per metre, positive".into());
        }
        if !(self.normal_scale == 0.0 || pos(self.normal_scale)) {
            return Err("normalScale: positive".into());
        }
        for (i, l) in self.layers.iter().enumerate() {
            let at = |w: &str| format!("layer {i} ({:?}): {w}", l.pattern);
            let needs_scale = !matches!(
                l.pattern,
                Pattern::Edges | Pattern::Cavities | Pattern::Facing | Pattern::Gradient
            );
            if needs_scale && !pos(l.scale) {
                return Err(at("scale must be positive (metres)"));
            }
            let needs_width = matches!(
                l.pattern,
                Pattern::Stripes
                    | Pattern::Panels
                    | Pattern::Rivets
                    | Pattern::Veins
                    | Pattern::Edges
                    | Pattern::Cavities
            );
            if needs_width && !pos(l.width) {
                return Err(at("width must be positive (metres)"));
            }
            for (k, v) in [
                ("width", l.width),
                ("cell", l.cell),
                ("inset", l.inset),
                ("softness", l.softness),
            ] {
                if !(v.is_finite() && v >= 0.0) {
                    return Err(at(&format!("{k} must be 0 or positive")));
                }
            }
            for (k, v) in [
                ("amount", l.amount),
                ("breakup", l.breakup),
                ("jitter", l.jitter),
            ] {
                if !(v.is_finite() && (0.0..=1.0).contains(&v)) {
                    return Err(at(&format!("{k} must be 0..1")));
                }
            }
            for (k, v) in [("roughness", l.roughness), ("metalness", l.metalness)] {
                if v.is_some_and(|v| !(v.is_finite() && (0.0..=1.0).contains(&v))) {
                    return Err(at(&format!("{k} must be 0..1")));
                }
            }
            if l.threshold.is_some_and(|v| !v.is_finite()) {
                return Err(at("threshold must be finite"));
            }
            if !(l.glow.is_finite() && l.glow >= 0.0) || !l.relief.is_finite() {
                return Err(at("glow 0 or positive, relief finite"));
            }
            if l.breakup > 0.0 && !(pos(l.width) || pos(l.scale)) {
                return Err(at("breakup needs a width or a scale (its noise's size)"));
            }
            if l.octaves > crate::edit::MAX_OCTAVES {
                return Err(at("octaves at most 8"));
            }
            if !(l.axis.is_finite() && l.axis.length() > 0.0) {
                return Err(at("axis must be a direction"));
            }
            for c in l.color.iter().chain(l.emissive.iter()).flatten() {
                if !c.is_finite() {
                    return Err(at("colours must be finite"));
                }
            }
        }
        Ok(())
    }
}

/// How to paint. The zero value caps the atlas at 2048 texels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Paint {
    /// The largest atlas, texels across; 0 is 2048. A model whose area at
    /// the density needs more is painted at this size, more coarsely
    /// ([`PaintReport::density`] says how).
    pub max_size: u32,
}

/// What a paint did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintReport {
    /// Texels across the atlas.
    pub size: u32,
    /// Texels per metre asked for (the largest surface's).
    pub asked: f64,
    /// Texels per metre the atlas holds.
    pub density: f64,
    /// The painted parts' surface, square metres.
    pub area: f64,
    /// Share of the atlas the charts cover.
    pub coverage: f64,
    /// Charts in the atlas.
    pub charts: usize,
    /// The maps written, in [`crate::tools::MAP_KINDS`] order.
    pub maps: Vec<BakedMap>,
}

/// Parts painted: their solids with the atlas's UVs, and the material
/// they share.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Painted {
    /// Per painted part (its index in the scene), its solid with new UVs.
    pub parts: Vec<(usize, Solid)>,
    /// The material every painted part takes.
    pub material: Material,
    /// The report.
    pub report: PaintReport,
}

impl Painted {
    /// Puts the painted solids and material into `scene`, with tangents
    /// (the frame the normal map was painted in).
    pub fn apply<'a>(&'a self, scene: &mut Scene<'a>) {
        for (k, s) in &self.parts {
            let p = &mut scene.parts[*k];
            p.solid = s;
            p.material = self.material.clone();
            p.tangents = true;
        }
    }
}

/// Why a scene could not be painted.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintError {
    /// A surface's values (which).
    BadSurface(String),
    /// The parts could not be unwrapped or packed (why).
    Atlas(String),
}

impl fmt::Display for PaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaintError::BadSurface(w) => write!(f, "surface: {w}"),
            PaintError::Atlas(w) => write!(f, "surface: atlas: {w}"),
        }
    }
}

impl std::error::Error for PaintError {}

/// The scene's surfaces painted, or `None` when no part's material has a
/// [`Surface`]. Parts whose material has an image map keep it; every
/// other part is painted, its material's factors in the maps.
pub fn paint_scene(scene: &Scene, d: &Paint, codec: &Codec) -> Result<Option<Painted>, PaintError> {
    let surfaces: Vec<&Surface> = scene
        .parts
        .iter()
        .filter_map(|p| p.material.surface.as_deref())
        .collect();
    if surfaces.is_empty() {
        return Ok(None);
    }
    for s in &surfaces {
        s.check().map_err(PaintError::BadSurface)?;
    }
    let asked = surfaces
        .iter()
        .map(|s| if s.density == 0.0 { 256.0 } else { s.density })
        .fold(0.0, f64::max);
    let max = if d.max_size == 0 { 2048 } else { d.max_size };
    let taken: Vec<usize> = scene
        .parts
        .iter()
        .enumerate()
        .filter(|(_, p)| p.material.map.is_none() && !p.solid.triangles.is_empty())
        .map(|(k, _)| k)
        .collect();
    // Square metres, the kernel's millimetres.
    let area: f64 = taken
        .iter()
        .map(|&k| solid_area(scene.parts[k].solid) / 1e6)
        .sum();
    // Charts fill about half the square with their padding.
    let want = (asked * (area / 0.5).sqrt()).ceil();
    let size = ((want / 64.0).ceil() as u32 * 64).clamp(64, max.max(64));
    let mut solids: Vec<Solid> = Vec::with_capacity(taken.len());
    for &k in &taken {
        let mut s = scene.parts[k].solid.clone();
        crate::tools::unwrap(
            &mut s,
            &Unwrap {
                resolution: size,
                conformal: true,
                seams: true,
                ..Default::default()
            },
        )
        .map_err(|e| PaintError::Atlas(format!("{}: {e}", scene.parts[k].name)))?;
        solids.push(s);
    }
    let packed = crate::tools::repack(
        &mut solids,
        &Atlas {
            size,
            padding: 4,
            no_turns: false,
        },
    )
    .map_err(|e| PaintError::Atlas(e.to_string()))?;
    let density = packed.scale * 1000.0;
    let texel = 1.0 / density;
    let n = size as usize;

    // Every painted triangle, in part order: its UVs, frames, positions,
    // the corners' bends, and its part's layers and factors.
    let mut uv: Vec<[DVec2; 3]> = Vec::new();
    let mut tris: Vec<Tri> = Vec::new();
    let mut looks: Vec<Look> = Vec::new();
    let mut edges = Edges::new(
        surfaces
            .iter()
            .flat_map(|s| &s.layers)
            .filter(|l| matches!(l.pattern, Pattern::Edges | Pattern::Cavities))
            .map(|l| l.width)
            .fold(0.0, f64::max),
    );
    for (j, s) in solids.iter().enumerate() {
        let m = &scene.parts[taken[j]].material;
        looks.push(Look::of(m));
        let frames = corner_frames(s);
        let bends = bends(s);
        edges.add(s);
        let corners = s.uvs.as_ref().expect("unwrapped");
        for (i, t) in s.triangles.iter().enumerate() {
            uv.push(corners[i]);
            tris.push(Tri {
                look: j as u32,
                at: t.map(|v| s.positions[v as usize] / 1000.0),
                frames: frames[i],
                bend: t.map(|v| bends.get(&(v, s.surfaces[i])).copied().unwrap_or(0.0)),
            });
        }
    }
    // A gradient runs across every part painted with its material.
    let names: Vec<&str> = taken
        .iter()
        .map(|&k| {
            let p = &scene.parts[k];
            if p.material.name.is_empty() {
                p.name.as_str()
            } else {
                p.material.name.as_str()
            }
        })
        .collect();
    for j in 0..looks.len() {
        for li in 0..looks[j].layers.len() {
            let l = &looks[j].layers[li];
            if l.pattern != Pattern::Gradient {
                continue;
            }
            let axis = l.axis / l.axis.length();
            let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
            for t in tris.iter().filter(|t| names[t.look as usize] == names[j]) {
                for v in t.at {
                    lo = lo.min(v.dot(axis));
                    hi = hi.max(v.dot(axis));
                }
            }
            looks[j].spans[li] = (lo, hi);
        }
    }
    let (owner, at) = rasterize(&uv, size);

    // Paint: each texel depends on itself only.
    let glow = looks.iter().map(Look::glow).fold(1.0, f64::max);
    let emits = looks.iter().any(Look::emits);
    let lin = linear_table();
    let texels: Vec<Option<Texel>> = (0..n * n)
        .map(|k| {
            let i = owner[k];
            if i == u32::MAX {
                return None;
            }
            let t = &tris[i as usize];
            let b = at[k].0;
            let f = frame_at(&t.frames, b);
            if f.normal == DVec3::ZERO {
                return None;
            }
            let p = t.at[0] * b[0] + t.at[1] * b[1] + t.at[2] * b[2];
            let bend = t.bend[0] * b[0] + t.bend[1] * b[1] + t.bend[2] * b[2];
            Some(looks[t.look as usize].texel(p, &f, bend, &edges, texel))
        })
        .collect();

    // The maps, rows from the top.
    let filled: Vec<bool> = texels.iter().map(Option::is_some).collect();
    let mut images: [Vec<u8>; 4] = std::array::from_fn(|_| vec![0u8; n * n * 4]);
    for (k, t) in texels.iter().enumerate() {
        let Some(t) = t else { continue };
        let px = ((n - 1 - k / n) * n + k % n) * 4;
        for c in 0..3 {
            images[0][px + c] = srgb_byte(&lin, t.color[c]);
            images[3][px + c] = srgb_byte(&lin, t.emissive[c] / glow);
        }
        images[0][px + 3] = 255;
        images[1][px..px + 4].copy_from_slice(&[
            255,
            data_byte(t.roughness),
            data_byte(t.metalness),
            255,
        ]);
        images[2][px..px + 3].copy_from_slice(&t.normal.map(|v| data_byte((v + 1.0) / 2.0)));
        images[2][px + 3] = 255;
        images[3][px + 3] = 255;
    }
    let (level, layers) = dilation(&filled, n, 0);
    // The colours the maps were mixed from, sRGB, each once.
    let byte = |c: f64| (c.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
    let mut palette = std::collections::BTreeSet::new();
    for &k in &taken {
        let m = &scene.parts[k].material;
        let c = m.color.unwrap_or([1.0; 3]);
        let c = match m.color_space {
            ColorSpace::Srgb => c,
            ColorSpace::Linear => c.map(crate::tools::sampler::to_srgb),
        };
        palette.insert(c.map(byte));
        for l in m.surface.iter().flat_map(|s| &s.layers) {
            for c in l.color.iter().chain(l.emissive.iter()) {
                palette.insert(c.map(byte));
            }
        }
    }
    let first = &scene.parts[taken[0]].material;
    let mut material = Material {
        name: if first.name.is_empty() {
            scene.parts[taken[0]].name.clone()
        } else {
            first.name.clone()
        },
        color: None,
        color_space: ColorSpace::Linear,
        metalness: 1.0,
        roughness: Some(1.0),
        emissive: if emits { [1.0; 3] } else { [0.0; 3] },
        emissive_intensity: (emits && glow > 1.0).then_some(glow),
        normal_scale: surfaces.iter().map(|s| s.normal_scale).find(|&v| v != 0.0),
        map: None,
        metalness_map: None,
        normal_map: None,
        emissive_map: None,
        ao_map: None,
        ao_map_intensity: None,
        surface: None,
        painted_from: Some(palette.into_iter().collect()),
        ..first.clone()
    };
    let mut report = PaintReport {
        size,
        asked,
        density,
        area,
        coverage: packed.measures.coverage,
        charts: packed.measures.charts,
        maps: Vec::new(),
    };
    let kinds = ["map", "metalness_map", "normal_map", "emissive_map"];
    for (k, mut rgba) in images.into_iter().enumerate() {
        if k == 3 && !emits {
            continue;
        }
        dilate(&mut rgba, n, &level, &layers);
        let png = (codec.png)(&Image {
            width: size,
            height: size,
            rgba,
        });
        report.maps.push(BakedMap {
            kind: kinds[k],
            size,
            file_bytes: png.len(),
        });
        let t = Some(Texture {
            name: format!("{}_{}.png", material.name, kinds[k]),
            mime: "image/png".into(),
            bytes: png,
        });
        match k {
            0 => material.map = t,
            1 => material.metalness_map = t,
            2 => material.normal_map = t,
            _ => material.emissive_map = t,
        }
    }
    Ok(Some(Painted {
        parts: taken.into_iter().zip(solids).collect(),
        material,
        report,
    }))
}

/// One painted triangle.
struct Tri {
    look: u32,
    /// Corners in metres.
    at: [DVec3; 3],
    frames: [CornerFrame; 3],
    /// Corners' bend, per metre (convex positive).
    bend: [f64; 3],
}

/// One texel's values: linear colour and emission, data, and the normal in
/// the texel's frame.
#[derive(Clone, Copy, Debug)]
struct Texel {
    color: [f64; 3],
    roughness: f64,
    metalness: f64,
    emissive: [f64; 3],
    normal: [f64; 3],
}

/// A part's factors (linear) and layers.
struct Look {
    color: [f64; 3],
    roughness: f64,
    metalness: f64,
    emissive: [f64; 3],
    layers: Vec<Layer>,
    /// Per layer, a gradient's lowest and highest point along its axis.
    spans: Vec<(f64, f64)>,
    seed: u32,
    normal_scale: f64,
}

fn srgb3(c: [f64; 3]) -> [f64; 3] {
    c.map(|v| to_linear(v.clamp(0.0, 1.0)))
}

fn mix3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn frac(x: f64) -> f64 {
    x - x.floor()
}

/// The `noise` pattern's stretch about 0.5: fbm's 5th and 95th
/// percentiles (about 0.3 and 0.7) land near 0.05 and 0.95.
const NOISE_STRETCH: f64 = 2.25;

/// A crease sharper than 40° (this, the cosine between the faces'
/// normals) is an edge for wear and dirt even within one surface: a hex
/// column's corners.
const CREASE_COS: f64 = 0.766_044_443_118_978;

/// Fractal (or ridged) noise in 0..1 at `p` (lattice units).
fn fbm(p: DVec3, octaves: u32, seed: u32, ridged: bool) -> f64 {
    let octaves = if octaves == 0 { 4 } else { octaves };
    let (mut sum, mut norm, mut f, mut w) = (0.0, 0.0, 1.0, 1.0);
    for o in 0..octaves {
        let n = gradient(p * f, seed.wrapping_add(o.wrapping_mul(0x9e37_79b9)));
        sum += w * if ridged {
            let r = 1.0 - n.abs();
            r * r
        } else {
            n
        };
        norm += w;
        f *= 2.0;
        w *= 0.5;
    }
    let v = sum / norm;
    if ridged {
        v.clamp(0.0, 1.0)
    } else {
        (v * 0.5 / 0.7 + 0.5).clamp(0.0, 1.0)
    }
}

/// The point's two coordinates on the plane across the normal's main axis.
fn project(p: DVec3, n: DVec3) -> (f64, f64) {
    let a = n.abs();
    if a.z >= a.x && a.z >= a.y {
        (p.x, p.y)
    } else if a.x >= a.y {
        (p.y, p.z)
    } else {
        (p.x, p.z)
    }
}

impl Look {
    fn of(m: &Material) -> Look {
        let lin = |c: [f64; 3]| match m.color_space {
            ColorSpace::Srgb => srgb3(c),
            ColorSpace::Linear => c,
        };
        let s = m.surface.as_deref().cloned().unwrap_or_default();
        let e = m.emissive_intensity.unwrap_or(1.0);
        Look {
            color: lin(m.color.unwrap_or([1.0; 3])),
            roughness: m.roughness.unwrap_or(1.0),
            metalness: m.metalness,
            emissive: lin(m.emissive).map(|v| v * e),
            normal_scale: if s.normal_scale == 0.0 {
                1.0
            } else {
                s.normal_scale
            },
            seed: s.seed,
            spans: vec![(0.0, 0.0); s.layers.len()],
            layers: s.layers,
        }
    }

    fn glow(&self) -> f64 {
        self.layers
            .iter()
            .filter(|l| l.emissive.is_some())
            .map(|l| l.glow)
            .chain(self.emissive.iter().copied())
            .fold(1.0, f64::max)
    }

    fn emits(&self) -> bool {
        self.emissive.iter().any(|&v| v > 0.0)
            || self
                .layers
                .iter()
                .any(|l| l.emissive.is_some_and(|e| e.iter().any(|&v| v > 0.0)) && l.glow > 0.0)
    }

    /// The layers' values at `p` (metres), the normal `n` choosing panel
    /// projections, `bend` the surface's bend there.
    fn sample(&self, p: DVec3, n: DVec3, bend: f64, edges: &Edges, texel: f64) -> Texel {
        let mut t = Texel {
            color: self.color,
            roughness: self.roughness,
            metalness: self.metalness,
            emissive: self.emissive,
            normal: [0.0, 0.0, 0.0],
        };
        // The height rides in normal[0] while sampling.
        let mut height = 0.0;
        for (li, l) in self.layers.iter().enumerate() {
            let seed = self
                .seed
                .wrapping_add(l.seed)
                .wrapping_add((li as u32).wrapping_mul(0x85eb_ca6b));
            let (mask, tone, lift) = l.mask(p, n, bend, edges, texel, seed, self.spans[li]);
            if l.jitter > 0.0 {
                let k = 1.0 + l.jitter * (tone * 2.0 - 1.0);
                t.color = t.color.map(|v| v * k);
            }
            let m = mask * l.amount;
            if let Some(c) = l.color {
                t.color = mix3(t.color, srgb3(c), m);
            }
            if let Some(r) = l.roughness {
                t.roughness += (r - t.roughness) * m;
            }
            if let Some(v) = l.metalness {
                t.metalness += (v - t.metalness) * m;
            }
            if let Some(e) = l.emissive {
                t.emissive = mix3(t.emissive, srgb3(e).map(|v| v * l.glow), m);
            }
            height += l.relief * lift.unwrap_or(m);
        }
        t.normal[0] = height;
        t
    }

    fn texel(&self, p: DVec3, f: &CornerFrame, bend: f64, edges: &Edges, texel: f64) -> Texel {
        let mut t = self.sample(p, f.normal, bend, edges, texel);
        let h = t.normal[0];
        let bt = f.normal.cross(f.tangent) * f.w;
        let e = texel * 0.5;
        let hs = |d: DVec3| self.sample(p + d * e, f.normal, bend, edges, texel).normal[0];
        let du = (hs(f.tangent) - hs(-f.tangent)) / (2.0 * e);
        let dv = (hs(bt) - hs(-bt)) / (2.0 * e);
        let _ = h;
        let k = self.normal_scale;
        let tilted = f.normal - f.tangent * (du * k) - bt * (dv * k);
        let l = tilted.length();
        let v = if l > 0.0 { tilted / l } else { f.normal };
        t.normal = [v.dot(f.tangent), v.dot(bt), v.dot(f.normal)];
        t
    }
}

impl Layer {
    /// The mask (0..1), the tone (0..1) and, for craters, the height's own
    /// shape (−1..0.6) at `p`.
    #[allow(clippy::too_many_arguments)]
    fn mask(
        &self,
        p: DVec3,
        n: DVec3,
        bend: f64,
        edges: &Edges,
        texel: f64,
        seed: u32,
        span: (f64, f64),
    ) -> (f64, f64, Option<f64>) {
        // Half a texel's blend on every hard line, so lines are smooth.
        let aa = texel * 0.5;
        let line = |d: f64, w: f64| 1.0 - smooth(w * 0.5 - aa, w * 0.5 + aa, d);
        let shape = |v: f64| match self.threshold {
            Some(th) => {
                let s = if self.softness == 0.0 {
                    0.05
                } else {
                    self.softness
                };
                smooth(th - s, th + s, v)
            }
            None => v,
        };
        let axis = self.axis / self.axis.length();
        let (mask, tone, lift) = match self.pattern {
            Pattern::Noise | Pattern::Ridged => {
                let ridged = self.pattern == Pattern::Ridged;
                let v = fbm(p / self.scale, self.octaves, seed, ridged);
                // Fbm sits between 0.3 and 0.7 nine times in ten: stretched
                // so mottling and jitter span the whole 0..1.
                let v = if ridged {
                    v
                } else {
                    ((v - 0.5) * NOISE_STRETCH + 0.5).clamp(0.0, 1.0)
                };
                (shape(v), v, None)
            }
            Pattern::Facing => {
                let v = (n.dot(axis) + 1.0) * 0.5;
                (shape(v), v, None)
            }
            Pattern::Gradient => {
                let (lo, hi) = span;
                let v = if hi > lo {
                    ((p.dot(axis) - lo) / (hi - lo)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                (shape(v), v, None)
            }
            Pattern::Craters => {
                let c = craters(p / self.scale, seed, 0.5);
                ((-c).clamp(0.0, 1.0), (c + 1.0) / 1.6, Some(c))
            }
            Pattern::Strata => {
                let warp = 0.35 * self.scale * (fbm(p / (self.scale * 2.0), 3, seed, false) - 0.5);
                let h = (p.dot(axis) + warp) / self.scale;
                let band = h.floor() as i64;
                let tone_of = |b: i64| hash_unit(corner(b, 0, 0, seed));
                let blend = smooth(1.0 - self.width / self.scale, 1.0, frac(h));
                let v = tone_of(band) + (tone_of(band + 1) - tone_of(band)) * blend;
                (v, v, None)
            }
            Pattern::Stripes => {
                let d = (frac(p.dot(axis) / self.scale + 0.5) - 0.5).abs() * self.scale;
                (line(d, self.width), 0.5, None)
            }
            Pattern::Panels => {
                let (u, v) = project(p, n);
                let h = if self.cell == 0.0 {
                    self.scale
                } else {
                    self.cell
                };
                let (cu, cv) = ((u / self.scale).floor(), (v / h).floor());
                let fu = (u - cu * self.scale).min((cu + 1.0) * self.scale - u);
                let fv = (v - cv * h).min((cv + 1.0) * h - v);
                let tone = hash_unit(corner(cu as i64, cv as i64, 7, seed));
                (line(fu.min(fv), self.width), tone, None)
            }
            Pattern::Rivets => {
                let (u, v) = project(p, n);
                let cell = if self.cell == 0.0 {
                    self.scale * 4.0
                } else {
                    self.cell
                };
                let inset = if self.inset == 0.0 {
                    self.width
                } else {
                    self.inset
                };
                let (fu, fv) = (u - (u / cell).floor() * cell, v - (v / cell).floor() * cell);
                // The nearest rivet: one row along each panel edge.
                let along = |x: f64| {
                    let k = ((x - inset) / self.scale + 0.5).floor();
                    (inset + k * self.scale).clamp(inset, cell - inset)
                };
                let mut d = f64::INFINITY;
                for (cx, cy) in [
                    (inset, along(fv)),
                    (cell - inset, along(fv)),
                    (along(fu), inset),
                    (along(fu), cell - inset),
                ] {
                    let (dx, dy) = (fu - cx, fv - cy);
                    d = d.min((dx * dx + dy * dy).sqrt());
                }
                let r = self.width * 0.5;
                let t = (d / r).min(1.0);
                let dome = (1.0 - t * t).max(0.0).sqrt();
                (line(d, self.width), dome, Some(dome))
            }
            Pattern::Veins => {
                let v = 1.0 - (fbm(p / self.scale, self.octaves, seed, false) * 2.0 - 1.0).abs();
                let w = (self.width / self.scale).min(1.0);
                (smooth(1.0 - w, 1.0 - w * 0.25, v), v, None)
            }
            Pattern::Edges | Pattern::Cavities => {
                let convex = self.pattern == Pattern::Edges;
                let d = edges.nearest(p, convex);
                let near = 1.0 - smooth(0.0, self.width, d);
                let k = if convex { bend } else { -bend };
                let c0 = 0.25 / self.width;
                let curved = smooth(c0, c0 * 3.0, k);
                let v = near.max(curved);
                (v, v, None)
            }
        };
        let mask = if self.breakup > 0.0 {
            let size = if self.width > 0.0 {
                self.width * 4.0
            } else if self.scale > 0.0 {
                self.scale
            } else {
                1.0
            };
            let b = smooth(0.35, 0.65, fbm(p / size, 3, seed ^ 0x5bd1_e995, false));
            mask * (1.0 - self.breakup + self.breakup * b)
        } else {
            mask
        };
        (mask, tone, lift.map(|v| v * self.amount))
    }
}

/// The solid's area, square millimetres.
fn solid_area(s: &Solid) -> f64 {
    s.triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|v| s.positions[v as usize]);
            (b - a).cross(c - a).length() * 0.5
        })
        .sum()
}

/// Each (vertex, surface)'s bend per metre: the mean over its neighbours
/// within the surface of `2 n·(p − q) / |p − q|²` (a sphere of radius r
/// gives 1/r; convex positive).
fn bends(s: &Solid) -> BTreeMap<(u32, u32), f64> {
    let mut normal: BTreeMap<(u32, u32), DVec3> = BTreeMap::new();
    let mut near: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let nn = (b - a).cross(c - a);
        for k in 0..3 {
            let key = (t[k], s.surfaces[i]);
            *normal.entry(key).or_default() += nn;
            let e = near.entry(key).or_default();
            for o in [t[(k + 1) % 3], t[(k + 2) % 3]] {
                if !e.contains(&o) {
                    e.push(o);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for (key, list) in near {
        let n = normal[&key];
        let l = n.length();
        if l == 0.0 || list.is_empty() {
            continue;
        }
        let n = n / l;
        let p = s.positions[key.0 as usize];
        let mut sum = 0.0;
        for q in &list {
            let d = p - s.positions[*q as usize];
            let d2 = d.length_squared();
            if d2 > 0.0 {
                sum += 2.0 * n.dot(d) / d2;
            }
        }
        // Per millimetre to per metre.
        out.insert(key, sum / list.len() as f64 * 1000.0);
    }
    out
}

/// An edge's ends (metres) and whether it is convex.
type Edge = (DVec3, DVec3, bool);

/// The sharp edges (between two surfaces) of the painted parts, convex or
/// concave, in metres, bucketed on a grid as wide as the widest wear.
struct Edges {
    cell: f64,
    grid: BTreeMap<(i64, i64, i64), Vec<Edge>>,
}

impl Edges {
    fn new(reach: f64) -> Edges {
        Edges {
            cell: reach,
            grid: BTreeMap::new(),
        }
    }

    fn key(&self, p: DVec3) -> (i64, i64, i64) {
        let c = p / self.cell;
        (c.x.floor() as i64, c.y.floor() as i64, c.z.floor() as i64)
    }

    fn add(&mut self, s: &Solid) {
        if self.cell <= 0.0 {
            return;
        }
        let mut by_edge: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
        for (i, t) in s.triangles.iter().enumerate() {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                by_edge.entry((a.min(b), a.max(b))).or_default().push(i);
            }
        }
        let normal = |i: usize| {
            let [a, b, c] = s.triangles[i].map(|v| s.positions[v as usize]);
            (b - a).cross(c - a)
        };
        for ((a, b), faces) in by_edge {
            let [f, g] = faces[..] else { continue };
            if s.surfaces[f] == s.surfaces[g] {
                let (nf, ng) = (normal(f), normal(g));
                let l = nf.length() * ng.length();
                if l == 0.0 || nf.dot(ng) / l > CREASE_COS {
                    continue;
                }
            }
            let (pa, pb) = (s.positions[a as usize], s.positions[b as usize]);
            // Convex when the other face's far corner is behind this one.
            let far = s.triangles[g]
                .iter()
                .copied()
                .find(|&v| v != a && v != b)
                .map(|v| s.positions[v as usize]);
            let Some(far) = far else { continue };
            let convex = normal(f).dot(far - pa) < 0.0;
            let (pa, pb) = (pa / 1000.0, pb / 1000.0);
            // Bucketed in every cell its box touches.
            let (lo, hi) = (self.key(pa.min(pb)), self.key(pa.max(pb)));
            if (hi.0 - lo.0 + 1) * (hi.1 - lo.1 + 1) * (hi.2 - lo.2 + 1) > 4096 {
                // A long edge: walk it instead.
                let steps = ((pb - pa).length() / self.cell).ceil() as usize + 1;
                let mut seen = std::collections::BTreeSet::new();
                for i in 0..=steps {
                    let q = pa + (pb - pa) * (i as f64 / steps as f64);
                    seen.insert(self.key(q));
                }
                for k in seen {
                    self.grid.entry(k).or_default().push((pa, pb, convex));
                }
                continue;
            }
            for x in lo.0..=hi.0 {
                for y in lo.1..=hi.1 {
                    for z in lo.2..=hi.2 {
                        self.grid
                            .entry((x, y, z))
                            .or_default()
                            .push((pa, pb, convex));
                    }
                }
            }
        }
    }

    /// The distance to the nearest edge of the kind within the reach;
    /// infinity past it.
    fn nearest(&self, p: DVec3, convex: bool) -> f64 {
        if self.cell <= 0.0 {
            return f64::INFINITY;
        }
        let k = self.key(p);
        let mut best = f64::INFINITY;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let Some(list) = self.grid.get(&(k.0 + x, k.1 + y, k.2 + z)) else {
                        continue;
                    };
                    for &(a, b, c) in list {
                        if c != convex {
                            continue;
                        }
                        let ab = b - a;
                        let l2 = ab.length_squared();
                        let t = if l2 > 0.0 {
                            ((p - a).dot(ab) / l2).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        best = best.min((p - (a + ab * t)).length());
                    }
                }
            }
        }
        best
    }
}
