//! `og sprite inspect <file>`: measurements of an animated sprite source
//! (an animated GIF, WebP or APNG, a still image) or of a flipbook sheet
//! built from them, for the agent deciding whether a generated animation
//! is usable: frame count and delays, each frame's drawn bounds and
//! centroid and their drift from the first frame, the loop seam (the last
//! frame's difference from the first beside the frame-to-frame steps) and
//! the alpha edge fringe. Numbers only; no thresholds (`og-cook`'s
//! `cook::animated` defines each).

use og_cook::cook::animated::{self, Border, Inspection, Key};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og sprite inspect <file.gif|.webp|.png|sheet.ogt> [--key auto|off|#rrggbb] [--tolerance T] [--out <frames.png>] [--json]";

pub fn run(args: &[&str]) -> ExitCode {
    match go(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og sprite: {e}");
            ExitCode::from(1)
        }
    }
}

fn go(args: &[&str]) -> Result<(), String> {
    let ["inspect", rest @ ..] = args else {
        return Err(USAGE.into());
    };
    let mut file = None;
    let mut key = None;
    let mut tolerance = 0.1f32;
    let mut as_json = false;
    let mut out = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i] {
            "--json" => as_json = true,
            "--key" => {
                i += 1;
                key = Some(match rest.get(i).copied() {
                    Some("auto") => Key::Auto,
                    Some("off") => Key::Off,
                    Some(h) => Key::Colour(
                        animated::parse_hex(h).ok_or(format!("--key {h}: auto, off or #rrggbb"))?,
                    ),
                    None => return Err(USAGE.into()),
                });
            }
            "--out" => {
                i += 1;
                out = Some(*rest.get(i).ok_or(USAGE)?);
            }
            "--tolerance" => {
                i += 1;
                tolerance = rest
                    .get(i)
                    .and_then(|t| t.parse().ok())
                    .filter(|t| (0.0..=0.5).contains(t))
                    .ok_or("--tolerance is 0 to 0.5")?;
            }
            f if !f.starts_with('-') && file.is_none() => file = Some(f),
            _ => return Err(USAGE.into()),
        }
        i += 1;
    }
    let file = Path::new(file.ok_or(USAGE)?);
    let report = if file.extension().is_some_and(|e| e == "ogt") {
        let built = animated::build_file(file)?;
        if let Some(out) = out {
            std::fs::write(out, &built.png).map_err(|e| format!("{out}: {e}"))?;
        }
        animated::inspect_built(&built)
    } else {
        let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let anim = animated::decode(&bytes)?;
        // By default the border's flat colour is keyed when it has one.
        let key = key.unwrap_or(
            match animated::border(&anim.frames[0].rgba, anim.width, anim.height) {
                Border::Flat { share, .. } if share >= 0.5 => Key::Auto,
                _ => Key::Off,
            },
        );
        let report = animated::inspect(&anim, key, tolerance)?;
        if let Some(out) = out {
            strip(&anim, report.keyed, tolerance, out)?;
        }
        report
    };
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&to_json(&report, tolerance)).expect("json")
        );
    } else {
        print!("{}", text(&file.display().to_string(), &report, tolerance));
    }
    Ok(())
}

/// The frames side by side, keyed as they were measured.
fn strip(
    anim: &animated::Animation,
    keyed: Option<[u8; 3]>,
    tolerance: f32,
    out: &str,
) -> Result<(), String> {
    let (w, h) = (anim.width, anim.height);
    let mut img = image::RgbaImage::new(w * anim.frames.len() as u32, h);
    for (i, f) in anim.frames.iter().enumerate() {
        let mut rgba = f.rgba.clone();
        if let Some(k) = keyed {
            animated::key_frame(&mut rgba, w, h, k, tolerance);
        }
        let frame = image::RgbaImage::from_raw(w, h, rgba).expect("a frame's size");
        image::imageops::replace(&mut img, &frame, i64::from(w) * i as i64, 0);
    }
    img.save(out).map_err(|e| format!("{out}: {e}"))
}

fn colour(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn text(name: &str, r: &Inspection, tolerance: f32) -> String {
    let mut s = format!(
        "{name}: {} {}x{}, {} frame(s), {:.3} s\n",
        r.format,
        r.width,
        r.height,
        r.frames.len(),
        r.duration()
    );
    if r.clamped > 0 {
        s += &format!(
            "  delays: {} of 10 ms or less, shown as 100 ms\n",
            r.clamped
        );
    }
    s += &match r.border {
        Border::Transparent => "  border: transparent".to_string(),
        Border::Flat { colour: c, share } => {
            format!("  border: {} over {:.1}%", colour(c), share * 100.0)
        }
    };
    s += &match r.keyed {
        Some(k) => format!("; measured with {} keyed at {tolerance}\n", colour(k)),
        None => "; measured as is\n".into(),
    };
    s += &format!("  alpha: {} level(s)\n", r.alpha_levels);
    s += "  frame  delay  bounds x0 y0 x1 y1   centroid x y     drift: centroid  bounds l t r b\n";
    for (i, f) in r.frames.iter().enumerate() {
        let bounds = f.shape.map_or("(empty)".to_string(), |sh| {
            let [a, b, c, d] = sh.bounds;
            format!("{a} {b} {c} {d}")
        });
        let centroid = f.shape.map_or("-".to_string(), |sh| {
            format!("{:.1} {:.1}", sh.centroid[0], sh.centroid[1])
        });
        let cd = f
            .centroid_drift
            .map_or("-".into(), |d| format!("{:+.1} {:+.1}", d[0], d[1]));
        let bd = f.bounds_drift.map_or("-".into(), |d| {
            format!("{:+} {:+} {:+} {:+}", d[0], d[1], d[2], d[3])
        });
        let dup = f
            .duplicate_of
            .map_or(String::new(), |j| format!("  = frame {j}"));
        s += &format!(
            "  {i:<5}  {:.3}  {bounds:<19}  {centroid:<15}  {cd:<15}  {bd}{dup}\n",
            f.delay
        );
    }
    let [l, t, rr, b] = r.max_bounds_drift;
    s += &format!(
        "  drift (largest): centroid {:.1} {:.1} px; bounds l {l} t {t} r {rr} b {b} px\n",
        r.max_centroid_drift[0], r.max_centroid_drift[1]
    );
    s += &format!(
        "  loop seam: {:.4} last to first; steps {:.4} mean, {:.4} largest\n",
        r.seam, r.step_mean, r.step_max
    );
    s += &format!(
        "  fringe: {} edge texel(s), {} partial; step to interior {:.3}",
        r.fringe.edge, r.fringe.partial, r.fringe.step
    );
    if let Some(d) = r.fringe.to_background {
        s += &format!("; distance to the background {d:.3}");
    }
    s.push('\n');
    s
}

/// An f32 as the f64 of its shortest decimal (0.05, not 0.05000000074505806).
fn n(v: f32) -> f64 {
    format!("{v}").parse().expect("a float's text")
}

fn to_json(r: &Inspection, tolerance: f32) -> serde_json::Value {
    json!({
        "format": r.format,
        "width": r.width,
        "height": r.height,
        "duration": n(r.duration()),
        "clampedDelays": r.clamped,
        "border": match r.border {
            Border::Transparent => json!({ "transparent": true }),
            Border::Flat { colour: c, share } => json!({ "colour": colour(c), "share": n(share) }),
        },
        "keyed": r.keyed.map(|k| json!({ "colour": colour(k), "tolerance": n(tolerance) })),
        "alphaLevels": r.alpha_levels,
        "frames": r.frames.iter().map(|f| json!({
            "delay": n(f.delay),
            "bounds": f.shape.map(|s| s.bounds),
            "centroid": f.shape.map(|s| s.centroid.map(n)),
            "coverage": f.shape.map(|s| n(s.coverage)),
            "centroidDrift": f.centroid_drift.map(|d| d.map(n)),
            "boundsDrift": f.bounds_drift,
            "duplicateOf": f.duplicate_of,
        })).collect::<Vec<_>>(),
        "maxCentroidDrift": r.max_centroid_drift.map(n),
        "maxBoundsDrift": r.max_bounds_drift,
        "seam": n(r.seam),
        "stepMean": n(r.step_mean),
        "stepMax": n(r.step_max),
        "fringe": {
            "edge": r.fringe.edge,
            "partial": r.fringe.partial,
            "step": n(r.fringe.step),
            "toBackground": r.fringe.to_background.map(n),
        },
    })
}
