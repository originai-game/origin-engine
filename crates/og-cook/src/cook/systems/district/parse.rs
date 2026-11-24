//! A district's text (`type district`) and a material library's (`type
//! materials`).

use std::collections::BTreeSet;

use og_core::ogt::{Block, Document, Entry, Value};
use og_shape::generators::layout::{GridDesc, LotsDesc, PropDesc, Region, StreetDesc};

use glam::DVec2;

pub(crate) const DISTRICT_TYPE: &str = "district";
pub(crate) const MATERIALS_TYPE: &str = "materials";

fn at(e: &Entry, msg: impl std::fmt::Display) -> String {
    format!("line {}: `{}`: {msg}", e.line, e.key)
}

fn numbers(e: &Entry, n: usize) -> Result<Vec<f64>, String> {
    if e.values.len() != n || e.block.is_some() {
        return Err(at(e, format!("takes {n} number(s)")));
    }
    e.values
        .iter()
        .map(|v| match v {
            Value::Number(x) if x.as_f64().is_finite() => Ok(x.as_f64()),
            _ => Err(at(e, format!("takes {n} number(s)"))),
        })
        .collect()
}

/// Above zero; NaN is not.
fn positive(x: f64) -> bool {
    x > 0.0
}

fn one(e: &Entry) -> Result<f64, String> {
    Ok(numbers(e, 1)?[0])
}

fn text(e: &Entry) -> Result<String, String> {
    match e.values.as_slice() {
        [Value::String(s)] if !s.is_empty() && e.block.is_none() => Ok(s.clone()),
        _ => Err(at(e, "takes one string, in quotes")),
    }
}

fn asset(e: &Entry) -> Result<String, String> {
    Ok(crate::runtime::asset_id(&text(e)?).to_string())
}

fn flag(e: &Entry) -> Result<bool, String> {
    match e.values.as_slice() {
        [v] if e.block.is_none() => v.as_bool().ok_or_else(|| at(e, "is true or false")),
        _ => Err(at(e, "is true or false")),
    }
}

fn block(e: &Entry) -> Result<&Block, String> {
    e.block.as_ref().ok_or_else(|| at(e, "takes a block"))
}

fn known(b: &Block, keys: &[&str], what: &str) -> Result<(), String> {
    match b.entries.iter().find(|e| !keys.contains(&e.key.as_str())) {
        Some(e) => Err(at(e, format!("{what} takes {}", keys.join(", ")))),
        None => Ok(()),
    }
}

fn points(b: &Block) -> Result<Vec<DVec2>, String> {
    b.get_all("point")
        .map(|e| numbers(e, 2).map(|v| DVec2::new(v[0], v[1])))
        .collect()
}

/// A range: two numbers, the least first.
fn range(e: &Entry) -> Result<[f64; 2], String> {
    let v = numbers(e, 2)?;
    Ok([v[0], v[1]])
}

/// What a prop collides as.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PropCollider {
    /// Width, height and depth, standing on the ground.
    Box([f64; 3]),
    /// Radius and height between the cap centres, standing on the ground.
    Capsule(f64, f64),
}

/// One kind of prop.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Prop {
    pub desc: PropDesc,
    pub model: String,
    pub collider: Option<PropCollider>,
}

/// A grammar the buildings take.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuleUse {
    pub grammar: String,
    pub weight: f64,
    pub street: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Buildings {
    pub density: f64,
    pub height: [f64; 2],
    /// `heightNoise`: the noise's frequency per metre and its share of
    /// the draw (the rest is each lot's own).
    pub noise: [f64; 2],
    pub rules: Vec<RuleUse>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cells {
    pub size: f64,
    pub load: f64,
}

/// A whole district.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct District {
    pub seed: u64,
    pub region: Region,
    pub grid: Option<GridDesc>,
    pub streets: Vec<StreetDesc>,
    pub kerb: f64,
    pub lots: LotsDesc,
    pub buildings: Buildings,
    pub props: Vec<Prop>,
    pub materials: String,
    pub cells: Option<Cells>,
    /// The `hlod` block, given to each cell's sub-scene as written.
    pub hlod: Option<Entry>,
    pub merge: bool,
}

fn street(e: &Entry) -> Result<StreetDesc, String> {
    let b = block(e)?;
    known(b, &["point", "width", "sidewalk"], "a street")?;
    let name = match e.values.as_slice() {
        [Value::String(s)] => s.clone(),
        [] => String::new(),
        _ => return Err(at(e, "names the street in quotes")),
    };
    Ok(StreetDesc {
        name,
        points: points(b)?,
        width: b.get("width").map(one).transpose()?.unwrap_or(0.0),
        sidewalk: b.get("sidewalk").map(one).transpose()?.unwrap_or(0.0),
    })
}

fn prop(e: &Entry) -> Result<Prop, String> {
    let name = match e.values.as_slice() {
        [Value::String(s)] if !s.is_empty() => s.clone(),
        _ => return Err(at(e, "names the prop in quotes")),
    };
    let b = block(e)?;
    known(
        b,
        &[
            "model", "every", "jitter", "kerb", "corners", "street", "collider",
        ],
        "a prop",
    )?;
    let model = asset(b.get("model").ok_or_else(|| at(e, "names its `model`"))?)?;
    let num = |k: &str| b.get(k).map(one).transpose().map(|v| v.unwrap_or(0.0));
    let collider = match b.get("collider") {
        None => None,
        Some(c) => match c.values.first().and_then(Value::as_word) {
            Some("box") => {
                let v: Vec<f64> = c.values[1..]
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                if v.len() != 3 || c.values.len() != 4 || !v.iter().all(|x| positive(*x)) {
                    return Err(at(
                        c,
                        "collider box takes its width, height and depth, above 0",
                    ));
                }
                Some(PropCollider::Box([v[0], v[1], v[2]]))
            }
            Some("capsule") => {
                let v: Vec<f64> = c.values[1..]
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                if v.len() != 2
                    || c.values.len() != 3
                    || !positive(v[0])
                    || v[1].is_nan()
                    || v[1] < 0.0
                {
                    return Err(at(
                        c,
                        "collider capsule takes its radius and the height between its caps' centres",
                    ));
                }
                Some(PropCollider::Capsule(v[0], v[1]))
            }
            _ => return Err(at(c, "is `box <w> <h> <d>` or `capsule <radius> <height>`")),
        },
    };
    Ok(Prop {
        desc: PropDesc {
            name,
            every: num("every")?,
            jitter: num("jitter")?,
            kerb: num("kerb")?,
            corners: b.get("corners").map(flag).transpose()?.unwrap_or(false),
            street: b.get("street").map(text).transpose()?,
        },
        model,
        collider,
    })
}

fn buildings(e: &Entry) -> Result<Buildings, String> {
    let b = block(e)?;
    known(
        b,
        &["density", "height", "heightNoise", "rule"],
        "buildings",
    )?;
    let mut rules = Vec::new();
    for r in b.get_all("rule") {
        let grammar = crate::runtime::asset_id(
            r.string(0)
                .map_err(|_| at(r, "names a grammar by its asset id, in quotes"))?,
        )
        .to_string();
        let (mut weight, mut street) = (1.0, None);
        if let Some(rb) = &r.block {
            known(rb, &["weight", "street"], "a rule")?;
            if let Some(w) = rb.get("weight") {
                weight = one(w)?;
            }
            street = rb.get("street").map(text).transpose()?;
        }
        if !positive(weight) {
            return Err(at(r, "a rule's weight is above 0"));
        }
        rules.push(RuleUse {
            grammar,
            weight,
            street,
        });
    }
    let out = Buildings {
        density: b.get("density").map(one).transpose()?.unwrap_or(0.0),
        height: b
            .get("height")
            .map(range)
            .transpose()?
            .unwrap_or([0.0, 0.0]),
        noise: b
            .get("heightNoise")
            .map(range)
            .transpose()?
            .unwrap_or([0.0, 0.0]),
        rules,
    };
    if !(0.0..=1.0).contains(&out.density) {
        return Err(at(e, format!("density is 0 to 1, not {}", out.density)));
    }
    if out.height[0] < 0.0
        || out.height[0] > out.height[1]
        || out.noise[0].is_nan()
        || out.noise[0] < 0.0
        || !(0.0..=1.0).contains(&out.noise[1])
    {
        return Err(at(
            e,
            "height is a least and a greatest, 0 or more, the least first; heightNoise is a frequency, 0 or more, and a share, 0 to 1",
        ));
    }
    if out.density > 0.0 && out.rules.is_empty() {
        return Err(at(e, "names at least one `rule` for its buildings"));
    }
    Ok(out)
}

/// Reads a district's text.
pub(crate) fn district(source: &str) -> Result<District, String> {
    let doc = Document::parse(source).map_err(|e| e.to_string())?;
    if doc.ty != DISTRICT_TYPE || doc.version != 1 {
        return Err(format!("a district is `type {DISTRICT_TYPE}`, version 1"));
    }
    let b = &doc.body;
    known(
        b,
        &[
            "seed",
            "region",
            "along",
            "streets",
            "lots",
            "buildings",
            "prop",
            "materials",
            "cells",
            "hlod",
            "merge",
        ],
        "a district",
    )?;
    let seed = b.get("seed").map(one).transpose()?.unwrap_or(0.0);
    if seed < 0.0 || seed.fract() != 0.0 || seed > u32::MAX as f64 {
        return Err("a district's seed is a whole number, 0 to 4294967295".into());
    }
    let region = match (b.get("region"), b.get("along")) {
        (Some(r), None) => {
            let rb = block(r)?;
            known(rb, &["point"], "a region")?;
            Region::Area(points(rb)?)
        }
        (None, Some(a)) => {
            let ab = block(a)?;
            known(
                ab,
                &["point", "width", "sidewalk", "depth", "name"],
                "along",
            )?;
            let num = |k: &str| ab.get(k).map(one).transpose().map(|v| v.unwrap_or(0.0));
            Region::Along {
                street: StreetDesc {
                    name: ab.get("name").map(text).transpose()?.unwrap_or_default(),
                    points: points(ab)?,
                    width: num("width")?,
                    sidewalk: num("sidewalk")?,
                },
                depth: num("depth")?,
            }
        }
        _ => {
            return Err(
                "a district has one `region { point x z ... }` or one `along { ... }`".into(),
            );
        }
    };
    let (mut grid, mut streets, mut kerb) = (None, Vec::new(), 0.0);
    if let Some(s) = b.get("streets") {
        let sb = block(s)?;
        known(
            sb,
            &[
                "grid", "turn", "jitter", "width", "sidewalk", "kerb", "street",
            ],
            "streets",
        )?;
        let num = |k: &str| sb.get(k).map(one).transpose().map(|v| v.unwrap_or(0.0));
        kerb = num("kerb")?;
        if let Some(g) = sb.get("grid") {
            let v = numbers(g, 2)?;
            grid = Some(GridDesc {
                size: DVec2::new(v[0], v[1]),
                turn: num("turn")?,
                jitter: num("jitter")?,
                width: num("width")?,
                sidewalk: num("sidewalk")?,
            });
        }
        streets = sb.get_all("street").map(street).collect::<Result<_, _>>()?;
    }
    if !(0.0..=2.0).contains(&kerb) {
        return Err(format!("streets' kerb is 0 to 2 m, not {kerb}"));
    }
    let lots = match b.get("lots") {
        Some(l) => {
            let lb = block(l)?;
            known(lb, &["width", "depth"], "lots")?;
            let need = |k: &str| {
                lb.get(k)
                    .ok_or_else(|| at(l, format!("states its `{k}`: least and greatest")))
                    .and_then(range)
            };
            LotsDesc {
                width: need("width")?,
                depth: need("depth")?,
            }
        }
        None => return Err(
            "a district states its `lots { width <least> <greatest>; depth <least> <greatest> }`"
                .into(),
        ),
    };
    let buildings = match b.get("buildings") {
        Some(e) => buildings(e)?,
        None => Buildings {
            density: 0.0,
            height: [0.0, 0.0],
            noise: [0.0, 0.0],
            rules: Vec::new(),
        },
    };
    let props = b.get_all("prop").map(prop).collect::<Result<Vec<_>, _>>()?;
    let mut names = BTreeSet::new();
    for p in &props {
        if !names.insert(&p.desc.name) {
            return Err(format!("prop `{}` is written twice", p.desc.name));
        }
    }
    let materials = asset(
        b.get("materials")
            .ok_or("a district names its material library: `materials \"<asset id>\"`")?,
    )?;
    let cells = match b.get("cells") {
        Some(c) => {
            let cb = block(c)?;
            known(cb, &["size", "load"], "cells")?;
            let size = cb.get("size").map(one).transpose()?.unwrap_or(0.0);
            let load = cb.get("load").map(one).transpose()?.unwrap_or(0.0);
            if !(8.0..=4096.0).contains(&size) || !(load.is_finite() && load >= 0.0) {
                return Err(at(c, "size is 8 to 4096 m and load 0 or more"));
            }
            Some(Cells { size, load })
        }
        None => None,
    };
    Ok(District {
        seed: seed as u64,
        region,
        grid,
        streets,
        kerb,
        lots,
        buildings,
        props,
        materials,
        cells,
        hlod: b.get("hlod").cloned(),
        merge: b.get("merge").map(flag).transpose()?.unwrap_or(true),
    })
}

/// A material of a library: what it is drawn with and what it covers.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Material {
    pub name: String,
    pub labels: Vec<String>,
    pub weight: f64,
    /// The metres one texture covers, across and up.
    pub tile: [f64; 2],
    /// Keep the raw metric v for ground-up aging, not a map's repeat and phase.
    pub metric_height: bool,
    /// Each building's own brightness and warmth, up to these either way.
    pub value: f64,
    pub warmth: f64,
    /// What its maps show, for an image model to draw (`brief`), and
    /// whether that is a facade module (`facade true`) rather than a
    /// tileable surface. `og materials` reads them; the cook also keeps
    /// a facade's bottom image row at the building's base.
    pub brief: Option<String>,
    pub facade: bool,
    /// How many metres of height the source image's darkest to lightest
    /// span, for the normal map `og materials make` derives (0: none;
    /// below 0, the lighter is lower, as pale mortar between bricks).
    pub relief: f64,
    /// Linear roughness at black and white; optional derived packed map.
    pub roughness_range: Option<[f64; 2]>,
    /// Cavity darkening strength and sampling radius in texels.
    pub occlusion: Option<[f64; 2]>,
    /// Its glTF fields, as `type model`'s `material` block takes them;
    /// `map` and `normalMap` name asset ids.
    pub fields: Block,
}

/// The keys of a library's material that are not the drawn material's.
const OWN: [&str; 9] = [
    "label",
    "weight",
    "tile",
    "vary",
    "brief",
    "facade",
    "relief",
    "roughnessRange",
    "occlusion",
];

/// Reads a material library's text.
pub(crate) fn materials(source: &str) -> Result<Vec<Material>, String> {
    let doc = Document::parse(source).map_err(|e| e.to_string())?;
    if doc.ty != MATERIALS_TYPE || doc.version != 1 {
        return Err(format!(
            "a material library is `type {MATERIALS_TYPE}`, version 1"
        ));
    }
    known(&doc.body, &["material"], "a material library")?;
    let mut out: Vec<Material> = Vec::new();
    for e in doc.body.get_all("material") {
        let name = text(&Entry {
            block: None,
            ..e.clone()
        })
        .map_err(|_| at(e, "names the material in quotes"))?;
        if out.iter().any(|m| m.name == name) {
            return Err(at(e, format!("material `{name}` is written twice")));
        }
        let b = block(e)?;
        let labels = b
            .get_all("label")
            .map(|l| match l.values.as_slice() {
                [Value::Word(w)] | [Value::String(w)] if !w.is_empty() => Ok(w.clone()),
                _ => Err(at(l, "names a label")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if labels.is_empty() {
            return Err(at(
                e,
                format!("material `{name}` covers at least one `label`"),
            ));
        }
        let weight = b.get("weight").map(one).transpose()?.unwrap_or(1.0);
        let tile = b.get("tile").map(range).transpose()?.unwrap_or([1.0, 1.0]);
        if !positive(weight) || !positive(tile[0]) || !positive(tile[1]) {
            return Err(at(e, "weight and tile are above 0"));
        }
        let (mut value, mut warmth) = (0.0, 0.0);
        if let Some(v) = b.get("vary") {
            let vb = block(v)?;
            known(vb, &["value", "warmth"], "vary")?;
            value = vb.get("value").map(one).transpose()?.unwrap_or(0.0);
            warmth = vb.get("warmth").map(one).transpose()?.unwrap_or(0.0);
            if !(0.0..=0.5).contains(&value) || !(0.0..=0.5).contains(&warmth) {
                return Err(at(v, "value and warmth are 0 to 0.5"));
            }
        }
        let brief = b.get("brief").map(text).transpose()?;
        let facade = b.get("facade").map(flag).transpose()?.unwrap_or(false);
        let relief = b.get("relief").map(one).transpose()?.unwrap_or(0.0);
        let roughness_range = b.get("roughnessRange").map(range).transpose()?;
        if roughness_range.is_some_and(|v| v.iter().any(|v| !(0.0..=1.0).contains(v))) {
            return Err(at(
                e,
                "roughnessRange gives roughness at black and white, both 0 to 1",
            ));
        }
        let occlusion = b.get("occlusion").map(range).transpose()?;
        if occlusion.is_some_and(|v| {
            !(0.0..=1.0).contains(&v[0]) || !(1.0..=64.0).contains(&v[1]) || v[1].fract() != 0.0
        }) {
            return Err(at(
                e,
                "occlusion takes strength (0 to 1) and a whole radius (1 to 64 texels)",
            ));
        }
        if !(-1.0..=1.0).contains(&relief) {
            return Err(at(e, "relief is -1 to 1 m"));
        }
        if (facade || relief != 0.0 || roughness_range.is_some() || occlusion.is_some())
            && brief.is_none()
        {
            return Err(at(
                e,
                "facade and relief are for the maps `og materials make` makes: the material states its `brief`",
            ));
        }
        let fields = Block {
            entries: b
                .entries
                .iter()
                .filter(|x| !OWN.contains(&x.key.as_str()))
                .cloned()
                .collect(),
        };
        out.push(Material {
            name,
            labels,
            weight,
            tile,
            metric_height: b
                .get("grime")
                .map(one)
                .transpose()?
                .is_some_and(|x| x > 0.0),
            value,
            warmth,
            brief,
            facade,
            relief,
            roughness_range,
            occlusion,
            fields,
        });
    }
    Ok(out)
}
