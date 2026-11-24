//! Resolved shot shutter. Mixed units stay separate through interruptions.

use super::*;
use crate::present::lens::post as resolve;
use glam::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Post {
    pub shutter: Vec2,
    pub physical: bool,
}

impl Post {
    pub fn read(camera: &record::Camera, artistic_shutter: f32) -> Self {
        Self {
            shutter: resolve::shutter(camera, artistic_shutter),
            physical: camera.post_blending
                || camera.lens_mode == crate::host::wire::CameraLensMode::Physical,
        }
    }

    pub fn held(shutter: Vec2, physical: bool) -> Self {
        Self { shutter, physical }
    }

    pub fn held_values(self) -> Value {
        json!({"x": self.shutter.x, "y": self.shutter.y})
    }

    pub fn blend(self, to: Self, w: f32) -> Self {
        if w <= 0.0 {
            return self;
        }
        if w >= 1.0 {
            return to;
        }
        Self {
            shutter: self.shutter.lerp(to.shutter, w),
            physical: self.physical || to.physical,
        }
    }

    pub fn write(self, camera: &mut Value, blending: bool) {
        camera["postBlending"] = json!(blending && self.physical);
        camera["postShutter"] = self.held_values();
    }
}
