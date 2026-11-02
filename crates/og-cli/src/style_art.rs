//! `og style art <pack> [--art <dir>] [--check]`: a style pack's painted
//! art, from the requests it writes to the atlases it packs (docs/art.md
//! §2.4, docs/ui.md §12.6).
//!
//! The command never paints and never reaches the network. It does three
//! things, each rerunnable:
//!
//! 1. **Requests.** It completes `<pack>/art-requests.json` (what `og style
//!    new` writes) with what a fulfiller needs to paint each slot: the file
//!    it drops (`art/<part>.png`), the cell's size in CSS px, the nine-slice
//!    insets as fractions, the states a button sheet draws left to right,
//!    the least pixels to deliver (the 3× atlas's), the dressings the art
//!    goes to and a hash of the request. A value already in the file wins,
//!    so a pack retunes a slot by editing it.
//! 2. **Verification.** It reads what arrived in the art directory
//!    (`<pack>/art/` unless `--art`): each slot's PNG and its entry in
//!    `provenance.json` (who painted it, from which prompt and references,
//!    for which request hash). Art without provenance is refused; art painted
//!    for an older request is reported stale; a slot with no art yet keeps
//!    the style's own dressing. `--check` stops here and fails on any of
//!    them.
//! 3. **Conversion and packing.** Each image is cut into its state cells,
//!    trimmed to the art (one box for every cell, so the states stay
//!    aligned), and resampled to the cell's size at 1×, 2× and 3× (Lanczos
//!    over premultiplied alpha; a pixel style takes the dominant colour of
//!    each block at 1× and enlarges it by whole pixels). A button delivered
//!    as one cell has its other states derived (lighter, darker, muted).
//!    The pieces are shelved below the pack's own atlas as it was before
//!    any art (`art/base.png`, kept on the first run), so no region the
//!    style already cuts moves; `atlas.png`, `atlas@2x.png` and
//!    `atlas@3x.png` share the layout at 1, 2 and 3 texels per CSS px. The
//!    style's `atlas` line names the densities, each painted part's `dress`
//!    block gets its `slice` (authored) and the `tone` of its centre, and an
//!    ornament line its region. `art/sheet.png` lays every piece out for
//!    review and `art/report.json` says what was packed from what.

use image::{Rgba, RgbaImage};
use og_core::ogt::{Block, Document, Entry, Number, Value};
use serde_json::{Map, Value as Json, json};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: og style art <pack dir> [--art <dir>] [--check]";

/// The densities every packed atlas carries, texels per CSS px.
const DENSITIES: [u32; 3] = [1, 2, 3];

/// CSS px between packed pieces at 1× (each piece's edge is extruded into it).
const GAP: u32 = 2;

/// The states of a button sheet, left to right (og-ui's `BUTTON_STATES`):
/// the dressing qualifier each one goes to; `focused` is drawn by the
/// style's focus ring, so its cell is read but not packed.
const BUTTON_STATES: [(&str, Option<&str>); 5] = [
    ("normal", Some("")),
    ("hover", Some("[hover]")),
    ("pressed", Some("[pressed]")),
    ("disabled", Some("[disabled]")),
    ("focused", None),
];

/// What a slot paints by default: its dressings (by state for a button) or
/// an ornament line, its cell in CSS px and its insets in CSS px.
struct Default {
    part: &'static str,
    target: Target,
    size: [u32; 2],
    insets: [u32; 4],
}

#[derive(Clone, Copy, PartialEq)]
enum Target {
    /// A nine-slice dressing (`Panel.frame`).
    Frame(&'static str),
    /// A button sheet: the dressing every state qualifies (`Button.base[secondary]`).
    Button(&'static str),
    /// An ornament line of the style (`corner`, `divider`, `flourish`).
    Ornament(&'static str),
}

const DEFAULTS: [Default; 11] = [
    Default {
        part: "panel",
        target: Target::Frame("Panel.frame"),
        size: [96, 96],
        insets: [24, 24, 24, 24],
    },
    Default {
        part: "card",
        target: Target::Frame("Card.frame"),
        size: [96, 96],
        insets: [20, 20, 20, 20],
    },
    Default {
        part: "field",
        target: Target::Frame("Field.frame"),
        size: [96, 40],
        insets: [12, 12, 12, 12],
    },
    Default {
        part: "slot",
        target: Target::Frame("Slot.frame"),
        size: [64, 64],
        insets: [16, 16, 16, 16],
    },
    Default {
        part: "tooltip",
        target: Target::Frame("Tooltip.frame"),
        size: [64, 48],
        insets: [12, 12, 12, 12],
    },
    Default {
        part: "hud",
        target: Target::Frame("Hud.frame"),
        size: [64, 40],
        insets: [14, 12, 14, 12],
    },
    // The plain button art dresses the secondary variant: the quiet and
    // ghost ones stay bare, and a danger one keeps the style's colour.
    Default {
        part: "button",
        target: Target::Button("Button.base[secondary]"),
        size: [96, 44],
        insets: [18, 14, 18, 14],
    },
    Default {
        part: "button-primary",
        target: Target::Button("Button.base[primary]"),
        size: [96, 44],
        insets: [18, 14, 18, 14],
    },
    Default {
        part: "corner",
        target: Target::Ornament("corner"),
        size: [32, 32],
        insets: [0; 4],
    },
    Default {
        part: "divider",
        target: Target::Ornament("divider"),
        size: [192, 16],
        insets: [0; 4],
    },
    Default {
        part: "flourish",
        target: Target::Ornament("flourish"),
        size: [64, 24],
        insets: [0; 4],
    },
];

/// One slot of the requests, completed.
struct Slot {
    /// `fantasy/panel`.
    name: String,
    /// `panel`.
    part: String,
    file: String,
    size: [u32; 2],
    /// Fractions of the cell: left, top, right, bottom (0 for an ornament).
    insets: [f64; 4],
    /// The dressings each state cell goes to (one for a frame, four for a
    /// button) or the ornament line it sets.
    target: Target,
    dress: Vec<(usize, String)>,
    hash: String,
}

/// A converted slot: its cells at each density, ready to pack.
struct Piece {
    slot: usize,
    /// The dressing (or ornament key) this cell goes to.
    to: String,
    /// The cell at 1×, 2× and 3×.
    images: [RgbaImage; 3],
    insets: [u32; 4],
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
    let doc = Document::parse(&style_text).map_err(|e| format!("{}: {e}", style_path.display()))?;
    if doc.version < 2 {
        return Err(format!(
            "{} is a version 1 style: og style upgrade it first (painted art dresses parts by selector)",
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
            "og style art: {s} was painted for an older request (its size, insets or dressings changed): repaint it or update its provenance"
        );
    }
    if !pending.is_empty() {
        println!(
            "og style art: no art yet for {} (the style keeps its own dressing there)",
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
        let insets = p
            .get("insets")
            .and_then(Json::as_array)
            .filter(|a| a.len() == 4)
            .map(|a| {
                let v: Vec<f64> = a.iter().filter_map(Json::as_f64).collect();
                [v[0], v[1], v[2], v[3]]
            })
            .unwrap_or(s.insets);
        let (cells, derived) = convert(s, img, pixel)?;
        let px = |f: f64, len: u32| (f * f64::from(len)).round() as u32;
        let mut ins = [
            px(insets[0], s.size[0]),
            px(insets[1], s.size[1]),
            px(insets[2], s.size[0]),
            px(insets[3], s.size[1]),
        ];
        if !matches!(s.target, Target::Ornament(_)) {
            // A middle of at least one CSS px each way, corners of at least one.
            for v in &mut ins {
                *v = (*v).max(1);
            }
            ins[2] = ins[2].min(s.size[0].saturating_sub(ins[0] + 1)).max(1);
            ins[3] = ins[3].min(s.size[1].saturating_sub(ins[1] + 1)).max(1);
        }
        let mut tos = Vec::new();
        for (cell, images) in cells {
            let Some(to) = s
                .dress
                .iter()
                .find(|(c, _)| *c == cell)
                .map(|(_, d)| d.clone())
            else {
                continue;
            };
            tos.push(to.clone());
            pieces.push(Piece {
                slot: *k,
                to,
                images,
                insets: ins,
                at: (0, 0),
            });
        }
        report.push(json!({
            "slot": s.name,
            "from": s.file,
            "source": [img.width(), img.height()],
            "size": s.size,
            "insets": ins,
            "to": tos,
            "derived": derived,
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
    let text = packed_style(doc, &pieces, &slots, (w, h));
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
    // Every default slot is requested, the ornaments only where the style
    // draws one (a line of that key) or already asks for it.
    for d in &DEFAULTS {
        let named = format!("{name}/{}", d.part);
        let wanted = match d.target {
            Target::Ornament(key) => {
                doc.body.get(key).is_some() || list.iter().any(|s| s["name"] == named.as_str())
            }
            _ => true,
        };
        if wanted && !list.iter().any(|s| s["name"] == named.as_str()) {
            let kind = match d.target {
                Target::Frame(_) => "styleFrame",
                Target::Button(_) => "styleButton",
                Target::Ornament(_) => "ornament",
            };
            list.push(json!({ "name": named, "kind": kind }));
        }
    }
    let pill = shape == "pill";
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
            // An icon set and a game's own slots are not atlas art.
            continue;
        };
        let o = s.as_object_mut().expect("a slot is an object");
        let size = o
            .get("size")
            .and_then(Json::as_array)
            .and_then(|a| Some([a.first()?.as_u64()? as u32, a.get(1)?.as_u64()? as u32]))
            .unwrap_or(d.size);
        let mut px = d.insets;
        if pill && matches!(d.target, Target::Button(_)) || pill && d.part == "field" {
            px[0] = size[1] / 2;
            px[2] = size[1] / 2;
        }
        let insets = o
            .get("insets")
            .and_then(Json::as_array)
            .filter(|a| a.len() == 4)
            .map(|a| {
                let v: Vec<f64> = a.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
                [v[0], v[1], v[2], v[3]]
            })
            .unwrap_or_else(|| {
                let f = |p: u32, l: u32| (f64::from(p) / f64::from(l) * 1000.0).round() / 1000.0;
                [
                    f(px[0], size[0]),
                    f(px[1], size[1]),
                    f(px[2], size[0]),
                    f(px[3], size[1]),
                ]
            });
        let dress: Vec<(usize, String)> = match o.get("dress").and_then(Json::as_array) {
            Some(a) => a
                .iter()
                .enumerate()
                .filter_map(|(i, v)| Some((i, v.as_str()?.to_string())))
                .filter(|(_, v)| !v.is_empty())
                .collect(),
            None => match d.target {
                Target::Frame(sel) => vec![(0, sel.to_string())],
                Target::Button(sel) => BUTTON_STATES
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (_, q))| Some((i, format!("{sel}{}", (*q)?))))
                    .collect(),
                Target::Ornament(key) => vec![(0, key.to_string())],
            },
        };
        let states: Vec<&str> = match d.target {
            Target::Button(_) => BUTTON_STATES.iter().map(|(s, _)| *s).collect(),
            _ => vec!["normal"],
        };
        let file = format!("art/{part}.png");
        o.insert("file".into(), json!(file));
        o.insert("size".into(), json!(size));
        if !matches!(d.target, Target::Ornament(_)) {
            o.insert("insets".into(), json!(insets));
        }
        o.insert("states".into(), json!(states));
        o.insert(
            "pixels".into(),
            json!([size[0] * 3 * states.len() as u32, size[1] * 3]),
        );
        // The dressings a sheet's cells go to, left to right ("" for a
        // cell read but not packed: a button's focused one).
        let mut by_cell = vec![String::new(); states.len()];
        for (c, sel) in &dress {
            if let Some(v) = by_cell.get_mut(*c) {
                v.clone_from(sel);
            }
        }
        o.insert("dress".into(), json!(by_cell));
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
            insets,
            target: d.target,
            dress,
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

/// The slot's cells at each density, by cell index, and the states derived
/// (a button delivered as one cell).
#[allow(clippy::type_complexity)]
fn convert(
    s: &Slot,
    img: &RgbaImage,
    pixel: bool,
) -> Result<(Vec<(usize, [RgbaImage; 3])>, Vec<&'static str>), String> {
    let states = match s.target {
        Target::Button(_) => BUTTON_STATES.len() as u32,
        _ => 1,
    };
    let cell_aspect = f64::from(s.size[0]) / f64::from(s.size[1]);
    let aspect = f64::from(img.width()) / f64::from(img.height());
    // A sheet is as many cells as states, side by side; a single cell is
    // read as the normal state.
    let n = if states > 1 && (aspect / (cell_aspect * f64::from(states)) - 1.0).abs() < 0.35 {
        states
    } else {
        1
    };
    let cw = img.width() / n;
    let cells: Vec<RgbaImage> = (0..n)
        .map(|c| image::imageops::crop_imm(img, c * cw, 0, cw, img.height()).to_image())
        .collect();
    // One trim box for every cell, so the states keep their places.
    let mut bbox: Option<(u32, u32, u32, u32)> = None;
    for c in &cells {
        if let Some(b) = alpha_box(c) {
            bbox = Some(match bbox {
                None => b,
                Some(a) => (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)),
            });
        }
    }
    let (x0, y0, x1, y1) =
        bbox.ok_or_else(|| format!("{}: the art is fully transparent", s.file))?;
    let trimmed: Vec<RgbaImage> = cells
        .iter()
        .map(|c| image::imageops::crop_imm(c, x0, y0, x1 - x0, y1 - y0).to_image())
        .collect();
    let got = f64::from(x1 - x0) / f64::from(y1 - y0);
    if (got / cell_aspect - 1.0).abs() > 0.12 {
        println!(
            "og style art: {} is {:.2}:1 where its cell is {:.2}:1; its corners are stretched to fit: paint it at the requested size",
            s.name, got, cell_aspect
        );
    }
    let mut out = Vec::new();
    for (i, c) in trimmed.iter().enumerate() {
        out.push((i, densities(c, s.size, pixel)));
    }
    let mut derived = Vec::new();
    if states > 1 && n == 1 {
        let base = out[0].1.clone();
        for (i, (state, _)) in BUTTON_STATES.iter().enumerate().skip(1) {
            let f = |img: &RgbaImage| derive(img, state);
            out.push((i, [f(&base[0]), f(&base[1]), f(&base[2])]));
            derived.push(*state);
        }
    }
    Ok((out, derived))
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

/// A button state from its normal cell: hover lighter, pressed darker,
/// disabled muted, focused as it is (the ring is the style's).
fn derive(img: &RgbaImage, state: &str) -> RgbaImage {
    let mut out = img.clone();
    for p in out.pixels_mut() {
        let [r, g, b, a] = p.0.map(f32::from);
        let (r, g, b) = match state {
            "hover" => (
                r + (255.0 - r) * 0.14,
                g + (255.0 - g) * 0.14,
                b + (255.0 - b) * 0.14,
            ),
            "pressed" => (r * 0.8, g * 0.8, b * 0.8),
            "disabled" => {
                let l = 0.299 * r + 0.587 * g + 0.114 * b;
                let m = |v: f32| (v + (l - v) * 0.75) * 0.85;
                (m(r), m(g), m(b))
            }
            _ => (r, g, b),
        };
        *p = Rgba([r, g, b, a].map(|v| v.round().clamp(0.0, 255.0) as u8));
    }
    out
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

/// The mean of a piece's centre (between its insets) in linear light over
/// its coverage, and its mean alpha: the `tone` a widget reads its label
/// ink against.
fn tone(img: &RgbaImage, insets: [u32; 4]) -> [f32; 4] {
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (w, h) = img.dimensions();
    let (x0, y0) = (insets[0].min(w - 1), insets[1].min(h - 1));
    let (x1, y1) = (
        w.saturating_sub(insets[2]).max(x0 + 1),
        h.saturating_sub(insets[3]).max(y0 + 1),
    );
    let (mut rgb, mut alpha, mut n) = ([0.0f64; 3], 0.0f64, 0.0f64);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y).0;
            let a = f64::from(p[3]) / 255.0;
            for k in 0..3 {
                rgb[k] += f64::from(lin(f32::from(p[k]) / 255.0)) * a;
            }
            alpha += a;
            n += 1.0;
        }
    }
    let c = rgb.map(|v| (v / alpha.max(1e-6)) as f32);
    [c[0], c[1], c[2], (alpha / n.max(1.0)) as f32]
}

fn num(v: f64) -> Value {
    Value::Number(Number::from_f64(v).expect("finite"))
}

/// The style file with its atlas line naming the densities, each painted
/// dressing's slice and tone, and each painted ornament's region.
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
        match slots[p.slot].target {
            Target::Ornament(key) => {
                let (w, h) = p.images[0].dimensions();
                match body.entries.iter_mut().find(|e| e.key == key) {
                    Some(e) => {
                        let rest: Vec<Value> = e.values.iter().skip(4).cloned().collect();
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
            Target::Frame(_) | Target::Button(_) => {
                let i = match body.entries.iter().position(|e| {
                    e.key == "dress" && e.values.first().and_then(Value::as_str) == Some(&p.to)
                }) {
                    Some(i) => i,
                    None => {
                        body.push(
                            Entry::new("dress")
                                .with(Value::String(p.to.clone()))
                                .with_block(Block::new()),
                        );
                        body.entries.len() - 1
                    }
                };
                let block = body.entries[i].block.get_or_insert_with(Block::new);
                let mut slice = Entry::new("slice");
                slice.values = region(p).to_vec();
                slice.values.extend(p.insets.map(|v| num(f64::from(v))));
                slice.values.push(Value::Word("authored".into()));
                // Its fill words (stretch, repeat, round) stay; painted art is authored.
                if let Some(old) = block.entries.iter().find(|e| e.key == "slice") {
                    slice.values.extend(
                        old.values
                            .iter()
                            .filter(|v| matches!(v.as_word(), Some("stretch" | "repeat" | "round")))
                            .cloned(),
                    );
                }
                let t = tone(&p.images[0], p.insets);
                let mut tone_e = Entry::new("tone");
                tone_e.values = t
                    .iter()
                    .enumerate()
                    .map(|(k, v)| {
                        let d = if k == 3 { 100.0 } else { 1000.0 };
                        num((f64::from(*v) * d).round() / d)
                    })
                    .collect();
                block
                    .entries
                    .retain(|e| e.key != "slice" && e.key != "tone");
                block.entries.insert(0, tone_e);
                block.entries.insert(0, slice);
            }
        }
    }
    doc.to_text().expect("a style file writes back")
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
