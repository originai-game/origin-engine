//! Projection texturing (`og bake projection`): a level's one texture painted
//! from views an image model paints, after trompe (Patch, Apache-2.0,
//! github.com/Patch-gg/trompe), on the CPU.
//!
//! A [`Level`] is the static geometry with its material uvs and part labels.
//! [`Texels`] rasterises it in uv space: every texel of the texture its
//! world point, normal and triangle. A [`ViewSpec`] is a camera; a
//! [`GBuffer`] is what it sees, ray cast through the level's BVH, and
//! [`clay`] lights it as grey clay under an overcast sky (ray-traced
//! ambient occlusion and a soft key light, no cast shadows), so what a model
//! paints over it comes out close to albedo. [`prepare`] composites the
//! texture painted so far over the clay (the image a model is asked to
//! continue) and decides what this view may paint; [`composite::finish`]
//! puts the model's answer back beside the known texture; [`project`]
//! writes it into the [`Atlas`] wherever this view sees a texel better than
//! what is stored. [`plan`] chooses the views.
//!
//! Nothing here knows about models, prompts or money: the caller asks a
//! model for each prepared view and hands back the image.

mod atlas;
pub mod composite;
mod level;
pub mod plan;
mod prepare;
mod semantics;
mod texels;
mod view;

pub use atlas::{Paint, Projected, Texel, padded, project, silhouette};
pub use level::{Level, SurfaceHit};
pub use prepare::{Prepared, prepare, unpainted_pixels};
pub use semantics::{Classes, Description, classify, describe};
pub use texels::Texels;
pub use view::{ClayLight, GBuffer, Sample, ViewKind, ViewSpec, clay};

use std::sync::Mutex;

/// Worker threads: `n`, or the machine's when 0.
pub(crate) fn resolve_threads(n: usize) -> usize {
    if n == 0 {
        std::thread::available_parallelism().map_or(4, |n| n.get())
    } else {
        n
    }
    .max(1)
}

/// Fills `out[i]` with `f(i, &mut out[i])` on `threads` workers, in chunks
/// handed out as workers free up (rows of a frame differ in cost).
pub(crate) fn par_for<T: Send>(out: &mut [T], threads: usize, f: impl Fn(usize, &mut T) + Sync) {
    let threads = resolve_threads(threads);
    if out.is_empty() {
        return;
    }
    let chunk = out.len().div_ceil(threads * 16).max(1);
    if threads == 1 {
        for (i, v) in out.iter_mut().enumerate() {
            f(i, v);
        }
        return;
    }
    let work = Mutex::new(out.chunks_mut(chunk).enumerate());
    std::thread::scope(|s| {
        for _ in 0..threads {
            let (work, f) = (&work, &f);
            s.spawn(move || {
                loop {
                    let next = work.lock().map(|mut w| w.next()).unwrap_or(None);
                    let Some((ci, c)) = next else { break };
                    for (j, v) in c.iter_mut().enumerate() {
                        f(ci * chunk + j, v);
                    }
                }
            });
        }
    });
}
