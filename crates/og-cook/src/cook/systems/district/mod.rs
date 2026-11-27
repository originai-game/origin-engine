//! Districts (`type district`): a whole area laid out from rules and a
//! seed, expanded by the scan into the sources the rest of the cook
//! already cooks, so levels of detail, colliders, instancing, streaming
//! cells and HLOD proxies are the existing ones.
//!
//! - og-shape's `layout` lays out the streets, blocks, lots and prop sites;
//!   og-shape's `grammar` builds each lot's building from the rules the
//!   district names (`type grammar`), every part with its label.
//! - A material library (`type materials`) gives each label its
//!   materials: each building draws one per label by weight from its own
//!   seed (labels covered by the same materials, a wall and its jambs,
//!   draw the same one), with its own brightness and warmth in its vertex colours and
//!   its own phase of the pattern, so neighbours differ and one building
//!   stays one brick.
//! - The district becomes a scene. Without a `cells` block it holds
//!   everything; with one, each cell of `size` metres that holds anything
//!   is an og.StreamCell whose sub-scene (`<district>.cell-<i>-<j>.ogt`)
//!   is its own cell group, with the district's `hlod` block. A cell's
//!   buildings, streets and pavements are one model per cell
//!   (`<district>.cell-<i>-<j>.glb`, a primitive per material), or one per
//!   building with `merge false`; props and the models a grammar inserts
//!   are entities placing their own model, so copies draw instanced.
//!   Every building collides as the box round its lot and its height, a
//!   prop as the shape its `collider` line states. Every entity is named
//!   under the district's id (`<district>/building-<k>`), as names are
//!   unique in the whole world, across cells and districts.
//!
//! The grammars and the library are part of the district (consumed); the
//! textures the library names are read by each model, as a glTF's are.

mod build;
mod glb;
pub(crate) mod parse;
pub(super) mod terrain;
mod texts;

use super::*;

pub(crate) use parse::{DISTRICT_TYPE, MATERIALS_TYPE};

/// A district's kind: expanded by the scan into a scene, so it plans
/// nothing of its own (an expansion that failed carries its error).
pub(crate) static DISTRICT: KindDef = KindDef {
    name: "district",
    extensions: &[],
    types: &[DISTRICT_TYPE],
    by_content: false,
    streams: false,
    plan: plan_nothing,
};

/// A grammar: part of the districts that name it.
pub(crate) static GRAMMAR: KindDef = KindDef {
    name: "grammar",
    extensions: &[],
    types: &[og_shape::generators::grammar::GRAMMAR_TYPE],
    by_content: false,
    streams: false,
    plan: plan_check,
};

/// A material library: part of the districts that name it.
pub(crate) static MATERIALS: KindDef = KindDef {
    name: "materials",
    extensions: &[],
    types: &[MATERIALS_TYPE],
    by_content: false,
    streams: false,
    plan: plan_check,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&DISTRICT, &GRAMMAR, &MATERIALS];

fn plan_nothing<'a>(
    _: &Cx<'a>,
    _: &mut PlanState,
    _: &'a Source,
    _: [u8; 8],
    _: &mut Vec<Out<'a>>,
) {
}

/// A grammar or a library no district names cooks nothing, but a mistake
/// in it is still reported.
fn plan_check<'a>(
    _: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    _: [u8; 8],
    _: &mut Vec<Out<'a>>,
) {
    let text = String::from_utf8_lossy(&s.bytes);
    let checked = if s.kind == &GRAMMAR {
        og_shape::generators::grammar::parse(&text).map(|_| ())
    } else {
        parse::materials(&text).map(|_| ())
    };
    if let Err(e) = checked {
        state.early_errors.push((s.rel.clone(), s.kind.name, e));
    }
}

/// Every district becomes a scene and the sources it places; the grammars
/// and libraries they name are consumed.
pub(crate) fn expand(sources: &mut Vec<Source>, consumed: &mut BTreeSet<String>) {
    let mut extra = Vec::new();
    let terrains: BTreeSet<String> = sources
        .iter()
        .filter(|s| s.kind == &super::terrain::TERRAIN)
        .map(|s| s.rel.clone())
        .collect();
    let mut carves = Vec::new();
    for s in sources.iter_mut() {
        if s.kind != &DISTRICT || s.error.is_some() {
            continue;
        }
        let root = crate::cook::assets_root(s);
        let read = |id: &str| {
            std::fs::read(root.join(id)).map_err(|_| format!("`{id}` is not a file under assets/"))
        };
        let text = String::from_utf8_lossy(&s.bytes).into_owned();
        let built = parse::district(&text).and_then(|d| {
            // What it names is part of it, built or not.
            consumed.insert(d.materials.clone());
            consumed.extend(d.buildings.rules.iter().map(|r| r.grammar.clone()));
            if let Some((id, _)) = &d.terrain
                && !terrains.contains(id)
            {
                return Err(format!(
                    "terrain `{id}` is not a terrain asset under assets/"
                ));
            }
            build::build(&s.rel, &d, &read)
        });
        let out = match built {
            Ok(o) => o,
            Err(e) => {
                s.error = Some(e);
                continue;
            }
        };
        s.bytes = out.scene.into_bytes();
        s.kind = &scene::SCENE;
        if let Some(carve) = out.carve {
            carves.push(carve);
        }
        let made = |rel: String, kind: Kind, bytes: Vec<u8>| Source {
            path: s
                .path
                .with_file_name(rel.rsplit('/').next().unwrap_or(&rel)),
            rel,
            kind,
            bytes,
            settings: None,
            settings_files: Vec::new(),
            hash: 0,
            gltf: None,
            error: None,
            encoded: Default::default(),
            whole: Default::default(),
            model: Default::default(),
            video: None,
            timeline: None,
            fields: Default::default(),
            terrains: Default::default(),
            conformed: None,
            fracture: None,
        };
        for (rel, text) in out.scenes {
            extra.push(made(rel, &scene::SCENE, text.into_bytes()));
        }
        for (rel, glb) in out.models {
            extra.push(made(rel, &crate::cook::model::MODEL, glb));
        }
    }
    for (id, carve) in carves {
        // Settings enter the cook stamp. A nav bake reading this terrain
        // needs the same carve and must invalidate when its streets move.
        let carve = carve.with(og_core::ogt::Value::String(id.clone()));
        for s in sources.iter_mut() {
            let nav_uses = s.kind == &super::navmesh::NAVMESH
                && Document::parse(&String::from_utf8_lossy(&s.bytes)).is_ok_and(|doc| {
                    doc.body
                        .get_all("terrain")
                        .any(|e| e.values.first().and_then(|v| v.as_str()) == Some(id.as_str()))
                });
            if s.rel != id && !nav_uses {
                continue;
            }
            let (text, doc) = s
                .settings
                .get_or_insert_with(|| (String::new(), Document::new("import", 1)));
            doc.body.push(carve.clone());
            *text = doc.to_text().expect("generated terrain settings");
        }
    }
    sources.extend(extra);
}
