//! og.Sprite.sway: a 2D plant drawn as the parts its texture was cooked
//! into (og-cook's sway2d.rs, the `<id>#sway` beside the texture), each
//! moved by og.Wind where it stands. The wood stands still (a conifer's
//! trunk bends in rows), every clump of leaves or tier of branches leans
//! downwind with the plant and walks a small loop (up, downwind, down,
//! back) in the wind's travelling wave, so the wave runs through the crown
//! and on from plant to plant, in step with the grass and the trees
//! (`og_render::Wind::lean_at`: the lean every vegetation tier takes),
//! as far as a crown gives ([`give`]: it stays over its trunk in a gale).
//! Pixel art moves by whole texels and holds each pose. Presentation only:
//! a function of the tick, never read by it.

use glam::{Vec2, Vec3};
use og_bundle::runtime::SwayParts;
use og_render::{Sprite, TextureHandle, Wind};

/// A texture's wind parts in the renderer.
pub(crate) struct SwayTexture {
    /// The `#sway` texture: the parts side by side, and its size (texels).
    pub texture: TextureHandle,
    pub atlas: [u32; 2],
    pub parts: SwayParts,
    /// Pixel art: its parts move by whole texels and hold each pose.
    pub pixel: bool,
}

/// How a swaying sprite stands in the wind this frame.
pub(crate) struct Stance<'a> {
    pub wind: &'a Wind,
    pub tick: u64,
    pub rate: f32,
    /// `ArtMotion.stepFps`: the poses a second the style's motion holds
    /// (0: none of its own).
    pub step_fps: f32,
    /// The world axes the quad's x and y run along before its rotation
    /// ([`og_render::face_axes`]).
    pub axes: (Vec3, Vec3),
    /// The quad lies in the world's xy plane (no facing): a 2D world,
    /// whose x and y the wind's gusts cross. A faced sprite stands on the
    /// ground of a 3D one (x and z).
    pub plane: bool,
}

/// The poses a second pixel art holds without an `ArtMotion.stepFps`: a
/// hand-animated loop's.
const POSE_HZ: f32 = 10.0;
/// How far a plant's crown goes with the wind, over a blade of grass of
/// its height: `sway` 1 in a fresh breeze (8 m/s) moves a 60-texel tree's
/// leaves 3 texels, 6 in the gusts.
const STIFF: f32 = 0.125;
/// The most of its height a crown goes with the wind at `sway` 1, however
/// hard it blows: it stays over its trunk in a gale.
const MOST: f32 = 0.07;

/// How far a crown goes for the wind's lean, as a share of the plant's
/// height: [`STIFF`] of it in a breeze, levelling off at [`MOST`].
fn give(lean: f32) -> f32 {
    // MOST tanh(lean STIFF / MOST), of the engine's own exp: the same on
    // every host.
    let e = og_core::math::exp(-2.0 * lean.max(0.0) * STIFF / MOST);
    MOST * (1.0 - e) / (1.0 + e)
}
/// A part's loop across, over the plant's lean where it stands.
const LOOP: f32 = 0.6;
/// The loop's rise over its width.
const LIFT: f32 = 0.4;

/// `tick` held to `hz` poses a second.
fn held(tick: u64, rate: f32, hz: f32) -> u64 {
    let per = f64::from(rate.max(1e-3)) / f64::from(hz);
    ((tick as f64 / per).floor() * per) as u64
}

/// `base` (a sprite drawn whole from the texture `tex` was cut from,
/// `flip` its flips: 1 x, 2 y) as its parts in the wind, in drawing order:
/// `out(part, stands)`, `stands` for the wood and the dark copy under the
/// leaves (which never overlap one another: what casts the drop shadow).
/// Each part is the same quad about the same anchor, so it turns, faces
/// and sorts as the sprite does; `sway` scales how far the wind moves it
/// ([`give`] of the wind's lean, times the plant's drawn height).
pub(crate) fn parts(
    base: &Sprite,
    flip: u8,
    sway: f32,
    tex: &SwayTexture,
    s: &Stance,
    mut out: impl FnMut(Sprite, bool),
) {
    let [w, h] = tex.parts.size.map(|v| v as f32);
    let [aw, ah] = tex.atlas.map(|v| v as f32);
    let texel = base.size / Vec2::new(w, h);
    let hz = match (s.step_fps > 0.0, tex.pixel) {
        (true, _) => s.step_fps,
        (false, true) => POSE_HZ,
        (false, false) => 0.0,
    };
    let tick = if hz > 0.0 {
        held(s.tick, s.rate, hz)
    } else {
        s.tick
    };
    let (x_axis, y_axis) = s.axes;
    let ground = |p: Vec3| {
        if s.plane {
            p.truncate()
        } else {
            Vec2::new(p.x, p.z)
        }
    };
    // How much of the wind blows along the quad.
    let direction = Vec3::new(s.wind.direction.x, 0.0, s.wind.direction.z).normalize_or_zero();
    let along = direction.dot(x_axis);
    let reach = sway * base.size.y;
    // The plant leans as one, by the wind at its foot.
    let foot = give(
        s.wind
            .lean_at(ground(base.position), tick, s.rate)
            .0
            .length(),
    ) * along
        * reach;
    let (sin, cos) = og_core::math::sin_cos(base.rotation);
    let (flip_x, flip_y) = (flip & 1 != 0, flip & 2 != 0);
    for p in &tex.parts.parts {
        let size = Vec2::new(p.size[0] as f32, p.size[1] as f32);
        // Where it lies on the quad as drawn: a flip mirrors the canvas.
        let mirrored = |v: f32, span: f32, flipped: bool| if flipped { span - v } else { v };
        let left = mirrored(
            p.at[0] as f32 + if flip_x { size.x } else { 0.0 },
            w,
            flip_x,
        );
        let top = mirrored(
            p.at[1] as f32 + if flip_y { size.y } else { 0.0 },
            h,
            flip_y,
        );
        let centre = Vec2::new(
            mirrored(p.centre[0], w, flip_x),
            mirrored(p.centre[1], h, flip_y),
        );
        // Its centre in the world: from the anchor in the quad's frame,
        // turned, along the quad's axes.
        let local = Vec2::new(centre.x / w, 1.0 - centre.y / h) - base.anchor;
        let local = local * base.size;
        let turned = Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
        let at = base.position + x_axis * turned.x + y_axis * turned.y;
        let (lean, phase) = s.wind.lean_at(ground(at), tick, s.rate);
        let swing = p.flutter * LOOP * give(lean.length()) * reach;
        let mut moved = Vec2::new(
            foot * p.lean + swing * along * (0.5 - 0.5 * og_core::math::cos(phase)),
            // The wave's phase falls as the tick goes on: up first, then
            // downwind, down and back.
            swing * LIFT * (-og_core::math::sin(phase)).max(0.0),
        );
        if tex.pixel {
            moved = (moved / texel).round() * texel;
        }
        // Downwind and up in the world; the anchor moves in the quad's own
        // turned frame.
        let moved = Vec2::new(
            moved.x * cos + moved.y * sin,
            -moved.x * sin + moved.y * cos,
        );
        let part_size = size * texel;
        let anchor = Vec2::new(
            (base.anchor.x * w - left) / size.x,
            (base.anchor.y * h - (h - top - size.y)) / size.y,
        ) - moved / part_size;
        let (u0, v0) = (p.atlas[0] as f32 / aw, p.atlas[1] as f32 / ah);
        let (u1, v1) = (u0 + size.x / aw, v0 + size.y / ah);
        let mut uv = [u0, v0, u1, v1];
        if flip_x {
            uv.swap(0, 2);
        }
        if flip_y {
            uv.swap(1, 3);
        }
        out(
            Sprite {
                texture: Some(tex.texture),
                size: part_size,
                anchor,
                uv_rect: uv,
                ..*base
            },
            p.flutter == 0.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_bundle::runtime::SwayPart;

    /// A 40 x 80 texel plant, 2.5 x 5 world units (16 texels a unit): its
    /// wood, then a clump at the crown's left edge and a stiffer one in
    /// its middle.
    fn plant(pixel: bool) -> SwayTexture {
        let part = |at: [u32; 2], size: [u32; 2], lean: f32, flutter: f32| SwayPart {
            atlas: [at[0] + 100, at[1]],
            size,
            at,
            centre: [
                at[0] as f32 + size[0] as f32 / 2.0,
                at[1] as f32 + size[1] as f32 / 2.0,
            ],
            lean,
            flutter,
        };
        SwayTexture {
            texture: TextureHandle::from_index(7),
            atlas: [200, 100],
            parts: SwayParts {
                size: [40, 80],
                parts: vec![
                    part([16, 30], [8, 50], 0.0, 0.0),
                    part([0, 10], [16, 20], 1.0, 1.0),
                    part([12, 0], [16, 24], 0.4, 0.4),
                ],
            },
            pixel,
        }
    }

    fn base() -> Sprite {
        Sprite {
            texture: Some(TextureHandle::from_index(3)),
            position: Vec3::new(10.0, 2.0, 0.0),
            size: Vec2::new(2.5, 5.0),
            anchor: Vec2::new(0.5, 0.0),
            ..Sprite::default()
        }
    }

    /// Each part's bottom-left corner in the world and its size, for a
    /// sprite in the xy plane that is not turned.
    fn corners(
        tex: &SwayTexture,
        b: &Sprite,
        flip: u8,
        sway: f32,
        s: &Stance,
    ) -> Vec<(Vec2, Vec2)> {
        let mut out = Vec::new();
        parts(b, flip, sway, tex, s, |p, _| {
            assert_eq!(p.texture, Some(tex.texture));
            out.push((p.position.truncate() - p.anchor * p.size, p.size));
        });
        out
    }

    fn stance(wind: &Wind, tick: u64) -> Stance<'_> {
        Stance {
            wind,
            tick,
            rate: 60.0,
            step_fps: 0.0,
            axes: (Vec3::X, Vec3::Y),
            plane: true,
        }
    }

    /// Ways this could fail: a part placed from the wrong corner or row
    /// (the texture's rows run down, the quad's up), its texels scaled by
    /// the atlas instead of the canvas, or a flip mirroring the texels and
    /// not the place.
    #[test]
    fn parts_at_rest_lie_where_the_texture_draws_them() {
        let tex = plant(true);
        // A wind too light to move a texel: every part rests.
        let wind = Wind {
            strength: 1e-4,
            ..Wind::default()
        };
        let b = base();
        // The quad's bottom-left is (8.75, 2); a texel is 1/16.
        let got = corners(&tex, &b, 0, 1.0, &stance(&wind, 40));
        let want = [
            (Vec2::new(8.75 + 1.0, 2.0), Vec2::new(0.5, 3.125)),
            (Vec2::new(8.75, 2.0 + 50.0 / 16.0), Vec2::new(1.0, 1.25)),
            (Vec2::new(8.75 + 0.75, 2.0 + 3.5), Vec2::new(1.0, 1.5)),
        ];
        for (g, w) in got.iter().zip(want) {
            assert!(
                (g.0 - w.0).length() < 1e-5 && (g.1 - w.1).length() < 1e-6,
                "{g:?} {w:?}"
            );
        }
        // Flipped left to right about the anchor's column: the left clump
        // is the right one, its texels mirrored.
        let mut flipped = b;
        flipped.uv_rect.swap(0, 2);
        let got = corners(&tex, &flipped, 1, 1.0, &stance(&wind, 40));
        assert!(
            (got[1].0 - Vec2::new(8.75 + 1.5, 5.125)).length() < 1e-5,
            "{:?}",
            got[1]
        );
        let mut uv = [0.0; 4];
        parts(&flipped, 1, 1.0, &tex, &stance(&wind, 40), |p, stands| {
            if !stands && p.size.x == 1.0 && p.size.y == 1.25 {
                uv = p.uv_rect;
            }
        });
        for (got, want) in uv.iter().zip([0.58, 0.1, 0.5, 0.3]) {
            assert!((got - want).abs() < 1e-6, "{uv:?}");
        }
    }

    /// Ways this could fail: the wood moving, a part moved upwind or
    /// down, the stiff clump moving as far as the loose one, pixel art off
    /// its texels or changing pose between its steps, or `sway` ignored.
    #[test]
    fn the_wind_moves_the_leaves_downwind_by_whole_texels_and_the_wood_stays() {
        let tex = plant(true);
        let wind = Wind {
            strength: 0.2,
            gust: 0.0,
            ..Wind::default()
        };
        let b = base();
        let rest = corners(
            &tex,
            &b,
            0,
            1.0,
            &stance(
                &Wind {
                    strength: 1e-4,
                    ..wind
                },
                0,
            ),
        );
        let texel = 1.0 / 16.0;
        let mut loose_most = 0.0f32;
        let mut poses = std::collections::BTreeSet::new();
        for tick in 0..240 {
            let got = corners(&tex, &b, 0, 4.0, &stance(&wind, tick));
            assert_eq!(got[0], rest[0], "the wood stands");
            for k in 1..3 {
                let d = (got[k].0 - rest[k].0) / texel;
                assert!((d - d.round()).length() < 1e-3, "{tick} {k}: {d}");
                assert!(d.x >= -1e-3 && d.y >= -1e-3, "{tick} {k}: {d}");
            }
            let (loose, stiff) = (got[1].0 - rest[1].0, got[2].0 - rest[2].0);
            assert!(loose.x >= stiff.x - 1e-5);
            loose_most = loose_most.max(loose.x);
            // Held: six ticks a pose at 10 a second.
            if tick % 6 != 0 {
                assert_eq!(
                    got,
                    corners(&tex, &b, 0, 4.0, &stance(&wind, tick - tick % 6))
                );
            }
            poses.insert((
                (loose.x / texel).round() as i32,
                (loose.y / texel).round() as i32,
            ));
        }
        // The lean is 0.2 of 5 units, an eighth of it a crown's less the
        // 4 % its limit takes, at sway 4: 0.48 units, 7.7 texels, and
        // the loop 0.6 of that on top: 8 to 12 texels across, up to 2 up.
        assert!(
            (11.5 * texel..12.5 * texel).contains(&loose_most),
            "{loose_most}"
        );
        assert!(
            poses
                .iter()
                .all(|p| (8..=12).contains(&p.0) && (0..=2).contains(&p.1)),
            "{poses:?}"
        );
        assert!(
            poses.len() >= 6 && poses.iter().any(|p| p.1 == 2),
            "{poses:?}"
        );
        // The wind from the other side moves them the other way.
        let back = Wind {
            direction: Vec3::NEG_X,
            ..wind
        };
        let got = corners(&tex, &b, 0, 4.0, &stance(&back, 30));
        assert!(got[1].0.x < rest[1].0.x - 7.5 * texel);
        // A gale (20 m/s, gusting to twice that) at sway 1: the crown goes
        // further than in the breeze but stays over its trunk, 0.07 of the
        // plant's 80 texels and the loop on top at the most.
        let gale = Wind {
            strength: 1.0,
            gust: 1.0,
            ..wind
        };
        let most = (0..240)
            .map(|tick| corners(&tex, &b, 0, 1.0, &stance(&gale, tick))[1].0.x - rest[1].0.x)
            .fold(0.0, f32::max);
        assert!((5.0 * texel..=9.0 * texel).contains(&most), "{most}");
    }

    /// A turned or faced sprite's parts move downwind in the world, not
    /// along the quad; smooth art moves between texels.
    #[test]
    fn a_faced_smooth_sprite_leans_by_the_wind_along_its_quad() {
        let tex = plant(false);
        let wind = Wind {
            strength: 0.2,
            gust: 0.0,
            direction: Vec3::new(0.0, 0.0, 1.0),
            ..Wind::default()
        };
        let b = base();
        // An upright card whose x runs along the world's z, the wind's way.
        let faced = Stance {
            axes: (Vec3::Z, Vec3::Y),
            plane: false,
            ..stance(&wind, 17)
        };
        let across = Stance {
            axes: (Vec3::X, Vec3::Y),
            plane: false,
            ..stance(&wind, 17)
        };
        let mut moved = Vec::new();
        parts(&b, 0, 4.0, &tex, &faced, |p, _| moved.push(p.anchor));
        let mut still = Vec::new();
        parts(&b, 0, 4.0, &tex, &across, |p, _| still.push(p.anchor));
        // The loose clump's anchor moves back as the clump moves along the
        // quad: 0.48 units of lean and more, over its width of 1.
        assert!(
            still[1].x - moved[1].x > 0.47,
            "{:?} {:?}",
            still[1],
            moved[1]
        );
        // Across the wind nothing leans; the wood never does.
        assert!((still[1].x - 20.0 / 16.0).abs() < 1e-5, "{:?}", still[1]);
        assert_eq!(moved[0], still[0]);
        let texels = (still[1].x - moved[1].x) * 16.0;
        assert!(
            (texels - texels.round()).abs() > 1e-3,
            "smooth art is not held to texels"
        );
    }
}
