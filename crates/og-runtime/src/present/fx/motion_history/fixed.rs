//! Capture motion comes from the tick-owned hierarchy and optical view,
//! not the last extraction (which can be interpolated, or several ticks old).

use super::*;

pub(super) struct Sample {
    pub camera: Option<og_render::Camera>,
    pub transforms: Vec<Mat4>,
    pub seconds: f32,
}

pub(super) fn capture(
    rs: &mut RenderState,
    h: &mut Hierarchy,
    keys: &[(u64, u32)],
    world: &World,
) -> Sample {
    let tick = world.time().tick;
    let view = rs
        .interp
        .tick_view(tick)
        .filter(|_| rs.geo.record_tick() == tick.checked_sub(1));
    let zero = crate::world::origin::zero(world, rs.draw.origin.as_ref());
    let delta = view.map_or(glam::DVec3::ZERO, |(_, before)| before - zero);
    let camera = view.map(|(mut camera, _)| {
        camera.position = (camera.position.as_dvec3() + delta).as_vec3();
        camera.target = (camera.target.as_dvec3() + delta).as_vec3();
        camera
    });
    let mut previous = rs.geo.take_hierarchy();
    let transforms = keys
        .iter()
        .zip(&rs.scene.objects.instances)
        .map(|(&(e, _), instance)| {
            if camera.is_none() || !previous.knows(e) || !h.knows(e) {
                return instance.transform;
            }
            let current = h.world_of(e);
            // No inverse exists for a collapsed root. It has no visible
            // rigid motion to reconstruct in the current view.
            if current.determinant() == 0.0 {
                return instance.transform;
            }
            let mut root = previous.world_of(e);
            root.w_axis = (root.w_axis.truncate().as_dvec3() + delta)
                .as_vec3()
                .extend(root.w_axis.w);
            root * current.inverse() * instance.transform
        })
        .collect();
    rs.geo.put_hierarchy(previous);
    Sample {
        camera,
        transforms,
        seconds: if camera.is_some() {
            world.time().dt()
        } else {
            0.0
        },
    }
}
