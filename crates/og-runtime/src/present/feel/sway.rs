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
