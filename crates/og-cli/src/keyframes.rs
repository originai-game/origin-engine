//! `og render --keyframes`: the frames of a render where something happened,
//! with a timeline synced to ticks (the captures guide). A frame is a key
//! frame when
//!
//! - it is the first of the render (`start`) or of a `--jobs` part (`part`:
//!   a part does not see the frames before it), or the last (`end`);
//! - the run's timeline (the probe's `timeline`: inputs, command steps,
//!   screen changes, assertion events, expectations) has an event at a tick
//!   after the frame before it, up to its own: each such event is a reason;
//! - it differs from the last key frame by at least `--keyframe-change`
//!   (default 0.02) of mean FLIP (og-probe's `frame::flip`, on both frames
//!   fitted to 192 pixels, the observer's pixels per degree scaled with
//!   them), at least `--keyframe-gap` seconds (default 0.5) after it: a
//!   `visual` reason with the measured change.
//!
//! Each key frame is written as `keyframes/<index>.png` fitted to the long
//! side 1280 (`frame::MODEL_LONG_SIDE`), all of them on one labelled sheet
//! `keyframes.png`, and `timeline.json` lists the key frames with their
//! reasons and every event with its tick and time. The numbers are
//! measurements; which frames matter is the reader's call.

use crate::contact_sheet;
use og_probe::frame::{MODEL_LONG_SIDE, Rgba, flip};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The long side frames are compared at.
const COMPARE_SIDE: u32 = 192;

/// What `--keyframes` was asked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyframeArgs {
    /// The mean FLIP from the last key frame that makes a visual key frame.
    pub change: f32,
    /// The fewest seconds between a key frame and a visual one after it.
    pub gap: f64,
}

impl Default for KeyframeArgs {
    fn default() -> Self {
        KeyframeArgs {
            change: 0.02,
            gap: 0.5,
        }
    }
}

/// One key frame.
#[derive(Clone, Debug)]
pub struct KeyFrame {
    pub index: u64,
    pub tick: u64,
    pub time: f64,
    /// Under the render's directory.
    pub file: String,
    /// Mean FLIP from the key frame before it; `None` for a part's first.
    pub change: Option<f32>,
    pub reasons: Vec<Value>,
    /// The sheet's cell.
    pub thumb: image::RgbaImage,
}

/// One job's key frame choice, frame by frame.
pub struct Picker {
    args: KeyframeArgs,
    dir: PathBuf,
    root: PathBuf,
    last: Option<(Rgba, f64)>,
    first: bool,
    /// The newest frame and whether it was kept, for the render's `end`.
    pub latest: Option<(u64, u64, f64, Rgba, bool)>,
    pub kept: Vec<KeyFrame>,
    pub events: Vec<Value>,
}

impl Picker {
    /// A picker writing under `root`/keyframes; `start` for the render's
    /// first part.
    pub fn new(args: KeyframeArgs, root: &Path) -> Picker {
        Picker {
            args,
            dir: root.join("keyframes"),
            root: root.to_path_buf(),
            last: None,
            first: true,
            latest: None,
            kept: Vec::new(),
            events: Vec::new(),
        }
    }

    /// One frame of the job, with the timeline's events since the frame
    /// before, at `rate` ticks a second.
    pub fn frame(
        &mut self,
        (index, tick, time): (u64, u64, f64),
        width: u32,
        height: u32,
        rgba: &[u8],
        events: &[Value],
        rate: f64,
    ) -> Result<(), String> {
        let image = Rgba {
            width,
            height,
            pixels: rgba.to_vec(),
        };
        let small = image.fit(COMPARE_SIDE);
        let mut reasons = Vec::new();
        if self.first {
            reasons.push(json!({ "kind": if index == 0 { "start" } else { "part" } }));
        }
        for e in events {
            let t = e["tick"].as_u64().unwrap_or(tick);
            let mut e = e.clone();
            e["time"] = json!(t as f64 / rate);
            reasons.push(json!({ "kind": e["kind"], "what": e["what"], "tick": t }));
            self.events.push(e);
        }
        let change = self.last.as_ref().map(|(last, _)| {
            let ppd = flip::DEFAULT_PPD * small.width as f32 / width.max(1) as f32;
            flip::flip(last, &small, ppd).map_or(0.0, |m| m.stats().mean)
        });
        if let (Some(c), Some((_, at))) = (change, &self.last)
            && c >= self.args.change
            && time - at >= self.args.gap
        {
            reasons.push(json!({ "kind": "visual", "change": c }));
        }
        let keep = !reasons.is_empty();
        if keep {
            self.keep(index, tick, time, &image, change, reasons)?;
            self.last = Some((small, time));
        }
        self.first = false;
        self.latest = Some((index, tick, time, image, keep));
        Ok(())
    }

    fn keep(
        &mut self,
        index: u64,
        tick: u64,
        time: f64,
        image: &Rgba,
        change: Option<f32>,
        reasons: Vec<Value>,
    ) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let path = self.dir.join(format!("{index:05}.png"));
        std::fs::write(&path, image.fit(MODEL_LONG_SIDE).png())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        self.kept.push(KeyFrame {
            index,
            tick,
            time,
            file: path
                .strip_prefix(&self.root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned(),
            change,
            reasons,
            thumb: contact_sheet::thumbnail(image.width, image.height, image.pixels.clone()),
        });
        Ok(())
    }

    /// The render's last frame, kept as `end` unless it already was.
    pub fn end(&mut self) -> Result<(), String> {
        if let Some((index, tick, time, image, kept)) = self.latest.take() {
            if kept {
                if let Some(k) = self.kept.iter_mut().find(|k| k.index == index) {
                    k.reasons.push(json!({ "kind": "end" }));
                }
            } else {
                let change = self.last.as_ref().map(|(last, _)| {
                    let small = image.fit(COMPARE_SIDE);
                    let ppd = flip::DEFAULT_PPD * small.width as f32 / image.width.max(1) as f32;
                    flip::flip(last, &small, ppd).map_or(0.0, |m| m.stats().mean)
                });
                self.keep(
                    index,
                    tick,
                    time,
                    &image,
                    change,
                    vec![json!({ "kind": "end" })],
                )?;
            }
        }
        Ok(())
    }
}

/// Writes `keyframes.png` (the key frames on one sheet, four to a row, each
/// labelled with its time, tick and reasons) and `timeline.json` under
/// `root`; answers their paths.
pub fn write(
    root: &Path,
    mut kept: Vec<KeyFrame>,
    mut events: Vec<Value>,
    head: Value,
) -> Result<(PathBuf, PathBuf), String> {
    kept.sort_by_key(|k| k.index);
    events.sort_by_key(|e| e["tick"].as_u64().unwrap_or(0));
    let cells: Vec<(image::RgbaImage, String)> = kept
        .iter()
        .map(|k| {
            let mut kinds: Vec<&str> = k
                .reasons
                .iter()
                .filter_map(|r| r["kind"].as_str())
                .map(|kind| match kind {
                    "assertion" => "ASSERT",
                    "start" => "START",
                    "part" => "PART",
                    "end" => "END",
                    "input" => "INPUT",
                    "step" => "STEP",
                    "screen" => "SCREEN",
                    "expect" => "EXPECT",
                    _ => "VISUAL",
                })
                .collect();
            kinds.dedup();
            let mut text = format!("{:.2}s t{} {}", k.time, k.tick, kinds.join(" "));
            // Cut to the cell (the sheet's labels are scale 2).
            text.truncate(contact_sheet::fits(k.thumb.width(), 2));
            (k.thumb.clone(), text)
        })
        .collect();
    let sheet = root.join("keyframes.png");
    contact_sheet::sheet(&cells, 4)
        .save(&sheet)
        .map_err(|e| format!("{}: {e}", sheet.display()))?;
    let mut doc = head;
    doc["sheet"] = json!("keyframes.png");
    doc["keyframes"] = json!(
        kept.iter()
            .map(|k| json!({
                "index": k.index,
                "tick": k.tick,
                "time": k.time,
                "file": k.file,
                "change": k.change,
                "reasons": k.reasons,
            }))
            .collect::<Vec<_>>()
    );
    doc["events"] = json!(events);
    let path = root.join("timeline.json");
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n",
    )
    .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((path, sheet))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32, square: Option<(u32, [u8; 3])>) -> Vec<u8> {
        let mut img = Rgba::filled(w, h, [60, 60, 70, 255]);
        if let Some((x, c)) = square {
            img.fill(i64::from(x), 20, 40, 40, [c[0], c[1], c[2], 255]);
        }
        img.pixels
    }

    #[test]
    fn events_changes_and_the_ends_make_key_frames_and_nothing_else_does() {
        let dir = std::env::temp_dir().join(format!("og-keyframes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut p = Picker::new(KeyframeArgs::default(), &dir);
        let rate = 60.0;
        let still = frame(320, 180, None);
        let tap = json!({ "tick": 9, "kind": "step", "what": "tap:Play", "detail": {} });
        let feed = |p: &mut Picker, i: u64, rgba: &[u8], events: &[Value]| {
            let tick = i * 2;
            p.frame((i, tick, tick as f64 / rate), 320, 180, rgba, events, rate)
                .unwrap()
        };
        // 0: start. 1..4: the same picture, nothing happens.
        for i in 0..5 {
            feed(&mut p, i, &still, &[]);
        }
        // 5: a tap, nothing changed yet: an event key frame.
        feed(&mut p, 5, &still, &[tap]);
        // 6: a square appears, 0.03 s after the tap's frame: within the gap.
        let square = frame(320, 180, Some((100, [250, 240, 30])));
        feed(&mut p, 6, &square, &[]);
        // 40: the same square, well after the gap: it differs from the last
        // key frame (5, without it), so a visual key frame.
        feed(&mut p, 40, &square, &[]);
        // 41..43: unchanged since 40.
        for i in 41..44 {
            feed(&mut p, i, &square, &[]);
        }
        p.end().unwrap();
        let picked: Vec<(u64, Vec<String>)> = p
            .kept
            .iter()
            .map(|k| {
                (
                    k.index,
                    k.reasons
                        .iter()
                        .map(|r| r["kind"].as_str().unwrap().to_owned())
                        .collect(),
                )
            })
            .collect();
        assert_eq!(
            picked,
            [
                (0, vec!["start".to_owned()]),
                (5, vec!["step".to_owned()]),
                (40, vec!["visual".to_owned()]),
                (43, vec!["end".to_owned()]),
            ]
        );
        assert!(p.kept[2].change.unwrap() >= 0.02);
        assert!(p.kept[3].change.unwrap() < 0.01, "43 is 40 again");
        // The event is listed with its own tick and time.
        assert_eq!(p.events[0]["tick"], json!(9));
        assert_eq!(p.events[0]["time"], json!(0.15));
        let img = image::open(dir.join(&p.kept[1].file)).unwrap();
        assert_eq!((img.width(), img.height()), (320, 180));
        let (timeline, _) = write(
            &dir,
            p.kept.clone(),
            p.events.clone(),
            json!({ "rate": 60 }),
        )
        .unwrap();
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(timeline).unwrap()).unwrap();
        assert_eq!(doc["keyframes"].as_array().unwrap().len(), 4);
        assert_eq!(doc["keyframes"][1]["reasons"][0]["what"], json!("tap:Play"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
