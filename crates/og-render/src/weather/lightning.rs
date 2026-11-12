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
    // A xorshift stream seeded by the strike.
    let mut state = strike.wrapping_mul(0x9e37_79b9) ^ 0x4c49_4748 | 1;
    let mut draw = move |r: f32| {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        (state as f32 / u32::MAX as f32 * 2.0 - 1.0) * r
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
    let mut state = strike.wrapping_mul(0x85eb_ca6b) ^ 0x4252_4e43 | 1;
    let mut unit = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
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
    scene.lighting.lights.push(crate::Light {
        kind: crate::LightKind::Point,
        position: at + Vec3::Y * 100.0,
        color: Vec3::new(0.75, 0.82, 1.0),
        intensity: 2.0e4 * flash,
        range: 600.0,
        cast_shadows: false,
    });
    if seconds < 0.35 {
        // The core's least width at each point: BOLT_CORE of the view's
        // height at its distance (a vertical field of view's tangent).
        let fov = match scene.frame.camera.projection {
            crate::Projection::Perspective { fov_y_degrees } => fov_y_degrees.to_radians(),
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

#[cfg(test)]
mod tests {
    use super::*;
    /// The flash: bright at once, a second stroke after the dip, dark from
    /// 0.6 s. The bolt runs from the cloud to the strike, the same for the
    /// same strike and different for the next.
    #[test]
    fn a_strike_flashes_twice_and_draws_its_own_bolt() {
        assert!(lightning_flash(0.03) > 0.9);
        assert!(lightning_flash(0.17) > lightning_flash(0.11));
        assert_eq!(lightning_flash(0.6), 0.0);
        let at = Vec3::new(10.0, 2.0, -40.0);
        let a = lightning_bolt(at, 602.0, 3);
        assert_eq!(a.first().unwrap().0, Vec3::new(10.0, 602.0, -40.0));
        assert_eq!(a.last().unwrap().0, at);
        assert_eq!(a.len(), 65);
        assert_eq!(a, lightning_bolt(at, 602.0, 3));
        assert_ne!(a, lightning_bolt(at, 602.0, 4));
        // One or two branches, leaving the channel and running down from it,
        // the same for the same strike.
        let b = lightning_branches(at, 602.0, 3);
        assert!((1..=2).contains(&b.len()));
        for br in &b {
            assert!(a.iter().any(|p| p.0 == br[0].0));
            // At least 8 % of the parent's height: no stub.
            assert!(br[0].0.y - br.last().unwrap().0.y >= 0.08 * 600.0);
            let (start, end) = (br[0].0, br.last().unwrap().0);
            assert!(end.y < start.y - 600.0 * 0.24, "{start} {end}");
            // Jagged: its path at least 15 % longer than its chord.
            let path: f32 = br.windows(2).map(|w| w[0].0.distance(w[1].0)).sum();
            assert!(
                path > 1.15 * start.distance(end),
                "{path} vs {}",
                start.distance(end)
            );
            assert!(end.y > at.y);
        }
        assert_eq!(b, lightning_branches(at, 602.0, 3));
    }

    /// The bolt's core stays BOLT_CORE of the view wide however far off,
    /// with a halo four times it around it: at 1.5 km under a 60° view the
    /// core is some 3 pixels of 720, not the 1.5 its own width gives.
    #[test]
    fn a_far_bolt_keeps_a_core_and_a_halo() {
        let mut scene = crate::RenderScene::new();
        scene.frame.camera.projection = crate::Projection::Perspective {
            fov_y_degrees: 60.0,
        };
        scene.frame.camera.position = Vec3::new(0.0, 2.0, 0.0);
        let at = Vec3::new(-150.0, 0.0, -1500.0);
        add_lightning(&mut scene, at, 900.0, 3, 0.03);
        assert!(
            scene.effects.ribbons.len() >= 4,
            "{}",
            scene.effects.ribbons.len()
        );
        let (halo, core) = (&scene.effects.ribbons[0], &scene.effects.ribbons[1]);
        let p = core.points.last().unwrap();
        let px = p.width
            / (2.0 * (30f32).to_radians().tan() * p.position.distance(scene.frame.camera.position))
            * 720.0;
        assert!((2.9..4.0).contains(&px), "{px} px");
        // Four times the core at the ground, eight at the deck.
        // (Both strips keep the bolt's two ends.)
        let ground = halo.points.last().unwrap().width / core.points.last().unwrap().width;
        let deck = halo.points[0].width / core.points[0].width;
        assert!(
            (ground - 4.0).abs() < 1e-3 && (deck - 8.0).abs() < 1e-3,
            "{ground} {deck}"
        );
        println!(
            "points: core {}, halo {}",
            core.points.len(),
            halo.points.len()
        );
        // No strip folds: every segment of every strip (core, halo, branches)
        // is at least STROKE_STEP of its width long, save a strip's last,
        // and still runs from the cloud to the ground.
        for r in &scene.effects.ribbons {
            let p = &r.points;
            assert!(p.len() >= 3, "{}", p.len());
            for w in p[..p.len() - 1].windows(2) {
                let len = w[0].position.distance(w[1].position);
                assert!(
                    len >= STROKE_STEP * w[0].width.min(w[1].width) - 1e-3,
                    "{len} vs {} {}",
                    w[0].width,
                    w[1].width
                );
            }
        }
        assert_eq!(core.points[0].position.y, 900.0);
        // The branches (the strips after the main channel's two) fade out to
        // nothing at their ends, from full width over their first 70 %.
        for r in &scene.effects.ribbons[2..] {
            let last = r.points.last().unwrap();
            assert_eq!(last.width, 0.0);
            assert_eq!(last.color[..3], [0.0; 3]);
            assert!(r.points[1].width > 0.5 * r.points[0].width);
            // And smoothly: several points over its fading end, the halo's too.
            let fading = r
                .points
                .iter()
                .filter(|p| p.width < 0.9 * r.points[0].width)
                .count();
            assert!(fading >= 4, "{fading} of {} points fade", r.points.len());
        }
        assert_eq!(core.points.last().unwrap().position, at);
    }
}
