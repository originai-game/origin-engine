//! One view rendered against the texture painted so far (trompe's
//! `prepareView`): the state image a model is asked to continue, which
//! pixels it may paint, and what they show.

use super::atlas::Paint;
use super::composite::{ViewMasks, dilate};
use super::semantics::{classify, describe};
use super::{Description, GBuffer, Level, Texels, ViewKind};

/// What a view asks for.
pub struct Prepared {
    /// The clay with every known texel's colour composited in (RGBA).
    pub state: Vec<[u8; 4]>,
    pub masks: ViewMasks,
    /// 1 where the model should paint (unknown geometry grown 3 px at 1024).
    pub edit: Vec<u8>,
    /// Only these pixels project (roof and fill passes, roofs left to their
    /// own pass); every geometry pixel when `None`.
    pub keep: Option<Vec<u8>>,
    /// Unpainted share of what this pass may paint.
    pub unknown_fraction: f32,
    /// `first` (nothing in view painted yet), `fill` or `roof`.
    pub mode: &'static str,
    /// Nothing worth a model call.
    pub skip: bool,
    pub description: Description,
}

/// A texel is known in a view when it was captured at no worse than
/// `ratio` × this view's quality, so a much better view repaints it.
fn known_at(
    texels: &Texels,
    paint: &Paint,
    g: &GBuffer,
    i: usize,
    ratio: f32,
) -> (bool, bool, [u8; 3]) {
    let Some(s) = g.sample(i) else {
        return (false, false, [0; 3]);
    };
    let Some(k) = texels.at_uv(s.hit.uv) else {
        return (false, false, [0; 3]);
    };
    let t = paint.texels[k];
    let painted = t.quality > 0.0;
    (painted && t.quality >= ratio * s.quality, painted, t.rgb)
}

/// Pixels of `g` whose texel nothing has painted.
pub fn unpainted_pixels(texels: &Texels, paint: &Paint, g: &GBuffer) -> usize {
    (0..g.hits.len())
        .filter(|&i| g.hits[i].is_some() && !known_at(texels, paint, g, i, 0.0).1)
        .count()
}

#[allow(clippy::too_many_arguments)]
pub fn prepare(
    level: &Level,
    texels: &Texels,
    paint: &Paint,
    g: &GBuffer,
    clay: &[[u8; 3]],
    ratio: f32,
    start_of_run: bool,
    roofs_by_pass: bool,
) -> Prepared {
    let n = g.hits.len();
    let size = g.size as usize;
    let fill_pass = g.view.kind == ViewKind::Fill;
    let roof_pass = g.view.kind == ViewKind::Roof;
    // Fill views keep everything painted, however poor, as reference.
    let r = if fill_pass { 0.0 } else { ratio };
    let mut state = Vec::with_capacity(n);
    let mut geometry = vec![0u8; n];
    let mut known = vec![0u8; n];
    let mut painted = vec![0u8; n];
    for i in 0..n {
        let (k, p, rgb) = known_at(texels, paint, g, i, r);
        geometry[i] = g.hits[i].is_some() as u8;
        known[i] = k as u8;
        painted[i] = p as u8;
        let c = if k { rgb } else { clay[i] };
        state.push([c[0], c[1], c[2], 255]);
    }
    let cls = classify(g, &level.labels, level.min.y);
    let mut keep = None;
    if roof_pass {
        // Roofs only: unpainted walls and ground are neither targets nor references.
        for i in 0..n {
            geometry[i] &= known[i] | cls.roof[i];
        }
        keep = Some(cls.roof.clone());
    } else if fill_pass {
        let holes: Vec<u8> = (0..n).map(|i| geometry[i] & (1 - painted[i])).collect();
        known = (0..n).map(|i| geometry[i] & painted[i]).collect();
        keep = Some(dilate(&holes, size, size, 2));
    } else if roofs_by_pass {
        let all = geometry.clone();
        for i in 0..n {
            geometry[i] &= known[i] | (1 - cls.roof[i]);
        }
        keep = Some((0..n).map(|i| all[i] & (1 - cls.roof[i])).collect());
    }
    let masks = ViewMasks { geometry, known };
    let unknown = masks.unknown();
    let of = if roof_pass {
        &cls.roof
    } else {
        &masks.geometry
    };
    let total = of.iter().filter(|&&v| v != 0).count();
    let unk = (0..n).filter(|&i| of[i] != 0 && unknown[i] != 0).count();
    let unknown_fraction = if total > 0 {
        unk as f32 / total as f32
    } else {
        0.0
    };
    let first = !roof_pass && !fill_pass && (start_of_run || unknown_fraction > 0.97);
    let skip = if fill_pass {
        unk == 0
    } else {
        !first && unknown_fraction < 0.03
    };
    let grow = (3.0 * size as f32 / 1024.0).round() as usize;
    let edit: Vec<u8> = dilate(&unknown, size, size, grow)
        .iter()
        .zip(&masks.geometry)
        .map(|(&e, &g)| e & g)
        .collect();
    let description = describe(
        &cls,
        &level.labels,
        g.size,
        if first { None } else { Some(&edit) },
    );
    Prepared {
        state,
        masks,
        edit,
        keep,
        unknown_fraction,
        mode: if roof_pass {
            "roof"
        } else if first {
            "first"
        } else {
            "fill"
        },
        skip,
        description,
    }
}
