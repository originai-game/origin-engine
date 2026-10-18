//! Sprite animation: flipbook sheets (frames of a texture, named clips with
//! a rate, a mode and frame events) and the `og.flipbook` system that plays
//! them on `og.Flipbook` components by the scaled tick.
//!
//! The sheet text (`.ogt` of `type flipbook`) is what the cook validates and
//! stores (with Aseprite and TexturePacker frames and tags resolved into
//! `frame` lines and clips), and what [`FlipbookSheet::parse`] reads back.

use og_core::ogt::{Document, Entry};
use og_core::{Error, ErrorKind, Reflect, Result, Str64};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const FLIPBOOK_SYSTEM: &str = "og.flipbook";

/// Plays a clip of a flipbook sheet. Draw it with an `og.Sprite` on the
/// same entity: extraction takes the texture, region and pivot of the
/// current frame from the sheet (`og.Sprite` keeps size, colour, flips and
/// layer).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Flipbook")]
pub struct Flipbook {
    /// The sheet (`og.load.flipbook`'s `sheet`); 0 is none.
    pub sheet: u64,
    /// The clip, an index into the sheet's clips; changing it restarts playback at its first frame.
    #[reflect(min = 0)]
    pub clip: u32,
    /// The playback rate: 1 as authored, 2 twice as fast, 0 holds the frame.
    #[reflect(default = 1, min = 0, max = 10, tunable)]
    pub speed: f32,
    /// Whether time advances.
    #[reflect(default = 1)]
    pub playing: bool,
    /// Written: the current frame, an index into the clip's frames.
    pub frame: u32,
    /// Written: seconds into the clip; set it to seek (no events fire across a seek).
    #[reflect(unit = "s", min = 0)]
    pub time: f32,
    /// Written: a `once` clip reached its last frame.
    pub finished: bool,
    /// Written: cycles completed (a `loop` or `pingPong` clip).
    pub loops: u32,
}

impl Default for Flipbook {
    fn default() -> Self {
        Flipbook {
            sheet: 0,
            clip: 0,
            speed: 1.0,
            playing: true,
            frame: 0,
            time: 0.0,
            finished: false,
            loops: 0,
        }
    }
}

/// A frame event a flipbook passed in the last tick (read the tick after,
/// like every event): the clip's frame named in its `event` line was entered.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.FlipbookEvent")]
pub struct FlipbookEvent {
    /// The entity with the og.Flipbook.
    pub entity: u64,
    /// The clip index.
    pub clip: u32,
    /// The frame index within the clip.
    pub frame: u32,
    /// The event's name.
    pub name: Str64,
}

/// How a clip runs past its last frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlipMode {
    /// Back to the first frame.
    #[default]
    Loop,
    /// Back and forth: 0 1 2 3 2 1 0 1 ...
    PingPong,
    /// Hold the last frame and set `finished`.
    Once,
}

impl FlipMode {
    pub fn name(self) -> &'static str {
        match self {
            FlipMode::Loop => "loop",
            FlipMode::PingPong => "pingPong",
            FlipMode::Once => "once",
        }
    }
    fn parse(s: &str) -> Option<FlipMode> {
        match s {
            "loop" => Some(FlipMode::Loop),
            "pingPong" | "pingpong" => Some(FlipMode::PingPong),
            "once" => Some(FlipMode::Once),
            _ => None,
        }
    }
}

/// A frame: a rectangle of the texture in pixels (x, y from the top left)
/// and the point of it at the entity's position, as a fraction from the
/// bottom-left corner.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetFrame {
    pub name: String,
    pub rect: [u32; 4],
    pub pivot: [f32; 2],
}

/// A named run of frames.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipClip {
    pub name: String,
    /// Frame indices into the sheet's frames.
    pub frames: Vec<u32>,
    /// Seconds each frame shows (one per frame).
    pub durations: Vec<f32>,
    pub mode: FlipMode,
    /// (clip frame index, name).
    pub events: Vec<(u32, String)>,
}

impl FlipClip {
    /// The frames of one cycle in play order (ping-pong adds the way back).
    fn cycle(&self) -> Vec<u32> {
        let n = self.frames.len() as u32;
        let mut c: Vec<u32> = (0..n).collect();
        if self.mode == FlipMode::PingPong && n > 2 {
            c.extend((1..n - 1).rev());
        }
        c
    }

    /// Seconds of one cycle.
    pub fn duration(&self) -> f32 {
        self.cycle()
            .iter()
            .map(|&i| self.durations[i as usize])
            .sum()
    }

    /// (step, clip frame) at `time` seconds: `step` counts frames entered
    /// since the clip started (across cycles), for events.
    fn at(&self, time: f32) -> (u64, u32, bool) {
        let cycle = self.cycle();
        let total: f32 = cycle.iter().map(|&i| self.durations[i as usize]).sum();
        if total <= 0.0 || cycle.is_empty() {
            return (0, 0, true);
        }
        let len = cycle.len() as u64;
        if self.mode == FlipMode::Once && time >= total {
            return (len - 1, *cycle.last().expect("non-empty"), true);
        }
        let k = (time / total).floor().max(0.0);
        let mut t = time - k * total;
        let mut i = 0usize;
        while i + 1 < cycle.len() && t >= self.durations[cycle[i] as usize] {
            t -= self.durations[cycle[i] as usize];
            i += 1;
        }
        (k as u64 * len + i as u64, cycle[i], false)
    }
}

/// Texture filtering a sheet asks for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetFilter {
    /// Pixel art: the nearest texel.
    #[default]
    Nearest,
    Linear,
}

/// A flipbook sheet: a texture, its frames and its clips.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipbookSheet {
    /// The texture's asset id.
    pub texture: String,
    /// The texture's size in pixels (the cook fills it in from the image).
    pub size: [u32; 2],
    pub filter: SheetFilter,
    pub frames: Vec<SheetFrame>,
    pub clips: Vec<FlipClip>,
}

fn parse_err(line: usize, what: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {line}: {what}"))
}

impl FlipbookSheet {
    /// Reads a `type flipbook` `.ogt` (the grammar is og-anim's README).
    /// `grid` frames come first (row-major, named by their index), then
    /// `frame` lines; clips name frames by index or name.
    pub fn parse(text: &str) -> Result<FlipbookSheet> {
        let doc = Document::parse(text)?;
        if doc.ty != "flipbook" {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("a flipbook sheet is `type flipbook`, not `{}`", doc.ty),
            ));
        }
        let mut sheet = FlipbookSheet {
            texture: String::new(),
            size: [0, 0],
            filter: SheetFilter::Nearest,
            frames: Vec::new(),
            clips: Vec::new(),
        };
        let mut pivot = [0.5f32, 0.5];
        let (mut margin, mut spacing) = (0u32, 0u32);
        let mut grid: Option<(Entry, usize)> = None;
        let num = |e: &Entry, i: usize| -> Result<f32> {
            e.f32(i)
                .map_err(|_| parse_err(e.line, format!("`{}` needs a number at {}", e.key, i + 1)))
        };
        let int = |e: &Entry, i: usize| -> Result<u32> {
            let v = num(e, i)?;
            if v < 0.0 || v.fract() != 0.0 {
                return Err(parse_err(
                    e.line,
                    format!("`{}` needs a whole number at {}", e.key, i + 1),
                ));
            }
            Ok(v as u32)
        };
        for e in &doc.body.entries {
            match e.key.as_str() {
                "texture" => {
                    sheet.texture = e
                        .string(0)
                        .map_err(|_| parse_err(e.line, "texture \"path\""))?
                        .to_string()
                }
                "size" => sheet.size = [int(e, 0)?, int(e, 1)?],
                "filter" => {
                    sheet.filter = match e.word(0).unwrap_or("") {
                        "nearest" => SheetFilter::Nearest,
                        "linear" => SheetFilter::Linear,
                        w => {
                            return Err(parse_err(
                                e.line,
                                format!("filter is nearest or linear, not `{w}`"),
                            ))
                        }
                    }
                }
                "pivot" => pivot = [num(e, 0)?, num(e, 1)?],
                "margin" => margin = int(e, 0)?,
                "spacing" => spacing = int(e, 0)?,
                "grid" => grid = Some((e.clone(), sheet.frames.len())),
                "frame" | "clip" | "atlas" => {}
                k => {
                    return Err(parse_err(
                        e.line,
                        format!("unknown key `{k}` in a flipbook sheet"),
                    ))
                }
            }
        }
        if sheet.texture.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "a flipbook sheet needs `texture \"path\"`",
            ));
        }
        if let Some((g, _)) = &grid {
            let (fw, fh) = (int(g, 0)?, int(g, 1)?);
            if fw == 0 || fh == 0 {
                return Err(parse_err(g.line, "grid cells are at least 1x1"));
            }
            let (cols, rows) = if g.values.len() >= 4 {
                (int(g, 2)?, int(g, 3)?)
            } else if sheet.size[0] > 0 {
                (
                    (sheet.size[0] - margin + spacing) / (fw + spacing),
                    (sheet.size[1] - margin + spacing) / (fh + spacing),
                )
            } else {
                return Err(parse_err(
                    g.line,
                    "grid w h [columns rows]: give the counts, or `size` of the texture",
                ));
            };
            for r in 0..rows {
                for c in 0..cols {
                    sheet.frames.push(SheetFrame {
                        name: (r * cols + c).to_string(),
                        rect: [
                            margin + c * (fw + spacing),
                            margin + r * (fh + spacing),
                            fw,
                            fh,
                        ],
                        pivot,
                    });
                }
            }
        }
        for e in doc.body.get_all("frame") {
            let name = e
                .values
                .first()
                .and_then(|v| {
                    v.as_str()
                        .or(v.as_word())
                        .map(str::to_string)
                        .or(v.as_number().map(|n| n.text().to_string()))
                })
                .ok_or_else(|| parse_err(e.line, "frame <name> x y w h [pivot x y]"))?;
            let rect = [int(e, 1)?, int(e, 2)?, int(e, 3)?, int(e, 4)?];
            let p = if e.values.len() >= 7 {
                [num(e, 5)?, num(e, 6)?]
            } else {
                pivot
            };
            sheet.frames.push(SheetFrame {
                name,
                rect,
                pivot: p,
            });
        }
        for e in doc.body.get_all("clip") {
            let name = e
                .values
                .first()
                .and_then(|v| v.as_word().or(v.as_str()))
                .ok_or_else(|| parse_err(e.line, "clip <name> { ... }"))?
                .to_string();
            let block = e
                .block
                .as_ref()
                .ok_or_else(|| parse_err(e.line, "clip <name> { frames ... }"))?;
            let mut frames = Vec::new();
            let mut fps = 12.0f32;
            let mut durations: Option<Vec<f32>> = None;
            let mut mode = FlipMode::Loop;
            let mut events = Vec::new();
            for c in &block.entries {
                match c.key.as_str() {
                    "frames" => {
                        for v in &c.values {
                            let key = v
                                .as_number()
                                .map(|n| n.text().to_string())
                                .or(v.as_word().or(v.as_str()).map(str::to_string))
                                .unwrap_or_default();
                            // By name, else a number is an index.
                            let i = sheet
                                .frames
                                .iter()
                                .position(|f| f.name == key)
                                .or_else(|| {
                                    key.parse::<usize>()
                                        .ok()
                                        .filter(|i| *i < sheet.frames.len())
                                })
                                .ok_or_else(|| {
                                    parse_err(
                                        c.line,
                                        format!(
                                            "clip {name}: no frame `{key}` ({} frames)",
                                            sheet.frames.len()
                                        ),
                                    )
                                })?;
                            frames.push(i as u32);
                        }
                    }
                    "fps" => fps = num(c, 0)?,
                    "durations" => {
                        durations = Some(
                            (0..c.values.len())
                                .map(|i| num(c, i))
                                .collect::<Result<_>>()?,
                        )
                    }
                    "mode" => {
                        mode =
                            c.word(0).ok().and_then(FlipMode::parse).ok_or_else(|| {
                                parse_err(c.line, "mode is loop, pingPong or once")
                            })?
                    }
                    "event" => events.push((
                        int(c, 0)?,
                        c.string(1)
                            .map_err(|_| parse_err(c.line, "event <frame> \"name\""))?
                            .to_string(),
                    )),
                    k => {
                        return Err(parse_err(
                            c.line,
                            format!("unknown key `{k}` in clip {name}"),
                        ))
                    }
                }
            }
            if frames.is_empty() {
                return Err(parse_err(e.line, format!("clip {name} has no frames")));
            }
            let durations = match durations {
                Some(d) if d.len() == frames.len() => d,
                Some(d) => {
                    return Err(parse_err(
                        e.line,
                        format!(
                            "clip {name}: {} durations for {} frames",
                            d.len(),
                            frames.len()
                        ),
                    ))
                }
                None => {
                    if !(fps > 0.0 && fps.is_finite()) {
                        return Err(parse_err(
                            e.line,
                            format!("clip {name}: fps must be above 0"),
                        ));
                    }
                    vec![1.0 / fps; frames.len()]
                }
            };
            if durations.iter().any(|d| !(*d > 0.0 && d.is_finite())) {
                return Err(parse_err(
                    e.line,
                    format!("clip {name}: every duration is above 0"),
                ));
            }
            for (f, ev) in &events {
                if *f as usize >= frames.len() {
                    return Err(parse_err(
                        e.line,
                        format!("clip {name}: event `{ev}` on frame {f} of {}", frames.len()),
                    ));
                }
                if ev.len() > 63 {
                    return Err(parse_err(
                        e.line,
                        format!("clip {name}: event `{ev}` is longer than 63 bytes"),
                    ));
                }
            }
            sheet.clips.push(FlipClip {
                name,
                frames,
                durations,
                mode,
                events,
            });
        }
        Ok(sheet)
    }

    /// The canonical text: `frame` lines for every frame (no grid), each
    /// clip with its frame indices and durations. Parses back equal.
    pub fn to_text(&self) -> String {
        let mut s = String::from("type flipbook\nversion 1\n---\n");
        s += &format!("texture {}\n", quote(&self.texture));
        s += &format!("size {} {}\n", self.size[0], self.size[1]);
        s += &format!(
            "filter {}\n",
            if self.filter == SheetFilter::Nearest {
                "nearest"
            } else {
                "linear"
            }
        );
        for f in &self.frames {
            s += &format!(
                "frame {} {} {} {} {} {} {}\n",
                quote(&f.name),
                f.rect[0],
                f.rect[1],
                f.rect[2],
                f.rect[3],
                num_text(f.pivot[0]),
                num_text(f.pivot[1])
            );
        }
        for c in &self.clips {
            s += &format!("clip {} {{\n", quote(&c.name));
            s += "  frames";
            for f in &c.frames {
                s += &format!(" {}", quote(&self.frames[*f as usize].name));
            }
            s += "\n  durations";
            for d in &c.durations {
                s += &format!(" {}", num_text(*d));
            }
            s += &format!("\n  mode {}\n", c.mode.name());
            for (f, n) in &c.events {
                s += &format!("  event {f} {}\n", quote(n));
            }
            s += "}\n";
        }
        s
    }

    /// The uv region `[u0, v0, u1, v1]` of a frame (v down).
    pub fn uv(&self, frame: u32) -> [f32; 4] {
        let Some(f) = self.frames.get(frame as usize) else {
            return [0.0, 0.0, 1.0, 1.0];
        };
        let (w, h) = (self.size[0].max(1) as f32, self.size[1].max(1) as f32);
        let [x, y, fw, fh] = f.rect.map(|v| v as f32);
        [x / w, y / h, (x + fw) / w, (y + fh) / h]
    }

    /// The clip index of a name.
    pub fn clip(&self, name: &str) -> Option<u32> {
        self.clips
            .iter()
            .position(|c| c.name == name)
            .map(|i| i as u32)
    }

    /// The sheet frame a flipbook shows (its clip's current frame).
    pub fn frame_of(&self, clip: u32, frame: u32) -> Option<&SheetFrame> {
        let c = self.clips.get(clip as usize)?;
        self.frames.get(*c.frames.get(frame as usize)? as usize)
    }

    pub fn validate(&self) -> Result<()> {
        for c in &self.clips {
            if let Some(f) = c.frames.iter().find(|f| **f as usize >= self.frames.len()) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("clip {}: frame {f} of {}", c.name, self.frames.len()),
                ));
            }
            if c.durations.len() != c.frames.len() {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("clip {}: durations and frames differ in length", c.name),
                ));
            }
        }
        Ok(())
    }
}

fn quote(s: &str) -> String {
    let mut q = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => q += "\\\"",
            '\\' => q += "\\\\",
            '\n' => q += "\\n",
            c => q.push(c),
        }
    }
    q + "\""
}

fn num_text(v: f32) -> String {
    og_core::ogt::Number::from_f32(v)
        .map(|n| n.text().to_string())
        .unwrap_or_else(|_| "0".into())
}

// ---- the system ---------------------------------------------------------------

/// Per entity: the clip it played last tick, the time it wrote, the step it
/// reached (frames entered since the clip started, for events) and the
/// whole cycles taken off `time` (so `time` stays within one cycle).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Playback {
    pub sheet: u64,
    pub clip: u32,
    pub time: f32,
    pub step: u64,
    pub base: u64,
}

#[derive(Default)]
struct Shared {
    sheets: BTreeMap<u64, Arc<FlipbookSheet>>,
    playback: BTreeMap<u64, Playback>,
}

fn lock(m: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Registers flipbook sheets and reads them back for drawing.
#[derive(Clone)]
pub struct FlipbookSystem {
    shared: Arc<Mutex<Shared>>,
}

impl FlipbookSystem {
    /// Registers (or replaces) sheet `handle` (not 0).
    pub fn add_sheet(&self, handle: u64, sheet: FlipbookSheet) -> Result<()> {
        if handle == 0 {
            return Err(Error::new(ErrorKind::Invalid, "sheet handle 0 means none"));
        }
        sheet.validate()?;
        lock(&self.shared).sheets.insert(handle, Arc::new(sheet));
        Ok(())
    }

    pub fn sheet(&self, handle: u64) -> Option<Arc<FlipbookSheet>> {
        lock(&self.shared).sheets.get(&handle).cloned()
    }
}

/// Registers `og.Flipbook` and `og.FlipbookEvent` and schedules
/// `og.flipbook` here, with its state block.
pub fn install_flipbook(world: &mut World) -> Result<FlipbookSystem> {
    world.register::<Flipbook>()?;
    world.register::<FlipbookEvent>()?;
    let shared = Arc::new(Mutex::new(Shared::default()));
    let desc = SystemDesc::new(FLIPBOOK_SYSTEM)
        .write::<Flipbook>()
        .write::<FlipbookEvent>()
        .query(QueryDesc::new().write::<Flipbook>());
    let s = shared.clone();
    world.add_system(desc, move |ctx| run(ctx, &s))?;
    world.add_state_block(FLIPBOOK_SYSTEM, Box::new(FlipState(shared.clone())))?;
    Ok(FlipbookSystem { shared })
}

/// The most events one entity sends per tick (a fast clip at a low tick
/// rate would otherwise flood the queue).
const MAX_EVENTS: u64 = 64;

fn run(ctx: &mut SystemContext<'_>, shared: &Mutex<Shared>) -> Result<()> {
    let dt = ctx.scaled_dt();
    let mut entities: Vec<Entity> = Vec::new();
    for a in ctx.query(0)? {
        entities.extend_from_slice(a.entities());
    }
    let mut sh = lock(shared);
    let mut events = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for e in entities {
        let key = e.to_bits();
        let mut f: Flipbook = ctx.get(e)?;
        let Some(sheet) = sh.sheets.get(&f.sheet).cloned() else {
            continue;
        };
        let Some(clip) = sheet.clips.get(f.clip as usize) else {
            continue;
        };
        seen.insert(key);
        let len = clip.cycle().len().max(1) as u64;
        let total = clip.duration();
        let prev = sh.playback.get(&key).copied();
        let fresh = prev.is_none_or(|p| p.clip != f.clip || p.sheet != f.sheet);
        let (mut base, mut step) = match prev {
            Some(p) if !fresh && p.time == f.time => (p.base, p.step),
            _ => {
                // A new clip starts at 0, unless the game set the time in
                // the same tick (a seek); a start at 0 enters the first
                // frame and fires its events, a seek lands without events.
                if fresh && prev.is_some_and(|p| p.time == f.time) {
                    f.time = 0.0;
                }
                f.finished = false;
                let s = clip.at(f.time).0;
                if fresh && f.time == 0.0 {
                    fire(&mut events, key, f.clip, clip, s, s);
                }
                (0, s)
            }
        };
        if f.playing && !f.finished {
            f.time += dt * f.speed.clamp(0.0, 10.0);
        }
        // Keep `time` inside one cycle; the cycles go to `base`.
        if clip.mode != FlipMode::Once && total > 0.0 && f.time >= total {
            let k = (f.time / total).floor();
            f.time -= k * total;
            base += k as u64 * len;
        }
        let (s, frame, done) = clip.at(f.time);
        let now = base + s;
        if now > step {
            fire(
                &mut events,
                key,
                f.clip,
                clip,
                step + 1,
                now.min(step + MAX_EVENTS),
            );
            step = now;
        }
        f.frame = frame;
        if clip.mode == FlipMode::Once {
            f.finished = done;
            if done {
                f.time = total;
            }
            f.loops = 0;
        } else {
            f.loops = (now / len) as u32;
        }
        ctx.set(e, f)?;
        sh.playback.insert(
            key,
            Playback {
                sheet: f.sheet,
                clip: f.clip,
                time: f.time,
                step,
                base,
            },
        );
    }
    sh.playback.retain(|k, _| seen.contains(k));
    drop(sh);
    for ev in events {
        ctx.send(ev)?;
    }
    Ok(())
}

/// Events of the frames entered at steps `from..=to` of `clip`'s cycle.
fn fire(
    out: &mut Vec<FlipbookEvent>,
    entity: u64,
    clip_index: u32,
    clip: &FlipClip,
    from: u64,
    to: u64,
) {
    if clip.events.is_empty() {
        return;
    }
    let cycle = clip.cycle();
    let len = cycle.len() as u64;
    for s in from..=to {
        let frame = cycle[(s % len) as usize];
        for (f, name) in &clip.events {
            if *f == frame {
                if let Ok(name) = Str64::new(name) {
                    out.push(FlipbookEvent {
                        entity,
                        clip: clip_index,
                        frame,
                        name,
                    });
                }
            }
        }
    }
}

struct FlipState(Arc<Mutex<Shared>>);

impl StateBlock for FlipState {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        out.extend_from_slice(&(sh.playback.len() as u32).to_le_bytes());
        for (k, p) in &sh.playback {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&p.sheet.to_le_bytes());
            out.extend_from_slice(&p.clip.to_le_bytes());
            out.extend_from_slice(&p.time.to_le_bytes());
            out.extend_from_slice(&p.step.to_le_bytes());
            out.extend_from_slice(&p.base.to_le_bytes());
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let bad = || Error::new(ErrorKind::Parse, "og.flipbook state block: truncated");
        let n =
            u32::from_le_bytes(bytes.get(0..4).ok_or_else(bad)?.try_into().expect("4")) as usize;
        if bytes.len() != 4 + n * 40 {
            return Err(bad());
        }
        let mut map = BTreeMap::new();
        for i in 0..n {
            let b = &bytes[4 + i * 40..4 + (i + 1) * 40];
            let u64_ = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().expect("8"));
            map.insert(
                u64_(0),
                Playback {
                    sheet: u64_(8),
                    clip: u32::from_le_bytes(b[16..20].try_into().expect("4")),
                    time: f32::from_le_bytes(b[20..24].try_into().expect("4")),
                    step: u64_(24),
                    base: u64_(32),
                },
            );
        }
        Ok(Box::new(map))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        lock(&self.0).playback = decoded
            .map(|d| {
                *d.downcast::<BTreeMap<u64, Playback>>()
                    .expect("decoded by FlipState")
            })
            .unwrap_or_default();
    }
}
