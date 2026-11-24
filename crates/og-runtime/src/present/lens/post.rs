//! Resolve shutter units without editing artistic inputs or simulation state.

use crate::host::wire::{CameraLensMode, record};
use glam::Vec2;
use og_render::PostEffects;

/// Keep seconds and frame fraction separate through interrupted blends:
/// the vector interval is only known when the frame is extracted.
pub(crate) fn shutter(c: &record::Camera, artistic: f32) -> Vec2 {
    if c.post_blending {
        c.post_shutter
    } else if c.lens_mode == CameraLensMode::Physical {
        Vec2::new(c.shutter_speed.max(0.0), 0.0)
    } else {
        Vec2::new(0.0, artistic)
    }
}

pub(crate) fn apply(c: &record::Camera, post: &mut PostEffects, interval: f32) {
    if (c.lens_mode == CameraLensMode::Physical || c.post_blending)
        && let Some(motion) = &mut post.motion_blur
    {
        let s = shutter(c, motion.shutter);
        motion.shutter = if interval > 0.0 {
            s.x / interval + s.y
        } else {
            0.0
        };
    }
}
