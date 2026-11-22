//! Baking a source mesh's look into a low mesh's atlas
//! (`docs/research.md` §34).
//!
//! Entry: [`bake`]. The low mesh carries new UVs (an atlas from
//! [`crate::tools::unwrap_atlas`]); the source carries its own UVs, maps and
//! vertex colours. For every texel of the atlas a low triangle covers:
//!
//! 1. **Coverage.** A texel whose centre lies in a low triangle's UV
//!    triangle belongs to it (exact `orient2d`; a centre on an edge goes
//!    to the triangle the edge runs up or flat-left in, as `measure`'s
//!    coverage rule; on two overlapping triangles, to the lower index). A
//!    texel whose square a triangle overlaps but whose centre no triangle
//!    holds takes the triangle whose UV triangle comes nearest its centre
//!    (conservative rasterisation), at that nearest point.
//! 2. **The low point.** Its position, and the frame a viewer shades it
//!    in: the normal and tangent interpolated from the corners' (as the
//!    plain GLB writes them: `TANGENT`, see below), normalised, the
//!    tangent made square to the normal, the bitangent `w · n × t`.
//! 3. **The source point.** Along the low normal, both ways, up to
//!    `distance`: every source triangle the segment crosses (exact
//!    `orient3d` sides). A crossing counts when the source face's normal
//!    agrees with the low normal (a positive dot), so the inside of a
//!    thin wall is not taken for its outside; the nearest agreeing one is
//!    taken (ties by the side, then the triangle). With none, the nearest
//!    point of the source within `distance`, whichever way it faces. With
//!    neither, the texel has no source: it is counted, and dilation fills
//!    it from its neighbours like the padding. A box tree over the source
//!    triangles finds both (no threads).
//! 4. **Sampling** at the source point, through the source's UVs,
//!    bilinear, repeat wrap: base colour in linear light times the vertex
//!    colour (or the vertex colour alone with no map); metallic-roughness
//!    and occlusion as stored; emissive in linear light (glTF stores it as
//!    sRGB). Where the source triangles' materials differ in their
//!    factors, each texel takes its material's factors too and the baked
//!    material's are neutral (1); otherwise the factors stay on the
//!    material and the maps are baked as they are.
//! 5. **Normals.** The source's shading normal (its smooth normal, with
//!    its normal map applied in its own frame, built the same way, at its
//!    `normal_scale`) in the low frame, `c = (s·t, s·b, s·n)`, written as
//!    the bytes `255 (c + 1) / 2` rounded. Baked from the source's normals
//!    alone when it has no normal map, so a dense mesh's shape survives
//!    decimation as shading.
//! 6. **Dilation.** Layer by layer, an empty texel next (8-neighbours) to
//!    filled ones takes their mean, until `dilate` layers or every texel is
//!    filled.
//!
//! **The tangent convention.** MikkTSpace ([`crate::tools::mikk`]), from each
//! corner's position, the surface's smooth normal and its UV
//! (`surface::corner_frames`): what Blender, Unity and Unreal compute.
//! The GLB carries it as `TANGENT` ([`crate::tools::PlainGlb::tangents`]).
//!
//! f64 `+ − × /` and `sqrt` for positions, `trig::acos` for the angle
//! measures only, integer texel arithmetic and exact predicates for every
//! side: the PNG bytes are the same on every host.

use core::cmp::Ordering;
use std::fmt;

use crate::math::predicates::orient2d;
use crate::math::{DVec2, DVec3, trig};
use crate::mesh::Solid;
use crate::mesh::tree::{Tree, closest_on_triangle, tri};

use crate::tools::export::{ColorSpace, Material};
use crate::tools::image::{Codec, Texture};
use crate::tools::intersect::tris_overlap_2d;
use crate::tools::json::Json;
use crate::tools::measure::takes;
use crate::tools::preview::Image;
use crate::tools::sampler::{Linear, linear_table, to_linear};
use crate::tools::surface::{CornerFrame, corner_frames};

mod dilate;
mod raster;
mod sample;
pub(crate) use dilate::*;
pub(crate) use raster::*;
pub(crate) use sample::*;

/// How to bake. The zero value bakes 1024² maps, looks 2 % of the
/// source's diagonal either way along the low normal, fills every empty
/// texel by dilation and bakes normals.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bake {
    /// Texels across each square map; 0 is 1024. The low mesh's UVs
    /// should be packed at this size ([`crate::tools::Atlas::size`]).
    pub size: u32,
    /// How far from the low surface a source point may be, in mm; 0 is
    /// 2 % of the source's bounding-box diagonal.
    pub distance: f64,
    /// Layers of dilation past the covered texels; 0 fills every texel.
    pub dilate: u32,
    /// No normal map.
    pub no_normals: bool,
}

/// What to bake from: the source solid, its materials, and which
/// material each triangle has.
#[derive(Clone, Copy, Debug)]
pub struct BakeSource<'a> {
    /// The source, with its UVs (when it has maps) and vertex colours.
    pub solid: &'a Solid,
    /// Its materials; empty is three.js's default.
    pub materials: &'a [Material],
    /// Per triangle, its index into `materials`; empty is all 0.
    pub material_of: &'a [u32],
}

/// One map baked.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakedMap {
    /// Its kind ([`crate::tools::MAP_KINDS`]).
    pub kind: &'static str,
    /// Texels across and down.
    pub size: u32,
    /// Bytes of its PNG.
    pub file_bytes: usize,
}

/// The shading normals' difference, in degrees, over the covered texels
/// that found a source.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NormalMeasures {
    /// Between the source's shading normal and the low mesh's with the
    /// baked map (as a viewer decodes its bytes in the written frame).
    pub mean: f64,
    /// Its 95th percentile (to 0.01°).
    pub p95: f64,
    /// Between the source's shading normal and the low mesh's own
    /// normal, without a map.
    pub mean_without: f64,
    /// Its 95th percentile.
    pub p95_without: f64,
}

/// What a bake did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakeReport {
    /// Texels across.
    pub size: u32,
    /// The search distance used, mm.
    pub distance: f64,
    /// Texels a low triangle covers (centre inside, or square overlapped).
    pub covered: usize,
    /// Of those, the ones whose centre no triangle holds (conservative).
    pub conservative: usize,
    /// Covered texels sampled where the normal line crossed the source.
    pub hits: usize,
    /// Covered texels sampled at the nearest source point instead (no
    /// agreeing crossing within the distance).
    pub nearest: usize,
    /// Covered texels with no source within the distance: filled by
    /// dilation, like the padding.
    pub no_source: usize,
    /// Texels where the nearest crossing faced away from the low normal,
    /// so a farther one (or the nearest point) was taken.
    pub facing_away: usize,
    /// Texels with agreeing crossings on both sides of the low surface
    /// (two sheets near): the nearer was taken.
    pub ambiguous: usize,
    /// The largest distance from a texel's low point to its source point,
    /// mm.
    pub largest_distance: f64,
    /// Texels filled by dilation (each map the same).
    pub dilated: usize,
    /// Texels left empty (dilation stopped short): transparent black.
    pub empty: usize,
    /// The maps written, in [`crate::tools::MAP_KINDS`] order.
    pub maps: Vec<BakedMap>,
    /// The source's factors went into the texels (its materials differ).
    pub factors_baked: bool,
    /// The source's vertex colours went into the base-colour map: the low
    /// mesh must not carry them as well.
    pub colors_baked: bool,
    /// What could not be baked, and why.
    pub not_baked: Vec<String>,
    /// The normal map's measures, when one was baked.
    pub normals: Option<NormalMeasures>,
}

/// The baked material and what was done.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Baked {
    /// The material for the low mesh: the source's factors (or neutral
    /// ones when they were baked into the texels) and the baked maps, as
    /// PNG.
    pub material: Material,
    /// The report.
    pub report: BakeReport,
}

/// Why a bake could not run.
#[derive(Clone, Debug, PartialEq)]
pub enum BakeError {
    /// A size, distance or index out of range (which).
    BadDesc(String),
    /// The low mesh has no UVs to bake into.
    NoUvs,
    /// The source or the low mesh has no triangles.
    Empty,
}

impl fmt::Display for BakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BakeError::BadDesc(w) => write!(f, "bake: {w}"),
            BakeError::NoUvs => f.write_str("bake: the low mesh has no UVs to bake into"),
            BakeError::Empty => f.write_str("bake: no triangles"),
        }
    }
}

impl std::error::Error for BakeError {}

// ── Maps ──────────────────────────────────────────────────────────────────

/// Bakes `src`'s look into `low`'s UV atlas (the module's rule). The low
/// mesh is not changed: when the report says `colors_baked`, write it
/// without its vertex colours, and with `TANGENT` when a normal map was
/// baked.
pub fn bake(src: &BakeSource, low: &Solid, d: &Bake, codec: &Codec) -> Result<Baked, BakeError> {
    let size = if d.size == 0 { 1024 } else { d.size };
    if size > 8192 {
        return Err(BakeError::BadDesc("size must be at most 8192".into()));
    }
    if !(d.distance >= 0.0 && d.distance.is_finite()) {
        return Err(BakeError::BadDesc("distance must be 0 or positive".into()));
    }
    let s = src.solid;
    if s.triangles.is_empty() || low.triangles.is_empty() {
        return Err(BakeError::Empty);
    }
    let Some(low_uv) = low.uvs.as_ref() else {
        return Err(BakeError::NoUvs);
    };
    if low_uv.len() != low.triangles.len() {
        return Err(BakeError::BadDesc(
            "the low mesh's UVs are for another number of triangles".into(),
        ));
    }
    let default = [Material::default()];
    let materials: &[Material] = if src.materials.is_empty() {
        &default
    } else {
        src.materials
    };
    if !src.material_of.is_empty() && src.material_of.len() != s.triangles.len() {
        return Err(BakeError::BadDesc(
            "material_of must have one entry per source triangle".into(),
        ));
    }
    if src
        .material_of
        .iter()
        .any(|&m| m as usize >= materials.len())
    {
        return Err(BakeError::BadDesc(
            "a material index is out of range".into(),
        ));
    }
    let mat_of = |i: usize| src.material_of.get(i).map_or(0, |&m| m as usize);
    let used: Vec<usize> = {
        let mut u: Vec<usize> = (0..s.triangles.len()).map(mat_of).collect();
        u.sort_unstable();
        u.dedup();
        u
    };
    let fold = used
        .iter()
        .any(|&m| bare(&materials[m]) != bare(&materials[used[0]]));
    let first = &materials[used[0]];
    let mut report = BakeReport {
        size,
        factors_baked: fold,
        ..Default::default()
    };
    // Decode each material's maps; a map that cannot be decoded is left
    // out, and so is its kind (it is not guessed from the others).
    let src_uv = s.uvs.as_ref().filter(|u| u.len() == s.triangles.len());
    let mut decoded: Vec<[Option<Map>; 5]> = Vec::with_capacity(materials.len());
    let mut kind_failed = [false; 5];
    for (mi, m) in materials.iter().enumerate() {
        let slots = m.map_slots();
        let mut row: [Option<Map>; 5] = Default::default();
        for (k, (kind, t)) in slots.iter().enumerate() {
            let Some(t) = t else { continue };
            if !used.contains(&mi) {
                continue;
            }
            if src_uv.is_none() {
                report.not_baked.push(format!(
                    "{kind} {:?}: the source has no UVs to read it through",
                    t.name
                ));
                kind_failed[k] = true;
                continue;
            }
            match (codec.decode)(&t.bytes) {
                Ok(image) => {
                    row[k] = Some(Map {
                        image,
                        color: Material::is_color_map(kind),
                    });
                }
                Err(e) => {
                    report.not_baked.push(format!("{kind} {:?}: {e}", t.name));
                    kind_failed[k] = true;
                }
            }
        }
        decoded.push(row);
    }
    let any_map = |k: usize| used.iter().any(|&m| decoded[m][k].is_some());
    let colors = s.colors.as_ref().filter(|c| c.len() == s.triangles.len());
    // Which maps to make: [base, metallic-roughness, normal, emissive, ao].
    let make = [
        !kind_failed[0] && (any_map(0) || colors.is_some() || fold),
        !kind_failed[1] && (any_map(1) || fold),
        !d.no_normals,
        !kind_failed[3]
            && (any_map(3) || (fold && used.iter().any(|&m| materials[m].emissive != [0.0; 3]))),
        !kind_failed[4] && any_map(4),
    ];
    for (k, failed) in kind_failed.iter().enumerate() {
        if *failed && k != 2 {
            report.not_baked.push(format!(
                "{}: not baked; the low mesh's material has none",
                crate::tools::export::MAP_KINDS[k]
            ));
        }
    }
    if kind_failed[2] && make[2] {
        report.not_baked.push(
            "normal_map: the source's normal map is not applied; normals baked from its geometry"
                .into(),
        );
    }
    report.colors_baked = make[0] && colors.is_some();
    let fac: Vec<Factors> = materials.iter().map(factors).collect();
    // The search distance.
    let (mut lo, mut hi) = (DVec3::INFINITY, DVec3::NEG_INFINITY);
    for p in &s.positions {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let reach = if d.distance > 0.0 {
        d.distance
    } else {
        (hi - lo).length() * 0.02
    };
    report.distance = reach;
    let tree = Tree::new(s);
    let src_frames = corner_frames(s);
    let low_frames = corner_frames(low);
    let lin = linear_table();
    let n = size as usize;
    let mut images: Vec<Option<Vec<u8>>> = make
        .iter()
        .map(|&m| m.then(|| vec![0u8; n * n * 4]))
        .collect();
    // Owner of each texel, and its point in the owner's UV triangle.
    let (owner, at) = rasterize(low_uv, size);
    let mut filled = vec![false; n * n];
    let mut hist = vec![0u32; 18001];
    let mut hist_without = vec![0u32; 18001];
    let (mut sum, mut sum_without, mut count) = (0.0, 0.0, 0usize);
    let mut hits: Vec<(u32, f64)> = Vec::new();
    for texel in 0..n * n {
        let li = owner[texel];
        if li == u32::MAX {
            continue;
        }
        report.covered += 1;
        let (b, conservative) = at[texel];
        if conservative {
            report.conservative += 1;
        }
        let lt = tri(low, li);
        let p = lt[0] * b[0] + lt[1] * b[1] + lt[2] * b[2];
        let lf = frame_at(&low_frames[li as usize], b);
        if lf.normal == DVec3::ZERO {
            report.no_source += 1;
            continue;
        }
        // The source point.
        let (a, z) = (p - lf.normal * reach, p + lf.normal * reach);
        tree.crossings(a, z, &mut hits);
        let mut best: Option<(f64, f64, u32)> = None;
        let mut nearest_any: Option<(f64, bool)> = None;
        let (mut front, mut back) = (false, false);
        for &(j, t) in &hits {
            let sd = (t * 2.0 - 1.0) * reach;
            let [q0, q1, q2] = tri(s, j);
            let agree = (q1 - q0).cross(q2 - q0).dot(lf.normal) > 0.0;
            if nearest_any.is_none_or(|(d0, _)| sd.abs() < d0) {
                nearest_any = Some((sd.abs(), agree));
            }
            if !agree {
                continue;
            }
            if sd > 0.0 {
                front = true;
            } else if sd < 0.0 {
                back = true;
            }
            let key = (sd.abs(), sd, j);
            if best.is_none_or(|bb| {
                (key.0, key.1).partial_cmp(&(bb.0, bb.1)) == Some(Ordering::Less)
                    || (key.0 == bb.0 && key.1 == bb.1 && key.2 < bb.2)
            }) {
                best = Some(key);
            }
        }
        if nearest_any.is_some_and(|(d0, agree)| !agree && best.is_none_or(|bb| d0 < bb.0)) {
            report.facing_away += 1;
        }
        if front && back {
            report.ambiguous += 1;
        }
        let (j, q, dist) = if let Some((dd, sd, j)) = best {
            report.hits += 1;
            (j, p + lf.normal * sd, dd)
        } else if let Some((j, q, d2)) = tree.nearest(p, reach) {
            report.nearest += 1;
            (j, q, d2.sqrt())
        } else {
            report.no_source += 1;
            continue;
        };
        report.largest_distance = report.largest_distance.max(dist);
        filled[texel] = true;
        let st = tri(s, j);
        let w = bary3(q, st);
        let ji = j as usize;
        let mi = mat_of(ji);
        let f = &fac[mi];
        let uv = src_uv.map(|u| u[ji][0] * w[0] + u[ji][1] * w[1] + u[ji][2] * w[2]);
        let sample = |k: usize| -> Option<[f64; 4]> {
            let m = decoded[mi][k].as_ref()?;
            Some(m.sample(&lin, uv?))
        };
        let px = {
            let (x, y) = (texel % n, texel / n);
            ((n - 1 - y) * n + x) * 4
        };
        if let Some(img) = &mut images[0] {
            let mut c = sample(0).unwrap_or([1.0; 4]);
            if let Some(col) = colors {
                let cc = col[ji].map(crate::mesh::wide);
                for (k, v) in c.iter_mut().enumerate() {
                    *v *= cc[0][k] * w[0] + cc[1][k] * w[1] + cc[2][k] * w[2];
                }
            }
            if fold {
                for (k, v) in c.iter_mut().enumerate() {
                    *v *= f.color[k];
                }
            }
            for k in 0..3 {
                img[px + k] = srgb_byte(&lin, c[k]);
            }
            img[px + 3] = data_byte(c[3]);
        }
        if let Some(img) = &mut images[1] {
            let mut c = sample(1).unwrap_or([1.0; 4]);
            if fold {
                c[1] *= f.roughness;
                c[2] *= f.metalness;
            }
            for k in 0..4 {
                img[px + k] = data_byte(c[k]);
            }
        }
        if let Some(img) = &mut images[3] {
            let mut c = sample(3).unwrap_or([1.0; 4]);
            if fold {
                for (k, v) in c.iter_mut().take(3).enumerate() {
                    *v *= f.emissive[k];
                }
            }
            for k in 0..3 {
                img[px + k] = srgb_byte(&lin, c[k]);
            }
            img[px + 3] = 255;
        }
        if let Some(img) = &mut images[4] {
            let mut c = sample(4).unwrap_or([1.0; 4]);
            if fold {
                c[0] = 1.0 + f.ao * (c[0] - 1.0);
            }
            for k in 0..4 {
                img[px + k] = data_byte(c[k]);
            }
        }
        if let Some(img) = &mut images[2] {
            let sf = frame_at(&src_frames[ji], w);
            let mut shade = sf.normal;
            if let Some(m) = sample(2) {
                let x = (m[0] * 2.0 - 1.0) * f.normal_scale;
                let y = (m[1] * 2.0 - 1.0) * f.normal_scale;
                let z = m[2] * 2.0 - 1.0;
                let bt = sf.normal.cross(sf.tangent) * sf.w;
                let v = unit(sf.tangent * x + bt * y + sf.normal * z);
                if v != DVec3::ZERO {
                    shade = v;
                }
            }
            if shade == DVec3::ZERO {
                shade = lf.normal;
            }
            let lb = lf.normal.cross(lf.tangent) * lf.w;
            let c = [shade.dot(lf.tangent), shade.dot(lb), shade.dot(lf.normal)];
            let bytes = c.map(|v| data_byte((v + 1.0) / 2.0));
            img[px..px + 3].copy_from_slice(&bytes);
            img[px + 3] = 255;
            // As a viewer decodes it, in the written frame.
            let dec = bytes.map(|v| f64::from(v) * 2.0 / 255.0 - 1.0);
            let seen = unit(lf.tangent * dec[0] + lb * dec[1] + lf.normal * dec[2]);
            let ang = |a: DVec3, b: DVec3| trig::to_degrees(trig::acos(a.dot(b).clamp(-1.0, 1.0)));
            let (e, e0) = (ang(seen, shade), ang(lf.normal, shade));
            sum += e;
            sum_without += e0;
            count += 1;
            hist[((e * 100.0 + 0.5).floor() as usize).min(18000)] += 1;
            hist_without[((e0 * 100.0 + 0.5).floor() as usize).min(18000)] += 1;
        }
    }
    if make[2] && count > 0 {
        let p95 = |h: &[u32]| {
            let need = (count as u64 * 95).div_ceil(100);
            let mut acc = 0u64;
            for (k, &c) in h.iter().enumerate() {
                acc += u64::from(c);
                if acc >= need {
                    return k as f64 / 100.0;
                }
            }
            180.0
        };
        report.normals = Some(NormalMeasures {
            mean: sum / count as f64,
            p95: p95(&hist),
            mean_without: sum_without / count as f64,
            p95_without: p95(&hist_without),
        });
    }
    // Dilation, the same layers for every map.
    let (level, layers) = dilation(&filled, n, d.dilate);
    report.dilated = layers.iter().map(Vec::len).sum();
    report.empty = n * n - filled.iter().filter(|&&f| f).count() - report.dilated;
    let mut material = if fold {
        Material {
            color: None,
            color_space: ColorSpace::Linear,
            metalness: 1.0,
            roughness: Some(1.0),
            emissive: if make[3] { [1.0; 3] } else { [0.0; 3] },
            emissive_intensity: None,
            ao_map_intensity: None,
            ..bare(first)
        }
    } else {
        bare(first)
    };
    material.normal_scale = None;
    for (k, img) in images.into_iter().enumerate() {
        let Some(mut rgba) = img else { continue };
        dilate(&mut rgba, n, &level, &layers);
        let kind = crate::tools::export::MAP_KINDS[k];
        let png = (codec.png)(&Image {
            width: size,
            height: size,
            rgba,
        });
        report.maps.push(BakedMap {
            kind,
            size,
            file_bytes: png.len(),
        });
        let t = Texture {
            name: format!("baked_{kind}.png"),
            mime: "image/png".into(),
            bytes: png,
        };
        match k {
            0 => material.map = Some(t),
            1 => material.metalness_map = Some(t),
            2 => material.normal_map = Some(t),
            3 => material.emissive_map = Some(t),
            _ => material.ao_map = Some(t),
        }
    }
    Ok(Baked { material, report })
}

impl BakeReport {
    /// The report as JSON with keys in a fixed order.
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        self.json().write(&mut s);
        s
    }

    pub(crate) fn json(&self) -> Json {
        let maps = self
            .maps
            .iter()
            .map(|m| {
                Json::Obj(vec![
                    ("kind", Json::Str(m.kind.into())),
                    ("size", Json::Int(u64::from(m.size))),
                    ("file_bytes", Json::Int(m.file_bytes as u64)),
                ])
            })
            .collect();
        let normals = self.normals.map_or(Json::Null, |m| {
            Json::Obj(vec![
                ("mean_degrees", Json::Num(m.mean)),
                ("p95_degrees", Json::Num(m.p95)),
                ("mean_degrees_without_map", Json::Num(m.mean_without)),
                ("p95_degrees_without_map", Json::Num(m.p95_without)),
            ])
        });
        Json::Obj(vec![
            ("size", Json::Int(u64::from(self.size))),
            ("distance", Json::Num(self.distance)),
            ("texels_covered", Json::Int(self.covered as u64)),
            ("conservative", Json::Int(self.conservative as u64)),
            ("hits", Json::Int(self.hits as u64)),
            ("nearest", Json::Int(self.nearest as u64)),
            ("no_source", Json::Int(self.no_source as u64)),
            ("facing_away", Json::Int(self.facing_away as u64)),
            ("ambiguous", Json::Int(self.ambiguous as u64)),
            ("largest_distance", Json::Num(self.largest_distance)),
            ("dilated", Json::Int(self.dilated as u64)),
            ("empty", Json::Int(self.empty as u64)),
            ("maps", Json::Arr(maps)),
            ("factors_baked", Json::Bool(self.factors_baked)),
            ("colors_baked", Json::Bool(self.colors_baked)),
            (
                "not_baked",
                Json::Arr(
                    self.not_baked
                        .iter()
                        .map(|w| Json::Str(w.clone()))
                        .collect(),
                ),
            ),
            ("normals", normals),
        ])
    }
}

#[cfg(test)]
mod tests;
