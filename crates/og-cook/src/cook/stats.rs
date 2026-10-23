//! Diagnostics as data (`og cook --report`, `og doctor --project`,
//! `docs/design.md` §3.4): what each cooked asset holds — triangles and
//! vertices per LOD, textures with their sizes and encodings, bones and
//! clips, bytes cooked — read back from `cooked/` through the runtime's own
//! loaders, and the project's budgets (`budgets { }` in `project.ogt`)
//! checked against it. The report is JSON, deterministic for the same
//! `cooked/` (no times but the cook's own, kept apart in `cookMs`).
//!
//! ```text
//! budgets {
//!   triangles 100000     # per model, LOD 0
//!   vertices 65536       # per model
//!   textureSize 2048     # the longer side of any texture
//!   bones 128            # joints of a skin
//!   clips 64             # animation clips of a model
//!   assetBytes 8388608   # cooked bytes of one source asset
//!   totalBytes 67108864  # cooked bytes of the project
//!   soundSeconds 600     # the length of one sound
//!   videoSeconds 300     # the length of one video
//!   fontGlyphs 8192      # glyphs cooked into one font's atlas
//!   navPolygons 65536    # polygons of one navmesh
//!   sceneEntities 10000  # entities one scene spawns
//!   emitterParticles 16384 # max particles of one emitter
//! }
//! ```
//!
//! Every output type has its own figures in the report: a sound its
//! seconds, channels, rate and chunks; a font its glyphs, kerning pairs and
//! atlas pages; a scene its entities and components; a navmesh its tiles,
//! polygons and vertices; a timeline its tracks and seconds; a style, a
//! string table, a behaviour, a tileset, a tilemap, a rig, a flipbook, a
//! shader, a post pass, an emitter, a look, a palette, a LUT, an
//! environment map, a video, a terrain and a stream pack theirs.

use super::Report;
use crate::format::{self, codec, kind};
use crate::runtime::Bundle;
use og_core::ogt::Block;
use serde_json::{Value, json};
use std::path::Path;

/// The limits a project's assets are held to. Every field has a default
/// (`Budgets::default`); `budgets { }` in project.ogt moves any of them.
#[derive(Clone, Debug, PartialEq)]
pub struct Budgets {
    /// LOD 0 triangles of one model.
    pub triangles: u64,
    /// Vertices of one model.
    pub vertices: u64,
    /// The longer side of a texture: 2048 is WebGL2's guaranteed
    /// `MAX_TEXTURE_SIZE`, the smallest of the tiers.
    pub texture_size: u64,
    /// Joints of one skin.
    pub bones: u64,
    /// Animation clips of one model.
    pub clips: u64,
    /// Cooked bytes of one source asset.
    pub asset_bytes: u64,
    /// Cooked bytes of the whole project.
    pub total_bytes: u64,
    /// Seconds of one sound (a long track belongs streamed, in chunks).
    pub sound_seconds: u64,
    /// Seconds of one video.
    pub video_seconds: u64,
    /// Glyphs cooked into one font (its charset).
    pub font_glyphs: u64,
    /// Polygons of one navmesh.
    pub nav_polygons: u64,
    /// Entities of one scene.
    pub scene_entities: u64,
    /// `max_particles` of one emitter.
    pub emitter_particles: u64,
}

impl Default for Budgets {
    fn default() -> Budgets {
        Budgets {
            triangles: 100_000,
            vertices: 65_536,
            texture_size: 2048,
            bones: 128,
            clips: 64,
            asset_bytes: 8 << 20,
            total_bytes: 64 << 20,
            sound_seconds: 600,
            video_seconds: 300,
            font_glyphs: 8192,
            nav_polygons: 65_536,
            scene_entities: 10_000,
            emitter_particles: 16_384,
        }
    }
}

const KEYS: [&str; 13] = [
    "triangles",
    "vertices",
    "textureSize",
    "bones",
    "clips",
    "assetBytes",
    "totalBytes",
    "soundSeconds",
    "videoSeconds",
    "fontGlyphs",
    "navPolygons",
    "sceneEntities",
    "emitterParticles",
];

impl Budgets {
    /// The budgets of a project manifest's body: its `budgets { }` block
    /// over the defaults. `Err` names an unknown key or a value that is not
    /// a whole number.
    pub fn from_project(body: &Block) -> Result<Budgets, String> {
        let mut b = Budgets::default();
        let Some(block) = body.get("budgets").and_then(|e| e.block.as_ref()) else {
            return Ok(b);
        };
        for e in &block.entries {
            let v = e
                .number(0)
                .ok()
                .and_then(|n| n.as_u64().ok())
                .ok_or_else(|| {
                    format!(
                        "project.ogt line {}: budgets.{} takes a whole number",
                        e.line, e.key
                    )
                })?;
            match e.key.as_str() {
                "triangles" => b.triangles = v,
                "vertices" => b.vertices = v,
                "textureSize" => b.texture_size = v,
                "bones" => b.bones = v,
                "clips" => b.clips = v,
                "assetBytes" => b.asset_bytes = v,
                "totalBytes" => b.total_bytes = v,
                "soundSeconds" => b.sound_seconds = v,
                "videoSeconds" => b.video_seconds = v,
                "fontGlyphs" => b.font_glyphs = v,
                "navPolygons" => b.nav_polygons = v,
                "sceneEntities" => b.scene_entities = v,
                "emitterParticles" => b.emitter_particles = v,
                k => {
                    return Err(format!(
                        "project.ogt line {}: unknown budget '{k}' (expected {})",
                        e.line,
                        KEYS.join(", ")
                    ));
                }
            }
        }
        Ok(b)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "triangles": self.triangles,
            "vertices": self.vertices,
            "textureSize": self.texture_size,
            "bones": self.bones,
            "clips": self.clips,
            "assetBytes": self.asset_bytes,
            "totalBytes": self.total_bytes,
            "soundSeconds": self.sound_seconds,
            "videoSeconds": self.video_seconds,
            "fontGlyphs": self.font_glyphs,
            "navPolygons": self.nav_polygons,
            "sceneEntities": self.scene_entities,
            "emitterParticles": self.emitter_particles,
        })
    }

    /// The budgets a report recorded (its `budgets`); defaults for a
    /// missing key.
    pub fn from_json(v: &Value) -> Budgets {
        let d = Budgets::default();
        let n = |k: &str, d: u64| v[k].as_u64().unwrap_or(d);
        Budgets {
            triangles: n("triangles", d.triangles),
            vertices: n("vertices", d.vertices),
            texture_size: n("textureSize", d.texture_size),
            bones: n("bones", d.bones),
            clips: n("clips", d.clips),
            asset_bytes: n("assetBytes", d.asset_bytes),
            total_bytes: n("totalBytes", d.total_bytes),
            sound_seconds: n("soundSeconds", d.sound_seconds),
            video_seconds: n("videoSeconds", d.video_seconds),
            font_glyphs: n("fontGlyphs", d.font_glyphs),
            nav_polygons: n("navPolygons", d.nav_polygons),
            scene_entities: n("sceneEntities", d.scene_entities),
            emitter_particles: n("emitterParticles", d.emitter_particles),
        }
    }
}

fn codec_name(c: u32) -> &'static str {
    match c {
        codec::ETC1S => "etc1s",
        codec::UASTC => "uastc",
        codec::RGBA8 => "rgba8",
        _ => "unknown",
    }
}

fn kind_name(k: u32) -> &'static str {
    match k {
        kind::COLOR => "color",
        kind::NORMAL => "normal",
        kind::LINEAR => "linear",
        kind::MSDF => "msdf",
        kind::PIXEL => "pixel",
        _ => "unknown",
    }
}

/// What one source asset's outputs hold, read from `cooked/`.
fn asset_stats(bundle: &Bundle, outputs: &[(String, String, u64)]) -> Value {
    let mut model = Value::Null;
    let mut textures = Vec::new();
    let mut animation = Value::Null;
    let mut lightmap = Value::Null;
    let mut others = serde_json::Map::new();
    let mut unread = Vec::new();
    for (id, ty, _) in outputs {
        let read = match ty.as_str() {
            "og.Model" => bundle.load_model(id).map(|m| {
                let lods = m.primitives.iter().map(|p| p.lods.len()).max().unwrap_or(0);
                let per_lod: Vec<u64> = (0..lods)
                    .map(|l| {
                        m.primitives
                            .iter()
                            .map(|p| p.triangle_count(l) as u64)
                            .sum()
                    })
                    .collect();
                // Each level's error: the largest of its primitives', in the
                // model's units (og-render's `lod` draws a level while it
                // projects to less than a pixel).
                let lod_errors: Vec<f32> = (0..lods)
                    .map(|l| {
                        m.primitives
                            .iter()
                            .map(|p| p.lods[l.min(p.lods.len() - 1)].error)
                            .fold(0.0, f32::max)
                    })
                    .collect();
                model = json!({
                    "triangles": per_lod.first().copied().unwrap_or(0),
                    "vertices": m.primitives.iter().map(|p| p.vertices.len() as u64).sum::<u64>(),
                    "lods": per_lod,
                    "lodErrors": lod_errors,
                    "meshes": m.meshes.len(),
                    "primitives": m.primitives.len(),
                    "materials": m.materials.len(),
                    "nodes": m.nodes.len(),
                    // The raycast BVHs' sections in the blob, compressed
                    // (`model { raycast false }`: a few bytes).
                    "bvhBytes": bundle.read(id).ok().and_then(|b| {
                        let blob = og_blob::Blob::parse(b.as_slice(), id).ok()?;
                        let n = blob.raw_opt("bvh_nodes")?.len() + blob.raw_opt("bvh_tris")?.len();
                        Some(n as u64)
                    }).unwrap_or(0),
                });
            }),
            "og.Texture" => bundle.load_texture(id).map(|t| {
                textures.push(json!({
                    "id": id,
                    "width": t.meta.width,
                    "height": t.meta.height,
                    "levels": t.meta.levels,
                    "format": codec_name(t.meta.codec),
                    "kind": kind_name(t.meta.kind),
                    "alpha": t.meta.has_alpha != 0,
                }));
            }),
            "og.Animation" => bundle.load_animation(id).map(|a| {
                let bones = a
                    .skeleton
                    .skins
                    .iter()
                    .map(|s| s.joints.len())
                    .max()
                    .unwrap_or(0);
                animation = json!({
                    "bones": bones,
                    "nodes": a.skeleton.nodes.len(),
                    "skins": a.skeleton.skins.len(),
                    "clips": a.clips.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
                    "morphTargets": a.morphs.len(),
                });
            }),
            "og.Lightmap" => bundle.load_lightmap(id).map(|l| {
                lightmap = json!({
                    "scene": l.file.scene,
                    "models": l.file.models.len(),
                    "pages": l.pages.len(),
                    "width": l.width,
                    "height": l.height,
                    "format": "rgba16f",
                    "volumes": l.file.volumes.len(),
                    "probes": l.file.volumes.iter().map(|v| v.probes.len()).sum::<usize>(),
                    "samples": l.file.samples,
                    "bakeSeconds": l.file.seconds,
                });
            }),
            _ => {
                match other_stats(bundle, id, ty) {
                    Ok(Some((key, v))) => {
                        others.entry(key).or_insert(v);
                    }
                    Ok(None) => {}
                    Err(e) => unread.push(format!("{id}: {e}")),
                }
                Ok(())
            }
        };
        if let Err(e) = read {
            unread.push(format!("{id}: {e}"));
        }
    }
    let mut out = serde_json::Map::new();
    if !model.is_null() {
        out.insert("model".into(), model);
    }
    if !textures.is_empty() {
        out.insert("textures".into(), Value::Array(textures));
    }
    if !animation.is_null() {
        out.insert("animation".into(), animation);
    }
    if !lightmap.is_null() {
        out.insert("lightmap".into(), lightmap);
    }
    out.extend(others);
    if !unread.is_empty() {
        out.insert("unread".into(), json!(unread));
    }
    Value::Object(out)
}

/// Rounds to a thousandth (seconds, sizes in metres).
fn milli(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The `meta` row of the blob of `id`.
fn meta<K: format::BlobKind, T: og_core::Reflect + bytemuck::Pod>(
    bundle: &Bundle,
    id: &str,
) -> Result<T, String> {
    let bytes = bundle.read(id).map_err(|e| e.to_string())?;
    let blob = bundle.blob::<K>(id, &bytes).map_err(|e| e.to_string())?;
    blob.table::<T>("meta", format::row_hash::<T>())
        .map_err(|e| e.to_string())?
        .first()
        .copied()
        .ok_or_else(|| "the blob has no meta row".to_string())
}

/// What an output of a type other than a model, texture, animation or
/// lightmap holds: the report's key for it and its figures; `None` for an
/// output whose figures its parent reports (a sound's chunks, a texture
/// array's layers are `og.Texture`s).
fn other_stats(bundle: &Bundle, id: &str, ty: &str) -> Result<Option<(String, Value)>, String> {
    let e = |e: crate::runtime::LoadError| e.to_string();
    let bytes = || bundle.read(id).map_err(e);
    let (key, v) = match ty {
        "og.Sound" => {
            let m = bundle.load_sound(id).map_err(e)?.meta;
            let rate = m.sample_rate.max(1) as f64;
            (
                "sound",
                json!({
                    "seconds": milli(m.frames as f64 / rate),
                    "channels": m.channels,
                    "sampleRate": m.sample_rate,
                    "frames": m.frames,
                    "chunks": m.chunk_count,
                }),
            )
        }
        "og.SoundChunk" => return Ok(None),
        "og.Font" => {
            let f = bundle.load_font(id).map_err(e)?;
            (
                "font",
                json!({
                    "glyphs": f.meta.glyph_count,
                    "kerningPairs": f.kerning.len(),
                    "pages": f.meta.page_count,
                    "pageWidth": f.meta.page_width,
                    "pageHeight": f.meta.page_height,
                    "emPx": milli(f.meta.em_px as f64),
                    "uncookedCodepoints": f.meta.uncooked_codepoints,
                }),
            )
        }
        "og.Scene" => {
            let s = bundle.load_scene(id).map_err(e)?;
            (
                "scene",
                json!({
                    "entities": s.entities.len(),
                    "instances": s.entities.iter().filter(|x| x.instance.is_some()).count(),
                    "componentTypes": s.components.len(),
                    "components": s.components.iter().map(|c| c.rows.len()).sum::<usize>(),
                    "resources": s.resources.len(),
                    "paths": s.paths.len(),
                }),
            )
        }
        "og.NavMesh" => {
            let (field, tiles) = bundle.navmesh(id, &bytes()?).map_err(e)?;
            (
                "navmesh",
                json!({
                    "tiles": tiles.len(),
                    "polygons": tiles.iter().map(|t| t.polys.len()).sum::<usize>(),
                    "vertices": tiles.iter().map(|t| t.verts.len()).sum::<usize>(),
                    "cells": [field.width, field.depth],
                    "spans": field.spans.len(),
                    "cellSize": milli(field.params.cell_size as f64),
                    "agentRadius": milli(field.params.agent_radius as f64),
                    "agentHeight": milli(field.params.agent_height as f64),
                }),
            )
        }
        "og.TimelineAsset" => {
            let t = bundle.timeline(id, &bytes()?).map_err(e)?;
            (
                "timeline",
                json!({
                    "tracks": t.tracks.len(),
                    "seconds": milli(t.duration as f64),
                    "paths": t.paths.len(),
                }),
            )
        }
        "og.UiStyle" => {
            let s = og_ui::parse_style(&bundle.style_text(id, &bytes()?).map_err(e)?)?;
            (
                "style",
                json!({
                    "name": s.name,
                    "fonts": s.fonts.iter().flatten().count(),
                    "sounds": s.sounds.iter().flatten().count(),
                    "icons": s.icons.as_ref().map_or(0, |i| i.3.len()),
                    "atlas": s.atlas.is_some(),
                    "references": s.references().len(),
                }),
            )
        }
        "og.Strings" => {
            let t = og_i18n::StringTable::parse(&bundle.strings_text(id, &bytes()?).map_err(e)?)?;
            (
                "strings",
                json!({
                    "locale": t.locale,
                    "keys": t.entries.len(),
                    "base": t.base,
                    "fonts": t.fonts.len(),
                }),
            )
        }
        "og.Behaviour" => {
            let b = og_ai::Behaviour::parse(&bundle.behaviour_text(id, &bytes()?).map_err(e)?)?;
            (
                "behaviour",
                json!({
                    "nodes": b.nodes.len(),
                    "keys": b.keys.len(),
                    "routes": b.routes.len(),
                }),
            )
        }
        "og.Tileset" => {
            let t = bundle.load_tileset(id).map_err(e)?;
            (
                "tileset",
                json!({
                    "tiles": t.columns * t.rows,
                    "tileWidth": t.tile_width,
                    "tileHeight": t.tile_height,
                    "definedTiles": t.tiles.len(),
                    "terrains": t.terrains.len(),
                }),
            )
        }
        "og.Tilemap" => {
            let m = bundle.load_tilemap(id).map_err(e)?;
            (
                "tilemap",
                json!({
                    "width": m.width,
                    "height": m.height,
                    "cells": m.width as u64 * m.height as u64,
                    "layers": m.layers.len(),
                    "objects": m.objects.len(),
                    "tilesets": m.tilesets.len(),
                }),
            )
        }
        "og.TextureArray" => {
            let a = bundle.load_texture_array(id).map_err(e)?.meta;
            (
                "textureArray",
                json!({
                    "width": a.width,
                    "height": a.height,
                    "layers": a.layers,
                    "levels": a.levels,
                    "kind": kind_name(a.kind),
                }),
            )
        }
        "og.Video" => {
            let v = meta::<format::Video, format::VideoMeta>(bundle, id)?;
            (
                "video",
                json!({
                    "seconds": milli(v.seconds as f64),
                    "frames": v.frames,
                    "fps": v.fps,
                    "width": v.width,
                    "height": v.height,
                    "audio": v.audio != 0,
                }),
            )
        }
        "og.Terrain" => {
            let t = meta::<format::Terrain, format::TerrainMeta>(bundle, id)?;
            (
                "terrain",
                json!({
                    "size": [milli(t.size_x as f64), milli(t.size_z as f64)],
                    "columns": t.columns,
                    "rows": t.rows,
                    "tile": t.tile,
                    "levels": t.levels,
                    "layers": t.layers,
                    "heightRange": [milli(t.height_min as f64), milli(t.height_max as f64)],
                }),
            )
        }
        "og.EnvironmentMap" => {
            let m = meta::<format::EnvironmentMap, format::EnvironmentMeta>(bundle, id)?;
            (
                "environment",
                json!({
                    "size": m.size,
                    "levels": m.levels,
                    "sourceWidth": m.source_width,
                    "sourceHeight": m.source_height,
                    "samples": m.samples,
                }),
            )
        }
        "og.Lut" => {
            let m = meta::<format::Lut, format::LutMeta>(bundle, id)?;
            ("lut", json!({ "size": m.size }))
        }
        "og.Palette" => {
            let p = bundle.palette(id, &bytes()?).map_err(e)?;
            ("palette", json!({ "colours": p.len() }))
        }
        "og.Look" => {
            let l = bundle.look(id, &bytes()?).map_err(e)?;
            (
                "look",
                json!({
                    "blocks": l.blocks.len(),
                    "extends": l.extends,
                    "ui": l.ui,
                }),
            )
        }
        "og.Rig" => {
            let r = bundle.rig(id, &bytes()?).map_err(e)?;
            ("rig", json!({ "bones": r.bones.len(), "scheme": r.scheme }))
        }
        "og.Rig2D" => {
            let r = bundle.load_rig2d(id).map_err(e)?;
            (
                "rig2d",
                json!({
                    "bones": r.bones.len(),
                    "slots": r.slots.len(),
                    "attachments": r.attachments.len(),
                    "iks": r.iks.len(),
                    "clips": r.clips.len(),
                }),
            )
        }
        "og.Flipbook" => {
            let f = bundle.load_flipbook(id).map_err(e)?;
            (
                "flipbook",
                json!({
                    "frames": f.frames.len(),
                    "clips": f.clips.len(),
                    "size": f.size,
                    "array": f.array,
                }),
            )
        }
        "og.MaterialShader" => {
            let s = bundle.shader(id, &bytes()?).map_err(e)?;
            (
                "shader",
                json!({
                    "params": s.params.len(),
                    "mode": format!("{:?}", s.mode).to_lowercase(),
                    "codeBytes": s.code.len(),
                }),
            )
        }
        "og.PostPass" => {
            let s = bundle.post(id, &bytes()?).map_err(e)?;
            (
                "post",
                json!({ "params": s.params.len(), "codeBytes": s.code.len() }),
            )
        }
        "og.ParticleEmitter" => {
            let text = bundle.emitter_text(id, &bytes()?).map_err(e)?;
            let p = crate::fx::parse_emitter(&text, &mut |_| Ok(None))?;
            (
                "emitter",
                json!({
                    "maxParticles": p.max_particles,
                    "rate": milli(p.rate as f64),
                    "bursts": p.bursts.len(),
                    "seconds": milli(p.duration as f64),
                    "looping": p.looping,
                }),
            )
        }
        "og.StreamPack" => {
            let m = meta::<format::StreamPack, format::StreamPackMeta>(bundle, id)?;
            ("streamPack", json!({ "files": m.files }))
        }
        _ => return Ok(None),
    };
    Ok(Some((key.to_string(), v)))
}

fn flag(budget: &str, limit: u64, value: u64, what: String) -> Value {
    json!({
        "budget": budget,
        "limit": limit,
        "value": value,
        "message": format!("{what}: {value} is over the {budget} budget of {limit}"),
    })
}

/// The budget flags of one asset entry of a report (its `cookedBytes`,
/// `model`, `textures`, `animation`, `sound`, `video`, `font`, `navmesh`,
/// `scene`, `emitter`).
pub fn asset_flags(asset: &Value, b: &Budgets) -> Vec<Value> {
    let source = asset["source"].as_str().unwrap_or("");
    let mut out = Vec::new();
    let bytes = asset["cookedBytes"].as_u64().unwrap_or(0);
    if bytes > b.asset_bytes {
        out.push(flag(
            "assetBytes",
            b.asset_bytes,
            bytes,
            format!("{source} cooks to bytes"),
        ));
    }
    let m = &asset["model"];
    if let Some(t) = m["triangles"].as_u64().filter(|t| *t > b.triangles) {
        out.push(flag(
            "triangles",
            b.triangles,
            t,
            format!("{source} LOD 0 triangles"),
        ));
    }
    if let Some(v) = m["vertices"].as_u64().filter(|v| *v > b.vertices) {
        out.push(flag(
            "vertices",
            b.vertices,
            v,
            format!("{source} vertices"),
        ));
    }
    for t in asset["textures"].as_array().into_iter().flatten() {
        let side = t["width"]
            .as_u64()
            .unwrap_or(0)
            .max(t["height"].as_u64().unwrap_or(0));
        if side > b.texture_size {
            out.push(flag(
                "textureSize",
                b.texture_size,
                side,
                format!("{} longer side", t["id"].as_str().unwrap_or(source)),
            ));
        }
    }
    let a = &asset["animation"];
    if let Some(n) = a["bones"].as_u64().filter(|n| *n > b.bones) {
        out.push(flag("bones", b.bones, n, format!("{source} skin joints")));
    }
    if let Some(n) = a["clips"]
        .as_array()
        .map(|c| c.len() as u64)
        .filter(|n| *n > b.clips)
    {
        out.push(flag("clips", b.clips, n, format!("{source} clips")));
    }
    // Lengths in seconds are flagged when over the whole-second budget
    // (the value reported rounded up).
    for (key, budget, limit, what) in [
        ("sound", "soundSeconds", b.sound_seconds, "seconds of sound"),
        ("video", "videoSeconds", b.video_seconds, "seconds of video"),
    ] {
        if let Some(s) = asset[key]["seconds"].as_f64().filter(|s| *s > limit as f64) {
            out.push(flag(
                budget,
                limit,
                s.ceil() as u64,
                format!("{source} {what}"),
            ));
        }
    }
    for (key, field, budget, limit, what) in [
        ("font", "glyphs", "fontGlyphs", b.font_glyphs, "font glyphs"),
        (
            "navmesh",
            "polygons",
            "navPolygons",
            b.nav_polygons,
            "navmesh polygons",
        ),
        (
            "scene",
            "entities",
            "sceneEntities",
            b.scene_entities,
            "scene entities",
        ),
        (
            "emitter",
            "maxParticles",
            "emitterParticles",
            b.emitter_particles,
            "emitter max particles",
        ),
    ] {
        if let Some(n) = asset[key][field].as_u64().filter(|n| *n > limit) {
            out.push(flag(budget, limit, n, format!("{source} {what}")));
        }
    }
    out
}

/// The project-wide flags of a report (its `totals`).
pub fn total_flags(report: &Value, b: &Budgets) -> Vec<Value> {
    let bytes = report["totals"]["cookedBytes"].as_u64().unwrap_or(0);
    if bytes > b.total_bytes {
        vec![flag(
            "totalBytes",
            b.total_bytes,
            bytes,
            "the project cooks to bytes".into(),
        )]
    } else {
        Vec::new()
    }
}

/// `cook-report.json`: every source asset of `report` with what its
/// outputs hold (read from `cooked`) and its budget flags, the totals, the
/// budgets used and the project's flags.
pub fn cook_report_json(report: &Report, cooked: &Path, budgets: &Budgets) -> Value {
    let bundle = Bundle::open(cooked).ok();
    let mut assets = Vec::new();
    let (mut cooked_bytes, mut source_bytes, mut triangles) = (0u64, 0u64, 0u64);
    for a in &report.assets {
        let outputs: Vec<(String, String, u64)> = a
            .outputs
            .iter()
            .map(|o| (o.id.clone(), o.ty.clone(), o.bytes))
            .collect();
        let bytes: u64 = outputs.iter().map(|o| o.2).sum();
        cooked_bytes += bytes;
        source_bytes += a.source_bytes;
        let mut entry = json!({
            "source": a.source,
            "kind": a.kind,
            "sourceBytes": a.source_bytes,
            "cookedBytes": bytes,
            "cookMs": (a.cook_ms * 1000.0).round() / 1000.0,
            "outputs": outputs.iter().map(|(id, ty, b)| json!({ "id": id, "type": ty, "bytes": b })).collect::<Vec<_>>(),
        });
        if let Some(e) = &a.error {
            entry["error"] = json!(e);
        }
        if let (Some(bundle), None) = (&bundle, &a.error)
            && let Value::Object(stats) = asset_stats(bundle, &outputs)
        {
            for (k, v) in stats {
                entry[k] = v;
            }
        }
        triangles += entry["model"]["triangles"].as_u64().unwrap_or(0);
        entry["flags"] = Value::Array(asset_flags(&entry, budgets));
        assets.push(entry);
    }
    let mut out = json!({
        "version": 1,
        "cooker": report.cooker,
        "budgets": budgets.to_json(),
        "totals": {
            "assets": report.assets.len(),
            "sourceBytes": source_bytes,
            "cookedBytes": cooked_bytes,
            "triangles": triangles,
            "flagged": assets.iter().filter(|a| a["flags"].as_array().is_some_and(|f| !f.is_empty())).count(),
        },
        "assets": assets,
    });
    out["flags"] = Value::Array(total_flags(&out, budgets));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budgets_read_over_the_defaults_and_refuse_unknown_keys() {
        let doc = og_core::ogt::Document::parse(
            "type project\nversion 1\n---\nbudgets {\n  triangles 500\n  totalBytes 100000000\n}\n",
        )
        .unwrap();
        let b = Budgets::from_project(&doc.body).unwrap();
        assert_eq!(b.triangles, 500);
        assert_eq!(b.total_bytes, 100_000_000);
        assert_eq!(b.texture_size, Budgets::default().texture_size);
        let bad = og_core::ogt::Document::parse(
            "type project\nversion 1\n---\nbudgets {\n  polygons 5\n}\n",
        )
        .unwrap();
        assert!(
            Budgets::from_project(&bad.body)
                .unwrap_err()
                .contains("unknown budget 'polygons'")
        );
    }

    #[test]
    fn flags_fire_above_the_limit_only() {
        let b = Budgets {
            triangles: 100,
            texture_size: 512,
            bones: 4,
            ..Budgets::default()
        };
        let asset = json!({
            "source": "models/a.glb",
            "cookedBytes": 10,
            "model": { "triangles": 100, "vertices": 10 },
            "textures": [{ "id": "models/a.glb#tex0", "width": 1024, "height": 256 }, { "id": "t1", "width": 512, "height": 512 }],
            "animation": { "bones": 5, "clips": [] },
        });
        let f = asset_flags(&asset, &b);
        let budgets: Vec<&str> = f.iter().map(|x| x["budget"].as_str().unwrap()).collect();
        // 100 triangles is at the limit (not over); the 1024 side and 5 bones are over.
        assert_eq!(budgets, ["textureSize", "bones"]);
        assert_eq!(f[0]["value"], 1024);
        assert_eq!(
            f[0]["message"],
            "models/a.glb#tex0 longer side: 1024 is over the textureSize budget of 512"
        );
    }
}
