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
//! }
//! ```

use super::Report;
use crate::format::{codec, kind};
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
        }
    }
}

const KEYS: [&str; 7] = [
    "triangles",
    "vertices",
    "textureSize",
    "bones",
    "clips",
    "assetBytes",
    "totalBytes",
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
                model = json!({
                    "triangles": per_lod.first().copied().unwrap_or(0),
                    "vertices": m.primitives.iter().map(|p| p.vertices.len() as u64).sum::<u64>(),
                    "lods": per_lod,
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
            _ => Ok(()),
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
    if !unread.is_empty() {
        out.insert("unread".into(), json!(unread));
    }
    Value::Object(out)
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
/// `model`, `textures`, `animation`).
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
