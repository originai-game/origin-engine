//! Checked TypeScript flipbooks become native sheets, using explicit frames,
//! grids or captured Aseprite/TexturePacker JSON exchange data. Texture refs
//! bind actual graph outputs; animated input paths are module-relative.
//!
//! A sheet with `array true` also cooks into a texture array: its frames,
//! cut out of the texture, are the layers of `<id>#array` (an
//! `og.TextureArray` whose layers are the textures `<id>#array#layer<n>`,
//! pixel art under `filter nearest`, colour otherwise), so the runtime
//! draws frame `n` as layer `n`. Every frame must then have one size, and
//! a sheet has at most [`super::texture_array::MAX_LAYERS`] of them.

use super::source_schemas::media::clips as input;
use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets, conform_off, stamp,
    texture_array,
};
use crate::format::{self, *};
use og_anim::{FlipClip, FlipMode, FlipbookSheet, SheetFilter, SheetFrame};
use og_blob::BlobWriter;

/// The hashes of what the sheet depends on (its texture and atlas files),
/// for the cook stamp: editing either recooks the sheet.
pub(crate) fn dependency_hashes(s: &Source) -> String {
    s.flipbook.as_ref().map_or_else(String::new, |sheet| {
        let mut native = Vec::new();
        sheet.encode(&mut native);
        let mut hash = og_core::hash::StableHasher::new();
        hash.write(&native);
        if let Some(image) = &s.conformed {
            hash.write(image);
        }
        format!("{}={:016x}", sheet.texture, hash.finish())
    })
}

/// The prepared image of the texture `id` the sheet names, updated alongside
/// its native frame rectangles by the art contract (`conform/texture2d.rs`).
fn texture_bytes(s: &Source, id: &str) -> Result<Vec<u8>, String> {
    s.conformed
        .as_ref()
        .map(|b| b.to_vec())
        .ok_or_else(|| format!("texture `{id}` has no prepared image"))
}

/// Splits a name into text and number runs for a natural order ("run 2" before "run 10").
fn natural(name: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut num: Option<u64> = None;
    for ch in name.chars() {
        if let Some(d) = ch.to_digit(10) {
            num = Some(
                num.unwrap_or(0)
                    .saturating_mul(10)
                    .saturating_add(u64::from(d)),
            );
        } else {
            if let Some(n) = num.take() {
                out.push((std::mem::take(&mut text), n));
            }
            text.push(ch);
        }
    }
    out.push((text, num.unwrap_or(0)));
    out
}

struct AtlasFrame {
    name: String,
    rect: [u32; 4],
    duration: Option<f32>,
    pivot: Option<[f32; 2]>,
}

fn json_u32(v: &serde_json::Value, k: &str) -> Result<u32, String> {
    v[k].as_f64()
        .filter(|x| *x >= 0.0 && x.fract() == 0.0)
        .map(|x| x as u32)
        .ok_or_else(|| format!("`{k}` is not a whole number"))
}

/// Frames and tags of an Aseprite or TexturePacker JSON, in sheet order.
fn read_atlas(text: &str) -> Result<(Vec<AtlasFrame>, Vec<input::Clip>), String> {
    let json: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let frame_of = |name: String, v: &serde_json::Value| -> Result<AtlasFrame, String> {
        let f = &v["frame"];
        let rect = [
            json_u32(f, "x")?,
            json_u32(f, "y")?,
            json_u32(f, "w")?,
            json_u32(f, "h")?,
        ];
        if v["rotated"].as_bool() == Some(true) {
            return Err(format!(
                "frame `{name}` is rotated in the atlas; export without rotation"
            ));
        }
        // TexturePacker pivots run from the top left; sheets from the bottom left.
        let pivot = v["pivot"].as_object().and_then(|p| {
            Some([
                p.get("x")?.as_f64()? as f32,
                1.0 - p.get("y")?.as_f64()? as f32,
            ])
        });
        Ok(AtlasFrame {
            name,
            rect,
            duration: v["duration"].as_f64().map(|ms| (ms / 1000.0) as f32),
            pivot,
        })
    };
    let mut frames = Vec::new();
    match &json["frames"] {
        serde_json::Value::Array(list) => {
            for (i, v) in list.iter().enumerate() {
                let name = v["filename"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| i.to_string());
                frames.push(frame_of(name, v).map_err(|e| format!("frames[{i}]: {e}"))?);
            }
        }
        serde_json::Value::Object(map) => {
            // A hash: in the natural order of its names (Aseprite numbers them).
            let mut names: Vec<&String> = map.keys().collect();
            names.sort_by_key(|n| natural(n));
            for n in names {
                frames.push(frame_of(n.clone(), &map[n]).map_err(|e| format!("frame `{n}`: {e}"))?);
            }
        }
        _ => return Err("no `frames` (an Aseprite or TexturePacker JSON export)".into()),
    }
    // Aseprite tags become clips with the frames' own durations.
    let mut clips = Vec::new();
    for t in json["meta"]["frameTags"].as_array().into_iter().flatten() {
        let name = t["name"].as_str().ok_or("a frame tag without a name")?;
        let (from, to) = (json_u32(t, "from")?, json_u32(t, "to")?);
        if from > to || to as usize >= frames.len() {
            return Err(format!(
                "tag `{name}` runs {from}..{to} of {} frames",
                frames.len()
            ));
        }
        let mut idx: Vec<u32> = (from..=to).collect();
        let mode = match t["direction"].as_str().unwrap_or("forward") {
            "forward" => input::FlipMode::Loop,
            "reverse" => {
                idx.reverse();
                input::FlipMode::Loop
            }
            "pingpong" | "pingpong_reverse" => input::FlipMode::PingPong,
            d => return Err(format!("tag `{name}`: direction `{d}`")),
        };
        clips.push(input::Clip {
            name: name.into(),
            frames: idx
                .iter()
                .map(|&i| frames[i as usize].name.clone())
                .collect(),
            fps: None,
            durations: Some(
                idx.iter()
                    .map(|&i| frames[i as usize].duration.unwrap_or(0.1))
                    .collect(),
            ),
            mode: Some(mode),
            events: Vec::new(),
        });
    }
    Ok((frames, clips))
}

/// The canonical sheet of a flipbook source.
pub(crate) fn sheet(s: &Source) -> Result<og_anim::FlipbookSheet, String> {
    s.flipbook
        .clone()
        .ok_or_else(|| "missing checked flipbook".into())
}

pub(crate) fn filter(f: Option<&input::Filter>) -> SheetFilter {
    match f {
        Some(input::Filter::Linear) => SheetFilter::Linear,
        _ => SheetFilter::Nearest,
    }
}

pub(crate) fn mode(m: Option<&input::FlipMode>) -> FlipMode {
    match m {
        Some(input::FlipMode::Once) => FlipMode::Once,
        Some(input::FlipMode::PingPong) => FlipMode::PingPong,
        _ => FlipMode::Loop,
    }
}

pub(crate) fn clips(sheet: &mut FlipbookSheet, inputs: &[input::Clip]) -> Result<(), String> {
    let names: std::collections::BTreeMap<_, _> = sheet
        .frames
        .iter()
        .enumerate()
        .map(|(i, f)| (f.name.as_str(), i as u32))
        .collect();
    for c in inputs {
        let frames = c
            .frames
            .iter()
            .map(|name| {
                names
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| format!("clip '{}': no frame '{name}'", c.name))
            })
            .collect::<Result<Vec<_>, _>>()?;
        sheet.clips.push(FlipClip {
            name: c.name.clone(),
            durations: c
                .durations
                .clone()
                .unwrap_or_else(|| vec![1.0 / c.fps.unwrap_or(12.0); frames.len()]),
            frames,
            mode: mode(c.mode.as_ref()),
            events: c.events.clone(),
        });
    }
    Ok(())
}

/// Resolve an explicit texture/atlas declaration directly into native fields.
pub(crate) fn declaration(
    input: input::TextureSheet,
    texture: String,
    bytes: &[u8],
) -> Result<FlipbookSheet, String> {
    let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let mut sheet = FlipbookSheet {
        texture,
        size: [image.width(), image.height()],
        filter: filter(input.filter.as_ref()),
        array: input.array,
        frames: Vec::new(),
        clips: Vec::new(),
    };
    let pivot = input.pivot.unwrap_or([0.5, 0.5]);
    if let Some(grid) = input.grid {
        let [fw, fh] = grid.size;
        let step = |n: u32| n.checked_add(input.spacing).ok_or("grid spacing overflow");
        let (sx, sy) = (step(fw)?, step(fh)?);
        let count = |n: u32, size: u32| {
            n.checked_sub(input.margin)
                .and_then(|n| n.checked_add(input.spacing))
                .map(|n| n / size)
                .ok_or("grid margin or spacing overflow")
        };
        let columns = grid.columns.unwrap_or(count(sheet.size[0], sx)?);
        let rows = grid.rows.unwrap_or(count(sheet.size[1], sy)?);
        for [n, step, side, limit] in [
            [columns, sx, fw, sheet.size[0]],
            [rows, sy, fh, sheet.size[1]],
        ] {
            if n > 0
                && u64::from(n - 1) * u64::from(step) + u64::from(input.margin) + u64::from(side)
                    > u64::from(limit)
            {
                return Err("grid frames are not inside the texture".into());
            }
        }
        if u64::from(columns) * u64::from(rows) > u64::from(u32::MAX) {
            return Err("sheet frame indices must fit u32".into());
        }
        for y in 0..rows {
            for x in 0..columns {
                let at = |n: u32, s: u32| {
                    n.checked_mul(s)
                        .and_then(|n| n.checked_add(input.margin))
                        .ok_or("grid position overflow")
                };
                sheet.frames.push(SheetFrame {
                    name: (y * columns + x).to_string(),
                    rect: [at(x, sx)?, at(y, sy)?, fw, fh],
                    pivot,
                });
            }
        }
    }
    for f in input.frames {
        sheet.frames.push(SheetFrame {
            name: f.name,
            rect: f.rect,
            pivot: f.pivot.unwrap_or(pivot),
        });
    }
    if let Some(text) = input.atlas {
        let (frames, tags) = read_atlas(&text)?;
        for f in frames {
            sheet.frames.push(SheetFrame {
                name: f.name,
                rect: f.rect,
                pivot: f.pivot.unwrap_or(pivot),
            });
        }
        let tags: Vec<_> = tags
            .into_iter()
            .filter(|t| !input.clips.iter().any(|c| c.name == t.name))
            .collect();
        clips(&mut sheet, &tags)?;
    }
    clips(&mut sheet, &input.clips)?;
    sheet.validate_native().map_err(|e| e.to_string())?;
    Ok(sheet)
}

/// The concrete texture output shared by animated and procedural sheet producers.
pub(crate) fn texture(s: &Source, png: Vec<u8>, settings: super::imports::Settings) -> Source {
    Source {
        rel: super::animated::sheet_texture_id(&s.rel),
        path: s.path.clone(),
        kind: &super::texture::TEXTURE,
        bytes: png,
        settings,
        hash: 0,
        gltf: None,
        error: None,
        encoded: Default::default(),
        whole: Default::default(),
        model: Default::default(),
        video: None,
        timeline: None,
        motion_action: None,
        strings: None,
        table: None,
        story: None,
        font_pack: None,
        fields: Default::default(),
        fx: None,
        terrains: Default::default(),
        navmesh: None,
        lightmap: None,
        tileset: None,
        tilemap: None,
        animgraph: None,
        flipbook: None,
        rig2d: None,
        scene: None,
        authored_model: None,
        conformed: None,
        fracture: None,
    }
}

/// Cooks a flipbook sheet blob.
pub(crate) fn cook(
    s: &Source,
    motion: Option<&super::conform::mesh::Motion>,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let mut sheet = sheet(s).map_err(CookError)?;
    // The art contract's motion (docs/art.md §2.3, `conform::mesh`).
    let conformed = motion.map(|m| m.flipbook(&mut sheet)).unwrap_or_default();
    sheet
        .validate_native()
        .map_err(|e| CookError(e.to_string()))?;
    let mut data = Vec::new();
    sheet.encode(&mut data);
    let note = format!(
        "{} frames, {} clips ({}) over {} ({}x{})",
        sheet.frames.len(),
        sheet.clips.len(),
        sheet
            .clips
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        sheet.texture,
        sheet.size[0],
        sheet.size[1]
    );
    let mut w = BlobWriter::new(
        format::Flipbook::type_hash(),
        format::Flipbook::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[FlipbookMeta {
            data_bytes: data.len() as u32,
        }],
        row_hash::<FlipbookMeta>(),
    )
    .raw("sheet", data, 4);
    let mut notes = vec![note];
    notes.extend(conformed);
    Ok((w.finish(), notes))
}

/// The texture array output of an `array true` sheet.
pub(crate) fn array_id(rel: &str) -> String {
    format!("{rel}#array")
}

/// Actual subresources produced by a prepared sheet.
pub(crate) fn outputs(sheet: &FlipbookSheet) -> std::collections::BTreeMap<String, String> {
    let mut outputs = std::collections::BTreeMap::new();
    if sheet.array {
        outputs.insert("array".into(), "texture-array".into());
        for n in 0..sheet.frames.len() {
            outputs.insert(format!("array#layer{n}"), "texture".into());
        }
    }
    outputs
}

/// The texture kind an `array true` sheet's layers cook as.
fn array_kind(sheet: &og_anim::FlipbookSheet) -> u32 {
    match sheet.filter {
        og_anim::SheetFilter::Nearest => format::kind::PIXEL,
        og_anim::SheetFilter::Linear => format::kind::COLOR,
    }
}

/// Checks that an `array true` sheet's frames can be an array's layers:
/// at least one, at most the array limit, all of one size, inside the
/// texture.
pub(crate) fn check_array(sheet: &og_anim::FlipbookSheet) -> Result<(), String> {
    let max = super::texture_array::MAX_LAYERS;
    let Some(first) = sheet.frames.first() else {
        return Err("an `array true` sheet needs frames".into());
    };
    if sheet.frames.len() > max {
        return Err(format!(
            "an `array true` sheet has at most {max} frames (the array's layers), not {}",
            sheet.frames.len()
        ));
    }
    for f in &sheet.frames {
        let [x, y, w, h] = f.rect;
        if [w, h] != [first.rect[2], first.rect[3]] {
            return Err(format!(
                "frame `{}` is {w}x{h} and frame `{}` {}x{}: an `array true` sheet's frames are one size",
                f.name, first.name, first.rect[2], first.rect[3]
            ));
        }
        if w == 0 || h == 0 || x + w > sheet.size[0] || y + h > sheet.size[1] {
            return Err(format!(
                "frame `{}` ({x} {y} {w} {h}) is not inside the {}x{} texture",
                f.name, sheet.size[0], sheet.size[1]
            ));
        }
    }
    Ok(())
}

/// The texture array blob of an `array true` sheet.
pub(crate) fn cook_array(
    s: &Source,
    sheet: &og_anim::FlipbookSheet,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    check_array(sheet).map_err(CookError)?;
    let [_, _, w, h] = sheet.frames[0].rect;
    let blob = super::texture_array::array_blob(
        &array_id(&s.rel),
        (w, h),
        sheet.frames.len(),
        array_kind(sheet),
        opts,
        stamp,
    );
    Ok((
        blob,
        vec![format!(
            "texture array: {} frames of {w}x{h} cut from {}",
            sheet.frames.len(),
            sheet.texture
        )],
    ))
}

/// Layer `n` of an `array true` sheet: frame `n` cut out of the texture.
pub(crate) fn cook_layer(
    s: &Source,
    sheet: &og_anim::FlipbookSheet,
    n: usize,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    check_array(sheet).map_err(CookError)?;
    let bytes = texture_bytes(s, &sheet.texture).map_err(CookError)?;
    let img = super::texture::decode(&bytes)
        .map_err(|e| CookError(format!("texture `{}`: {e}", sheet.texture)))?;
    let [x, y, w, h] = sheet.frames[n].rect;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for row in y..y + h {
        let start = ((row * img.width + x) * 4) as usize;
        rgba.extend_from_slice(&img.rgba[start..start + (w * 4) as usize]);
    }
    let frame = super::texture::Image {
        width: w,
        height: h,
        rgba,
    };
    super::texture::cook(&frame, array_kind(sheet), opts, stamp)
}

pub(crate) static FLIPBOOK: KindDef = KindDef {
    name: "flipbook",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: true,
    plan: plan_flipbook,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&FLIPBOOK];

fn plan_flipbook<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options, tex_desc, ..
    } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let deps = dependency_hashes(s);
    assets::record_dependencies(parsed_refs, &s.rel, &deps);
    let motion = options
        .mesh
        .conform
        .as_ref()
        .and_then(|c| c.motion.as_ref())
        .filter(|_| !conform_off(s, options))
        .map(|m| m.describe());
    let st = match &motion {
        Some(m) => stamp(
            &[Area::Effect],
            &[b"flipbook", &hash, deps.as_bytes(), m.as_bytes()],
        ),
        None => stamp(&[Area::Effect], &[b"flipbook", &hash, deps.as_bytes()]),
    };
    outs.push((
        s.rel.clone(),
        format::Flipbook::NAME,
        st,
        Box::new(FlipbookJob(s)),
    ));
    // `array true`: the frames as the layers of a texture array.
    if let Ok(sheet) = sheet(s)
        && sheet.array
    {
        let base = [
            b"flipbook array 1".as_slice(),
            tex_desc.as_bytes(),
            &hash,
            deps.as_bytes(),
        ];
        let sheet = Box::new(sheet);
        for n in 0..sheet.frames.len() {
            let st = stamp(
                &[Area::Effect, Area::Texture],
                &[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat(),
            );
            outs.push((
                texture_array::layer_id(&array_id(&s.rel), n),
                format::Texture::NAME,
                st,
                Box::new(FlipbookLayerJob(s, sheet.clone(), n)),
            ));
        }
        outs.push((
            array_id(&s.rel),
            format::TextureArray::NAME,
            stamp(&[Area::Effect, Area::Texture], &base),
            Box::new(FlipbookArrayJob(s, sheet)),
        ));
    }
}

/// A flipbook's sheet.
struct FlipbookJob<'a>(&'a Source);

impl Job for FlipbookJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let FlipbookJob(s) = self;
        let Exec { options, .. } = ex;
        cook(
            s,
            options
                .mesh
                .conform
                .as_ref()
                .and_then(|c| c.motion.as_ref())
                .filter(|_| !conform_off(s, options)),
            p.stamp,
        )
    }
}

/// A flipbook's frames as a texture array.
struct FlipbookArrayJob<'a>(&'a Source, Box<og_anim::FlipbookSheet>);

impl Job for FlipbookArrayJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let FlipbookArrayJob(s, sheet) = self;
        let Exec { options, .. } = ex;
        cook_array(s, sheet, &options.texture, p.stamp)
    }
}

/// One layer of a flipbook's texture array.
struct FlipbookLayerJob<'a>(&'a Source, Box<og_anim::FlipbookSheet>, usize);

impl Job for FlipbookLayerJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let FlipbookLayerJob(s, sheet, n) = self;
        let Exec { options, .. } = ex;
        cook_layer(s, sheet, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
    }
}
