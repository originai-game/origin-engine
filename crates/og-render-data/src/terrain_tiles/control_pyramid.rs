//! A terrain's control map resampled for its tiles.

use super::*;

/// A terrain's control map resampled for its tiles: level 0 at
/// `control` texels per tile (bilinear between the map's texel centres),
/// each level after it halved (2 × 2 means, rounded); one such pyramid per
/// page of four channels.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlPyramid {
    /// Per page, (width, height, RGBA8) per level.
    pub pages: Vec<Vec<(u32, u32, Vec<u8>)>>,
}

impl ControlPyramid {
    /// `None` for a layout without a control map.
    pub fn new(layout: &TerrainLayout, map: &TerrainControl) -> Option<ControlPyramid> {
        if layout.control == 0 {
            return None;
        }
        let c = layout.control;
        let pages = (0..map.pages())
            .map(|p| {
                let map = map.page(p);
                let (tx, tz) = layout.tiles(0);
                let (w, h) = (c * tx, c * tz);
                let mut base = vec![0u8; (w * h * 4) as usize];
                for j in 0..h {
                    for i in 0..w {
                        let at = ((j * w + i) * 4) as usize;
                        base[at..at + 4].copy_from_slice(&Self::base(layout, &map, i, j));
                    }
                }
                let mut levels = vec![(w, h, base)];
                for k in 1..layout.levels {
                    let (pw, ph, prev) = levels.last().expect("level 0");
                    let (tx, tz) = layout.tiles(k);
                    let (w, h) = (c * tx, c * tz);
                    let mut next = vec![0u8; (w * h * 4) as usize];
                    for j in 0..h {
                        for i in 0..w {
                            let at = ((j * w + i) * 4) as usize;
                            next[at..at + 4].copy_from_slice(&Self::mean(*pw, *ph, prev, i, j));
                        }
                    }
                    levels.push((w, h, next));
                }
                levels
            })
            .collect();
        Some(ControlPyramid { pages })
    }

    /// Level 0's texel (`i`, `j`) of one page's map: bilinear between the
    /// map's texel centres.
    fn base(layout: &TerrainLayout, map: &TerrainControl, i: u32, j: u32) -> [u8; 4] {
        let c = layout.control;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let texel = |x: i64, y: i64, k: usize| {
            let x = x.clamp(0, map.width as i64 - 1) as usize;
            let y = y.clamp(0, map.height as i64 - 1) as usize;
            map.rgba[(y * map.width as usize + x) * 4 + k] as f32
        };
        // The texel's centre as a fraction of the terrain, then in the map's texels.
        let uv = Vec2::new(
            (i as f32 + 0.5) * layout.tile as f32 / c as f32,
            (j as f32 + 0.5) * layout.tile as f32 / c as f32,
        ) / quads;
        let t =
            uv.clamp(Vec2::ZERO, Vec2::ONE) * Vec2::new(map.width as f32, map.height as f32) - 0.5;
        let f0 = t.floor();
        let f = t - f0;
        let (x0, y0) = (f0.x as i64, f0.y as i64);
        std::array::from_fn(|k| {
            let a = texel(x0, y0, k) + (texel(x0 + 1, y0, k) - texel(x0, y0, k)) * f.x;
            let b = texel(x0, y0 + 1, k) + (texel(x0 + 1, y0 + 1, k) - texel(x0, y0 + 1, k)) * f.x;
            (a + (b - a) * f.y).round().clamp(0.0, 255.0) as u8
        })
    }

    /// A coarser level's texel (`i`, `j`): the mean of the 2 × 2 finer ones
    /// under it (`prev`, `pw` × `ph`), clamped, rounded.
    fn mean(pw: u32, ph: u32, prev: &[u8], i: u32, j: u32) -> [u8; 4] {
        std::array::from_fn(|ch| {
            let mut sum = 0u32;
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let x = (2 * i + ox).min(pw - 1);
                let y = (2 * j + oy).min(ph - 1);
                sum += prev[((y * pw + x) * 4 + ch as u32) as usize] as u32;
            }
            ((sum + 2) / 4) as u8
        })
    }

    /// Resamples what an edit of `map` (the edited map) inside `lo`..`hi`
    /// (the terrain's space) reaches on every level: the pyramid is then
    /// [`ControlPyramid::new`]'s of the edited map.
    pub fn update(&mut self, layout: &TerrainLayout, map: &TerrainControl, lo: Vec2, hi: Vec2) {
        // The edit in cells, widened by the two map texels a bilinear read
        // spans, then in level 0's texels (each `tile / control` cells).
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let texel = quads / Vec2::new(map.width as f32, map.height as f32);
        let to = |p: Vec2| (p + layout.size * 0.5) / layout.cell();
        let per = layout.tile as f32 / layout.control as f32;
        let a = ((to(lo) - 2.0 * texel) / per - 0.5).floor() - 1.0;
        let b = ((to(hi) + 2.0 * texel) / per - 0.5).ceil() + 1.0;
        for (p, levels) in self.pages.iter_mut().enumerate() {
            let map = map.page(p as u32);
            let (w, h, base) = &mut levels[0];
            let clamp = |v: f32, n: u32| (v.max(0.0) as u32).min(n - 1);
            let (mut x0, mut x1) = (clamp(a.x, *w), clamp(b.x, *w));
            let (mut y0, mut y1) = (clamp(a.y, *h), clamp(b.y, *h));
            for j in y0..=y1 {
                for i in x0..=x1 {
                    let at = ((j * *w + i) * 4) as usize;
                    base[at..at + 4].copy_from_slice(&Self::base(layout, &map, i, j));
                }
            }
            for k in 1..levels.len() {
                let (done, rest) = levels.split_at_mut(k);
                let (pw, ph, prev) = &done[k - 1];
                let (w, h, next) = &mut rest[0];
                // The texels past the finer level's last read its last.
                let up = |a: u32, n: u32, pn: u32| {
                    if a + 1 >= pn {
                        n - 1
                    } else {
                        (a / 2).min(n - 1)
                    }
                };
                (x0, x1, y0, y1) = (
                    (x0 / 2).min(*w - 1),
                    up(x1, *w, *pw),
                    (y0 / 2).min(*h - 1),
                    up(y1, *h, *ph),
                );
                for j in y0..=y1 {
                    for i in x0..=x1 {
                        let at = ((j * *w + i) * 4) as usize;
                        next[at..at + 4].copy_from_slice(&Self::mean(*pw, *ph, prev, i, j));
                    }
                }
            }
        }
    }

    /// The `(control + 2)²` texels of a tile (its texels and a border of
    /// one, clamped at the map's edge), page after page.
    pub fn tile(&self, layout: &TerrainLayout, index: u32) -> Vec<u8> {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let c = layout.control as i64;
        let side = layout.control_side() as usize;
        let mut out = Vec::with_capacity(side * side * 4 * self.pages.len());
        for page in &self.pages {
            let (w, h, px) = &page[k as usize];
            for j in -1..=c {
                for i in -1..=c {
                    let x = (tx as i64 * c + i).clamp(0, *w as i64 - 1) as usize;
                    let y = (tz as i64 * c + j).clamp(0, *h as i64 - 1) as usize;
                    let at = (y * *w as usize + x) * 4;
                    out.extend_from_slice(&px[at..at + 4]);
                }
            }
        }
        out
    }
}
