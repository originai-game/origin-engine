//! `og style check-project [project] [--title <commands>] [--play <commands>]
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
//!   with a skin or one that says `conform character`, none for one that
//!   says `conform effect`, as the cook holds it), the flat-normal share (corners on their face's normal;
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
//! a capture failed. Numbers cannot judge a style (the `art-review` guide): `og
//! build --publish` prints the summary as warnings and never refuses a
//! build, and `og doctor --project` has one line of it.

use crate::compare::{delta_e00, lab};
use og_cook::cook::conform::{ModelRole, TextureRole};
use og_cook::format::{codec, kind};
use og_cook::look::LookFile;
use og_cook::runtime::{Bundle, CookedModel, MANIFEST};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod contract;
mod measures;
mod output;

pub use contract::*;
pub use measures::*;
use measures::{LINE_RGB, pct, srgb8};
pub use output::*;

const USAGE: &str = "usage: og style check-project [project] [--title <commands>] [--play <commands>] [--results <commands>] [--size WxH | --sizes desktop,phone,...] [--no-frames] [--json]";

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
    let model_roles = og_cook::cook::conform::model_roles(root)?;
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
        let said = model_roles.get(&asset.source).copied().unwrap_or_default();
        let skinned = asset
            .outputs
            .iter()
            .filter(|x| x.ty == "og.Animation")
            .any(|x| {
                bundle
                    .load_animation(&x.id)
                    .is_ok_and(|a| !a.skin_streams.is_empty())
            });
        let role = match said {
            ModelRole::Prop if skinned => ModelRole::Character,
            r => r,
        };
        if let Some([median, ..]) = measures.density {
            medians.push(median);
        }
        let mut a = judge_model(&c, &o.id, &measures, role);
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
            eprintln!("og style check-project: {e}");
            return ExitCode::from(2);
        }
    };
    let root = PathBuf::from(&a.project);
    if !root.join("project.ogt").is_file() {
        eprintln!(
            "og style check-project: {} is not a project (no project.ogt)",
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
                println!("og style check-project: {NO_CONTRACT}");
            }
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("og style check-project: {e}");
            return ExitCode::FAILURE;
        }
    };
    let cooked = match crate::cook::cook(&root, 0, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("og style check-project: {e}");
            return ExitCode::FAILURE;
        }
    };
    let measured =
        match contract_of(&root, &look).and_then(|c| measure(&root, c, opt_outs_of(&cooked))) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("og style check-project: {e}");
                return ExitCode::FAILURE;
            }
        };
    let mut frames = Vec::new();
    let mut frame_errors = Vec::new();
    let sizes = match (&a.sizes, &a.size) {
        (Some(s), _) => match viewports(s, portrait(&root)) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("og style check-project: {e}");
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
            eprintln!("og style check-project: {e}");
            return ExitCode::FAILURE;
        }
    };
    cov.screens = scripts.iter().map(|(n, _)| n.clone()).collect();
    match uncovered_scenes(&root, &cov.screens) {
        Ok(u) => cov.uncovered = u,
        Err(e) => {
            eprintln!("og style check-project: {e}");
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
        println!("og style check-project {}", root.display());
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
                "run `og cook` in the project, then `og style check-project`",
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
                    "run `og style check-project` for the frames and the numbers",
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
                    "run `og style check-project` to see each number against the contract",
                )
            }
        }
        Err(e) => line(
            "warning",
            format!("style: the cooked assets could not be measured: {e}"),
            "run `og cook` in the project, then `og style check-project`",
        ),
    }
}

#[cfg(test)]
mod tests;
