//! Armature, skin and clip text parsing.

use super::*;

pub(super) fn parse_err(line: usize, what: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Parse, format!("line {line}: {what}"))
}

pub(super) fn name_of(e: &Entry) -> Result<String> {
    e.values
        .first()
        .and_then(|v| v.as_word().or(v.as_str()))
        .map(str::to_string)
        .ok_or_else(|| parse_err(e.line, format!("`{}` needs a name", e.key)))
}

pub(super) fn num(e: &Entry, i: usize) -> Result<f32> {
    let v = e
        .f32(i)
        .map_err(|_| parse_err(e.line, format!("`{}` needs a number at {}", e.key, i + 1)))?;
    if !v.is_finite() {
        return Err(parse_err(e.line, format!("`{}`: {v} is not finite", e.key)));
    }
    Ok(v)
}

fn word(e: &Entry, i: usize) -> Result<String> {
    e.values
        .get(i)
        .and_then(|v| v.as_word().or(v.as_str()))
        .map(str::to_string)
        .ok_or_else(|| parse_err(e.line, format!("`{}` needs a name at {}", e.key, i + 1)))
}

pub(super) fn block(e: &Entry) -> Result<&Block> {
    e.block.as_ref().ok_or_else(|| {
        parse_err(
            e.line,
            format!("`{} <name> {{ ... }}` needs a block", e.key),
        )
    })
}

/// The ease written from value `from` on: nothing, `step`, or `curve` and
/// its control points.
fn ease_at(e: &Entry, from: usize) -> Result<Ease> {
    match e.values.get(from).map(Value::as_word) {
        None => Ok(Ease::Linear),
        Some(Some("step")) if e.values.len() == from + 1 => Ok(Ease::Step),
        Some(Some("curve")) => {
            let c = (from + 1..e.values.len())
                .map(|i| num(e, i))
                .collect::<Result<Vec<f32>>>()?;
            check_curve(&c).map_err(|w| parse_err(e.line, w))?;
            Ok(Ease::Bezier(c))
        }
        _ => Err(parse_err(
            e.line,
            format!(
                "`{}`: after its values a key takes `step` or `curve x1 y1 x2 y2 [x y x1 y1 x2 y2]...`",
                e.key
            ),
        )),
    }
}

fn index(names: &[String], name: &str, what: &str, line: usize) -> Result<usize> {
    names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| parse_err(line, format!("no {what} `{name}`")))
}

/// The keys of an armature's body (the rig's top level, or an `armature` block).
pub(super) const BODY_KEYS: [&str; 11] = [
    "bone",
    "slot",
    "region",
    "mesh",
    "box",
    "path",
    "child",
    "ik",
    "transform",
    "along",
    "clip",
];

/// Reads an armature's bones, slots, attachments, constraints and clips
/// from `body` (`armatures`: the rig's child armatures, by name).
pub(super) fn parse_body(body: &[Entry], armatures: &[String]) -> Result<Armature2d> {
    let mut rig = Armature2d::default();
    // Bones, parent first.
    let mut bone_names: Vec<String> = Vec::new();
    for e in body.iter().filter(|e| e.key == "bone") {
        let name = name_of(e)?;
        if bone_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("bone `{name}` is declared twice"),
            ));
        }
        let mut b = Bone2d {
            name: name.clone(),
            parent: None,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            skew: 0.0,
            scale: [1.0, 1.0],
            length: 0.0,
        };
        if let Some(blk) = &e.block {
            for c in &blk.entries {
                match c.key.as_str() {
                    "parent" => {
                        let p = word(c, 0)?;
                        b.parent =
                            Some(bone_names.iter().position(|n| *n == p).ok_or_else(|| {
                                parse_err(
                                    c.line,
                                    format!("bone {name}: parent `{p}` is not declared before it"),
                                )
                            })?);
                    }
                    "at" => (b.x, b.y) = (num(c, 0)?, num(c, 1)?),
                    "rotation" => b.rotation = num(c, 0)?,
                    "skew" => b.skew = num(c, 0)?,
                    "scale" => b.scale = [num(c, 0)?, num(c, 1)?],
                    "length" => b.length = num(c, 0)?,
                    k => {
                        return Err(parse_err(
                            c.line,
                            format!("unknown key `{k}` in bone {name}"),
                        ));
                    }
                }
            }
        }
        bone_names.push(name);
        rig.bones.push(b);
    }
    if rig.bones.is_empty() {
        return Err(Error::new(
            ErrorKind::Parse,
            "a 2D rig (and each armature) needs at least one bone",
        ));
    }
    // Slots, then attachments (which name their slot), then the slots' setup attachments.
    let mut slot_names: Vec<String> = Vec::new();
    let mut slot_attachment: Vec<Option<(String, usize)>> = Vec::new();
    for e in body.iter().filter(|e| e.key == "slot") {
        let name = name_of(e)?;
        if slot_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("slot `{name}` is declared twice"),
            ));
        }
        let blk = block(e)?;
        let mut bone = None;
        let mut attachment = None;
        let mut color = [1.0; 4];
        let mut offset = [0.0; 4];
        for c in &blk.entries {
            match c.key.as_str() {
                "bone" => bone = Some(index(&bone_names, &word(c, 0)?, "bone", c.line)?),
                "attachment" => attachment = Some((word(c, 0)?, c.line)),
                "color" => color = [num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?],
                "offset" => offset = [num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?],
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in slot {name}"),
                    ));
                }
            }
        }
        let bone = bone.ok_or_else(|| parse_err(e.line, format!("slot {name} needs `bone`")))?;
        rig.slots.push(Slot2d {
            name: name.clone(),
            bone,
            attachment: None,
            color,
            offset,
        });
        slot_attachment.push(attachment);
        slot_names.push(name);
    }
    let mut attachment_names: Vec<String> = Vec::new();
    for e in body
        .iter()
        .filter(|e| matches!(e.key.as_str(), "region" | "mesh" | "box" | "path" | "child"))
    {
        let name = name_of(e)?;
        if attachment_names.contains(&name) {
            return Err(parse_err(
                e.line,
                format!("attachment `{name}` is declared twice"),
            ));
        }
        let blk = block(e)?;
        let slot = blk
            .get("slot")
            .ok_or_else(|| parse_err(e.line, format!("{} {name} needs `slot`", e.key)))?;
        let slot_index = index(&slot_names, &word(slot, 0)?, "slot", slot.line)?;
        let unknown = |c: &Entry| {
            parse_err(
                c.line,
                format!("unknown key `{}` in {} {name}", c.key, e.key),
            )
        };
        let kind = match e.key.as_str() {
            "region" => {
                let mut rect = None;
                let mut at = [0.0; 2];
                let mut rotation = 0.0;
                let mut skew = 0.0;
                let mut size = None;
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "rect" => rect = Some([num(c, 0)?, num(c, 1)?, num(c, 2)?, num(c, 3)?]),
                        "at" => at = [num(c, 0)?, num(c, 1)?],
                        "rotation" => rotation = num(c, 0)?,
                        "skew" => skew = num(c, 0)?,
                        "size" => size = Some([num(c, 0)?, num(c, 1)?]),
                        _ => return Err(unknown(c)),
                    }
                }
                let rect = rect.ok_or_else(|| {
                    parse_err(e.line, format!("region {name} needs `rect x y w h`"))
                })?;
                AttachmentKind::Region {
                    rect,
                    at,
                    rotation,
                    skew,
                    size: size.unwrap_or([rect[2], rect[3]]),
                }
            }
            "box" => {
                let mut polygon = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "polygon" => {
                            if c.values.len() < 6 || c.values.len() % 2 != 0 {
                                return Err(parse_err(
                                    c.line,
                                    format!("box {name}: polygon is at least three x y points"),
                                ));
                            }
                            for i in (0..c.values.len()).step_by(2) {
                                polygon.push([num(c, i)?, num(c, i + 1)?]);
                            }
                        }
                        _ => return Err(unknown(c)),
                    }
                }
                if polygon.is_empty() {
                    return Err(parse_err(
                        e.line,
                        format!("box {name} needs `polygon x y x y x y ...`"),
                    ));
                }
                AttachmentKind::Box { polygon }
            }
            "path" => {
                let slot_bone = rig.slots[slot_index].bone;
                let mut points = Vec::new();
                let mut closed = false;
                let mut constant = true;
                let mut lengths = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "closed" => closed = flag(c)?,
                        "constant" => constant = flag(c)?,
                        "lengths" => {
                            lengths = (0..c.values.len())
                                .map(|i| num(c, i))
                                .collect::<Result<_>>()?
                        }
                        "vertex" => points.push(if c.values.len() == 2 {
                            vec![Influence {
                                bone: slot_bone,
                                x: num(c, 0)?,
                                y: num(c, 1)?,
                                weight: 1.0,
                            }]
                        } else {
                            weighted(c, 0, &bone_names, &format!("path {name}"))?
                        }),
                        _ => return Err(unknown(c)),
                    }
                }
                if points.len() < 6 || !points.len().is_multiple_of(3) {
                    return Err(parse_err(
                        e.line,
                        format!(
                            "path {name}: its vertices are three per knot (handle in, knot, handle out), at least two knots ({} vertices)",
                            points.len()
                        ),
                    ));
                }
                let curves = points.len() / 3 - usize::from(!closed);
                if !constant && lengths.len() != curves {
                    return Err(parse_err(
                        e.line,
                        format!(
                            "path {name}: `constant false` needs `lengths`, each of its {curves} curves' end along the path"
                        ),
                    ));
                }
                AttachmentKind::Path {
                    points,
                    closed,
                    constant,
                    lengths,
                }
            }
            "child" => {
                let mut armature = None;
                let mut clip = None;
                let mut at = [0.0; 2];
                let mut rotation = 0.0;
                let mut skew = 0.0;
                let mut scale = [1.0; 2];
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "armature" => {
                            armature = Some(index(armatures, &word(c, 0)?, "armature", c.line)?)
                        }
                        "clip" => clip = Some(word(c, 0)?),
                        "at" => at = [num(c, 0)?, num(c, 1)?],
                        "rotation" => rotation = num(c, 0)?,
                        "skew" => skew = num(c, 0)?,
                        "scale" => scale = [num(c, 0)?, num(c, 1)?],
                        _ => return Err(unknown(c)),
                    }
                }
                AttachmentKind::Child {
                    armature: armature.ok_or_else(|| {
                        parse_err(e.line, format!("child {name} needs `armature <name>`"))
                    })?,
                    clip,
                    at,
                    rotation,
                    skew,
                    scale,
                }
            }
            _ => {
                let slot_bone = rig.slots[slot_index].bone;
                let mut vertices = Vec::new();
                let mut triangles = Vec::new();
                for c in &blk.entries {
                    match c.key.as_str() {
                        "slot" => {}
                        "vertex" => {
                            let uv = [num(c, 0)?, num(c, 1)?];
                            let influences = if c.values.len() == 4 {
                                vec![Influence {
                                    bone: slot_bone,
                                    x: num(c, 2)?,
                                    y: num(c, 3)?,
                                    weight: 1.0,
                                }]
                            } else {
                                weighted(c, 2, &bone_names, &format!("mesh {name}"))?
                            };
                            vertices.push(MeshVertex2d { uv, influences });
                        }
                        "triangles" => {
                            for i in 0..c.values.len() {
                                let v = num(c, i)?;
                                if v < 0.0 || v.fract() != 0.0 {
                                    return Err(parse_err(
                                        c.line,
                                        format!("mesh {name}: triangles are vertex indices"),
                                    ));
                                }
                                triangles.push(v as u32);
                            }
                        }
                        _ => return Err(unknown(c)),
                    }
                }
                if triangles.is_empty() || triangles.len() % 3 != 0 {
                    return Err(parse_err(
                        e.line,
                        format!("mesh {name}: triangles are three indices each"),
                    ));
                }
                if let Some(t) = triangles.iter().find(|t| **t as usize >= vertices.len()) {
                    return Err(parse_err(
                        e.line,
                        format!("mesh {name}: index {t} of {} vertices", vertices.len()),
                    ));
                }
                AttachmentKind::Mesh {
                    vertices,
                    triangles,
                }
            }
        };
        rig.attachments.push(Attachment2d {
            name: name.clone(),
            slot: slot_index,
            kind,
        });
        attachment_names.push(name);
    }
    for (s, a) in slot_attachment.into_iter().enumerate() {
        if let Some((a, line)) = a
            && a != "none"
        {
            let i = index(&attachment_names, &a, "attachment", line)?;
            if rig.attachments[i].slot != s {
                return Err(parse_err(
                    line,
                    format!("attachment {a} belongs to another slot"),
                ));
            }
            rig.slots[s].attachment = Some(i);
        }
    }
    for e in body.iter().filter(|e| e.key == "ik") {
        let name = name_of(e)?;
        let blk = block(e)?;
        let mut ik = Ik2d {
            name: name.clone(),
            bones: Vec::new(),
            target: usize::MAX,
            counterclockwise: false,
            weight: 1.0,
        };
        for c in &blk.entries {
            match c.key.as_str() {
                "bones" => {
                    for i in 0..c.values.len() {
                        ik.bones
                            .push(index(&bone_names, &word(c, i)?, "bone", c.line)?);
                    }
                }
                "target" => ik.target = index(&bone_names, &word(c, 0)?, "bone", c.line)?,
                "bend" => ik.counterclockwise = bend(c, 0)?,
                "weight" => ik.weight = num(c, 0)?.clamp(0.0, 1.0),
                k => return Err(parse_err(c.line, format!("unknown key `{k}` in ik {name}"))),
            }
        }
        if ik.bones.is_empty() {
            return Err(parse_err(
                e.line,
                format!("ik {name}: `bones` names the chain, root first"),
            ));
        }
        if ik
            .bones
            .windows(2)
            .any(|w| rig.bones[w[1]].parent != Some(w[0]))
        {
            return Err(parse_err(
                e.line,
                format!("ik {name}: each bone of `bones` is the previous one's child"),
            ));
        }
        if ik.target == usize::MAX {
            return Err(parse_err(e.line, format!("ik {name} needs `target`")));
        }
        rig.iks.push(ik);
    }
    let bones_of = |c: &Entry, what: &str| -> Result<Vec<usize>> {
        let list = (0..c.values.len())
            .map(|i| index(&bone_names, &word(c, i)?, "bone", c.line))
            .collect::<Result<Vec<_>>>()?;
        if list.is_empty() {
            return Err(parse_err(
                c.line,
                format!("{what}: `bones` names at least one"),
            ));
        }
        Ok(list)
    };
    for e in body.iter().filter(|e| e.key == "transform") {
        let name = name_of(e)?;
        let mut t = TransformConstraint2d {
            name: name.clone(),
            bones: Vec::new(),
            target: usize::MAX,
            rotation: 0.0,
            at: [0.0; 2],
            scale: [0.0; 2],
            shear: 0.0,
            mix: [1.0, 1.0, 1.0, 1.0],
        };
        for c in &block(e)?.entries {
            match c.key.as_str() {
                "bones" => t.bones = bones_of(c, &format!("transform {name}"))?,
                "target" => t.target = index(&bone_names, &word(c, 0)?, "bone", c.line)?,
                "rotation" => t.rotation = num(c, 0)?,
                "at" => t.at = [num(c, 0)?, num(c, 1)?],
                "scale" => t.scale = [num(c, 0)?, num(c, 1)?],
                "shear" => t.shear = num(c, 0)?,
                "mix" => {
                    t.mix = [0, 1, 2, 3]
                        .map(|i| num(c, i).map(|v| v.clamp(0.0, 1.0)))
                        .into_iter()
                        .collect::<Result<Vec<_>>>()?
                        .try_into()
                        .expect("four")
                }
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in transform {name}"),
                    ));
                }
            }
        }
        if t.bones.is_empty() || t.target == usize::MAX {
            return Err(parse_err(
                e.line,
                format!("transform {name} needs `bones` and `target`"),
            ));
        }
        if t.bones.contains(&t.target) {
            return Err(parse_err(
                e.line,
                format!("transform {name}: its target is not one of its bones"),
            ));
        }
        rig.transforms.push(t);
    }
    for e in body.iter().filter(|e| e.key == "along") {
        let name = name_of(e)?;
        let mut p = PathConstraint2d {
            name: name.clone(),
            path: usize::MAX,
            bones: Vec::new(),
            position: 0.0,
            position_unit: PathUnit::Percent,
            spacing: 0.0,
            spacing_mode: PathSpacing::Length,
            rotate: PathRotate::Tangent,
            rotation: 0.0,
            mix: [1.0, 1.0],
        };
        let mode = |c: &Entry, i: usize, words: &[&str]| -> Result<usize> {
            let w = word(c, i)?;
            words.iter().position(|x| *x == w).ok_or_else(|| {
                parse_err(
                    c.line,
                    format!(
                        "along {name}: `{}` takes {}, not `{w}`",
                        c.key,
                        words.join(" or ")
                    ),
                )
            })
        };
        for c in &block(e)?.entries {
            match c.key.as_str() {
                "path" => {
                    let a = index(&attachment_names, &word(c, 0)?, "attachment", c.line)?;
                    if !matches!(rig.attachments[a].kind, AttachmentKind::Path { .. }) {
                        return Err(parse_err(
                            c.line,
                            format!("along {name}: {} is not a path", attachment_names[a]),
                        ));
                    }
                    p.path = a;
                }
                "bones" => p.bones = bones_of(c, &format!("along {name}"))?,
                "position" => {
                    p.position = num(c, 0)?;
                    if c.values.len() > 1 {
                        p.position_unit = [PathUnit::Percent, PathUnit::Fixed]
                            [mode(c, 1, &["percent", "fixed"])?];
                    }
                }
                "spacing" => {
                    p.spacing = num(c, 0)?;
                    if c.values.len() > 1 {
                        p.spacing_mode = [
                            PathSpacing::Length,
                            PathSpacing::Fixed,
                            PathSpacing::Percent,
                        ][mode(c, 1, &["length", "fixed", "percent"])?];
                    }
                }
                "rotate" => {
                    p.rotate = [
                        PathRotate::Tangent,
                        PathRotate::Chain,
                        PathRotate::ChainScale,
                    ][mode(c, 0, &["tangent", "chain", "chainscale"])?];
                    if c.values.len() > 1 {
                        p.rotation = num(c, 1)?;
                    }
                }
                "mix" => p.mix = [num(c, 0)?.clamp(0.0, 1.0), num(c, 1)?.clamp(0.0, 1.0)],
                k => {
                    return Err(parse_err(
                        c.line,
                        format!("unknown key `{k}` in along {name}"),
                    ));
                }
            }
        }
        if p.bones.is_empty() || p.path == usize::MAX {
            return Err(parse_err(
                e.line,
                format!("along {name} needs `path` and `bones`"),
            ));
        }
        rig.paths.push(p);
    }
    let names = Names {
        bones: &bone_names,
        slots: &slot_names,
        attachments: &attachment_names,
    };
    for e in body.iter().filter(|e| e.key == "clip") {
        rig.clips.push(parse_clip(e, &rig, &names)?);
    }
    Ok(rig)
}

/// A body's `skin` blocks (the top level's, or an armature's):
/// `swap <attachment> <attachment or none>` lines, each pair in one slot
/// of that body's `attachments`.
pub(super) fn parse_skins(body: &[Entry], attachments: &[Attachment2d]) -> Result<Vec<Skin2d>> {
    let find = |n: &str, line: usize| -> Result<usize> {
        attachments
            .iter()
            .position(|a| a.name == n)
            .ok_or_else(|| parse_err(line, format!("no attachment `{n}`")))
    };
    let mut out: Vec<Skin2d> = Vec::new();
    for e in body.iter().filter(|e| e.key == "skin") {
        let name = name_of(e)?;
        if name == "default" || out.iter().any(|k| k.name == name) {
            return Err(parse_err(
                e.line,
                format!(
                    "skin `{name}` is declared twice (the rig's own attachments are `default`)"
                ),
            ));
        }
        let mut skin = Skin2d {
            name,
            swaps: Vec::new(),
        };
        for c in &block(e)?.entries {
            if c.key != "swap" {
                return Err(parse_err(
                    c.line,
                    format!(
                        "unknown key `{}` in skin {} (swap <attachment> <attachment or none>)",
                        c.key, skin.name
                    ),
                ));
            }
            let from = find(&word(c, 0)?, c.line)?;
            let to = match word(c, 1)?.as_str() {
                "none" => None,
                t => Some(find(t, c.line)?),
            };
            if to.is_some_and(|t| attachments[t].slot != attachments[from].slot) {
                return Err(parse_err(
                    c.line,
                    format!("skin {}: a swap stays in its slot", skin.name),
                ));
            }
            skin.swaps.push((from, to));
        }
        out.push(skin);
    }
    Ok(out)
}

/// A vertex's influences from value `from` on: `bone x y weight` each,
/// the weights summing to 1.
fn weighted(c: &Entry, from: usize, bones: &[String], what: &str) -> Result<Vec<Influence>> {
    if c.values.len() < from + 4 || !(c.values.len() - from).is_multiple_of(4) {
        return Err(parse_err(
            c.line,
            format!(
                "{what}: a vertex is {}x y, or {}bone x y weight per influence",
                if from == 2 { "u v " } else { "" },
                if from == 2 { "u v then " } else { "" }
            ),
        ));
    }
    let mut out = Vec::new();
    for k in (from..c.values.len()).step_by(4) {
        out.push(Influence {
            bone: index(bones, &word(c, k)?, "bone", c.line)?,
            x: num(c, k + 1)?,
            y: num(c, k + 2)?,
            weight: num(c, k + 3)?,
        });
    }
    let total: f32 = out.iter().map(|i| i.weight).sum();
    if (total - 1.0).abs() > 1e-3 {
        return Err(parse_err(
            c.line,
            format!("{what}: the weights sum to {total}, not 1"),
        ));
    }
    Ok(out)
}

/// `true` or `false` at value 0.
fn flag(c: &Entry) -> Result<bool> {
    c.values
        .first()
        .and_then(Value::as_bool)
        .ok_or_else(|| parse_err(c.line, format!("`{}` is true or false", c.key)))
}

/// `clockwise` (false) or `counterclockwise` (true) at value `i`.
fn bend(c: &Entry, i: usize) -> Result<bool> {
    match word(c, i)?.as_str() {
        "clockwise" => Ok(false),
        "counterclockwise" => Ok(true),
        w => Err(parse_err(
            c.line,
            format!("bend is clockwise or counterclockwise, not `{w}`"),
        )),
    }
}

/// An armature's names, for its clips' lookups.
struct Names<'a> {
    bones: &'a [String],
    slots: &'a [String],
    attachments: &'a [String],
}

fn parse_clip(e: &Entry, rig: &Armature2d, names: &Names<'_>) -> Result<Clip2d> {
    let name = name_of(e)?;
    let blk = block(e)?;
    let mut clip = Clip2d {
        name: name.clone(),
        duration: 0.0,
        looping: true,
        bones: Vec::new(),
        slots: Vec::new(),
        deforms: Vec::new(),
        order: Vec::new(),
        iks: Vec::new(),
        events: Vec::new(),
    };
    let time = |c: &Entry| -> Result<f32> {
        let t = num(c, 0)?;
        if t < 0.0 {
            return Err(parse_err(
                c.line,
                format!("clip {name}: a key's time is at least 0"),
            ));
        }
        Ok(t)
    };
    let sorted = |times: &[f32], line: usize| -> Result<()> {
        if times.windows(2).any(|w| w[1] < w[0]) {
            return Err(parse_err(
                line,
                format!("clip {name}: keys are in time order"),
            ));
        }
        Ok(())
    };
    fn times<T>(keys: &[Key<T>]) -> Vec<f32> {
        keys.iter().map(|k| k.time).collect()
    }
    let key1 = |k: &Entry| -> Result<Key<f32>> {
        Ok(Key {
            time: time(k)?,
            value: num(k, 1)?,
            ease: ease_at(k, 2)?,
        })
    };
    let key2 = |k: &Entry| -> Result<Key<[f32; 2]>> {
        Ok(Key {
            time: time(k)?,
            value: [num(k, 1)?, num(k, 2)?],
            ease: ease_at(k, 3)?,
        })
    };
    let key4 = |k: &Entry| -> Result<Key<[f32; 4]>> {
        Ok(Key {
            time: time(k)?,
            value: [num(k, 1)?, num(k, 2)?, num(k, 3)?, num(k, 4)?],
            ease: ease_at(k, 5)?,
        })
    };
    for c in &blk.entries {
        match c.key.as_str() {
            "duration" => clip.duration = num(c, 0)?,
            "loop" => clip.looping = c.values.first().and_then(|v| v.as_bool()).unwrap_or(true),
            "event" => {
                let n = c
                    .string(1)
                    .map_err(|_| parse_err(c.line, "event <time> \"name\""))?
                    .to_string();
                if n.len() > 63 {
                    return Err(parse_err(
                        c.line,
                        format!("event `{n}` is longer than 63 bytes"),
                    ));
                }
                clip.events.push((time(c)?, n));
            }
            "bone" => {
                let mut t = BoneTimeline {
                    bone: index(names.bones, &name_of(c)?, "bone", c.line)?,
                    ..Default::default()
                };
                for k in &block(c)?.entries {
                    match k.key.as_str() {
                        "rotate" => {
                            // `clockwise <turns>` or `counterclockwise <turns>` may follow the angle.
                            let (turn, ease_from) = match k.values.get(2).and_then(Value::as_word) {
                                Some(w @ ("clockwise" | "counterclockwise")) => {
                                    let n = num(k, 3)?;
                                    if n < 0.0 || n.fract() != 0.0 || n > 1e6 {
                                        return Err(parse_err(
                                            k.line,
                                            format!(
                                                "rotate: {w} takes a whole number of further turns (0 the nearest {w} way)"
                                            ),
                                        ));
                                    }
                                    let n = n as u32;
                                    let turn = if w == "clockwise" {
                                        Turn::Clockwise(n)
                                    } else {
                                        Turn::Counterclockwise(n)
                                    };
                                    (turn, 4)
                                }
                                _ => (Turn::Shorter, 2),
                            };
                            t.rotate.push(Key {
                                time: time(k)?,
                                value: Rotation {
                                    degrees: num(k, 1)?,
                                    turn,
                                },
                                ease: ease_at(k, ease_from)?,
                            });
                        }
                        "skew" => t.skew.push(key1(k)?),
                        "translate" => t.translate.push(key2(k)?),
                        "scale" => t.scale.push(key2(k)?),
                        x => {
                            return Err(parse_err(
                                k.line,
                                format!("unknown key `{x}` in a bone timeline"),
                            ));
                        }
                    }
                }
                sorted(&times(&t.rotate), c.line)?;
                sorted(&times(&t.skew), c.line)?;
                sorted(&times(&t.translate), c.line)?;
                sorted(&times(&t.scale), c.line)?;
                clip.bones.push(t);
            }
            "slot" => {
                let slot = index(names.slots, &name_of(c)?, "slot", c.line)?;
                let mut t = SlotTimeline {
                    slot,
                    ..Default::default()
                };
                for k in &block(c)?.entries {
                    match k.key.as_str() {
                        "attachment" => {
                            let a = word(k, 1)?;
                            let a = if a == "none" {
                                None
                            } else {
                                let i = index(names.attachments, &a, "attachment", k.line)?;
                                if rig.attachments[i].slot != slot {
                                    return Err(parse_err(
                                        k.line,
                                        format!("attachment {a} belongs to another slot"),
                                    ));
                                }
                                Some(i)
                            };
                            t.attachment.push((time(k)?, a));
                        }
                        "color" => t.color.push(key4(k)?),
                        "offset" => t.offset.push(key4(k)?),
                        x => {
                            return Err(parse_err(
                                k.line,
                                format!("unknown key `{x}` in a slot timeline"),
                            ));
                        }
                    }
                }
                sorted(
                    &t.attachment.iter().map(|k| k.0).collect::<Vec<_>>(),
                    c.line,
                )?;
                sorted(&times(&t.color), c.line)?;
                sorted(&times(&t.offset), c.line)?;
                clip.slots.push(t);
            }
            "deform" => {
                let a = index(names.attachments, &name_of(c)?, "attachment", c.line)?;
                let len = 2 * rig.attachments[a].influences();
                if len == 0 {
                    return Err(parse_err(
                        c.line,
                        format!(
                            "clip {name}: deform names {}, not a mesh or a path; only those deform",
                            names.attachments[a]
                        ),
                    ));
                }
                let mut keys = Vec::new();
                for k in &block(c)?.entries {
                    if k.key != "key" {
                        return Err(parse_err(
                            k.line,
                            format!("unknown key `{}` in a deform timeline", k.key),
                        ));
                    }
                    // The offsets run to the first word (`step` or `curve`).
                    let end = k
                        .values
                        .iter()
                        .position(|v| v.as_word().is_some())
                        .unwrap_or(k.values.len());
                    let count = end.saturating_sub(1);
                    if count != len {
                        return Err(parse_err(
                            k.line,
                            format!(
                                "deform {}: {count} offsets for {} influences (x y each)",
                                names.attachments[a],
                                len / 2
                            ),
                        ));
                    }
                    keys.push(Key {
                        time: time(k)?,
                        value: (1..=count).map(|i| num(k, i)).collect::<Result<_>>()?,
                        ease: ease_at(k, end)?,
                    });
                }
                sorted(&times(&keys), c.line)?;
                clip.deforms.push(DeformTimeline {
                    attachment: a,
                    keys,
                });
            }
            "order" => {
                let mut order = Vec::new();
                for i in 1..c.values.len() {
                    order.push(index(names.slots, &word(c, i)?, "slot", c.line)?);
                }
                let mut seen = order.clone();
                seen.sort_unstable();
                seen.dedup();
                if !order.is_empty()
                    && (seen.len() != order.len() || order.len() != rig.slots.len())
                {
                    return Err(parse_err(
                        c.line,
                        format!(
                            "clip {name}: an order names each of the {} slots once (or none, the setup order)",
                            rig.slots.len()
                        ),
                    ));
                }
                clip.order.push((time(c)?, order));
            }
            "ik" => {
                let ik_name = name_of(c)?;
                let ik = rig
                    .iks
                    .iter()
                    .position(|k| k.name == ik_name)
                    .ok_or_else(|| parse_err(c.line, format!("no ik `{ik_name}`")))?;
                let mut keys = Vec::new();
                for k in &block(c)?.entries {
                    if k.key != "key" {
                        return Err(parse_err(
                            k.line,
                            format!(
                                "unknown key `{}` in an ik timeline (key <time> <weight> <bend>)",
                                k.key
                            ),
                        ));
                    }
                    keys.push(Key {
                        time: time(k)?,
                        value: IkKey {
                            weight: num(k, 1)?.clamp(0.0, 1.0),
                            counterclockwise: bend(k, 2)?,
                        },
                        ease: ease_at(k, 3)?,
                    });
                }
                sorted(&times(&keys), c.line)?;
                clip.iks.push(IkTimeline { ik, keys });
            }
            x => {
                return Err(parse_err(
                    c.line,
                    format!("unknown key `{x}` in clip {name}"),
                ));
            }
        }
    }
    if clip.duration.is_nan() || clip.duration <= 0.0 {
        return Err(parse_err(
            e.line,
            format!("clip {name} needs `duration` above 0"),
        ));
    }
    sorted(&clip.order.iter().map(|o| o.0).collect::<Vec<_>>(), e.line)?;
    og_core::sort::sort_by(&mut clip.events, |a, b| a.0.total_cmp(&b.0));
    Ok(clip)
}
