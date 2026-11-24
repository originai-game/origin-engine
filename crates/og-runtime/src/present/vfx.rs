//! Composed effects in a frame: every playing og.Vfx expanded into the
//! renderer's particle systems, effect meshes, trails, beams, lights and
//! decals from (asset, seed, startTick, the tick) alone
//! (og_bundle::vfx's `Vfx::draws`), with the handles `crate::bind::vfx`
//! linked at load. Nothing extraction makes is world state.

use crate::bind::vfx::{Link, UNSET, index, lock};
use crate::host::codec::{Rec, Slot};
use crate::present::extract::{Hierarchy, RenderState};
use glam::{Mat4, Vec3};
use og_bundle::vfx::{VfxDrawKind, VfxInstance};
use og_ecs::World;
use og_render::{
    Beam, ColorCurve, Curve, Decal, EffectMesh, LightDesc, LightKind, ParticleBlend,
    ParticleSystem, RibbonUv, Trail, TrailStyle,
};

/// A playing og.Vfx expanded for the frame: its asset, start, look and
/// draws, and the particle slots its emitters hold.
struct Playing<'a> {
    l: &'a crate::bind::vfx::Loaded,
    start: u64,
    tint: Vec3,
    emission: f32,
    hue: f32,
    draws: Vec<og_bundle::vfx::VfxDraw>,
    slots: u64,
}

/// Which of the playing effects draw: all of them within
/// RenderSettings.maxVfx and maxVfxParticles; past either, the oldest
/// one-shot effects are left out first, then the oldest looping ones
/// (the same start: the later in the scene's order first kept).
fn kept(playing: &[Playing], q: &og_render::RenderQuality) -> Vec<bool> {
    let mut kept = vec![true; playing.len()];
    let (mut count, mut slots) = (playing.len(), playing.iter().map(|p| p.slots).sum::<u64>());
    let mut order: Vec<usize> = (0..playing.len()).collect();
    order.sort_by_key(|&k| (playing[k].l.vfx.looping, playing[k].start, k));
    for k in order {
        if count <= q.max_vfx as usize && slots <= u64::from(q.max_vfx_particles) {
            break;
        }
        kept[k] = false;
        count -= 1;
        slots -= playing[k].slots;
    }
    kept
}

/// Collects playing `og.Vfx` instances directly from packed columns.
/// When `separate_sim_transforms` is true and display interpolation is active,
/// simulation-state elements (`Emitter`, `Trail`) evaluate against the
/// uninterpolated tick transform while single-frame visuals use the blended
/// world transform.
fn collect_playing<'a>(
    world: &World,
    rs: &RenderState,
    h: &mut Hierarchy,
    loaded: &'a [std::sync::Arc<crate::bind::vfx::Loaded>],
    tick: u64,
    rate: f32,
    separate_sim_transforms: bool,
) -> Vec<Playing<'a>> {
    let Some(layout) = rs.draw.fx.vfx.as_ref() else {
        return Vec::new();
    };
    let rows = crate::present::extract::column_rows(world, layout);
    if rows.is_empty() {
        return Vec::new();
    }
    let asset_s = Slot::of(layout, "asset");
    let seed_s = Slot::of(layout, "seed");
    let start_s = Slot::of(layout, "startTick");
    let scale_s = Slot::of(layout, "scale");
    let tint_s = Slot::of(layout, "tint");
    let emission_s = Slot::of(layout, "emission");
    let hue_s = Slot::of(layout, "hue");
    let params_s = Slot::of(layout, "params");
    let interpolated = separate_sim_transforms && h.frame_alpha().is_some();
    let mut params_buf = Vec::new();
    let mut playing = Vec::new();
    for (e, v) in crate::present::fx::sorted_column_rows(&rows) {
        let asset = v.uint(&asset_s).unwrap_or(0);
        let Some(l) = index(asset).and_then(|i| loaded.get(i)) else {
            continue;
        };
        let start = v.uint(&start_s).unwrap_or(UNSET);
        if start == UNSET {
            continue;
        }
        let seed = v.uint(&seed_s).map_or(0, |x| x as u32);
        let scale = v.num(&scale_s).map_or(1.0, |x| x as f32).max(0.0);
        let tint = Vec3::new(
            v.lane(&tint_s, "r").map_or(1.0, |x| x as f32),
            v.lane(&tint_s, "g").map_or(1.0, |x| x as f32),
            v.lane(&tint_s, "b").map_or(1.0, |x| x as f32),
        );
        let emission = v.num(&emission_s).map_or(1.0, |x| x as f32);
        let hue = v.num(&hue_s).map_or(0.0, |x| x as f32);
        let hue = if hue.is_finite() { hue } else { 0.0 };
        v.list_f32(&params_s, &mut params_buf);
        let params = l.vfx.values(&params_buf);
        let scale_m = Mat4::from_scale(Vec3::splat(scale));
        let draws = if interpolated && l.has_sim_elements && l.has_visual_elements {
            let tick_inst = VfxInstance {
                id: e,
                seed,
                transform: h.tick_world_of(e) * scale_m,
                scale,
                tint,
                hue,
                params,
            };
            let drawn_inst = VfxInstance {
                id: e,
                seed,
                transform: h.world_of(e) * scale_m,
                scale,
                tint,
                hue,
                params,
            };
            let tick_draws = l.vfx.draws(&tick_inst, start, tick, rate);
            let drawn_draws = l.vfx.draws(&drawn_inst, start, tick, rate);
            tick_draws
                .into_iter()
                .zip(drawn_draws)
                .map(|(td, dd)| match td.kind {
                    VfxDrawKind::Emitter { .. } | VfxDrawKind::Trail { .. } => td,
                    _ => dd,
                })
                .collect()
        } else {
            let world_m = if interpolated && !l.has_sim_elements {
                h.world_of(e)
            } else {
                h.tick_world_of(e)
            };
            let instance = VfxInstance {
                id: e,
                seed,
                transform: world_m * scale_m,
                scale,
                tint,
                hue,
                params,
            };
            l.vfx.draws(&instance, start, tick, rate)
        };
        if draws.is_empty() {
            continue;
        }
        let slots = draws
            .iter()
            .map(|d| match &l.links[d.element] {
                Link::Emitter(_, handle) if matches!(d.kind, VfxDrawKind::Emitter { .. }) => rs
                    .scene
                    .emitter(*handle)
                    .map_or(0, |m| u64::from(m.max_particles)),
                _ => 0,
            })
            .sum();
        playing.push(Playing {
            l,
            start,
            tint,
            emission,
            hue,
            draws,
            slots,
        });
    }
    playing
}

/// Lightweight tick-start extraction of composed VFX emitters only, using the
/// live world `rate` (never stale `scene.frame.tick_rate` before frame 1).
pub(crate) fn extract_particles(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    tick: u64,
    rate: f32,
    out: &mut Vec<ParticleSystem>,
) {
    let Some(registry) = rs.effects.vfx.clone() else {
        return;
    };
    let loaded = lock(&registry).clone();
    if loaded.iter().all(|l| !l.has_emitters) {
        return;
    }
    let rate = rate.max(1.0);
    let playing = collect_playing(world, rs, h, &loaded, tick, rate, false);
    if playing.is_empty() {
        return;
    }
    let kept = kept(&playing, &rs.scene.frame.quality);
    let mut air: Option<og_env::air::Air> = None;
    for (p, _) in playing.into_iter().zip(kept).filter(|(_, k)| *k) {
        if !p.l.has_emitters {
            continue;
        }
        let Playing {
            l,
            tint,
            emission,
            hue,
            draws,
            ..
        } = p;
        for d in draws {
            if let (
                VfxDrawKind::Emitter {
                    transform,
                    start_tick,
                    stop_tick,
                    seed,
                    emission: amount,
                },
                Link::Emitter(index, handle),
            ) = (d.kind, &l.links[d.element])
            {
                if !crate::present::fx::emitter_ready(
                    &mut rs.scene,
                    &rs.fx,
                    &rs.texture_arrays,
                    *index,
                    *handle,
                ) {
                    continue;
                }
                let air = air.get_or_insert_with(|| rs.draw.atmo.air(world, h));
                out.push(ParticleSystem {
                    stop_tick,
                    start_tick,
                    air: air.at(transform.w_axis.truncate()),
                    tint,
                    emission: emission * amount,
                    hue,
                    ..ParticleSystem::new(d.id, *handle, transform, seed, tick)
                });
            }
        }
    }
}

/// Every playing og.Vfx expanded into the scene's systems, meshes, trails,
/// beams, lights and decals for the frame at `tick`. A `frame`'s
/// extraction also notes a decal drawn from an atlas region with no atlas set,
/// and counts the effects drawn and left out (`kept`).
pub(crate) fn extract(
    world: &World,
    rs: &mut RenderState,
    h: &mut Hierarchy,
    tick: u64,
    frame: bool,
) {
    let Some(registry) = rs.effects.vfx.clone() else {
        return;
    };
    let loaded = lock(&registry).clone();
    let rate = rs.scene.frame.tick_rate.max(1.0);
    let playing = collect_playing(world, rs, h, &loaded, tick, rate, true);
    if playing.is_empty() {
        if frame {
            rs.vfx_drawn = 0;
            rs.vfx_dropped = 0;
        }
        return;
    }
    let kept = kept(&playing, &rs.scene.frame.quality);
    if frame {
        rs.vfx_drawn = kept.iter().filter(|k| **k).count() as u32;
        rs.vfx_dropped = playing.len() as u32 - rs.vfx_drawn;
    }
    let mut air: Option<og_env::air::Air> = None;
    for (p, _) in playing.into_iter().zip(kept).filter(|(_, k)| *k) {
        let Playing {
            l,
            tint,
            emission,
            hue,
            draws,
            ..
        } = p;
        for d in draws {
            let id = d.id;
            match (d.kind, &l.links[d.element]) {
                (
                    VfxDrawKind::Emitter {
                        transform,
                        start_tick,
                        stop_tick,
                        seed,
                        emission: amount,
                    },
                    Link::Emitter(index, handle),
                ) => {
                    if !crate::present::fx::emitter_ready(
                        &mut rs.scene,
                        &rs.fx,
                        &rs.texture_arrays,
                        *index,
                        *handle,
                    ) {
                        continue;
                    }
                    // The frame's air, read once for the first emitter it carries.
                    let air = air.get_or_insert_with(|| rs.draw.atmo.air(world, h));
                    rs.scene.effects.particles.push(ParticleSystem {
                        stop_tick,
                        start_tick,
                        air: air.at(transform.w_axis.truncate()),
                        tint,
                        emission: emission * amount,
                        hue,
                        ..ParticleSystem::new(id, *handle, transform, seed, tick)
                    });
                }
                (
                    VfxDrawKind::Mesh {
                        transform,
                        life,
                        seed,
                        color,
                        blend,
                        billboard,
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
                        hue,
                        billboard,
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
                    let handle = rs.fx.trail_style(&mut rs.scene, id, style);
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
                    rs.scene.lighting.lights.push(LightDesc {
                        kind: LightKind::Point,
                        position,
                        color,
                        intensity,
                        range,
                        radius: 0.0,
                        cast_shadows: false,
                        mobility: og_render::LightMobility::Movable,
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
                    // A region of no atlas draws the decal's colour as a square.
                    if frame
                        && rs.scene.effects.decal_atlas.is_none()
                        && uv != [0.0, 0.0, 1.0, 1.0]
                        && rs.vfx_decals_warned.insert(l.id.clone())
                    {
                        rs.notes.push(format!(
                            "og.vfx: '{}' draws a decal from region {uv:?} of og.RenderSettings.decalAtlas, which is not set, so the decal is a square of its colour; set decalAtlas to the texture its regions are in",
                            l.id
                        ));
                    }
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
