//! From a laid-out frame to og-render's sprite and text batches.
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
//! **Clipping is on the CPU** (og-render exposes neither scissor nor
//! stencil): quads and images are cut to the clip rectangle with their UVs
//! adjusted; text is clipped by whole lines vertically and whole glyphs
//! horizontally.

use crate::layout::Rect;
use crate::Viewport;
use og_core::glam::{Vec2, Vec3};
use og_render::{
    Camera, ColorSpace, Exposure, FontHandle, Mips, Sprite, Text, TextureData, TextureHandle,
};

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
    /// One line of text: `x` the pen start, `baseline` its baseline.
    Text {
        x: f32,
        baseline: f32,
        size: f32,
        text: String,
        color: [f32; 4],
        font: FontHandle,
        /// The line box, for clipping and overlap.
        bounds: Rect,
        clip: Rect,
        /// Per-character advances in CSS pixels, for horizontal clipping.
        advances: Vec<f32>,
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
            DrawCmd::Quad { .. } | DrawCmd::Image { texture: None, .. } => Batch::Atlas,
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
    /// The layer of painter level 0; level n draws on `layer + n`. Put the UI
    /// above a game's own sprites with a high base.
    pub layer: i32,
    /// The world z of the UI plane.
    pub z: f32,
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

/// The orthographic camera that maps the viewport's CSS pixels onto the
/// render target: CSS (x, y) is world (x, -y) on the z = 0 plane.
pub fn camera(viewport: &Viewport) -> Camera {
    Camera {
        exposure: Exposure::Manual { ev: 0.0 },
        ..Camera::orthographic_2d(
            Vec2::new(viewport.width / 2.0, -viewport.height / 2.0),
            viewport.height,
        )
    }
}

/// A piece of a quad cut to the clip, its UVs following.
fn clip_piece(rect: Rect, uv: [f32; 4], clip: &Rect) -> Option<(Rect, [f32; 4])> {
    let c = rect.intersect(clip);
    if c.is_empty() || rect.is_empty() {
        return None;
    }
    let fx = |x: f32| (x - rect.x) / rect.width;
    let fy = |y: f32| (y - rect.y) / rect.height;
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    Some((
        c,
        [
            lerp(uv[0], uv[2], fx(c.x)),
            lerp(uv[1], uv[3], fy(c.y)),
            lerp(uv[0], uv[2], fx(c.right())),
            lerp(uv[1], uv[3], fy(c.bottom())),
        ],
    ))
}

fn sprite(
    rect: Rect,
    uv: [f32; 4],
    texture: TextureHandle,
    color: [f32; 4],
    layer: i32,
    z: f32,
) -> Sprite {
    Sprite {
        texture: Some(texture),
        position: Vec3::new(rect.x, -rect.bottom(), z),
        size: Vec2::new(rect.width, rect.height),
        anchor: Vec2::ZERO,
        rotation: 0.0,
        color,
        uv_rect: uv,
        layer,
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

pub(crate) fn emit(
    items: &[DrawItem],
    desc: &EmitDesc,
    sprites: &mut Vec<Sprite>,
    texts: &mut Vec<Text>,
) {
    let z = desc.z;
    for item in items {
        let layer = desc.layer + item.level;
        match &item.cmd {
            DrawCmd::Quad {
                rect,
                radius,
                color,
                clip,
            } => {
                for (piece, uv) in nine_slice(*rect, *radius) {
                    if let Some((r, uv)) = clip_piece(piece, uv, clip) {
                        sprites.push(sprite(r, uv, desc.atlas, *color, layer, z));
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
                if let Some((r, uv)) = clip_piece(*rect, uv, clip) {
                    sprites.push(sprite(r, uv, tex, *color, layer, z));
                }
            }
            DrawCmd::Text {
                x,
                baseline,
                size,
                text,
                color,
                font,
                bounds,
                clip,
                advances,
            } => {
                // Whole lines vertically (a half-visible line is hidden).
                let eps = 0.01;
                if bounds.y < clip.y - eps || bounds.bottom() > clip.bottom() + eps {
                    continue;
                }
                // Whole glyphs horizontally.
                let mut pen = *x;
                let mut start_x = None;
                let mut visible = String::new();
                for (ch, adv) in text.chars().zip(advances) {
                    let inside = pen >= clip.x - eps && pen + adv <= clip.right() + eps;
                    if inside {
                        if start_x.is_none() {
                            start_x = Some(pen);
                        }
                        visible.push(ch);
                    } else if start_x.is_some() {
                        break;
                    }
                    pen += adv;
                }
                let Some(sx) = start_x else { continue };
                texts.push(Text {
                    font: *font,
                    text: visible,
                    position: Vec3::new(sx, -baseline, z),
                    size: *size,
                    rotation: 0.0,
                    color: *color,
                    layer,
                });
            }
        }
    }
}
