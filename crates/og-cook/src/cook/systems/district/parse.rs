//! Native district declarations, normalized directly from checked author data.
use crate::cook::source_schemas::world::district as input;
use glam::DVec2;
use og_shape::generators::layout::{GridDesc, LotsDesc, PropDesc, Region, StreetDesc};
use std::collections::BTreeSet;

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
    /// whether that is a facade module (`facade: true`) rather than a
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
