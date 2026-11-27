//! `og vfx preview <effect.ogt>... [--project DIR] [--out DIR] [--size WxH]
//! [--stage dusk|day] [--quality low|medium|high] [--no-cost | --cost-only | --together] [--keep] [--json]`: what an effect looks like, how big
//! it is and what it costs, without writing a scene for it.
//!
//! Each effect (an asset id of the project, `fx/hit.ogt`, or a file under
//! its `assets/`; `type vfx`, or `type vfx2d` in a 2D project) is played
//! alone at the origin of a dark stage (`vfx_preview/stage.ts`, run inside
//! the project so its cooked assets load as in the game), in four steps:
//!
//! 1. **timing**, one run without a renderer: its length, whether it
//!    loops, its elements and the time of its `impact`;
//! 2. **bounds**, measured, not declared: the effect is drawn alone over
//!    black from far in front and from far above at [`SAMPLES`] moments,
//!    and the pixels that differ from the empty stage give the box it
//!    reaches (x and y from the front, x and z from above, to about
//!    [`MEASURE_PX`]'s share of the view). An effect that reaches the
//!    view's edge is measured again from twice as far. Its core is the
//!    same box without the outermost hundredth of its pixels on each side
//!    (the few sparks that fly furthest). The ground there is unlit, so a
//!    light's reach is not part of the box, and the front view is level
//!    with it and sees it edge on, so what the effect draws below its
//!    origin is measured too (from the front alone: its depth is not).
//!    Particles that collide with the scene by depth have no ground to
//!    meet in that view, so they fall a little further than in a game.
//!    An effect with [`CENTRE_SHARE`] or more of itself below its origin
//!    is anchored at its `centre` (a ring about its origin, a shield),
//!    any other on the `ground`, and so is every effect with a `decal`,
//!    which lands on the surface at its origin. From above, what a centred effect draws
//!    below its origin is under the ground, so it is seen from above once
//!    more raised to stand on it, for its x and z. A decal lands on lit surfaces alone, so
//!    an effect with a `decal` that drew nothing over black (a bullet
//!    hole, a skid mark) is measured once more from above on a ground lit
//!    from straight overhead: its box is flat, what it left there;
//! 3. **shots**: six frames in time order (`frameSeconds`), the impact
//!    frame among them, spread over the time it is shown, or for an effect
//!    that leaves a mark (a `decal`) over the time it drew anything else
//!    over black and a quarter more, so the last shows what is left, the
//!    camera placed so the measured core fills the view (an effect that
//!    draws nothing measurable gets the default view) and an effect
//!    anchored at its centre raised to stand whole on the ground, each with the
//!    render dump's draw calls and particle slots, tiled 3 x 2 into
//!    `<out>/<effect>.png`, and the engine's warnings in that run that
//!    quote the effect's id;
//! 4. **cost**, one short traced run per effect: the 90th percentile of its
//!    frames' GPU time over the empty frames before it in the same run,
//!    with the simulation pass and the main pass's growth. Framed as the
//!    shots are, so an effect is timed filling the view. `--no-cost` skips
//!    it; `--cost-only` does nothing else and reads the rest from an
//!    earlier preview in `--out` (a pack's previews take their shots
//!    holding one GPU slot and their timings holding all of them).
//!
//! `--together` plays every effect named at once instead, on a grid
//! [`SPACING`] metres apart (a fight with many effects on screen), in one
//! traced run framed to show the whole grid: the frame's and each render
//! pass's GPU milliseconds over the empty stage, at the run's peak (90th
//! percentile) and typical (median) frame, the same rows for the CPU's
//! time recording them (`cpu`: a pass's draws and dispatches are its CPU
//! time), and three frames of it tiled into
//! `<out>/together.png`, written to `<out>/together.json`. Each effect starts
//! at its origin on the ground, whatever its anchor.
//!
//! `--stage day` shows them on a grey ground under an overcast day instead
//! of the dusk: what is lit (smoke, dust, debris, marks) reads there, and
//! what glows reads at dusk. The measuring and the numbers are the same.
//!
//! `--quality low` plays them with
//! that graphics preset applied (og.render.applyQuality), as a player who
//! picked it sees them: every frame, size and millisecond is at it (the
//! default is the stage's own settings, medium's). `--keep` leaves the stage's
//! runs in `.og/vfx-preview/` (the measuring frames among them).
//!
//! `<out>/<effect>.json` (default out: `<project>/og-out/vfx-preview`)
//! holds every number; they are printed as a table, or with `--json` as one
//! array. These are facts: what is too big or too slow is the `effects`
//! guide's to say.

use crate::project::{self, Project};
use image::RgbaImage;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og vfx preview <effect.ogt>... [--project DIR] [--out DIR] [--size WxH] [--stage dusk|day] [--project-look | --look asset.ogt] [--scene asset.ogt] [--position x,y,z] [--camera px,py,pz,tx,ty,tz,fov] [--quality low|medium|high] [--no-cost | --cost-only | --together] [--keep] [--json]";

mod measure;

use measure::bounds;

const STAGE: &str = include_str!("vfx_preview/stage.ts");
const TICK: f64 = 60.0;
/// Seconds between two effects' starts in the timing run: an effect's
/// events are looked for this long.
const SLOT: f64 = 4.0;
/// Seconds a looping effect is shown.
const LOOP_SHOW: f64 = 2.0;
/// Where in an effect's shown time its six shots are taken.
const FRACTIONS: [f64; 6] = [0.04, 0.15, 0.3, 0.5, 0.7, 0.9];
/// How much longer than it drew anything but its mark an effect that
/// leaves one is shot, as a multiple.
const MARK_TAIL: f64 = 1.25;
/// Where its bounds are sampled.
const SAMPLES: [f64; 10] = [0.03, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.95];
/// Effects per shot or measuring run.
const CHUNK: usize = 6;
/// The measuring views' side in pixels.
const MEASURE_PX: u32 = 768;
/// Half the measuring view's side at the origin, metres, to start with,
/// and the widest tried.
const MEASURE_HALF: f64 = 6.0;
const MEASURE_HALF_MAX: f64 = 48.0;
/// The share of an effect's pixels left outside its core at each end of
/// each axis: the few sparks that fly furthest do not set the camera.
const CORE_TRIM: f64 = 0.01;
/// An effect with this share of its pixels below its origin, seen from the
/// front, is anchored at its centre, unless it has a decal.
const CENTRE_SHARE: f64 = 0.1;
/// A channel differing by more than this from the empty stage (of 255) is
/// the effect's.
const THRESHOLD: i16 = 12;
/// The default view when nothing was measured: the centre it looks at and
/// the radius it frames (a camera 5.4 m away, as the packs' sheets had).
const DEFAULT_CENTRE: [f64; 3] = [0.0, 0.8, 0.0];
const DEFAULT_RADIUS: f64 = 2.07;
/// The smallest radius framed: a spark is not blown up to fill the view.
const MIN_RADIUS: f64 = 0.5;
/// From the framed centre to the camera, and its vertical field of view.
const VIEW_DIR: [f64; 3] = [0.0, 1.4, 5.2];
const VIEW_FOV: f64 = 50.0;
/// The shot camera's direction for a box flatter than [`FLAT_SHARE`] of its
/// longer side: 42 degrees above the ground.
const VIEW_DIR_FLAT: [f64; 3] = [0.0, 3.6, 4.0];
const FLAT_SHARE: f64 = 0.05;
/// The timed run: empty frames from `BASE_FROM` seconds, the effect at
/// `BASE_FROM + GAP`, its first `WARM` ticks (one-off uploads) not counted.
const BASE_FROM: f64 = 0.4;
const GAP: f64 = 0.8;
const WARM: f64 = 5.0;
/// Metres between the effects of `--together`.
const SPACING: f64 = 4.0;

struct Args {
    effects: Vec<String>,
    project: String,
    out: Option<String>,
    size: (u32, u32),
    cost: bool,
    cost_only: bool,
    json: bool,
    /// The graphics preset the stage applies ("": none).
    quality: String,
    /// The shown stage: "dusk" or "day".
    stage: String,
    keep: bool,
    together: bool,
    project_look: bool,
    look: String,
    scene: String,
    position: [f64; 3],
    camera: Option<[f64; 7]>,
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let mut a = Args {
        effects: Vec::new(),
        project: ".".into(),
        out: None,
        size: (960, 540),
        cost: true,
        cost_only: false,
        json: false,
        quality: String::new(),
        stage: "dusk".into(),
        keep: false,
        together: false,
        project_look: false,
        look: String::new(),
        scene: String::new(),
        position: [0.0; 3],
        camera: None,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => a.json = true,
            "--no-cost" => a.cost = false,
            "--cost-only" => a.cost_only = true,
            "--keep" => a.keep = true,
            "--together" => a.together = true,
            "--project-look" => a.project_look = true,
            flag @ ("--project" | "--out" | "--size" | "--quality" | "--stage" | "--look"
            | "--scene" | "--position" | "--camera") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--project" => a.project = v.to_string(),
                    "--out" => a.out = Some(v.to_string()),
                    "--look" => a.look = v.to_string(),
                    "--scene" => a.scene = v.to_string(),
                    "--position" => a.position = coordinates(v)?,
                    "--camera" => {
                        let camera = coordinates::<7>(v)?;
                        if !(0.0 < camera[6] && camera[6] < 180.0) || camera[..3] == camera[3..6] {
                            return Err(
                                "--camera needs distinct position/target and fov >0..<180 degrees"
                                    .into(),
                            );
                        }
                        a.camera = Some(camera);
                    }
                    "--quality" if matches!(*v, "low" | "medium" | "high") => {
                        a.quality = v.to_string()
                    }
                    "--quality" => return Err(format!("--quality {v}: low, medium or high")),
                    "--stage" if matches!(*v, "dusk" | "day") => a.stage = v.to_string(),
                    "--stage" => return Err(format!("--stage {v}: dusk or day")),
                    _ => {
                        let size = v
                            .split_once('x')
                            .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                            .filter(|&(w, h): &(u32, u32)| w >= 16 && h >= 16);
                        a.size = size.ok_or_else(|| format!("--size {v}: <width>x<height>"))?;
                    }
                }
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`. {USAGE}"));
            }
            other => a.effects.push(other.to_string()),
        }
        i += 1;
    }
    if a.effects.is_empty()
        || (a.cost_only && !a.cost)
        || (a.together && (a.cost_only || !a.cost))
        || (a.project_look && !a.look.is_empty())
    {
        return Err(USAGE.into());
    }
    Ok(a)
}

fn coordinates<const N: usize>(text: &str) -> Result<[f64; N], String> {
    text.split(',')
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .filter(|v| v.iter().all(|n| n.is_finite()))
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| format!("expected {N} comma-separated finite numbers, got '{text}'"))
}

/// One effect asked for.
struct Effect {
    /// Its asset id (`fx/hit.ogt`).
    id: String,
    /// `type vfx2d`: a pixel effect, played as a sprite.
    pixel: bool,
}

/// When an effect plays, from the timing run.
#[derive(Clone)]
struct Timing {
    duration: f64,
    looping: bool,
    /// Seconds from its start, when it sends one.
    impact: Option<f64>,
    /// Seconds it is shown and timed.
    show: f64,
    elements: Vec<String>,
}

/// What an effect was measured to cover: the box everything it drew
/// reached (a spark thrown far counts), and the core, the box without the
/// outer [`CORE_TRIM`] of its pixels on each side, which the shots frame.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Measured {
    reach: Bounds,
    core: Bounds,
    /// The share of its pixels below its origin, seen from the front.
    below: f64,
    /// The share of its shown time after which it drew nothing more over
    /// black (the first of [`SAMPLES`] past the last that showed it; 1 for
    /// one that showed to the end, and for a mark alone).
    busy: f64,
    /// It has a `decal`: the decal lands on the surface at its origin, so
    /// it stands there whatever else it draws below.
    marks: bool,
}

impl Measured {
    /// Its origin is inside it, not under it.
    fn centred(&self) -> bool {
        !self.marks && self.below >= CENTRE_SHARE
    }

    /// How far above the origin the shots play it: an effect anchored at
    /// its centre stands whole on the ground.
    fn lift(&self) -> f64 {
        if self.centred() {
            (-self.core.min[1]).max(0.0)
        } else {
            0.0
        }
    }
}

/// A box, metres from the effect's origin.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    min: [f64; 3],
    max: [f64; 3],
    /// It still reached the widest view's edge: the box is at least this.
    clipped: bool,
}

struct Stage<'a> {
    project: &'a Project,
    package: &'a Path,
    scratch: PathBuf,
    runs: std::cell::Cell<u32>,
    /// The graphics preset applied at the start ("": none).
    quality: &'a str,
    /// Leave each run's outputs in the scratch directory.
    keep: bool,
    context: Value,
}

impl Stage<'_> {
    fn done(&self, run: &Path) {
        if !self.keep {
            let _ = std::fs::remove_dir_all(run);
        }
    }
}

impl Stage<'_> {
    /// The stage bundled with `antialias` for effects of `dim`.
    fn bundle(
        &self,
        dim: &str,
        pixel: &[&str],
        size: (u32, u32),
        antialias: &str,
    ) -> Result<project::NativeBundle, String> {
        let entry = self.scratch.join(format!("main-{antialias}.ts"));
        let text = STAGE
            .replace("__DIM__", dim)
            .replace("__VFX2D__", &json!(pixel).to_string())
            .replace(
                "__PIVOTS__",
                &pixel_pivots(&self.project.root.join("assets"), pixel)?.to_string(),
            )
            .replace("__W__", &size.0.to_string())
            .replace("__H__", &size.1.to_string())
            .replace(
                "__CONTEXT__",
                &if antialias == "off" {
                    json!({})
                } else {
                    self.context.clone()
                }
                .to_string(),
            )
            .replace("__QUALITY__", self.quality)
            .replace("__ANTIALIAS__", antialias);
        std::fs::write(&entry, text).map_err(|e| format!("{}: {e}", entry.display()))?;
        project::bundle_native(
            &Project {
                entry,
                ..self.project.clone()
            },
            self.package,
        )
    }

    /// One run of the stage: its commands, then `quit` at `seconds`; the
    /// run's summary and the directory its outputs are in.
    fn run(
        &self,
        bundle: &Path,
        size: (u32, u32),
        mut commands: Vec<String>,
        seconds: f64,
        extra: &[&str],
    ) -> Result<(Value, PathBuf), String> {
        let n = self.runs.get();
        self.runs.set(n + 1);
        let out = self.scratch.join(format!("run-{n:03}"));
        commands.push(format!("{seconds:.3}:quit"));
        let output = project::bun(self.package, "src/node/run.ts")
            .arg("--entry")
            .arg(bundle)
            .arg("--root")
            .arg(&self.project.root)
            .arg("--project")
            .arg(&self.scratch)
            .arg("--size")
            .arg(format!("{}x{}", size.0, size.1))
            .arg("--json")
            .arg("--commands")
            .arg(commands.join(";"))
            .arg("--out")
            .arg(&out)
            .args(extra)
            .output()
            .map_err(|e| format!("{} ({e})", project::NO_BUN))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let summary = stdout
            .lines()
            .rev()
            .find(|l| l.starts_with('{'))
            .and_then(|l| serde_json::from_str::<Value>(l).ok());
        match summary {
            Some(s) if output.status.success() => Ok((s, out)),
            _ => {
                let err = String::from_utf8_lossy(&output.stderr);
                let tail: Vec<&str> = err.lines().rev().take(12).collect();
                Err(format!(
                    "the stage's run failed ({}):\n{}",
                    output.status,
                    tail.into_iter().rev().collect::<Vec<_>>().join("\n")
                ))
            }
        }
    }
}

/// Each `type vfx2d` effect's sheet pivot (a fraction from the bottom left),
/// so the stage centres the frame rather than the pivot: an effect that
/// stands on the ground has its pivot near the frame's foot.
fn pixel_pivots(assets: &Path, ids: &[&str]) -> Result<serde_json::Value, String> {
    let mut out = serde_json::Map::new();
    for id in ids {
        let text = std::fs::read_to_string(assets.join(id)).map_err(|e| format!("{id}: {e}"))?;
        let dir = Path::new(id).parent().unwrap_or(Path::new(""));
        let resolve = |p: &str| {
            let local = dir.join(p);
            if assets.join(&local).is_file() {
                local.to_string_lossy().replace('\\', "/")
            } else {
                p.to_string()
            }
        };
        let read = |f: &str| std::fs::read(assets.join(f)).map_err(|e| format!("{f}: {e}"));
        let (x, y) = og_cook::cook::vfx2d::pivot(&text, &resolve, &read)
            .map_err(|e| format!("{id}: {e}"))?;
        out.insert(id.to_string(), json!([x, y]));
    }
    Ok(serde_json::Value::Object(out))
}

/// The asset id an argument names: an id as given, or a file under the
/// project's `assets/`.
fn asset_id(root: &Path, arg: &str) -> Result<Effect, String> {
    let assets = root.join("assets");
    let file = [assets.join(arg), PathBuf::from(arg)]
        .into_iter()
        .find(|p| p.is_file())
        .ok_or_else(|| format!("{arg}: no such effect in {}", assets.display()))?;
    let abs = file.canonicalize().map_err(|e| format!("{arg}: {e}"))?;
    let base = assets.canonicalize().map_err(|e| format!("{arg}: {e}"))?;
    let id = abs
        .strip_prefix(&base)
        .map_err(|_| format!("{arg}: not under {}", assets.display()))?
        .to_string_lossy()
        .replace('\\', "/");
    let text = std::fs::read_to_string(&abs).map_err(|e| format!("{arg}: {e}"))?;
    let ty = text
        .lines()
        .find_map(|l| l.strip_prefix("type "))
        .map(str::trim)
        .unwrap_or_default();
    match ty {
        "vfx" => Ok(Effect { id, pixel: false }),
        "vfx2d" => Ok(Effect { id, pixel: true }),
        other => Err(format!(
            "{arg}: a `type {other}` file; a preview plays a `type vfx` or `type vfx2d` effect (an emitter or an effect material is previewed through a vfx that names it)"
        )),
    }
}

/// Step 1: every effect's timing from one run without a renderer.
fn timings(stage: &Stage, bundle: &Path, effects: &[Effect]) -> Result<Vec<Timing>, String> {
    let mut commands = Vec::new();
    let mut t = 0.5;
    for e in effects {
        commands.push(format!("{t:.3}:play:{}", e.id));
        t += SLOT;
    }
    commands.push(format!("{t:.3}:clear"));
    let (summary, _) = stage.run(bundle, (64, 64), commands, t + 0.2, &["--no-render"])?;
    let facts = &summary["facts"];
    let none = Vec::new();
    let starts = facts["starts"].as_array().unwrap_or(&none);
    let log = facts["log"].as_array().unwrap_or(&none);
    effects
        .iter()
        .map(|e| {
            let s = starts
                .iter()
                .find(|s| s["path"] == e.id.as_str())
                .ok_or_else(|| format!("{}: never started", e.id))?;
            let start = s["tick"].as_f64().unwrap_or(0.0);
            let duration = s["duration"].as_f64().unwrap_or(0.0);
            let looping = s["loop"] == true;
            let impact = log
                .iter()
                .filter(|l| l["name"] == "impact")
                .filter_map(|l| l["tick"].as_f64())
                .find(|&t| t >= start && t <= start + SLOT * TICK)
                // Read the tick after it was sent.
                .map(|t| ((t - start - 1.0) / TICK).max(0.0));
            Ok(Timing {
                duration,
                looping,
                impact,
                show: if looping {
                    LOOP_SHOW
                } else {
                    duration.clamp(0.2, SLOT - 0.1)
                },
                elements: s["elements"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect()
}

/// Writes a sheet, packed as small as PNG allows: it is written once and
/// kept (a pack pins one per effect), and the encoder's default leaves a
/// sheet several times larger.
fn save(sheet: RgbaImage, path: &Path) -> Result<(), String> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let encoder = PngEncoder::new_with_quality(
        std::io::BufWriter::new(file),
        CompressionType::Best,
        FilterType::Adaptive,
    );
    image::DynamicImage::ImageRgba8(sheet)
        .to_rgb8()
        .write_with_encoder(encoder)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn open(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|i| i.to_rgba8())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The shot camera for a box: the centre it looks at and how far it stands.
fn framing(b: Option<Bounds>) -> ([f64; 3], f64) {
    let (centre, radius) = match b {
        None => (DEFAULT_CENTRE, DEFAULT_RADIUS),
        Some(b) => {
            let c: [f64; 3] = std::array::from_fn(|i| (b.min[i] + b.max[i]) / 2.0);
            let r = (0..3)
                .map(|i| (b.max[i] - b.min[i]).powi(2))
                .sum::<f64>()
                .sqrt()
                / 2.0;
            (c, r.max(MIN_RADIUS))
        }
    };
    // The box's sphere inside the vertical field of view, with a margin.
    let distance = radius / (VIEW_FOV / 2.0).to_radians().sin() * 1.1;
    (centre, distance)
}

fn view_command(b: Option<Bounds>, lift: f64) -> String {
    let (mut c, distance) = framing(b);
    c[1] += lift;
    // A mark on the ground (a decal, a ground ring) is looked down on: from
    // the side it is a line.
    let flat = b.is_some_and(|b| {
        let side = (b.max[0] - b.min[0]).max(b.max[2] - b.min[2]);
        b.max[1] - b.min[1] < side * FLAT_SHARE
    });
    let dir = if flat { VIEW_DIR_FLAT } else { VIEW_DIR };
    let len = dir.iter().map(|v| v * v).sum::<f64>().sqrt();
    let p: [f64; 3] = std::array::from_fn(|i| c[i] + dir[i] / len * distance);
    format!(
        "eye:{:.4}:{:.4}:{:.4}:{:.4}:{:.4}:{:.4}:{VIEW_FOV}",
        p[0], p[1], p[2], c[0], c[1], c[2]
    )
}

/// What the six shots of one effect gave.
struct Shots {
    sheet: PathBuf,
    draws: u64,
    particles: u64,
    /// The engine's warnings of the run that name the effect.
    warnings: Vec<String>,
    /// Seconds from the effect's start to each frame.
    times: Vec<f64>,
}

/// Step 3: the six shots of each effect, tiled into its sheet under `out`.
fn shots(
    stage: &Stage,
    bundle: &Path,
    size: (u32, u32),
    effects: &[Effect],
    timing: &[Timing],
    boxes: &[Option<Measured>],
    out: &Path,
) -> Result<Vec<Shots>, String> {
    let mut done = Vec::new();
    let all: Vec<usize> = (0..effects.len()).collect();
    for chunk in all.chunks(CHUNK) {
        let mut commands = vec!["0:stage:show".to_string()];
        let mut t = 0.5;
        // Each effect's frame times, in the chunk's order.
        let mut taken = Vec::new();
        for &i in chunk {
            let tm = &timing[i];
            let start = t + 0.2;
            // An effect that leaves a mark lasts as long as the mark: its
            // shots are spread over the time it drew anything else and a
            // quarter more, so the last shows what is left.
            let span = match boxes[i] {
                Some(m) if !tm.looping && tm.elements.iter().any(|e| e == "decal") => {
                    (m.busy * MARK_TAIL).min(1.0) * tm.show
                }
                _ => tm.show,
            };
            let mut times: Vec<f64> = FRACTIONS.iter().map(|f| f * span).collect();
            if let Some(impact) = tm.impact {
                // The shot nearest the impact is taken two ticks after it, when
                // what it started shows.
                let j = (0..times.len())
                    .min_by(|&a, &b| {
                        (times[a] - impact)
                            .abs()
                            .total_cmp(&(times[b] - impact).abs())
                    })
                    .unwrap_or(0);
                times[j] = impact + 2.0 / TICK;
                times.sort_by(f64::total_cmp);
            }
            let lift = if stage.context["enabled"] == true {
                0.0
            } else {
                boxes[i].map_or(0.0, |m| m.lift())
            };
            commands.push(format!(
                "{:.3}:{}",
                t + 0.1,
                view_command(boxes[i].map(|m| m.core), lift)
            ));
            commands.push(format!("{:.3}:lift:{lift:.4}", t + 0.1));
            commands.push(format!("{start:.3}:play:{}", effects[i].id));
            for dt in &mut times {
                *dt = dt.max(1.0 / TICK);
            }
            for (n, dt) in times.iter().enumerate() {
                let at = start + dt;
                commands.push(format!("{at:.3}:screenshot:s{i}-{n}.png"));
                commands.push(format!("{at:.3}:dump_render:s{i}-{n}.json"));
            }
            t = start + tm.show;
            commands.push(format!("{t:.3}:clear"));
            taken.push(times);
        }
        let (summary, dir) = stage.run(bundle, size, commands, t + 0.3, &[])?;
        // "WARN tick N: <text>", the run's log; each effect keeps the
        // texts that quote its id.
        let logged: Vec<&str> = summary["warnings"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|l| l.split_once(": ").map_or(l, |(_, text)| text))
            .collect();
        for (&i, times) in chunk.iter().zip(taken) {
            let quoted = format!("'{}'", effects[i].id);
            let mut warnings: Vec<String> = logged
                .iter()
                .filter(|w| w.contains(&quoted))
                .map(|w| w.to_string())
                .collect();
            warnings.dedup();
            let mut sheet = RgbaImage::new(size.0 * 3, size.1 * 2);
            let (mut draws, mut particles) = (0, 0);
            for n in 0..FRACTIONS.len() as u32 {
                let frame = open(&dir.join(format!("s{i}-{n}.png")))?;
                image::imageops::overlay(
                    &mut sheet,
                    &frame,
                    i64::from((n % 3) * size.0),
                    i64::from((n / 3) * size.1),
                );
                let dump = dir.join(format!("s{i}-{n}.json"));
                let dump: Value = std::fs::read(&dump)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .ok_or_else(|| format!("{}: no render dump", dump.display()))?;
                draws = draws.max(dump["draws"].as_u64().unwrap_or(0));
                let p = &dump["particles"];
                particles = particles
                    .max(p["capacity"].as_u64().unwrap_or(0) + p["cpu"].as_u64().unwrap_or(0));
            }
            let path = out.join(Path::new(&effects[i].id).with_extension("png"));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: {e}", parent.display()))?;
            }
            save(sheet, &path)?;
            done.push(Shots {
                sheet: path,
                draws,
                particles,
                warnings,
                times,
            });
        }
        stage.done(&dir);
    }
    Ok(done)
}

/// A rendered frame of a trace: its tick, its milliseconds and each
/// pass's, on the GPU ([`gpu_frames`]) or the CPU ([`cpu_frames`]).
type GpuFrame = (f64, f64, Vec<(String, f64)>);

/// The trace's rendered frames as the CPU spent them: recording and
/// submitting each frame and each of its passes (the draws a pass issues
/// are its CPU time).
fn cpu_frames(trace: &Path) -> Result<Vec<GpuFrame>, String> {
    let text = std::fs::read(trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let doc: Value =
        serde_json::from_slice(&text).map_err(|e| format!("{}: {e}", trace.display()))?;
    let none = Vec::new();
    let events = doc["traceEvents"]
        .as_array()
        .or(doc.as_array())
        .unwrap_or(&none);
    let mut frames: std::collections::BTreeMap<u64, (f64, Vec<(String, f64)>)> = Default::default();
    for e in events.iter().filter(|e| e["ph"] == "X") {
        let (Some(tick), Some(name)) = (e["args"]["tick"].as_u64(), e["name"].as_str()) else {
            continue;
        };
        let ms = e["dur"].as_f64().unwrap_or(0.0) / 1000.0;
        let slot = frames.entry(tick).or_default();
        if e["cat"] == "render" && name.starts_with("frame ") {
            slot.0 += ms;
        } else if e["cat"] == "pass" {
            match slot.1.iter_mut().find(|p| p.0 == name) {
                Some(p) => p.1 += ms,
                None => slot.1.push((name.to_string(), ms)),
            }
        }
    }
    Ok(frames
        .into_iter()
        .filter(|(_, f)| f.0 > 0.0)
        .map(|(tick, (ms, passes))| (tick as f64, ms, passes))
        .collect())
}

fn gpu_frames(trace: &Path) -> Result<Vec<GpuFrame>, String> {
    let text = std::fs::read(trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let doc: Value =
        serde_json::from_slice(&text).map_err(|e| format!("{}: {e}", trace.display()))?;
    let none = Vec::new();
    let events = doc["traceEvents"]
        .as_array()
        .or(doc.as_array())
        .unwrap_or(&none);
    let mut renders: Vec<(f64, f64)> = events
        .iter()
        .filter(|e| {
            e["ph"] == "X"
                && e["cat"] == "render"
                && e["name"].as_str().is_some_and(|n| n.starts_with("frame "))
        })
        .map(|e| {
            (
                e["ts"].as_f64().unwrap_or(0.0),
                e["args"]["tick"].as_f64().unwrap_or(0.0),
            )
        })
        .collect();
    renders.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut frames: std::collections::BTreeMap<u64, (f64, Vec<(String, f64)>)> = Default::default();
    for e in events {
        if e["cat"] != "gpu" || e["ph"] != "X" {
            continue;
        }
        let Some(f) = e["args"]["frame"].as_u64() else {
            continue;
        };
        let ms = e["dur"].as_f64().unwrap_or(0.0) / 1000.0;
        let name = e["name"].as_str().unwrap_or_default();
        let slot = frames.entry(f).or_default();
        if name.starts_with("gpu frame") {
            slot.0 = ms;
        } else if let Some(p) = slot.1.iter_mut().find(|p| p.0 == name) {
            p.1 += ms;
        } else {
            slot.1.push((name.to_string(), ms));
        }
    }
    Ok(frames
        .into_iter()
        .filter_map(|(f, (total, passes))| {
            renders
                .get(f as usize)
                .map(|&(_, tick)| (tick, total, passes))
        })
        .collect())
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    match v.len() {
        0 => 0.0,
        n if n % 2 == 1 => v[n / 2],
        n => (v[n / 2 - 1] + v[n / 2]) / 2.0,
    }
}

/// The 90th percentile: an effect's heavy moment without one frame's
/// scheduling noise.
fn p90(mut v: Vec<f64>, default: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(((0.9 * v.len() as f64) as usize).min(v.len().saturating_sub(1)))
        .copied()
        .unwrap_or(default)
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Step 4: one effect's cost from its own traced run.
fn cost(
    stage: &Stage,
    bundle: &Path,
    size: (u32, u32),
    effect: &Effect,
    timing: &Timing,
    b: Option<Bounds>,
    lift: f64,
) -> Result<Value, String> {
    let start = BASE_FROM + GAP;
    let end = start + timing.show;
    let trace = stage.scratch.join("trace.json");
    let _ = std::fs::remove_file(&trace);
    let commands = vec![
        "0:stage:show".to_string(),
        format!("0.1:{}", view_command(b, lift)),
        format!("0.1:lift:{lift:.4}"),
        format!("{start:.3}:play:{}", effect.id),
        format!("{end:.3}:clear"),
    ];
    let trace_arg = trace.to_string_lossy().into_owned();
    let (summary, dir) = stage.run(bundle, size, commands, end + 0.1, &["--trace", &trace_arg])?;
    stage.done(&dir);
    let frames = gpu_frames(&trace)?;
    let _ = std::fs::remove_file(&trace);
    let pass = |f: &GpuFrame, name: &str| f.2.iter().find(|p| p.0 == name).map_or(0.0, |p| p.1);
    let base: Vec<_> = frames
        .iter()
        .filter(|f| f.0 >= BASE_FROM * TICK && f.0 < start * TICK)
        .collect();
    let window: Vec<_> = frames
        .iter()
        .filter(|f| f.0 >= start * TICK + WARM && f.0 <= end * TICK)
        .collect();
    if base.is_empty() || window.is_empty() {
        return Err(format!(
            "{}: the trace has no GPU times (the renderer `{}` reports none)",
            effect.id,
            summary["renderer"].as_str().unwrap_or("?")
        ));
    }
    let base_total = median(base.iter().map(|f| f.1).collect());
    let base_main = median(base.iter().map(|f| pass(f, "main")).collect());
    Ok(json!({
        "renderer": summary["renderer"],
        "size": format!("{}x{}", size.0, size.1),
        "baselineMs": round3(base_total),
        "peakFrameMs": {
            "total": round3((p90(window.iter().map(|f| f.1).collect(), base_total) - base_total).max(0.0)),
            "sim": round3(p90(window.iter().map(|f| pass(f, "particles")).collect(), 0.0)),
            // The main pass draws the blended surfaces, sprites and
            // particles after the opaque scene: its growth is theirs.
            "transparent": round3((p90(window.iter().map(|f| pass(f, "main")).collect(), base_main) - base_main).max(0.0)),
        },
    }))
}

/// `--together`: every effect at once on a grid, one traced run.
fn together(
    stage: &Stage,
    bundle: &Path,
    size: (u32, u32),
    effects: &[Effect],
    timing: &[Timing],
    out: &Path,
) -> Result<Value, String> {
    let n = effects.len();
    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let place = |i: usize| {
        (
            ((i % cols) as f64 - (cols - 1) as f64 / 2.0) * SPACING,
            ((i / cols) as f64 - (rows - 1) as f64 / 2.0) * SPACING,
        )
    };
    let half = [cols as f64 * SPACING / 2.0, rows as f64 * SPACING / 2.0];
    let grid = Bounds {
        min: [-half[0], 0.0, -half[1]],
        max: [half[0], 2.0, half[1]],
        clipped: false,
    };
    let start = BASE_FROM + GAP;
    let show = timing.iter().map(|t| t.show).fold(0.2, f64::max);
    let end = start + show;
    let shots = [0.1, 0.4, 1.0].map(|t: f64| t.min(show));
    let mut commands = vec![
        "0:stage:show".to_string(),
        format!("0.1:{}", view_command(Some(grid), 0.0)),
    ];
    for (i, e) in effects.iter().enumerate() {
        let (x, z) = place(i);
        commands.push(format!("{start:.3}:add:{x}:{z}:{}", e.id));
    }
    for (k, t) in shots.iter().enumerate() {
        commands.push(format!("{:.3}:screenshot:together-{k}.png", start + t));
    }
    commands.push(format!("{end:.3}:clear"));
    let trace = stage.scratch.join("trace.json");
    let _ = std::fs::remove_file(&trace);
    let trace_arg = trace.to_string_lossy().into_owned();
    let (summary, dir) = stage.run(bundle, size, commands, end + 0.1, &["--trace", &trace_arg])?;
    let mut sheet = RgbaImage::new(size.0 * 3, size.1);
    for k in 0..shots.len() {
        let frame = open(&dir.join(format!("together-{k}.png")))?;
        image::imageops::overlay(&mut sheet, &frame, i64::from(k as u32 * size.0), 0);
    }
    let sheet_path = out.join("together.png");
    save(sheet, &sheet_path)?;
    stage.done(&dir);
    let gpu = gpu_frames(&trace)?;
    let cpu = cpu_frames(&trace)?;
    let _ = std::fs::remove_file(&trace);
    // Each clock's frame, then each pass the effects grew by a microsecond
    // or more.
    let growth = |frames: &[GpuFrame]| {
        let base: Vec<_> = frames
            .iter()
            .filter(|f| f.0 >= BASE_FROM * TICK && f.0 < start * TICK)
            .collect();
        let window: Vec<_> = frames
            .iter()
            .filter(|f| f.0 >= start * TICK + WARM && f.0 <= end * TICK)
            .collect();
        if base.is_empty() || window.is_empty() {
            return None;
        }
        let row = |of: &dyn Fn(&GpuFrame) -> f64| {
            let was = median(base.iter().map(|f| of(f)).collect());
            let all: Vec<f64> = window.iter().map(|f| of(f)).collect();
            json!({
                "emptyMs": round3(was),
                "peakMs": round3((p90(all.clone(), was) - was).max(0.0)),
                "typicalMs": round3((median(all) - was).max(0.0)),
            })
        };
        let mut names: Vec<&str> = Vec::new();
        for f in base.iter().chain(&window) {
            for (name, _) in &f.2 {
                if !names.contains(&name.as_str()) {
                    names.push(name);
                }
            }
        }
        let mut passes = serde_json::Map::new();
        for name in names {
            let r = row(&|f: &GpuFrame| f.2.iter().find(|p| p.0 == name).map_or(0.0, |p| p.1));
            if r["peakMs"].as_f64().unwrap_or(0.0) >= 0.001 {
                passes.insert(name.to_string(), r);
            }
        }
        Some((row(&|f: &GpuFrame| f.1), passes))
    };
    let Some((frame, passes)) = growth(&gpu) else {
        return Err(format!(
            "the trace has no GPU times (the renderer `{}` reports none)",
            summary["renderer"].as_str().unwrap_or("?")
        ));
    };
    let (cpu_frame, cpu_passes) = growth(&cpu).ok_or("the trace has no rendered frames")?;
    Ok(json!({
        "effects": effects.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        "spacing": SPACING,
        "context": stage.context,
        "renderer": summary["renderer"],
        "size": format!("{}x{}", size.0, size.1),
        "shownSeconds": round3(show),
        "sheet": sheet_path,
        "frame": frame,
        "passes": passes,
        "cpu": { "frame": cpu_frame, "passes": cpu_passes },
    }))
}

fn round2(v: [f64; 3]) -> [f64; 3] {
    v.map(|x| (x * 100.0).round() / 100.0)
}

/// The record's core back as a box.
fn core_of(record: &Value) -> Option<Bounds> {
    let v = |k: &str| -> Option<[f64; 3]> {
        let a = record["core"][k].as_array()?;
        Some([
            a.first()?.as_f64()?,
            a.get(1)?.as_f64()?,
            a.get(2)?.as_f64()?,
        ])
    };
    Some(Bounds {
        min: v("min")?,
        max: v("max")?,
        clipped: false,
    })
}

fn go(args: &[&str]) -> Result<Vec<Value>, (u8, String)> {
    let a = parse(args).map_err(|e| (2, e))?;
    let fail = |e: String| (1, e);
    let p = Project::open(Path::new(&a.project)).map_err(fail)?;
    if !p.manifest {
        return Err((
            1,
            format!(
                "{}: no og-project.json; a preview plays an effect of a project's assets (--project DIR)",
                p.root.display()
            ),
        ));
    }
    let package =
        project::engine_package(Some(&p.root)).ok_or((1, project::NO_ENGINE.to_string()))?;
    let effects: Vec<Effect> = a
        .effects
        .iter()
        .map(|e| asset_id(&p.root, e))
        .collect::<Result<_, _>>()
        .map_err(fail)?;
    let pixel: Vec<&str> = effects
        .iter()
        .filter(|e| e.pixel)
        .map(|e| e.id.as_str())
        .collect();
    if !pixel.is_empty() && pixel.len() != effects.len() {
        return Err((2, "preview `type vfx` and `type vfx2d` effects in separate calls: one stage is 3D, the other 2D".into()));
    }
    let dim = if pixel.is_empty() { "3d" } else { "2d" };
    if dim == "2d"
        && (a.project_look
            || !a.look.is_empty()
            || !a.scene.is_empty()
            || a.camera.is_some()
            || a.position != [0.0; 3])
    {
        return Err((
            2,
            "look/scene/position/camera preview options currently require type vfx (3D)".into(),
        ));
    }
    let out = a
        .out
        .as_ref()
        .map(|o| std::path::absolute(o).unwrap_or_else(|_| PathBuf::from(o)))
        .unwrap_or_else(|| p.root.join("og-out/vfx-preview"));
    std::fs::create_dir_all(&out).map_err(|e| fail(format!("{}: {e}", out.display())))?;
    let record_path = |e: &Effect| out.join(Path::new(&e.id).with_extension("json"));
    if p.has_assets() {
        let report = crate::cook::cook(&p.root, 0, false).map_err(fail)?;
        if report
            .assets
            .iter()
            .flat_map(|a| a.outputs.iter())
            .any(|o| o.cooked)
        {
            eprintln!("{}", crate::cook::summary(&report));
        }
    }
    let scratch = p.work_dir().join("vfx-preview");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).map_err(|e| fail(format!("{}: {e}", scratch.display())))?;
    // Runtime metadata is deliberately isolated from the project's boot
    // scene, seed and look. Assets still come from the project's bundle.
    std::fs::write(
        scratch.join("og-project.json"),
        json!({"version": 1, "engine": p.engine, "name": "VFX preview"}).to_string(),
    )
    .map_err(|e| fail(e.to_string()))?;
    std::fs::write(scratch.join("entry.ts"), "export {};\n").map_err(|e| fail(e.to_string()))?;
    std::fs::write(scratch.join("project.ts"), format!(
        "import {{ defineProject }} from '@origingame/engine/author';\nexport default defineProject({{id:'vfx-preview',entry:'entry.ts',create:{{form:'{dim}'}}}});\n"
    )).map_err(|e| fail(e.to_string()))?;
    let look = if a.project_look {
        crate::project::snapshot(&p.root).map_err(fail)?["startup"]["look"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| {
                (
                    2,
                    "--project-look requires og-project.json's look declaration".into(),
                )
            })?
    } else {
        a.look.clone()
    };
    let context = json!({
        "enabled": !look.is_empty() || !a.scene.is_empty() || a.camera.is_some() || a.position != [0.0; 3],
        "look": look, "scene": a.scene, "position": a.position, "camera": a.camera,
        "stage": a.stage,
    });
    let stage = Stage {
        project: &p,
        package: &package,
        scratch: scratch.clone(),
        runs: Default::default(),
        quality: &a.quality,
        keep: a.keep,
        context,
    };
    let show = stage.bundle(dim, &pixel, a.size, "tier").map_err(fail)?;
    if a.together {
        if dim == "2d" {
            return Err((
                2,
                "--together places 3D effects (type vfx) on a grid".into(),
            ));
        }
        let timing = timings(&stage, &show, &effects).map_err(fail)?;
        let record = together(&stage, &show, a.size, &effects, &timing, &out).map_err(fail)?;
        let path = out.join("together.json");
        std::fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&record).unwrap_or_default()
            ),
        )
        .map_err(|e| fail(format!("{}: {e}", path.display())))?;
        drop(show);
        if !a.keep {
            let _ = std::fs::remove_dir_all(&scratch);
        }
        if !a.json {
            let f = &record["frame"];
            println!(
                "{} effects at once, {} m apart, at {} on {}: GPU +{} ms at the peak, +{} ms typical, over an empty frame of {} ms",
                effects.len(),
                SPACING,
                record["size"].as_str().unwrap_or_default(),
                record["renderer"].as_str().unwrap_or("?"),
                f["peakMs"],
                f["typicalMs"],
                f["emptyMs"]
            );
            for (name, r) in record["passes"].as_object().into_iter().flatten() {
                println!(
                    "  {name}: +{} ms peak, +{} ms typical (empty {} ms)",
                    r["peakMs"], r["typicalMs"], r["emptyMs"]
                );
            }
            println!("  sheet {}", record["sheet"].as_str().unwrap_or_default());
        }
        return Ok(vec![record]);
    }
    let mut records: Vec<Value> = Vec::new();
    if a.cost_only {
        for e in &effects {
            let path = record_path(e);
            let record = std::fs::read(&path)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .ok_or_else(|| fail(format!("{}: --cost-only reads an earlier preview's record; run `og vfx preview {} --no-cost` with the same --out first", path.display(), e.id)))?;
            if record["context"] != stage.context {
                return Err((2, "--cost-only requires the same look, scene, position and camera as the recorded preview".into()));
            }
            records.push(record);
        }
    } else {
        let timing = timings(&stage, &show, &effects).map_err(fail)?;
        let boxes = if dim == "3d" {
            let measure = stage.bundle(dim, &pixel, a.size, "off").map_err(fail)?;
            bounds(&stage, &measure, &effects, &timing).map_err(fail)?
        } else {
            vec![None; effects.len()]
        };
        let sheets = shots(&stage, &show, a.size, &effects, &timing, &boxes, &out).map_err(fail)?;
        for (((e, tm), b), s) in effects.iter().zip(&timing).zip(&boxes).zip(&sheets) {
            let core = b.map(|m| json!({ "min": round2(m.core.min), "max": round2(m.core.max) }));
            let bounds = b.map(|m| {
                let b = m.reach;
                let size: [f64; 3] = std::array::from_fn(|i| b.max[i] - b.min[i]);
                json!({
                    "min": round2(b.min), "max": round2(b.max), "size": round2(size),
                    "anchor": if m.centred() { "centre" } else { "ground" },
                    "belowOrigin": (m.below * 100.0).round() / 100.0,
                    "clipped": b.clipped,
                })
            });
            records.push(json!({
                "effect": e.id,
                "type": if e.pixel { "vfx2d" } else { "vfx" },
                "sheet": s.sheet,
                "durationSeconds": round3(tm.duration),
                "loop": tm.looping,
                "impactSeconds": tm.impact.map(round3),
                "shownSeconds": round3(tm.show),
                "frameSeconds": s.times.iter().copied().map(round3).collect::<Vec<_>>(),
                "elements": tm.elements,
                "bounds": bounds,
                "core": core,
                "raised": if stage.context["enabled"] == true { 0.0 } else { b.map_or(0.0, |m| (m.lift() * 100.0).round() / 100.0) },
                "context": stage.context,
                "draws": s.draws,
                "peakParticles": s.particles,
                "warnings": s.warnings,
                "cost": Value::Null,
            }));
        }
    }
    if a.cost {
        for (e, record) in effects.iter().zip(&mut records) {
            let timing = Timing {
                duration: record["durationSeconds"].as_f64().unwrap_or(0.0),
                looping: record["loop"] == true,
                impact: record["impactSeconds"].as_f64(),
                show: record["shownSeconds"].as_f64().unwrap_or(LOOP_SHOW),
                elements: Vec::new(),
            };
            let lift = record["raised"].as_f64().unwrap_or(0.0);
            record["cost"] =
                cost(&stage, &show, a.size, e, &timing, core_of(record), lift).map_err(fail)?;
        }
    }
    for (e, record) in effects.iter().zip(&records) {
        let path = record_path(e);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| fail(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(record).unwrap_or_default()
            ),
        )
        .map_err(|e| fail(format!("{}: {e}", path.display())))?;
    }
    drop(show);
    if !a.keep {
        let _ = std::fs::remove_dir_all(&scratch);
    }
    if !a.json {
        for r in &records {
            let b = &r["bounds"];
            let size = if r["type"] == "vfx2d" {
                "a pixel effect, drawn at whole screen pixels".to_string()
            } else if b.is_null() {
                "nothing measurable".to_string()
            } else {
                format!(
                    "{} x {} x {} m{}, {}",
                    b["size"][0],
                    b["size"][1],
                    b["size"][2],
                    if b["clipped"] == true {
                        " at least"
                    } else {
                        ""
                    },
                    if b["anchor"] == "centre" {
                        "about its origin (anchor centre)"
                    } else {
                        "standing on its origin (anchor ground)"
                    }
                )
            };
            let length = if r["loop"] == true {
                "loops".to_string()
            } else {
                format!("{} s", r["durationSeconds"])
            };
            let impact = r["impactSeconds"]
                .as_f64()
                .map_or(String::new(), |t| format!(", impact at {t} s"));
            println!(
                "{}: {length}{impact}; {size}",
                r["effect"].as_str().unwrap_or_default()
            );
            println!(
                "  draws {}, particle slots {}",
                r["draws"], r["peakParticles"]
            );
            if let Some(c) = r["cost"].as_object() {
                let ms = &c["peakFrameMs"];
                println!(
                    "  GPU +{} ms at its peak over an empty frame of {} ms (simulation {}, blended pass +{}) at {} on {}",
                    ms["total"],
                    c["baselineMs"],
                    ms["sim"],
                    ms["transparent"],
                    c["size"].as_str().unwrap_or_default(),
                    c["renderer"].as_str().unwrap_or("?")
                );
            }
            for w in r["warnings"].as_array().into_iter().flatten() {
                println!("  warning: {}", w.as_str().unwrap_or_default());
            }
            println!("  sheet {}", r["sheet"].as_str().unwrap_or_default());
        }
    }
    Ok(records)
}

pub fn run(args: &[&str]) -> ExitCode {
    let json = args.contains(&"--json");
    match go(args) {
        Ok(records) => {
            if json {
                println!("{}", Value::Array(records));
            }
            ExitCode::SUCCESS
        }
        Err((code, e)) => {
            eprintln!("og vfx preview: {e}");
            ExitCode::from(code)
        }
    }
}
