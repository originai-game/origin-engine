//! `og style check [project] [--title <commands>] [--play <commands>]
//! [--results <commands>] [--size WxH] [--no-frames] [--critic] [--json]`:
//! the project's style measured against its look's art contract
//! (docs/art.md §2.5).
//!
//! The project is cooked first, then every cooked asset is measured from
//! `cooked/` itself (never from the cook's notes, which an up-to-date cook
//! does not carry), and each number is compared with the contract of the
//! look project.ogt names (`look "looks/<id>.ogt"`):
//!
//! - **sprites** (textures of a `profile "2d"` project, or cooked as pixel
//!   art): distinct colours against `ArtPalette.maxColors`, the palette hit
//!   rate (the share of opaque texels within ΔE00 2 of a colour of
//!   `ArtPalette.colors`; judged when `strict`), the pixel grid (texels per
//!   logical pixel, the same across the project's sprites, and no half
//!   pixels: a boundary between two colours off the grid; judged when
//!   `ArtPixels` sets one) and the outline width found (the rings of one
//!   line colour round the silhouette; judged against `ArtLine.width`);
//! - **textures** of 3D surfaces: colours and palette as above,
//!   `ArtTexels.maxSize`, `filter` and `mips`;
//! - **models**: the triangles drawn (each primitive times the nodes that
//!   draw it) against `ArtShape.propFaces` (`characterFaces` for a model
//!   with a skin), the flat-normal share (corners on their face's normal;
//!   judged when `normals flat`), the texel density (texels per metre over
//!   the model's surface, area-weighted, against `ArtTexels.density` within
//!   a factor of two) and, where untextured, the colours of its faces.
//!
//! Every opt-out (`conform off`, project.ogt's `conform { off ... }`, and
//! what the cook leaves alone by kind) is listed: an opted-out asset is
//! still measured and judged, since it is still the game's art.
//!
//! Frames: the title, play and results screens are captured by `og run
//! --headless --commands` (project.ogt's `screens { title "<commands>" ...
//! }`, or `--title`, `--play`, `--results`; a script without a
//! `screenshot` gets one at its last step), and each is measured: colour
//! count, palette hit rate, edge statistics (edge density, the share of
//! hard one-pixel steps, mean contrast) and its pixel grid. `--critic`
//! prints only the catalogue entry's three checks as written and the
//! frames' paths, for a blind critic: never the style's name, slug or the
//! look's id. `--json` prints one JSON document.
//!
//! A project whose project.ogt names no look has no contract: it says so
//! and exits 0. Otherwise the exit status is 0 when every asset meets the
//! contract, 3 when one does not (the check says what), 1 when the cook or
//! a capture failed. Numbers cannot judge a style (og-taste §11): `og
//! build --publish` prints the summary as warnings and never refuses a
//! build, and `og doctor --project` has one line of it.

use crate::compare::{delta_e00, lab};
use og_cook::format::{codec, kind};
use og_cook::look::LookFile;
use og_cook::runtime::{Bundle, CookedModel, MANIFEST};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og style check [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH] [--no-frames] [--critic] [--json]";

/// The catalogue of styles (the knowledge package's), for `--critic`'s checks.
const CATALOGUE: &str = include_str!("../../../packages/knowledge/looks/catalogue.json");

/// The screens a style check captures, in order.
pub const SCREENS: [&str; 3] = ["title", "play", "results"];

/// A texel or pixel on a palette colour: within this ΔE00 of one.
pub const PALETTE_DELTA_E: f64 = 2.0;

/// A period k holds when at most this share of the positions off its grid
/// carry a colour boundary: art drawn one texel per pixel has boundaries at
/// nearly every position, art drawn at k texels per pixel only at its half
/// pixels.
const GRID_OFF: f64 = 0.25;

/// The largest texels-per-logical-pixel the grid detector tries.
const GRID_MAX: u32 = 8;

/// A corner is flat when its normal is within 1° of its face's.
const FLAT_COS: f32 = 0.999_848;

/// `normals flat` holds when this share of the corners is flat.
const FLAT_SHARE: f64 = 0.99;

// ── the contract ────────────────────────────────────────────────────────────

/// The art contract of the project's look, as the check reads it from the
/// cooked look (flattened by the cook) and the palette it names.
#[derive(Clone, Debug, Default)]
pub struct Contract {
    /// The look's asset id (project.ogt's `look`).
    pub look: String,
    /// The look's chain of bases, nearest first (a preset's slug last).
    pub extends: Vec<String>,
    /// The art resources the look sets.
    pub sets: Vec<String>,
    /// `ArtPalette.colors`: its id and sRGB colours.
    pub palette: Option<(String, Vec<[u8; 3]>)>,
    pub strict: bool,
    pub max_colors: u32,
    /// `ArtPixels` sets a grid (`unit`, `logicalHeight` or `spriteSizes`).
    pub pixels: bool,
    /// `ArtLine.width`.
    pub line: f64,
    /// `ArtLine.color` in sRGB (the IDL's default when the look sets none).
    pub line_rgb: [u8; 3],
    pub prop_faces: u32,
    pub character_faces: u32,
    /// `ArtShape.normals`: `any`, `smooth`, `flat` or `angle`.
    pub normals: String,
    pub density: f64,
    pub max_size: u32,
    /// `ArtTexels.filter`: `any`, `nearest` or `linear`.
    pub filter: String,
    /// `ArtTexels.mips`: `any`, `full` or `none`.
    pub mips: String,
}

fn field<'a>(look: &'a LookFile, resource: &str, name: &str) -> Option<&'a og_core::ogt::Value> {
    look.blocks
        .iter()
        .find(|b| b.resource == resource)?
        .get(name)?
        .first()
}

fn num(look: &LookFile, resource: &str, name: &str) -> f64 {
    field(look, resource, name)
        .and_then(|v| v.as_number())
        .map_or(0.0, |n| n.as_f64())
}

fn word(look: &LookFile, resource: &str, name: &str, zero: &str) -> String {
    field(look, resource, name)
        .and_then(|v| v.as_word())
        .unwrap_or(zero)
        .to_string()
}

/// `ArtLine.color` (linear numbers in the cooked look) in sRGB.
fn line_colour(look: &LookFile) -> [u8; 3] {
    let v = look
        .blocks
        .iter()
        .find(|b| b.resource == "ArtLine")
        .and_then(|b| b.get("color"));
    match v {
        Some([r, g, b]) => {
            let c = |x: &og_core::ogt::Value| x.as_number().map_or(0.0, |n| n.as_f32());
            [srgb8(c(r)), srgb8(c(g)), srgb8(c(b))]
        }
        _ => LINE_RGB,
    }
}

/// Reads a palette asset's sRGB colours by id.
type PaletteOf<'a> = dyn Fn(&str) -> Result<Vec<[u8; 3]>, String> + 'a;

impl Contract {
    /// The contract of `look` (a cooked, flattened look), its palette read
    /// through `palette` (an id to its colours).
    pub fn of(id: &str, look: &LookFile, palette: &PaletteOf) -> Result<Contract, String> {
        let colors = field(look, "ArtPalette", "colors").and_then(|v| v.as_str());
        let palette = match colors {
            Some(name) if !name.is_empty() => {
                let id = og_cook::cook::look::builtin(name).map_or_else(
                    || og_cook::runtime::asset_id(name).to_string(),
                    |(id, _)| id.to_string(),
                );
                Some((id.clone(), palette(&id)?))
            }
            _ => None,
        };
        let sizes = look
            .blocks
            .iter()
            .find(|b| b.resource == "ArtPixels")
            .and_then(|b| b.get("spriteSizes"))
            .is_some_and(|v| !v.is_empty());
        Ok(Contract {
            look: id.to_string(),
            extends: look.extends.clone(),
            sets: look
                .resources()
                .into_iter()
                .filter(|r| og_cook::look::ART_RESOURCES.contains(r))
                .map(str::to_string)
                .collect(),
            palette,
            strict: field(look, "ArtPalette", "strict").and_then(|v| v.as_bool()) == Some(true),
            max_colors: num(look, "ArtPalette", "maxColors") as u32,
            pixels: num(look, "ArtPixels", "unit") > 0.0
                || num(look, "ArtPixels", "logicalHeight") > 0.0
                || sizes,
            line: num(look, "ArtLine", "width"),
            line_rgb: line_colour(look),
            prop_faces: num(look, "ArtShape", "propFaces") as u32,
            character_faces: num(look, "ArtShape", "characterFaces") as u32,
            normals: word(look, "ArtShape", "normals", "any"),
            density: num(look, "ArtTexels", "density"),
            max_size: num(look, "ArtTexels", "maxSize") as u32,
            filter: word(look, "ArtTexels", "filter", "any"),
            mips: word(look, "ArtTexels", "mips", "any"),
        })
    }

    /// The catalogue slug the look builds on: the first of its chain (or
    /// its own file's name) that is a catalogue style.
    pub fn slug(&self) -> Option<String> {
        let own = Path::new(&self.look)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string());
        let styles = catalogue_styles();
        own.into_iter()
            .chain(self.extends.iter().cloned())
            .find(|s| styles.iter().any(|e| e["slug"] == s.as_str()))
    }
}

fn catalogue_styles() -> Vec<Value> {
    serde_json::from_str::<Value>(CATALOGUE)
        .ok()
        .and_then(|c| c["styles"].as_array().cloned())
        .unwrap_or_default()
}

/// The catalogue's three checks for `slug`, as written.
pub fn catalogue_checks(slug: &str) -> Vec<String> {
    catalogue_styles()
        .iter()
        .find(|e| e["slug"] == slug)
        .and_then(|e| e["checks"].as_array())
        .map(|c| {
            c.iter()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// project.ogt's body, parsed.
fn project_body(root: &Path) -> Result<og_core::ogt::Block, String> {
    let path = root.join("project.ogt");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    og_core::ogt::parse(&text)
        .map(|d| d.body)
        .map_err(|e| format!("{}: {}", path.display(), e.reason()))
}

/// The contract of the project at `root`, read from its cooked look;
/// `None` when project.ogt names no look.
pub fn contract(root: &Path) -> Result<Option<Contract>, String> {
    let body = project_body(root)?;
    let Some(id) = og_cook::look::project_look(&body).map_err(|e| format!("project.ogt:{e}"))?
    else {
        return Ok(None);
    };
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let bytes = bundle.read(&id).map_err(|e| e.to_string())?;
    let look = bundle.look(&id, &bytes).map_err(|e| e.to_string())?;
    let palette = |pid: &str| -> Result<Vec<[u8; 3]>, String> {
        let bytes = bundle
            .read(pid)
            .map_err(|e| format!("the look's ArtPalette.colors: {e}"))?;
        bundle.palette(pid, &bytes).map_err(|e| e.to_string())
    };
    Contract::of(&id, &look, &palette).map(Some)
}

// ── measures ────────────────────────────────────────────────────────────────

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
fn axis_grid(counts: &[u64]) -> (u32, Vec<u32>) {
    let len = counts.len() as u32;
    let edge = |i: u32| i > 0 && counts[i as usize] > 0;
    for k in (2..=GRID_MAX.min(len / 2)).rev() {
        for o in 0..k {
            let on = |i: u32| (i + k - o % k).is_multiple_of(k);
            let positions = (1..len).filter(|&i| !on(i)).count();
            let off: Vec<u32> = (1..len).filter(|&i| !on(i) && edge(i)).collect();
            let hits: Vec<u32> = (1..len).filter(|&i| on(i) && edge(i)).collect();
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

fn srgb8(linear: f32) -> u8 {
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
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Sprite => "sprite",
            Role::Texture => "texture",
            Role::Model => "model",
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

fn pct(x: f64) -> String {
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
const LINE_RGB: [u8; 3] = [39, 39, 48];

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
pub fn judge_model(c: &Contract, id: &str, m: &ModelMeasures, character: bool) -> Asset {
    let mut a = Asset {
        id: id.to_string(),
        role: Role::Model,
        off: None,
        measures: json!({ "faces": m.faces, "character": character }),
        checks: Vec::new(),
        notes: Vec::new(),
    };
    let (budget, name) = if character {
        (c.character_faces, "ArtShape.characterFaces")
    } else {
        (c.prop_faces, "ArtShape.propFaces")
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

// ── the project ─────────────────────────────────────────────────────────────

/// The measured project: its contract, its assets, its opt-outs and the
/// spread of texel density across its models.
#[derive(Clone, Debug)]
pub struct Measured {
    pub contract: Contract,
    pub assets: Vec<Asset>,
    /// Every asset kept as made, with why: (asset id, reason).
    pub opt_outs: Vec<(String, String)>,
    /// The project's texels per logical pixel (its sprites' most common).
    pub grid: Option<u32>,
    /// The models' median densities: the highest over the lowest.
    pub density_spread: Option<f64>,
}

impl Measured {
    pub fn off_contract(&self) -> Vec<&Asset> {
        self.assets.iter().filter(|a| !a.ok()).collect()
    }
}

/// The opt-outs the cook reported (its `conform` entries with action
/// `off`), as (asset id, why), from a report or cook-report.json.
pub fn opt_outs_of(report: &og_cook::Report) -> Vec<(String, String)> {
    report
        .conform
        .iter()
        .filter(|e| e.action == og_cook::cook::ConformAction::Off)
        .map(|e| (e.source.clone(), e.what.clone()))
        .collect()
}

/// The textures whose layout the cook keeps: those a 2D rig or a UI style
/// names (true for a UI style's), read from their sources (the game's under assets/, a built-in
/// style's from the engine).
fn fixed_layouts(root: &Path, bundle: &Bundle) -> Vec<(String, bool)> {
    let assets = &bundle.manifest().assets;
    let mut out = Vec::new();
    for a in assets
        .iter()
        .filter(|a| a.kind == "rig2d" || a.kind == "style")
    {
        let text = match og_cook::cook::look::STYLES
            .iter()
            .find(|(id, _)| *id == a.source)
        {
            Some((_, bytes)) => String::from_utf8_lossy(bytes).to_string(),
            None => {
                std::fs::read_to_string(root.join("assets").join(&a.source)).unwrap_or_default()
            }
        };
        let dir = a.source.rsplit_once('/').map_or("", |(d, _)| d);
        for t in assets.iter().filter(|t| t.kind == "texture") {
            let (tdir, name) = t.source.rsplit_once('/').unwrap_or(("", &t.source));
            let quoted = |n: &str| text.contains(&format!("\"{n}\""));
            if quoted(&t.source) || (tdir == dir && quoted(name)) {
                out.push((t.source.clone(), a.kind == "style"));
            }
        }
    }
    out
}

/// The tiles of each tile set's texture, by texture id, from the cooked
/// tile sets (their layout after conformance).
fn tile_cells(bundle: &Bundle) -> HashMap<String, Vec<[u32; 4]>> {
    let mut out: HashMap<String, Vec<[u32; 4]>> = HashMap::new();
    for o in bundle.manifest().outputs().filter(|o| o.ty == "og.Tileset") {
        let Ok(t) = bundle.load_tileset(&o.id) else {
            continue;
        };
        let cells = out
            .entry(og_cook::runtime::asset_id(&t.texture).to_string())
            .or_default();
        for r in 0..t.rows {
            for c in 0..t.columns {
                cells.push([
                    t.margin + c * (t.tile_width + t.spacing),
                    t.margin + r * (t.tile_height + t.spacing),
                    t.tile_width,
                    t.tile_height,
                ]);
            }
        }
    }
    out
}

/// Measures every cooked asset of the project at `root` against `c`.
/// `opt_outs` are the cook's.
pub fn measure(
    root: &Path,
    c: Contract,
    opt_outs: Vec<(String, String)>,
) -> Result<Measured, String> {
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let profile_2d = project_body(root)
        .ok()
        .and_then(|b| {
            b.get("profile")
                .and_then(|e| e.string(0).ok().map(str::to_string))
        })
        .as_deref()
        == Some("2d");
    let fixed = fixed_layouts(root, &bundle);
    let tiles = tile_cells(&bundle);
    let mut assets = Vec::new();
    let mut medians = Vec::new();
    let off_of = |id: &str| -> Option<String> {
        let source = id.split('#').next().unwrap_or(id);
        opt_outs
            .iter()
            .find(|(s, _)| s == source)
            .map(|(_, w)| w.clone())
    };
    for asset in &bundle.manifest().assets {
        // Index textures, rotations and palettes the cook derives are
        // checked through the asset they come from.
        if asset.source.contains('#') {
            continue;
        }
        let model = asset.outputs.iter().find(|o| o.ty == "og.Model");
        for o in &asset.outputs {
            if o.ty != "og.Texture" {
                continue;
            }
            let t = bundle.load_texture(&o.id).map_err(|e| e.to_string())?;
            if matches!(t.meta.kind, kind::NORMAL | kind::LINEAR | kind::MSDF) {
                continue;
            }
            let data = t.to_texture_data().map_err(|e| e.to_string())?;
            let img = Image::from_rgba(data.width, data.height, &data.rgba);
            let lossy = t.meta.codec != codec::RGBA8;
            let sprite = model.is_none() && (profile_2d || t.is_pixel_art());
            let mut a = if sprite {
                let cells = tiles.get(&o.id).map_or(&[][..], Vec::as_slice);
                let layout = fixed.iter().find(|(f, _)| f == &o.id);
                let mut a = judge_sprite(&c, &o.id, &img, lossy, layout.is_some(), cells);
                if layout.is_some_and(|(_, ui)| *ui) {
                    a.measures["ui"] = json!(true);
                }
                a
            } else {
                judge_texture(&c, &o.id, &img, t.is_pixel_art(), t.meta.levels, lossy)
            };
            a.off = off_of(&o.id);
            assets.push(a);
        }
        let Some(o) = model else { continue };
        let m = bundle.load_model(&o.id).map_err(|e| e.to_string())?;
        let size = |id: &str| {
            let t = bundle.load_texture(id).ok()?;
            Some((t.meta.width, t.meta.height))
        };
        let measures = measure_model(&m, &size);
        let character = asset
            .outputs
            .iter()
            .filter(|x| x.ty == "og.Animation")
            .any(|x| {
                bundle
                    .load_animation(&x.id)
                    .is_ok_and(|a| !a.skin_streams.is_empty())
            });
        if let Some([median, ..]) = measures.density {
            medians.push(median);
        }
        let mut a = judge_model(&c, &o.id, &measures, character);
        a.off = off_of(&o.id);
        assets.push(a);
    }
    let sprites: Vec<(u32, u32)> = assets
        .iter()
        .filter(|a| a.role == Role::Sprite && a.measures["ui"] != true)
        .filter_map(|a| {
            let k = a.measures["grid"]["texelsPerPixel"].as_array()?;
            Some((k.first()?.as_u64()? as u32, k.get(1)?.as_u64()? as u32))
        })
        .collect();
    let grid = project_grid(&sprites);
    judge_grids(&c, &mut assets, grid);
    let density_spread = (medians.len() > 1).then(|| {
        let max = medians.iter().copied().fold(f64::MIN, f64::max);
        let min = medians.iter().copied().fold(f64::MAX, f64::min);
        if min > 0.0 { max / min } else { f64::INFINITY }
    });
    Ok(Measured {
        contract: c,
        assets,
        opt_outs,
        grid,
        density_spread,
    })
}

/// A captured frame and its numbers.
#[derive(Clone, Debug)]
pub struct Frame {
    pub screen: String,
    pub path: PathBuf,
    pub colours: usize,
    /// The fewest colours that cover 95 % of the pixels.
    pub colours95: usize,
    pub palette_hit: Option<f64>,
    pub edges: Edges,
    pub grid: u32,
}

/// Measures one frame (a PNG) against the contract's palette.
pub fn measure_frame(
    screen: &str,
    path: &Path,
    palette: Option<&[[u8; 3]]>,
) -> Result<Frame, String> {
    let img = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8();
    let mut img = Image::from_rgba(img.width(), img.height(), img.as_raw());
    for p in &mut img.px {
        p[3] = 255;
    }
    let counts = colours(&img);
    let mut by: Vec<u64> = counts.values().copied().collect();
    by.sort_unstable_by(|a, b| b.cmp(a));
    let total: u64 = by.iter().sum();
    let mut acc = 0;
    let colours95 = by
        .iter()
        .take_while(|&&n| {
            let before = acc;
            acc += n;
            (before as f64) < 0.95 * total as f64
        })
        .count();
    let g = grid(&img);
    Ok(Frame {
        screen: screen.to_string(),
        path: path.to_path_buf(),
        colours: counts.len(),
        colours95,
        palette_hit: palette.and_then(|p| palette_hit(&counts, p)),
        edges: edges(&img),
        grid: if g.kx == g.ky { g.kx } else { g.kx.min(g.ky) },
    })
}

/// The scripts that reach each screen: project.ogt's `screens { title
/// "<commands>" play "..." results "..." }`, each replaced by its flag.
pub fn screens(
    root: &Path,
    flags: &BTreeMap<String, String>,
) -> Result<Vec<(String, String)>, String> {
    let body = project_body(root)?;
    let mut out = Vec::new();
    let block = body.get("screens").and_then(|e| e.block.clone());
    if let Some(b) = &block {
        for e in &b.entries {
            if !SCREENS.contains(&e.key.as_str()) {
                return Err(format!(
                    "project.ogt:{}: screens names {}; `{}` is not one",
                    e.line,
                    SCREENS.join(", "),
                    e.key
                ));
            }
        }
    }
    for s in SCREENS {
        let script = flags.get(s).cloned().or_else(|| {
            block
                .as_ref()
                .and_then(|b| b.get(s))
                .and_then(|e| e.string(0).ok().map(str::to_string))
        });
        if let Some(script) = script {
            out.push((s.to_string(), with_screenshot(&script)));
        }
    }
    Ok(out)
}

/// `script` with a `screenshot` at its last step's time when it takes none.
pub fn with_screenshot(script: &str) -> String {
    let steps: Vec<&str> = script
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if steps
        .iter()
        .any(|s| s.split(':').nth(1) == Some("screenshot"))
    {
        return steps.join(";");
    }
    let at = steps
        .last()
        .and_then(|s| s.split(':').next())
        .unwrap_or("0")
        .to_string();
    let mut all: Vec<String> = steps.iter().map(|s| s.to_string()).collect();
    all.push(format!("{at}:screenshot"));
    all.join(";")
}

/// Captures one screen with `og run --headless --commands`: its PNGs.
fn capture(
    root: &Path,
    screen: &str,
    script: &str,
    size: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let dir = root.join("og-out/style").join(screen);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("run")
        .arg(root)
        .args(["--headless", "--no-check", "--commands", script, "--out"])
        .arg(&dir);
    if let Some(s) = size {
        cmd.args(["--size", s]);
    }
    let out = cmd.output().map_err(|e| format!("og run: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let tail: Vec<&str> = err
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        return Err(format!(
            "the {screen} screen's run (`{script}`) failed:\n  {}",
            tail.join("\n  ")
        ));
    }
    let mut pngs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .collect();
    pngs.sort();
    if pngs.is_empty() {
        return Err(format!(
            "the {screen} screen's run (`{script}`) wrote no frame: no renderer in this environment, or its screenshot step never ran"
        ));
    }
    Ok(pngs)
}

// ── output ──────────────────────────────────────────────────────────────────

fn asset_json(a: &Asset) -> Value {
    json!({
        "asset": a.id,
        "role": a.role.name(),
        "optedOut": a.off,
        "measures": a.measures,
        "checks": a.checks.iter().map(|c| json!({
            "name": c.name, "value": c.value, "limit": c.limit, "ok": c.ok,
        })).collect::<Vec<_>>(),
        "notes": a.notes,
        "ok": a.ok(),
    })
}

fn frame_json(f: &Frame) -> Value {
    json!({
        "screen": f.screen,
        "path": f.path.display().to_string(),
        "colours": f.colours,
        "colours95": f.colours95,
        "paletteHit": f.palette_hit,
        "edges": { "density": f.edges.density, "hard": f.edges.hard, "contrast": f.edges.contrast },
        "texelsPerPixel": f.grid,
    })
}

/// The whole check as one JSON document.
pub fn to_json(m: &Measured, frames: &[Frame], frame_errors: &[String]) -> Value {
    let c = &m.contract;
    json!({
        "contract": {
            "look": c.look,
            "sets": c.sets,
            "palette": c.palette.as_ref().map(|(id, p)| json!({ "id": id, "colours": p.len() })),
        },
        "assets": m.assets.iter().map(asset_json).collect::<Vec<_>>(),
        "optOuts": m.opt_outs.iter().map(|(a, w)| json!({ "asset": a, "why": w })).collect::<Vec<_>>(),
        "texelsPerPixel": m.grid,
        "densitySpread": m.density_spread,
        "frames": frames.iter().map(frame_json).collect::<Vec<_>>(),
        "frameErrors": frame_errors,
        "summary": {
            "assets": m.assets.len(),
            "offContract": m.off_contract().len(),
            "optOuts": m.opt_outs.len(),
        },
    })
}

/// One line per asset, its numbers and each check (✓ or ✗).
fn asset_line(a: &Asset) -> String {
    let mut parts = Vec::new();
    let m = &a.measures;
    if let Some(n) = m["faces"].as_u64() {
        parts.push(format!("faces {n}"));
    }
    if let Some(f) = m["flatShare"].as_f64() {
        parts.push(format!("flat {}", pct(f)));
    }
    if let Some(d) = m["density"]["median"].as_f64() {
        parts.push(format!("density {d:.1}/m"));
    }
    if let Some(n) = m["colours"].as_u64() {
        parts.push(format!("colours {n}"));
    }
    if let Some(h) = m["paletteHit"].as_f64() {
        parts.push(format!("palette {}", pct(h)));
    }
    if let Some(k) = m["grid"]["texelsPerPixel"].as_array() {
        let k: Vec<String> = k.iter().map(Value::to_string).collect();
        parts.push(format!("grid {}", k.join("x")));
    }
    if let Some(l) = m["line"].as_u64() {
        parts.push(format!("line {l}"));
    }
    let mut s = format!(
        "  {} {:<7} {:<44} {}",
        if a.ok() { "ok " } else { "off" },
        a.role.name(),
        a.id,
        parts.join("  ")
    );
    if a.off.is_some() {
        s.push_str("  (opted out)");
    }
    for c in &a.checks {
        s.push_str(&format!(
            "\n        {} {}: {} against {}",
            if c.ok { "✓" } else { "✗" },
            c.name,
            c.value,
            c.limit
        ));
    }
    for n in &a.notes {
        s.push_str(&format!("\n        · {n}"));
    }
    s
}

/// The check as lines for a reader.
pub fn report(m: &Measured, frames: &[Frame], frame_errors: &[String]) -> String {
    let c = &m.contract;
    let mut s = format!(
        "contract: {} sets {}\n",
        c.look,
        if c.sets.is_empty() {
            "no art resource: nothing to hold the assets to".into()
        } else {
            c.sets.join(", ")
        }
    );
    if let Some((id, p)) = &c.palette {
        s.push_str(&format!("palette: {id} ({} colours)\n", p.len()));
    }
    let off = m.off_contract().len();
    s.push_str(&format!(
        "assets: {} measured, {} off the contract{}\n",
        m.assets.len(),
        off,
        m.grid
            .map(|k| format!("; sprites at {k} texel(s) per logical pixel"))
            .unwrap_or_default()
    ));
    for a in &m.assets {
        s.push_str(&asset_line(a));
        s.push('\n');
    }
    if let Some(sp) = m.density_spread {
        s.push_str(&format!("texel density spread across models: x{sp:.2}\n"));
    }
    s.push_str(&format!("opt-outs: {}\n", m.opt_outs.len()));
    for (a, w) in &m.opt_outs {
        s.push_str(&format!("  {a}: {w}\n"));
    }
    if frames.is_empty() && frame_errors.is_empty() {
        s.push_str("frames: none: name the screens in project.ogt (screens { title \"<commands>\" play \"...\" results \"...\" }) or pass --title, --play, --results\n");
    } else {
        s.push_str("frames:\n");
    }
    for f in frames {
        s.push_str(&format!(
            "  {:<7} {}\n          colours {} (95% in {}){}  edges {} of pixels, {} hard, contrast {:.2}  grid {} px\n",
            f.screen,
            f.path.display(),
            f.colours,
            f.colours95,
            f.palette_hit.map(|h| format!("  palette {}", pct(h))).unwrap_or_default(),
            pct(f.edges.density),
            pct(f.edges.hard),
            f.edges.contrast,
            f.grid
        ));
    }
    for e in frame_errors {
        s.push_str(&format!("  failed: {e}\n"));
    }
    s
}

/// `--critic`: the catalogue's checks as written and the frames, nothing
/// that names the style.
pub fn critic(checks: &[String], frames: &[Frame], json: bool) -> String {
    let prompt = "Look at the frames below. For each check, answer yes or no, with one sentence on what in the frames shows it.";
    if json {
        return json!({
            "prompt": prompt,
            "checks": checks,
            "frames": frames.iter().map(|f| f.path.display().to_string()).collect::<Vec<_>>(),
        })
        .to_string();
    }
    let mut s = format!("{prompt}\n\nchecks:\n");
    if checks.is_empty() {
        s.push_str("  (none: the look builds on no catalogue style)\n");
    }
    for (i, c) in checks.iter().enumerate() {
        s.push_str(&format!("  {}. {c}\n", i + 1));
    }
    s.push_str("\nframes:\n");
    if frames.is_empty() {
        s.push_str("  (none captured)\n");
    }
    for f in frames {
        s.push_str(&format!("  {}\n", f.path.display()));
    }
    s
}

/// The summary `og build --publish` prints as warnings: one line per asset
/// off the contract (at most `max`), and how many more.
pub fn warnings(m: &Measured, max: usize) -> Vec<String> {
    let off = m.off_contract();
    if off.is_empty() {
        return Vec::new();
    }
    let mut out = vec![format!(
        "style: {} of {} asset(s) off the look's art contract ({}); `og style check` says what",
        off.len(),
        m.assets.len(),
        m.contract.look
    )];
    for a in off.iter().take(max) {
        let failed: Vec<String> = a
            .checks
            .iter()
            .filter(|c| !c.ok)
            .map(|c| format!("{} {} against {}", c.name, c.value, c.limit))
            .collect();
        out.push(format!("style: {}: {}", a.id, failed.join("; ")));
    }
    if off.len() > max {
        out.push(format!("style: and {} more", off.len() - max));
    }
    out
}

/// `og build --publish`'s summary of the asset part of the check, after
/// its cook: warnings only, empty when the project has no look or every
/// asset meets the contract.
pub fn publish_warnings(root: &Path, cooked: &og_cook::Report) -> Vec<String> {
    match contract(root).and_then(|c| match c {
        Some(c) => measure(root, c, opt_outs_of(cooked)).map(Some),
        None => Ok(None),
    }) {
        Ok(Some(m)) => warnings(&m, 8),
        Ok(None) => Vec::new(),
        Err(e) => vec![format!("style: the assets could not be measured: {e}")],
    }
}

// ── the command ─────────────────────────────────────────────────────────────

struct Args {
    project: String,
    flags: BTreeMap<String, String>,
    size: Option<String>,
    frames: bool,
    critic: bool,
    json: bool,
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let mut a = Args {
        project: ".".into(),
        flags: BTreeMap::new(),
        size: None,
        frames: true,
        critic: false,
        json: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => a.json = true,
            "--critic" => a.critic = true,
            "--no-frames" => a.frames = false,
            f @ ("--title" | "--play" | "--results" | "--size") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{f} needs a value. {USAGE}"));
                };
                if f == "--size" {
                    a.size = Some(v.to_string());
                } else {
                    a.flags.insert(f[2..].to_string(), v.to_string());
                }
                i += 1;
            }
            f if f.starts_with('-') => return Err(format!("unknown option `{f}`. {USAGE}")),
            p => a.project = p.to_string(),
        }
        i += 1;
    }
    Ok(a)
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::from(2);
        }
    };
    let root = PathBuf::from(&a.project);
    if !root.join("project.ogt").is_file() {
        eprintln!(
            "og style check: {} is not a project (no project.ogt)",
            root.display()
        );
        return ExitCode::FAILURE;
    }
    let root = std::path::absolute(&root).unwrap_or(root);
    match project_body(&root)
        .and_then(|b| og_cook::look::project_look(&b).map_err(|e| format!("project.ogt:{e}")))
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            if a.json {
                println!("{}", json!({ "contract": null, "message": NO_CONTRACT }));
            } else {
                println!("og style check: {NO_CONTRACT}");
            }
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    }
    let cooked = match crate::cook::cook(&root, 0, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    };
    let measured = match contract(&root)
        .and_then(|c| c.ok_or_else(|| NO_CONTRACT.to_string()))
        .and_then(|c| measure(&root, c, opt_outs_of(&cooked)))
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut frames = Vec::new();
    let mut frame_errors = Vec::new();
    if a.frames {
        let scripts = match screens(&root, &a.flags) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("og style check: {e}");
                return ExitCode::FAILURE;
            }
        };
        let palette = measured
            .contract
            .palette
            .as_ref()
            .map(|(_, p)| p.as_slice());
        for (screen, script) in scripts {
            match capture(&root, &screen, &script, a.size.as_deref()) {
                Ok(pngs) => {
                    for p in pngs {
                        match measure_frame(&screen, &p, palette) {
                            Ok(f) => frames.push(f),
                            Err(e) => frame_errors.push(e),
                        }
                    }
                }
                Err(e) => frame_errors.push(e),
            }
        }
    }
    if a.critic {
        let checks = measured
            .contract
            .slug()
            .map(|s| catalogue_checks(&s))
            .unwrap_or_default();
        print!("{}", critic(&checks, &frames, a.json));
        if a.json {
            println!();
        }
        for e in &frame_errors {
            eprintln!(
                "og style check: a frame could not be captured: {}",
                e.lines().next().unwrap_or("")
            );
        }
        return if frame_errors.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    if a.json {
        println!("{}", to_json(&measured, &frames, &frame_errors));
    } else {
        println!("og style check {}", root.display());
        print!("{}", report(&measured, &frames, &frame_errors));
    }
    if !frame_errors.is_empty() {
        ExitCode::FAILURE
    } else if measured.off_contract().is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

/// What the check says of a project whose project.ogt names no look.
pub const NO_CONTRACT: &str = "project.ogt names no look (`look \"looks/<id>.ogt\"`), so there is no art contract to check against";

/// `og doctor --project`'s line: the asset part of the check, read from
/// `cooked/` as it is (no cook) with the opt-outs of the last cook report.
pub fn doctor_finding(root: &Path, report: Option<&Value>) -> Value {
    let line = |severity: &str, message: String, fix: &str| json!({ "severity": severity, "code": "style", "message": message, "fix": fix });
    let c = match contract(root) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return line(
                "info",
                format!("style: {NO_CONTRACT}"),
                "name the game's look in project.ogt to hold its assets to the style",
            );
        }
        Err(e) => {
            return line(
                "warning",
                format!("style: the art contract could not be read: {e}"),
                "run `og cook` in the project, then `og style check`",
            );
        }
    };
    if !root.join("cooked").join(MANIFEST).is_file() {
        return line(
            "warning",
            "style: nothing cooked to measure".into(),
            "run `og cook`",
        );
    }
    let opt_outs = report
        .and_then(|r| r["conform"].as_array())
        .into_iter()
        .flatten()
        .filter(|e| e["action"] == "off")
        .map(|e| {
            (
                e["source"].as_str().unwrap_or("").to_string(),
                e["what"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();
    match measure(root, c, opt_outs) {
        Ok(m) => {
            let off = m.off_contract();
            let opted = if m.opt_outs.is_empty() {
                String::new()
            } else {
                format!(", {} opted out of conformance", m.opt_outs.len())
            };
            if off.is_empty() {
                line(
                    "info",
                    format!(
                        "style: {} asset(s) meet the art contract of {}{opted}",
                        m.assets.len(),
                        m.contract.look
                    ),
                    "run `og style check` for the frames and the numbers",
                )
            } else {
                let names: Vec<&str> = off.iter().take(5).map(|a| a.id.as_str()).collect();
                line(
                    "warning",
                    format!(
                        "style: {} of {} asset(s) off the art contract of {} ({}{}){opted}",
                        off.len(),
                        m.assets.len(),
                        m.contract.look,
                        names.join(", "),
                        if off.len() > 5 { ", ..." } else { "" }
                    ),
                    "run `og style check` to see each number against the contract",
                )
            }
        }
        Err(e) => line(
            "warning",
            format!("style: the cooked assets could not be measured: {e}"),
            "run `og cook` in the project, then `og style check`",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_cook::runtime::{CookedMesh, CookedNode, CookedPrimitive, CookedScene, Lod};
    use og_render::Vertex;

    const CLEAR: [u8; 4] = [0, 0, 0, 0];

    /// An image from rows of letters, each a colour of `key` ('.' clear).
    fn art(rows: &[&str], key: &[(char, [u8; 3])]) -> Image {
        let h = rows.len() as u32;
        let w = rows[0].len() as u32;
        let mut px = Vec::new();
        for r in rows {
            for ch in r.chars() {
                px.push(match key.iter().find(|(k, _)| *k == ch) {
                    Some((_, c)) => [c[0], c[1], c[2], 255],
                    None => CLEAR,
                });
            }
        }
        Image { w, h, px }
    }

    fn contract() -> Contract {
        Contract {
            look: "looks/main.ogt".into(),
            palette: Some((
                "palettes/two.hex".into(),
                vec![[128, 128, 128], [200, 40, 40]],
            )),
            strict: true,
            pixels: true,
            line: 1.0,
            line_rgb: [0, 0, 0],
            normals: "flat".into(),
            ..Contract::default()
        }
    }

    // The palette's grey 128 and grey 136 differ by ΔE00 2.95 (by hand: L*
    // 53.585 and 56.708, S_L 1.0583 at L̄ 55.15), grey 129 by about 0.95.
    #[test]
    fn an_on_palette_sprite_hits_and_one_three_delta_e_off_misses() {
        let key = [
            ('g', [128, 128, 128]),
            ('r', [200, 40, 40]),
            ('o', [136, 136, 136]),
            ('n', [129, 129, 129]),
        ];
        // 5 grey, 3 red, and a clear texel whose colour is ignored.
        let on = art(&["ggggg", "rrr.."], &key);
        let off = art(&["ooooo", "rrr.."], &key);
        let near = art(&["nnnnn", "rrr.."], &key);
        let c = contract();
        let pal = &c.palette.as_ref().unwrap().1;
        assert_eq!(palette_hit(&colours(&on), pal), Some(1.0));
        assert_eq!(palette_hit(&colours(&off), pal), Some(3.0 / 8.0));
        assert_eq!(palette_hit(&colours(&near), pal), Some(1.0));
        let judged = |img: &Image| {
            judge_sprite(&c, "s.png", img, false, false, &[])
                .checks
                .into_iter()
                .find(|k| k.name == "palette")
                .unwrap()
                .ok
        };
        assert!(judged(&on));
        assert!(!judged(&off));
        // Not strict: the rate is measured, not judged.
        let loose = Contract {
            strict: false,
            ..contract()
        };
        let a = judge_sprite(&loose, "s.png", &off, false, false, &[]);
        assert!(a.checks.iter().all(|k| k.name != "palette"));
        assert_eq!(a.measures["paletteHit"], json!(0.375));
    }

    /// `rows` drawn at `k` texels per letter.
    fn scaled(rows: &[&str], k: usize) -> Vec<String> {
        rows.iter()
            .flat_map(|r| {
                let wide: String = r.chars().flat_map(|c| std::iter::repeat_n(c, k)).collect();
                std::iter::repeat_n(wide, k)
            })
            .collect()
    }

    #[test]
    fn a_two_texel_sprite_with_a_half_pixel_column_is_found() {
        let key = [
            ('a', [10, 10, 10]),
            ('b', [90, 200, 90]),
            ('c', [220, 220, 40]),
        ];
        let logical = ["abcab.cb", "bcabca.a", ".abcabcb", "cab.bcab"];
        let clean: Vec<String> = scaled(&logical, 2);
        let rows: Vec<&str> = clean.iter().map(String::as_str).collect();
        let g = grid(&art(&rows, &key));
        assert_eq!((g.kx, g.ky), (2, 2));
        assert!(g.half_columns.is_empty() && g.half_rows.is_empty(), "{g:?}");
        // Column 13 (odd) takes column 14's colour in every row: the boundary
        // between logical pixels 6 and 7 moves half a pixel left.
        let mut half = clean.clone();
        for r in &mut half {
            let mut chars: Vec<char> = r.chars().collect();
            chars[13] = chars[14];
            *r = chars.into_iter().collect();
        }
        let rows: Vec<&str> = half.iter().map(String::as_str).collect();
        let g = grid(&art(&rows, &key));
        assert_eq!((g.kx, g.ky), (2, 2));
        assert_eq!(g.half_columns, [13]);
        assert!(g.half_rows.is_empty());
        // One texel per pixel art is not taken for a coarser grid.
        let g = grid(&art(&logical, &key));
        assert_eq!((g.kx, g.ky), (1, 1));

        // Judged against the project's grid of 2: the clean one holds, the
        // half pixel does not; against a project of 1, neither.
        let c = contract();
        let mut assets = vec![
            judge_sprite(
                &c,
                "clean.png",
                &art(&clean.iter().map(String::as_str).collect::<Vec<_>>(), &key),
                false,
                false,
                &[],
            ),
            judge_sprite(
                &c,
                "half.png",
                &art(&half.iter().map(String::as_str).collect::<Vec<_>>(), &key),
                false,
                false,
                &[],
            ),
        ];
        let mut ones = assets.clone();
        judge_grids(&c, &mut assets, Some(2));
        let ok = |a: &Asset| a.checks.iter().find(|k| k.name == "grid").unwrap().ok;
        assert!(ok(&assets[0]));
        assert!(!ok(&assets[1]));
        judge_grids(&c, &mut ones, Some(1));
        assert!(!ok(&ones[0]) && !ok(&ones[1]));
    }

    #[test]
    fn the_project_grid_is_the_sprites_most_common() {
        assert_eq!(project_grid(&[(2, 2), (1, 1), (2, 2), (3, 1)]), Some(2));
        assert_eq!(project_grid(&[(1, 1), (2, 2)]), Some(1));
        assert_eq!(project_grid(&[]), None);
    }

    #[test]
    fn the_outline_width_is_the_rings_of_one_dark_colour() {
        let key = [
            ('k', [0, 0, 0]),
            ('f', [230, 120, 60]),
            ('s', [250, 200, 160]),
        ];
        let one = art(
            &["......", ".kkkk.", ".kffk.", ".kfsk.", ".kkkk.", "......"],
            &key,
        );
        assert_eq!(outline_width(&one, Some([0, 0, 0]), &[]), Some(1));
        let two = art(
            &[
                "........", ".kkkkkk.", ".kkkkkk.", ".kkffkk.", ".kkfskk.", ".kkkkkk.", ".kkkkkk.",
                "........",
            ],
            &key,
        );
        assert_eq!(outline_width(&two, Some([0, 0, 0]), &[]), Some(2));
        let none = art(
            &["......", ".ffff.", ".fssf.", ".fssf.", ".ffff.", "......"],
            &key,
        );
        assert_eq!(outline_width(&none, Some([0, 0, 0]), &[]), Some(0));
        // A tile cut by a clear gutter: inside its cell, its edge is not
        // clear, so only the line round its own hole is the silhouette.
        let tiles = art(&["ffff.ffff", "fkkf.ffff", "fk.f.ffff", "fkkf.ffff"], &key);
        assert_eq!(
            outline_width(&tiles, Some([0, 0, 0]), &[[0, 0, 4, 4], [5, 0, 4, 4]]),
            Some(1)
        );
        assert_eq!(outline_width(&tiles, Some([0, 0, 0]), &[]), Some(0));
    }

    /// A model of `tris` triangles in one primitive, drawn by `nodes` nodes;
    /// the first `tilted` triangles' corners carry a normal 10° off their face.
    fn model(tris: usize, nodes: usize, tilted: usize) -> CookedModel {
        let mut vertices = Vec::new();
        let n10 = [10f32.to_radians().sin(), 0.0, 10f32.to_radians().cos()];
        for t in 0..tris {
            let x = t as f32;
            let normal = if t < tilted { n10 } else { [0.0, 0.0, 1.0] };
            for p in [[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]] {
                vertices.push(Vertex {
                    position: p,
                    normal,
                    ..Vertex::default()
                });
            }
        }
        let count = tris * 3;
        CookedModel {
            id: "m.glb".into(),
            primitives: vec![CookedPrimitive {
                material: 0,
                vertices,
                tangents: Vec::new(),
                indices: (0..count as u32).collect(),
                lods: vec![Lod {
                    first: 0,
                    count,
                    error: 0.0,
                }],
                min: glam::Vec3::ZERO,
                max: glam::Vec3::ONE,
                bvh_parts: None,
            }],
            meshes: vec![CookedMesh {
                name: "m".into(),
                primitives: 0..1,
            }],
            materials: Vec::new(),
            textures: Vec::new(),
            nodes: (0..nodes)
                .map(|i| CookedNode {
                    name: format!("n{i}"),
                    parent: None,
                    mesh: Some(0),
                    translation: glam::Vec3::new(0.0, i as f32 * 2.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                    visible: true,
                })
                .collect(),
            scenes: vec![CookedScene {
                name: "s".into(),
                roots: (0..nodes).collect(),
            }],
            default_scene: Some(0),
            min: glam::Vec3::ZERO,
            max: glam::Vec3::ONE,
            skipped_animations: 0,
            skipped_skins: 0,
        }
    }

    #[test]
    fn a_300_face_prop_is_over_200_and_within_500() {
        // 100 triangles drawn three times: 300 faces, not 100.
        let m = measure_model(&model(100, 3, 30), &|_| None);
        assert_eq!(m.faces, 300);
        // 30 of 100 triangles' corners are 10° off their face, in every node.
        assert_eq!(m.flat_share, Some(0.7));
        let ok = |budget: u32| {
            let c = Contract {
                prop_faces: budget,
                ..contract()
            };
            judge_model(&c, "m.glb", &m, false)
                .checks
                .iter()
                .find(|k| k.name == "faces")
                .unwrap()
                .ok
        };
        assert!(!ok(200));
        assert!(ok(500));
        let normals = judge_model(&contract(), "m.glb", &m, false);
        assert!(
            !normals
                .checks
                .iter()
                .find(|k| k.name == "normals")
                .unwrap()
                .ok
        );
        // A character is held to characterFaces, not propFaces.
        let c = Contract {
            prop_faces: 200,
            character_faces: 500,
            ..contract()
        };
        let a = judge_model(&c, "m.glb", &m, true);
        assert!(a.checks.iter().find(|k| k.name == "faces").unwrap().ok);
        assert_eq!(
            measure_model(&model(10, 1, 0), &|_| None).flat_share,
            Some(1.0)
        );
    }

    #[test]
    fn the_critic_reads_the_checks_and_frames_and_never_the_style() {
        let checks = catalogue_checks("16bit-pixel");
        assert_eq!(checks.len(), 3);
        let frames = vec![Frame {
            screen: "play".into(),
            path: PathBuf::from("/g/og-out/style/play/screenshot-000210.png"),
            colours: 0,
            colours95: 0,
            palette_hit: None,
            edges: Edges::default(),
            grid: 1,
        }];
        let styles = catalogue_styles();
        let entry = styles.iter().find(|e| e["slug"] == "16bit-pixel").unwrap();
        let name = entry["name"].as_str().unwrap();
        for json in [false, true] {
            let out = critic(&checks, &frames, json);
            for c in &checks {
                let c = if json {
                    serde_json::to_string(c).unwrap()
                } else {
                    c.clone()
                };
                assert!(out.contains(c.trim_matches('"')), "{out}");
            }
            assert!(out.contains("screenshot-000210.png"));
            assert!(
                !out.contains("16bit") && !out.contains(name) && !out.contains("looks/"),
                "{out}"
            );
        }
    }

    #[test]
    fn a_script_gets_a_screenshot_at_its_last_step() {
        assert_eq!(
            with_screenshot("1:key:Escape;2.5:wait"),
            "1:key:Escape;2.5:wait;2.5:screenshot"
        );
        assert_eq!(
            with_screenshot("3:screenshot;4:quit"),
            "3:screenshot;4:quit"
        );
    }
}
