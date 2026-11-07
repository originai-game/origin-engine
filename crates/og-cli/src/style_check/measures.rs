//! Image and model measurements and art-contract judgments.

use super::*;

/// Texels per logical pixel along each axis, and the boundaries between
/// colours that lie off that grid (half pixels).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    pub kx: u32,
    pub ky: u32,
    /// Columns `x` where a colour boundary lies between `x - 1` and `x`
    /// off the grid.
    pub half_columns: Vec<u32>,
    pub half_rows: Vec<u32>,
}

/// An RGBA8 image.
#[derive(Clone, Debug)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub px: Vec<[u8; 4]>,
}

impl Image {
    pub fn from_rgba(w: u32, h: u32, bytes: &[u8]) -> Image {
        Image {
            w,
            h,
            px: bytes.as_chunks::<4>().0.to_vec(),
        }
    }

    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        self.px[(y * self.w + x) as usize]
    }

    fn opaque(&self, x: u32, y: u32) -> bool {
        self.at(x, y)[3] >= 128
    }

    /// The texel as the grid compares it: clear texels are one value.
    fn key(&self, x: u32, y: u32) -> [u8; 4] {
        let p = self.at(x, y);
        if p[3] < 128 {
            [0; 4]
        } else {
            [p[0], p[1], p[2], 255]
        }
    }
}

/// The opaque texels' colours and how many of each.
pub fn colours(img: &Image) -> HashMap<[u8; 3], u64> {
    let mut out = HashMap::new();
    for p in &img.px {
        if p[3] >= 128 {
            *out.entry([p[0], p[1], p[2]]).or_insert(0) += 1;
        }
    }
    out
}

/// The share of `counts` (colours and how many of each) within ΔE00
/// [`PALETTE_DELTA_E`] of a colour of `palette`; `None` with nothing to count.
pub fn palette_hit(counts: &HashMap<[u8; 3], u64>, palette: &[[u8; 3]]) -> Option<f64> {
    let total: u64 = counts.values().sum();
    if total == 0 || palette.is_empty() {
        return None;
    }
    let pal: Vec<[f64; 3]> = palette.iter().map(|&c| lab(c)).collect();
    let hit: u64 = counts
        .iter()
        .filter(|(c, _)| {
            let l = lab(**c);
            pal.iter().any(|p| delta_e00(l, *p) <= PALETTE_DELTA_E)
        })
        .map(|(_, n)| n)
        .sum();
    Some(hit as f64 / total as f64)
}

/// The grid along one axis from the count of colour boundaries at each
/// position (`counts[i]`: boundaries between `i - 1` and `i`): the largest
/// period k (up to [`GRID_MAX`]) whose grid, at its best offset, has at
/// most [`GRID_OFF`] of the positions off it carrying a boundary, with at
/// least three positions on it that do and two of them one logical pixel
/// apart (so a sprite of few, large blocks is not taken for a coarse
/// grid); and the positions off it that carry one (half pixels).
///
/// Only the positions from the first boundary to the last count: a run
/// of one flat colour (a tile's solid ground under a fringe of grass) has
/// no boundary on or off any grid, so it is no evidence for a period.
fn axis_grid(counts: &[u64]) -> (u32, Vec<u32>) {
    let len = counts.len() as u32;
    let edge = |i: u32| i > 0 && counts[i as usize] > 0;
    let Some(first) = (1..len).find(|&i| edge(i)) else {
        return (1, Vec::new());
    };
    let last = (1..len).rfind(|&i| edge(i)).unwrap_or(first);
    let span = first..last + 1;
    for k in (2..=GRID_MAX.min(len / 2)).rev() {
        for o in 0..k {
            let on = |i: u32| (i + k - o % k).is_multiple_of(k);
            let positions = span.clone().filter(|&i| !on(i)).count();
            let off: Vec<u32> = span.clone().filter(|&i| !on(i) && edge(i)).collect();
            let hits: Vec<u32> = span.clone().filter(|&i| on(i) && edge(i)).collect();
            let adjacent = hits.windows(2).any(|w| w[1] - w[0] == k);
            if positions > 0
                && (off.len() as f64) <= GRID_OFF * positions as f64
                && hits.len() >= 3
                && adjacent
            {
                return (k, off);
            }
        }
    }
    (1, Vec::new())
}

/// The image's pixel grid: texels per logical pixel across and down, and
/// the boundaries between colours that lie off it.
pub fn grid(img: &Image) -> Grid {
    let mut cols = vec![0u64; img.w as usize];
    let mut rows = vec![0u64; img.h as usize];
    for y in 0..img.h {
        for x in 0..img.w {
            if x > 0 && img.key(x, y) != img.key(x - 1, y) {
                cols[x as usize] += 1;
            }
            if y > 0 && img.key(x, y) != img.key(x, y - 1) {
                rows[y as usize] += 1;
            }
        }
    }
    let (kx, half_columns) = axis_grid(&cols);
    let (ky, half_rows) = axis_grid(&rows);
    Grid {
        kx,
        ky,
        half_columns,
        half_rows,
    }
}

/// The width of the line round the image's silhouette, in texels: the
/// rings of opaque texels in from the clear ones (4-neighbour steps; the
/// image's border is not clear, as tiles continue into each other) that
/// one dark colour holds, at least 60 % of each ring. `line` is the
/// contract's line colour, when it sets one; otherwise the ring's colour
/// must be dark (L* under 35). `cells` are the sheet's frames or tiles
/// (empty: the whole image is one). `None` when it has no silhouette.
pub fn outline_width(img: &Image, line: Option<[u8; 3]>, cells: &[[u32; 4]]) -> Option<u32> {
    let whole = [[0, 0, img.w, img.h]];
    let cells = if cells.is_empty() { &whole[..] } else { cells };
    // Distance (in 4-neighbour steps) of each opaque texel from a clear
    // one of its own cell; a cell's edge is not clear.
    let mut dist = vec![u32::MAX; img.px.len()];
    let (mut clear, mut opaque) = (false, false);
    for &[cx, cy, cw, ch] in cells {
        let mut queue = std::collections::VecDeque::new();
        for y in cy..(cy + ch).min(img.h) {
            for x in cx..(cx + cw).min(img.w) {
                if img.opaque(x, y) {
                    opaque = true;
                } else {
                    clear = true;
                    dist[(y * img.w + x) as usize] = 0;
                    queue.push_back((x, y));
                }
            }
        }
        while let Some((x, y)) = queue.pop_front() {
            let d = dist[(y * img.w + x) as usize];
            let (x, y) = (i64::from(x), i64::from(y));
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                let inside = nx >= i64::from(cx)
                    && ny >= i64::from(cy)
                    && nx < i64::from((cx + cw).min(img.w))
                    && ny < i64::from((cy + ch).min(img.h));
                if !inside {
                    continue;
                }
                let j = (ny as u32 * img.w + nx as u32) as usize;
                if dist[j] == u32::MAX {
                    dist[j] = d + 1;
                    queue.push_back((nx as u32, ny as u32));
                }
            }
        }
    }
    if !clear || !opaque {
        return None;
    }
    let ring = |r: u32| -> HashMap<[u8; 3], u64> {
        let mut out = HashMap::new();
        for (i, p) in img.px.iter().enumerate() {
            if p[3] >= 128 && dist[i] == r {
                *out.entry([p[0], p[1], p[2]]).or_insert(0) += 1;
            }
        }
        out
    };
    let first = ring(1);
    let total: u64 = first.values().sum();
    let (&colour, &n) = first.iter().max_by_key(|(c, n)| (**n, **c))?;
    let ink = lab(colour);
    let dark = match line {
        Some(l) => delta_e00(ink, lab(l)) <= 15.0,
        None => ink[0] < 35.0,
    };
    if !dark || (n as f64) < 0.6 * total as f64 {
        return Some(0);
    }
    let mut width = 1;
    loop {
        let r = ring(width + 1);
        let total: u64 = r.values().sum();
        let near: u64 = r
            .iter()
            .filter(|(c, _)| delta_e00(lab(**c), ink) <= 3.0)
            .map(|(_, n)| n)
            .sum();
        if total == 0 || (near as f64) < 0.6 * total as f64 {
            return Some(width);
        }
        width += 1;
    }
}

/// A model's numbers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelMeasures {
    /// Triangles drawn: each primitive's LOD 0 times the nodes drawing it.
    pub faces: u64,
    /// The share of triangle corners whose normal is its face's.
    pub flat_share: Option<f64>,
    /// Texels per metre over the textured surface: area-weighted median,
    /// 10th and 90th percentiles.
    pub density: Option<[f64; 3]>,
    /// The untextured faces' colours (vertex colour × base colour), per corner.
    pub colours: HashMap<[u8; 3], u64>,
}

pub(super) fn srgb8(linear: f32) -> u8 {
    let c = linear.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// Weighted percentile `q` of `(value, weight)` pairs.
fn percentile(sorted: &[(f64, f64)], q: f64) -> f64 {
    let total: f64 = sorted.iter().map(|p| p.1).sum();
    let mut acc = 0.0;
    for &(v, w) in sorted {
        acc += w;
        if acc >= q * total {
            return v;
        }
    }
    sorted.last().map_or(0.0, |p| p.0)
}

/// Measures a cooked model; `texture_size` gives a texture id's size.
pub fn measure_model(
    m: &CookedModel,
    texture_size: &dyn Fn(&str) -> Option<(u32, u32)>,
) -> ModelMeasures {
    let world = m.node_world();
    let nodes: Vec<usize> = match m.default_scene {
        Some(s) => m.scene_nodes(s),
        None => (0..m.nodes.len()).collect(),
    };
    let mut out = ModelMeasures::default();
    let (mut corners, mut flat) = (0u64, 0u64);
    let mut densities: Vec<(f64, f64)> = Vec::new();
    for &n in &nodes {
        let Some(mesh) = m.nodes[n].mesh.and_then(|i| m.meshes.get(i)) else {
            continue;
        };
        for p in &m.primitives[mesh.primitives.clone()] {
            out.faces += p.triangle_count(0) as u64;
            let lod = p.lods[0];
            let tris = &p.indices[lod.first..lod.first + lod.count];
            let material = m.materials.get(p.material);
            let tex = material
                .and_then(|mat| mat.base_color_texture)
                .and_then(|t| m.textures.get(t))
                .and_then(|t| texture_size(&t.id));
            let factor = material.map_or([1.0; 4], |mat| mat.base_color);
            for t in tris.as_chunks::<3>().0 {
                let v = [0, 1, 2].map(|k| &p.vertices[t[k] as usize]);
                let pos = v.map(|x| glam::Vec3::from(x.position));
                let face = (pos[1] - pos[0]).cross(pos[2] - pos[0]);
                if face.length_squared() <= 1e-20 {
                    continue;
                }
                let fnorm = face.normalize();
                for x in &v {
                    let vn = glam::Vec3::from(x.normal);
                    if vn.length_squared() > 0.0 {
                        corners += 1;
                        if vn.normalize().dot(fnorm).abs() >= FLAT_COS {
                            flat += 1;
                        }
                    }
                }
                match tex {
                    Some((tw, th)) => {
                        let wp = pos.map(|q| world[n].transform_point3(q));
                        let area = f64::from((wp[1] - wp[0]).cross(wp[2] - wp[0]).length()) / 2.0;
                        let uv = v.map(|x| glam::Vec2::from(x.uv));
                        let (a, b) = (uv[1] - uv[0], uv[2] - uv[0]);
                        let uv_area = f64::from((a.x * b.y - a.y * b.x).abs()) / 2.0;
                        if area > 1e-12 && uv_area > 0.0 {
                            let d = (uv_area * f64::from(tw) * f64::from(th) / area).sqrt();
                            densities.push((d, area));
                        }
                    }
                    None => {
                        for x in &v {
                            let c = [0, 1, 2].map(|k| srgb8(x.color[k] * factor[k]));
                            *out.colours.entry(c).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }
    if corners > 0 {
        out.flat_share = Some(flat as f64 / corners as f64);
    }
    if !densities.is_empty() {
        densities.sort_by(|a, b| a.0.total_cmp(&b.0));
        out.density = Some([
            percentile(&densities, 0.5),
            percentile(&densities, 0.1),
            percentile(&densities, 0.9),
        ]);
    }
    out
}

/// Frame edge statistics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Edges {
    /// The share of pixels with a step of luma over 0.1 to the next one
    /// across or down.
    pub density: f64,
    /// Of those steps, the share that is one hard step: the pixels on
    /// either side flat (luma within 0.02 of their own neighbour), as
    /// pixel art and hard outlines draw; blurred and filtered edges ramp.
    pub hard: f64,
    /// The mean luma step at an edge.
    pub contrast: f64,
}

fn luma(p: [u8; 4]) -> f64 {
    (0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2])) / 255.0
}

/// The frame's edge statistics.
pub fn edges(img: &Image) -> Edges {
    let (w, h) = (img.w as i64, img.h as i64);
    let l = |x: i64, y: i64| luma(img.at(x as u32, y as u32));
    let (mut steps, mut hard, mut sum) = (0u64, 0u64, 0.0);
    let mut edge = vec![false; img.px.len()];
    for (dx, dy) in [(1i64, 0i64), (0, 1)] {
        for y in 0..h - dy {
            for x in 0..w - dx {
                let d = (l(x + dx, y + dy) - l(x, y)).abs();
                if d <= 0.1 {
                    continue;
                }
                steps += 1;
                sum += d;
                edge[(y * w + x) as usize] = true;
                let before =
                    !(x - dx >= 0 && y - dy >= 0) || (l(x, y) - l(x - dx, y - dy)).abs() <= 0.02;
                let after = !(x + 2 * dx < w && y + 2 * dy < h)
                    || (l(x + 2 * dx, y + 2 * dy) - l(x + dx, y + dy)).abs() <= 0.02;
                if before && after {
                    hard += 1;
                }
            }
        }
    }
    Edges {
        density: edge.iter().filter(|e| **e).count() as f64 / img.px.len().max(1) as f64,
        hard: if steps > 0 {
            hard as f64 / steps as f64
        } else {
            0.0
        },
        contrast: if steps > 0 { sum / steps as f64 } else { 0.0 },
    }
}

// ── judging ─────────────────────────────────────────────────────────────────

/// One number against one term of the contract.
#[derive(Clone, Debug, PartialEq)]
pub struct Check {
    pub name: &'static str,
    pub value: String,
    pub limit: String,
    pub ok: bool,
}

impl Check {
    fn new(name: &'static str, value: String, limit: String, ok: bool) -> Check {
        Check {
            name,
            value,
            limit,
            ok,
        }
    }
}

/// What an asset is to the check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// A 2D texture: a sprite, flipbook, tile set or UI image.
    Sprite,
    /// A texture on 3D surfaces.
    Texture,
    Model,
    /// A clip: a model's skeletal or morph clip, or a flipbook's.
    Clip,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Sprite => "sprite",
            Role::Texture => "texture",
            Role::Model => "model",
            Role::Clip => "clip",
        }
    }
}

/// One measured asset (a cooked output) and its checks.
#[derive(Clone, Debug)]
pub struct Asset {
    pub id: String,
    pub role: Role,
    /// Why it opted out of conformance, when it did.
    pub off: Option<String>,
    pub measures: Value,
    pub checks: Vec<Check>,
    /// Numbers that are not judged and why (a lossy texture's colours).
    pub notes: Vec<String>,
}

impl Asset {
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|c| c.ok)
    }
}

pub(super) fn pct(x: f64) -> String {
    format!("{:.1}%", x * 100.0)
}

/// The colour checks of `counts` (opaque colours of an asset).
fn colour_checks(
    c: &Contract,
    counts: &HashMap<[u8; 3], u64>,
    measures: &mut Value,
    checks: &mut Vec<Check>,
    judged: bool,
) {
    measures["colours"] = json!(counts.len());
    let hit = c.palette.as_ref().and_then(|(_, p)| palette_hit(counts, p));
    if let Some(h) = hit {
        measures["paletteHit"] = json!(h);
    }
    if !judged {
        return;
    }
    if c.max_colors > 0 {
        checks.push(Check::new(
            "colours",
            counts.len().to_string(),
            format!("<= {} (ArtPalette.maxColors)", c.max_colors),
            counts.len() <= c.max_colors as usize,
        ));
    }
    if let (Some(h), true) = (hit, c.strict) {
        checks.push(Check::new(
            "palette",
            pct(h),
            format!("100% within ΔE00 {PALETTE_DELTA_E} (ArtPalette.strict)"),
            h >= 1.0,
        ));
    }
}

/// Judges a sprite: colours, palette and line now; the grid once the
/// project's texels per pixel are known ([`judge_grids`]).
/// `fixed`: a texture whose layout the cook keeps (a 2D rig's parts, a UI
/// style's atlas), which takes the palette and alpha but no line.
pub fn judge_sprite(
    c: &Contract,
    id: &str,
    img: &Image,
    lossy: bool,
    fixed: bool,
    cells: &[[u32; 4]],
) -> Asset {
    let mut a = Asset {
        id: id.to_string(),
        role: Role::Sprite,
        off: None,
        measures: json!({ "size": [img.w, img.h] }),
        checks: Vec::new(),
        notes: Vec::new(),
    };
    let counts = colours(img);
    colour_checks(c, &counts, &mut a.measures, &mut a.checks, !lossy);
    if lossy {
        a.notes
            .push("lossy encoding: colours and palette not judged".into());
    }
    let g = grid(img);
    a.measures["grid"] = json!({
        "texelsPerPixel": [g.kx, g.ky],
        "halfColumns": g.half_columns,
        "halfRows": g.half_rows,
    });
    let line_rgb = (c.line > 0.0).then_some(c.line_rgb);
    if let Some(found) = outline_width(img, line_rgb, cells) {
        a.measures["line"] = json!(found);
        if c.line > 0.0 && fixed {
            a.notes.push("a fixed layout (a 2D rig's parts or a UI atlas): the cook draws no line into it, so the line is not judged".into());
        } else if c.line > 0.0 {
            let want = c.line.round() as u32;
            a.checks.push(Check::new(
                "line",
                format!("{found} px"),
                format!("{want} px (ArtLine.width)"),
                found == want,
            ));
        }
    }
    a
}

/// The default `ArtLine.color` (0.02 0.02 0.03 linear) in sRGB; the ring
/// test accepts a line within ΔE00 15 of the contract's colour.
pub(super) const LINE_RGB: [u8; 3] = [39, 39, 48];

/// The project's texels per logical pixel: the most common among the
/// sprites' grids (the smaller on a tie); `None` without sprites.
pub fn project_grid(sprites: &[(u32, u32)]) -> Option<u32> {
    let mut count: BTreeMap<u32, usize> = BTreeMap::new();
    for &(kx, ky) in sprites {
        if kx == ky {
            *count.entry(kx).or_insert(0) += 1;
        }
    }
    count
        .into_iter()
        .max_by_key(|&(k, n)| (n, std::cmp::Reverse(k)))
        .map(|(k, _)| k)
}

/// Adds the grid check to each sprite, against the project's texels per
/// logical pixel `k`.
pub fn judge_grids(c: &Contract, assets: &mut [Asset], k: Option<u32>) {
    if !c.pixels {
        return;
    }
    let Some(k) = k else { return };
    for a in assets.iter_mut().filter(|a| a.role == Role::Sprite) {
        // A UI style's atlas draws at the UI's own scale, not the world's
        // grid: its grid is measured, not judged.
        if a.measures["ui"] == true {
            a.notes.push("a UI style's atlas draws at the UI's scale: its grid is not judged against the sprites'".into());
            continue;
        }
        let g = &a.measures["grid"];
        let kxy = g["texelsPerPixel"].as_array().cloned().unwrap_or_default();
        let (kx, ky) = (
            kxy.first().and_then(Value::as_u64),
            kxy.get(1).and_then(Value::as_u64),
        );
        let half = g["halfColumns"].as_array().map_or(0, Vec::len)
            + g["halfRows"].as_array().map_or(0, Vec::len);
        let mut value = match (kx, ky) {
            (Some(x), Some(y)) if x == y => format!("{x} texel(s) per pixel"),
            (Some(x), Some(y)) => format!("{x}x{y} texels per pixel"),
            _ => "unknown".into(),
        };
        if half > 0 {
            let cols: Vec<String> = g["halfColumns"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| v.to_string())
                .collect();
            let rows: Vec<String> = g["halfRows"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| v.to_string())
                .collect();
            let mut parts = Vec::new();
            if !cols.is_empty() {
                parts.push(format!("columns {}", cols.join(" ")));
            }
            if !rows.is_empty() {
                parts.push(format!("rows {}", rows.join(" ")));
            }
            value.push_str(&format!(", half pixels at {}", parts.join(", ")));
        }
        let ok = kx == Some(u64::from(k)) && ky == Some(u64::from(k)) && half == 0;
        a.checks.push(Check::new(
            "grid",
            value,
            format!("{k} texel(s) per pixel, no half pixels (ArtPixels)"),
            ok,
        ));
    }
}

/// Judges a texture on 3D surfaces.
pub fn judge_texture(
    c: &Contract,
    id: &str,
    img: &Image,
    pixel_art: bool,
    levels: u32,
    lossy: bool,
) -> Asset {
    let mut a = Asset {
        id: id.to_string(),
        role: Role::Texture,
        off: None,
        measures: json!({ "size": [img.w, img.h], "nearest": pixel_art, "mips": levels }),
        checks: Vec::new(),
        notes: Vec::new(),
    };
    colour_checks(c, &colours(img), &mut a.measures, &mut a.checks, !lossy);
    if lossy {
        a.notes
            .push("lossy encoding: colours and palette not judged".into());
    }
    if c.max_size > 0 {
        let side = img.w.max(img.h);
        a.checks.push(Check::new(
            "size",
            format!("{}x{}", img.w, img.h),
            format!("<= {} (ArtTexels.maxSize)", c.max_size),
            side <= c.max_size,
        ));
    }
    match c.filter.as_str() {
        "nearest" | "linear" => {
            let nearest = c.filter == "nearest";
            a.checks.push(Check::new(
                "filter",
                (if pixel_art { "nearest" } else { "linear" }).into(),
                format!("{} (ArtTexels.filter)", c.filter),
                pixel_art == nearest,
            ));
        }
        _ => {}
    }
    match c.mips.as_str() {
        "none" => a.checks.push(Check::new(
            "mips",
            format!("{levels} level(s)"),
            "1 level (ArtTexels.mips none)".into(),
            levels <= 1,
        )),
        "full" if img.w.max(img.h) > 1 => a.checks.push(Check::new(
            "mips",
            format!("{levels} level(s)"),
            "a mip chain (ArtTexels.mips full)".into(),
            levels > 1,
        )),
        _ => {}
    }
    a
}

/// Judges a model's numbers.
pub fn judge_model(c: &Contract, id: &str, m: &ModelMeasures, role: ModelRole) -> Asset {
    let mut a = Asset {
        id: id.to_string(),
        role: Role::Model,
        off: None,
        measures: json!({ "faces": m.faces, "character": role == ModelRole::Character }),
        checks: Vec::new(),
        notes: Vec::new(),
    };
    let (budget, name) = match role {
        ModelRole::Character => (c.character_faces, "ArtShape.characterFaces"),
        ModelRole::Prop => (c.prop_faces, "ArtShape.propFaces"),
        ModelRole::Effect => (0, ""),
    };
    if budget > 0 {
        a.checks.push(Check::new(
            "faces",
            m.faces.to_string(),
            format!("<= {budget} ({name})"),
            m.faces <= u64::from(budget),
        ));
    }
    if let Some(f) = m.flat_share {
        a.measures["flatShare"] = json!(f);
        if c.normals == "flat" {
            a.checks.push(Check::new(
                "normals",
                format!("{} flat", pct(f)),
                format!(">= {} flat (ArtShape.normals flat)", pct(FLAT_SHARE)),
                f >= FLAT_SHARE,
            ));
        }
    }
    if let Some([median, p10, p90]) = m.density {
        a.measures["density"] = json!({ "median": median, "p10": p10, "p90": p90 });
        if c.density > 0.0 {
            a.checks.push(Check::new(
                "density",
                format!("{median:.1} texels/m (p10 {p10:.1}, p90 {p90:.1})"),
                format!(
                    "{:.1} to {:.1} (ArtTexels.density {} within x2)",
                    c.density / 2.0,
                    c.density * 2.0,
                    c.density
                ),
                median >= c.density / 2.0 && median <= c.density * 2.0,
            ));
        }
    }
    if !m.colours.is_empty() {
        colour_checks(c, &m.colours, &mut a.measures, &mut a.checks, true);
    }
    a
}

// ── motion and camera (ArtMotion, ArtCamera) ────────────────────────────────

/// A rate within this share of `ArtMotion.stepFps` holds it (the cook's
/// rates are the contract's own number, through f32).
const RATE_SLACK: f64 = 1e-3;

/// A held duration is a whole number of steps within this many seconds.
const STEP_SLACK: f64 = 1e-4;

/// A camera angle within this many degrees of `ArtCamera.pitch` or `yaw`
/// holds it.
pub const ANGLE_SLACK: f64 = 0.5;

/// A cooked model clip against `ArtMotion`: its resampling rate and
/// whether its moving channels hold each pose (`stepFps`), and its frames
/// (`maxFrames`).
pub fn judge_clip(c: &Contract, id: &str, clip: &og_anim::ClipAsset) -> Asset {
    let moving: Vec<&og_anim::Channel> = clip
        .channels
        .iter()
        .filter(|ch| !matches!(ch.samples, og_anim::Samples::Constant(_)))
        .collect();
    let stepped = moving.iter().all(|ch| ch.step);
    let mut a = Asset {
        id: format!("{id}:{}", clip.name),
        role: Role::Clip,
        off: None,
        measures: json!({ "rate": clip.rate, "frames": clip.frames, "stepped": stepped }),
        checks: Vec::new(),
        notes: Vec::new(),
    };
    if c.step_fps > 0.0 {
        let rate = f64::from(clip.rate);
        let ok = (rate - c.step_fps).abs() <= c.step_fps * RATE_SLACK
            || (c.max_frames > 0 && rate < c.step_fps);
        a.checks.push(Check::new(
            "stepFps",
            format!(
                "{rate:.3} Hz, {}",
                if stepped {
                    "each pose held"
                } else {
                    "interpolated"
                }
            ),
            format!("{} Hz, held (ArtMotion.stepFps)", c.step_fps),
            ok && (stepped || moving.is_empty()),
        ));
    }
    if c.max_frames > 0 {
        a.checks.push(Check::new(
            "maxFrames",
            clip.frames.to_string(),
            format!("<= {} (ArtMotion.maxFrames)", c.max_frames),
            clip.frames <= c.max_frames,
        ));
    }
    a
}

/// A cooked flipbook's clips against `ArtMotion`: every frame held a whole
/// number of steps (`stepFps`), and at most `maxFrames` frames a clip.
pub fn judge_flipbook(c: &Contract, id: &str, sheet: &og_anim::FlipbookSheet) -> Vec<Asset> {
    sheet
        .clips
        .iter()
        .map(|clip| {
            let mut a = Asset {
                id: format!("{id}:{}", clip.name),
                role: Role::Clip,
                off: None,
                measures: json!({ "frames": clip.frames.len(), "durations": clip.durations }),
                checks: Vec::new(),
                notes: Vec::new(),
            };
            if c.step_fps > 0.0 {
                let off = clip
                    .durations
                    .iter()
                    .filter(|&&d| {
                        let steps = f64::from(d) * c.step_fps;
                        steps < 1.0 - STEP_SLACK * c.step_fps
                            || (steps - steps.round()).abs() > STEP_SLACK * c.step_fps
                    })
                    .count();
                a.checks.push(Check::new(
                    "stepFps",
                    format!("{off} of {} frames off the steps", clip.durations.len()),
                    format!(
                        "every frame a whole number of 1/{} s (ArtMotion.stepFps)",
                        c.step_fps
                    ),
                    off == 0,
                ));
            }
            if c.max_frames > 0 {
                a.checks.push(Check::new(
                    "maxFrames",
                    clip.frames.len().to_string(),
                    format!("<= {} (ArtMotion.maxFrames)", c.max_frames),
                    clip.frames.len() <= c.max_frames as usize,
                ));
            }
            a
        })
        .collect()
}

/// The numbers of an `og.Camera` read through the probe (`get:og.Camera`):
/// a vector field as `{x, y, z[, w]}` or an array.
fn lanes(v: &Value, n: usize) -> Option<Vec<f64>> {
    match v {
        Value::Array(a) if a.len() >= n => a[..n].iter().map(Value::as_f64).collect(),
        Value::Object(o) => ["x", "y", "z", "w"][..n]
            .iter()
            .map(|k| o.get(*k).and_then(Value::as_f64))
            .collect(),
        _ => None,
    }
}

/// The camera a frame was captured with (its `og.Camera` fields) against
/// `ArtCamera`: the projection, with `fixedAngle` its pitch (below the
/// horizon) and yaw (about +Y, the camera looking down its local -Z), and
/// a perspective camera's field of view against `fovMin` and `fovMax`.
/// The measures and the checks; no check where the contract sets none.
pub fn judge_camera(c: &Contract, camera: &Value) -> (Value, Vec<Check>) {
    let mut checks = Vec::new();
    let ortho = camera["orthographic"].as_bool().unwrap_or(false);
    let fov = camera["fov"].as_f64();
    // The forward axis: the rotation applied to (0, 0, -1).
    let (pitch, yaw) = match lanes(&camera["quaternion"], 4).as_deref() {
        Some(&[x, y, z, w]) => {
            let f = [
                -2.0 * (x * z + w * y),
                -2.0 * (y * z - w * x),
                -(1.0 - 2.0 * (x * x + y * y)),
            ];
            let len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt().max(1e-12);
            (
                Some((-f[1] / len).clamp(-1.0, 1.0).asin().to_degrees()),
                Some((-f[0]).atan2(-f[2]).to_degrees()),
            )
        }
        _ => (None, None),
    };
    let measures = json!({
        "projection": if ortho { "orthographic" } else { "perspective" },
        "pitch": pitch,
        "yaw": yaw,
        "fov": fov,
    });
    if c.projection == "perspective" || c.projection == "orthographic" {
        let is = if ortho { "orthographic" } else { "perspective" };
        checks.push(Check::new(
            "projection",
            is.to_string(),
            format!("{} (ArtCamera.projection)", c.projection),
            is == c.projection,
        ));
    }
    if c.fixed_angle {
        // Angles compared the short way round.
        let near =
            |a: f64, b: f64| ((a - b + 540.0).rem_euclid(360.0) - 180.0).abs() <= ANGLE_SLACK;
        let show = |v: Option<f64>| v.map_or("unknown".to_string(), |v| format!("{v:.2}°"));
        checks.push(Check::new(
            "pitch",
            show(pitch),
            format!("{}° ± {ANGLE_SLACK} (ArtCamera.pitch)", c.pitch),
            pitch.is_some_and(|p| near(p, c.pitch)),
        ));
        // Straight down, the yaw is any (the forward axis has no heading).
        let down = pitch.is_some_and(|p| p.abs() > 90.0 - ANGLE_SLACK);
        checks.push(Check::new(
            "yaw",
            show(yaw),
            format!("{}° ± {ANGLE_SLACK} (ArtCamera.yaw)", c.yaw),
            down || yaw.is_some_and(|y| near(y, c.yaw)),
        ));
    }
    if !ortho && (c.fov_min > 0.0 || c.fov_max > 0.0) {
        let lo = if c.fov_min > 0.0 { c.fov_min } else { 0.0 };
        let hi = if c.fov_max > 0.0 { c.fov_max } else { 180.0 };
        checks.push(Check::new(
            "fov",
            fov.map_or("unknown".into(), |f| format!("{f:.1}°")),
            format!("{lo}° to {hi}° (ArtCamera.fovMin, fovMax)"),
            fov.is_some_and(|f| f >= lo - 1e-3 && f <= hi + 1e-3),
        ));
    }
    (measures, checks)
}
