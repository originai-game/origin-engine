//! `og style check [project] [--title <commands>] [--play <commands>]
//! [--results <commands>] [--size WxH] [--no-frames] [--json]`:
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
//! - **clips** (`ArtMotion`): each cooked model clip's resampling rate and
//!   whether its moving channels hold each pose, against `stepFps`, and its
//!   frames against `maxFrames`; each flipbook clip's frame durations
//!   (whole steps of `stepFps`) and frame count.
//!
//! Every opt-out (`conform off`, project.ogt's `conform { off ... }`, and
//! what the cook leaves alone by kind) is listed: an opted-out asset is
//! still measured and judged, since it is still the game's art.
//!
//! Frames: every screen the project declares is captured by `og run
//! --headless --commands`: project.ogt's `screens { title "<commands>"
//! play ... results ... shop ... }` (title, play and results first, each
//! replaceable by `--title`, `--play`, `--results`; any other name a
//! screen of the game's own, such as a shop or a map), then the `ui {
//! screen "<name>" "<commands>" }` entries og ui check visits; a script
//! without a `screenshot` gets one at its last step, one with several
//! (a mid-transition still) gives a frame for each. Each screen is
//! captured once per viewport: `--size WxH` one size, `--sizes
//! desktop,phone` several (desktop 1280×720, phoneLandscape 844×390,
//! phonePortrait 390×844, tablet 820×1180, or WxH; `phone` is the phone
//! in the game's orientation, portrait under `publish { orientation
//! portrait }`, else landscape); the default is desktop alone. Each frame
//! is labelled `<screen>@<viewport>`, carries its size and the renderer
//! tier that drew it, and is measured: colour count, palette hit rate,
//! edge statistics (edge density, the share of hard one-pixel steps, mean
//! contrast), its pixel grid and the camera it was captured with
//! (`og.Camera`, read through the probe at the screenshot) against
//! `ArtCamera`: the projection, the pitch and yaw with `fixedAngle`, a
//! perspective camera's field of view against `fovMin` and `fovMax`; a
//! camera off the contract makes the exit status 3. The coverage line
//! names each scene project.ogt declares with no capture script (a screen
//! of its name; the start scene is covered by `play`). `--json` prints one JSON document, its `contract` the flattened art
//! blocks the frames were judged against.
//!
//! A project with no look in project.ogt or GameDesc has no contract: it says so
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

const USAGE: &str = "usage: og style check [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH | --sizes desktop,phone,...] [--no-frames] [--json]";

/// The catalogue of styles (the knowledge package's), to name the slug a look builds on.
const CATALOGUE: &str = include_str!("../../../packages/knowledge/looks/catalogue.json");

/// The screens every game is captured at, first and in this order (each
/// replaceable by its flag); `screens {}` may name more.
pub const SCREENS: [&str; 3] = ["title", "play", "results"];

/// The named viewports of `--sizes` (og ui check's breakpoints): name,
/// width, height in CSS px.
pub const VIEWPORTS: [(&str, u32, u32); 4] = [
    ("desktop", 1280, 720),
    ("phoneLandscape", 844, 390),
    ("phonePortrait", 390, 844),
    ("tablet", 820, 1180),
];

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
    /// Every art block of the cooked (flattened) look as JSON, field by
    /// field: numbers, booleans, words, strings, lists of numbers, and a
    /// colour as sRGB `#rrggbb` (the terms the frames were judged against).
    pub art: Value,
    /// `ArtCamera.projection`: `any`, `perspective` or `orthographic`.
    pub projection: String,
    /// `ArtCamera.fixedAngle`, `pitch` and `yaw` (degrees).
    pub fixed_angle: bool,
    pub pitch: f64,
    pub yaw: f64,
    /// `ArtCamera.fovMin` and `fovMax` (degrees; 0 no limit).
    pub fov_min: f64,
    pub fov_max: f64,
    /// `ArtMotion.stepFps` (0 smooth) and `maxFrames` (0 no limit).
    pub step_fps: f64,
    pub max_frames: u32,
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

/// One look value as JSON: a number, `true`/`false`, a word or a string.
fn value_json(v: &og_core::ogt::Value) -> Value {
    match v {
        og_core::ogt::Value::Number(n) => json!(n.as_f64()),
        og_core::ogt::Value::String(s) => json!(s),
        og_core::ogt::Value::Word(w) => match w.as_str() {
            "true" => json!(true),
            "false" => json!(false),
            w => json!(w),
        },
    }
}

/// The look's art blocks as JSON (`{ "ArtLine": { "width": 2, "color":
/// "#1a1a2e" }, ... }`), in the order og_cook's ART_RESOURCES lists them.
fn art_blocks(look: &LookFile) -> Value {
    let mut out = serde_json::Map::new();
    for r in og_cook::look::ART_RESOURCES {
        let Some(b) = look.blocks.iter().find(|b| b.resource == *r) else {
            continue;
        };
        let mut fields = serde_json::Map::new();
        for (name, values, _) in &b.fields {
            let numbers: Option<Vec<f32>> = values
                .iter()
                .map(|v| v.as_number().map(|n| n.as_f32()))
                .collect();
            let v = match (name.as_str(), values.as_slice(), numbers) {
                // A colour is cooked linear: written back as the sRGB a look writes.
                ("color", [_, _, _, ..], Some(n)) => json!(format!(
                    "#{:02x}{:02x}{:02x}",
                    srgb8(n[0]),
                    srgb8(n[1]),
                    srgb8(n[2])
                )),
                (_, [one], _) => value_json(one),
                (_, many, _) => json!(many.iter().map(value_json).collect::<Vec<_>>()),
            };
            fields.insert(name.clone(), v);
        }
        out.insert(r.to_string(), Value::Object(fields));
    }
    Value::Object(out)
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
            art: art_blocks(look),
            projection: word(look, "ArtCamera", "projection", "any"),
            fixed_angle: field(look, "ArtCamera", "fixedAngle").and_then(|v| v.as_bool())
                == Some(true),
            pitch: num(look, "ArtCamera", "pitch"),
            yaw: num(look, "ArtCamera", "yaw"),
            fov_min: num(look, "ArtCamera", "fovMin"),
            fov_max: num(look, "ArtCamera", "fovMax"),
            step_fps: num(look, "ArtMotion", "stepFps"),
            max_frames: num(look, "ArtMotion", "maxFrames") as u32,
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

/// project.ogt's body, parsed.
fn project_body(root: &Path) -> Result<og_core::ogt::Block, String> {
    let path = root.join("project.ogt");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    og_core::ogt::parse(&text)
        .map(|d| d.body)
        .map_err(|e| format!("{}: {}", path.display(), e.reason()))
}

/// Resolve a declaration from project.ogt, then the game's evaluated GameDesc.
/// Importing the bundle records declarations only: no world or start callback.
fn declared_look(root: &Path) -> Result<Option<String>, String> {
    let body = project_body(root)?;
    if let Some(id) = og_cook::look::project_look(&body).map_err(|e| format!("project.ogt:{e}"))? {
        return Ok(Some(id));
    }
    let p = crate::project::Project::open(root)?;
    let package = crate::project::engine_package(Some(root)).ok_or(crate::project::NO_ENGINE)?;
    let bundle = crate::project::bundle_native(&p, &package)?;
    let output = bundle.with_file_name("game-look.json");
    let status = crate::project::bun(&package, "src/node/game-look.ts")
        .arg(&bundle)
        .arg(&output)
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("could not read GameDesc.look (the error is above)".into());
    }
    let look: String = serde_json::from_slice(&std::fs::read(&output).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok((!look.is_empty()).then(|| og_cook::runtime::asset_id(&look).to_string()))
}

/// The contract of the project at `root`, read from its cooked look.
pub fn contract(root: &Path) -> Result<Option<Contract>, String> {
    declared_look(root)?
        .map(|id| contract_of(root, &id))
        .transpose()
}

fn contract_of(root: &Path, id: &str) -> Result<Contract, String> {
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let bytes = bundle.read(id).map_err(|e| e.to_string())?;
    let look = bundle.look(id, &bytes).map_err(|e| e.to_string())?;
    let palette = |pid: &str| -> Result<Vec<[u8; 3]>, String> {
        let bytes = bundle
            .read(pid)
            .map_err(|e| format!("the look's ArtPalette.colors: {e}"))?;
        bundle.palette(pid, &bytes).map_err(|e| e.to_string())
    };
    Contract::of(id, &look, &palette)
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
        // ArtMotion: the model's clips and a flipbook's, as cooked.
        if c.step_fps > 0.0 || c.max_frames > 0 {
            for o in &asset.outputs {
                if o.ty == "og.Animation" {
                    let anim = bundle.load_animation(&o.id).map_err(|e| e.to_string())?;
                    for clip in &anim.clips {
                        let mut a = judge_clip(&c, &o.id, clip);
                        a.off = off_of(&o.id);
                        assets.push(a);
                    }
                } else if o.ty == "og.Flipbook" {
                    let sheet = bundle.load_flipbook(&o.id).map_err(|e| e.to_string())?;
                    for mut a in judge_flipbook(&c, &o.id, &sheet) {
                        a.off = off_of(&o.id);
                        assets.push(a);
                    }
                }
            }
        }
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
    /// The viewport's name (`desktop`, `phoneLandscape`) or `WxH`.
    pub viewport: String,
    /// The frame's size in pixels, as captured.
    pub size: [u32; 2],
    /// The renderer tier that drew it (`og run --json`'s `tier`), when known.
    pub tier: Option<String>,
    pub path: PathBuf,
    pub colours: usize,
    /// The fewest colours that cover 95 % of the pixels.
    pub colours95: usize,
    pub palette_hit: Option<f64>,
    pub edges: Edges,
    pub grid: u32,
    /// The camera it was captured with (`og.Camera` through the probe),
    /// and its checks against `ArtCamera`.
    pub camera: Value,
    pub camera_checks: Vec<Check>,
}

impl Frame {
    /// `<screen>@<viewport>`: the frame's name for a reviewer.
    pub fn label(&self) -> String {
        format!("{}@{}", self.screen, self.viewport)
    }
}

/// Measures one frame (a PNG) against the contract's palette.
pub fn measure_frame(
    screen: &str,
    viewport: &str,
    tier: Option<&str>,
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
        viewport: viewport.to_string(),
        size: [img.w, img.h],
        tier: tier.map(str::to_string),
        path: path.to_path_buf(),
        colours: counts.len(),
        colours95,
        palette_hit: palette.and_then(|p| palette_hit(&counts, p)),
        edges: edges(&img),
        grid: if g.kx == g.ky { g.kx } else { g.kx.min(g.ky) },
        camera: Value::Null,
        camera_checks: Vec::new(),
    })
}

/// The scripts that reach each screen: project.ogt's `screens { title
/// "<commands>" play "..." results "..." }` (title, play and results first,
/// each replaced by its flag, then the block's other screens in order),
/// then the `ui { screen "<name>" "<commands>" }` entries not named there.
pub fn screens(
    root: &Path,
    flags: &BTreeMap<String, String>,
) -> Result<Vec<(String, String)>, String> {
    let body = project_body(root)?;
    let mut out: Vec<(String, String)> = Vec::new();
    let block = body.get("screens").and_then(|e| e.block.clone());
    let mut declared: Vec<(String, String)> = Vec::new();
    if let Some(b) = &block {
        for e in &b.entries {
            let script = e.string(0).map_err(|_| {
                format!(
                    "project.ogt:{}: screens takes <name> \"<commands>\" (e.g. play \"0:key:Enter\"); `{}` has no commands",
                    e.line, e.key
                )
            })?;
            if declared.iter().any(|(n, _)| n == &e.key) {
                return Err(format!(
                    "project.ogt:{}: screens names `{}` twice",
                    e.line, e.key
                ));
            }
            declared.push((e.key.clone(), script.to_string()));
        }
    }
    for s in SCREENS {
        let script = flags.get(s).cloned().or_else(|| {
            declared
                .iter()
                .find(|(n, _)| n == s)
                .map(|(_, c)| c.clone())
        });
        if let Some(script) = script {
            out.push((s.to_string(), with_screenshot(&script)));
        }
    }
    for (name, script) in declared
        .into_iter()
        .filter(|(n, _)| !SCREENS.contains(&n.as_str()))
    {
        out.push((name, with_screenshot(&script)));
    }
    let ui = og_cook::cook::ui_screens::screens(&body).map_err(|e| format!("project.ogt:{e}"))?;
    for s in ui {
        if !out.iter().any(|(n, _)| n == &s.name) {
            out.push((s.name, with_screenshot(&s.commands)));
        }
    }
    Ok(out)
}

/// The scenes project.ogt declares (`scenes { <name> "<path>" }`) that no
/// capture script reaches: no screen of their name, and for the start
/// scene (`scene <name>`) no `play` either.
pub fn uncovered_scenes(root: &Path, captured: &[String]) -> Result<Vec<String>, String> {
    let body = project_body(root)?;
    let start = body
        .get("scene")
        .and_then(|e| e.word(0).or_else(|_| e.string(0)).ok())
        .map(str::to_string);
    let has = |n: &str| captured.iter().any(|c| c == n);
    Ok(body
        .get("scenes")
        .and_then(|e| e.block.as_ref())
        .map(|b| b.entries.iter().map(|e| e.key.clone()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|s| !has(s) && !(start.as_deref() == Some(s.as_str()) && has("play")))
        .collect())
}

/// `--sizes`' list as (label, `WxH`): a named viewport or `WxH`; `phone`
/// is phonePortrait when `portrait`, else phoneLandscape.
pub fn viewports(spec: &str, portrait: bool) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for v in spec.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let name = match v {
            "phone" if portrait => "phonePortrait",
            "phone" => "phoneLandscape",
            v => v,
        };
        let entry = if let Some((n, w, h)) = VIEWPORTS.iter().find(|(n, _, _)| *n == name) {
            (n.to_string(), format!("{w}x{h}"))
        } else {
            let ok = name.split_once('x').is_some_and(|(w, h)| {
                w.parse::<u32>().is_ok_and(|w| w > 0) && h.parse::<u32>().is_ok_and(|h| h > 0)
            });
            if !ok {
                return Err(format!(
                    "--sizes: `{v}` is not WxH nor one of phone, {}",
                    VIEWPORTS.map(|(n, _, _)| n).join(", ")
                ));
            }
            (name.to_string(), name.to_string())
        };
        if out.iter().any(|(_, s)| *s == entry.1) {
            return Err(format!("--sizes names {} twice", entry.1));
        }
        out.push(entry);
    }
    if out.is_empty() {
        return Err("--sizes names no viewport".into());
    }
    Ok(out)
}

/// Whether the game is designed portrait (`publish { orientation portrait }`).
fn portrait(root: &Path) -> bool {
    project_body(root).ok().is_some_and(|b| {
        b.get("publish")
            .and_then(|e| e.block.as_ref())
            .and_then(|p| p.get("orientation"))
            .and_then(|e| e.word(0).ok())
            == Some("portrait")
    })
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

/// `script` reading the camera (`get:og.Camera`) into `camera.json` at its
/// first screenshot's time, so the frame's camera is measured with it.
pub fn with_camera(script: &str) -> String {
    let at = script
        .split(';')
        .map(str::trim)
        .find(|s| s.split(':').nth(1) == Some("screenshot"))
        .and_then(|s| s.split(':').next())
        .unwrap_or("0")
        .to_string();
    format!("{script};{at}:get:og.Camera:{CAMERA_FILE}")
}

/// Where a screen's capture writes its camera.
const CAMERA_FILE: &str = "camera.json";

/// Captures one screen with `og run --headless --commands` into `dir`
/// (under og-out/style/): its PNGs and the renderer tier that drew them.
fn capture(
    root: &Path,
    dir: &str,
    screen: &str,
    script: &str,
    size: Option<&str>,
) -> Result<(Vec<PathBuf>, Option<String>), String> {
    let script = &with_camera(script);
    let dir = root.join("og-out/style").join(dir);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("run")
        .arg(root)
        .args([
            "--headless",
            "--no-check",
            "--json",
            "--commands",
            script,
            "--out",
        ])
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
    Ok((pngs, run_tier(&String::from_utf8_lossy(&out.stdout))))
}

/// The renderer tier of `og run --json`'s summary (its last JSON line).
pub fn run_tier(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .rev()
        .filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
        .find_map(|v| v["tier"].as_str().map(str::to_string))
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
        "viewport": f.viewport,
        "label": f.label(),
        "size": f.size,
        "tier": f.tier,
        "path": f.path.display().to_string(),
        "colours": f.colours,
        "colours95": f.colours95,
        "paletteHit": f.palette_hit,
        "edges": { "density": f.edges.density, "hard": f.edges.hard, "contrast": f.edges.contrast },
        "texelsPerPixel": f.grid,
        "camera": f.camera,
        "cameraChecks": f.camera_checks.iter().map(|c| json!({
            "name": c.name, "value": c.value, "limit": c.limit, "ok": c.ok,
        })).collect::<Vec<_>>(),
    })
}

/// What a check captured: the screens it ran and the scenes none reaches.
#[derive(Clone, Debug, Default)]
pub struct Coverage {
    pub screens: Vec<String>,
    pub viewports: Vec<String>,
    pub uncovered: Vec<String>,
}

/// The whole check as one JSON document.
pub fn to_json(m: &Measured, frames: &[Frame], frame_errors: &[String], cov: &Coverage) -> Value {
    let c = &m.contract;
    let slug = c.slug();
    json!({
        "contract": {
            "look": c.look,
            "extends": c.extends,
            "sets": c.sets,
            "palette": c.palette.as_ref().map(|(id, p)| json!({
                "id": id,
                "colours": p.len(),
                "hex": p.iter().map(|q| format!("#{:02x}{:02x}{:02x}", q[0], q[1], q[2])).collect::<Vec<_>>(),
            })),
            "art": c.art,
            "style": slug.as_ref().map(|s| json!({ "slug": s })),
        },
        "coverage": {
            "screens": cov.screens,
            "viewports": cov.viewports,
            "uncoveredScenes": cov.uncovered,
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
pub fn report(m: &Measured, frames: &[Frame], frame_errors: &[String], cov: &Coverage) -> String {
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
            "  {:<16} {}{}\n          colours {} (95% in {}){}  edges {} of pixels, {} hard, contrast {:.2}  grid {} px\n",
            f.label(),
            f.path.display(),
            f.tier.as_ref().map(|t| format!("  ({}×{}, {t})", f.size[0], f.size[1])).unwrap_or_else(|| format!("  ({}×{})", f.size[0], f.size[1])),
            f.colours,
            f.colours95,
            f.palette_hit.map(|h| format!("  palette {}", pct(h))).unwrap_or_default(),
            pct(f.edges.density),
            pct(f.edges.hard),
            f.edges.contrast,
            f.grid
        ));
        for c in &f.camera_checks {
            s.push_str(&format!(
                "          {} camera {}: {} against {}\n",
                if c.ok { "✓" } else { "✗" },
                c.name,
                c.value,
                c.limit
            ));
        }
    }
    for e in frame_errors {
        s.push_str(&format!("  failed: {e}\n"));
    }
    if !cov.uncovered.is_empty() {
        s.push_str(&format!(
            "coverage: no capture script for the scene(s) {}: add <name> \"<commands>\" to project.ogt's screens {{}} for each\n",
            cov.uncovered.join(", ")
        ));
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
    sizes: Option<String>,
    frames: bool,
    json: bool,
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let mut a = Args {
        project: ".".into(),
        flags: BTreeMap::new(),
        size: None,
        sizes: None,
        frames: true,
        json: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => a.json = true,
            "--no-frames" => a.frames = false,
            f @ ("--title" | "--play" | "--results" | "--size" | "--sizes") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{f} needs a value. {USAGE}"));
                };
                if f == "--size" {
                    a.size = Some(v.to_string());
                } else if f == "--sizes" {
                    a.sizes = Some(v.to_string());
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
    if a.size.is_some() && a.sizes.is_some() {
        return Err(format!("--size and --sizes: give one. {USAGE}"));
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
    let look = match declared_look(&root) {
        Ok(Some(id)) => id,
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
    };
    let cooked = match crate::cook::cook(&root, 0, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    };
    let measured =
        match contract_of(&root, &look).and_then(|c| measure(&root, c, opt_outs_of(&cooked))) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("og style check: {e}");
                return ExitCode::FAILURE;
            }
        };
    let mut frames = Vec::new();
    let mut frame_errors = Vec::new();
    let sizes = match (&a.sizes, &a.size) {
        (Some(s), _) => match viewports(s, portrait(&root)) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("og style check: {e}");
                return ExitCode::from(2);
            }
        },
        (None, Some(s)) => vec![(s.clone(), s.clone())],
        (None, None) => vec![("desktop".to_string(), "1280x720".to_string())],
    };
    let mut cov = Coverage {
        viewports: sizes.iter().map(|(n, _)| n.clone()).collect(),
        ..Coverage::default()
    };
    // What would be captured, and the scenes nothing reaches, said with or without frames.
    let scripts = match screens(&root, &a.flags) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    };
    cov.screens = scripts.iter().map(|(n, _)| n.clone()).collect();
    match uncovered_scenes(&root, &cov.screens) {
        Ok(u) => cov.uncovered = u,
        Err(e) => {
            eprintln!("og style check: {e}");
            return ExitCode::FAILURE;
        }
    }
    if a.frames {
        let palette = measured
            .contract
            .palette
            .as_ref()
            .map(|(_, p)| p.as_slice());
        for (screen, script) in &scripts {
            for (viewport, size) in &sizes {
                // One viewport keeps og-out/style/<screen>/; several, one directory each.
                let dir = if a.sizes.is_some() {
                    format!("{screen}@{viewport}")
                } else {
                    screen.clone()
                };
                match capture(&root, &dir, screen, script, Some(size)) {
                    Ok((pngs, tier)) => {
                        // The camera the screen was captured with.
                        let camera = pngs
                            .first()
                            .and_then(|p| p.parent())
                            .and_then(|d| std::fs::read_to_string(d.join(CAMERA_FILE)).ok())
                            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                            .map(|v| v["value"].clone());
                        for p in pngs {
                            match measure_frame(screen, viewport, tier.as_deref(), &p, palette) {
                                Ok(mut f) => {
                                    if let Some(cam) = &camera {
                                        let (m, checks) = judge_camera(&measured.contract, cam);
                                        f.camera = m;
                                        f.camera_checks = checks;
                                    }
                                    frames.push(f)
                                }
                                Err(e) => frame_errors.push(e),
                            }
                        }
                    }
                    Err(e) => frame_errors.push(e),
                }
            }
        }
    }
    if a.json {
        println!("{}", to_json(&measured, &frames, &frame_errors, &cov));
    } else {
        println!("og style check {}", root.display());
        print!("{}", report(&measured, &frames, &frame_errors, &cov));
    }
    let camera_off = frames.iter().any(|f| f.camera_checks.iter().any(|c| !c.ok));
    if !frame_errors.is_empty() {
        ExitCode::FAILURE
    } else if measured.off_contract().is_empty() && !camera_off {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

/// What the check says of a project whose project.ogt names no look.
pub const NO_CONTRACT: &str = "neither project.ogt nor GameDesc declares a look, so there is no art contract to check against";

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

    #[test]
    fn contract_follows_game_desc_look_without_starting_the_game() {
        let root = std::env::temp_dir().join(format!("og-style-game-look-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets/looks")).unwrap();
        std::fs::write(
            root.join("project.ogt"),
            "type project\nversion 1\n---\nname \"Test\"\nentry \"main.ts\"\n",
        )
        .unwrap();
        std::fs::write(root.join("main.ts"), r#"
            import { og } from "@origingame/engine";
            const family = "custom";
            export default og.game({ look: `looks/${family}.ogt`, start() { throw new Error("must not start"); } });
        "#).unwrap();
        std::fs::write(
            root.join("assets/looks/custom.ogt"),
            "type look\nversion 1\n---\nArtLine {\n  width 2\n}\n",
        )
        .unwrap();
        assert_eq!(
            declared_look(&root).unwrap().as_deref(),
            Some("looks/custom.ogt")
        );
        crate::cook::cook(&root, 0, false).unwrap();
        let c = super::contract(&root).unwrap().unwrap();
        assert_eq!(c.look, "looks/custom.ogt");
        assert_eq!(c.line, 2.0);
        std::fs::write(
            root.join("main.ts"),
            "import { og } from '@origingame/engine'; export default og.game({});",
        )
        .unwrap();
        assert!(declared_look(&root).unwrap().is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

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
            skin_binds: Default::default(),
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
                foliage: None,
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

    /// The rotation looking `pitch` degrees below the horizon turned `yaw`
    /// about +Y (Ry(yaw) Rx(-pitch)), as {x, y, z, w}, worked out by hand.
    fn camera_quat(pitch: f64, yaw: f64) -> Value {
        let (sx, cx) = (-pitch.to_radians() / 2.0).sin_cos();
        let (sy, cy) = (yaw.to_radians() / 2.0).sin_cos();
        json!({ "x": cy * sx, "y": sy * cx, "z": -sy * sx, "w": cy * cx })
    }

    /// ArtCamera measured on the captured camera: projection, the fixed
    /// angle and the field of view, each judged only where the contract
    /// sets it. Old code measured no camera at all.
    #[test]
    fn the_check_measures_the_frames_camera_against_art_camera() {
        let iso = Contract {
            projection: "orthographic".into(),
            fixed_angle: true,
            pitch: 30.0,
            yaw: 45.0,
            ..contract()
        };
        let cam = |ortho: bool, pitch: f64, yaw: f64, fov: f64| json!({ "orthographic": ortho, "quaternion": camera_quat(pitch, yaw), "fov": fov });
        let by = |checks: &[Check], n: &str| checks.iter().find(|c| c.name == n).map(|c| c.ok);
        let (m, checks) = judge_camera(&iso, &cam(true, 30.0, 45.0, 50.0));
        assert!((m["pitch"].as_f64().unwrap() - 30.0).abs() < 1e-6, "{m}");
        assert!((m["yaw"].as_f64().unwrap() - 45.0).abs() < 1e-6, "{m}");
        assert!(checks.iter().all(|c| c.ok), "{checks:?}");
        let (_, checks) = judge_camera(&iso, &cam(false, 30.0, 45.0, 50.0));
        assert_eq!(
            by(&checks, "projection"),
            Some(false),
            "a perspective camera"
        );
        let (_, checks) = judge_camera(&iso, &cam(true, 20.0, -45.0, 50.0));
        assert_eq!(by(&checks, "pitch"), Some(false));
        assert_eq!(by(&checks, "yaw"), Some(false));
        // Straight down has no heading: any yaw holds.
        let top = Contract {
            pitch: 90.0,
            yaw: 0.0,
            ..iso.clone()
        };
        let (_, checks) = judge_camera(&top, &cam(true, 90.0, 0.0, 50.0));
        assert_eq!(by(&checks, "yaw"), Some(true), "{checks:?}");
        // The field of view, for a perspective camera only.
        let lens = Contract {
            fov_min: 30.0,
            fov_max: 45.0,
            ..contract()
        };
        let (_, checks) = judge_camera(&lens, &cam(false, 0.0, 0.0, 60.0));
        assert_eq!(by(&checks, "fov"), Some(false));
        let (_, checks) = judge_camera(&lens, &cam(false, 0.0, 0.0, 40.0));
        assert_eq!(by(&checks, "fov"), Some(true));
        let (_, checks) = judge_camera(&contract(), &cam(false, 10.0, 0.0, 90.0));
        assert!(checks.is_empty(), "no ArtCamera, no checks: {checks:?}");
    }

    /// ArtMotion measured on the cooked clips: a model clip's rate and held
    /// poses against stepFps and its frames against maxFrames, a
    /// flipbook's frame durations as whole steps.
    #[test]
    fn the_check_measures_clips_against_art_motion() {
        let c = Contract {
            step_fps: 12.0,
            max_frames: 8,
            ..contract()
        };
        let clip = |rate: f32, frames: u32, step: bool| og_anim::ClipAsset {
            name: "walk".into(),
            duration: (frames - 1) as f32 / rate,
            rate,
            source_rate: 30.0,
            frames,
            channels: vec![og_anim::Channel {
                node: 0,
                node_name: "hip".into(),
                path: og_anim::ChannelPath::Translation,
                step,
                lanes: 3,
                samples: og_anim::Samples::F32(vec![0.0; frames as usize * 3]),
            }],
            events: Vec::new(),
        };
        let ok = |a: &Asset, n: &str| a.checks.iter().find(|k| k.name == n).unwrap().ok;
        let held = judge_clip(&c, "m.glb#anim", &clip(12.0, 7, true));
        assert!(
            ok(&held, "stepFps") && ok(&held, "maxFrames"),
            "{:?}",
            held.checks
        );
        assert_eq!(held.role, Role::Clip);
        let smooth = judge_clip(&c, "m.glb#anim", &clip(30.0, 31, false));
        assert!(!ok(&smooth, "stepFps") && !ok(&smooth, "maxFrames"));
        let interpolated = judge_clip(&c, "m.glb#anim", &clip(12.0, 7, false));
        assert!(!ok(&interpolated, "stepFps"), "a pose not held");
        let sheet = og_anim::FlipbookSheet::parse(
            "type flipbook\nversion 1\n---\ntexture \"t.png\"\nsize 64 8\ngrid 8 8\nclip \"run\" {\n  frames 0 1 2 3\n  fps 30\n}\n",
        )
        .unwrap();
        let fb = judge_flipbook(&c, "run.ogt", &sheet);
        assert_eq!(fb.len(), 1);
        assert!(
            !ok(&fb[0], "stepFps"),
            "1/30 s frames are not whole 1/12 s steps"
        );
        assert!(ok(&fb[0], "maxFrames"));
        let mut stepped = sheet.clone();
        stepped.clips[0].durations.fill(2.0 / 12.0);
        assert!(ok(&judge_flipbook(&c, "run.ogt", &stepped)[0], "stepFps"));
    }

    fn project(text: &str) -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "og-style-screens-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("project.ogt"),
            format!("type project\nversion 1\n---\nname \"x\"\n{text}"),
        )
        .unwrap();
        dir
    }

    // A game's own screens are captured too: title, play and results first
    // (whatever order the block writes them), then the block's others in
    // order, then the ui block's screens it does not name; a flag replaces
    // its screen's script. A scene with no screen of its name is uncovered,
    // but the start scene counts as covered by play.
    #[test]
    fn every_declared_screen_is_captured_and_uncovered_scenes_are_named() {
        let dir = project(
            "scenes {\n  main \"scenes/main.ogt\"\n  shop \"scenes/shop.ogt\"\n  map \"scenes/map.ogt\"\n}\nscene main\nscreens {\n  shop \"1:screenshot\"\n  results \"2:screenshot\"\n  play \"0.5:key:Enter\"\n}\nui {\n  screen \"hud\" \"0:key:Enter\"\n  screen \"shop\" \"9:key:KeyS\"\n}\n",
        );
        let flags = BTreeMap::from([("results".to_string(), "3:key:Escape".to_string())]);
        let got = screens(&dir, &flags).unwrap();
        assert_eq!(
            got,
            vec![
                (
                    "play".to_string(),
                    "0.5:key:Enter;0.5:screenshot".to_string()
                ),
                (
                    "results".to_string(),
                    "3:key:Escape;3:screenshot".to_string()
                ),
                ("shop".to_string(), "1:screenshot".to_string()),
                ("hud".to_string(), "0:key:Enter;0:screenshot".to_string()),
            ]
        );
        let names: Vec<String> = got.into_iter().map(|(n, _)| n).collect();
        assert_eq!(
            uncovered_scenes(&dir, &names).unwrap(),
            vec!["map".to_string()]
        );
        // Without play, the start scene is uncovered too.
        assert_eq!(
            uncovered_scenes(&dir, &["shop".to_string()]).unwrap(),
            vec!["main".to_string(), "map".to_string()]
        );
        // A screen without commands, or named twice, is refused with its line.
        let bad = project("screens {\n  shop 3\n}\n");
        assert!(
            screens(&bad, &BTreeMap::new())
                .unwrap_err()
                .contains("project.ogt:6")
        );
        let twice = project("screens {\n  shop \"1:screenshot\"\n  shop \"2:screenshot\"\n}\n");
        assert!(
            screens(&twice, &BTreeMap::new())
                .unwrap_err()
                .contains("twice")
        );
        for d in [dir, bad, twice] {
            let _ = std::fs::remove_dir_all(d);
        }
    }

    // `phone` follows the game's orientation; WxH passes as itself; an
    // unknown name or a size given twice is refused.
    #[test]
    fn sizes_name_viewports_and_phone_follows_the_orientation() {
        assert_eq!(
            viewports("desktop,phone", false).unwrap(),
            vec![
                ("desktop".to_string(), "1280x720".to_string()),
                ("phoneLandscape".to_string(), "844x390".to_string()),
            ]
        );
        assert_eq!(
            viewports("phone,1600x900", true).unwrap(),
            vec![
                ("phonePortrait".to_string(), "390x844".to_string()),
                ("1600x900".to_string(), "1600x900".to_string()),
            ]
        );
        assert!(viewports("watch", false).is_err());
        assert!(viewports("desktop,1280x720", false).is_err());
        let p = project("publish {\n  orientation portrait\n}\n");
        assert!(portrait(&p));
        let _ = std::fs::remove_dir_all(p);
    }

    #[test]
    fn the_tier_comes_from_the_runs_last_json_line() {
        let out = "{\"event\":\"check\",\"ok\":true}\n{\"tick\":120,\"tier\":\"high\"}\n";
        assert_eq!(run_tier(out).as_deref(), Some("high"));
        assert_eq!(run_tier("tick 120 hash 1\n"), None);
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
