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
//!   with a skin or one that says `conform character`, as the cook holds
//!   it), the flat-normal share (corners on their face's normal;
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
use og_cook::cook::conform::TextureRole;
use og_cook::format::{codec, kind};
use og_cook::look::LookFile;
use og_cook::runtime::{Bundle, CookedModel, MANIFEST};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod measures;

pub use measures::*;
use measures::{LINE_RGB, pct, srgb8};

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
        let text = match og_cook::cook::look::styles()
            .iter()
            .find(|f| f.id() == a.source)
        {
            Some(f) => String::from_utf8_lossy(f.bytes()).to_string(),
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
    let roles = og_cook::cook::conform::texture_roles(root)?;
    let characters = og_cook::cook::conform::characters(root)?;
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
            // The cook's role for it (`texture_roles`), so the check judges
            // a texture as the pass that conformed it; a `.ktx2`, which no
            // pass touches, by the profile and its filter.
            let sprite = model.is_none()
                && match roles.get(&asset.source) {
                    Some(TextureRole::Sprite | TextureRole::Sheet | TextureRole::Fixed) => true,
                    Some(TextureRole::Mesh | TextureRole::Data) => false,
                    None => profile_2d || t.is_pixel_art(),
                };
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
        let character = characters.contains(&asset.source)
            || asset
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
    fn a_tile_of_flat_ground_under_a_fringe_is_not_taken_for_a_coarser_grid() {
        // monochrome-pixel's cooked ground (32x32, one texel per pixel):
        // grass in its top five rows, solid ink below. Its row boundaries
        // are rows 1 to 5; the 26 flat rows carry none, and counted as off
        // a period of 2 they let rows 2 and 4 pass as its half pixels.
        let key = [('#', [20, 38, 27]), ('.', [201, 218, 143])];
        let mut rows = vec![
            "................................",
            ".##....#....###......#....####..",
            "###.#.####.####.###.###.##.####.",
            "######.############.############",
            "##############..##.#############",
        ];
        rows.resize(32, "################################");
        let g = grid(&art(&rows, &key));
        assert_eq!((g.ky, g.half_rows.len()), (1, 0), "{g:?}");
        // The same tile drawn at 2 texels per pixel still reads as 2 down.
        let wide = scaled(&rows, 2);
        let g = grid(&art(
            &wide.iter().map(String::as_str).collect::<Vec<_>>(),
            &key,
        ));
        assert_eq!((g.ky, g.half_rows.len()), (2, 0), "{g:?}");
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
            lights: Vec::new(),
            instances: Default::default(),
            variants: Vec::new(),
            variant_materials: Vec::new(),
            impostor: None,
            primitives: vec![CookedPrimitive {
                material: 0,
                vertices,
                tangents: Vec::new(),
                indices: (0..count as u32).collect(),
                lods: vec![Lod {
                    first: 0,
                    count,
                    error: 0.0,
                    ..Default::default()
                }],
                clusters: Vec::new(),
                cluster_indices: Vec::new(),
                min: glam::Vec3::ZERO,
                max: glam::Vec3::ONE,
                bvh_parts: None,
                collider: None,
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
