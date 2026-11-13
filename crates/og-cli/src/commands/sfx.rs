//! `og sfx`: the procedural sound-preset library (og-cook's `type sfx`).
//!
//! - `og sfx` lists the presets;
//! - `og sfx <preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ogt|file.wav>`
//!   writes the preset as an asset (`og cook` synthesises it into a sound)
//!   or renders it to a 16-bit WAV;
//! - `og sfx library <dir>` writes every preset's `.ogt` into `dir`.

use og_cook::cook::sfx::{PRESETS, RATE, Variation, preset_ogt, render};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: og sfx | og sfx <preset> [--seed N] [--pitch P] [--length L] [--volume V] --out <file.ogt|file.wav> | og sfx library <dir>";

fn wav(samples: &[i16]) -> Vec<u8> {
    let data = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn go(args: &[&str]) -> Result<(), String> {
    match args {
        [] => {
            for (name, what) in PRESETS {
                println!("{name:<10} {what}");
            }
            Ok(())
        }
        ["library", dir] => {
            for (name, _) in PRESETS {
                let path = Path::new(dir).join(format!("{name}.ogt"));
                write(&path, preset_ogt(name, Variation::default()).as_bytes())?;
            }
            println!("og sfx: {} presets in {dir}", PRESETS.len());
            Ok(())
        }
        [name, rest @ ..] => {
            if !PRESETS.iter().any(|p| p.0 == *name) {
                return Err(format!("no preset '{name}' (`og sfx` lists them). {USAGE}"));
            }
            let mut v = Variation::default();
            let mut out = None;
            let mut i = 0;
            while i < rest.len() {
                let value = rest
                    .get(i + 1)
                    .ok_or(format!("{} needs a value. {USAGE}", rest[i]))?;
                let num = || {
                    value
                        .parse::<f64>()
                        .map_err(|_| format!("{} takes a number", rest[i]))
                };
                match rest[i] {
                    "--seed" => v.seed = num()?.max(0.0) as u32,
                    "--pitch" => v.pitch = num()?,
                    "--length" => v.length = num()?,
                    "--volume" => v.volume = num()?,
                    "--out" => out = Some(value.to_string()),
                    other => return Err(format!("unknown option `{other}`. {USAGE}")),
                }
                i += 2;
            }
            let out = out.ok_or(format!("--out is required. {USAGE}"))?;
            let path = Path::new(&out);
            if out.ends_with(".wav") {
                write(path, &wav(&render(name, v).expect("a listed preset")))?;
            } else if out.ends_with(".ogt") {
                write(path, preset_ogt(name, v).as_bytes())?;
            } else {
                return Err(format!("--out is a .ogt (an asset) or a .wav. {USAGE}"));
            }
            println!("og sfx: wrote {out}");
            Ok(())
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    match go(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og sfx: {e}");
            ExitCode::from(2)
        }
    }
}
