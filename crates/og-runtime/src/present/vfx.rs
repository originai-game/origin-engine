//! Composed effects in a frame: every playing og.Vfx expanded into the
//! renderer's particle systems, effect meshes, trails, beams, lights and
//! decals from (asset, seed, startTick, the tick) alone
//! (og_bundle::vfx's `Vfx::draws`), with the handles `crate::bind::vfx`
//! linked at load. Nothing extraction makes is world state.

use crate::bind::vfx::{Link, UNSET, index, lock};
use crate::host::wire::{FromJson, record};
use crate::present::extract::{Hierarchy, RenderState};
use glam::{Mat4, Vec3};
use og_bundle::vfx::{VfxDrawKind, VfxInstance};
use og_ecs::World;
use og_render::{
    Beam, ColorCurve, Curve, Decal, EffectMesh, EmitterHandle, Light, LightKind, ParticleBlend,
    ParticleSystem, RibbonUv, Trail, TrailStyle,
};

/// Every playing og.Vfx expanded into the scene's systems, meshes, trails,
/// beams, lights and decals for the frame at `tick`.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy, tick: u64) {
    let Some(registry) = rs.effects.vfx.clone() else {
        return;
    };
    let rows = crate::present::fx::rows(world, rs.draw.fx.vfx.as_ref());
    if rows.is_empty() {
        return;
    }
    let loaded = lock(&registry).clone();
    let rate = rs.scene.frame.tick_rate.max(1.0);
    let mut air: Option<og_env::air::Air> = None;
    for (e, v) in rows {
        let Ok(row) = record::Vfx::from_json(&v) else {
            continue;
        };
        let Some(l) = index(row.asset).and_then(|i| loaded.get(i)) else {
            continue;
        };
        let start = u64::from(row.start_tick);
        if start == UNSET {
            continue;
        }
        let tint = Vec3::new(row.tint.r, row.tint.g, row.tint.b);
        let scale = row.scale.max(0.0);
        let instance = VfxInstance {
            id: e,
            seed: row.seed,
            transform: h.world_of(e) * Mat4::from_scale(Vec3::splat(scale)),
            scale,
            tint,
        };
        for d in l.vfx.draws(&instance, start, tick, rate) {
            let id = d.id;
            match (d.kind, &l.links[d.element]) {
                (
                    VfxDrawKind::Emitter {
                        transform,
                        start_tick,
                        stop_tick,
                        seed,
                    },
                    Link::Emitter(handle),
                ) => {
                    let emitter = tinted_emitter(rs, *handle, tint);
                    // The frame's air, read once for the first emitter it carries.
                    let air = air.get_or_insert_with(|| rs.draw.atmo.air(world, h));
                    rs.scene.effects.particles.push(ParticleSystem {
                        stop_tick,
                        start_tick,
                        air: air.at(transform.w_axis.truncate()),
                        ..ParticleSystem::new(id, emitter, transform, seed, tick)
                    });
                }
                (
                    VfxDrawKind::Mesh {
                        transform,
                        life,
                        seed,
                        color,
                        blend,
                    },
                    Link::Mesh(mesh, material),
                ) => {
                    rs.scene.effects.effect_meshes.push(EffectMesh {
                        id,
                        mesh: *mesh,
                        material: *material,
                        transform,
                        life,
                        seed,
                        color,
                        blend,
                        layers: 1,
                    });
                }
                (
                    VfxDrawKind::Trail {
                        position,
                        lifetime,
                        min_distance,
                        width,
                        color,
                    },
                    Link::Effect(effect),
                ) => {
                    let style = TrailStyle {
                        lifetime,
                        min_distance,
                        width: Curve::linear(width[0], width[1]),
                        color: ColorCurve::constant(color),
                        uv: RibbonUv::Stretch,
                        blend: ParticleBlend::Additive,
                        effect: *effect,
                        ..Default::default()
                    };
                    let key = crate::present::fx::style_key(&style);
                    let handle = match rs.fx.trail_styles.get(&key) {
                        Some(h) => *h,
                        None => {
                            let h = rs.scene.add_trail_style(style);
                            rs.fx.trail_styles.insert(key, h);
                            h
                        }
                    };
                    rs.scene.effects.trails.push(Trail {
                        id,
                        style: handle,
                        position,
                        tick,
                    });
                }
                (
                    VfxDrawKind::Beam {
                        start,
                        end,
                        width,
                        color,
                        segments,
                        jitter,
                        seed,
                    },
                    Link::Effect(effect),
                ) => {
                    rs.scene.effects.beams.push(Beam {
                        start,
                        end,
                        width,
                        color,
                        segments,
                        jitter,
                        seed,
                        blend: ParticleBlend::Additive,
                        effect: *effect,
                        ..Default::default()
                    });
                }
                (
                    VfxDrawKind::Light {
                        position,
                        color,
                        intensity,
                        range,
                    },
                    _,
                ) => {
                    rs.scene.lighting.lights.push(Light {
                        kind: LightKind::Point,
                        position,
                        color,
                        intensity,
                        range,
                        cast_shadows: false,
                    });
                }
                (
                    VfxDrawKind::Decal {
                        transform,
                        uv,
                        color,
                    },
                    _,
                ) => {
                    rs.scene.effects.decals.push(Decal {
                        transform,
                        uv_rect: uv,
                        color,
                        ..Default::default()
                    });
                }
                _ => {}
            }
        }
    }
}

/// `handle` drawn with its colour times `tint`: a copy of the emitter per
/// tint, made once.
fn tinted_emitter(rs: &mut RenderState, handle: EmitterHandle, tint: Vec3) -> EmitterHandle {
    if tint == Vec3::ONE {
        return handle;
    }
    let key = (handle, tint.to_array().map(f32::to_bits));
    if let Some(&h) = rs.effects.tinted.get(&key) {
        return h;
    }
    let Some(mut e) = rs.scene.emitter(handle).cloned() else {
        return handle;
    };
    e.color = [
        e.color[0] * tint.x,
        e.color[1] * tint.y,
        e.color[2] * tint.z,
        e.color[3],
    ];
    let h = rs.scene.add_emitter(e);
    rs.effects.tinted.insert(key, h);
    h
}
