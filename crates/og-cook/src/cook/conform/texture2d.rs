//! Conformance of 2D textures (docs/art.md §2.3, stream A2): every texture
//! a sprite, a flipbook, a tile set or a UI image draws is cooked to the
//! project's art contract. The source image is never written; the pass
//! puts the conformed image in the texture's `Source::conformed`, rewrites
//! the in-memory text of the sheets whose cells it re-lays out, and adds
//! the index texture (and the rotations and the asset's own palette) as
//! sources of their own. og-cook's README (§Art conformance) is the
//! contract of what it writes.
//!
//! Which textures: those a flipbook or a tile set names (cells: its frames
//! or tiles), those a 2D rig, a UI style or a map's own tileset names (a
//! layout the cook cannot move: palette and alpha only), and the textures
//! no such asset names that are 2D by themselves: `kind pixel` in their
//! import settings, or any image in a `profile "2d"` project. An image's
//! import settings may say `conform sprite` or `conform ui` for one the
//! cook would not find (a HUD image of a 3D game), `conform off` to keep
//! it as it is, and `conform { size 32 }` for the cell size it takes.
//!
//! What, in order, per cell: the `Stylise` paint fields in texel space
//! (painted styles); the pixel-art downscale to the contract's cell size
//! (`ArtPixels.spriteSizes`, a tile set's tiles to `ArtPixels.unit`) and
//! 1-bit alpha; the palette (`ArtPalette`); the line (`ArtLine`, inside
//! the cell); the rotations (`ArtPixels.rotation prerotated`).

mod cleanedge;
mod colour;
mod outline;
mod paint;
mod palette;
mod resample;

use super::super::{Kind, Source, flipbook, look, settings_block, tilemap};
use super::{AssetConform, ConformAction, ConformEntry, ProjectConform};
use crate::look::LookFile;
use og_core::hash::StableHasher;
use og_core::ogt::{Document, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use outline::Brush;
use paint::Paint;
use palette::{Dither, Palette};

/// Moves every conformed output's stamp when the pass's algorithms change.
const VERSION: &str = "conform-2d 1";

/// A cell of a texture: x, y, width, height in texels.
type Rect = [u32; 4];

/// An RGBA8 image.
#[derive(Clone, Debug, PartialEq)]
struct Rgba {
    w: u32,
    h: u32,
    px: Vec<[u8; 4]>,
}

impl Rgba {
    fn new(w: u32, h: u32) -> Rgba {
        Rgba {
            w,
            h,
            px: vec![[0; 4]; (w * h) as usize],
        }
    }

    fn decode(bytes: &[u8]) -> Result<Rgba, String> {
        let img = super::super::texture::decode(bytes)?;
        Ok(Rgba {
            w: img.width,
            h: img.height,
            px: img.rgba.as_chunks::<4>().0.to_vec(),
        })
    }

    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        self.px[(y * self.w + x) as usize]
    }

    fn crop(&self, r: Rect) -> Rgba {
        let mut out = Rgba::new(r[2], r[3]);
        for y in 0..r[3] {
            for x in 0..r[2] {
                out.px[(y * r[2] + x) as usize] = self.at(r[0] + x, r[1] + y);
            }
        }
        out
    }

    fn blit(&mut self, x0: u32, y0: u32, cell: &Rgba) {
        for y in 0..cell.h {
            for x in 0..cell.w {
                self.px[((y0 + y) * self.w + x0 + x) as usize] = cell.at(x, y);
            }
        }
    }

    /// PNG bytes (the cook decodes them as any source image).
    fn png(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let flat: Vec<u8> = self.px.iter().flatten().copied().collect();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(&flat, self.w, self.h, image::ExtendedColorType::Rgba8)
            .expect("an in-memory PNG");
        out
    }
}

use image::ImageEncoder;

/// The 2D part of the art contract, every field at its meaning.
#[derive(Clone, Debug)]
struct Art {
    colors: Option<(String, Palette)>,
    strict: bool,
    max_colors: u32,
    ramp_steps: u32,
    accents: Vec<u32>,
    dither: Dither,
    unit: f64,
    sprite_sizes: Vec<u32>,
    /// Pixel art: `ArtPixels` sets a grid (`unit`, `logicalHeight` or
    /// `spriteSizes`), so alpha is one bit and textures are lossless.
    pixels: bool,
    rotation_steps: u32,
    line: f64,
    line_rgb: [u8; 3],
    place: String,
    brush: Brush,
    paint: Paint,
    /// Every input of the contract's (with the palette's and canvas's
    /// bytes): the stamps of conformed outputs carry it.
    hash: u64,
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
    og_api::host::RECORDS
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

impl Art {
    /// The contract of the flattened look `full` (every block), or `None`
    /// when it constrains no 2D texture.
    fn of(full: &LookFile, contract: &LookFile, assets: &Path) -> Result<Option<Art>, String> {
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
        let mut paint = Paint {
            radius: num(full, "Stylise", "paintRadius"),
            anisotropy: num(full, "Stylise", "paintAnisotropy"),
            edge: num(full, "Stylise", "paintEdgeDarkening"),
            bleed: num(full, "Stylise", "paintBleed"),
            canvas: None,
            canvas_strength: num(full, "Stylise", "paintCanvasStrength"),
            canvas_scale: num(full, "Stylise", "paintCanvasScale"),
        };
        if let Some([Value::String(id)]) = block(full, "Stylise", "paintCanvas")
            && paint.canvas_strength > 0.0
            && let Some(bytes) = named(assets, id)
        {
            h.write(&bytes);
            let img = Rgba::decode(&bytes).map_err(|e| format!("Stylise.paintCanvas {id}: {e}"))?;
            let grain = img.px.iter().map(|p| colour::to_linear(p[0])).collect();
            paint.canvas = Some((grain, img.w, img.h));
        }
        for b in full.blocks.iter().filter(|b| b.resource == "Stylise") {
            for (f, v, _) in b.fields.iter().filter(|(f, _, _)| f.starts_with("paint")) {
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
        Ok((art.any_field(contract) && (constrains || art.paint.active())).then_some(art))
    }

    /// Any art field of the contract set off its zero value.
    fn any_field(&self, contract: &LookFile) -> bool {
        contract.blocks.iter().any(|b| {
            b.fields.iter().any(|(_, v, _)| {
                v.iter().any(|x| match x {
                    Value::Number(n) => n.as_f64() != 0.0,
                    Value::Word(w) => !matches!(
                        w.as_str(),
                        "false" | "free" | "any" | "none" | "outside" | "diamond"
                    ),
                    Value::String(_) => true,
                })
            })
        })
    }

    fn palette_active(&self) -> bool {
        self.colors.is_some() || self.max_colors > 0 || self.ramp_steps > 0
    }
}

/// How the cell layout of a sheet's texture is laid out on a grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Grid {
    cols: u32,
    rows: u32,
    w: u32,
    h: u32,
    mx: u32,
    my: u32,
    sx: u32,
    sy: u32,
    /// Texels right of and under the last cell.
    right: u32,
    bottom: u32,
}

/// The grid a list of cells lies on, when they do: one size, at steps of
/// one spacing from one margin.
fn grid_of(cells: &[Rect], size: (u32, u32)) -> Option<Grid> {
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
struct Layout {
    size: (u32, u32),
    cells: Vec<(Rect, Rect)>,
    grid: Option<Grid>,
}

impl Layout {
    fn identity(size: (u32, u32), cells: &[Rect]) -> Layout {
        Layout {
            size,
            cells: cells.iter().map(|&c| (c, c)).collect(),
            grid: None,
        }
    }

    fn moved(&self, old: Rect) -> Option<Rect> {
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
fn relayout(cells: &[Rect], size: (u32, u32), t: u32, long: u32, grid: Option<Grid>) -> Layout {
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

/// What a texture is to the 2D conformance.
#[derive(Clone, Debug)]
enum Role {
    /// A sprite drawn whole: one cell, the image.
    Sprite,
    /// A sheet's cells (a flipbook's frames, a tile set's tiles).
    Sheet {
        cells: Vec<Rect>,
        grid: Option<Grid>,
        tiles: bool,
    },
    /// A layout the cook does not move (a UI style's atlas, a 2D rig's
    /// parts, a map's own tileset): palette and alpha only.
    Fixed(String),
}

/// One asset that names a texture.
#[derive(Clone, Debug)]
struct Use {
    by: usize,
    kind: UseKind,
    off: bool,
    size: Option<u32>,
}

#[derive(Clone, Debug)]
enum UseKind {
    Flipbook(Box<og_anim::FlipbookSheet>),
    Tileset(Box<og_tilemap::Tileset>),
    Fixed(&'static str),
}

/// The result of conforming one texture.
struct Done {
    image: Rgba,
    layout: Layout,
    /// The colours mapped onto (for the index), with their source: the
    /// contract's palette id, or the asset's own.
    palette: Option<(Palette, Option<String>)>,
    index: Option<Rgba>,
    rotations: Option<(Rgba, Option<Rgba>, (u32, u32))>,
    pixel: bool,
    what: Vec<String>,
}

/// The texture ids a sheet-like asset's `texture` line names, resolved as
/// its own importer resolves it.
fn style_images(s: &Source, known: &BTreeSet<String>) -> Vec<String> {
    let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
        return Vec::new();
    };
    let dir = s.rel.rsplit_once('/').map_or("", |(d, _)| d);
    ["atlas", "icons"]
        .iter()
        .filter_map(|k| {
            doc.body
                .get(k)?
                .values
                .first()?
                .as_str()
                .map(str::to_string)
        })
        .map(|p| {
            let rel = if dir.is_empty() {
                p.clone()
            } else {
                format!("{dir}/{p}")
            };
            if known.contains(&rel) {
                rel
            } else {
                crate::runtime::asset_id(&p).to_string()
            }
        })
        .collect()
}

const IMAGE_ROLES: [&str; 2] = ["sprite", "ui"];

type SheetText = (usize, Vec<u8>, Arc<Vec<u8>>, u64);

/// Conforms the 2D textures of `sources` to the project's look
/// (`contract`: its id and art blocks), in place. Called once by
/// `cook_project` before planning; returns the report's lines.
pub(in crate::cook) fn conform(
    root: &Path,
    assets: &Path,
    sources: &mut Vec<Source>,
    known: &BTreeSet<String>,
    contract: Option<&(String, LookFile)>,
) -> Result<Vec<ConformEntry>, String> {
    let project = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok());
    let project_off = match &project {
        Some(d) => ProjectConform::parse(&d.body).map_err(|e| format!("project.ogt:{e}"))?,
        None => ProjectConform::default(),
    };
    let two_d = project
        .as_ref()
        .and_then(|d| {
            d.body
                .get("profile")?
                .values
                .first()?
                .as_str()
                .map(|p| p == "2d")
        })
        .unwrap_or(false);

    // A sheet's own `conform` line is read here and taken out of its text
    // (its importer does not know it), with or without a look.
    let mut sheet_conform: BTreeMap<usize, AssetConform> = BTreeMap::new();
    let mut entries = Vec::new();
    for (i, s) in sources.iter_mut().enumerate() {
        if !matches!(s.kind, Kind::Flipbook | Kind::Tileset | Kind::Rig2d) {
            continue;
        }
        if let Some((text, c)) = super::strip(&String::from_utf8_lossy(&s.bytes), &[]) {
            match c {
                Ok(c) => {
                    sheet_conform.insert(i, c);
                }
                Err(e) => s.error = Some(e),
            }
            s.bytes = text.into_bytes();
            s.hash = rehash(s, &[]);
        }
    }

    let Some((look_id, contract)) = contract else {
        return Ok(entries);
    };
    let Some(look_source) = sources.iter().find(|s| &s.rel == look_id) else {
        return Ok(entries);
    };
    let full = look::flattened(look_source, assets, &|id| known.contains(id))?;
    let Some(art) = Art::of(&full, contract, assets)? else {
        return Ok(entries);
    };

    // Who names which texture.
    let mut uses: BTreeMap<String, Vec<Use>> = BTreeMap::new();
    for (i, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        let c = sheet_conform.get(&i);
        let mut add = |id: String, kind: UseKind| {
            uses.entry(id).or_default().push(Use {
                by: i,
                kind,
                off: c.is_some_and(|c| c.off) || project_off.off_by(&s.rel).is_some(),
                size: c.and_then(|c| c.size),
            });
        };
        match s.kind {
            Kind::Flipbook => {
                if let Ok(sheet) = flipbook::sheet(s) {
                    add(sheet.texture.clone(), UseKind::Flipbook(Box::new(sheet)));
                }
            }
            Kind::Tileset => {
                if let Ok(t) = og_tilemap::Tileset::parse(&String::from_utf8_lossy(&s.bytes))
                    && !t.texture.is_empty()
                {
                    add(
                        crate::runtime::asset_id(&t.texture).to_string(),
                        UseKind::Tileset(Box::new(t)),
                    );
                }
            }
            Kind::Rig2d => {
                if let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes))
                    && let Some(p) = doc
                        .body
                        .get("texture")
                        .and_then(|e| e.values.first()?.as_str())
                {
                    add(flipbook::resolve(s, p), UseKind::Fixed("a 2D rig's parts"));
                }
            }
            Kind::Style => {
                for id in style_images(s, known) {
                    add(id, UseKind::Fixed("a UI style's atlas"));
                }
            }
            Kind::Tilemap => {
                if let Ok(m) = tilemap::MapSettings::parse(settings_block(s, "tilemap"))
                    && let Ok(map) = tilemap::import(&s.rel, &s.bytes, &m, &|id| known.contains(id))
                {
                    for t in &map.tilesets {
                        if let og_tilemap::TilesetRef::Embedded(t) = t
                            && !t.texture.is_empty()
                        {
                            add(t.texture.clone(), UseKind::Fixed("a map's own tileset"));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Each texture: its role, conformed, and what it writes.
    let mut extra: Vec<Source> = Vec::new();
    // A sheet's new text, the conformed image it reads, the look's hash.
    let mut sheet_texts: Vec<SheetText> = Vec::new();
    for i in 0..sources.len() {
        let s = &sources[i];
        if s.kind != Kind::Texture
            || s.error.is_some()
            || s.rel.to_ascii_lowercase().ends_with(".ktx2")
        {
            continue;
        }
        let used: Vec<Use> = uses.get(&s.rel).cloned().unwrap_or_default();
        let own = match s
            .settings
            .as_ref()
            .map(|(_, d)| AssetConform::of(&d.body, &IMAGE_ROLES))
        {
            Some(Err(e)) => {
                sources[i].error = Some(e);
                continue;
            }
            Some(Ok(c)) => c,
            None => None,
        };
        let kind = super::super::texture_kind(s).unwrap_or(crate::format::kind::COLOR);
        let data = kind == crate::format::kind::NORMAL || kind == crate::format::kind::LINEAR;
        let role = if let Some(r) = own.as_ref().and_then(|c| c.role.as_deref()) {
            Some(if r == "ui" {
                Role::Fixed("its import settings (conform ui)".into())
            } else {
                Role::Sprite
            })
        } else if let Some(u) = used.iter().find(|u| matches!(u.kind, UseKind::Fixed(_))) {
            let UseKind::Fixed(what) = u.kind else {
                unreachable!()
            };
            Some(Role::Fixed(format!("{what} ({})", sources[u.by].rel)))
        } else if !used.is_empty() {
            sheet_role(s, &used)
        } else if !data && (two_d || kind == crate::format::kind::PIXEL) {
            Some(Role::Sprite)
        } else {
            None
        };
        let Some(role) = role else { continue };
        let off_why = if own.as_ref().is_some_and(|c| c.off) {
            Some(format!("conform off (assets/{}.ogt)", s.rel))
        } else if let Some(p) = project_off.off_by(&s.rel) {
            Some(format!("project.ogt's conform block: off \"{p}\""))
        } else if let Some(u) = used.iter().find(|u| u.off) {
            Some(match project_off.off_by(&sources[u.by].rel) {
                Some(p) => format!(
                    "{} is off in project.ogt's conform block (\"{p}\")",
                    sources[u.by].rel
                ),
                None => format!("conform off in {}", sources[u.by].rel),
            })
        } else if data {
            Some("a normal or linear texture is data, not art: not conformed".into())
        } else {
            None
        };
        if let Some(why) = off_why {
            entries.push(ConformEntry {
                source: s.rel.clone(),
                action: ConformAction::Off,
                what: why,
            });
            continue;
        }
        let bytes = s.conformed.as_deref().map_or(&s.bytes[..], |b| &b[..]);
        let img = match Rgba::decode(bytes) {
            Ok(img) => img,
            Err(_) => continue, // its own cook reports the image
        };
        let size = own
            .as_ref()
            .and_then(|c| c.size)
            .or_else(|| used.iter().find_map(|u| u.size));
        let done = run(&art, &img, &role, size);
        let changed = done.image != img || done.index.is_some() || done.rotations.is_some();
        let base = s.rel.clone();
        let mut what = done.what.clone();
        let png = Arc::new(done.image.png());
        // Sheets whose cells moved get their text again; every sheet of a
        // conformed texture reads the conformed image.
        if changed {
            for u in &used {
                let text = match &u.kind {
                    UseKind::Flipbook(sheet) => {
                        let mut sheet = (**sheet).clone();
                        for f in &mut sheet.frames {
                            if let Some(r) = done.layout.moved(f.rect) {
                                f.rect = r;
                            }
                        }
                        sheet.size = [done.layout.size.0, done.layout.size.1];
                        sheet.to_text().into_bytes()
                    }
                    UseKind::Tileset(t) => match done.layout.grid {
                        Some(g) if done.layout.size != (img.w, img.h) => {
                            let mut t = (**t).clone();
                            let f = |v: f32, old: u32, new: u32| v * new as f32 / old.max(1) as f32;
                            for d in t.tiles.values_mut() {
                                if let Some(og_tilemap::Collision::Rect { x, y, w, h }) =
                                    &mut d.collision
                                {
                                    *x = f(*x, t.tile_width, g.w);
                                    *w = f(*w, t.tile_width, g.w);
                                    *y = f(*y, t.tile_height, g.h);
                                    *h = f(*h, t.tile_height, g.h);
                                }
                            }
                            t.tile_width = g.w;
                            t.tile_height = g.h;
                            t.margin = g.mx;
                            t.spacing = g.sx;
                            t.to_text().into_bytes()
                        }
                        _ => sources[u.by].bytes.clone(),
                    },
                    UseKind::Fixed(_) => continue,
                };
                if text != sources[u.by].bytes {
                    entries.push(ConformEntry {
                        source: sources[u.by].rel.clone(),
                        action: ConformAction::Changed,
                        what: format!(
                            "its cells follow {base}'s conformed layout ({}x{} texels)",
                            done.layout.size.0, done.layout.size.1
                        ),
                    });
                }
                sheet_texts.push((u.by, text, png.clone(), art.hash));
            }
        }
        // The extra outputs, as sources of their own.
        let pixel_import = "type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n";
        let path = s.path.clone();
        let mut push = |rel: String, kind: Kind, bytes: Vec<u8>, settings: Option<&str>| {
            let settings =
                settings.map(|t| (t.to_string(), Document::parse(t).expect("the import text")));
            let mut src = Source {
                rel,
                path: path.clone(),
                kind,
                bytes,
                settings,
                hash: 0,
                gltf: None,
                error: None,
                encoded: Default::default(),
                conformed: None,
            };
            src.hash = rehash(&src, &[]);
            extra.push(src);
        };
        if let Some(index) = &done.index {
            push(
                format!("{base}#index"),
                Kind::Texture,
                index.png(),
                Some(pixel_import),
            );
            what.push(format!("index texture {base}#index"));
        }
        if let Some((pal, None)) = &done.palette
            && done.index.is_some()
        {
            let hex: String = pal
                .rgb
                .iter()
                .map(|c| format!("{:02x}{:02x}{:02x}\n", c[0], c[1], c[2]))
                .collect();
            push(
                format!("{base}#palette"),
                Kind::Palette,
                hex.into_bytes(),
                None,
            );
            what.push(format!("its own palette {base}#palette"));
        }
        if let Some((rot, rot_index, (sx, sy))) = &done.rotations {
            push(
                format!("{base}#rotations"),
                Kind::Texture,
                rot.png(),
                Some(pixel_import),
            );
            let mut w = format!(
                "{} rotations by cleanEdge in {sx}x{sy} cells: {base}#rotations",
                art.rotation_steps
            );
            if let Some(ri) = rot_index {
                push(
                    format!("{base}#rotations-index"),
                    Kind::Texture,
                    ri.png(),
                    Some(pixel_import),
                );
                w.push_str(&format!(" (index {base}#rotations-index)"));
            }
            what.push(w);
        }
        let s = &mut sources[i];
        if changed {
            if done.pixel && kind != crate::format::kind::PIXEL {
                what.push("cooks as pixel art (lossless RGBA8, no mips)".into());
                set_pixel_kind(s);
            }
            s.conformed = Some(png);
        }
        s.hash = rehash(
            s,
            &[
                &art.hash.to_le_bytes(),
                format!("{role:?} {size:?}").as_bytes(),
            ],
        );
        entries.push(ConformEntry {
            source: s.rel.clone(),
            action: if changed {
                ConformAction::Changed
            } else {
                ConformAction::Unchanged
            },
            what: if changed {
                what.join("; ")
            } else {
                "meets the contract as it is".into()
            },
        });
    }
    for (i, text, png, look_hash) in sheet_texts {
        let s = &mut sources[i];
        s.bytes = text;
        s.conformed = Some(png);
        s.hash = rehash(s, &[&look_hash.to_le_bytes()]);
    }
    sources.extend(extra);
    entries.sort_by(|a, b| a.source.cmp(&b.source));
    Ok(entries)
}

/// A sheet texture's role: its cells when every sheet that names it agrees
/// on them; a tile set's tiles; else the whole image as one fixed layout.
fn sheet_role(s: &Source, used: &[Use]) -> Option<Role> {
    let size = Rgba::decode(&s.bytes).ok().map(|i| (i.w, i.h))?;
    let mut cells: Option<(Vec<Rect>, Option<Grid>, bool)> = None;
    for u in used {
        let these = match &u.kind {
            UseKind::Flipbook(sheet) => {
                let mut v: Vec<Rect> = Vec::new();
                for f in &sheet.frames {
                    if !v.contains(&f.rect) {
                        v.push(f.rect);
                    }
                }
                let g = grid_of(&v, size);
                (v, g, false)
            }
            UseKind::Tileset(t) => {
                let g = Grid {
                    cols: t.columns,
                    rows: t.rows,
                    w: t.tile_width,
                    h: t.tile_height,
                    mx: t.margin,
                    my: t.margin,
                    sx: t.spacing,
                    sy: t.spacing,
                    right: size.0.saturating_sub(
                        t.margin
                            + t.columns * t.tile_width
                            + t.columns.saturating_sub(1) * t.spacing,
                    ),
                    bottom: size.1.saturating_sub(
                        t.margin + t.rows * t.tile_height + t.rows.saturating_sub(1) * t.spacing,
                    ),
                };
                let v = (0..t.rows)
                    .flat_map(|r| {
                        (0..t.columns).map(move |c| {
                            [
                                t.margin + c * (t.tile_width + t.spacing),
                                t.margin + r * (t.tile_height + t.spacing),
                                t.tile_width,
                                t.tile_height,
                            ]
                        })
                    })
                    .collect();
                (v, Some(g), true)
            }
            UseKind::Fixed(_) => continue,
        };
        match &cells {
            None => cells = Some(these),
            Some(c) if c.0 == these.0 => {}
            Some(_) => {
                return Some(Role::Fixed(
                    "sheets that cut it into different cells".into(),
                ));
            }
        }
    }
    let (cells, grid, tiles) = cells?;
    if cells.is_empty()
        || cells
            .iter()
            .any(|c| c[2] == 0 || c[3] == 0 || c[0] + c[2] > size.0 || c[1] + c[3] > size.1)
    {
        return Some(Role::Fixed("a sheet whose cells leave the image".into()));
    }
    Some(Role::Sheet { cells, grid, tiles })
}

/// The import settings of `s` with `texture { kind pixel }`.
fn set_pixel_kind(s: &mut Source) {
    let (text, mut doc) = s
        .settings
        .take()
        .unwrap_or_else(|| (String::new(), Document::new("import", 1)));
    let kind = og_core::ogt::Entry::new("kind").with(Value::Word("pixel".into()));
    match doc.body.entries.iter_mut().find(|e| e.key == "texture") {
        Some(e) => {
            let b = e.block.get_or_insert_with(og_core::ogt::Block::new);
            b.entries.retain(|e| e.key != "kind");
            b.entries.push(kind);
        }
        None => doc.body.push(
            og_core::ogt::Entry::new("texture").with_block(og_core::ogt::Block::new().with(kind)),
        ),
    }
    s.settings = Some((text, doc));
}

/// A source's hash as the cook takes it (its id, bytes, settings), with
/// `more` after it.
fn rehash(s: &Source, more: &[&[u8]]) -> u64 {
    let mut h = StableHasher::new();
    h.write_str(&s.rel);
    h.write(&s.bytes);
    if let Some((text, _)) = &s.settings {
        h.write_str(text);
    }
    if let Some(c) = &s.conformed {
        h.write(c);
    }
    for m in more {
        h.write(m);
    }
    h.finish()
}

/// Conforms one image in its role: the steps of the module's doc.
fn run(art: &Art, src: &Rgba, role: &Role, size: Option<u32>) -> Done {
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
