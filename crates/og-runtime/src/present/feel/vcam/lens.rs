//! Shot lens samples and brain-owned interrupted projection state.
//! A held crossfade is a projection, not an invented photographic lens.

use super::*;
use crate::host::wire::{CameraExposureMode, CameraGateFit};
use og_render::{GateFit, PhysicalLens};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Lens {
    pub physical: Option<PhysicalLens>,
    exposure_mode: CameraExposureMode,
    ev100: f32,
    pub projection: Vec3,
    pub resolved: bool,
}

impl Lens {
    pub fn read(camera: &record::Camera, aspect: f32) -> Self {
        Self {
            physical: crate::present::lens::of(camera),
            exposure_mode: camera.exposure_mode,
            ev100: camera.ev100,
            projection: crate::present::lens::facts(camera, aspect),
            resolved: camera.lens_blending,
        }
    }

    pub fn held(projection: Vec3, fov: f32) -> Self {
        let (s, c) = og_core::math::sin_cos(fov.to_radians() * 0.5);
        Self {
            physical: None,
            exposure_mode: CameraExposureMode::Artistic,
            ev100: 0.0,
            projection: if projection.x > 0.0 {
                projection
            } else {
                Vec3::new(c / s, 0.0, 0.0)
            },
            resolved: projection.x > 0.0,
        }
    }

    pub fn blend(self, to: Self, weight: f32, fov: f32, aspect: f32) -> Self {
        if weight <= 0.0 {
            return self;
        }
        if weight >= 1.0 {
            return to;
        }
        let mut out = to;
        out.ev100 = lerp(self.ev100, to.ev100, weight);
        match (self.physical, to.physical) {
            (Some(a), Some(b)) if !self.resolved && !to.resolved && a.gate_fit == b.gate_fit => {
                let lens = a.blend(b, weight);
                if let Some(focal) = lens.focal_y(aspect, 1.0) {
                    out.projection = Vec3::new(focal, lens.shift.x, lens.shift.y);
                    out.physical = Some(lens);
                    return out;
                }
            }
            (None, None) if !self.resolved && !to.resolved => {
                let (s, c) = og_core::math::sin_cos(fov.to_radians() * 0.5);
                out.projection = Vec3::new(c / s, 0.0, 0.0);
                return out;
            }
            _ => {}
        }
        out.projection = self.projection.lerp(to.projection, weight);
        out.resolved = true;
        out
    }

    pub fn write(self, camera: &mut Value) {
        camera["lensBlending"] = json!(self.resolved);
        camera["lensProjection"] = jv3(self.projection);
        camera["lensMode"] = json!(if self.physical.is_some() {
            "physical"
        } else {
            "artistic"
        });
        if let Some(p) = self.physical {
            camera["focalLength"] = json!(p.focal_length);
            camera["sensorSize"] = json!({ "x": p.sensor_size.x, "y": p.sensor_size.y });
            camera["gateFit"] = json!(
                match p.gate_fit {
                    GateFit::Vertical => CameraGateFit::Vertical,
                    GateFit::Horizontal => CameraGateFit::Horizontal,
                    GateFit::Fill => CameraGateFit::Fill,
                    GateFit::Overscan => CameraGateFit::Overscan,
                }
                .as_str()
            );
            camera["aperture"] = json!(p.aperture);
            camera["focusDistance"] = json!(p.focus_distance);
            camera["shutterSpeed"] = json!(p.shutter_speed);
            camera["iso"] = json!(p.iso);
            camera["lensShift"] = json!({ "x": p.shift.x, "y": p.shift.y });
        }
        camera["exposureMode"] = json!(self.exposure_mode.as_str());
        camera["ev100"] = json!(self.ev100);
    }

    pub fn held_projection(self) -> Vec3 {
        if self.physical.is_some() || self.resolved {
            self.projection
        } else {
            Vec3::ZERO
        }
    }
}
