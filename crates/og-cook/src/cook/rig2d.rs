//! 2D rigs: an `.ogt` of `type rig2d`, written by hand (og-anim's
//! `rig2d.rs` documents every line) or naming a DragonBones export
//! (`dragonbones "hero_ske.json"`, with `atlas "hero_tex.json"`, its texture
//! atlas), into the canonical rig text (`og_anim::Rig2dAsset::to_text`)
//! with the texture's id and size filled in.
//!
//! The DragonBones 5.5 mapping: y runs down there and up here, so every y
//! and every angle changes sign; a transform's `skY` (or `rotate`) is the
//! rotation and `skX` minus `skY` (or `skew`) the skew, negated; a bone
//! keeps `length`; a slot's `displayIndex` is its setup attachment and its
//! colour's multipliers (percent) and offsets (-255..255) its `color` and
//! `offset`; the first skin's `image` displays become regions (their atlas
//! sub-texture, transform and pivot), its `mesh` displays meshes (`uvs`
//! within the sub-texture, `triangles`, unweighted `vertices` in the slot's
//! bone, weighted ones through `slotPose` into the setup pose and from
//! there into each bone of `weights`), its `boundingBox` displays boxes
//! (a rectangle's corners, a polygon's `vertices`, an ellipse as 32 sides,
//! through the display's transform) and its `armature` displays child
//! attachments (the armature of that name in the same export, imported as
//! a child armature, playing the display's or the slot's `actions`); an
//! `ik`'s `bone` with `chain` n is that bone and its n ancestors, root
//! first, and `bendPositive` (clockwise on screen) is `bend clockwise`; an animation's `duration` and frame
//! `duration`s are frames of `frameRate`, `playTimes` 0 loops,
//! `translateFrame`, `rotateFrame` (`rotate` and `skew`, a frame's
//! `clockwise` turns as the key's turn, accumulated as DragonBones' parser
//! accumulates them) and `scaleFrame` become bone keys (relative to the
//! setup pose, as there), `displayFrame`
//! and `colorFrame` slot keys (multipliers and offsets), `ffd` (or
//! `deform`) frames deform keys (`offset` and `vertices`, per influence),
//! `zOrder` frames (slot and offset pairs) draw orders, `ik` frames
//! (`weight`, `bendPositive`) IK keys, and frame `events` (or `actions` of
//! type 10 and 11) events. A frame's `curve` is a Bézier key; a
//! `tweenEasing` of 0 is linear, up to 1 DragonBones' quadratic eases as
//! Béziers (exact: a quadratic is a cubic), above 1 its sine in-out as a
//! Bézier of Hermite segments within 1e-7 of it; a frame with neither
//! holds its value (DragonBones' no-tween). A `path` display is a path
//! attachment (its `vertices` three per knot, weighted as a mesh's,
//! `closed`, `constantSpeed` and `lengths`), and the armature's `path`
//! constraints (`target` slot and `targetDisplay`, `bones`,
//! `positionMode`, `spacingMode`, `rotateMode`, `position`, `spacing`,
//! `rotateOffset`, `rotateMix`, `translateMix`) `along` constraints;
//! DragonBones has no transform constraint.

use super::{CookError, Source};
use crate::format::{self, *};
use og_anim::{
    Affine2, Armature2d, Attachment2d, AttachmentKind, Bone2d, BoneTimeline, Clip2d,
    DeformTimeline, Ease, Ik2d, IkKey, IkTimeline, Influence, Key, MeshVertex2d, PathConstraint2d,
    PathRotate, PathSpacing, PathUnit, Rig2dAsset, Rotation, SheetFilter, Skin2d, Slot2d,
    SlotTimeline, Turn, turn_delta,
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
                "bone" | "slot" | "region" | "mesh" | "box" | "child" | "ik" | "clip" | "armature"
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
            "{} bones, {} slots, {} attachments, {} IK, {} child armatures, {} clips ({}) over {} ({}x{})",
            rig.bones.len(),
            rig.slots.len(),
            rig.attachments.len(),
            rig.iks.len(),
            rig.armatures.len(),
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

/// A DragonBones transform here: x, y (up), rotation and skew (degrees,
/// counter-clockwise), scale. DragonBones' x axis turns by `skY` (or
/// `rotate`) and its y axis by `skX` (or `rotate + skew`), both clockwise
/// with y down; mirroring y negates every angle.
struct Place {
    x: f32,
    y: f32,
    rotation: f32,
    skew: f32,
    scale: [f32; 2],
}

fn place(t: &Json) -> Place {
    let (rotation, skew) = if t.get("rotate").is_some() || t.get("skew").is_some() {
        (-f(t, "rotate", 0.0), -f(t, "skew", 0.0))
    } else {
        let (skx, sky) = (f(t, "skX", 0.0), f(t, "skY", 0.0));
        (-sky, sky - skx)
    };
    Place {
        x: f(t, "x", 0.0),
        y: -f(t, "y", 0.0),
        rotation,
        skew,
        scale: [f(t, "scX", 1.0), f(t, "scY", 1.0)],
    }
}

impl Place {
    fn affine(&self) -> Affine2 {
        Affine2::trsk(
            self.x,
            self.y,
            self.rotation,
            self.skew,
            self.scale[0],
            self.scale[1],
        )
    }
}

/// A DragonBones colour transform: multipliers (percent) and offsets
/// (-255..255) as linear 0..1 multipliers and offsets.
fn color(c: &Json) -> ([f32; 4], [f32; 4]) {
    (
        ["rM", "gM", "bM", "aM"].map(|k| f(c, k, 100.0) / 100.0),
        ["rO", "gO", "bO", "aO"].map(|k| f(c, k, 0.0) / 255.0),
    )
}

/// What a frame's tween is: a `curve` (Bézier control points), a
/// `tweenEasing` number (0 linear; below 0 quadratic in, to 1 quadratic
/// out, above 1 sine in-out, blended with the line by its amount), or
/// neither (or null): no tween, the value holds until the next frame.
fn ease(k: &Json, notes: &mut Vec<String>) -> Ease {
    if let Some(curve) = k["curve"].as_array() {
        let mut c: Vec<f32> = curve
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        // The long form includes the end points (0, 0) and (1, 1).
        if c.len() % 3 != 1 && c.len() >= 8 {
            c = c[2..c.len() - 2].to_vec();
        }
        return match og_anim::check_curve(&c) {
            Ok(()) => Ease::Bezier(c),
            Err(e) => {
                notes.push(format!("a curve played linearly: {e}"));
                Ease::Linear
            }
        };
    }
    let Some(e) = k.get("tweenEasing").and_then(Json::as_f64) else {
        return Ease::Step;
    };
    let e = e as f32;
    let third = 1.0 / 3.0;
    if e == 0.0 {
        Ease::Linear
    } else if e < 0.0 {
        // s² is the cubic (0, 0) (1/3, 0) (2/3, 1/3) (1, 1); blended with the line by -e.
        let a = (-e).min(1.0);
        Ease::Bezier(vec![
            third,
            third * (1.0 - a),
            2.0 * third,
            third * (2.0 - a),
        ])
    } else if e <= 1.0 {
        // 2s - s²: (1/3, 2/3) (2/3, 1).
        Ease::Bezier(vec![
            third,
            third * (1.0 + e),
            2.0 * third,
            third * (2.0 + e),
        ])
    } else {
        // DragonBones keeps (e - 1) in hundredths, unclamped (_parseTweenFrame).
        Ease::Bezier(sine_curve((f64::from(e) * 100.0 - 100.0).round() * 0.01))
    }
}

/// DragonBones' sine in-out blended with the line by `a` (its
/// `_getEasingValue`: `s + a (0.5 (1 - cos πs) - s)`), which has no cubic
/// form, as a Bézier of equal segments: each the cubic Hermite through
/// the curve's values and slopes at its ends (control points a third of
/// the segment along the tangents). Its error is at most `max |y''''|
/// h⁴ / 384` with `y'''' = a π⁴ / 2`; the segments are enough to keep it
/// under 1e-7 of the key's change.
fn sine_curve(a: f64) -> Vec<f32> {
    const PI: f64 = std::f64::consts::PI;
    const ERROR: f64 = 1e-7;
    let y = |s: f64| s + a * (0.5 * (1.0 - og_anim::sincos(PI * s).1) - s);
    let slope = |s: f64| 1.0 + a * (0.5 * PI * og_anim::sincos(PI * s).0 - 1.0);
    let bound = a.abs() * PI.powi(4) / 2.0 / 384.0;
    let mut n = 1usize;
    while n < 256 && bound / (n as f64).powi(4) > ERROR {
        n += 1;
    }
    let h = 1.0 / n as f64;
    let mut c = Vec::with_capacity(6 * n - 2);
    for k in 0..n {
        let (x0, x1) = (k as f64 * h, (k + 1) as f64 * h);
        if k > 0 {
            c.extend([x0, y(x0)]);
        }
        c.extend([
            x0 + h / 3.0,
            y(x0) + h / 3.0 * slope(x0),
            x1 - h / 3.0,
            y(x1) - h / 3.0 * slope(x1),
        ]);
    }
    c.into_iter().map(|v| v as f32).collect()
}

/// The animation a DragonBones `actions` list (a name, or `gotoAndPlay`
/// or a `play` action) starts on a child armature.
fn play_action(actions: &Json) -> Option<String> {
    if let Some(s) = actions.as_str() {
        return Some(s.to_string());
    }
    actions.as_array()?.iter().find_map(|a| {
        a["gotoAndPlay"].as_str().map(str::to_string).or_else(|| {
            let play = matches!(a["type"].as_str(), Some("play")) || a["type"].as_i64() == Some(0);
            play.then(|| a["name"].as_str().map(str::to_string))
                .flatten()
        })
    })
}

/// Converts a DragonBones 5.5 skeleton export with its texture atlas: the
/// first armature is the rig, and every armature an armature display shows
/// (directly or through another) becomes one of its child armatures;
/// `notes` collects what has no equivalent.
pub fn dragonbones(
    ske: &Json,
    atlas: &Json,
    notes: &mut Vec<String>,
) -> Result<Rig2dAsset, String> {
    let list = ske["armature"].as_array().ok_or("no armature")?;
    let first = list.first().ok_or("no armature")?;
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
    // Child armatures in the order displays first name them.
    let mut children: Vec<String> = Vec::new();
    let (top, top_skins) = armature(first, ske, &subs, &mut children, notes)?;
    let mut armatures = Vec::new();
    let mut i = 0;
    while i < children.len() {
        let n = children[i].clone();
        let raw = list
            .iter()
            .find(|a| a["name"].as_str() == Some(n.as_str()))
            .ok_or(format!(
                "an armature display shows `{n}`, not an armature of the export"
            ))?;
        // Its other skins are its own: it wears the one named as the skin
        // the rig wears (og-anim's rig2d module docs).
        let (mut a, skins) = armature(raw, ske, &subs, &mut children, notes)
            .map_err(|e| format!("armature {n}: {e}"))?;
        a.skins = skins;
        a.name = n;
        armatures.push(a);
        i += 1;
    }
    let unused: Vec<String> = list
        .iter()
        .skip(1)
        .filter_map(|a| a["name"].as_str())
        .filter(|n| !children.iter().any(|c| c == n))
        .map(str::to_string)
        .collect();
    if !unused.is_empty() {
        notes.push(format!(
            "armatures no display of the first shows are not imported: {}",
            unused.join(", ")
        ));
    }
    let rig = Rig2dAsset {
        texture: String::new(),
        size: [0, 0],
        filter: SheetFilter::Linear,
        unit: 1.0,
        bones: top.bones,
        slots: top.slots,
        attachments: top.attachments,
        iks: top.iks,
        transforms: top.transforms,
        paths: top.paths,
        clips: top.clips,
        skins: top_skins,
        armatures,
    };
    // What the runtime reads back: refused here (a display showing its own armature) rather than there.
    let mut probe = rig.clone();
    probe.texture = "probe".into();
    Rig2dAsset::parse(&probe.to_text()).map_err(|e| e.reason().to_string())?;
    Ok(rig)
}

/// One DragonBones armature's bones, slots, displays, constraints and
/// animations; `children` collects the armatures its displays show.
fn armature(
    arm: &Json,
    ske: &Json,
    subs: &[(String, [f32; 4])],
    children: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> Result<(Armature2d, Vec<Skin2d>), String> {
    let rate = arm["frameRate"]
        .as_f64()
        .or(ske["frameRate"].as_f64())
        .unwrap_or(24.0) as f32;
    let mut rig = Armature2d::default();
    let bone_of = |rig: &Armature2d, n: &str| rig.bones.iter().position(|b| b.name == n);
    for b in arm["bone"].as_array().into_iter().flatten() {
        let n = name(b)?;
        let parent = match b["parent"].as_str() {
            Some(p) => {
                Some(bone_of(&rig, p).ok_or(format!("bone {n}: parent {p} comes after it"))?)
            }
            None => None,
        };
        let t = place(&b["transform"]);
        rig.bones.push(Bone2d {
            name: n,
            parent,
            x: t.x,
            y: t.y,
            rotation: t.rotation,
            skew: t.skew,
            scale: t.scale,
            length: f(b, "length", 0.0),
        });
    }
    if rig.bones.is_empty() {
        return Err("the armature has no bones".into());
    }
    let setup = {
        let probe = Rig2dAsset {
            texture: String::new(),
            size: [0, 0],
            filter: SheetFilter::Linear,
            unit: 1.0,
            bones: rig.bones.clone(),
            slots: Vec::new(),
            attachments: Vec::new(),
            iks: Vec::new(),
            transforms: Vec::new(),
            paths: Vec::new(),
            clips: Vec::new(),
            skins: Vec::new(),
            armatures: Vec::new(),
        };
        probe.pose(None, 0.0, &[]).world
    };
    let sub = |n: &str| -> Result<[f32; 4], String> {
        subs.iter()
            .find(|(s, _)| s == n)
            .map(|(_, r)| *r)
            .ok_or(format!("no sub-texture `{n}` in the atlas"))
    };
    let raw_slots: Vec<&Json> = arm["slot"].as_array().into_iter().flatten().collect();
    for sl in &raw_slots {
        let n = name(sl)?;
        let bone_name = sl["parent"]
            .as_str()
            .ok_or(format!("slot {n} has no parent bone"))?;
        let bone = bone_of(&rig, bone_name).ok_or(format!("slot {n}: no bone {bone_name}"))?;
        let (color, offset) = color(&sl["color"]);
        rig.slots.push(Slot2d {
            name: n,
            bone,
            attachment: None,
            color,
            offset,
        });
    }
    // The skins, the default first (DragonBones' defaultSkin: the one named
    // `default`, else the first); each other one swaps the default's
    // displays, slot by slot and index by index (its replaceSkin).
    let all: Vec<&Json> = arm["skin"].as_array().into_iter().flatten().collect();
    let first = all
        .iter()
        .position(|k| k["name"].as_str() == Some("default"))
        .unwrap_or(0);
    let order: Vec<&Json> = all
        .get(first)
        .into_iter()
        .copied()
        .chain(
            all.iter()
                .enumerate()
                .filter(|(i, _)| *i != first)
                .map(|(_, k)| *k),
        )
        .collect();
    // Per slot: the default skin's displays' attachment indices (None for a display not imported).
    let mut displays: Vec<Vec<Option<usize>>> = vec![Vec::new(); raw_slots.len()];
    let mut skipped = Vec::new();
    let mut skins: Vec<Skin2d> = Vec::new();
    for (ki, skin) in order.iter().enumerate() {
        let skin_name = skin["name"].as_str().unwrap_or("").to_string();
        if ki > 0
            && (skin_name.is_empty()
                || skin_name == "default"
                || skins.iter().any(|k: &Skin2d| k.name == skin_name))
        {
            notes.push(format!(
                "a skin without a name of its own is not imported (`{skin_name}`)"
            ));
            continue;
        }
        let mut swaps = Vec::new();
        for (si, sl) in raw_slots.iter().enumerate() {
            let n = rig.slots[si].name.clone();
            let bone = rig.slots[si].bone;
            let list = skin["slot"]
                .as_array()
                .and_then(|l| l.iter().find(|s| s["name"].as_str() == Some(n.as_str())))
                .and_then(|s| s["display"].as_array());
            let Some(list) = list else { continue };
            let mut ids = Vec::new();
            for d in list {
                if d.is_null() {
                    ids.push(None);
                    continue;
                }
                let dn = name(d)?;
                let path = d["path"].as_str().unwrap_or(&dn).to_string();
                let mut an = if ki == 0 {
                    dn.clone()
                } else {
                    format!("{skin_name}/{dn}")
                };
                if rig.attachments.iter().any(|a| a.name == an) {
                    an = if ki == 0 {
                        format!("{n}/{dn}")
                    } else {
                        format!("{skin_name}/{n}/{dn}")
                    };
                }
                let t = place(&d["transform"]);
                let kind = match d["type"].as_str().unwrap_or("image") {
                    "image" => {
                        let rect = sub(&path)?;
                        let size = [rect[2] * t.scale[0], rect[3] * t.scale[1]];
                        // The pivot (0.5 the centre) moves the centre off the display's origin.
                        let (px, py) = (f(&d["pivot"], "x", 0.5), f(&d["pivot"], "y", 0.5));
                        let shift = Affine2::trsk(0.0, 0.0, t.rotation, t.skew, 1.0, 1.0)
                            .apply((0.5 - px) * size[0], -(0.5 - py) * size[1]);
                        AttachmentKind::Region {
                            rect,
                            at: [t.x + shift[0], t.y + shift[1]],
                            rotation: t.rotation,
                            skew: t.skew,
                            size,
                        }
                    }
                    "mesh" => mesh(d, &dn, &sub(&path)?, bone, &setup)?,
                    "path" => {
                        // Points as a mesh's vertices are read (y flipped).
                        let points = influences(d, &format!("path {an}"), bone, &setup)?;
                        if points.len() < 6 || points.len() % 3 != 0 {
                            skipped.push(format!(
                            "{n}/{dn} (a path of {} vertices, not three per knot of two or more)",
                            points.len()
                        ));
                            ids.push(None);
                            continue;
                        }
                        let closed = d["closed"].as_bool().unwrap_or(false);
                        let curves = points.len() / 3 - usize::from(!closed);
                        let lengths: Vec<f32> = d["lengths"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                            .take(curves)
                            .collect();
                        let constant = d["constantSpeed"].as_bool().unwrap_or(false);
                        AttachmentKind::Path {
                            points,
                            closed,
                            // Without a length per curve only arc length can place a position.
                            constant: constant || lengths.len() != curves,
                            lengths: if lengths.len() == curves {
                                lengths
                            } else {
                                Vec::new()
                            },
                        }
                    }
                    "boundingBox" => {
                        let m = t.affine();
                        let (w, h) = (f(d, "width", 0.0) / 2.0, f(d, "height", 0.0) / 2.0);
                        let local: Vec<[f32; 2]> = match d["subType"]
                            .as_str()
                            .unwrap_or("rectangle")
                        {
                            "rectangle" => vec![[-w, -h], [w, -h], [w, h], [-w, h]],
                            "ellipse" => {
                                notes.push(format!("box {an}: the ellipse is a 32-sided polygon"));
                                (0..32)
                                    .map(|i| {
                                        let (s, c) = og_anim::sincos(
                                            f64::from(i) * std::f64::consts::PI / 16.0,
                                        );
                                        [w * c as f32, h * s as f32]
                                    })
                                    .collect()
                            }
                            "polygon" => {
                                let v: Vec<f32> = d["vertices"]
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                                    .collect();
                                v.as_chunks::<2>().0.iter().map(|p| [p[0], -p[1]]).collect()
                            }
                            other => return Err(format!("box {an}: subType {other}")),
                        };
                        if local.len() < 3 {
                            return Err(format!("box {an}: fewer than three points"));
                        }
                        AttachmentKind::Box {
                            polygon: local.into_iter().map(|p| m.apply_to(p)).collect(),
                        }
                    }
                    "armature" => {
                        let index = match children.iter().position(|c| *c == path) {
                            Some(i) => i,
                            None => {
                                children.push(path.clone());
                                children.len() - 1
                            }
                        };
                        AttachmentKind::Child {
                            armature: index,
                            clip: play_action(&d["actions"])
                                .or_else(|| play_action(&sl["actions"])),
                            at: [t.x, t.y],
                            rotation: t.rotation,
                            skew: t.skew,
                            scale: t.scale,
                        }
                    }
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
            if ki == 0 {
                let shown = sl["displayIndex"].as_i64().unwrap_or(0);
                rig.slots[si].attachment = usize::try_from(shown)
                    .ok()
                    .and_then(|i| ids.get(i).copied().flatten());
                displays[si] = ids;
                continue;
            }
            for (i, to) in ids.into_iter().enumerate() {
                match displays[si].get(i).copied().flatten() {
                Some(from) => swaps.push((from, to)),
                None if to.is_some() => notes.push(format!(
                    "skin {skin_name}: slot {n}'s display {i} has none in the default skin to replace, and is not shown"
                )),
                None => {}
            }
            }
        }
        if ki > 0 {
            skins.push(Skin2d {
                name: skin_name,
                swaps,
            });
        }
    }
    if !skipped.is_empty() {
        notes.push(format!("displays not imported: {}", skipped.join(", ")));
    }
    for k in arm["ik"].as_array().into_iter().flatten() {
        let n = name(k)?;
        let bone =
            bone_of(&rig, k["bone"].as_str().unwrap_or("")).ok_or(format!("ik {n}: no bone"))?;
        let target = bone_of(&rig, k["target"].as_str().unwrap_or(""))
            .ok_or(format!("ik {n}: no target"))?;
        // `chain` counts the ancestors above `bone` in the chain.
        let mut bones = vec![bone];
        for _ in 0..k["chain"].as_u64().unwrap_or(0) {
            let up = rig.bones[bones[0]]
                .parent
                .ok_or(format!("ik {n}: its chain runs past the root bone"))?;
            bones.insert(0, up);
        }
        rig.iks.push(Ik2d {
            name: n,
            bones,
            target,
            counterclockwise: !k["bendPositive"].as_bool().unwrap_or(true),
            weight: f(k, "weight", 1.0),
        });
    }
    // Path constraints (the armature's `path` list): bones along a path display.
    for k in arm["path"].as_array().into_iter().flatten() {
        let n = name(k)?;
        let target = k["target"].as_str().unwrap_or("");
        let slot = rig
            .slots
            .iter()
            .position(|s| s.name == target)
            .ok_or(format!("path {n}: no slot {target}"))?;
        let display = k["targetDisplay"].as_str().unwrap_or(target);
        let path = rig
            .attachments
            .iter()
            .position(|a| {
                a.slot == slot
                    && matches!(a.kind, AttachmentKind::Path { .. })
                    && (a.name == display || a.name.ends_with(&format!("/{display}")))
            })
            .ok_or(format!("path {n}: slot {target} shows no path {display}"))?;
        let bones = k["bones"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|b| {
                let b = b.as_str().unwrap_or("");
                bone_of(&rig, b).ok_or(format!("path {n}: no bone {b}"))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if bones.is_empty() {
            return Err(format!("path {n}: no bones"));
        }
        let mode = |key: &str| k[key].as_str().unwrap_or("").to_lowercase();
        rig.paths.push(PathConstraint2d {
            name: n,
            path,
            bones,
            position: f(k, "position", 0.0),
            position_unit: if mode("positionMode") == "fixed" {
                PathUnit::Fixed
            } else {
                PathUnit::Percent
            },
            spacing: f(k, "spacing", 0.0),
            spacing_mode: match mode("spacingMode").as_str() {
                "fixed" => PathSpacing::Fixed,
                "percent" => PathSpacing::Percent,
                _ => PathSpacing::Length,
            },
            rotate: match mode("rotateMode").as_str() {
                "chain" => PathRotate::Chain,
                "chainscale" => PathRotate::ChainScale,
                _ => PathRotate::Tangent,
            },
            // Clockwise with y down is negative here.
            rotation: -f(k, "rotateOffset", 0.0),
            mix: [
                f(k, "rotateMix", 1.0).clamp(0.0, 1.0),
                f(k, "translateMix", 1.0).clamp(0.0, 1.0),
            ],
        });
    }
    for a in arm["animation"].as_array().into_iter().flatten() {
        let n = name(a)?;
        let mut clip = Clip2d {
            name: n.clone(),
            duration: f(a, "duration", 1.0).max(1.0) / rate,
            looping: a["playTimes"].as_u64().unwrap_or(1) == 0,
            bones: Vec::new(),
            slots: Vec::new(),
            deforms: Vec::new(),
            order: Vec::new(),
            iks: Vec::new(),
            events: Vec::new(),
        };
        for t in a["bone"].as_array().into_iter().flatten() {
            let bone = bone_of(&rig, t["name"].as_str().unwrap_or(""))
                .ok_or(format!("animation {n}: no bone {}", t["name"]))?;
            let mut timeline = BoneTimeline {
                bone,
                ..Default::default()
            };
            for (time, k) in frames(&t["translateFrame"], rate) {
                timeline.translate.push(Key {
                    time,
                    value: [f(k, "x", 0.0), -f(k, "y", 0.0)],
                    ease: ease(k, notes),
                });
            }
            let rotate_frames = frames(&t["rotateFrame"], rate);
            let turns = turns(&rotate_frames);
            for ((time, k), turn) in rotate_frames.into_iter().zip(turns) {
                let e = ease(k, notes);
                timeline.rotate.push(Key {
                    time,
                    value: Rotation {
                        degrees: -f(k, "rotate", 0.0),
                        turn,
                    },
                    ease: e.clone(),
                });
                timeline.skew.push(Key {
                    time,
                    value: -f(k, "skew", 0.0),
                    ease: e,
                });
            }
            if timeline.skew.iter().all(|k| k.value == 0.0) {
                timeline.skew.clear();
            }
            for (time, k) in frames(&t["scaleFrame"], rate) {
                timeline.scale.push(Key {
                    time,
                    value: [f(k, "x", 1.0), f(k, "y", 1.0)],
                    ease: ease(k, notes),
                });
            }
            clip.bones.push(timeline);
        }
        for t in a["slot"].as_array().into_iter().flatten() {
            let slot = rig
                .slots
                .iter()
                .position(|s| Some(s.name.as_str()) == t["name"].as_str());
            let slot = slot.ok_or(format!("animation {n}: no slot {}", t["name"]))?;
            let mut timeline = SlotTimeline {
                slot,
                attachment: frames(&t["displayFrame"], rate)
                    .into_iter()
                    .map(|(time, k)| {
                        let i = k["value"]
                            .as_i64()
                            .or(k["displayIndex"].as_i64())
                            .unwrap_or(0);
                        (
                            time,
                            usize::try_from(i)
                                .ok()
                                .and_then(|i| displays[slot].get(i).copied().flatten()),
                        )
                    })
                    .collect(),
                ..Default::default()
            };
            for (time, k) in frames(&t["colorFrame"], rate) {
                let raw = if k.get("value").is_some() {
                    &k["value"]
                } else {
                    &k["color"]
                };
                let (mul, add) = color(raw);
                let e = ease(k, notes);
                timeline.color.push(Key {
                    time,
                    value: mul,
                    ease: e.clone(),
                });
                timeline.offset.push(Key {
                    time,
                    value: add,
                    ease: e,
                });
            }
            if timeline.offset.iter().all(|k| k.value == [0.0; 4])
                && rig.slots[slot].offset == [0.0; 4]
            {
                timeline.offset.clear();
            }
            clip.slots.push(timeline);
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
            // A deform of a skin other than the default names that skin.
            let skin = t["skin"]
                .as_str()
                .filter(|k| !k.is_empty() && *k != "default");
            let attachment = rig
                .attachments
                .iter()
                .position(|x| {
                    let own = x.name.as_str();
                    let in_skin = skins
                        .iter()
                        .any(|k| own.starts_with(&format!("{}/", k.name)));
                    let name = t["name"].as_str().unwrap_or("");
                    Some(x.slot) == slot
                        && match skin {
                            Some(k) => {
                                own.starts_with(&format!("{k}/"))
                                    && own.ends_with(&format!("/{name}"))
                            }
                            None => !in_skin && (own == name || own.ends_with(&format!("/{name}"))),
                        }
                })
                .ok_or(format!(
                    "animation {n}: no mesh {} in slot {}",
                    t["name"], t["slot"]
                ))?;
            let len = 2 * rig.attachments[attachment].influences();
            let mut keys = Vec::new();
            for (time, k) in frames(&t["frame"], rate) {
                let mut v = vec![0.0f32; len];
                let off = k["offset"].as_u64().unwrap_or(0) as usize;
                for (i, d) in k["vertices"].as_array().into_iter().flatten().enumerate() {
                    if let Some(slot) = v.get_mut(off + i) {
                        // Every odd entry is a y: down there, up here.
                        let s = if (off + i) % 2 == 1 { -1.0 } else { 1.0 };
                        *slot = s * d.as_f64().unwrap_or(0.0) as f32;
                    }
                }
                keys.push(Key {
                    time,
                    value: v,
                    ease: ease(k, notes),
                });
            }
            clip.deforms.push(DeformTimeline { attachment, keys });
        }
        // zOrder: each frame moves slots (index, offset pairs) from the setup order.
        for (time, k) in frames(&a["zOrder"]["frame"], rate) {
            let pairs: Vec<i64> = k["zOrder"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|v| v.as_i64().unwrap_or(0))
                .collect();
            let order = z_order(&pairs, rig.slots.len()).ok_or(format!(
                "animation {n}: a zOrder frame moves a slot out of range"
            ))?;
            clip.order.push((time, order));
        }
        for t in a["ik"].as_array().into_iter().flatten() {
            let ik = rig
                .iks
                .iter()
                .position(|k| Some(k.name.as_str()) == t["name"].as_str())
                .ok_or(format!("animation {n}: no ik {}", t["name"]))?;
            let mut keys = Vec::new();
            for (time, k) in frames(&t["frame"], rate) {
                keys.push(Key {
                    time,
                    value: IkKey {
                        weight: f(k, "weight", 1.0).clamp(0.0, 1.0),
                        counterclockwise: !k["bendPositive"].as_bool().unwrap_or(true),
                    },
                    ease: ease(k, notes),
                });
            }
            clip.iks.push(IkTimeline { ik, keys });
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
        clip.events.sort_by(|a, b| a.0.total_cmp(&b.0));
        rig.clips.push(clip);
    }
    Ok((rig, skins))
}

/// Each rotate frame's turn to the next, as DragonBones' parser
/// (`_parseBoneRotateFrame`) accumulates the angles: after a frame with
/// `clockwise` 0 the next angle is the nearer one; after `clockwise` n > 0
/// (n < 0) it is the next angle at or past this one in the increasing
/// (decreasing) direction plus n - 1 whole turns (n + 1), or n whole turns
/// when the raw angle is behind. With y down there, the increasing
/// direction is clockwise on screen: here the turn's sign flips, and each
/// turn is the one whose [`og_anim::turn_delta`] over the written angles
/// is that difference.
fn turns(list: &[(f32, &Json)]) -> Vec<Turn> {
    let raw: Vec<f64> = list
        .iter()
        .map(|(_, k)| k.get("rotate").and_then(Json::as_f64).unwrap_or(0.0))
        .collect();
    let mut out = vec![Turn::Shorter; list.len()];
    let (mut prev, mut prev_cw) = (0.0f64, 0i64);
    for (i, (_, k)) in list.iter().enumerate() {
        let mut r = raw[i];
        if i > 0 {
            if prev_cw == 0 {
                // Transform.normalizeRadian in degrees: (-180, 180].
                let mut d = (r - prev + 180.0) % 360.0;
                d += if d > 0.0 { -180.0 } else { 180.0 };
                r = prev + d;
            } else {
                let mut n = prev_cw;
                if (n > 0 && r >= prev) || (n < 0 && r <= prev) {
                    n -= n.signum();
                }
                r += 360.0 * n as f64;
                // The written angles are y up: the turn here is -(r - prev).
                let delta = -(r - prev) as f32;
                let (a, b) = (-raw[i - 1] as f32, -raw[i] as f32);
                let way = |n: u32| {
                    if delta >= 0.0 {
                        Turn::Counterclockwise(n)
                    } else {
                        Turn::Clockwise(n)
                    }
                };
                let whole = ((delta.abs() - turn_delta(a, b, way(0)).abs()) / 360.0).round();
                out[i - 1] = way(whole.max(0.0) as u32);
            }
        }
        prev = r;
        prev_cw = k
            .get("clockwise")
            .and_then(Json::as_f64)
            .unwrap_or(0.0)
            .round() as i64;
    }
    out
}

/// A DragonBones zOrder frame (pairs of a slot index in the setup order
/// and how far it moves) as a draw order of every slot; empty for none.
fn z_order(pairs: &[i64], count: usize) -> Option<Vec<usize>> {
    if pairs.is_empty() {
        return Some(Vec::new());
    }
    let mut order: Vec<Option<usize>> = vec![None; count];
    let mut unchanged = Vec::new();
    let mut original = 0usize;
    for p in pairs.as_chunks::<2>().0 {
        let slot = usize::try_from(p[0]).ok().filter(|s| *s < count)?;
        while original < slot {
            unchanged.push(original);
            original += 1;
        }
        let to = usize::try_from(original as i64 + p[1])
            .ok()
            .filter(|t| *t < count)?;
        *order.get_mut(to)? = Some(original);
        original += 1;
    }
    unchanged.extend(original..count);
    // The slots not moved fill the places left, last first.
    for o in order.iter_mut().rev() {
        if o.is_none() {
            *o = Some(unchanged.pop()?);
        }
    }
    order.into_iter().collect()
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
    let count = vs.len() / 2;
    if uvs.len() != vs.len() || count == 0 {
        return Err(format!(
            "mesh {n}: {} vertex numbers and {} uv numbers",
            vs.len(),
            uvs.len()
        ));
    }
    let influences = influences(d, &format!("mesh {n}"), slot_bone, setup)?;
    let vertices = influences
        .into_iter()
        .enumerate()
        .map(|(i, influences)| MeshVertex2d {
            uv: [
                rect[0] + uvs[2 * i] * rect[2],
                rect[1] + uvs[2 * i + 1] * rect[3],
            ],
            influences,
        })
        .collect();
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

/// A DragonBones geometry's vertices (a mesh's or a path's) as influences:
/// unweighted, in the slot's bone; weighted, through `slotPose` into the
/// setup pose and from there into each bone of `weights`.
fn influences(
    d: &Json,
    what: &str,
    slot_bone: usize,
    setup: &[Affine2],
) -> Result<Vec<Vec<Influence>>, String> {
    let nums = |k: &str| -> Vec<f32> {
        d[k].as_array()
            .into_iter()
            .flatten()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect()
    };
    let vs = nums("vertices");
    let weights = nums("weights");
    let count = vs.len() / 2;
    let n = what;
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
            let k = *weights.get(w).ok_or(format!("{n}: weights end early"))? as usize;
            w += 1;
            let mut out = Vec::with_capacity(k);
            for _ in 0..k {
                let (b, wt) = (weights.get(w).copied(), weights.get(w + 1).copied());
                let (Some(b), Some(wt)) = (b, wt) else {
                    return Err(format!("{n}: weights end early"));
                };
                w += 2;
                let b = b as usize;
                let m = setup.get(b).ok_or(format!("{n}: weight on bone {b}"))?;
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
        vertices.push(influences);
    }
    Ok(vertices)
}
