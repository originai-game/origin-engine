//! A lightning strike drawn: its flash, its bolt and branches, its light.

use crate::ParticleBlend;
use glam::Vec3;

/// How bright a strike is `t` seconds after it: a sharp first stroke, a
/// dip, a second return stroke, then a fade (0 from 0.6 s).
pub fn lightning_flash(t: f32) -> f32 {
    let pulse = |c: f32, w: f32| (1.0 - ((t - c) / w).abs()).max(0.0);
    (pulse(0.03, 0.05)
        + 0.8 * pulse(0.17, 0.07)
        + 0.35 * (1.0 - t / 0.6).max(0.0) * f32::from(t > 0.05))
    .min(1.0)
        * f32::from(t < 0.6)
}

/// The bolt of strike number `strike`: a jagged line from (at.x, top, at.z)
/// down to `at` by midpoint displacement from a generator seeded by the
/// strike, so the same strike draws the same bolt. (point, width) pairs:
/// the channel's glow, not its centimetres of plasma, so a few thousandths
/// of the bolt's height (3.6 to 6 m for a 900 m bolt, a couple of pixels
/// at 1.5 km), widening towards the ground.
pub fn lightning_bolt(at: Vec3, top: f32, strike: u32) -> Vec<(Vec3, f32)> {
    // A stream seeded by the strike.
    let mut state = u64::from(strike) ^ 0x4c49_4748;
    let mut draw = move |r: f32| {
        let u = og_core::hash::unit24_of64(og_core::hash::splitmix64_next(&mut state));
        (u * 2.0 - 1.0) * r
    };
    let mut pts = vec![Vec3::new(at.x, top, at.z), at];
    let mut jag = (top - at.y) * 0.18;
    for _ in 0..6 {
        let mut next = Vec::with_capacity(pts.len() * 2);
        for pair in pts.windows(2) {
            let mid = (pair[0] + pair[1]) * 0.5 + Vec3::new(draw(jag), 0.0, draw(jag));
            next.push(pair[0]);
            next.push(mid);
        }
        next.push(*pts.last().unwrap_or(&at));
        pts = next;
        jag *= 0.55;
    }
    let n = pts.len() as f32;
    pts.into_iter()
        .enumerate()
        .map(|(i, p)| (p, (top - at.y) * (0.004 + 0.0028 * i as f32 / n)))
        .collect()
}

/// The side branches of strike number `strike`'s bolt ([`lightning_bolt`]):
/// one or two channels leaving the main one in its upper two thirds,
/// heading down and outwards for 25 to 40 % of the bolt's height, each
/// jagged by midpoint displacement as the main channel is (its jag the
/// same share of its length, six levels) and thinning to its tip, as
/// (point, width) pairs (the same strike, the same branches).
pub fn lightning_branches(at: Vec3, top: f32, strike: u32) -> Vec<Vec<(Vec3, f32)>> {
    let main = lightning_bolt(at, top, strike);
    let mut state = u64::from(strike) ^ 0x4252_4e43;
    let mut unit = move || og_core::hash::unit24_of64(og_core::hash::splitmix64_next(&mut state));
    let height = top - at.y;
    let count = 1 + (unit() > 0.4) as usize;
    (0..count)
        .map(|_| {
            let i = ((0.12 + 0.5 * unit()) * (main.len() - 1) as f32) as usize;
            let (from, width) = main[i];
            let angle = unit() * std::f32::consts::TAU;
            let reach = height * (0.25 + 0.15 * unit());
            let to = from + Vec3::new(angle.cos() * reach * 0.6, -reach, angle.sin() * reach * 0.6);
            let mut pts = vec![from, to];
            let mut jag = reach * 0.18;
            for _ in 0..6 {
                let mut next = Vec::with_capacity(pts.len() * 2);
                for pair in pts.windows(2) {
                    let d = Vec3::new(unit() * 2.0 - 1.0, 0.0, unit() * 2.0 - 1.0) * jag;
                    next.push(pair[0]);
                    next.push((pair[0] + pair[1]) * 0.5 + d);
                }
                next.push(to);
                pts = next;
                jag *= 0.55;
            }
            let n = (pts.len() - 1) as f32;
            pts.into_iter()
                .enumerate()
                .map(|(k, p)| (p, width * (0.7 - 0.5 * k as f32 / n)))
                .collect()
        })
        .collect()
}

/// A lightning strike `seconds` after it struck `at`, from a cloud base at
/// height `top`, added to `scene` (nothing from 0.6 s): the double flash
/// brightens the sky's light and turns the scene's directional light to
/// the bolt's channel while it outshines it, the clouds (when the scene
/// has them) glow from inside 150 m over the cloud base above the strike
/// ([`crate::Clouds::flash`]), a bright blue-white light hangs 100 m over
/// the strike, lighting the ground with a falloff from it, and for 0.35 s
/// the bolt of strike number `strike` ([`lightning_bolt`], with its
/// [`lightning_branches`]) is drawn as additive strips: a white core at
/// least [`BOLT_CORE`] of the view's height across wherever it is, so it
/// never thins to an aliased line, inside a violet-white halo four times as
/// wide that the bloom spreads; the branches dimmer.
pub fn add_lightning(
    scene: &mut crate::RenderScene,
    at: Vec3,
    top: f32,
    strike: u32,
    seconds: f32,
) {
    let flash = lightning_flash(seconds);
    if flash <= 0.0 {
        return;
    }
    scene.lighting.environment.intensity *= 1.0 + 6.0 * flash;
    // The bolt lights the land like a low sun for the flash's length, with
    // hard shadows away from it: the scene's directional light turns to the
    // channel's middle (half the cloud base up; the whole channel shines)
    // while the flash outshines it.
    let channel = at + Vec3::Y * (0.5 * top).max(150.0);
    let toward = (channel - scene.frame.camera.position).normalize_or(Vec3::Y);
    let bolt_light = 2.0 * flash;
    // The cloud over the root glows from inside (the clouds' own flash).
    if let Some(c) = scene.atmosphere.clouds.as_mut() {
        // In the cloud's underside over the bolt's top, a little towards the
        // camera, so the glow lights the base from inside where the channel
        // enters it rather than a ball in the deck's middle.
        let toward_eye = Vec3::new(
            scene.frame.camera.position.x - at.x,
            0.0,
            scene.frame.camera.position.z - at.z,
        )
        .normalize_or(Vec3::ZERO);
        let root = lightning_bolt(at, top, strike)[0].0;
        c.flash = (root + Vec3::Y * 40.0 + toward_eye * 60.0).extend(12.0 * flash);
    }
    let own = scene.lighting.light.intensity * scene.lighting.light.color.max_element();
    if bolt_light > own {
        let k = own / bolt_light;
        scene.lighting.light.direction_to_light = toward
            .lerp(scene.lighting.light.direction_to_light, k)
            .normalize_or(toward);
        scene.lighting.light.color = Vec3::new(0.78, 0.85, 1.0).lerp(scene.lighting.light.color, k);
        scene.lighting.light.intensity = bolt_light + own * k;
    }
    scene.lighting.lights.push(crate::LightDesc {
        kind: crate::LightKind::Point,
        position: at + Vec3::Y * 100.0,
        color: Vec3::new(0.75, 0.82, 1.0),
        intensity: 2.0e4 * flash,
        range: 600.0,
        radius: 0.0,
        cast_shadows: false,
        mobility: crate::LightMobility::Movable,
    });
    if seconds < 0.35 {
        // The core's least width at each point: BOLT_CORE of the view's
        // height at its distance (a vertical field of view's tangent).
        let fov = match scene.frame.camera.projection {
            crate::Projection::Perspective { fov_y_degrees }
            | crate::Projection::PerspectiveShifted { fov_y_degrees, .. } => {
                fov_y_degrees.to_radians()
            }
            _ => 1.0,
        };
        let eye = scene.frame.camera.position;
        let span = 2.0 * (fov * 0.5).tan() * BOLT_CORE;
        // Bright enough to bloom, not so bright the bloom and its
        // reflection burn a white column across the frame.
        let core = [
            0.9 + 2.2 * flash,
            0.95 + 2.4 * flash,
            1.2 + 2.8 * flash,
            1.0,
        ];
        let halo = [
            0.3 + 0.5 * flash,
            0.28 + 0.5 * flash,
            0.5 + 0.9 * flash,
            1.0,
        ];
        // The halo widens and brightens towards the deck, where the channel
        // enters the lit cloud (twice as wide and half again as bright at the
        // base as at the ground), so the bloom spreads it into the glow.
        let height = (top - at.y).max(1.0);
        let mut strip = |pts: &[(Vec3, f32)], gain: f32, halo: bool, c: [f32; 4]| {
            let points: Vec<crate::RibbonPoint> = pts
                .iter()
                .map(|&(position, width)| {
                    let up = ((position.y - at.y) / height).clamp(0.0, 1.0);
                    let (scale, lift) = if halo {
                        (4.0 + 4.0 * up, 1.0 + 0.5 * up)
                    } else {
                        (1.0, 1.0)
                    };
                    let g = gain * lift;
                    crate::RibbonPoint {
                        position,
                        width: width.max(span * position.distance(eye)) * scale,
                        color: [c[0] * g, c[1] * g, c[2] * g, 1.0],
                    }
                })
                .collect();
            // A branch fades before its points are thinned, so the thinning
            // keeps the narrowing end's points (spaced by their own width):
            // faded after it, the halo's wide strip kept one segment over
            // its last 30 %, and its width fell along a straight line to a
            // hard diagonal edge.
            let mut points = points;
            if gain < 1.0 {
                taper(&mut points);
            }
            let points = strokeable(points);
            scene.effects.ribbons.push(crate::Ribbon {
                points,
                texture: None,
                uv: crate::RibbonUv::Stretch,
                blend: ParticleBlend::Additive,
                effect: None,
            });
        };
        let main = lightning_bolt(at, top, strike);
        let branches = lightning_branches(at, top, strike);
        for (pts, gain) in std::iter::once((&main, 1.0)).chain(branches.iter().map(|b| (b, 0.6))) {
            strip(pts, gain, true, halo);
            strip(pts, gain, false, core);
        }
    }
}

/// A strip's points with none closer to the last kept than [`STROKE_STEP`]
/// times its width there (the narrower of the segment's two ends, so a
/// fading end keeps its points): a camera-facing strip folds over itself where its
/// segments are shorter than it is wide (each point's edge is across the
/// tangent through its neighbours, which swings at every jag), and the
/// folds drew a bolt's core and halo as short ticks across the stroke, a
/// fishbone. Jags finer than the stroke cannot show anyway. The last point
/// is kept (in place of the one before it when that is too near).
fn strokeable(points: Vec<crate::RibbonPoint>) -> Vec<crate::RibbonPoint> {
    let mut out: Vec<crate::RibbonPoint> = Vec::with_capacity(points.len());
    let n = points.len();
    for (i, p) in points.into_iter().enumerate() {
        match out.last() {
            None => out.push(p),
            Some(last)
                if last.position.distance(p.position) >= STROKE_STEP * last.width.min(p.width) =>
            {
                out.push(p)
            }
            Some(_) if i + 1 == n => {
                if out.len() > 1 {
                    out.pop();
                }
                out.push(p);
            }
            Some(_) => {}
        }
    }
    out
}

/// A branch's end fades out, width and brightness to nothing over the last
/// [`TAPER`] of its length: it ended on a stub at an angle (its kept last
/// point), which read as a kink rather than a channel dying out.
fn taper(points: &mut [crate::RibbonPoint]) {
    let lens: Vec<f32> = points
        .windows(2)
        .map(|w| w[0].position.distance(w[1].position))
        .collect();
    let total: f32 = lens.iter().sum();
    if total <= 0.0 {
        return;
    }
    let mut along = 0.0;
    for (i, p) in points.iter_mut().enumerate() {
        let t = ((along / total - (1.0 - TAPER)) / TAPER).clamp(0.0, 1.0);
        let keep = 1.0 - t * t * (3.0 - 2.0 * t);
        p.width *= keep;
        for c in &mut p.color[..3] {
            *c *= keep;
        }
        along += lens.get(i).copied().unwrap_or(0.0);
    }
}

/// The share of a branch's length its end fades over ([`taper`]).
const TAPER: f32 = 0.3;

/// A drawn bolt's least segment, in its strip's widths ([`strokeable`]).
const STROKE_STEP: f32 = 2.5;

/// The least width of a bolt's white core, as a share of the view's height
/// (3 pixels of 720): [`add_lightning`].
pub const BOLT_CORE: f32 = 3.0 / 720.0;
