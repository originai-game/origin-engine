//! Reflected CoC source values at the runtime/render boundary. No history:
//! CameraBrain and Camera carry the complete mix through snapshots.

use crate::host::wire::{CameraLensMode, record};
use og_render::{COC_SOURCE_CAPACITY, CocMix, CocSource, WeightedCoc};

const _: () = assert!(COC_SOURCE_CAPACITY == og_api::consts::MAX_CAMERA_COC_SOURCES as usize);

pub(crate) fn source(c: &record::Camera, p: &record::PostEffects, aspect: f32) -> CocSource {
    if !c.orthographic
        && let Some(lens) = super::of(c)
    {
        return CocSource::physical(lens, aspect, c.zoom.max(0.01), p.dof_max_blur);
    }
    if c.lens_mode == CameraLensMode::Physical {
        return CocSource::default();
    }
    CocSource {
        focus_distance: p.dof_focus_distance,
        focus_range: p.dof_focus_range,
        falloff: p.dof_falloff,
        max_blur: p.dof_max_blur,
        tilt_shift: p.dof_tilt_shift,
        ..Default::default()
    }
}

pub(crate) fn read(values: &[f32]) -> CocMix {
    let mut mix = CocMix::default();
    for (slot, s) in mix.sources.iter_mut().zip(values.as_chunks::<9>().0) {
        *slot = WeightedCoc {
            source: CocSource {
                focal_length: s[0],
                gate_height: s[1],
                aperture: s[2],
                focus_distance: s[3],
                focus_range: s[4],
                falloff: s[5],
                max_blur: s[6],
                tilt_shift: s[7],
            },
            weight: s[8],
        };
    }
    mix
}

pub(crate) fn entry(s: WeightedCoc) -> [f32; 9] {
    let c = s.source;
    [
        c.focal_length,
        c.gate_height,
        c.aperture,
        c.focus_distance,
        c.focus_range,
        c.falloff,
        c.max_blur,
        c.tilt_shift,
        s.weight,
    ]
}

pub(crate) fn write(mix: CocMix) -> Vec<f32> {
    mix.sources
        .into_iter()
        .filter(|s| s.weight > 0.0)
        .flat_map(entry)
        .collect()
}
