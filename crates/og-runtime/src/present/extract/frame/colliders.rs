//! The colliders particles, visual bodies and debris meet, and the force
//! fields particles are accelerated by.

use super::*;

/// The colliders CPU-simulated particles bounce off, visual bodies meet and
/// the debris layer's pieces land on (`RenderScene::colliders`): every
/// solid, nonzero-layer og.Collider that is a box, sphere, capsule,
/// cylinder, convex hull or terrain height field, placed as physics
/// places it, while the frame has debris, a visual body or fluid, or a
/// particle system with collision. Compute particles
/// read these where their emitter names them (`collision ... colliders`)
/// and in sprite mode, the depth buffer otherwise; the CPU path always
/// reads these, so sparks bounce on the ground and on solid shapes.
pub(super) fn colliders(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    rs.scene.world.colliders.clear();
    let colliding = !rs.scene.objects.debris.is_empty()
        || rs.scene.has_visual_bodies()
        || rs.scene.world.fluids_3d.iter().any(|f| f.visual.is_some())
        || rs.scene.effects.particles.iter().any(|s| {
            rs.scene
                .emitter(s.emitter)
                .is_some_and(|e| e.collision.is_some())
        });
    let Some(layout) = rs.draw.collider.as_ref().filter(|_| colliding) else {
        return;
    };
    let rows = column_rows(world, layout);
    if rows.is_empty() {
        return;
    }
    let slot = |key| FieldSlot::of(layout, key);
    let (shape_s, size_s, radius_s, height_s) =
        (slot("shape"), slot("size"), slot("radius"), slot("height"));
    let (mesh_s, offset_s, quat_s) = (slot("mesh"), slot("offset"), slot("quaternion"));
    let (sensor_s, layers_s) = (slot("sensor"), slot("layers"));
    for (e, v) in rows.iter() {
        if v.flag(&sensor_s).unwrap_or(false) || v.uint(&layers_s) == Some(0) {
            continue;
        }
        let num = |s: &FieldSlot, d: f32| v.num(s).map_or(d, |x| x as f32);
        let vec3 = |s: &FieldSlot, d: Vec3| {
            Vec3::new(
                v.lane(s, "x").map_or(d.x, |x| x as f32),
                v.lane(s, "y").map_or(d.y, |x| x as f32),
                v.lane(s, "z").map_or(d.z, |x| x as f32),
            )
        };
        let quat = glam::Quat::from_xyzw(
            v.lane(&quat_s, "x").map_or(0.0, |x| x as f32),
            v.lane(&quat_s, "y").map_or(0.0, |x| x as f32),
            v.lane(&quat_s, "z").map_or(0.0, |x| x as f32),
            v.lane(&quat_s, "w").map_or(1.0, |x| x as f32),
        )
        .normalize();
        let offset = vec3(&offset_s, Vec3::ZERO);
        let world_m = h.tick_world_of(e);
        let local = Mat4::from_rotation_translation(quat, offset);
        let m = world_m * local;
        let shape_name = v.text(&shape_s);
        let shape = shape_name
            .as_deref()
            .unwrap_or_else(|| match v.uint(&shape_s) {
                None | Some(0) => "box",
                Some(1) => "sphere",
                Some(2) => "capsule",
                Some(3) => "cylinder",
                Some(4) => "convex",
                Some(6) => "heightfield",
                _ => "",
            });
        let scale = world_m
            .to_scale_rotation_translation()
            .0
            .abs()
            .max_element();
        let collider = match shape {
            "box" => og_render::SceneCollider::Box {
                transform: m,
                half_extents: vec3(&size_s, Vec3::ONE) * 0.5,
            },
            "sphere" => og_render::SceneCollider::Sphere {
                center: m.transform_point3(Vec3::ZERO),
                radius: num(&radius_s, 0.5) * scale,
            },
            "capsule" => {
                let half = num(&height_s, 1.0) * 0.5;
                og_render::SceneCollider::Capsule {
                    a: m.transform_point3(Vec3::new(0.0, -half, 0.0)),
                    b: m.transform_point3(Vec3::new(0.0, half, 0.0)),
                    radius: num(&radius_s, 0.5) * scale,
                }
            }
            "cylinder" => og_render::SceneCollider::Cylinder {
                transform: m,
                radius: num(&radius_s, 0.5),
                half_height: num(&height_s, 1.0) * 0.5,
            },
            "convex" => {
                let geometry = v.uint(&mesh_s).unwrap_or(0);
                let Some(&handle) = crate::world::assets::index_of(geometry as f64)
                    .and_then(|i| rs.geometries.get(i))
                else {
                    continue;
                };
                let revision = rs.scene.mesh_revision(handle.index());
                if let Some((old, rev, planes)) = rs.convex_planes.get(&geometry)
                    && *old == handle
                    && *rev == revision
                {
                    rs.scene
                        .world
                        .colliders
                        .push(og_render::SceneCollider::Convex {
                            transform: m,
                            planes: planes.clone(),
                        });
                    continue;
                }
                let Some(mesh) = rs.scene.mesh(handle) else {
                    continue;
                };
                let points: Vec<_> = mesh
                    .vertices
                    .iter()
                    .map(|v| Vec3::from(v.position).as_dvec3())
                    .collect();
                let Ok(hull) = og_shape::generators::hull::convex_hull(&points) else {
                    continue;
                };
                let planes: Vec<_> = hull
                    .triangles
                    .iter()
                    .map(|t| {
                        let [a, b, c] = t.map(|i| hull.positions[i as usize]);
                        let n = (b - a).cross(c - a).normalize();
                        n.as_vec3().extend(n.dot(a) as f32)
                    })
                    .collect();
                let planes: std::sync::Arc<[glam::Vec4]> = planes.into();
                rs.convex_planes
                    .insert(geometry, (handle, revision, planes.clone()));
                og_render::SceneCollider::Convex {
                    transform: m,
                    planes,
                }
            }
            "heightfield" => {
                let geometry = v.uint(&mesh_s).unwrap_or(0);
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

/// The force fields visual bodies and particles with a `forceFields`
/// share are accelerated by (`RenderScene::world.force_fields`): every
/// enabled og.ForceField where the frame places it, while either is drawn.
pub(super) fn force_fields(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let pulled = rs.scene.has_visual_bodies()
        || rs.scene.effects.particles.iter().any(|s| {
            rs.scene
                .emitter(s.emitter)
                .is_some_and(|e| e.force_fields > 0.0)
        });
    rs.scene.world.force_fields = if pulled {
        crate::bind::forces::frame_forces(
            world,
            rs.draw.force_field.as_ref(),
            rs.draw.volume.as_ref(),
            h,
        )
    } else {
        Default::default()
    };
}
