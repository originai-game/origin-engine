//! Generators whose output is checked here, where `shape-bool`'s exact
//! tests are in reach: [`loft`].
//!
//! A loft on a curve (`steps` > 1) can overshoot between sections and pass
//! its surface through itself; `crate::generators::paths::loft` builds whatever
//! the curve gives. [`loft`] tests the result with
//! [`crate::boolean::selfcheck::self_crossings`] and refuses a folded one,
//! naming the sections it folds between.

use crate::boolean::selfcheck::self_crossings;
use crate::generators::paths::{Loft, LoftAlong};
use crate::generators::solids::GenError;
use crate::mesh::Solid;

use crate::model::error::ModelError;

/// [`crate::generators::paths::loft`], refused with [`GenError::Invalid`] when the
/// surface passes through itself: the message names the lowest pair of
/// sections a crossing lies between, and how many triangle pairs cross.
pub fn loft(d: &Loft) -> Result<Solid, ModelError> {
    loft_with(d, false)
}

/// [`loft`], with UVs when `uvs` (`crate::generators::solids::textured`).
pub fn loft_with(d: &Loft, uvs: bool) -> Result<Solid, ModelError> {
    let s = if uvs {
        crate::generators::solids::textured(&crate::generators::solids::Generator::Loft(d))?
    } else {
        crate::generators::paths::loft(d)?
    };
    let pairs = self_crossings(&s.positions, &s.triangles);
    let Some(&(i, j)) = pairs.first() else {
        return Ok(s);
    };
    let heights: Vec<f64> = d.sections.iter().map(|x| x.1).collect();
    // The span a triangle lies in: the last section at or below its lowest
    // point (the top section's own plane belongs to the span below it).
    let span = |t: u32| {
        let lo = s.triangles[t as usize]
            .iter()
            .map(|&v| s.positions[v as usize].z)
            .fold(f64::INFINITY, f64::min);
        let k = heights.iter().rposition(|&h| h <= lo).unwrap_or(0);
        k.min(heights.len() - 2)
    };
    let low = pairs
        .iter()
        .map(|&(a, b)| span(a).min(span(b)))
        .min()
        .unwrap_or(span(i).min(span(j)));
    Err(ModelError::Gen(GenError::Invalid(format!(
        "loft: the surface passes through itself between sections {low} and {} \
         (z {} to {}; {} crossing triangle pairs): the curve overshoots there. \
         Add a section between them, bring the sizes closer, or set steps to 1 \
         (straight)",
        low + 1,
        heights[low],
        heights[low + 1],
        pairs.len()
    ))))
}

/// [`crate::generators::paths::loft_along`], with UVs when `uvs`, refused with
/// [`GenError::Invalid`] when the surface passes through itself (a bend
/// tighter than the sections are wide, or frames that twist against each
/// other), with how many triangle pairs cross.
pub fn loft_along_with(d: &LoftAlong, uvs: bool) -> Result<Solid, ModelError> {
    let s = if uvs {
        crate::generators::solids::textured(&crate::generators::solids::Generator::LoftAlong(d))?
    } else {
        crate::generators::paths::loft_along(d)?
    };
    let pairs = self_crossings(&s.positions, &s.triangles);
    if pairs.is_empty() {
        return Ok(s);
    }
    Err(ModelError::Gen(GenError::Invalid(format!(
        "loft: the surface passes through itself ({} crossing triangle pairs): the path \
         bends tighter than the sections are wide, or the frames' x axes turn against \
         each other. Open the bend, add a section, or line the frames up",
        pairs.len()
    ))))
}
