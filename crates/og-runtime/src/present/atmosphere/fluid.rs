//! Fluids drawn: og.Fluid2D as metaballs (a group as one surface) and
//! og.Fluid3D as a screen-space surface (a group's particles coloured by
//! their mix), from the physics' last step or simulated by the renderer.

use super::soft::visual;
use crate::host::wire::{FromJson, record};
use crate::present::extract::{Hierarchy, RenderState};
use crate::present::fx::kept_rows;
use glam::Vec3;
use og_ecs::{Entity, World};
use og_render::Metaballs;
use std::collections::BTreeMap;
use std::sync::Arc;

/// og.Fluid2D and og.Fluid3D into `rs.scene`.
// Rendering only: drawn fluid volumes.
#[allow(clippy::disallowed_methods)]
pub(super) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let RenderState { scene, draw, .. } = rs;
    scene.sprite.metaballs.clear();
    scene.world.fluids_3d.clear();
    let layouts = &draw.atmo;
    let Some(physics) = layouts.physics.as_ref() else {
        return;
    };
    // Fluids as metaballs.
    for (e, v) in kept_rows(world, layouts.fluid.as_ref()) {
        let Ok(v) = record::Fluid2D::from_json(&v) else {
            continue;
        };
        if !v.visible {
            continue;
        }
        let Some(particles) = physics.fluid_particles(Entity::from_bits(e)) else {
            continue;
        };
        // A fluid sharing its group is drawn with the group as one surface,
        // its particles by their mix (og-physics' fluid_mix: its lane).
        let (group, lane, mix) = match physics.fluid_mix(Entity::from_bits(e)) {
            Some((lane, mix)) => (v.group, lane, Arc::from(mix)),
            None => (0, 0, Arc::from(Vec::new())),
        };
        scene.sprite.metaballs.push(Metaballs {
            particles: Arc::from(particles),
            radius: v.radius * v.blob_scale,
            threshold: v.threshold,
            // Fluid2D.color is a Vector3 (x, y, z are r, g, b).
            color: v.color.extend(v.opacity),
            layer: v.layer,
            z: h.world_of(e).transform_point3(Vec3::ZERO).z,
            group,
            lane,
            mix,
            ..Metaballs::default()
        });
    }
    // 3D fluids as a screen-space surface. A fluid sharing a group colours
    // its particles by their mix over the group's fluids' colours (lanes as
    // og-physics' fluid3d_mix gives them).
    // Fluid3D.color is a Vector3 (x, y, z are r, g, b).
    let fluids3: Vec<(u64, record::Fluid3D)> = kept_rows(world, layouts.fluid3d.as_ref())
        .into_iter()
        .filter_map(|(e, v)| {
            // The hand read's size default (zero), not the IDL's (2, 1, 2).
            let base = record::Fluid3D {
                size: Vec3::ZERO,
                ..Default::default()
            };
            Some((e, record::Fluid3D::from_json_over(&v, base).ok()?))
        })
        .collect();
    let mut lanes: BTreeMap<(u32, u32), Vec3> = BTreeMap::new();
    let mut mixes = BTreeMap::new();
    for (e, v) in &fluids3 {
        if let Some((lane, mix)) = physics.fluid3d_mix(Entity::from_bits(*e)) {
            lanes.insert((v.group, lane), v.color);
            mixes.insert(*e, (v.group, mix));
        }
    }
    for (e, v) in &fluids3 {
        if !v.visible {
            continue;
        }
        let Some(particles) = physics.fluid3d_particles(Entity::from_bits(*e)) else {
            continue;
        };
        let radius = v.radius;
        // SIMULATION_VISUAL: the renderer's particles (the tick has none),
        // spawned and moved on the GPU from this.
        if visual(v.simulation) {
            let at = h.world_of(*e).transform_point3(Vec3::ZERO);
            let size = v.size;
            let count = v.count;
            let fluid = og_render::VisualFluid {
                key: *e,
                count,
                centre: at,
                size,
                bounds: v.bounds,
                cell: radius,
                max_count: v.max_count,
                // The solver's own are water at the component's defaults.
                stiffness: v.stiffness / 0.02,
                viscosity: v.viscosity / 0.4,
                gravity: physics.gravity().unwrap_or(Vec3::new(0.0, -9.81, 0.0)) * v.gravity_scale,
                substeps: v.substeps,
                seed: v.seed,
                emit_rate: v.emit_rate,
                emit_at: at + v.emit_offset,
                emit_radius: v.emit_radius,
                emit_velocity: v.emit_velocity,
                sink: {
                    let half = v.sink_size * 0.5;
                    let c = at + v.sink_offset;
                    half.cmpgt(Vec3::ZERO).all().then_some((c - half, c + half))
                },
            };
            // Each particle stands for the spawn box's volume over the
            // count, and is drawn at least as wide as that spacing.
            let volume = (size.x * size.y * size.z).max(1e-6) / count.max(1) as f32;
            scene.world.fluids_3d.push(og_render::Fluid3d {
                particles: Arc::from(Vec::new()),
                colors: None,
                color: v.color,
                opacity: v.opacity,
                radius: (radius * v.draw_scale).max(0.9 * volume.cbrt()),
                volume,
                reflectivity: v.reflectivity,
                refraction: v.refraction,
                smoothing: v.smoothing,
                visual: Some(Arc::new(fluid)),
            });
            continue;
        }
        let colors = mixes
            .get(e)
            .filter(|(_, mix)| mix.len() == particles.len())
            .map(|(group, mix)| {
                mix.iter()
                    .map(|m| {
                        (0..4u32)
                            .filter_map(|k| Some(*lanes.get(&(*group, k))? * m[k as usize]))
                            .sum::<Vec3>()
                    })
                    .collect::<Arc<[Vec3]>>()
            });
        scene.world.fluids_3d.push(og_render::Fluid3d {
            particles: Arc::from(particles),
            colors,
            color: v.color,
            opacity: v.opacity,
            radius: radius * v.draw_scale,
            // The share of the volume each particle stands for: at rest
            // they sit about half the interaction radius apart.
            volume: (radius * 0.5).powi(3),
            reflectivity: v.reflectivity,
            refraction: v.refraction,
            smoothing: v.smoothing,
            visual: None,
        });
    }
}
