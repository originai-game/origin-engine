//! Native district declarations and the material library's author parser.

use std::collections::BTreeSet;

use og_core::ogt::{Block, Document, Entry, Value};
use og_shape::generators::layout::{GridDesc, LotsDesc, PropDesc, Region, StreetDesc};

use glam::DVec2;

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
    pub hlod: Option<crate::cook::source_schemas::world::scene::HlodInput>,
    pub merge: bool,
    /// The terrain asset, in the district's local space, and carve depth.
    pub terrain: Option<(String, f64)>,
}

fn street(s: input::Street) -> StreetDesc {
    StreetDesc {
        name: s.name.unwrap_or_default(),
        points: s.points.into_iter().map(DVec2::from_array).collect(),
        width: s.width.unwrap_or(0.0),
        sidewalk: s.sidewalk.unwrap_or(0.0),
    }
}

use crate::cook::source_schemas::world::district as input;

/// Checked TS data enters layout without a second author text format.
pub(crate) fn district(d: input::DistrictInput) -> Result<District, String> {
    let region = match d.region {
        input::Region::Area { points } => {
            Region::Area(points.into_iter().map(DVec2::from_array).collect())
        }
        input::Region::Along { street: s, depth } => Region::Along {
            street: street(s),
            depth: depth.unwrap_or(0.0),
        },
    };
    let (grid, streets, kerb) = match d.streets {
        Some(s) => (
            s.grid.map(|g| GridDesc {
                size: DVec2::from_array(g.size),
                turn: g.turn.unwrap_or(0.0),
                jitter: g.jitter.unwrap_or(0.0),
                width: g.width.unwrap_or(0.0),
                sidewalk: g.sidewalk.unwrap_or(0.0),
            }),
            s.streets.into_iter().map(street).collect(),
            s.kerb.unwrap_or(0.0),
        ),
        None => (None, Vec::new(), 0.0),
    };
    let buildings = match d.buildings {
        Some(b) => Buildings {
            density: b.density.unwrap_or(0.0),
            height: b.height.unwrap_or([0.0; 2]),
            noise: b.height_noise.map_or([0.0; 2], |n| [n.frequency, n.share]),
            rules: b
                .rules
                .into_iter()
                .map(|r| RuleUse {
                    grammar: r.grammar.id,
                    weight: r.weight.unwrap_or(1.0),
                    street: r.street,
                })
                .collect(),
        },
        None => Buildings {
            density: 0.0,
            height: [0.0; 2],
            noise: [0.0; 2],
            rules: Vec::new(),
        },
    };
    if buildings.height[0] > buildings.height[1] {
        return Err("building height is a least and a greatest, the least first".into());
    }
    if buildings.density > 0.0 && buildings.rules.is_empty() {
        return Err("nonzero building density needs at least one grammar rule".into());
    }
    let mut names = BTreeSet::new();
    let mut props = Vec::new();
    for p in d.props {
        if !names.insert(p.name.clone()) {
            return Err(format!("prop `{}` is written twice", p.name));
        }
        props.push(Prop {
            desc: PropDesc {
                name: p.name,
                every: p.every.unwrap_or(0.0),
                jitter: p.jitter.unwrap_or(0.0),
                kerb: p.kerb.unwrap_or(0.0),
                corners: p.corners.unwrap_or(false),
                street: p.street,
            },
            model: p.model.id,
            collider: p.collider.map(|c| match c {
                input::Collider::Box { size } => PropCollider::Box(size),
                input::Collider::Capsule { radius, height } => {
                    PropCollider::Capsule(radius, height)
                }
            }),
        });
    }
    Ok(District {
        seed: d.seed.unwrap_or(0).into(),
        region,
        grid,
        streets,
        kerb,
        lots: LotsDesc {
            width: d.lots.width,
            depth: d.lots.depth,
        },
        buildings,
        props,
        materials: d.materials.id,
        cells: d.cells.map(|c| Cells {
            size: c.size,
            load: c.load.unwrap_or(0.0),
        }),
        hlod: d.hlod,
        merge: d.merge.unwrap_or(true),
        terrain: d
            .terrain
            .map(|t| (t.terrain.id, t.depth.unwrap_or(0.0).into())),
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
    /// Explicit input image for cook-time map preparation; asset-root relative.
    pub generate: Option<String>,
    pub facade: bool,
    /// How many metres of height the source image's darkest to lightest
    /// span, for the normal map `og materials make` derives (0: none;
    /// below 0, the lighter is lower, as pale mortar between bricks).
    pub relief: f64,
    /// Linear roughness at black and white; optional derived packed map.
    pub roughness_range: Option<[f64; 2]>,
    /// Cavity darkening strength and sampling radius in texels.
    pub occlusion: Option<[f64; 2]>,
    /// Native glTF surface fields; maps name asset ids.
    pub fields: crate::cook::source_schemas::world::materials::Material,
}

/// The keys of a library's material that are not the drawn material's.
pub(crate) const MATERIAL_GENERATE_LINES: &[crate::grammar::Line] = &[crate::grammar::Line::new(
    "source",
    "\"<image>\"",
    "required",
    "Authored input image, relative to assets/. Its bytes and processing parameters form the action key.",
    "",
)];

pub(crate) const MATERIAL_LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "generate",
        "{ source \"<image>\" }",
        "off",
        "Prepare the declared map destinations during cook, without writing generated files into assets/.",
        "",
    ),
    crate::grammar::Line::new(
        "label",
        "<label>",
        "required",
        "Repeat labels this material covers.",
        "",
    ),
    crate::grammar::Line::new(
        "weight",
        "<number > 0>",
        "1",
        "Explicit selection weight within a label.",
        "",
    ),
    crate::grammar::Line::new(
        "tile",
        "<width> <height>",
        "1 1",
        "Positive texture coverage in metres.",
        "",
    ),
    crate::grammar::Line::new(
        "vary",
        "{ ... }",
        "",
        "Per-building value and warmth variation.",
        "",
    ),
    crate::grammar::Line::new(
        "brief",
        "\"<text>\"",
        "",
        "Author's image request; the engine does not generate an image model's response.",
        "",
    ),
    crate::grammar::Line::new(
        "facade",
        "true | false",
        "false",
        "Facade source repeats horizontally; requires brief.",
        "",
    ),
    crate::grammar::Line::new(
        "relief",
        "<-1..1 metres>",
        "0",
        "Height range for the derived normal map; nonzero needs brief and normalMap.",
        "",
    ),
    crate::grammar::Line::new(
        "roughnessRange",
        "<black> <white>",
        "",
        "Roughness endpoints 0..1; requires brief and metalnessMap.",
        "",
    ),
    crate::grammar::Line::new(
        "occlusion",
        "<strength> <radius>",
        "",
        "Strength 0..1, whole radius 1..64 texels; requires brief, relief and aoMap.",
        "",
    ),
];

pub(crate) const VARY_LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "value",
        "<0..0.5>",
        "0",
        "Brightness variation either way.",
        "",
    ),
    crate::grammar::Line::new(
        "warmth",
        "<0..0.5>",
        "0",
        "Warmth variation either way.",
        "",
    ),
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
            known(
                vb,
                &VARY_LINES.iter().map(|f| f.key).collect::<Vec<_>>(),
                "vary",
            )?;
            value = vb.get("value").map(one).transpose()?.unwrap_or(0.0);
            warmth = vb.get("warmth").map(one).transpose()?.unwrap_or(0.0);
            if !(0.0..=0.5).contains(&value) || !(0.0..=0.5).contains(&warmth) {
                return Err(at(v, "value and warmth are 0 to 0.5"));
            }
        }
        let brief = b.get("brief").map(text).transpose()?;
        let generate = b
            .get("generate")
            .map(|e| {
                if b.get_all("generate").count() != 1 || !e.values.is_empty() {
                    return Err(at(e, "declare generate once, as a block"));
                }
                let g = block(e)?;
                for e in &g.entries {
                    if crate::grammar::find(MATERIAL_GENERATE_LINES, &e.key, &[]).is_none() {
                        return Err(at(e, "unknown material generation field"));
                    }
                }
                if g.entries.len() != 1 {
                    return Err(at(e, "generate contains one source"));
                }
                asset(
                    g.get("source")
                        .ok_or_else(|| at(e, "generate needs source"))?,
                )
            })
            .transpose()?;
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
            && generate.is_none()
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
                .filter(|x| crate::grammar::find(MATERIAL_LINES, &x.key, &[]).is_none())
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
            generate,
            facade,
            relief,
            roughness_range,
            occlusion,
            fields: crate::cook::materials::material_input(&fields)?,
        });
    }
    Ok(out)
}
