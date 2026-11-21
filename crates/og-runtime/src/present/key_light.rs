//! The key light: the one directional light a frame draws, chosen once from
//! every candidate. The candidates are the world's og.DirectionalLights
//! and, while og.DayNight runs, its sun and moon: the cycle drives the
//! DirectionalLight the lights alone would draw, so its sun and moon take
//! that light's place and its shadow settings. `og_render::key_light`
//! draws the brightest, two handing over. A static light whose light its
//! scene's drawn bake holds is no candidate, unless og.DayNight runs (the
//! cycle moves the light, so it is movable whatever it says). The frame's
//! extraction, the probe's `get` and the warnings all read this stage, so
//! what is answered is what is drawn; `og bake lightmaps` keys its scene's
//! lights with the same `og_render::key_light`.

use crate::host::codec::Layout;
use crate::host::wire::{FromJson, record};
use glam::{Mat4, Vec3};
use og_ecs::{Entity, World};
use og_render::DirectionalLight;

/// A candidate for the key light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Source {
    /// An og.DirectionalLight, by its entity's bits.
    Entity(u64),
    /// og.DayNight's sun.
    Sun,
    /// og.DayNight's moon.
    Moon,
}

/// The records the stage reads.
#[derive(Clone, Copy, Default)]
pub(crate) struct Layouts<'a> {
    pub directional: Option<&'a Layout>,
    pub day_night: Option<&'a Layout>,
    pub physical_sky: Option<&'a Layout>,
    pub season: Option<&'a Layout>,
}

/// The drawn directional light and where it comes from. Where it comes
/// from is the probe's (`publish` has no probe).
#[cfg_attr(feature = "publish", allow(dead_code))]
pub(crate) struct Key {
    pub light: DirectionalLight,
    /// Every candidate and its share of the drawn light (0 for one not
    /// drawn), in candidate order.
    pub shares: Vec<(Source, f32)>,
    /// The og.DirectionalLight og.DayNight drives (its sun and moon took
    /// its place), when the cycle runs and the world has one.
    pub driven: Option<u64>,
    /// The og.DirectionalLights left out of the drawn light while they
    /// shine with at least a quarter of its radiance.
    pub left_out: Vec<u64>,
    /// How many og.DirectionalLights the world has.
    pub lights: usize,
}

#[cfg_attr(feature = "publish", allow(dead_code))]
impl Key {
    /// The share of the drawn light that comes from og.DayNight.
    fn cycle_share(&self) -> f32 {
        self.shares
            .iter()
            .filter(|(s, _)| matches!(s, Source::Sun | Source::Moon))
            .map(|(_, w)| w)
            .sum()
    }

    /// `entity`'s share of the drawn light: the cycle's for the light it
    /// drives, 0 for one not drawn.
    pub(crate) fn share_of(&self, entity: u64) -> f32 {
        if self.driven == Some(entity) {
            return self.cycle_share();
        }
        self.shares
            .iter()
            .find(|(s, _)| *s == Source::Entity(entity))
            .map_or(0.0, |(_, w)| *w)
    }
}

/// The world's og.DirectionalLights as the renderer's, their direction
/// from `world_of` (towards the light, against the entity's -Z) and every
/// setting not on the record from `base`, in spawn order; those `left_out`
/// names are not.
fn directional_lights(
    world: &World,
    layout: Option<&Layout>,
    world_of: &mut dyn FnMut(u64) -> Mat4,
    base: DirectionalLight,
    left_out: &dyn Fn(u64) -> bool,
) -> Vec<(u64, DirectionalLight)> {
    let mut found = Vec::new();
    for (e, v) in crate::present::fx::rows(world, layout) {
        let Ok(v) = record::DirectionalLight::from_json(&v) else {
            continue;
        };
        if left_out(e) {
            continue;
        }
        let dir = world_of(e)
            .transform_vector3(og_core::facing::CAMERA_FORWARD)
            .normalize_or_zero();
        found.push((
            e,
            DirectionalLight {
                direction_to_light: if dir == Vec3::ZERO { Vec3::Y } else { -dir },
                color: Vec3::new(v.color.r, v.color.g, v.color.b),
                intensity: v.intensity,
                mobility: crate::present::baked::mobility(v.mobility),
                cast_shadows: v.cast_shadow,
                max_shadow_distance: v.shadow_distance,
                shadow_box: v.shadow_box,
                shadow_focus: v.shadow_focus,
                shadow_focus_drop: v.shadow_focus_drop,
                shadow_bias: v.shadow_bias,
                shadow_normal_bias: v.shadow_normal_bias,
                shadow_softness: v.shadow_softness,
                cascade_blend: v.cascade_blend,
                ..base
            },
        ));
    }
    // Equal lights go to the one spawned first.
    og_core::sort::sort_by_key(&mut found, |(e, _)| Entity::from_bits(*e).index());
    found
}

/// The key light of `world` at its tick: `None` without a directional
/// light or a running og.DayNight. `base` gives the settings the records
/// leave out (and the cycle's shadow settings while it drives no light);
/// `baked` the static lights the drawn bakes hold.
pub(crate) fn key(
    world: &World,
    layouts: Layouts<'_>,
    world_of: &mut dyn FnMut(u64) -> Mat4,
    base: DirectionalLight,
    baked: &crate::present::baked::BakedState,
) -> Option<Key> {
    let cycle = crate::present::atmosphere::day_night_look(
        world,
        layouts.day_night,
        layouts.physical_sky,
        layouts.season,
    );
    let running = cycle.is_some();
    let found = directional_lights(world, layouts.directional, world_of, base, &|e| {
        !running && baked.baked_light(e)
    });
    let mut candidates: Vec<(Source, DirectionalLight)> =
        found.iter().map(|&(e, l)| (Source::Entity(e), l)).collect();
    let mut driven = None;
    if let Some((look, scale)) = cycle {
        // The cycle drives the light the lights alone draw: its sun and
        // moon take that light's place and its shadow settings.
        let lights: Vec<_> = found.iter().map(|(_, l)| *l).collect();
        let at = og_render::key_light(&lights).map(|k| k.source);
        let settings = at.map_or(base, |i| found[i].1);
        let [sun, moon] = look.lights(&settings, scale);
        let bodies = [(Source::Sun, sun), (Source::Moon, moon)];
        match at {
            Some(i) => {
                driven = Some(found[i].0);
                candidates.splice(i..=i, bodies);
            }
            None => candidates.extend(bodies),
        }
    }
    let lights: Vec<_> = candidates.iter().map(|(_, l)| *l).collect();
    let k = og_render::key_light(&lights)?;
    Some(Key {
        light: k.light,
        shares: candidates
            .iter()
            .enumerate()
            .map(|(i, (s, _))| (*s, k.share(i)))
            .collect(),
        driven,
        left_out: k
            .left_out
            .iter()
            .filter_map(|&i| match candidates[i].0 {
                Source::Entity(e) => Some(e),
                Source::Sun | Source::Moon => None,
            })
            .collect(),
        lights: found.len(),
    })
}
