//! Offline measurements of arbitrary footage. Decode timestamps, never a
//! frame number divided by the advertised frame rate, drive every sample.

use super::resolve;
use crate::contact_sheet;
use og_cook::cook::{conform::palette, video::ffmpeg_bin};
use og_probe::frame::{Rgba, flip};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SIDE: u32 = 192;
const SHEET_CELLS: usize = 24;

#[derive(Serialize)]
struct Frame {
    index: usize,
    pts_ticks: i64,
    pts: f64,
    time: f64,
    scene_score: f64,
}

struct Timeline {
    frames: Vec<Frame>,
    time_base: [i64; 2],
    frame_rate: [i64; 2],
    width: u32,
    height: u32,
    duration: f64,
}

fn ffmpeg(args: &[&str], media: &Path, tail: &[&str]) -> Result<Output, String> {
    let out = Command::new(ffmpeg_bin())
        .args(["-hide_banner", "-nostdin", "-nostats"])
        .args(args)
        .arg("-i")
        .arg(media)
        .args(["-map", "0:v:0", "-an", "-sn", "-dn"])
        .args(tail)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", ffmpeg_bin().display()))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out)
}

fn ratio(text: &str) -> Result<[i64; 2], String> {
    let (a, b) = text.trim().split_once('/').ok_or("missing time ratio")?;
    let a = a.parse().map_err(|_| "invalid time ratio")?;
    let b = b.parse().map_err(|_| "invalid time ratio")?;
    if b == 0 {
        return Err("zero time denominator".into());
    }
    Ok([a, b])
}

/// framecrc is a documented muxer: integer PTS and time base survive
/// decoding/reordering and passthrough prevents CFR duplication. A second,
/// stream-copy pass reads packet ends for the duration (including a VFR
/// final packet), rather than guessing a last-frame duration from fps.
fn timeline(media: &Path, limit: Option<f64>) -> Result<Timeline, String> {
    let decoded = ffmpeg(
        &["-v", "info", "-copyts"],
        media,
        &[
            "-vf",
            "select='gte(scene,0)',metadata=mode=print,showinfo",
            "-fps_mode",
            "passthrough",
            "-enc_time_base",
            "-1",
            "-c:v",
            "rawvideo",
            "-f",
            "framecrc",
            "-",
        ],
    )?;
    let crc = String::from_utf8_lossy(&decoded.stdout);
    let log = String::from_utf8_lossy(&decoded.stderr);
    let time_base = ratio(
        crc.lines()
            .find_map(|l| l.strip_prefix("#tb 0: "))
            .ok_or("no video time base")?,
    )?;
    let frame_rate = ratio(
        log.lines()
            .find_map(|l| {
                l.contains("config in time_base:")
                    .then(|| l.split_once("frame_rate: ").map(|p| p.1))
                    .flatten()
            })
            .ok_or("no video frame rate")?,
    )?;
    let (w, h) = crc
        .lines()
        .find_map(|l| l.strip_prefix("#dimensions 0: "))
        .and_then(|s| s.split_once('x'))
        .ok_or("no video dimensions")?;
    let width = w.trim().parse().map_err(|_| "invalid video width")?;
    let height = h.trim().parse().map_err(|_| "invalid video height")?;
    let scores: Vec<f64> = log
        .lines()
        .filter_map(|l| l.split_once("lavfi.scene_score=").map(|p| p.1))
        .map(|s| {
            s.trim()
                .parse()
                .map_err(|_| "invalid scene score".to_owned())
        })
        .collect::<Result<_, _>>()?;
    let ticks = |s: &str| -> Result<Vec<(i64, i64)>, String> {
        s.lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .map(|l| {
                let fields: Vec<_> = l.split(',').map(str::trim).collect();
                let pts = fields
                    .get(2)
                    .ok_or("missing frame PTS")?
                    .parse()
                    .map_err(|_| "invalid frame PTS")?;
                let duration = fields
                    .get(3)
                    .ok_or("missing frame duration")?
                    .parse()
                    .map_err(|_| "invalid frame duration")?;
                Ok((pts, duration))
            })
            .collect()
    };
    let pts = ticks(&crc)?;
    if pts.is_empty() || pts.len() != scores.len() {
        return Err("no decoded frames or scene/PTS count mismatch".into());
    }
    if pts.windows(2).any(|p| p[0].0 >= p[1].0) {
        return Err("video PTS must increase strictly".into());
    }
    let seconds = |t: i64| t as f64 * time_base[0] as f64 / time_base[1] as f64;
    let origin = seconds(pts[0].0);
    let packets = ffmpeg(
        &["-v", "error", "-copyts"],
        media,
        &["-c:v", "copy", "-f", "framecrc", "-"],
    )?;
    let packets = String::from_utf8_lossy(&packets.stdout);
    let packet_tb = ratio(
        packets
            .lines()
            .find_map(|l| l.strip_prefix("#tb 0: "))
            .ok_or("no packet time base")?,
    )?;
    let end = ticks(&packets)?
        .iter()
        .map(|&(p, d)| (p as f64 + d as f64) * packet_tb[0] as f64 / packet_tb[1] as f64)
        .fold(f64::NEG_INFINITY, f64::max);
    let duration = limit
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(f64::INFINITY)
        .min(end - origin);
    if !duration.is_finite() || duration <= 0.0 {
        return Err("video has no positive duration".into());
    }
    let frames = pts
        .iter()
        .zip(scores)
        .enumerate()
        .map(|(index, (&(p, _), scene_score))| Frame {
            index,
            pts_ticks: p,
            pts: seconds(p),
            time: seconds(p - pts[0].0),
            scene_score,
        })
        .take_while(|f| f.time < duration)
        .collect();
    Ok(Timeline {
        frames,
        time_base,
        frame_rate,
        width,
        height,
        duration,
    })
}

fn number(word: &str, option: &str, zero: bool) -> Result<f64, String> {
    let n: f64 = word
        .parse()
        .map_err(|_| format!("{option}: expected seconds/number"))?;
    if !n.is_finite() || n < 0.0 || (!zero && n == 0.0) {
        return Err(format!(
            "{option}: expected a finite {} number",
            if zero { "nonnegative" } else { "positive" }
        ));
    }
    Ok(n)
}

pub(super) fn run(args: &[&str]) -> Result<(), String> {
    let (mut path, mut out) = (None, None);
    let (mut interval, mut window, mut cut) = (0.25, 3.0, 0.35);
    let mut at = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let option = args[i];
        match option {
            "--out" | "--interval" | "--at" | "--window" | "--cut-change" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| format!("{option} needs a value"))?;
                match option {
                    "--out" => out = Some(PathBuf::from(value)),
                    "--interval" => interval = number(value, option, false)?,
                    "--window" => window = number(value, option, false)?,
                    "--cut-change" => {
                        cut = number(value, option, true)?;
                        if cut > 1.0 {
                            return Err("--cut-change must be in 0..1".into());
                        }
                    }
                    _ => {
                        for t in value.split(',') {
                            at.push(number(t, option, true)?);
                        }
                    }
                }
                i += 2;
            }
            a if a.starts_with('-') => {
                return Err(format!("unknown option '{a}'; see og video study --help"));
            }
            a => {
                if path.replace(PathBuf::from(a)).is_some() {
                    return Err("study takes one clip".into());
                }
                i += 1;
            }
        }
    }
    let path = path.ok_or("expected a clip; see og video study --help")?;
    let (media, limit) = resolve(&path)?;
    let t = timeline(&media, limit)?;
    if at.iter().any(|s| *s >= t.duration) {
        return Err(format!("--at times must be before {:.9}s", t.duration));
    }
    let dir = out.unwrap_or_else(|| {
        path.with_file_name(format!(
            "{}.study",
            path.file_stem().unwrap_or_default().to_string_lossy()
        ))
    });
    std::fs::create_dir_all(dir.join("frames")).map_err(|e| e.to_string())?;

    // The frame actually on screen at a requested time: greatest PTS <= t.
    let pick = |time: f64| {
        t.frames
            .partition_point(|f| f.time <= time)
            .saturating_sub(1)
    };
    let sample_times: Vec<_> = (0..)
        .map(|i| i as f64 * interval)
        .take_while(|s| *s < t.duration)
        .collect();
    let samples: Vec<_> = sample_times.iter().map(|&s| pick(s)).collect();
    let cuts: Vec<_> = t
        .frames
        .iter()
        .skip(1)
        .filter(|f| f.scene_score > cut)
        .map(|f| f.index)
        .collect();
    let mut starts = vec![0];
    starts.extend(&cuts);
    let mut palette_frames = samples.clone();
    for &index in &starts {
        if !palette_frames.contains(&index) {
            palette_frames.push(index);
        }
    }
    palette_frames.sort_unstable();
    let selected: BTreeSet<_> = palette_frames
        .iter()
        .copied()
        .chain(at.iter().map(|&s| pick(s)))
        .collect();
    let count = selected.len().min(SHEET_CELLS);
    // Same evenly spaced, rounded positions as contact_sheet::picks,
    // with a fixed 24-cell budget rather than render's column-based one.
    let sheet_frames: BTreeSet<_> = (0..count)
        .map(|i| {
            let position = (i * (selected.len() - 1) + (count - 1) / 2) / (count - 1).max(1);
            *selected.iter().nth(position).expect("selected position")
        })
        .collect();
    let mut terms: Vec<_> = selected.iter().map(|i| format!("eq(n,{i})")).collect();
    // A flat sum is left associative and exceeds ffmpeg's expression
    // recursion limit at ~100 selections. Balance it for long recordings.
    while terms.len() > 1 {
        terms = terms
            .chunks(2)
            .map(|pair| format!("({})", pair.join("+")))
            .collect();
    }
    let filter = format!("select='{}'", terms[0]);
    let mut child = Command::new(ffmpeg_bin())
        .args(["-hide_banner", "-nostdin", "-nostats", "-v", "error", "-i"])
        .arg(&media)
        .args([
            "-map",
            "0:v:0",
            "-an",
            "-sn",
            "-dn",
            "-vf",
            &filter,
            "-fps_mode",
            "passthrough",
            "-c:v",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-f",
            "rawvideo",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run {}: {e}", ffmpeg_bin().display()))?;
    let stride = t.width as usize * t.height as usize * 4;
    let mut small = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut cells = Vec::new();
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    std::thread::scope(|scope| -> Result<(), String> {
        // Drain errors concurrently so a full stderr pipe cannot stall
        // the decoder while its stdout is consumed one frame at a time.
        let errors = scope.spawn(move || {
            let mut bytes = Vec::new();
            stderr.read_to_end(&mut bytes).map(|_| bytes)
        });
        let result = (|| -> Result<(), String> {
            let mut image = Rgba {
                width: t.width,
                height: t.height,
                pixels: vec![0; stride],
            };
            for &index in &selected {
                stdout
                    .read_exact(&mut image.pixels)
                    .map_err(|e| format!("decoded frame {index} is incomplete: {e}"))?;
                let f = &t.frames[index];
                let file = format!("frames/{:.9}s-pts{}-f{index:06}.png", f.time, f.pts_ticks);
                std::fs::write(dir.join(&file), image.png()).map_err(|e| e.to_string())?;
                if sheet_frames.contains(&index) {
                    cells.push((
                        contact_sheet::thumbnail(t.width, t.height, image.pixels.clone()),
                        format!("{:.3}s", f.time),
                    ));
                }
                small.insert(index, image.fit(SIDE));
                files.insert(index, file);
            }
            if stdout.read(&mut [0]).map_err(|e| e.to_string())? != 0 {
                return Err("decoded frame size/count changed".into());
            }
            Ok(())
        })();
        if result.is_err() {
            // A PNG write/read failure must not leave ffmpeg blocked on
            // stdout, nor leave its child process unreaped.
            let _ = child.kill();
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        let errors = errors
            .join()
            .map_err(|_| "ffmpeg stderr reader panicked")?
            .map_err(|e| e.to_string())?;
        result.map_err(|e| {
            if errors.is_empty() {
                e
            } else {
                format!("{e}; ffmpeg: {}", String::from_utf8_lossy(&errors).trim())
            }
        })?;
        if !status.success() {
            return Err(format!(
                "ffmpeg: {}",
                String::from_utf8_lossy(&errors).trim()
            ));
        }
        Ok(())
    })?;
    contact_sheet::sheet(&cells, 4)
        .save(dir.join("contact-sheet.png"))
        .map_err(|e| e.to_string())?;
    let frame_ref = |index: usize| {
        json!({ "index": index, "pts": t.frames[index].pts,
        "pts_ticks": t.frames[index].pts_ticks, "time": t.frames[index].time, "file": files[&index] })
    };
    let mut changes = Vec::new();
    for (i, pair) in samples.windows(2).enumerate() {
        let (a, b) = (&small[&pair[0]], &small[&pair[1]]);
        let ppd = flip::DEFAULT_PPD * b.width as f32 / t.width as f32;
        let change = flip::flip(a, b, ppd)?.stats().mean;
        changes.push(json!({"from": sample_times[i], "to": sample_times[i+1],
            "from_index": pair[0], "to_index": pair[1], "mean_flip": change}));
    }
    let mut windows = Vec::new();
    for start in sample_times
        .iter()
        .copied()
        .take_while(|s| s + window <= t.duration)
    {
        let mut values: Vec<_> = changes
            .iter()
            .filter(|c| {
                let end = c["to"].as_f64().unwrap();
                end > start && end <= start + window
            })
            .map(|c| c["mean_flip"].as_f64().unwrap())
            .collect();
        values.sort_by(f64::total_cmp);
        let n = values.len();
        let (mean, median) = if n == 0 {
            (None, None)
        } else {
            (
                Some(values.iter().sum::<f64>() / n as f64),
                Some((values[(n - 1) / 2] + values[n / 2]) / 2.0),
            )
        };
        windows.push(json!({ "start": start, "end": start + window, "count": n, "mean": mean, "median": median }));
    }
    let palette_for = |indices: Vec<usize>| -> Value {
        let px: Vec<[u8; 4]> = indices
            .iter()
            .flat_map(|i| small[i].pixels.as_chunks::<4>().0.iter().copied())
            .collect();
        let pal = palette::adaptive(&px, None, 8, 0);
        let all: Vec<_> = (0..pal.rgb.len()).collect();
        let mapped = palette::map(
            &px,
            px.len() as u32,
            &|_, _| (0, 0),
            &pal,
            &all,
            palette::Dither::None,
        );
        let used = palette::usage(&mapped, pal.rgb.len());
        let opaque_pixels: u64 = used.iter().sum();
        json!({ "frames": indices, "pixels": px.len(), "opaque_pixels": opaque_pixels, "colors": pal.rgb.iter().zip(used).map(|(rgb, n)| {
            json!({ "rgb": rgb, "hex": format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]), "fraction": n as f64 / opaque_pixels as f64 })
        }).collect::<Vec<_>>() })
    };
    let segments: Vec<_> = starts.iter().enumerate().map(|(i, &start)| {
        let end = starts.get(i+1).copied().unwrap_or(t.frames.len());
        json!({ "start": t.frames[start].time, "end": t.frames.get(end).map_or(t.duration, |f| f.time),
            "palette": palette_for(palette_frames.iter().copied().filter(|i| *i >= start && *i < end).collect()) })
    }).collect();
    let doc = json!({ "schema": 1, "source": path, "media": media, "duration": t.duration,
        "width": t.width, "height": t.height, "time_base": t.time_base,
        "frame_rate": t.frame_rate, "fps": t.frame_rate[0] as f64 / t.frame_rate[1] as f64,
        "source_frames": t.frames, "interval": interval, "window": window,
        "sheet": "contact-sheet.png", "sheet_frames": sheet_frames,
        "frames": selected.iter().map(|&i| frame_ref(i)).collect::<Vec<_>>(),
        "samples": sample_times.iter().zip(&samples).map(|(&s, &i)| json!({ "requested_time": s, "frame": frame_ref(i) })).collect::<Vec<_>>(),
        "at": at.iter().map(|&s| json!({ "requested_time": s, "frame": frame_ref(pick(s)) })).collect::<Vec<_>>(),
        "cuts": cuts.iter().map(|&i| json!({ "index": i, "time": t.frames[i].time, "pts": t.frames[i].pts, "scene_score": t.frames[i].scene_score })).collect::<Vec<_>>(),
        "cut_change": cut, "palette": palette_for(palette_frames.to_vec()), "segments": segments,
        "palette_method": "8-color Oklab adaptive cut, nearest Oklab opaque pixel counts; equal-weight interval thumbnails plus segment starts, long side <=192",
        "change_method": "mean FLIP of adjacent interval samples, long side <=192, PPD scaled with width",
        "window_method": "full windows at interval steps; changes ending in (start,end]; even median averages middle two; empty statistics are null",
        "changes": changes, "windows": windows });
    std::fs::write(
        dir.join("study.json"),
        serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;
    println!("{}", dir.join("study.json").display());
    Ok(())
}
