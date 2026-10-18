//! From a laid-out frame to og-render's overlay ([`og_render::Overlay`]).
//!
//! Every rectangle (solid, rounded, a texture-less image) is a 9-slice of one
//! small atlas texture (a white disc, [`atlas_texture`]), so all of them
//! share one sprite run; images with their own texture and MSDF text are
//! the other runs. og-render sorts by (layer, kind, texture), so draw order is
//! kept with **painter levels**: a command's level is one above every earlier
//! command it overlaps that batches differently, and equal to that of an
//! overlapping earlier command of its own batch. Commands of different
//! batches on one level never overlap, so the sort cannot reorder anything
//! visible, and a typical screen is two to four draws.
//!
//! **Clipping is per pixel**: every sprite and text carries the index of its
//! clip rectangle in [`og_render::Overlay::clips`], and the overlay pass
//! discards what falls outside it, so a scrolled row, a rotated chevron or a
//! half-visible line of text is cut exactly at the container's edge.

use crate::layout::Rect;
use og_core::glam::{Vec2, Vec3};
use og_render::{ColorSpace, FontHandle, Mips, Overlay, Sprite, Text, TextureData, TextureHandle};

/// One thing to draw, in CSS pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    /// A rectangle with rounded corners (radius 0: square).
    Quad {
        rect: Rect,
        radius: f32,
        /// Linear RGBA.
        color: [f32; 4],
        clip: Rect,
    },
    /// A textured rectangle (`texture: None` is solid).
    Image {
        rect: Rect,
        texture: Option<TextureHandle>,
        uv: [f32; 4],
        color: [f32; 4],
        clip: Rect,
    },
    /// A solid bar `w`×`h` centred at `center`, rotated clockwise on screen by
    /// `angle` radians (chevrons, ticks, crosses). Drawn only when it lies
    /// wholly inside its clip.
    Bar {
        center: (f32, f32),
        w: f32,
        h: f32,
        angle: f32,
        color: [f32; 4],
        clip: Rect,
    },
    /// Half a disc of `radius` around `center`, on the side `angle` points to
    /// (0: right, clockwise on screen): the atlas disc's right half on a
    /// rotated quad. Pies and rings are two of them. Drawn only inside its clip.
    Half {
        center: (f32, f32),
        radius: f32,
        angle: f32,
        color: [f32; 4],
        clip: Rect,
    },
    /// One line of text: `x` the pen start, `baseline` its baseline.
    Text {
        x: f32,
        baseline: f32,
        size: f32,
        text: String,
        color: [f32; 4],
        font: FontHandle,
        /// The line box, for overlap.
        bounds: Rect,
        clip: Rect,
    },
}

/// A command and the painter level it draws on.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawItem {
    pub level: i32,
    pub cmd: DrawCmd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Batch {
    Atlas,
    Texture(TextureHandle),
    Font(FontHandle),
}

impl DrawCmd {
    fn batch(&self) -> Batch {
        match self {
            DrawCmd::Quad { .. }
            | DrawCmd::Bar { .. }
            | DrawCmd::Half { .. }
            | DrawCmd::Image { texture: None, .. } => Batch::Atlas,
            DrawCmd::Image {
                texture: Some(t), ..
            } => Batch::Texture(*t),
            DrawCmd::Text { font, .. } => Batch::Font(*font),
        }
    }

    fn bounds(&self) -> Rect {
        match self {
            DrawCmd::Quad { rect, clip, .. } | DrawCmd::Image { rect, clip, .. } => {
                rect.intersect(clip)
            }
            DrawCmd::Text { bounds, clip, .. } => bounds.intersect(clip),
            DrawCmd::Bar {
                center, w, h, clip, ..
            } => {
                let d = (w * w + h * h).sqrt();
                Rect::new(center.0 - d / 2.0, center.1 - d / 2.0, d, d).intersect(clip)
            }
            DrawCmd::Half {
                center,
                radius,
                clip,
                ..
            } => Rect::new(
                center.0 - radius,
                center.1 - radius,
                2.0 * radius,
                2.0 * radius,
            )
            .intersect(clip),
        }
    }
}

/// Assigns painter levels in submission order.
pub(crate) fn assign_levels(cmds: Vec<DrawCmd>) -> Vec<DrawItem> {
    let mut items: Vec<DrawItem> = Vec::with_capacity(cmds.len());
    let mut placed: Vec<(Rect, Batch, i32)> = Vec::with_capacity(cmds.len());
    for cmd in cmds {
        let bounds = cmd.bounds();
        if bounds.is_empty() {
            continue;
        }
        let batch = cmd.batch();
        let mut level = 0;
        for (r, b, l) in &placed {
            if r.overlaps(&bounds) {
                level = level.max(if *b == batch { *l } else { *l + 1 });
            }
        }
        placed.push((bounds, batch, level));
        items.push(DrawItem { level, cmd });
    }
    items
}

/// Where a frame's sprites and text go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitDesc {
    /// The scene's handle of [`atlas_texture`].
    pub atlas: TextureHandle,
    /// The overlay layer of painter level 0; level n draws on `layer + n`.
    /// Put the UI above other overlay sprites with a high base.
    pub layer: i32,
}

/// The atlas: a 64×64 white disc (alpha is coverage, 8×8 supersampled),
/// linear, with mips. Add it to the scene once and pass its handle in
/// [`EmitDesc::atlas`].
pub fn atlas_texture() -> TextureData {
    const N: u32 = 64;
    let r = N as f32 / 2.0;
    let mut rgba = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        for x in 0..N {
            let mut covered = 0u32;
            for sy in 0..8 {
                for sx in 0..8 {
                    let px = x as f32 + (sx as f32 + 0.5) / 8.0 - r;
                    let py = y as f32 + (sy as f32 + 0.5) / 8.0 - r;
                    if px * px + py * py <= r * r {
                        covered += 1;
                    }
                }
            }
            let a = (covered * 255 + 32) / 64;
            rgba.extend_from_slice(&[255, 255, 255, a as u8]);
        }
    }
    TextureData {
        width: N,
        height: N,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::Generate,
    }
}

fn sprite(
    rect: Rect,
    uv: [f32; 4],
    texture: TextureHandle,
    color: [f32; 4],
    layer: i32,
    clip: u32,
) -> Sprite {
    Sprite {
        texture: Some(texture),
        position: Vec3::new(rect.x, rect.y, 0.0),
        size: Vec2::new(rect.width, rect.height),
        anchor: Vec2::ZERO,
        rotation: 0.0,
        color,
        uv_rect: uv,
        layer,
        clip: Some(clip),
    }
}

/// The nine pieces of a rounded rectangle (fewer when `radius` is 0).
fn nine_slice(rect: Rect, radius: f32) -> Vec<(Rect, [f32; 4])> {
    let r = radius.min(rect.width / 2.0).min(rect.height / 2.0).max(0.0);
    let c = [0.5, 0.5, 0.5, 0.5];
    if r <= 0.0 {
        return vec![(rect, c)];
    }
    let (x0, x1, x2, x3) = (rect.x, rect.x + r, rect.right() - r, rect.right());
    let (y0, y1, y2, y3) = (rect.y, rect.y + r, rect.bottom() - r, rect.bottom());
    let cell = |a: f32, b: f32, c: f32, d: f32| Rect::new(a, b, c - a, d - b);
    vec![
        (cell(x0, y0, x1, y1), [0.0, 0.0, 0.5, 0.5]),
        (cell(x1, y0, x2, y1), [0.5, 0.0, 0.5, 0.5]),
        (cell(x2, y0, x3, y1), [0.5, 0.0, 1.0, 0.5]),
        (cell(x0, y1, x1, y2), [0.0, 0.5, 0.5, 0.5]),
        (cell(x1, y1, x2, y2), c),
        (cell(x2, y1, x3, y2), [0.5, 0.5, 1.0, 0.5]),
        (cell(x0, y2, x1, y3), [0.0, 0.5, 0.5, 1.0]),
        (cell(x1, y2, x2, y3), [0.5, 0.5, 0.5, 1.0]),
        (cell(x2, y2, x3, y3), [0.5, 0.5, 1.0, 1.0]),
    ]
}

/// The overlay clip index of a rectangle, adding it the first time.
fn clip_index(overlay: &mut Overlay, known: &mut Vec<(Rect, u32)>, clip: &Rect) -> u32 {
    if let Some((_, i)) = known.iter().rev().find(|(r, _)| r == clip) {
        return *i;
    }
    let i = overlay.add_clip([clip.x, clip.y, clip.width, clip.height]);
    known.push((*clip, i));
    i
}

pub(crate) fn emit(items: &[DrawItem], desc: &EmitDesc, overlay: &mut Overlay) {
    let mut known: Vec<(Rect, u32)> = Vec::new();
    for item in items {
        let layer = desc.layer + item.level;
        match &item.cmd {
            DrawCmd::Quad {
                rect,
                radius,
                color,
                clip,
            } => {
                let c = clip_index(overlay, &mut known, clip);
                for (piece, uv) in nine_slice(*rect, *radius) {
                    if !piece.is_empty() {
                        overlay
                            .sprites
                            .push(sprite(piece, uv, desc.atlas, *color, layer, c));
                    }
                }
            }
            DrawCmd::Image {
                rect,
                texture,
                uv,
                color,
                clip,
            } => {
                let (tex, uv) = match texture {
                    Some(t) => (*t, *uv),
                    None => (desc.atlas, [0.5, 0.5, 0.5, 0.5]),
                };
                let c = clip_index(overlay, &mut known, clip);
                overlay
                    .sprites
                    .push(sprite(*rect, uv, tex, *color, layer, c));
            }
            DrawCmd::Bar {
                center,
                w,
                h,
                angle,
                color,
                clip,
            } => {
                let c = clip_index(overlay, &mut known, clip);
                overlay.sprites.push(Sprite {
                    texture: Some(desc.atlas),
                    position: Vec3::new(center.0, center.1, 0.0),
                    size: Vec2::new(*w, *h),
                    anchor: Vec2::new(0.5, 0.5),
                    rotation: *angle,
                    color: *color,
                    uv_rect: [0.5, 0.5, 0.5, 0.5],
                    layer,
                    clip: Some(c),
                })
            }
            DrawCmd::Half {
                center,
                radius,
                angle,
                color,
                clip,
            } => {
                let c = clip_index(overlay, &mut known, clip);
                overlay.sprites.push(Sprite {
                    texture: Some(desc.atlas),
                    position: Vec3::new(center.0, center.1, 0.0),
                    size: Vec2::new(*radius, 2.0 * radius),
                    anchor: Vec2::new(0.0, 0.5),
                    rotation: *angle,
                    color: *color,
                    uv_rect: [0.5, 0.0, 1.0, 1.0],
                    layer,
                    clip: Some(c),
                })
            }
            DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color,
                font,
                clip,
                ..
            } => {
                let c = clip_index(overlay, &mut known, clip);
                overlay.texts.push(Text {
                    font: *font,
                    text: text.clone(),
                    position: Vec3::new(*x, *baseline, 0.0),
                    size: *size,
                    rotation: 0.0,
                    color: *color,
                    layer,
                    clip: Some(c),
                });
            }
        }
    }
}
