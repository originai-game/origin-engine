//! 2D rigs: an `.ogt` of `type rig2d`, written by hand (og-anim's
//! `rig2d.rs` documents every line) or naming a DragonBones export
//! (`dragonbones "hero_ske.json"`, with `atlas "hero_tex.json"`, its texture
//! atlas), into the canonical rig text (`og_anim::Rig2dAsset::to_text`)
//! with the texture's id and size filled in.
//!
//! The DragonBones 5.5 mapping: y runs down there and up here, so every y
//! and every angle changes sign (`skY` is the rotation; a skew, `skX` other
//! than `skY`, is dropped with a note); a bone keeps `length`; a slot's
//! `displayIndex` is its setup attachment and its colour multipliers its
//! colour (offsets are dropped); the first skin's `image` displays become
//! regions (their atlas sub-texture, transform and pivot) and its `mesh`
//! displays meshes (`uvs` within the sub-texture, `triangles`, unweighted
//! `vertices` in the slot's bone, weighted ones through `slotPose` into the
//! setup pose and from there into each bone of `weights`); an `ik`'s `bone`
//! with `chain` 1 is two bones, `bendPositive` (clockwise on screen) is
//! `bend clockwise`; an animation's `duration` and frame `duration`s are
//! frames of `frameRate`, `playTimes` 0 loops, `translateFrame`,
//! `rotateFrame` and `scaleFrame` become bone keys (relative to the setup
//! pose, as there), `tweenEasing: null` a `step` key and a `curve` linear,
//! `displayFrame` and `colorFrame` slot keys, `ffd` (or `deform`) frames
//! deform keys (`offset` and `vertices`, per influence), and frame `events`
//! (or `actions` of type 10 and 11) events. Nested armatures, bounding
//! boxes, z-order and IK timelines have no line here and are noted.

use super::{CookError, Source};
use crate::format::{self, *};
use og_anim::{
    Affine2, Attachment2d, AttachmentKind, Bone2d, BoneTimeline, Clip2d, DeformTimeline, Ik2d,
    Influence, Key, MeshVertex2d, Rig2dAsset, SheetFilter, Slot2d, SlotTimeline,
};
use og_blob::BlobWriter;
use og_core::ogt::{Document, Entry, Value};
use serde_json::Value as Json;

fn string_of(doc: &Document, key: &str) -> Option<String> {
    doc.body
        .get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// The hashes of what the rig depends on (its texture, its DragonBones
/// export and atlas), for the cook stamp.
pub(super) fn dependency_hashes(s: &Source) -> String {
    let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    let root = super::flipbook::assets_root(s);
    ["texture", "dragonbones", "atlas"]
        .iter()
        .filter_map(|k| string_of(&doc, k))
        .map(|p| {
            let id = super::flipbook::resolve(s, &p);
            let h = std::fs::read(root.join(&id))
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{id}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The canonical rig of a source, and notes on what an import dropped.
pub(super) fn rig(s: &Source) -> Result<(Rig2dAsset, Vec<String>), String> {
    let text = std::str::from_utf8(&s.bytes).map_err(|_| "a 2D rig is text (UTF-8)".to_string())?;
    let mut doc = Document::parse(text).map_err(|e| e.reason().to_string())?;
    let root = super::flipbook::assets_root(s);
    let mut notes = Vec::new();
    let read = |p: &str| -> Result<(String, Vec<u8>), String> {
        let id = super::flipbook::resolve(s, p);
        let bytes =
            std::fs::read(root.join(&id)).map_err(|_| format!("`{id}` is not in assets/"))?;
        Ok((id, bytes))
    };
    let mut rig = if let Some(db) = string_of(&doc, "dragonbones") {
        if let Some(e) = doc.body.entries.iter().find(|e| {
            matches!(
                e.key.as_str(),
                "bone" | "slot" | "region" | "mesh" | "ik" | "clip"
            )
        }) {
            return Err(format!(
                "line {}: a rig that imports `dragonbones` takes its {} from the export; keep texture, atlas, filter and unit here",
                e.line, e.key
            ));
        }
        let (db_id, db_bytes) = read(&db)?;
        let atlas_path = string_of(&doc, "atlas")
            .ok_or("a DragonBones rig needs `atlas \"<name>_tex.json\"`")?;
        let (atlas_id, atlas_bytes) = read(&atlas_path)?;
        let ske: Json =
            serde_json::from_slice(&db_bytes).map_err(|e| format!("{db_id}: not JSON: {e}"))?;
        let atlas: Json = serde_json::from_slice(&atlas_bytes)
            .map_err(|e| format!("{atlas_id}: not JSON: {e}"))?;
        let mut rig = dragonbones(&ske, &atlas, &mut notes).map_err(|e| format!("{db_id}: {e}"))?;
        // The texture: the rig's own line, else the atlas's imagePath beside it.
        rig.texture = match string_of(&doc, "texture") {
            Some(t) => t,
            None => {
                let image = atlas["imagePath"]
                    .as_str()
                    .ok_or(format!("{atlas_id}: no imagePath; name the texture"))?;
                let dir = atlas_id
                    .rsplit_once('/')
                    .map_or(String::new(), |(d, _)| format!("{d}/"));
                format!("{dir}{image}")
            }
        };
        if let Some(e) = doc.body.get("unit") {
            rig.unit = e
                .f32(0)
                .map_err(|_| format!("line {}: unit <pixels per world unit>", e.line))?;
        }
        if doc.body.get("filter").and_then(|e| e.word(0).ok()) == Some("nearest") {
            rig.filter = SheetFilter::Nearest;
        }
        rig
    } else {
        // The hand-written text: the size of its texture filled in and checked as og-anim reads it.
        doc.body.entries.retain(|e| e.key != "size");
        let size = Entry::new("size")
            .with(Value::Number(
                og_core::ogt::Number::from_f64(0.0).expect("finite"),
            ))
            .with(Value::Number(
                og_core::ogt::Number::from_f64(0.0).expect("finite"),
            ));
        doc.body.entries.insert(0, size);
        let text = doc.to_text().map_err(|e| e.reason().to_string())?;
        Rig2dAsset::parse(&text).map_err(|e| e.reason().to_string())?
    };
    let (texture_id, image) = read(&rig.texture.clone())?;
    let (w, h) = image::load_from_memory(&image)
        .map(|i| (i.width(), i.height()))
        .map_err(|e| format!("texture `{texture_id}`: {e}"))?;
    rig.texture = texture_id;
    rig.size = [w, h];
    Ok((rig, notes))
}

pub(super) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let (rig, mut notes) = rig(s).map_err(CookError)?;
    let text = rig.to_text();
    notes.insert(
        0,
        format!(
            "{} bones, {} slots, {} attachments, {} IK, {} clips ({}) over {} ({}x{})",
            rig.bones.len(),
            rig.slots.len(),
            rig.attachments.len(),
            rig.iks.len(),
            rig.clips.len(),
            rig.clips
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            rig.texture,
            rig.size[0],
            rig.size[1]
        ),
    );
    let mut w = BlobWriter::new(
        format::Rig2D::type_hash(),
        format::Rig2D::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[Rig2DMeta {
            text_bytes: text.len() as u32,
        }],
        row_hash::<Rig2DMeta>(),
    )
    .raw("text", text.into_bytes(), 4);
    Ok((w.finish(), notes))
}

// ── DragonBones ────────────────────────────────────────────────────────────

fn f(v: &Json, k: &str, default: f64) -> f32 {
    v.get(k).and_then(Json::as_f64).unwrap_or(default) as f32
}

fn name(v: &Json) -> Result<String, String> {
    v["name"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "an entry without a name".into())
}

/// Converts a DragonBones 5.5 skeleton export (its first armature) with
/// its texture atlas; `notes` collects what has no equivalent.
pub fn dragonbones(
    ske: &Json,
    atlas: &Json,
    notes: &mut Vec<String>,
) -> Result<Rig2dAsset, String> {
    let arm = ske["armature"].get(0).ok_or("no armature")?;
    if ske["armature"].as_array().is_some_and(|a| a.len() > 1) {
        notes.push("only the first armature is imported".into());
    }
    let rate = arm["frameRate"]
        .as_f64()
        .or(ske["frameRate"].as_f64())
        .unwrap_or(24.0) as f32;
    let mut rig = Rig2dAsset {
        texture: String::new(),
        size: [0, 0],
        filter: SheetFilter::Linear,
        unit: 1.0,
        bones: Vec::new(),
        slots: Vec::new(),
        attachments: Vec::new(),
        iks: Vec::new(),
        clips: Vec::new(),
    };
    let mut skewed = Vec::new();
    for b in arm["bone"].as_array().into_iter().flatten() {
        let n = name(b)?;
        let parent = match b["parent"].as_str() {
            Some(p) => Some(
                rig.bone(p)
                    .ok_or(format!("bone {n}: parent {p} comes after it"))?,
            ),
            None => None,
        };
        let t = &b["transform"];
        if (f(t, "skX", 0.0) - f(t, "skY", 0.0)).abs() > 1e-3 {
            skewed.push(n.clone());
        }
        rig.bones.push(Bone2d {
            name: n,
            parent,
            x: f(t, "x", 0.0),
            y: -f(t, "y", 0.0),
            rotation: -f(t, "skY", 0.0),
            scale: [f(t, "scX", 1.0), f(t, "scY", 1.0)],
            length: f(b, "length", 0.0),
        });
    }
    if !skewed.is_empty() {
        notes.push(format!("skew dropped on bones {}", skewed.join(", ")));
    }
    if rig.bones.is_empty() {
        return Err("the armature has no bones".into());
    }
    let setup = rig.pose(None, 0.0, &[]).world;
    // Sub-textures by name.
    let subs: Vec<(String, [f32; 4])> = atlas["SubTexture"]
        .as_array()
        .ok_or("the atlas has no SubTexture list")?
        .iter()
        .map(|t| {
            Ok((
                name(t)?,
                [
                    f(t, "x", 0.0),
                    f(t, "y", 0.0),
                    f(t, "width", 0.0),
                    f(t, "height", 0.0),
                ],
            ))
        })
        .collect::<Result<_, String>>()?;
    let sub = |n: &str| -> Result<[f32; 4], String> {
        subs.iter()
            .find(|(s, _)| s == n)
            .map(|(_, r)| *r)
            .ok_or(format!("no sub-texture `{n}` in the atlas"))
    };
    let skin = arm["skin"].get(0);
    // Per slot: its displays' attachment indices (None for a display not imported).
    let mut displays: Vec<Vec<Option<usize>>> = Vec::new();
    let mut skipped = Vec::new();
    for (si, sl) in arm["slot"].as_array().into_iter().flatten().enumerate() {
        let n = name(sl)?;
        let bone_name = sl["parent"]
            .as_str()
            .ok_or(format!("slot {n} has no parent bone"))?;
        let bone = rig
            .bone(bone_name)
            .ok_or(format!("slot {n}: no bone {bone_name}"))?;
        let c = &sl["color"];
        rig.slots.push(Slot2d {
            name: n.clone(),
            bone,
            attachment: None,
            color: [
                f(c, "rM", 100.0) / 100.0,
                f(c, "gM", 100.0) / 100.0,
                f(c, "bM", 100.0) / 100.0,
                f(c, "aM", 100.0) / 100.0,
            ],
        });
        let list = skin
            .and_then(|k| k["slot"].as_array())
            .and_then(|l| l.iter().find(|s| s["name"].as_str() == Some(n.as_str())))
            .and_then(|s| s["display"].as_array());
        let mut ids = Vec::new();
        for d in list.into_iter().flatten() {
            let dn = name(d)?;
            let path = d["path"].as_str().unwrap_or(&dn).to_string();
            let mut an = dn.clone();
            if rig.attachments.iter().any(|a| a.name == an) {
                an = format!("{n}/{dn}");
            }
            let kind = match d["type"].as_str().unwrap_or("image") {
                "image" => {
                    let rect = sub(&path)?;
                    let t = &d["transform"];
                    let (sx, sy) = (f(t, "scX", 1.0), f(t, "scY", 1.0));
                    let rotation = -f(t, "skY", 0.0);
                    let size = [rect[2] * sx, rect[3] * sy];
                    // The pivot (0.5 the centre) moves the centre off the display's origin.
                    let (px, py) = (f(&d["pivot"], "x", 0.5), f(&d["pivot"], "y", 0.5));
                    let shift = Affine2::trs(0.0, 0.0, rotation, 1.0, 1.0)
                        .apply((0.5 - px) * size[0], -(0.5 - py) * size[1]);
                    AttachmentKind::Region {
                        rect,
                        at: [f(t, "x", 0.0) + shift[0], -f(t, "y", 0.0) + shift[1]],
                        rotation,
                        size,
                    }
                }
                "mesh" => mesh(d, &dn, &sub(&path)?, bone, &setup)?,
                other => {
                    skipped.push(format!("{n}/{dn} ({other})"));
                    ids.push(None);
                    continue;
                }
            };
            ids.push(Some(rig.attachments.len()));
            rig.attachments.push(Attachment2d {
                name: an,
                slot: si,
                kind,
            });
        }
        let shown = sl["displayIndex"].as_i64().unwrap_or(0);
        rig.slots[si].attachment = usize::try_from(shown)
            .ok()
            .and_then(|i| ids.get(i).copied().flatten());
        displays.push(ids);
    }
    if !skipped.is_empty() {
        notes.push(format!("displays not imported: {}", skipped.join(", ")));
    }
    for k in arm["ik"].as_array().into_iter().flatten() {
        let n = name(k)?;
        let bone = rig
            .bone(k["bone"].as_str().unwrap_or(""))
            .ok_or(format!("ik {n}: no bone"))?;
        let target = rig
            .bone(k["target"].as_str().unwrap_or(""))
            .ok_or(format!("ik {n}: no target"))?;
        let bones = match k["chain"].as_u64().unwrap_or(0) {
            0 => vec![bone],
            1 => vec![
                rig.bones[bone]
                    .parent
                    .ok_or(format!("ik {n}: chain 1 on a root bone"))?,
                bone,
            ],
            c => return Err(format!("ik {n}: chain {c}; one or two bones are imported")),
        };
        rig.iks.push(Ik2d {
            name: n,
            bones,
            target,
            counterclockwise: !k["bendPositive"].as_bool().unwrap_or(true),
            weight: f(k, "weight", 1.0),
        });
    }
    let mut dropped = Vec::new();
    for a in arm["animation"].as_array().into_iter().flatten() {
        let n = name(a)?;
        let step = |k: &Json| k.get("tweenEasing").is_some_and(Json::is_null);
        let mut clip = Clip2d {
            name: n.clone(),
            duration: f(a, "duration", 1.0).max(1.0) / rate,
            looping: a["playTimes"].as_u64().unwrap_or(1) == 0,
            bones: Vec::new(),
            slots: Vec::new(),
            deforms: Vec::new(),
            events: Vec::new(),
        };
        for t in a["bone"].as_array().into_iter().flatten() {
            let bone = rig
                .bone(t["name"].as_str().unwrap_or(""))
                .ok_or(format!("animation {n}: no bone {}", t["name"]))?;
            clip.bones.push(BoneTimeline {
                bone,
                translate: frames(&t["translateFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| Key {
                        time,
                        value: [f(k, "x", 0.0), -f(k, "y", 0.0)],
                        step: step(k),
                    })
                    .collect(),
                rotate: frames(&t["rotateFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| Key {
                        time,
                        value: -f(k, "rotate", 0.0),
                        step: step(k),
                    })
                    .collect(),
                scale: frames(&t["scaleFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| Key {
                        time,
                        value: [f(k, "x", 1.0), f(k, "y", 1.0)],
                        step: step(k),
                    })
                    .collect(),
            });
        }
        for t in a["slot"].as_array().into_iter().flatten() {
            let slot = rig
                .slots
                .iter()
                .position(|s| Some(s.name.as_str()) == t["name"].as_str());
            let slot = slot.ok_or(format!("animation {n}: no slot {}", t["name"]))?;
            clip.slots.push(SlotTimeline {
                slot,
                attachment: frames(&t["displayFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| {
                        let i = k["value"].as_i64().unwrap_or(0);
                        (
                            time,
                            usize::try_from(i)
                                .ok()
                                .and_then(|i| displays[slot].get(i).copied().flatten()),
                        )
                    })
                    .collect(),
                color: frames(&t["colorFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| {
                        let c = &k["color"];
                        Key {
                            time,
                            value: [
                                f(c, "rM", 100.0) / 100.0,
                                f(c, "gM", 100.0) / 100.0,
                                f(c, "bM", 100.0) / 100.0,
                                f(c, "aM", 100.0) / 100.0,
                            ],
                            step: step(k),
                        }
                    })
                    .collect(),
            });
        }
        for t in a["ffd"]
            .as_array()
            .or(a["deform"].as_array())
            .into_iter()
            .flatten()
        {
            let slot = t["slot"]
                .as_str()
                .and_then(|s| rig.slots.iter().position(|x| x.name == s));
            let attachment = rig
                .attachments
                .iter()
                .position(|x| {
                    Some(x.slot) == slot
                        && (Some(x.name.as_str()) == t["name"].as_str()
                            || x.name
                                .ends_with(&format!("/{}", t["name"].as_str().unwrap_or(""))))
                })
                .ok_or(format!(
                    "animation {n}: no mesh {} in slot {}",
                    t["name"], t["slot"]
                ))?;
            let len = 2 * rig.attachments[attachment].influences();
            let keys = frames(&t["frame"], rate)
                .into_iter()
                .map(|(time, k)| {
                    let mut v = vec![0.0f32; len];
                    let off = k["offset"].as_u64().unwrap_or(0) as usize;
                    for (i, d) in k["vertices"].as_array().into_iter().flatten().enumerate() {
                        if let Some(slot) = v.get_mut(off + i) {
                            // Every odd entry is a y: down there, up here.
                            let s = if (off + i) % 2 == 1 { -1.0 } else { 1.0 };
                            *slot = s * d.as_f64().unwrap_or(0.0) as f32;
                        }
                    }
                    Key {
                        time,
                        value: v,
                        step: step(k),
                    }
                })
                .collect();
            clip.deforms.push(DeformTimeline { attachment, keys });
        }
        for (time, k) in frames(&a["frame"], rate) {
            let mut push = |e: &str| clip.events.push((time, e.chars().take(63).collect()));
            if let Some(e) = k["event"].as_str() {
                push(e);
            }
            for e in k["events"].as_array().into_iter().flatten() {
                if let Some(e) = e["name"].as_str() {
                    push(e);
                }
            }
            for e in k["actions"].as_array().into_iter().flatten() {
                if matches!(e["type"].as_i64(), Some(10 | 11))
                    && let Some(e) = e["name"].as_str()
                {
                    push(e);
                }
            }
        }
        for key in ["zOrder", "ik"] {
            if a.get(key).is_some() {
                dropped.push(format!("{n}.{key}"));
            }
        }
        clip.events.sort_by(|a, b| a.0.total_cmp(&b.0));
        rig.clips.push(clip);
    }
    if !dropped.is_empty() {
        notes.push(format!("timelines not imported: {}", dropped.join(", ")));
    }
    Ok(rig)
}

/// Keyframes with their times: each frame's `duration` is frames of `rate`.
fn frames(list: &Json, rate: f32) -> Vec<(f32, &Json)> {
    let mut at = 0.0f32;
    list.as_array()
        .into_iter()
        .flatten()
        .map(|k| {
            let t = at / rate;
            at += f(k, "duration", 1.0);
            (t, k)
        })
        .collect()
}

/// A DragonBones mesh display: uvs into the sub-texture, its vertices into
/// the slot's bone (unweighted) or each weighted bone's setup space.
fn mesh(
    d: &Json,
    n: &str,
    rect: &[f32; 4],
    slot_bone: usize,
    setup: &[Affine2],
) -> Result<AttachmentKind, String> {
    let nums = |k: &str| -> Vec<f32> {
        d[k].as_array()
            .into_iter()
            .flatten()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect()
    };
    let (vs, uvs) = (nums("vertices"), nums("uvs"));
    let triangles: Vec<u32> = nums("triangles").into_iter().map(|t| t as u32).collect();
    let weights = nums("weights");
    let count = vs.len() / 2;
    if uvs.len() != vs.len() || count == 0 {
        return Err(format!(
            "mesh {n}: {} vertex numbers and {} uv numbers",
            vs.len(),
            uvs.len()
        ));
    }
    // slotPose (a b c d tx ty) takes a weighted vertex into the setup pose (y down).
    let sp = nums("slotPose");
    let slot_pose = if sp.len() == 6 {
        Affine2 {
            a: sp[0],
            b: sp[1],
            c: sp[2],
            d: sp[3],
            tx: sp[4],
            ty: sp[5],
        }
    } else {
        Affine2::IDENTITY
    };
    let mut vertices = Vec::with_capacity(count);
    let mut w = 0usize;
    for i in 0..count {
        let uv = [
            rect[0] + uvs[2 * i] * rect[2],
            rect[1] + uvs[2 * i + 1] * rect[3],
        ];
        let (x, y) = (vs[2 * i], vs[2 * i + 1]);
        let influences = if weights.is_empty() {
            vec![Influence {
                bone: slot_bone,
                x,
                y: -y,
                weight: 1.0,
            }]
        } else {
            let g = slot_pose.apply(x, y);
            // The setup position here (y up), into each bone.
            let here = [g[0], -g[1]];
            let k = *weights
                .get(w)
                .ok_or(format!("mesh {n}: weights end early"))? as usize;
            w += 1;
            let mut out = Vec::with_capacity(k);
            for _ in 0..k {
                let (b, wt) = (weights.get(w).copied(), weights.get(w + 1).copied());
                let (Some(b), Some(wt)) = (b, wt) else {
                    return Err(format!("mesh {n}: weights end early"));
                };
                w += 2;
                let b = b as usize;
                let m = setup
                    .get(b)
                    .ok_or(format!("mesh {n}: weight on bone {b}"))?;
                let l = m.inverse().apply(here[0], here[1]);
                out.push(Influence {
                    bone: b,
                    x: l[0],
                    y: l[1],
                    weight: wt,
                });
            }
            // Normalise rounding in the export so the weights sum to 1.
            let total: f32 = out.iter().map(|i| i.weight).sum();
            if total > 0.0 {
                out.iter_mut().for_each(|i| i.weight /= total);
            }
            out
        };
        vertices.push(MeshVertex2d { uv, influences });
    }
    if triangles.is_empty() || triangles.iter().any(|t| *t as usize >= count) {
        return Err(format!(
            "mesh {n}: triangles index past its {count} vertices"
        ));
    }
    Ok(AttachmentKind::Mesh {
        vertices,
        triangles,
    })
}
