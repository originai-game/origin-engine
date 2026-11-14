//! Soft bodies drawn: visible og.Rope as a strip (sprites in 2D), og.Cloth
//! and og.SoftBody as meshes replaced every frame, from the physics' last
//! step or simulated by the renderer (SIMULATION_VISUAL).

use crate::host::wire::{FromJson, record};
use crate::present::extract::RenderState;
use crate::present::fx::kept_rows;
use glam::{Mat4, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_render::{MeshData, Projection, Ribbon, RibbonPoint, RibbonUv, Sprite, Vertex};
use std::sync::Arc;

/// The bending of a cloth the renderer simulates (og.Cloth has no field
/// for it; the tick's solve has no bending): enough to round its folds.
const CLOTH_BENDING: f32 = 0.2;

/// Whether a Cloth, SoftBody, Garment or Fluid3D `simulation` is visual.
pub(crate) fn visual(simulation: crate::host::wire::Simulation) -> bool {
    simulation == crate::host::wire::Simulation::Visual
}

/// The GPU solver's small steps per tick for a row's `substeps` and
/// `iterations`: twice their product, so its one pass a step converges as
/// the tick's iterations would (Macklin et al. 2019).
pub(crate) fn small_steps(substeps: u32, iterations: u32) -> u32 {
    let s = substeps.clamp(1, 16);
    let i = iterations.clamp(1, 64);
    (2 * s * i).clamp(4, 96)
}

/// og.Rope, og.Cloth and og.SoftBody into `rs.scene`. Returns the keys of
/// the instances it added, for per-object motion.
// Rendering only: drawn lines.
#[allow(clippy::disallowed_methods)]
pub(super) fn extract(world: &World, rs: &mut RenderState) -> Vec<(u64, u32)> {
    let mut keys = Vec::new();
    let RenderState {
        scene,
        draw,
        atmo,
        materials,
        ..
    } = rs;
    let layouts = &draw.atmo;
    let Some(physics) = layouts.physics.as_ref() else {
        return keys;
    };
    let dt = world.time().dt;
    let orthographic = matches!(
        scene.frame.camera.projection,
        Projection::Orthographic { .. }
    );
    // Ropes: a strip through their points in 3D, a sprite per segment in 2D.
    for (e, v) in kept_rows(world, layouts.rope.as_ref()) {
        // The hand read's colour default (black), not the IDL's brown.
        let base = record::Rope {
            color: Vec3::ZERO,
            ..Default::default()
        };
        let Ok(v) = record::Rope::from_json_over(&v, base) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(points) = physics.rope_points(Entity::from_bits(e)) else {
            continue;
        };
        // Rope.color is a Vector3 (x, y, z are r, g, b).
        let c = v.color;
        let width = v.width;
        if orthographic {
            for pair in points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let d = b - a;
                scene.sprite.sprites.push(Sprite {
                    position: ((a + b) * 0.5).truncate().extend(a.z),
                    size: Vec2::new(d.truncate().length() + width, width),
                    rotation: d.y.atan2(d.x),
                    color: [c.x, c.y, c.z, 1.0],
                    ..Default::default()
                });
            }
        } else {
            scene.effects.ribbons.push(Ribbon {
                points: points
                    .iter()
                    .map(|&position| RibbonPoint {
                        position,
                        width: width * 1.6,
                        color: [c.x, c.y, c.z, 1.0],
                    })
                    .collect(),
                texture: None,
                uv: RibbonUv::Stretch,
                blend: og_render::ParticleBlend::Alpha,
                effect: None,
            });
        }
    }
    // og.Physics.gravity, for the bodies the renderer simulates.
    let gravity = physics.gravity().unwrap_or(Vec3::new(0.0, -9.81, 0.0));
    // Cloth: a double-sided mesh of its points, replaced every frame.
    for (e, v) in kept_rows(world, layouts.cloth.as_ref()) {
        let Ok(row) = record::Cloth::from_json(&v) else {
            continue;
        };
        if !row.visible {
            continue;
        }
        let Some(c) = physics.cloth_points(Entity::from_bits(e)) else {
            continue;
        };
        let mesh = cloth_mesh(c.columns, c.rows, &c.points);
        let handle = match atmo.cloths.get(&e) {
            Some(&handle) => handle,
            None => {
                let handle = scene.add_mesh(MeshData::default());
                atmo.cloths.insert(e, handle);
                handle
            }
        };
        scene.set_simulation_surface(
            handle,
            mesh,
            og_render::SimulationSurface::Cloth {
                columns: c.columns,
                rows: c.rows,
            },
        );
        if visual(row.simulation) {
            // The air where it hangs, as the tick's steps take it (og.Wind
            // and the og.WindMotors reaching it), beside its own.
            let (min, max) = c.points.iter().fold(
                (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
                |(min, max), p| (min.min(*p), max.max(*p)),
            );
            let blowing = physics.air_at((min + max) * 0.5);
            let body = og_render::VisualBody {
                pinned: c.pinned.into(),
                substeps: small_steps(row.substeps, row.iterations),
                keep: og_core::math::damping_keep(row.damping, dt),
                gravity: gravity * row.gravity_scale,
                wind: row.wind + blowing,
                gust: row.gust,
                radius: row.radius,
                friction: row.friction,
                thickness: row.thickness,
                stiffness: 1.0,
                bending: CLOTH_BENDING,
                plane: c.flat.then(|| c.points.first().map_or(0.0, |p| p.z)),
                ..Default::default()
            };
            scene.set_visual_body(handle, Some(Arc::new(body)));
        }
        let material = crate::world::assets::index_of(row.material as f64)
            .and_then(|i| materials.get(i).copied())
            .unwrap_or_else(|| {
                *atmo.cloth_material.get_or_insert_with(|| {
                    scene.add_standard_material(og_render::StandardMaterial {
                        color: Vec3::new(0.72, 0.16, 0.14),
                        roughness: 0.8,
                        side: 2,
                        ..Default::default()
                    })
                })
            });
        scene.add_instance(handle, material, Mat4::IDENTITY);
        keys.push((e, u32::MAX));
    }
    // Soft volumes: their surface, smooth-shaded, replaced every frame.
    for (e, v) in kept_rows(world, layouts.soft.as_ref()) {
        let Ok(row) = record::SoftBody::from_json(&v) else {
            continue;
        };
        if !row.visible {
            continue;
        }
        let Some(s) = physics.soft_body_points(Entity::from_bits(e)) else {
            continue;
        };
        let mesh = soft_mesh(&s.points, &s.indices);
        let handle = match atmo.softs.get(&e) {
            Some(&handle) => handle,
            None => {
                let handle = scene.add_mesh(MeshData::default());
                atmo.softs.insert(e, handle);
                handle
            }
        };
        scene.set_simulation_surface(handle, mesh, og_render::SimulationSurface::SoftBody);
        if visual(row.simulation) {
            let body = og_render::VisualBody {
                pinned: s.pinned.into(),
                substeps: small_steps(row.substeps, row.iterations),
                keep: og_core::math::damping_keep(row.damping, dt),
                gravity: gravity * row.gravity_scale,
                radius: row.thickness,
                friction: row.friction,
                stiffness: row.stiffness,
                pressure: row.pressure,
                volume_stiffness: if row.pressure > 0.0 {
                    row.volume_stiffness
                } else {
                    0.0
                },
                plane: s.flat.then(|| s.points.first().map_or(0.0, |p| p.z)),
                ..Default::default()
            };
            scene.set_visual_body(handle, Some(Arc::new(body)));
        }
        let material = crate::world::assets::index_of(row.material as f64)
            .and_then(|i| materials.get(i).copied())
            .unwrap_or_else(|| {
                *atmo.soft_material.get_or_insert_with(|| {
                    scene.add_standard_material(og_render::StandardMaterial {
                        color: Vec3::new(0.25, 0.75, 0.35),
                        roughness: 0.35,
                        side: 2,
                        ..Default::default()
                    })
                })
            });
        scene.add_instance(handle, material, Mat4::IDENTITY);
        keys.push((e, u32::MAX));
    }
    keys
}

/// The solve's coarse draw mesh; the renderer owns normals and refinement.
fn soft_mesh(points: &[Vec3], indices: &[u32]) -> MeshData {
    let vertices = points
        .iter()
        .map(|p| Vertex {
            tangent: [0.0; 4],
            position: p.to_array(),
            normal: [0.0; 3],
            ..Default::default()
        })
        .collect();
    MeshData {
        vertices,
        indices: indices.to_vec(),
    }
}

/// A coarse `columns` x `rows` grid of world points and UVs.
fn cloth_mesh(columns: u32, rows: u32, points: &[Vec3]) -> MeshData {
    let (w, ht) = (columns.max(2) as usize, rows.max(2) as usize);
    if points.len() < w * ht {
        return MeshData {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    let at = |x: usize, y: usize| points[y * w + x];
    let mut vertices = Vec::with_capacity(w * ht);
    for y in 0..ht {
        for x in 0..w {
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: at(x, y).to_array(),
                normal: [0.0; 3],
                uv: [x as f32 / (w - 1) as f32, y as f32 / (ht - 1) as f32],
                ..Default::default()
            });
        }
    }
    let mut indices = Vec::with_capacity((w - 1) * (ht - 1) * 6);
    for y in 0..ht - 1 {
        for x in 0..w - 1 {
            let i = (y * w + x) as u32;
            let (r, d) = (i + 1, i + w as u32);
            indices.extend_from_slice(&[i, d, r, r, d, d + 1]);
        }
    }
    MeshData { vertices, indices }
}
