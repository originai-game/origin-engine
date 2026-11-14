//! `og vfx preview <effect.ogt>... [--project DIR] [--out DIR] [--size WxH]
//! [--decal-atlas <texture>] [--no-cost | --cost-only] [--keep] [--json]`: what an effect looks like, how big
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
//!    any other on the `ground`;
//! 3. **shots**: six frames in time order, the impact frame among them, the
//!    camera placed so the measured core fills the view (an effect that
//!    draws nothing measurable gets the default view) and an effect
//!    anchored at its centre raised to stand whole on the ground, each with the
//!    render dump's draw calls and particle slots, tiled 3 x 2 into
//!    `<out>/<effect>.png`;
//! 4. **cost**, one short traced run per effect: the 90th percentile of its
//!    frames' GPU time over the empty frames before it in the same run,
//!    with the simulation pass and the main pass's growth. Framed as the
//!    shots are, so an effect is timed filling the view. `--no-cost` skips
//!    it; `--cost-only` does nothing else and reads the rest from an
//!    earlier preview in `--out` (a pack's previews take their shots
//!    holding one GPU slot and their timings holding all of them).
//!
//! `--decal-atlas textures/atlas.png` is the texture the stage sets as
//! og.RenderSettings.decalAtlas, as the game does, for effects with `decal`
//! elements (without one a decal draws white). `--keep` leaves the stage's
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

const USAGE: &str = "usage: og vfx preview <effect.ogt>... [--project DIR] [--out DIR] [--size WxH] [--decal-atlas <texture>] [--no-cost | --cost-only] [--keep] [--json]";

const STAGE: &str = include_str!("vfx_preview/stage.ts");
const TICK: f64 = 60.0;
/// Seconds between two effects' starts in the timing run: an effect's
/// events are looked for this long.
const SLOT: f64 = 4.0;
/// Seconds a looping effect is shown.
const LOOP_SHOW: f64 = 2.0;
/// Where in an effect's shown time its six shots are taken.
const FRACTIONS: [f64; 6] = [0.04, 0.15, 0.3, 0.5, 0.7, 0.9];
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
/// front, is anchored at its centre.
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
/// The timed run: empty frames from `BASE_FROM` seconds, the effect at
/// `BASE_FROM + GAP`, its first `WARM` ticks (one-off uploads) not counted.
const BASE_FROM: f64 = 0.4;
const GAP: f64 = 0.8;
const WARM: f64 = 5.0;

struct Args {
    effects: Vec<String>,
    project: String,
    out: Option<String>,
    size: (u32, u32),
    cost: bool,
    cost_only: bool,
    json: bool,
    decal_atlas: String,
    keep: bool,
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
        decal_atlas: String::new(),
        keep: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => a.json = true,
            "--no-cost" => a.cost = false,
            "--cost-only" => a.cost_only = true,
            "--keep" => a.keep = true,
            flag @ ("--project" | "--out" | "--size" | "--decal-atlas") => {
                let Some(v) = args.get(i + 1) else {
                    return Err(format!("{flag} needs a value. {USAGE}"));
                };
                match flag {
                    "--project" => a.project = v.to_string(),
                    "--out" => a.out = Some(v.to_string()),
                    "--decal-atlas" => a.decal_atlas = v.to_string(),
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
    if a.effects.is_empty() || (a.cost_only && !a.cost) {
        return Err(USAGE.into());
    }
    Ok(a)
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
}

impl Measured {
    /// Its origin is inside it, not under it.
    fn centred(&self) -> bool {
        self.below >= CENTRE_SHARE
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
    decal_atlas: &'a str,
    /// Leave each run's outputs in the scratch directory.
    keep: bool,
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
            .replace("__W__", &size.0.to_string())
            .replace("__H__", &size.1.to_string())
            .replace("__DECAL_ATLAS__", self.decal_atlas)
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

/// Where an effect showed in one measuring view, over every sample: how
/// many of its pixels fell in each column and each row.
struct Seen {
    cols: Vec<u64>,
    rows: Vec<u64>,
}

impl Seen {
    fn new() -> Seen {
        Seen {
            cols: vec![0; MEASURE_PX as usize],
            rows: vec![0; MEASURE_PX as usize],
        }
    }

    /// Counts the pixels of `frame` that differ from `empty`.
    fn add(&mut self, empty: &RgbaImage, frame: &RgbaImage) {
        for (x, y, p) in frame.enumerate_pixels() {
            let q = empty.get_pixel(x, y);
            if (0..3).any(|c| (i16::from(p[c]) - i16::from(q[c])).abs() > THRESHOLD) {
                self.cols[x as usize] += 1;
                self.rows[y as usize] += 1;
            }
        }
    }

    /// The pixel box (x0, y0, x1, y1, inclusive) holding all it showed but
    /// `trim` of its pixels at each end of each axis (0: everything), or
    /// None when it showed nothing.
    fn pixels(&self, trim: f64) -> Option<[u32; 4]> {
        let span = |counts: &[u64]| -> Option<(u32, u32)> {
            let total: u64 = counts.iter().sum();
            if total == 0 {
                return None;
            }
            let cut = (total as f64 * trim) as u64;
            let mut sum = 0;
            let lo = counts.iter().position(|&c| {
                sum += c;
                sum > cut
            })?;
            let mut sum = 0;
            let hi = counts.iter().rposition(|&c| {
                sum += c;
                sum > cut
            })?;
            Some((lo as u32, hi as u32))
        };
        let (x0, x1) = span(&self.cols)?;
        let (y0, y1) = span(&self.rows)?;
        Some([x0, y0, x1, y1])
    }

    /// The share of its pixels in the view's lower half.
    fn below(&self) -> f64 {
        let total: u64 = self.rows.iter().sum();
        let below: u64 = self.rows[self.rows.len() / 2..].iter().sum();
        if total == 0 {
            0.0
        } else {
            below as f64 / total as f64
        }
    }
}

fn open(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|i| i.to_rgba8())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The measuring views for a half side of `half` metres at the origin:
/// from the front, level with the ground (x right, y up, the origin at the
/// centre), and from above (x right, z down the image).
fn measure_views(half: f64) -> [String; 2] {
    let d = half * 40.0;
    let fov = 2.0 * (half / d).atan().to_degrees();
    [
        format!("eye:0:0:{d}:0:0:0:{fov}"),
        format!("eye:0:{d}:{}:0:0:0:{fov}", d * 1e-3),
    ]
}

/// The box in metres of the pixel boxes seen from the front and from above
/// with a half side of `half`, and whether either reached its view's edge.
fn metres(front: Option<[u32; 4]>, top: Option<[u32; 4]>, half: f64) -> Option<(Bounds, bool)> {
    let s = f64::from(MEASURE_PX);
    // A pixel column's or row's place across the view, -half to half.
    let at = |px: u32, edge: f64| ((f64::from(px) + edge) / s - 0.5) * 2.0 * half;
    let edge =
        |b: [u32; 4]| b[0] == 0 || b[1] == 0 || b[2] == MEASURE_PX - 1 || b[3] == MEASURE_PX - 1;
    let mut x: Option<(f64, f64)> = None;
    let mut widen = |lo: f64, hi: f64| {
        x = Some(x.map_or((lo, hi), |(a, b): (f64, f64)| (a.min(lo), b.max(hi))));
    };
    let mut y = (0.0, 0.0);
    let mut z = (0.0, 0.0);
    if let Some(f) = front {
        widen(at(f[0], 0.0), at(f[2], 1.0));
        y = (-at(f[3], 1.0), -at(f[1], 0.0));
    }
    if let Some(t) = top {
        widen(at(t[0], 0.0), at(t[2], 1.0));
        z = (at(t[1], 0.0), at(t[3], 1.0));
    }
    let (x0, x1) = x?;
    // Seen from above alone (a decal, a ground ring): flat on the ground.
    // Seen from the front alone: under the ground, its depth unknown.
    Some((
        Bounds {
            min: [x0, y.0, z.0],
            max: [x1, y.1, z.1],
            clipped: false,
        },
        front.is_some_and(edge) || top.is_some_and(edge),
    ))
}

/// Step 2: each effect's measured bounds (None for one that draws nothing
/// that shows over black).
fn bounds(
    stage: &Stage,
    bundle: &Path,
    effects: &[Effect],
    timing: &[Timing],
) -> Result<Vec<Option<Measured>>, String> {
    let mut out: Vec<Option<Measured>> = vec![None; effects.len()];
    let mut todo: Vec<usize> = (0..effects.len()).collect();
    let mut half = MEASURE_HALF;
    while !todo.is_empty() {
        let views = measure_views(half);
        let mut again = Vec::new();
        for chunk in todo.chunks(CHUNK) {
            let mut commands = vec!["0:stage:measure".to_string()];
            for (v, name) in views.iter().zip(["f", "t"]) {
                commands.push(format!("0.3:{v}"));
                commands.push(format!("0.3:screenshot:empty-{name}.png"));
            }
            let mut t = 0.5;
            for &i in chunk {
                commands.push(format!("{t:.3}:play:{}", effects[i].id));
                for (k, f) in SAMPLES.iter().enumerate() {
                    let at = t + (f * timing[i].show).max(1.0 / TICK);
                    for (v, name) in views.iter().zip(["f", "t"]) {
                        commands.push(format!("{at:.3}:{v}"));
                        commands.push(format!("{at:.3}:screenshot:m{i}-{name}{k}.png"));
                    }
                }
                t += timing[i].show;
                commands.push(format!("{t:.3}:clear"));
                t += 0.2;
            }
            let (_, dir) = stage.run(bundle, (MEASURE_PX, MEASURE_PX), commands, t + 0.1, &[])?;
            let empty = [
                open(&dir.join("empty-f.png"))?,
                open(&dir.join("empty-t.png"))?,
            ];
            for &i in chunk {
                let mut seen = [Seen::new(), Seen::new()];
                for k in 0..SAMPLES.len() {
                    for (v, name) in ["f", "t"].iter().enumerate() {
                        let frame = open(&dir.join(format!("m{i}-{name}{k}.png")))?;
                        seen[v].add(&empty[v], &frame);
                    }
                }
                let reach = metres(seen[0].pixels(0.0), seen[1].pixels(0.0), half);
                let core = metres(seen[0].pixels(CORE_TRIM), seen[1].pixels(CORE_TRIM), half);
                match reach.zip(core) {
                    None => out[i] = None,
                    Some(((b, edge), (core, _))) => {
                        let widest = half * 2.0 > MEASURE_HALF_MAX;
                        out[i] = Some(Measured {
                            reach: Bounds {
                                clipped: edge && widest,
                                ..b
                            },
                            core,
                            below: seen[0].below(),
                        });
                        if edge && !widest {
                            again.push(i);
                        }
                    }
                }
            }
            stage.done(&dir);
        }
        todo = again;
        half *= 2.0;
    }
    Ok(out)
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
    let len = VIEW_DIR.iter().map(|v| v * v).sum::<f64>().sqrt();
    let p: [f64; 3] = std::array::from_fn(|i| c[i] + VIEW_DIR[i] / len * distance);
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
        for &i in chunk {
            let tm = &timing[i];
            let start = t + 0.2;
            let mut times: Vec<f64> = FRACTIONS.iter().map(|f| f * tm.show).collect();
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
            let lift = boxes[i].map_or(0.0, |m| m.lift());
            commands.push(format!(
                "{:.3}:{}",
                t + 0.1,
                view_command(boxes[i].map(|m| m.core), lift)
            ));
            commands.push(format!("{:.3}:lift:{lift:.4}", t + 0.1));
            commands.push(format!("{start:.3}:play:{}", effects[i].id));
            for (n, dt) in times.iter().enumerate() {
                let at = start + dt.max(1.0 / TICK);
                commands.push(format!("{at:.3}:screenshot:s{i}-{n}.png"));
                commands.push(format!("{at:.3}:dump_render:s{i}-{n}.json"));
            }
            t = start + tm.show;
            commands.push(format!("{t:.3}:clear"));
        }
        let (_, dir) = stage.run(bundle, size, commands, t + 0.3, &[])?;
        for &i in chunk {
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
            image::DynamicImage::ImageRgba8(sheet)
                .to_rgb8()
                .save(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            done.push(Shots {
                sheet: path,
                draws,
                particles,
            });
        }
        stage.done(&dir);
    }
    Ok(done)
}

/// A rendered frame of a trace: its tick, its GPU milliseconds and each
/// pass's.
type GpuFrame = (f64, f64, Vec<(String, f64)>);

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
                "{}: no project.ogt; a preview plays an effect of a project's assets (--project DIR)",
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
    let stage = Stage {
        project: &p,
        package: &package,
        scratch: scratch.clone(),
        runs: Default::default(),
        decal_atlas: &a.decal_atlas,
        keep: a.keep,
    };
    let show = stage.bundle(dim, &pixel, a.size, "tier").map_err(fail)?;
    let mut records: Vec<Value> = Vec::new();
    if a.cost_only {
        for e in &effects {
            let path = record_path(e);
            let record = std::fs::read(&path)
                .ok()
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .ok_or_else(|| fail(format!("{}: --cost-only reads an earlier preview's record; run `og vfx preview {} --no-cost` with the same --out first", path.display(), e.id)))?;
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
                "elements": tm.elements,
                "bounds": bounds,
                "core": core,
                "raised": b.map_or(0.0, |m| (m.lift() * 100.0).round() / 100.0),
                "draws": s.draws,
                "peakParticles": s.particles,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_measured_box_is_the_changed_pixels_in_metres() {
        // A half side of 6 m over 768 px: 64 px a metre, the origin at the
        // front view's centre; an effect 2 m wide about x = 0 standing 3 m
        // tall on the ground, and 1 m deep from z = -0.5.
        let front = Some([384 - 64, 384 - 192, 384 + 63, 383]);
        let top = Some([384 - 64, 384 - 32, 384 + 63, 384 + 31]);
        let (b, edge) = metres(front, top, 6.0).unwrap();
        assert!(!edge);
        let near = |a: [f64; 3], e: [f64; 3]| (0..3).all(|i| (a[i] - e[i]).abs() < 1e-9);
        assert!(near(b.min, [-1.0, 0.0, -0.5]), "{b:?}");
        assert!(near(b.max, [1.0, 3.0, 0.5]), "{b:?}");
        // A ring of 1 m about the origin reaches as far below it as above.
        let ring = Some([384 - 64, 384 - 64, 384 + 63, 384 + 63]);
        let (b, _) = metres(ring, None, 6.0).unwrap();
        assert!(near(b.min, [-1.0, -1.0, 0.0]), "{b:?}");
        assert!(near(b.max, [1.0, 1.0, 0.0]), "{b:?}");
        // Reaching the view's edge asks for a wider view.
        assert!(metres(Some([0, 10, 20, 30]), None, 6.0).unwrap().1);
        assert_eq!(metres(None, None, 6.0), None);
    }

    #[test]
    fn the_core_leaves_out_the_few_pixels_furthest_out() {
        // A 100 px wide body on rows 300 to 399 and one spark far to the
        // right: the reach holds the spark, the core does not.
        let mut seen = Seen::new();
        seen.cols[200..300].fill(100);
        seen.rows[300..400].fill(100);
        seen.cols[700] = 4;
        seen.rows[350] += 4;
        assert_eq!(seen.pixels(0.0), Some([200, 300, 700, 399]));
        // 10004 pixels: 100 are trimmed from each end. On the right those are
        // the spark's 4 and 96 of column 299's 100, so 299 stays.
        assert_eq!(seen.pixels(CORE_TRIM), Some([201, 301, 299, 398]));
        assert_eq!(Seen::new().pixels(0.0), None);
        // The origin is on row 384: three quarters of this is above it.
        let mut seen = Seen::new();
        seen.rows[100] = 30;
        seen.rows[383] = 30;
        seen.rows[384] = 15;
        seen.rows[700] = 5;
        assert_eq!(seen.below(), 0.25);
        assert_eq!(Seen::new().below(), 0.0);
    }

    #[test]
    fn the_camera_stands_where_the_box_fills_the_view() {
        // Nothing measured: the default view, 5.4 m from a point 0.8 m up.
        let (c, d) = framing(None);
        assert_eq!(c, DEFAULT_CENTRE);
        assert!((d - 5.39).abs() < 0.02, "{d}");
        // A box 8 m across stands further back than one 1 m across, and a
        // speck is framed as if it were MIN_RADIUS.
        let cube = |h: f64| {
            Some(Bounds {
                min: [-h, 0.0, -h],
                max: [h, 2.0 * h, h],
                clipped: false,
            })
        };
        assert!(framing(cube(4.0)).1 > 4.0 * framing(cube(0.5)).1);
        assert_eq!(framing(cube(0.01)).1, framing(cube(0.1)).1);
        assert_eq!(framing(cube(4.0)).0, [0.0, 4.0, 0.0]);
    }
}
