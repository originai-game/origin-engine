//! The colliders particles, visual bodies and debris meet.

use super::*;

/// The colliders CPU-simulated particles bounce off, visual bodies meet and
/// the debris layer's pieces land on (`RenderScene::colliders`): every
/// og.Collider that is a box, a sphere, a capsule or a terrain's height
/// field, placed as physics places it, while the frame has debris, a
/// visual body or a particle system with collision. Compute particles
/// read these where their emitter names them (`collision ... colliders`)
/// and in sprite mode, the depth buffer otherwise; the CPU path always
/// reads these, so sparks bounce on the ground and on solid shapes.
pub(super) fn colliders(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.world.colliders.clear();
    let colliding = !rs.scene.objects.debris.is_empty()
        || rs.scene.has_visual_bodies()
        || rs.scene.effects.particles.iter().any(|s| {
            rs.scene
                .emitter(s.emitter)
                .is_some_and(|e| e.collision.is_some())
        });
    let Some(layout) = rs.draw.collider.clone().filter(|_| colliding) else {
        return;
    };
    for (e, v) in rows(world, &layout) {
        let Ok(v) = crate::host::wire::record::Collider::from_json(&v) else {
            continue;
        };
        let world_m = h.world_of(e);
        let q = v.quaternion.normalize();
        let local = Mat4::from_rotation_translation(q, v.offset);
        let m = world_m * local;
        let collider = match v.shape {
            0 => og_render::SceneCollider::Box {
                transform: m,
                half_extents: v.size * 0.5,
            },
            1 => {
                let scale = world_m.to_scale_rotation_translation().0.max_element();
                og_render::SceneCollider::Sphere {
                    center: m.transform_point3(Vec3::ZERO),
                    radius: v.radius * scale,
                }
            }
            2 => {
                let scale = world_m.to_scale_rotation_translation().0.max_element();
                let half = v.height * 0.5;
                og_render::SceneCollider::Capsule {
                    a: m.transform_point3(Vec3::new(0.0, -half, 0.0)),
                    b: m.transform_point3(Vec3::new(0.0, half, 0.0)),
                    radius: v.radius * scale,
                }
            }
            6 => {
                let geometry = v.mesh;
                let Some(data) = super::terrain::field_of_geometry(&rs.terrain, geometry) else {
                    continue;
                };
                og_render::SceneCollider::HeightField { data, transform: m }
            }
            _ => continue,
        };
        rs.scene.world.colliders.push(collider);
    }
}
