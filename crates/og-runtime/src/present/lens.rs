//! Convert the reflected lens once, shared by tick queries and extraction.
//! Optics stay in render-data; this file only joins record and render types.

use crate::host::wire::{CameraGateFit, CameraLensMode, record};
use glam::{Vec2, Vec3};
use og_render::{GateFit, PhysicalLens};

pub(crate) mod coc;
pub(crate) mod post;

pub(crate) fn of(c: &record::Camera) -> Option<PhysicalLens> {
    (c.lens_mode == CameraLensMode::Physical).then_some(PhysicalLens {
        focal_length: c.focal_length,
        sensor_size: c.sensor_size,
        gate_fit: match c.gate_fit {
            CameraGateFit::Vertical => GateFit::Vertical,
            CameraGateFit::Horizontal => GateFit::Horizontal,
            CameraGateFit::Fill => GateFit::Fill,
            CameraGateFit::Overscan => GateFit::Overscan,
        },
        aperture: c.aperture,
        focus_distance: c.focus_distance,
        shutter_speed: c.shutter_speed,
        iso: c.iso,
        shift: c.lens_shift,
    })
}

/// Focal factor and shift before zoom. Held blends use these resolved
/// values, so an interrupted blend need not reconstruct a fictitious lens.
pub(crate) fn facts(c: &record::Camera, aspect: f32) -> Vec3 {
    if c.lens_blending && c.lens_projection.x > 0.0 {
        return c.lens_projection;
    }
    if let Some(lens) = of(c)
        && let Some(focal) = lens.focal_y(aspect, 1.0)
    {
        return Vec3::new(focal, lens.shift.x, lens.shift.y);
    }
    let (s, cos) = og_core::math::sin_cos(c.fov.to_radians() * 0.5);
    Vec3::new(cos / s, 0.0, 0.0)
}

/// None preserves the existing artistic constructor (including pinned
/// query bits). Orthographic and explicit secondary views stay independent.
pub(crate) fn projection(c: &record::Camera, aspect: f32) -> Option<og_render::Projection> {
    if c.orthographic || (!c.lens_blending && of(c)?.focal_y(aspect, 1.0).is_none()) {
        return None;
    }
    let p = facts(c, aspect);
    Some(og_render::Projection::PerspectiveShifted {
        fov_y_degrees: (2.0 * og_core::math::atan(1.0 / (p.x * c.zoom.max(0.01)))).to_degrees(),
        shift: Vec2::new(p.y, p.z),
    })
}

/// The authored view, before display interpolation or the render-origin shift.
/// Tick motion samples and scene extraction must resolve the same projection.
#[allow(clippy::disallowed_methods)]
pub(crate) fn view(c: &record::Camera, aspect: f32) -> og_render::Camera {
    let (position, rotation) = super::interpolate::camera_pose(c);
    let zoom = c.zoom.max(0.01);
    let projection = if c.orthographic {
        og_render::Projection::Orthographic {
            height: c.height / zoom,
        }
    } else if let Some(projection) = projection(c, aspect) {
        projection
    } else {
        let half = (c.fov.to_radians() / 2.0).tan() / zoom;
        og_render::Projection::Perspective {
            fov_y_degrees: (2.0 * half.atan()).to_degrees(),
        }
    };
    og_render::Camera {
        position,
        target: position + og_core::facing::camera_forward(rotation),
        up: og_core::facing::up(rotation),
        near: c.near,
        far: c.far,
        projection,
        exposure: Default::default(),
    }
}

pub(crate) fn status(c: &record::Camera, aspect: f32) -> &'static str {
    if c.orthographic {
        "orthographic"
    } else if projection(c, aspect).is_some() {
        "physical"
    } else if c.lens_mode == CameraLensMode::Physical {
        "invalid"
    } else {
        "artistic"
    }
}
