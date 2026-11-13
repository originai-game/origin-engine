//! A character's parts from its layered art: a PSD's layers, or a folder
//! of PNGs, each named for a part of the template.

use super::super::{psd, texture};
use super::template::Template;
use std::path::Path;

/// One part: its pixels trimmed to their alpha, where they sit on the canvas.
#[derive(Clone, Debug)]
pub struct Part {
    pub name: String,
    /// Where it came from, for a refusal: "layer `head` of hero.psd".
    pub source: String,
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    /// RGBA8, straight alpha, `width * height`.
    pub rgba: Vec<u8>,
}

impl Part {
    /// The alpha at a canvas pixel (0 outside the part).
    pub fn alpha(&self, x: i32, y: i32) -> u8 {
        let (lx, ly) = (x - self.left, y - self.top);
        if lx < 0 || ly < 0 || lx >= self.width as i32 || ly >= self.height as i32 {
            return 0;
        }
        self.rgba[((ly as u32 * self.width + lx as u32) * 4 + 3) as usize]
    }

    /// Canvas pixels whose alpha is at least half.
    pub fn solid(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (0..self.height as i32).flat_map(move |y| {
            (0..self.width as i32).filter_map(move |x| {
                let a = self.rgba[((y as u32 * self.width + x as u32) * 4 + 3) as usize];
                (a >= 128).then_some((self.left + x, self.top + y))
            })
        })
    }
}

/// The parts of one character, back to front.
#[derive(Debug)]
pub struct Art {
    /// The PSD's or the folder's asset id.
    pub label: String,
    /// The canvas's width, pixels.
    pub width: u32,
    pub parts: Vec<Part>,
}

impl Art {
    /// The height of what is drawn (the parts' union), pixels.
    pub fn drawn_height(&self) -> f32 {
        let top = self.parts.iter().map(|p| p.top).min().unwrap_or(0);
        let bottom = self
            .parts
            .iter()
            .map(|p| p.top + p.height as i32)
            .max()
            .unwrap_or(0);
        (bottom - top) as f32
    }

    pub fn part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name == name)
    }
}

/// A layer or file name as a part name: lower case, `_` and spaces as `-`.
pub fn normalise(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c == '_' || c.is_whitespace() {
                '-'
            } else {
                c
            }
        })
        .collect()
}

/// A folder's PNG parts in draw order (their names sorted), leaving out
/// those that start with `_`.
pub fn folder_files(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .map(|r| {
            r.filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter(|n| n.to_lowercase().ends_with(".png") && !n.starts_with('_'))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

/// A file's part name: its stem without a `NN-` draw-order prefix.
fn file_part(file: &str) -> String {
    let stem = &file[..file.len() - 4];
    let digits = stem.chars().take_while(char::is_ascii_digit).count();
    let rest = &stem[digits..];
    let stem = match rest.strip_prefix(['-', '_', ' ', '.']) {
        Some(r) if digits > 0 => r,
        _ => stem,
    };
    normalise(stem)
}

/// The edit distance between two names.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for j in 0..b.len() {
            let cur = row[j + 1];
            row[j + 1] = (prev + (ca != b[j]) as usize).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

/// "did you mean `x`? " for a name close to one of the template's.
fn suggestion(name: &str, t: &Template) -> String {
    t.parts
        .iter()
        .map(|p| (distance(name, p.name), p.name))
        .filter(|(d, _)| *d <= 3)
        .min()
        .map_or(String::new(), |(_, n)| format!("did you mean `{n}`? "))
}

fn unknown(source: &str, name: &str, t: &Template) -> String {
    format!(
        "{source}: `{name}` is not a part of the {} template; {}rename it to one of {}, or start its name with `_` to leave it out",
        t.name,
        suggestion(name, t),
        t.parts
            .iter()
            .map(|p| p.name)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// One layer or file before trimming: canvas placement and pixels.
struct Piece {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// Composites `pieces` (bottom first, normal blending) and trims the
/// result to its alpha; `None` when nothing is drawn.
fn flatten(pieces: &[Piece]) -> Option<(i32, i32, u32, u32, Vec<u8>)> {
    let l = pieces.iter().map(|p| p.left).min()?;
    let t = pieces.iter().map(|p| p.top).min()?;
    let r = pieces.iter().map(|p| p.left + p.width as i32).max()?;
    let b = pieces.iter().map(|p| p.top + p.height as i32).max()?;
    let (w, h) = ((r - l) as usize, (b - t) as usize);
    let mut out = vec![0f32; w * h * 4];
    for p in pieces {
        for y in 0..p.height as usize {
            for x in 0..p.width as usize {
                let s = &p.rgba[(y * p.width as usize + x) * 4..][..4];
                let sa = s[3] as f32 / 255.0;
                if sa == 0.0 {
                    continue;
                }
                let o = ((y + (p.top - t) as usize) * w + x + (p.left - l) as usize) * 4;
                let da = out[o + 3];
                let a = sa + da * (1.0 - sa);
                for c in 0..3 {
                    out[o + c] = (s[c] as f32 * sa + out[o + c] * da * (1.0 - sa)) / a;
                }
                out[o + 3] = a;
            }
        }
    }
    let solid = |i: usize| out[i * 4 + 3] > 0.0;
    let xs = (0..w).filter(|&x| (0..h).any(|y| solid(y * w + x)));
    let (x0, x1) = {
        let v: Vec<usize> = xs.collect();
        (*v.first()?, *v.last()?)
    };
    let ys: Vec<usize> = (0..h)
        .filter(|&y| (0..w).any(|x| solid(y * w + x)))
        .collect();
    let (y0, y1) = (*ys.first()?, *ys.last()?);
    let (tw, th) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut rgba = Vec::with_capacity(tw * th * 4);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let o = (y * w + x) * 4;
            let a = out[o + 3];
            for c in 0..3 {
                rgba.push(if a > 0.0 { out[o + c].round() as u8 } else { 0 });
            }
            rgba.push((a * 255.0).round() as u8);
        }
    }
    Some((l + x0 as i32, t + y0 as i32, tw as u32, th as u32, rgba))
}

/// Reads the parts `id` names (a `.psd`, or a folder ending in `/`) and
/// checks them against the template: every name one of its parts, each
/// part drawn once, nothing empty, every required part there.
pub fn read(root: &Path, id: &str, t: &Template, notes: &mut Vec<String>) -> Result<Art, String> {
    // (part, source, pieces), back to front by the first piece.
    let mut found: Vec<(String, String, Vec<Piece>)> = Vec::new();
    let width;
    if let Some(dir) = id.strip_suffix('/') {
        let files = folder_files(&root.join(dir));
        if files.is_empty() {
            return Err(format!(
                "{id}: no PNG parts in the folder; export each layer as a PNG named for its part (head.png, 01-arm-upper-l.png, ...)"
            ));
        }
        let mut size = None;
        for f in &files {
            let path = format!("{dir}/{f}");
            let bytes = std::fs::read(root.join(&path)).map_err(|e| format!("{path}: {e}"))?;
            let img = texture::decode(&bytes).map_err(|e| format!("{path}: {e}"))?;
            match size {
                None => size = Some((img.width, img.height, path.clone())),
                Some((w, h, ref first)) if (w, h) != (img.width, img.height) => {
                    return Err(format!(
                        "{path}: {}x{}, not the {w}x{h} of {first}; export every part at the canvas size, untrimmed, so each sits where it is drawn",
                        img.width, img.height
                    ));
                }
                _ => {}
            }
            let name = file_part(f);
            if t.spec(&name).is_none() {
                return Err(unknown(&path, &name, t));
            }
            if let Some(prev) = found.iter().find(|p| p.0 == name) {
                return Err(format!(
                    "{path}: a second `{name}` after {}; merge them into one PNG",
                    prev.1
                ));
            }
            found.push((
                name,
                path,
                vec![Piece {
                    left: 0,
                    top: 0,
                    width: img.width,
                    height: img.height,
                    rgba: img.rgba,
                }],
            ));
        }
        width = size.expect("a file").0;
    } else {
        let bytes =
            std::fs::read(root.join(id)).map_err(|_| format!("`{id}` is not in assets/"))?;
        if !id.to_lowercase().ends_with(".psd") && !id.to_lowercase().ends_with(".psb") {
            return Err(format!(
                "{id}: `parts` names a layered PSD (\"hero.psd\") or a folder of PNG parts (\"hero/\")"
            ));
        }
        let doc = psd::layers(&bytes).map_err(|e| format!("{id}: {e}"))?;
        width = doc.width;
        let mut hidden = Vec::new();
        for l in doc
            .layers
            .iter()
            .filter(|l| l.kind == psd::LayerKind::Pixels)
        {
            if l.name.starts_with('_') || l.groups.iter().any(|g| g.starts_with('_')) {
                continue;
            }
            if l.hidden {
                hidden.push(l.name.clone());
                continue;
            }
            // A group named for a part merges what it holds into that part.
            let group = l
                .groups
                .iter()
                .map(|g| normalise(g))
                .find(|g| t.spec(g).is_some());
            let (name, source) = match &group {
                Some(g) => (g.clone(), format!("group `{g}` of {id}")),
                None => (normalise(&l.name), format!("layer `{}` of {id}", l.name)),
            };
            if let Some(why) = &l.refusal {
                return Err(format!("{id}: layer `{}` {why}", l.name));
            }
            if t.spec(&name).is_none() {
                return Err(unknown(&format!("{id}: layer `{}`", l.name), &name, t));
            }
            let piece = Piece {
                left: l.left,
                top: l.top,
                width: l.width,
                height: l.height,
                rgba: l.rgba.clone(),
            };
            match found.iter_mut().find(|p| p.0 == name) {
                Some(p) if group.is_some() && p.1 == source => p.2.push(piece),
                Some(p) => {
                    return Err(format!(
                        "{id}: layer `{}` is a second `{name}` beside {}; merge them (Layer › Merge Layers) or put them in one group named `{name}`",
                        l.name, p.1
                    ));
                }
                None => found.push((name, source, vec![piece])),
            }
        }
        if !hidden.is_empty() {
            notes.push(format!(
                "{id}: hidden layers left out: {}",
                hidden.join(", ")
            ));
        }
    }
    let mut parts = Vec::new();
    for (name, source, pieces) in found {
        let Some((left, top, w, h, rgba)) = flatten(&pieces) else {
            return Err(format!(
                "{source}: `{name}` has no pixels; draw it, or delete or hide the layer"
            ));
        };
        parts.push(Part {
            name,
            source,
            left,
            top,
            width: w,
            height: h,
            rgba,
        });
    }
    let missing: Vec<String> = t
        .parts
        .iter()
        .filter(|s| s.required && !parts.iter().any(|p| p.name == s.name))
        .map(|s| format!("`{}` ({})", s.name, s.what))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{id}: the {} template needs {}; add {} named for {} ({})",
            t.name,
            missing.join(", "),
            if missing.len() == 1 {
                "a layer"
            } else {
                "layers"
            },
            if missing.len() == 1 { "it" } else { "them" },
            t.sides,
        ));
    }
    Ok(Art {
        label: id.to_string(),
        width,
        parts,
    })
}
