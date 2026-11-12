use super::*;

/// The 2D part of the art contract, every field at its meaning.
#[derive(Clone, Debug)]
pub(super) struct Art {
    pub(super) colors: Option<(String, Palette)>,
    pub(super) strict: bool,
    pub(super) max_colors: u32,
    pub(super) ramp_steps: u32,
    pub(super) accents: Vec<u32>,
    pub(super) dither: Dither,
    pub(super) unit: f64,
    pub(super) sprite_sizes: Vec<u32>,
    /// Pixel art: `ArtPixels` sets a grid (`unit`, `logicalHeight` or
    /// `spriteSizes`), so alpha is one bit and textures are lossless.
    pub(super) pixels: bool,
    pub(super) rotation_steps: u32,
    pub(super) line: f64,
    pub(super) line_rgb: [u8; 3],
    pub(super) place: String,
    pub(super) brush: Brush,
    pub(super) paint: Paint,
    /// Every input of the contract's (with the palette's and canvas's
    /// bytes): the stamps of conformed outputs carry it.
    pub(super) hash: u64,
}

fn block<'a>(look: &'a LookFile, resource: &str, field: &str) -> Option<&'a [Value]> {
    look.blocks
        .iter()
        .find(|b| b.resource == resource)
        .and_then(|b| b.get(field))
}

fn num(look: &LookFile, resource: &str, field: &str) -> f64 {
    match block(look, resource, field) {
        Some([v, ..]) => v.as_number().map_or(0.0, |n| n.as_f64()),
        _ => default(resource, field).first().copied().unwrap_or(0.0),
    }
}

/// A field's default in the IDL (its record's `default`), as numbers.
fn default(resource: &str, field: &str) -> Vec<f64> {
    let schema = format!("og.{resource}");
    og_records::RECORDS
        .iter()
        .find(|r| r.schema == schema)
        .and_then(|r| r.fields.iter().find(|f| f.name == field))
        .map(|f| {
            f.default
                .split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn word<'a>(look: &'a LookFile, resource: &str, field: &str, zero: &'a str) -> &'a str {
    match block(look, resource, field) {
        Some([v]) => v.as_word().unwrap_or(zero),
        _ => zero,
    }
}

fn list(look: &LookFile, resource: &str, field: &str) -> Vec<u32> {
    block(look, resource, field)
        .unwrap_or_default()
        .iter()
        .filter_map(|v| v.as_number().map(|n| n.as_f64() as u32))
        .collect()
}

/// The bytes of an asset the look names: a built-in, else under `assets/`.
fn named(assets: &Path, id: &str) -> Option<Vec<u8>> {
    match look::builtin(id) {
        Some((_, b)) => Some(b.to_vec()),
        None => std::fs::read(assets.join(id)).ok(),
    }
}

/// The look's `Stylise` paint fields read as texel space (docs/art.md
/// §2.3): the 2D pass paints sprites with them, the 3D pass a model's
/// colour textures (`mesh::Conform::paint`). Inactive when the look sets
/// none of them.
pub(in crate::cook) fn paint_of(full: &LookFile, assets: &Path) -> Result<Paint, String> {
    let paper = match block(full, "Stylise", "watercolorPaper") {
        Some(v) if v.len() == 3 => [0, 1, 2].map(|i| v[i].as_number().map_or(1.0, |n| n.as_f64())),
        _ => [1.0; 3],
    };
    let mut paint = Paint {
        radius: num(full, "Stylise", "paintRadius"),
        anisotropy: num(full, "Stylise", "paintAnisotropy"),
        edge: num(full, "Stylise", "paintEdgeDarkening"),
        bleed: num(full, "Stylise", "paintBleed"),
        canvas: None,
        canvas_strength: num(full, "Stylise", "paintCanvasStrength"),
        canvas_scale: num(full, "Stylise", "paintCanvasScale"),
        wash: Wash {
            edge: num(full, "Stylise", "watercolorEdgeDarkening"),
            granulation: num(full, "Stylise", "watercolorGranulation"),
            turbulence: num(full, "Stylise", "watercolorTurbulence"),
            dry_brush: num(full, "Stylise", "watercolorDryBrush"),
            wobble: num(full, "Stylise", "watercolorWobble"),
            dilution: num(full, "Stylise", "watercolorDilution"),
            paper,
        },
    };
    if let Some([Value::String(id)]) = block(full, "Stylise", "paintCanvas")
        && (paint.canvas_strength > 0.0 || paint.wash.reads_paper())
        && let Some(bytes) = named(assets, id)
    {
        let img = Rgba::decode(&bytes).map_err(|e| format!("Stylise.paintCanvas {id}: {e}"))?;
        let grain = img.px.iter().map(|p| colour::to_linear(p[0])).collect();
        paint.canvas = Some((grain, img.w, img.h));
    }
    Ok(paint)
}

impl Art {
    /// The contract of the flattened look `full` (every block), or `None`
    /// when it constrains no 2D texture.
    pub(super) fn of(
        full: &LookFile,
        contract: &LookFile,
        assets: &Path,
    ) -> Result<Option<Art>, String> {
        let mut h = StableHasher::new();
        h.write_str(VERSION);
        h.write_str(&contract.to_text());
        let colors = match block(full, "ArtPalette", "colors") {
            Some([Value::String(id)]) => {
                let bytes = named(assets, id)
                    .ok_or_else(|| format!("ArtPalette.colors: {id} is not in the bundle"))?;
                h.write(&bytes);
                let rgb = crate::look::parse_palette(id, &String::from_utf8_lossy(&bytes))?;
                (!rgb.is_empty()).then(|| (id.clone(), Palette::new(rgb)))
            }
            _ => None,
        };
        let line_rgb = match block(full, "ArtLine", "color") {
            Some(v) if v.len() == 3 => {
                [0, 1, 2].map(|i| v[i].as_number().map_or(0.0, |n| n.as_f64()))
            }
            _ => {
                let d = default("ArtLine", "color");
                [0, 1, 2].map(|i| d.get(i).copied().unwrap_or(0.0))
            }
        }
        .map(colour::to_srgb8);
        // Painted 2D: the frame's paint fields, read in texel space.
        let paint = paint_of(full, assets)?;
        if let Some([Value::String(id)]) = block(full, "Stylise", "paintCanvas")
            && paint.canvas.is_some()
            && let Some(bytes) = named(assets, id)
        {
            h.write(&bytes);
        }
        for b in full.blocks.iter().filter(|b| b.resource == "Stylise") {
            for (f, v, _) in b
                .fields
                .iter()
                .filter(|(f, _, _)| f.starts_with("paint") || f.starts_with("watercolor"))
            {
                h.write_str(&format!("{f} {v:?}"));
            }
        }
        let art = Art {
            colors,
            strict: word(full, "ArtPalette", "strict", "false") == "true",
            max_colors: num(full, "ArtPalette", "maxColors") as u32,
            ramp_steps: num(full, "ArtPalette", "rampSteps") as u32,
            accents: list(full, "ArtPalette", "accents"),
            dither: Dither::from_word(word(full, "ArtPalette", "dither", "none")),
            unit: num(full, "ArtPixels", "unit"),
            sprite_sizes: list(full, "ArtPixels", "spriteSizes"),
            pixels: num(full, "ArtPixels", "unit") > 0.0
                || num(full, "ArtPixels", "logicalHeight") > 0.0
                || !list(full, "ArtPixels", "spriteSizes").is_empty(),
            rotation_steps: if word(full, "ArtPixels", "rotation", "free") == "prerotated" {
                num(full, "ArtPixels", "rotationSteps") as u32
            } else {
                0
            },
            line: num(full, "ArtLine", "width"),
            line_rgb,
            place: word(full, "ArtLine", "place", "outside").to_string(),
            brush: Brush::from_word(word(full, "ArtLine", "brush", "diamond")),
            paint,
            hash: h.finish(),
        };
        // The contract is the switch: a look without art blocks (or with
        // them all at zero) changes no texture, painted or not.
        let constrains =
            art.pixels || art.palette_active() || art.line > 0.0 || art.rotation_steps >= 2;
        Ok((contract.constrains_art() && (constrains || art.paint.active())).then_some(art))
    }

    pub(super) fn palette_active(&self) -> bool {
        self.colors.is_some() || self.max_colors > 0 || self.ramp_steps > 0
    }
}

/// How the cell layout of a sheet's texture is laid out on a grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Grid {
    pub(super) cols: u32,
    pub(super) rows: u32,
    pub(super) w: u32,
    pub(super) h: u32,
    pub(super) mx: u32,
    pub(super) my: u32,
    pub(super) sx: u32,
    pub(super) sy: u32,
    /// Texels right of and under the last cell.
    pub(super) right: u32,
    pub(super) bottom: u32,
}

/// The grid a list of cells lies on, when they do: one size, at steps of
/// one spacing from one margin.
pub(super) fn grid_of(cells: &[Rect], size: (u32, u32)) -> Option<Grid> {
    let first = cells.first()?;
    let (w, h) = (first[2], first[3]);
    if cells.iter().any(|c| c[2] != w || c[3] != h) {
        return None;
    }
    let axis = |vals: BTreeSet<u32>, cell: u32| -> Option<(u32, u32, u32)> {
        let v: Vec<u32> = vals.into_iter().collect();
        let step = if v.len() > 1 { v[1] - v[0] } else { cell };
        if step < cell || v.windows(2).any(|p| p[1] - p[0] != step) {
            return None;
        }
        Some((v[0], step - cell, v.len() as u32))
    };
    let (mx, sx, cols) = axis(cells.iter().map(|c| c[0]).collect(), w)?;
    let (my, sy, rows) = axis(cells.iter().map(|c| c[1]).collect(), h)?;
    let used_w = mx + cols * w + (cols - 1) * sx;
    let used_h = my + rows * h + (rows - 1) * sy;
    Some(Grid {
        cols,
        rows,
        w,
        h,
        mx,
        my,
        sx,
        sy,
        right: size.0.saturating_sub(used_w),
        bottom: size.1.saturating_sub(used_h),
    })
}

/// The layout of the conformed texture: its size and where each source
/// cell went.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Layout {
    pub(super) size: (u32, u32),
    pub(super) cells: Vec<(Rect, Rect)>,
    pub(super) grid: Option<Grid>,
}

impl Layout {
    pub(super) fn identity(size: (u32, u32), cells: &[Rect]) -> Layout {
        Layout {
            size,
            cells: cells.iter().map(|&c| (c, c)).collect(),
            grid: None,
        }
    }

    pub(super) fn moved(&self, old: Rect) -> Option<Rect> {
        self.cells.iter().find(|(o, _)| *o == old).map(|(_, n)| *n)
    }
}

/// `n` texels at the scale `t / long`, rounded, at least `min`.
fn scaled(n: u32, t: u32, long: u32, min: u32) -> u32 {
    (((u64::from(n) * u64::from(t)) + u64::from(long) / 2) / u64::from(long)).max(u64::from(min))
        as u32
}

/// The cells `cells` of an image of `size` laid out again with each cell
/// at the scale `t / long`: on the same grid when they lie on one (its
/// margins and spacing scaled, a spacing at least 1), else packed in
/// rows in their order with one texel between.
pub(super) fn relayout(
    cells: &[Rect],
    size: (u32, u32),
    t: u32,
    long: u32,
    grid: Option<Grid>,
) -> Layout {
    let s = |n: u32| scaled(n, t, long, 1);
    let s0 = |n: u32| if n == 0 { 0 } else { scaled(n, t, long, 1) };
    if let Some(g) = grid {
        let (w, h) = (s(g.w), s(g.h));
        let (mx, my, sx, sy) = (
            scaled(g.mx, t, long, 0),
            scaled(g.my, t, long, 0),
            s0(g.sx),
            s0(g.sy),
        );
        let ng = Grid {
            w,
            h,
            mx,
            my,
            sx,
            sy,
            right: scaled(g.right, t, long, 0),
            bottom: scaled(g.bottom, t, long, 0),
            ..g
        };
        let size = (
            mx + g.cols * w + (g.cols - 1) * sx + ng.right,
            my + g.rows * h + (g.rows - 1) * sy + ng.bottom,
        );
        let cells = cells
            .iter()
            .map(|&c| {
                let col = (c[0] - g.mx) / (g.w + g.sx);
                let row = (c[1] - g.my) / (g.h + g.sy);
                (c, [mx + col * (w + sx), my + row * (h + sy), w, h])
            })
            .collect();
        return Layout {
            size,
            cells,
            grid: Some(ng),
        };
    }
    let _ = size;
    let new: Vec<(u32, u32)> = cells.iter().map(|c| (s(c[2]), s(c[3]))).collect();
    let per_row = (cells.len() as f64).sqrt().ceil() as u32;
    let widest = new.iter().map(|n| n.0).max().unwrap_or(1);
    let limit = per_row * widest + per_row.saturating_sub(1);
    let (mut x, mut y, mut row_h, mut width) = (0u32, 0u32, 0u32, 0u32);
    let mut out = Vec::new();
    for (c, (w, h)) in cells.iter().zip(new) {
        if x > 0 && x + w > limit {
            x = 0;
            y += row_h + 1;
            row_h = 0;
        }
        out.push((*c, [x, y, w, h]));
        width = width.max(x + w);
        row_h = row_h.max(h);
        x += w + 1;
    }
    Layout {
        size: (width.max(1), (y + row_h).max(1)),
        cells: out,
        grid: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_is_found_and_laid_out_again_at_scale() {
        // Six 16 x 16 frames, 2 apart, after a margin of 1: 3 x 2.
        let cells: Vec<Rect> = (0..6)
            .map(|i| [1 + (i % 3) * 18, 1 + (i / 3) * 18, 16, 16])
            .collect();
        let g = grid_of(&cells, (53, 35)).unwrap();
        assert_eq!(
            (g.cols, g.rows, g.mx, g.sx, g.right, g.bottom),
            (3, 2, 1, 2, 0, 0)
        );
        // At half scale: 8 x 8 cells, spacing 1, margin 1 (0.5 rounds up).
        let l = relayout(&cells, (53, 35), 8, 16, Some(g));
        assert_eq!(l.moved([19, 19, 16, 16]), Some([10, 10, 8, 8]));
        assert_eq!(l.size, (1 + 3 * 8 + 2, 1 + 2 * 8 + 1));
        // Frames off a grid pack in rows with one texel between.
        let odd = vec![[0, 0, 10, 4], [10, 0, 6, 6], [0, 6, 10, 10]];
        assert!(grid_of(&odd, (16, 16)).is_none());
        let l = relayout(&odd, (16, 16), 5, 10, None);
        assert_eq!(
            l.cells.iter().map(|c| c.1).collect::<Vec<_>>(),
            vec![[0, 0, 5, 2], [6, 0, 3, 3], [0, 4, 5, 5]]
        );
    }
}
