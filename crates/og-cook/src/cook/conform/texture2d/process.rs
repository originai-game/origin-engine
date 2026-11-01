use super::*;

/// Conforms one image in its role: the steps of the module's doc.
pub(super) fn run(art: &Art, src: &Rgba, role: &Role, size: Option<u32>) -> Done {
    let mut what = Vec::new();
    let whole = [0, 0, src.w, src.h];
    let (cells, grid, tiles, fixed) = match role {
        Role::Sprite => (vec![whole], None, false, false),
        Role::Sheet { cells, grid, tiles } => (cells.clone(), *grid, *tiles, false),
        Role::Fixed(why) => {
            what.push(format!("named by {why}: its layout stays"));
            (vec![whole], None, false, true)
        }
    };
    let mut img = src.clone();

    // Painted 2D, in each cell's texels.
    if art.paint.active() && !fixed {
        for &c in &cells {
            let mut cell = img.crop(c);
            art.paint.apply(&mut cell.px, cell.w, cell.h);
            img.blit(c[0], c[1], &cell);
        }
        what.push(format!("painted in texel space ({})", art.paint.describe()));
    }

    // The pixel grid: the cell size and the downscale, 1-bit alpha.
    let mut layout = Layout::identity((img.w, img.h), &cells);
    if art.pixels && !fixed {
        let candidates: Vec<u32> = match size {
            Some(n) => vec![n],
            None if tiles && art.unit > 0.0 => vec![art.unit.round() as u32],
            None => art.sprite_sizes.clone(),
        };
        let largest = *cells
            .iter()
            .max_by_key(|c| {
                (
                    u64::from(c[2]) * u64::from(c[3]),
                    std::cmp::Reverse(c[1]),
                    std::cmp::Reverse(c[0]),
                )
            })
            .expect("a cell");
        if let Some((tw, th)) = resample::choose((largest[2], largest[3]), &candidates, |w, h| {
            resample::error(&img, largest, w, h)
        }) {
            let long = largest[2].max(largest[3]);
            let t = tw.max(th);
            layout = relayout(&cells, (img.w, img.h), t, long, grid);
            let mut out = Rgba::new(layout.size.0, layout.size.1);
            for (old, new) in &layout.cells {
                let px = resample::downscale(&img, *old, new[2], new[3], true);
                out.blit(
                    new[0],
                    new[1],
                    &Rgba {
                        w: new[2],
                        h: new[3],
                        px,
                    },
                );
            }
            what.push(format!(
                "{}x{} cells to {}x{} by the dominant-colour box ({}x{} to {}x{} texels)",
                largest[2], largest[3], tw, th, img.w, img.h, out.w, out.h
            ));
            img = out;
        }
        let mut cut = 0usize;
        for p in &mut img.px {
            let a = if p[3] >= resample::OPAQUE { 255 } else { 0 };
            if a != p[3] {
                cut += 1;
            }
            *p = if a == 0 {
                [0, 0, 0, 0]
            } else {
                [p[0], p[1], p[2], 255]
            };
        }
        if cut > 0 {
            what.push(format!("alpha to 1 bit ({cut} texels)"));
        }
        if !art.sprite_sizes.is_empty() && !tiles {
            let off: Vec<String> = layout
                .cells
                .iter()
                .map(|(_, n)| (n[2], n[3]))
                .filter(|(w, h)| !art.sprite_sizes.contains(w) || !art.sprite_sizes.contains(h))
                .map(|(w, h)| format!("{w}x{h}"))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            if !off.is_empty() {
                what.push(format!(
                    "warning: cells {} are not of spriteSizes {}",
                    off.join(", "),
                    art.sprite_sizes
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
        }
    }
    let new_cells: Vec<Rect> = layout.cells.iter().map(|(_, n)| *n).collect();
    let origin_grid = cell_origins(&new_cells, img.w, img.h);
    let origin = |x: u32, y: u32| origin_grid[(y * img.w + x) as usize];

    // The palette.
    let mut index: Option<Vec<Option<usize>>> = None;
    let mut pal: Option<(Palette, Option<String>)> = None;
    if art.palette_active() {
        let before = distinct(&img);
        if let (Some((id, p)), true) = (&art.colors, art.strict) {
            let mut allowed: Vec<usize> = (0..p.rgb.len()).collect();
            if tiles && !art.accents.is_empty() {
                allowed.retain(|i| !art.accents.contains(&(*i as u32)));
                if allowed.is_empty() {
                    allowed = (0..p.rgb.len()).collect();
                }
            }
            let mut ix = palette::map(&img.px, img.w, &origin, p, &allowed, art.dither);
            if art.max_colors > 0 || art.ramp_steps > 0 {
                let kept = palette::limit(
                    p,
                    &palette::usage(&ix, p.rgb.len()),
                    art.ramp_steps,
                    art.max_colors,
                );
                if kept.len()
                    < palette::usage(&ix, p.rgb.len())
                        .iter()
                        .filter(|n| **n > 0)
                        .count()
                    && !kept.is_empty()
                {
                    ix = palette::map(&img.px, img.w, &origin, p, &kept, art.dither);
                }
            }
            what.push(format!(
                "{before} colours to {} of {id} ({} colours; dither {}{})",
                palette::usage(&ix, p.rgb.len())
                    .iter()
                    .filter(|n| **n > 0)
                    .count(),
                p.rgb.len(),
                art.dither.name(),
                if tiles && !art.accents.is_empty() {
                    "; tiles keep off the accents"
                } else {
                    ""
                }
            ));
            index = Some(ix);
            pal = Some((p.clone(), Some(id.clone())));
        } else {
            if let Some((id, p)) = &art.colors {
                for px in img.px.iter_mut().filter(|p| p[3] > 0) {
                    let c = palette::hold_hue(p, [px[0], px[1], px[2]]);
                    px[..3].copy_from_slice(&c);
                }
                what.push(format!("hues held to {id}"));
            }
            if art.max_colors > 0 || art.ramp_steps > 0 {
                let reserve = u32::from(art.line > 0.0 && !fixed);
                let max = if art.max_colors > 0 {
                    art.max_colors.saturating_sub(reserve).max(1)
                } else {
                    0
                };
                let p = palette::adaptive(&img.px, max, art.ramp_steps);
                let all: Vec<usize> = (0..p.rgb.len()).collect();
                let ix = if p.rgb.is_empty() {
                    vec![None; img.px.len()]
                } else {
                    palette::map(&img.px, img.w, &origin, &p, &all, art.dither)
                };
                what.push(format!(
                    "{before} colours to {} of its own{}{}",
                    p.rgb.len(),
                    if art.max_colors > 0 {
                        format!(" (maxColors {})", art.max_colors)
                    } else {
                        String::new()
                    },
                    if art.ramp_steps > 0 {
                        format!(" (rampSteps {})", art.ramp_steps)
                    } else {
                        String::new()
                    },
                ));
                index = Some(ix);
                pal = Some((p, None));
            }
        }
        if let (Some(ix), Some((p, _))) = (&index, &pal) {
            for (px, i) in img.px.iter_mut().zip(ix) {
                if let Some(i) = i {
                    px[..3].copy_from_slice(&p.rgb[*i]);
                }
            }
        }
    }

    // The line, into each cell; the cells before it are the rotations'.
    let before_line = img.clone();
    let line_colour = |pal: &mut Option<(Palette, Option<String>)>| -> ([u8; 3], Option<usize>) {
        match pal {
            Some((p, Some(_))) => {
                let lab = colour::lab(art.line_rgb);
                let i = (0..p.rgb.len())
                    .min_by_key(|&i| colour::dist2(lab, p.lab[i]))
                    .expect("a colour");
                (p.rgb[i], Some(i))
            }
            Some((p, None)) => match p.rgb.iter().position(|c| *c == art.line_rgb) {
                Some(i) => (art.line_rgb, Some(i)),
                None => {
                    let mut rgb = p.rgb.clone();
                    rgb.push(art.line_rgb);
                    *p = Palette::new(rgb);
                    (art.line_rgb, Some(p.rgb.len() - 1))
                }
            },
            None => (art.line_rgb, None),
        }
    };
    let (inner, outer) = outline::split(art.line, &art.place);
    let mut line_index = None;
    if art.line > 0.0 && !fixed {
        let (rgb, li) = line_colour(&mut pal);
        line_index = li;
        let mut drawn = 0usize;
        for &c in &new_cells {
            let cell = img.crop(c);
            for (k, m) in draw_line(&cell, inner, outer, art.brush, tiles)
                .into_iter()
                .enumerate()
            {
                if m {
                    let (x, y) = (c[0] + k as u32 % c[2], c[1] + k as u32 / c[2]);
                    img.px[(y * img.w + x) as usize] = [rgb[0], rgb[1], rgb[2], 255];
                    if let Some(ix) = &mut index {
                        ix[(y * img.w + x) as usize] = li;
                    }
                    drawn += 1;
                }
            }
        }
        what.push(format!(
            "line {} px {} ({}) in #{:02x}{:02x}{:02x}: {drawn} texels",
            art.line,
            art.place,
            art.brush.name(),
            rgb[0],
            rgb[1],
            rgb[2]
        ));
    }

    // Rotations: each cell before its line, turned by cleanEdge, then lined.
    let mut rotations = None;
    if art.rotation_steps >= 2 && !fixed && !tiles {
        let n = art.rotation_steps;
        let diag =
            |c: &Rect| ((f64::from(c[2]).powi(2) + f64::from(c[3]).powi(2)).sqrt()).ceil() as u32;
        let par = |v: u32, like: u32| {
            if (v - like).is_multiple_of(2) {
                v
            } else {
                v + 1
            }
        };
        let big = *new_cells
            .iter()
            .max_by_key(|c| (c[2].max(c[3]), c[2], c[3]))
            .expect("a cell");
        let (sx, sy) = (
            par(diag(&big).max(big[2]), big[2]),
            par(diag(&big).max(big[3]), big[3]),
        );
        let mut sheet = Rgba::new(sx * n, sy * new_cells.len() as u32);
        for (row, c) in new_cells.iter().enumerate() {
            let cell = before_line.crop(*c);
            let tex = |x: u32, y: u32| cell.at(x, y).map(|v| f64::from(v) / 255.0);
            for k in 0..n {
                let deg = 360.0 * f64::from(k) / f64::from(n);
                let px: Vec<[u8; 4]> = cleanedge::rotate(cell.w, cell.h, &tex, deg, sx, sy)
                    .into_iter()
                    .map(|v| v.map(|c| (c * 255.0).round() as u8))
                    .collect();
                let mut turned = Rgba { w: sx, h: sy, px };
                if art.line > 0.0 {
                    let rgb = match (&pal, line_index) {
                        (Some((p, _)), Some(i)) => p.rgb[i],
                        _ => art.line_rgb,
                    };
                    for (t, m) in draw_line(&turned, inner, outer, art.brush, tiles)
                        .into_iter()
                        .enumerate()
                    {
                        if m {
                            turned.px[t] = [rgb[0], rgb[1], rgb[2], 255];
                        }
                    }
                }
                sheet.blit(k * sx, row as u32 * sy, &turned);
            }
        }
        let rot_index = pal
            .as_ref()
            .filter(|(p, _)| p.rgb.len() <= 255)
            .map(|(p, _)| index_image(&sheet, p, None));
        rotations = Some((sheet, rot_index, (sx, sy)));
    }

    let index_img = match (&index, &pal) {
        (Some(ix), Some((p, _))) if p.rgb.len() <= 255 => Some(index_image(&img, p, Some(ix))),
        (Some(_), Some((p, _))) => {
            what.push(format!(
                "no index texture: {} colours are more than an index holds (255)",
                p.rgb.len()
            ));
            None
        }
        _ => None,
    };
    Done {
        pixel: art.pixels || index.is_some(),
        image: img,
        layout,
        palette: pal,
        index: index_img,
        rotations,
        what,
    }
}

/// Each texel's cell corner (the dither's origin); texels between cells
/// take their own position's cell or (0, 0).
fn cell_origins(cells: &[Rect], w: u32, h: u32) -> Vec<(u32, u32)> {
    let mut o = vec![(0, 0); (w * h) as usize];
    for c in cells {
        for y in c[1]..(c[1] + c[3]).min(h) {
            for x in c[0]..(c[0] + c[2]).min(w) {
                o[(y * w + x) as usize] = (c[0], c[1]);
            }
        }
    }
    o
}

/// The texels of a cell that take the line: inside and outside widths,
/// both from the shape before either is drawn.
fn draw_line(cell: &Rgba, inner: f64, outer: f64, brush: Brush, tile: bool) -> Vec<bool> {
    let alpha = |x: u32, y: u32| cell.at(x, y)[3];
    let a = outline::mask(cell.w, cell.h, &alpha, inner, brush, true, !tile);
    let b = outline::mask(cell.w, cell.h, &alpha, outer, brush, false, !tile);
    a.into_iter().zip(b).map(|(a, b)| a || b).collect()
}

/// The index texture (og-cook's README, §Art conformance): the conformed
/// colour in RGB, the palette index plus one in alpha (0 transparent).
fn index_image(img: &Rgba, p: &Palette, ix: Option<&Vec<Option<usize>>>) -> Rgba {
    let mut exact: HashMap<[u8; 3], usize> = HashMap::new();
    for (i, c) in p.rgb.iter().enumerate().rev() {
        exact.insert(*c, i);
    }
    let px = img
        .px
        .iter()
        .enumerate()
        .map(|(k, q)| {
            if q[3] == 0 {
                return [0, 0, 0, 0];
            }
            let i = ix
                .and_then(|ix| ix[k])
                .or_else(|| exact.get(&[q[0], q[1], q[2]]).copied())
                .unwrap_or_else(|| {
                    let lab = colour::lab([q[0], q[1], q[2]]);
                    (0..p.rgb.len())
                        .min_by_key(|&i| colour::dist2(lab, p.lab[i]))
                        .unwrap_or(0)
                });
            [q[0], q[1], q[2], (i + 1) as u8]
        })
        .collect();
    Rgba {
        w: img.w,
        h: img.h,
        px,
    }
}

fn distinct(img: &Rgba) -> usize {
    img.px
        .iter()
        .filter(|p| p[3] > 0)
        .map(|p| [p[0], p[1], p[2]])
        .collect::<BTreeSet<_>>()
        .len()
}
