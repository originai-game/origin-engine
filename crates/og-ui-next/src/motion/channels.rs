//! Animatable skin channels, OKLab colour packing, and shadow/transform
//! lane codecs.

use super::*;

/// Eight lanes: every channel's track has the same shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct L(pub(super) [f32; 8]);

impl Animatable for L {
    const LANES: usize = 8;
    fn write_lanes(&self, out: &mut [f32]) {
        out[..8].copy_from_slice(&self.0);
    }
    fn read_lanes(l: &[f32]) -> Self {
        let mut v = [0.0; 8];
        v.copy_from_slice(&l[..8]);
        L(v)
    }
}

/// A skin's animated values, one track each.
pub(super) const CHANNELS: usize = 15 + TEXT_SHADOWS - 1;
pub(super) const OPACITY_CH: usize = 0;
pub(super) const RADIUS_CH: usize = 5;
pub(super) const TRANSFORM_CH: usize = 6;
/// The first box shadow's channel; the other three follow.
pub(super) const SHADOW_CH: usize = 7;
/// The first text-shadow layer's channel; the others follow.
pub(super) const TEXT_SHADOW_CH: usize = SHADOW_CH + SHADOWS;
pub(super) const BLUR_CH: usize = TEXT_SHADOW_CH + TEXT_SHADOWS;
pub(super) const SKEW_CH: usize = BLUR_CH + 1;
pub(super) const WIPE_CH: usize = SKEW_CH + 1;
const _: () = assert!(WIPE_CH + 1 == CHANNELS);
const _: () = assert!(
    CHANNELS < 32,
    "tracks is a u32 with FRAME_BIT above the channels"
);

/// Each channel's transition group and settle class.
pub(super) const GROUPS: [(u8, PropClass); CHANNELS] = [
    (OPACITY, PropClass::Opacity),
    (COLORS, PropClass::Color), // fill
    (COLORS, PropClass::Color), // ink
    (COLORS, PropClass::Color), // border colour
    (COLORS, PropClass::Color), // outline colour
    (SHAPE, PropClass::Shape),
    (TRANSFORM, PropClass::Scale),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color), // text shadow layers
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Color),
    (SHADOW, PropClass::Blur),          // backdrop blur
    (TRANSFORM, PropClass::Scale),      // skew (x, y)
    (LAYOUT | SHAPE, PropClass::Shape), // radial image wipe
];

pub(super) const PROPS: [Prop<L>; CHANNELS] = {
    let mut p = [Prop::custom(64, PropClass::Other); CHANNELS];
    let mut k = 0;
    while k < CHANNELS {
        p[k] = Prop::custom(64 + k as u16, GROUPS[k].1);
        k += 1;
    }
    p
};

/// Premultiplied OKLab lanes of a linear colour.
fn lab(c: Rgba) -> [f32; 4] {
    let [l, a, b] = linear_to_oklab(c[0], c[1], c[2]);
    let al = c[3];
    [l * al, a * al, b * al, al]
}

fn unlab(v: &[f32]) -> Rgba {
    let al = v[3].clamp(0.0, 1.0);
    if al <= 1e-6 {
        return [0.0; 4];
    }
    let [r, g, b] = oklab_to_linear([v[0] / al, v[1] / al, v[2] / al]);
    [r.max(0.0), g.max(0.0), b.max(0.0), al]
}

fn shadow_lanes(s: &Shadow) -> [f32; 8] {
    let c = lab(s.color);
    [
        s.offset[0],
        s.offset[1],
        s.blur,
        s.spread,
        c[0],
        c[1],
        c[2],
        c[3],
    ]
}

fn put_shadow(s: &mut Shadow, v: &[f32; 8]) {
    s.offset = [v[0], v[1]];
    s.blur = v[2].max(0.0);
    s.spread = v[3];
    s.color = unlab(&v[4..]);
}

pub(super) fn read(ch: usize, s: &Skin) -> L {
    let mut v = [0.0; 8];
    match ch {
        OPACITY_CH => v[0] = s.opacity,
        1..=4 => {
            let c = [s.fill, s.ink, s.border_color, s.outline.color][ch - 1];
            v[..4].copy_from_slice(&lab(c));
        }
        RADIUS_CH => v[..4].copy_from_slice(&s.radius),
        TRANSFORM_CH => {
            let t = &s.transform;
            v[..7].copy_from_slice(&[
                t.translate[0],
                t.translate[1],
                t.scale[0],
                t.scale[1],
                t.rotate,
                t.percent[0],
                t.percent[1],
            ]);
        }
        BLUR_CH => v[0] = s.backdrop_blur,
        SKEW_CH => {
            v[0] = s.transform.skew[0];
            v[1] = s.transform.skew[1];
        }
        WIPE_CH => v[0] = s.image.wipe,
        TEXT_SHADOW_CH.. => v = shadow_lanes(&s.text_shadows.layers[ch - TEXT_SHADOW_CH]),
        _ => v = shadow_lanes(&s.shadows[ch - SHADOW_CH]),
    }
    L(v)
}

pub(super) fn write(ch: usize, s: &mut Skin, L(v): &L) {
    match ch {
        OPACITY_CH => s.opacity = v[0].clamp(0.0, 1.0),
        1 => s.fill = unlab(v),
        2 => s.ink = unlab(v),
        3 => s.border_color = unlab(v),
        4 => s.outline.color = unlab(v),
        RADIUS_CH => s.radius = [v[0], v[1], v[2], v[3]].map(|r| r.max(0.0)),
        TRANSFORM_CH => {
            let t = &mut s.transform;
            (t.translate, t.scale, t.rotate) = ([v[0], v[1]], [v[2], v[3]], v[4]);
            t.percent = [v[5], v[6]];
        }
        BLUR_CH => s.backdrop_blur = v[0].max(0.0),
        SKEW_CH => s.transform.skew = [v[0], v[1]],
        WIPE_CH => s.image.wipe = v[0].clamp(0.0, 1.0),
        TEXT_SHADOW_CH.. => {
            let (k, t) = (ch - TEXT_SHADOW_CH, &mut s.text_shadows);
            put_shadow(&mut t.layers[k], v);
            // As a box shadow's: a layer only one side has draws while it
            // moves; its ink is the resting skin's.
            t.count = t.count.max(k as u8 + 1);
        }
        _ => {
            let k = ch - SHADOW_CH;
            put_shadow(&mut s.shadows[k], v);
            // A layer only one side has draws while it moves (from or to
            // transparent, as CSS pads the shorter list).
            s.shadow_count = s.shadow_count.max(k as u8 + 1);
        }
    }
}

pub(super) fn rect(b: &BoxLayout) -> Rect {
    Rect::from_edges(b.x, b.y, b.x + b.w, b.y + b.h)
}

pub(super) fn node(id: Id) -> NodeId {
    NodeId(id as u64)
}

pub(super) const NONE: u32 = u32::MAX;
pub(super) const FRAME_BIT: u32 = 1 << CHANNELS;
