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
    /// Glass: the frame under `rect` blurred by `blur` CSS px, tinted
    /// (`tint`: linear RGB, alpha how much covers the blur), masked to the
    /// rounded rectangle, at `alpha` (og-render's [`og_render::Glass`]).
    Glass {
        rect: Rect,
        radius: f32,
        blur: f32,
        tint: [f32; 4],
        alpha: f32,
        clip: Rect,
    },
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
        /// Added to each glyph's advance, em (the type step's tracking).
        tracking: f32,
        /// Thickening at the glyphs' edge, CSS px (a faux bold).
        weight: f32,
        /// A glow from the font's distance field: linear RGBA and radius, CSS px.
        glow: Option<([f32; 4], f32)>,
        /// A visual line's placements (bidi runs, shaped glyphs), one per
        /// character of `text`; empty: the font's advances and kerning.
        placements: Vec<og_render::GlyphPlacement>,
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
    Glass,
    Atlas,
    Texture(TextureHandle),
    Font(FontHandle),
}

impl DrawCmd {
    fn batch(&self) -> Batch {
        match self {
            DrawCmd::Glass { .. } => Batch::Glass,
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
            DrawCmd::Quad { rect, clip, .. }
            | DrawCmd::Image { rect, clip, .. }
            | DrawCmd::Glass { rect, clip, .. } => rect.intersect(clip),
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

/// Assigns painter levels in submission order: a command goes above every
/// earlier one it overlaps (on the same level when they share a batch).
/// Each placed rectangle is filed in the cells of a coarse grid it touches,
/// and a command tests only the rectangles of its own cells: a screen
/// transition's dissolve is 1,620 quads over the frame, and testing each
/// against every earlier command was 7 to 17 ms of a dev tick (Lantern
/// Vale's playtests, tests/evidence/P4/perf-2).
pub(crate) fn assign_levels(cmds: Vec<DrawCmd>) -> Vec<DrawItem> {
    let bounds: Vec<Rect> = cmds.iter().map(DrawCmd::bounds).collect();
    let grid = Grid::over(&bounds);
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); grid.cols * grid.rows];
    let mut items: Vec<DrawItem> = Vec::with_capacity(cmds.len());
    let mut placed: Vec<(Rect, Batch, i32)> = Vec::with_capacity(cmds.len());
    for (cmd, bounds) in cmds.into_iter().zip(bounds) {
        if bounds.is_empty() {
            continue;
        }
        let batch = cmd.batch();
        let (cols, rows) = grid.span(&bounds);
        let mut level = 0;
        for row in rows.clone() {
            for col in cols.clone() {
                for &p in &cells[row * grid.cols + col] {
                    let (r, b, l) = &placed[p as usize];
                    if r.overlaps(&bounds) {
                        level = level.max(if *b == batch { *l } else { *l + 1 });
                    }
                }
            }
        }
        let id = placed.len() as u32;
        for row in rows {
            for col in cols.clone() {
                cells[row * grid.cols + col].push(id);
            }
        }
        placed.push((bounds, batch, level));
        items.push(DrawItem { level, cmd });
    }
    items
}

/// [`assign_levels`]' grid: about one cell for every four commands, at
/// most 64 by 64 cells of at least 32 CSS px, over the commands' bounds. A rectangle is filed in every cell its closed
/// extent touches (clamped to the grid), so two rectangles that overlap
/// share a cell.
struct Grid {
    x: f32,
    y: f32,
    cell_w: f32,
    cell_h: f32,
    cols: usize,
    rows: usize,
}

impl Grid {
    fn over(bounds: &[Rect]) -> Grid {
        let (mut x0, mut y0) = (f32::MAX, f32::MAX);
        let (mut x1, mut y1) = (f32::MIN, f32::MIN);
        for r in bounds.iter().filter(|r| !r.is_empty()) {
            if r.x.is_finite() && r.right().is_finite() {
                x0 = x0.min(r.x);
                x1 = x1.max(r.right());
            }
            if r.y.is_finite() && r.bottom().is_finite() {
                y0 = y0.min(r.y);
                y1 = y1.max(r.bottom());
            }
        }
        // About one cell for every four commands, each at least 32 CSS px
        // across: a grid of the frame's 920 cells cost more to fill (a
        // backdrop is filed in every cell) than a HUD of a few dozen
        // commands cost to test pair by pair (tests/evidence/r4/perf-ui).
        let cells = (bounds.len() / 4).clamp(1, 64 * 64) as f32;
        let area = ((x1 - x0).max(1.0) * (y1 - y0).max(1.0)).max(1.0);
        let side = (area / cells).sqrt().max(32.0);
        let axis = |lo: f32, hi: f32| {
            if hi <= lo {
                return (0.0, 1.0, 1);
            }
            let n = ((hi - lo) / side).ceil().clamp(1.0, 64.0);
            (lo, (hi - lo) / n, n as usize)
        };
        let (x, cell_w, cols) = axis(x0, x1);
        let (y, cell_h, rows) = axis(y0, y1);
        Grid {
            x,
            y,
            cell_w,
            cell_h,
            cols,
            rows,
        }
    }

    /// The columns and rows `r` touches.
    fn span(
        &self,
        r: &Rect,
    ) -> (
        std::ops::RangeInclusive<usize>,
        std::ops::RangeInclusive<usize>,
    ) {
        // `as` saturates (NaN is 0), and the clamp keeps every index in the grid.
        let at = |v: f32, o: f32, size: f32, n: usize| {
            (((v - o) / size).floor() as i64).clamp(0, n as i64 - 1) as usize
        };
        (
            at(r.x, self.x, self.cell_w, self.cols)..=at(r.right(), self.x, self.cell_w, self.cols),
            at(r.y, self.y, self.cell_h, self.rows)
                ..=at(r.bottom(), self.y, self.cell_h, self.rows),
        )
    }
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
            DrawCmd::Glass {
                rect,
                radius,
                blur,
                tint,
                alpha,
                clip,
            } => {
                let c = clip_index(overlay, &mut known, clip);
                overlay.glass.push(og_render::Glass {
                    position: og_core::glam::Vec2::new(rect.x, rect.y),
                    size: og_core::glam::Vec2::new(rect.width, rect.height),
                    radius: *radius,
                    blur: *blur,
                    tint: *tint,
                    alpha: *alpha,
                    layer,
                    clip: Some(c),
                });
            }
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
                tracking,
                weight,
                glow,
                placements,
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
                    tracking: *tracking,
                    weight: *weight,
                    glow: glow.map(|(color, radius)| og_render::TextGlow { color, radius }),
                    placements: placements.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod levels {
    use super::*;

    /// The levels as every command tested against every earlier one (what
    /// the grid must answer).
    fn quadratic(cmds: &[DrawCmd]) -> Vec<i32> {
        let mut placed: Vec<(Rect, Batch, i32)> = Vec::new();
        let mut out = Vec::new();
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
            out.push(level);
        }
        out
    }

    fn quad(rect: Rect) -> DrawCmd {
        DrawCmd::Quad {
            rect,
            radius: 0.0,
            color: [1.0; 4],
            clip: Rect::new(-1e9, -1e9, 2e9, 2e9),
        }
    }

    fn image(rect: Rect, slot: u32) -> DrawCmd {
        DrawCmd::Image {
            rect,
            texture: og_render::TextureHandle::from_slot(slot),
            uv: [0.0, 0.0, 1.0, 1.0],
            color: [1.0; 4],
            clip: Rect::new(-1e9, -1e9, 2e9, 2e9),
        }
    }

    #[test]
    fn the_grid_assigns_the_levels_of_every_pair_tested() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f32 / (1u64 << 53) as f32
        };
        let mut cases: Vec<Vec<DrawCmd>> = Vec::new();
        // Random rectangles of two batches, some touching edge to edge, some
        // far off the frame, some empty or not finite.
        for n in [1, 7, 60, 400] {
            let cmds = (0..n)
                .map(|i| {
                    let (x, y) = (rand() * 1400.0 - 60.0, rand() * 800.0 - 40.0);
                    let (w, h) = (rand() * 300.0, rand() * 200.0);
                    let r = match i % 23 {
                        0 => Rect::new(x, y, 0.0, h),
                        1 => Rect::new(f32::NAN, y, w, h),
                        2 => Rect::new(x, y, f32::INFINITY, h),
                        3 => Rect::new(-1e7, y, 2e7, h),
                        4 => Rect::new((x / 24.0).floor() * 24.0, 0.0, 24.0, 24.0),
                        _ => Rect::new(x, y, w, h),
                    };
                    if i % 3 == 0 { image(r, i % 2) } else { quad(r) }
                })
                .collect();
            cases.push(cmds);
        }
        // A screen under a dissolve's grid of cells, as a transition draws it.
        let mut screen = vec![quad(Rect::new(0.0, 0.0, 1280.0, 720.0))];
        screen.extend((0..20).map(|i| image(Rect::new(40.0 * i as f32, 30.0, 120.0, 40.0), 0)));
        for y in 0..30 {
            for x in 0..54 {
                screen.push(quad(Rect::new(
                    x as f32 * 24.0,
                    y as f32 * 24.0,
                    24.0,
                    24.0,
                )));
            }
        }
        cases.push(screen);
        for cmds in cases {
            let want = quadratic(&cmds);
            let got: Vec<i32> = assign_levels(cmds).iter().map(|i| i.level).collect();
            assert_eq!(got, want);
        }
    }
}
