//! A model's stated orientation (`front: '-x', up: '+y'` in its
//! import settings): the file's axes that are the object's front (its
//! nose, the way it travels) and its up, turned in the cook to the
//! engine's +Z and +Y, as other engines' importers take
//! a forward and an up axis. A generated or bought model often faces
//! another way; the game then places it as authored, with no turn of its
//! own.
//!
//! The source's glTF is rewritten before anything reads it (the fracture
//! cut, the load, the animation and LOD bakes): each scene's roots go
//! under one new node holding the turn, so meshes, skins, animations and
//! fracture chunks all turn together and no index the file holds moves.
//! The axes read as `og model place --front/--up` reads them.

use og_shape::tools::Axis;
use serde_json::{Value, json};

/// The node the turn is held in.
pub const NODE: &str = "og:orient";

/// The model's stated front and up (`None`: it faces as authored). Either
/// may be left out: up is +y unless the front lies along y; with only up,
/// the front is the smallest turn's (`+z`, or `-y` for a `+z` up, `+y`
/// for a `-z` up, as `og model place --z-up`).
pub fn from_input(
    input: Option<&crate::cook::source_schemas::world::import::Model>,
) -> Result<Option<(Axis, Axis)>, String> {
    let Some(d) = input else {
        return Ok(None);
    };
    let axis = |a: &crate::cook::source_schemas::world::import::Axis| {
        use crate::cook::source_schemas::world::import::Axis as Input;
        match a {
            Input::X => Axis::PosX,
            Input::NegX => Axis::NegX,
            Input::Y => Axis::PosY,
            Input::NegY => Axis::NegY,
            Input::Z => Axis::PosZ,
            Input::NegZ => Axis::NegZ,
        }
    };
    let (front, up) = (d.front.as_ref().map(axis), d.up.as_ref().map(axis));
    let (front, up) = match (front, up) {
        (None, None) => return Ok(None),
        (Some(f), Some(u)) => (f, u),
        (Some(f), None) => (
            f,
            match f {
                Axis::PosY | Axis::NegY => {
                    return Err(format!(
                        "front {f} lies along y, the default up: give up too (`up: '+z'`)"
                    ));
                }
                _ => Axis::PosY,
            },
        ),
        (None, Some(u)) => (
            match u {
                Axis::PosZ => Axis::NegY,
                Axis::NegZ => Axis::PosY,
                _ => Axis::PosZ,
            },
            u,
        ),
    };
    if up.cross(front).is_none() {
        return Err(format!(
            "front {front} and up {up} lie along one axis; give two axes at right angles"
        ));
    }
    Ok(Some((front, up)))
}

/// The rotation taking `front` to +Z and `up` to +Y, as a glTF quaternion
/// (x, y, z, w). Right-handed: the side is up × front, to +X.
pub fn rotation(front: Axis, up: Axis) -> [f32; 4] {
    let v = |a: Axis| {
        let d = a.vector();
        glam::Vec3::new(d.x as f32, d.y as f32, d.z as f32)
    };
    let (f, u) = (v(front), v(up));
    let side = u.cross(f);
    let turn = glam::Mat3::from_cols(side, u, f).transpose();
    glam::Quat::from_mat3(&turn).normalize().to_array()
}

/// The glTF (a `.glb`'s bytes or a `.gltf`'s text) with each scene's roots
/// under one node turning `front` to +Z and `up` to +Y.
pub fn apply(bytes: &[u8], front: Axis, up: Axis) -> Result<Vec<u8>, String> {
    let (text, bin) = super::plain::chunks(bytes)?;
    let mut json: Value =
        serde_json::from_slice(text).map_err(|e| format!("glTF: {e} (orienting the model)"))?;
    let nodes = json["nodes"].as_array().map_or(0, Vec::len);
    // The roots of each scene; a file without scenes is one scene of the
    // nodes no node holds.
    let mut scenes: Vec<Vec<Value>> = match json.get("scenes").and_then(Value::as_array) {
        Some(s) => s
            .iter()
            .map(|s| s["nodes"].as_array().cloned().unwrap_or_default())
            .collect(),
        None => {
            let mut held = vec![false; nodes];
            for n in json["nodes"].as_array().into_iter().flatten() {
                for c in n["children"].as_array().into_iter().flatten() {
                    if let Some(c) = c.as_u64().filter(|&c| (c as usize) < nodes) {
                        held[c as usize] = true;
                    }
                }
            }
            vec![(0..nodes).filter(|&n| !held[n]).map(|n| json!(n)).collect()]
        }
    };
    // A node has one parent: two scenes sharing a root cannot both hold it
    // under their own turn.
    let mut seen = std::collections::BTreeSet::new();
    for r in scenes.iter().flatten() {
        if !seen.insert(r.as_u64()) {
            return Err(format!(
                "glTF: node {r} is a root of more than one scene; front and up turn each scene's roots, so a node can be in one scene only"
            ));
        }
    }
    let rotation = rotation(front, up);
    if json.get("nodes").and_then(Value::as_array).is_none() {
        json["nodes"] = json!([]);
    }
    let list = json["nodes"].as_array_mut().expect("an array");
    let mut wrappers = Vec::new();
    for roots in scenes.drain(..) {
        list.push(json!({ "name": NODE, "rotation": rotation, "children": roots }));
        wrappers.push(list.len() - 1);
    }
    match json.get_mut("scenes").and_then(Value::as_array_mut) {
        Some(s) => {
            for (scene, w) in s.iter_mut().zip(&wrappers) {
                scene["nodes"] = json!([w]);
            }
        }
        None => {
            json["scenes"] = json!([{ "nodes": [wrappers[0]] }]);
            json["scene"] = json!(0);
        }
    }
    let mut text = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    if bin.is_none() && !bytes.starts_with(b"glTF") {
        return Ok(text);
    }
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let bin = bin.unwrap_or_default();
    let padded = bin.len().next_multiple_of(4);
    let total = 12 + 8 + text.len() + if bin.is_empty() { 0 } else { 8 + padded };
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    if !bin.is_empty() {
        out.extend_from_slice(&(padded as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&bin);
        out.resize(total, 0);
    }
    Ok(out)
}
