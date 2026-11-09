//! `og style art <pack> [--art <dir>] [--check]`: a style pack's painted
//! ornament art, from the requests it writes to the atlases it packs (docs/art.md
//! §2.7, docs/ui.md §3.3).
//!
//! The command never paints and never reaches the network. It does three
//! things, each rerunnable:
//!
//! 1. **Requests.** It completes `<pack>/art-requests.json` (what `og style
//!    new` writes) with what a fulfiller needs to paint each slot: the file
//!    it drops (`art/<part>.png`), the cell's size in CSS px,
//!    the least pixels to deliver (the 3× atlas's), and a hash of the request.
//!    A value already in the file wins,
//!    so a pack retunes a slot by editing it.
//! 2. **Verification.** It reads what arrived in the art directory
//!    (`<pack>/art/` unless `--art`): each slot's PNG and its entry in
//!    `provenance.json` (who painted it, from which prompt and references,
//!    for which request hash). Art without provenance is refused; art painted
//!    for an older request is reported stale; a slot with no art yet keeps
//!    the style's own ornament. `--check` stops here and fails on any of
//!    them.
//! 3. **Conversion and packing.** Each image is trimmed to the art
//!    and resampled to the cell's size at 1×, 2× and 3× (Lanczos
//!    over premultiplied alpha; a pixel style takes the dominant colour of
//!    each block at 1× and enlarges it by whole pixels).
//!    The pieces are shelved below the pack's own atlas as it was before
//!    any art (`art/base.png`, kept on the first run), so no region the
//!    style already cuts moves; `atlas.png`, `atlas@2x.png` and
//!    `atlas@3x.png` share the layout at 1, 2 and 3 texels per CSS px. The
//!    style's `atlas` line names the densities and each
//!    ornament line its region. `art/sheet.png` lays every piece out for
//!    review and `art/report.json` says what was packed from what.

use image::{Rgba, RgbaImage};
use og_core::ogt::{Document, Entry, Number, Value};
use serde_json::{Map, Value as Json, json};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: og style art <pack dir> [--art <dir>] [--check]";

/// The densities every packed atlas carries, texels per CSS px.
const DENSITIES: [u32; 3] = [1, 2, 3];

/// CSS px between packed pieces at 1× (each piece's edge is extruded into it).
const GAP: u32 = 2;

/// The ornament line a slot paints and its default size in CSS px.
struct Default {
    part: &'static str,
    size: [u32; 2],
}

const DEFAULTS: [Default; 5] = [
    Default {
        part: "corner",
        size: [32, 32],
    },
    Default {
        part: "divider",
        size: [192, 16],
    },
    Default {
        part: "flourish",
        size: [64, 24],
    },
    Default {
        part: "crest",
        size: [64, 32],
    },
    Default {
        part: "edge",
        size: [16, 64],
    },
];

/// One slot of the requests, completed.
struct Slot {
    /// `fantasy/corner`.
    name: String,
    /// `corner`.
    part: String,
    file: String,
    size: [u32; 2],
    hash: String,
}

/// A converted slot: its cells at each density, ready to pack.
struct Piece {
    slot: usize,
    /// The cell at 1×, 2× and 3×.
    images: [RgbaImage; 3],
    at: (u32, u32),
}

pub fn run(args: &[&str]) -> Result<(), String> {
    let mut pack: Option<PathBuf> = None;
    let mut art: Option<PathBuf> = None;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--art" => {
                i += 1;
                art = Some(PathBuf::from(args.get(i).ok_or(USAGE)?));
            }
            "--check" => check = true,
            a if !a.starts_with("--") && pack.is_none() => pack = Some(PathBuf::from(a)),
            _ => return Err(USAGE.into()),
        }
        i += 1;
    }
    let pack = pack.ok_or(USAGE)?;
    let art = art.unwrap_or_else(|| pack.join("art"));
    let style_path = pack.join("style.ogt");
    let style_text = std::fs::read_to_string(&style_path)
        .map_err(|e| format!("{}: {e}", style_path.display()))?;
    let doc =
        og_ui::read_style_doc(&style_text).map_err(|e| format!("{}: {e}", style_path.display()))?;
    if doc.version < 2 {
        return Err(format!(
            "{} is a version 1 style: og style upgrade it first",
            style_path.display()
        ));
    }
    if doc
        .body
        .get("scale")
        .and_then(|e| e.values.first())
        .and_then(Value::as_number)
        .is_some_and(|n| n.as_f64() != 1.0)
    {
        return Err("the style sets `scale`: painted art packs at one texel per CSS px, with denser atlases for sharper screens".into());
    }
    let file =
        og_ui::parse_style(&style_text).map_err(|e| format!("{}: {e}", style_path.display()))?;
    let name = doc
        .body
        .get("name")
        .and_then(|e| e.values.first())
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            pack.file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "style".into());

    // 1. The requests.
    let req_path = pack.join("art-requests.json");
    let old: Json = match std::fs::read_to_string(&req_path) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", req_path.display()))?,
        Err(_) => json!({}),
    };
    let (requests, slots) = complete_requests(&old, &name, &doc);
    let text = serde_json::to_string_pretty(&requests).expect("json") + "\n";
    if !check && std::fs::read_to_string(&req_path).ok().as_deref() != Some(text.as_str()) {
        std::fs::write(&req_path, &text).map_err(|e| format!("{}: {e}", req_path.display()))?;
        println!("og style art: {} written", req_path.display());
    }
    let pixel = requests["resample"] == "pixel";

    // 2. What arrived.
    let prov_path = art.join("provenance.json");
    let provenance: Map<String, Json> = match std::fs::read_to_string(&prov_path) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", prov_path.display()))?,
        Err(_) => Map::new(),
    };
    let mut problems: Vec<String> = Vec::new();
    let mut pending: Vec<&str> = Vec::new();
    let mut stale: Vec<&str> = Vec::new();
    let mut arrived: Vec<(usize, RgbaImage)> = Vec::new();
    for (k, s) in slots.iter().enumerate() {
        let path = art.join(format!("{}.png", s.part));
        if !path.exists() {
            pending.push(&s.name);
            continue;
        }
        let Some(p) = provenance.get(&s.name) else {
            problems.push(format!(
                "{}: no entry for {} in {} (who painted it, from what prompt and references, for which request)",
                path.display(),
                s.name,
                prov_path.display()
            ));
            continue;
        };
        for key in ["model", "prompt", "request"] {
            if p.get(key).is_none() {
                problems.push(format!(
                    "{}: {} has no `{key}`",
                    prov_path.display(),
                    s.name
                ));
            }
        }
        if p.get("request").and_then(Json::as_str) != Some(s.hash.as_str()) {
            stale.push(&s.name);
        }
        match image::open(&path) {
            Ok(img) => arrived.push((k, img.to_rgba8())),
            Err(e) => problems.push(format!("{}: {e}", path.display())),
        }
    }
    for s in &stale {
        println!(
            "og style art: {s} was painted for an older request (its size or note changed): repaint it or update its provenance"
        );
    }
    if !pending.is_empty() {
        println!(
            "og style art: no art yet for {} (the style keeps its own ornament there)",
            pending.join(", ")
        );
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    if check {
        if pending.is_empty() && stale.is_empty() {
            println!(
                "og style art: {} slots arrived, each with its provenance",
                arrived.len()
            );
            return Ok(());
        }
        return Err(format!(
            "{} slots without art, {} stale",
            pending.len(),
            stale.len()
        ));
    }
    if arrived.is_empty() {
        println!("og style art: nothing to pack yet");
        return Ok(());
    }

    // 3. Conversion.
    let mut pieces: Vec<Piece> = Vec::new();
    let mut report: Vec<Json> = Vec::new();
    for (k, img) in &arrived {
        let s = &slots[*k];
        let p = &provenance[&s.name];
        let (images, size) = convert(s, img, pixel)?;
        pieces.push(Piece {
            slot: *k,
            images,
            at: (0, 0),
        });
        report.push(json!({
            "slot": s.name,
            "from": s.file,
            "source": [img.width(), img.height()],
            "size": size,
            "to": [s.part],
            "provenance": p,
        }));
    }

    // 4. Packing, below the atlas the style had before any art.
    std::fs::create_dir_all(&art).map_err(|e| format!("{}: {e}", art.display()))?;
    let base_path = art.join("base.png");
    // A previous run's report says whether there was a base: a style that
    // had no atlas of its own has none, and its packed atlas is not one.
    let packed_before = std::fs::read_to_string(art.join("report.json")).is_ok();
    let base: Option<RgbaImage> = if base_path.exists() {
        Some(load(&base_path)?)
    } else if packed_before {
        None
    } else if let Some(id) = file.atlas.as_deref() {
        let path = pack.join(id);
        match image::open(&path) {
            Ok(img) => {
                let img = img.to_rgba8();
                img.save(&base_path)
                    .map_err(|e| format!("{}: {e}", base_path.display()))?;
                Some(img)
            }
            Err(_) => None,
        }
    } else {
        None
    };
    let (bw, bh) = base.as_ref().map_or((0, 0), |b| b.dimensions());
    let sizes: Vec<(u32, u32)> = pieces.iter().map(|p| p.images[0].dimensions()).collect();
    let (places, w, h) = shelve(&sizes, bw);
    for (p, at) in pieces.iter_mut().zip(places) {
        p.at = (at.0, bh + at.1);
    }
    let (w, h) = (w.max(bw), bh + h);
    let import = std::fs::read_to_string(pack.join("atlas.png.ogt")).unwrap_or_else(|_| {
        "type import\nversion 1\n---\n# A style atlas painted (og style art).\ntexture {\n  kind color\n}\n".into()
    });
    for (d, k) in DENSITIES.into_iter().enumerate() {
        let mut atlas = RgbaImage::new(w * k, h * k);
        if let Some(b) = &base {
            let up = scale_up(b, k, pixel);
            image::imageops::replace(&mut atlas, &up, 0, 0);
        }
        for p in &pieces {
            put(
                &mut atlas,
                &p.images[d],
                (p.at.0 * k, p.at.1 * k),
                GAP / 2 * k,
            );
        }
        let file_name = if k == 1 {
            "atlas.png".to_string()
        } else {
            og_ui::style_file::dense_name("atlas.png", k)
        };
        let path = pack.join(&file_name);
        atlas
            .save(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let settings = pack.join(format!("{file_name}.ogt"));
        if !settings.exists() {
            std::fs::write(&settings, &import)
                .map_err(|e| format!("{}: {e}", settings.display()))?;
        }
    }
    let mut text = packed_style(doc, &pieces, &slots, (w, h));
    // The style's leading comments (what it is, where its art comes from)
    // stay: the text format's writer keeps none.
    let comments: Vec<&str> = style_text
        .lines()
        .skip_while(|l| *l != "---")
        .skip(1)
        .take_while(|l| l.trim_start().starts_with('#'))
        .collect();
    if !comments.is_empty()
        && let Some(at) = text.find("---\n")
    {
        text.insert_str(at + 4, &(comments.join("\n") + "\n"));
    }
    og_ui::parse_style(&text).map_err(|e| format!("the packed style does not parse: {e}"))?;
    std::fs::write(&style_path, &text).map_err(|e| format!("{}: {e}", style_path.display()))?;
    let sheet = review_sheet(&pieces);
    let sheet_path = art.join("sheet.png");
    sheet
        .save(&sheet_path)
        .map_err(|e| format!("{}: {e}", sheet_path.display()))?;
    let report_path = art.join("report.json");
    let report = json!({
        "style": name,
        "atlas": [w, h],
        "densities": DENSITIES,
        "base": [bw, bh],
        "resample": if pixel { "pixel" } else { "smooth" },
        "slots": report,
        "pending": pending,
        "stale": stale,
    });
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).expect("json") + "\n",
    )
    .map_err(|e| format!("{}: {e}", report_path.display()))?;
    println!(
        "og style art: {} slots ({} pieces) into atlas.png {w}×{h} at 1×, 2× and 3×; {} updated; review {}",
        arrived.len(),
        pieces.len(),
        style_path.display(),
        sheet_path.display()
    );
    Ok(())
}

/// The requests with every slot completed, and the slots this command packs.
fn complete_requests(old: &Json, name: &str, doc: &Document) -> (Json, Vec<Slot>) {
    let mut out = old.as_object().cloned().unwrap_or_default();
    out.entry("style").or_insert_with(|| json!(name));
    let shape = old["choices"]["shape"].as_str().unwrap_or("");
    let pixel = shape == "pixel" || name == "pixel";
    out.entry("resample")
        .or_insert_with(|| json!(if pixel { "pixel" } else { "smooth" }));
    out.insert("densities".into(), json!(DENSITIES));
    let palette = doc
        .body
        .get("palette")
        .and_then(|e| e.block.as_ref())
        .map(|b| {
            b.entries
                .iter()
                .filter_map(|e| Some((e.key.clone(), json!(e.values.first()?.as_str()?))))
                .collect::<Map<_, _>>()
        })
        .unwrap_or_default();
    out.entry("palette")
        .or_insert_with(|| Json::Object(palette));
    let mut list: Vec<Json> = old["slots"].as_array().cloned().unwrap_or_default();
    // Only ornaments that the style draws or already requests are packed.
    for d in &DEFAULTS {
        let named = format!("{name}/{}", d.part);
        let wanted = doc.body.get(d.part).is_some();
        if wanted && !list.iter().any(|s| s["name"] == named.as_str()) {
            list.push(json!({ "name": named, "kind": "ornament" }));
        }
    }
    let mut slots = Vec::new();
    for s in list.iter_mut() {
        let Some(slot_name) = s["name"].as_str().map(str::to_string) else {
            continue;
        };
        let part = slot_name
            .rsplit('/')
            .next()
            .unwrap_or(&slot_name)
            .to_string();
        let Some(d) = DEFAULTS.iter().find(|d| d.part == part) else {
            println!(
                "og style art: {slot_name} skipped; only corner, divider, flourish, crest and edge ornaments are packed (part art belongs to CSS)"
            );
            continue;
        };
        let o = s.as_object_mut().expect("a slot is an object");
        // An ornament the style already cuts keeps its region's size, so the
        // size its line draws at keeps its proportions.
        let region = doc.body.get(d.part).and_then(|e| {
            let n = |i: usize| e.values.get(i)?.as_number().map(|n| n.as_f64() as u32);
            Some([n(2)?, n(3)?])
        });
        let size = o
            .get("size")
            .and_then(Json::as_array)
            .and_then(|a| Some([a.first()?.as_u64()? as u32, a.get(1)?.as_u64()? as u32]))
            .or(region)
            .unwrap_or(d.size);
        let file = format!("art/{part}.png");
        o.insert("file".into(), json!(file));
        o.insert("size".into(), json!(size));
        o.insert("states".into(), json!(["normal"]));
        o.insert("pixels".into(), json!([size[0] * 3, size[1] * 3]));
        o.remove("hash");
        let hash = format!(
            "{:016x}",
            og_core::hash::hash_bytes(serde_json::to_string(&*o).expect("json").as_bytes())
        );
        o.insert("hash".into(), json!(hash));
        slots.push(Slot {
            name: slot_name,
            part,
            file,
            size,
            hash,
        });
    }
    out.insert("slots".into(), Json::Array(list));
    (Json::Object(out), slots)
}

fn load(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|i| i.to_rgba8())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The ornament at each density and its size in CSS px.
fn convert(s: &Slot, img: &RgbaImage, pixel: bool) -> Result<([RgbaImage; 3], [u32; 2]), String> {
    let cell_aspect = f64::from(s.size[0]) / f64::from(s.size[1]);
    let (x0, y0, x1, y1) =
        alpha_box(img).ok_or_else(|| format!("{}: the art is fully transparent", s.file))?;
    let trimmed = image::imageops::crop_imm(img, x0, y0, x1 - x0, y1 - y0).to_image();
    // The ornament fits inside its box, keeping its proportions.
    let got = f64::from(x1 - x0) / f64::from(y1 - y0);
    let [w, h] = s.size;
    let size = if (got / cell_aspect - 1.0).abs() <= 0.12 {
        s.size
    } else if got > cell_aspect {
        [w, ((f64::from(w) / got).round() as u32).max(1)]
    } else {
        [((f64::from(h) * got).round() as u32).max(1), h]
    };
    if size != s.size {
        println!(
            "og style art: {} is {:.2}:1 where its cell is {:.2}:1: packed at {}×{} CSS px",
            s.name, got, cell_aspect, size[0], size[1]
        );
    }
    Ok((densities(&trimmed, size, pixel), size))
}

/// The box of pixels above a faint alpha: x0, y0, x1, y1 (exclusive).
fn alpha_box(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 8 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x1 > x0).then_some((x0, y0, x1, y1))
}

/// `img` at the cell's size at 1×, 2× and 3×.
fn densities(img: &RgbaImage, size: [u32; 2], pixel: bool) -> [RgbaImage; 3] {
    if pixel {
        let one = dominant(img, size[0], size[1]);
        return [
            one.clone(),
            scale_up(&one, 2, true),
            scale_up(&one, 3, true),
        ];
    }
    DENSITIES.map(|k| smooth(img, size[0] * k, size[1] * k))
}

/// Lanczos resampling over premultiplied alpha, so transparent texels add
/// no dark fringe.
fn smooth(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    if img.dimensions() == (w, h) {
        return img.clone();
    }
    let mut pre = image::Rgba32FImage::new(img.width(), img.height());
    for (x, y, p) in img.enumerate_pixels() {
        let a = f32::from(p[3]) / 255.0;
        let c = |v: u8| f32::from(v) / 255.0 * a;
        pre.put_pixel(x, y, Rgba([c(p[0]), c(p[1]), c(p[2]), a]));
    }
    let out = image::imageops::resize(&pre, w, h, image::imageops::FilterType::Lanczos3);
    let mut img = RgbaImage::new(w, h);
    for (x, y, p) in out.enumerate_pixels() {
        let a = p[3].clamp(0.0, 1.0);
        let c = |v: f32| {
            if a > 0.0 {
                ((v / a).clamp(0.0, 1.0) * 255.0).round() as u8
            } else {
                0
            }
        };
        img.put_pixel(
            x,
            y,
            Rgba([c(p[0]), c(p[1]), c(p[2]), (a * 255.0).round() as u8]),
        );
    }
    img
}

/// A pixel style's cell: each texel the most common colour of its block
/// of the source, its alpha cut to on or off.
fn dominant(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let mut out = RgbaImage::new(w, h);
    let (sw, sh) = img.dimensions();
    for y in 0..h {
        for x in 0..w {
            let (bx0, by0) = (x * sw / w, y * sh / h);
            let (bx1, by1) = (
                ((x + 1) * sw / w).max(bx0 + 1),
                ((y + 1) * sh / h).max(by0 + 1),
            );
            let mut counts: Vec<([u8; 4], u32)> = Vec::new();
            for yy in by0..by1.min(sh) {
                for xx in bx0..bx1.min(sw) {
                    let p = img.get_pixel(xx, yy).0;
                    // Quantised to 5 bits so the anti-aliased edges of a
                    // generated image vote with their colour.
                    let key = if p[3] < 128 {
                        [0, 0, 0, 0]
                    } else {
                        [p[0] & 0xf8, p[1] & 0xf8, p[2] & 0xf8, 255]
                    };
                    match counts.iter_mut().find(|(k, _)| *k == key) {
                        Some(c) => c.1 += 1,
                        None => counts.push((key, 1)),
                    }
                }
            }
            let best = counts
                .iter()
                .max_by_key(|(_, n)| *n)
                .map_or([0; 4], |(k, _)| *k);
            out.put_pixel(x, y, Rgba(best));
        }
    }
    out
}

/// `img` at `k` times its size: whole pixels for a pixel style, else smooth.
fn scale_up(img: &RgbaImage, k: u32, pixel: bool) -> RgbaImage {
    if k == 1 {
        return img.clone();
    }
    if pixel {
        return image::imageops::resize(
            img,
            img.width() * k,
            img.height() * k,
            image::imageops::FilterType::Nearest,
        );
    }
    smooth(img, img.width() * k, img.height() * k)
}

/// Places pieces of the given sizes on shelves, tallest first, in an atlas
/// at least `min_width` wide (the next power of two over the square of
/// their area): each piece's place and the shelves' size.
fn shelve(sizes: &[(u32, u32)], min_width: u32) -> (Vec<(u32, u32)>, u32, u32) {
    let area: u64 = sizes
        .iter()
        .map(|(w, h)| u64::from(w + GAP) * u64::from(h + GAP))
        .sum();
    let widest = sizes.iter().map(|(w, _)| w + GAP).max().unwrap_or(1);
    let side = ((area as f64).sqrt().ceil() as u32).max(widest);
    let width = side.next_power_of_two().max(min_width);
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(sizes[i].1), i));
    let mut places = vec![(0, 0); sizes.len()];
    let (mut x, mut y, mut shelf) = (GAP / 2, GAP / 2, 0);
    for i in order {
        let (w, h) = sizes[i];
        if x + w + GAP / 2 > width {
            x = GAP / 2;
            y += shelf + GAP;
            shelf = 0;
        }
        places[i] = (x, y);
        x += w + GAP;
        shelf = shelf.max(h);
    }
    (places, width, y + shelf + GAP / 2)
}

/// `img` into `atlas` at `at`, its edge texels extruded `e` into the gap.
fn put(atlas: &mut RgbaImage, img: &RgbaImage, at: (u32, u32), e: u32) {
    let (w, h) = atlas.dimensions();
    let (iw, ih) = img.dimensions();
    for y in 0..ih + 2 * e {
        for x in 0..iw + 2 * e {
            let sx = (x as i64 - e as i64).clamp(0, iw as i64 - 1) as u32;
            let sy = (y as i64 - e as i64).clamp(0, ih as i64 - 1) as u32;
            let (ax, ay) = (at.0 + x, at.1 + y);
            if ax >= e && ay >= e && ax - e < w && ay - e < h {
                atlas.put_pixel(ax - e, ay - e, *img.get_pixel(sx, sy));
            }
        }
    }
}

fn num(v: f64) -> Value {
    Value::Number(Number::from_f64(v).expect("finite"))
}

/// The style file with its atlas densities and painted ornament regions.
fn packed_style(mut doc: Document, pieces: &[Piece], slots: &[Slot], size: (u32, u32)) -> String {
    let body = &mut doc.body;
    let mut atlas = Entry::new("atlas");
    atlas.values = vec![
        Value::String("atlas.png".into()),
        num(f64::from(size.0)),
        num(f64::from(size.1)),
        num(2.0),
        num(3.0),
    ];
    match body.entries.iter_mut().find(|e| e.key == "atlas") {
        Some(e) => *e = atlas,
        None => body.entries.insert(0, atlas),
    }
    let region = |p: &Piece| {
        let (w, h) = p.images[0].dimensions();
        [p.at.0, p.at.1, w, h].map(|v| num(f64::from(v)))
    };
    for p in pieces {
        let key = &slots[p.slot].part;
        let (w, h) = p.images[0].dimensions();
        match body.entries.iter_mut().find(|e| &e.key == key) {
            Some(e) => {
                // The size it draws at keeps the scale it had to its region,
                // in the new region's proportions.
                let n = |i: usize| {
                    e.values
                        .get(i)
                        .and_then(Value::as_number)
                        .map(Number::as_f64)
                };
                let mut rest: Vec<Value> = e.values.iter().skip(4).cloned().collect();
                if let (Some(rw), Some(rh), Some(dw), Some(dh)) = (n(2), n(3), n(4), n(5)) {
                    let k = (dw / rw).min(dh / rh);
                    let r = |v: u32| num((f64::from(v) * k * 10.0).round() / 10.0);
                    rest[0] = r(w);
                    rest[1] = r(h);
                }
                e.values = region(p).to_vec();
                e.values.extend(rest);
            }
            None => {
                let mut e = Entry::new(key);
                e.values = region(p).to_vec();
                e.values.extend([
                    num(f64::from(w)),
                    num(f64::from(h)),
                    Value::Word("authored".into()),
                ]);
                body.push(e);
            }
        }
    }
    og_ui::write_style_doc(&doc).expect("a style file writes back")
}

/// Every piece at 2× on a neutral ground, in rows by slot, for review.
fn review_sheet(pieces: &[Piece]) -> RgbaImage {
    const PAD: u32 = 16;
    let mut rows: Vec<Vec<&Piece>> = Vec::new();
    for p in pieces {
        match rows.last_mut() {
            Some(r) if r[0].slot == p.slot => r.push(p),
            _ => rows.push(vec![p]),
        }
    }
    let width = rows
        .iter()
        .map(|r| r.iter().map(|p| p.images[1].width() + PAD).sum::<u32>() + PAD)
        .max()
        .unwrap_or(PAD);
    let height = rows
        .iter()
        .map(|r| r.iter().map(|p| p.images[1].height()).max().unwrap_or(0) + PAD)
        .sum::<u32>()
        + PAD;
    let mut sheet = RgbaImage::from_pixel(width, height, Rgba([128, 128, 128, 255]));
    // A checker under the art shows its transparency.
    for (x, y, p) in sheet.enumerate_pixels_mut() {
        if ((x / 8) + (y / 8)) % 2 == 0 {
            *p = Rgba([150, 150, 150, 255]);
        }
    }
    let mut y = PAD;
    for r in rows {
        let mut x = PAD;
        let mut tall = 0;
        for p in r {
            image::imageops::overlay(&mut sheet, &p.images[1], i64::from(x), i64::from(y));
            x += p.images[1].width() + PAD;
            tall = tall.max(p.images[1].height());
        }
        y += tall + PAD;
    }
    sheet
}
