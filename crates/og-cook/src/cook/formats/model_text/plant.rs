//! A `plant` block of a model text: an og-geom plant generator's levels,
//! leaves and look, or one of the built-in kinds (`plants/<kind>.ogt`).

use super::*;

/// A `plant` block's levels of branches, by their block's name.
const PLANT_LEVELS: [&str; og_geom::plant::LEVELS] = ["trunk", "branch1", "branch2", "branch3"];

/// What a `plant` block draws with, beside its meshes.
pub(super) struct PlantLook {
    /// The wood's material by name, else one of `bark_color`.
    pub(super) wood: Option<(String, usize)>,
    pub(super) bark_color: DVec3,
    /// The leaves' and flowers' material by name, else one that shows
    /// their vertex colours from both sides with `translucency` and
    /// `dome_normals`.
    pub(super) leaf: Option<(String, usize)>,
    pub(super) translucency: f64,
    pub(super) dome_normals: f64,
}

fn unit_field(f: &Fields, k: &str, default: f64) -> Result<f64, String> {
    let v = f.num(k, default)?;
    if !(0.0..=1.0).contains(&v) {
        return Err(format!("{}.{k} is 0 to 1, got {v}", f.what));
    }
    Ok(v)
}

/// A linear colour: three numbers, none negative.
fn color_field(f: &Fields, k: &str, default: [f32; 3]) -> Result<[f32; 3], String> {
    let c = f.vec3(k, DVec3::from(default.map(f64::from)))?;
    if c.min_element() < 0.0 {
        return Err(format!(
            "{}.{k} is a linear colour, no component under 0",
            f.what
        ));
    }
    Ok(c.as_vec3().to_array())
}

fn word_of<'a>(f: &Fields<'a>, k: &str, words: &[&str]) -> Result<Option<&'a str>, String> {
    let Some(e) = f.get(k) else { return Ok(None) };
    match e.values.as_slice() {
        [v] if v.as_word().is_some_and(|w| words.contains(&w)) => Ok(v.as_word()),
        _ => Err(at(e, format!("is one of: {}", words.join(", ")))),
    }
}

/// The kinds a `plant` block starts from (`kind`): each a plant block of
/// its own under `plants/`, read as the block's defaults.
const PLANT_KINDS: [(&str, &str); 12] = [
    ("oak", include_str!("plants/oak.ogt")),
    ("birch", include_str!("plants/birch.ogt")),
    ("willow", include_str!("plants/willow.ogt")),
    ("pine", include_str!("plants/pine.ogt")),
    ("palm", include_str!("plants/palm.ogt")),
    ("bamboo", include_str!("plants/bamboo.ogt")),
    ("cactus", include_str!("plants/cactus.ogt")),
    ("shrub", include_str!("plants/shrub.ogt")),
    ("fern", include_str!("plants/fern.ogt")),
    ("reed", include_str!("plants/reed.ogt")),
    ("daisy", include_str!("plants/daisy.ogt")),
    ("tulip", include_str!("plants/tulip.ogt")),
];

/// A kind's plant and look: its own block, read from the defaults.
fn plant_kind(kind: &str) -> Result<(og_geom::Plant, PlantLook), String> {
    let text = PLANT_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, text)| *text)
        .expect("a kind of PLANT_KINDS");
    let doc = Document::parse(text).map_err(|e| format!("kind {kind}: {}", e.reason()))?;
    let block = doc
        .body
        .get("plant")
        .and_then(|e| e.block.as_ref())
        .ok_or_else(|| format!("kind {kind}: no plant block"))?;
    plant_desc(block).map_err(|e| format!("kind {kind}: {e}"))
}

/// A `plant` block as og-geom's [`og_geom::Plant`] (its fields by their
/// names, a level a block: `trunk`, `branch1` to `branch3`) and its look.
/// What it leaves out is its `kind`'s; without one the defaults', with
/// `leaves` and `flowers` only when their block is there.
pub(super) fn plant_desc(b: &Block) -> Result<(og_geom::Plant, PlantLook), String> {
    use og_geom::plant::{Flowers, Growth, LeafShape, Leaves, Level, Plant};
    let mut keys = vec![
        "kind",
        "seed",
        "growth",
        "levels",
        "force",
        "leaves",
        "flowers",
        "material",
        "barkColor",
    ];
    keys.extend(PLANT_LEVELS);
    let f = Fields::new(b, "plant", &keys)?;
    let kinds: Vec<&str> = PLANT_KINDS.iter().map(|(k, _)| *k).collect();
    let (d, mut look) = match word_of(&f, "kind", &kinds)? {
        Some(kind) => plant_kind(kind)?,
        None => (
            Plant {
                leaves: None,
                flowers: None,
                ..Plant::default()
            },
            PlantLook {
                wood: None,
                bark_color: DVec3::new(0.13, 0.085, 0.05),
                leaf: None,
                translucency: 0.5,
                dome_normals: 0.6,
            },
        ),
    };
    let mut level = d.level;
    for (k, l) in PLANT_LEVELS.iter().zip(&mut level) {
        let Some(lb) = f.block(k)? else { continue };
        let what = format!("plant.{k}");
        let mut keys = vec![
            "length",
            "radius",
            "sections",
            "segments",
            "taper",
            "gnarliness",
            "twist",
            "children",
        ];
        if *k != "trunk" {
            keys.extend(["angle", "start"]);
        }
        let lf = Fields::new(lb, &what, &keys)?;
        *l = Level {
            length: lf.size("length", l.length)?,
            radius: lf.size("radius", l.radius)?,
            sections: lf.uint("sections", l.sections, 1, 64)?,
            segments: lf.uint("segments", l.segments, 3, 32)?,
            taper: unit_field(&lf, "taper", l.taper)?,
            gnarliness: lf.num("gnarliness", l.gnarliness)?,
            twist: lf.num("twist", l.twist)?,
            children: lf.uint("children", l.children, 0, 400)?,
            angle: lf.num("angle", l.angle)?,
            start: unit_field(&lf, "start", l.start)?,
        };
    }
    let (force_direction, force_strength) = match f.get("force") {
        Some(e) => {
            let v = exactly(e, 4)?;
            (DVec3::new(v[0], v[1], v[2]), v[3])
        }
        None => (d.force_direction, d.force_strength),
    };
    look.bark_color = f.vec3("barkColor", look.bark_color)?;
    if look.bark_color.min_element() < 0.0 {
        return Err("plant.barkColor is a linear colour, no component under 0".into());
    }
    let named = |e: &Entry| -> Result<(String, usize), String> {
        Ok((e.string(0).map_err(|x| at(e, x))?.to_string(), e.line))
    };
    look.wood = f.get("material").map(named).transpose()?;
    let leaves = match f.block("leaves")? {
        Some(lb) => {
            let lf = Fields::new(
                lb,
                "plant.leaves",
                &[
                    "shape",
                    "count",
                    "start",
                    "end",
                    "angle",
                    "facing",
                    "rows",
                    "size",
                    "width",
                    "sizeVariance",
                    "color",
                    "color2",
                    "colorVariation",
                    "material",
                    "translucency",
                    "domeNormals",
                ],
            )?;
            let l = d.leaves.unwrap_or_default();
            look.leaf = lf.get("material").map(named).transpose()?;
            for k in ["translucency", "domeNormals"] {
                if let (Some(_), Some(e)) = (&look.leaf, lf.get(k)) {
                    return Err(at(
                        e,
                        "is its material's when the leaves name one (the same key in the material block)",
                    ));
                }
            }
            look.translucency = unit_field(&lf, "translucency", look.translucency)?;
            look.dome_normals = unit_field(&lf, "domeNormals", look.dome_normals)?;
            let color = color_field(&lf, "color", l.color)?;
            let start = unit_field(&lf, "start", l.start)?;
            let end = unit_field(&lf, "end", l.end)?;
            if let (true, Some(e)) = (end < start, lf.get("end")) {
                return Err(at(e, "is before start: the leaves grow from start to end"));
            }
            Some(Leaves {
                shape: match word_of(&lf, "shape", &["blade", "card", "cross"])? {
                    Some("card") => LeafShape::Card,
                    Some("cross") => LeafShape::Cross,
                    _ => LeafShape::Blade,
                },
                count: lf.uint("count", l.count, 0, 2000)?,
                start,
                end,
                angle: lf.num("angle", l.angle)?,
                facing: unit_field(&lf, "facing", l.facing)?,
                rows: lf.uint("rows", l.rows, 0, 8)?,
                size: lf.size("size", l.size)?,
                width: lf.size("width", l.width)?,
                size_variance: unit_field(&lf, "sizeVariance", l.size_variance)?,
                color,
                // One colour named: every leaf that colour.
                color2: color_field(
                    &lf,
                    "color2",
                    if lf.get("color").is_some() {
                        color
                    } else {
                        l.color2
                    },
                )?,
                color_variation: unit_field(&lf, "colorVariation", f64::from(l.color_variation))?
                    as f32,
            })
        }
        None => d.leaves,
    };
    let flowers = match f.block("flowers")? {
        Some(fb) => {
            let ff = Fields::new(
                fb,
                "plant.flowers",
                &[
                    "share",
                    "count",
                    "petals",
                    "size",
                    "width",
                    "cup",
                    "color",
                    "color2",
                    "centerColor",
                ],
            )?;
            let l = d.flowers.unwrap_or_default();
            let color = color_field(&ff, "color", l.color)?;
            Some(Flowers {
                share: unit_field(&ff, "share", l.share)?,
                count: ff.uint("count", l.count, 0, 200)?,
                petals: ff.uint("petals", l.petals, 1, 32)?,
                size: ff.size("size", l.size)?,
                width: ff.size("width", l.width)?,
                cup: ff.num("cup", l.cup)?,
                color,
                // One colour named: every petal that colour.
                color2: color_field(
                    &ff,
                    "color2",
                    if ff.get("color").is_some() {
                        color
                    } else {
                        l.color2
                    },
                )?,
                center_color: color_field(&ff, "centerColor", l.center_color)?,
            })
        }
        None => d.flowers,
    };
    let plant = Plant {
        seed: f.uint("seed", d.seed, 0, u32::MAX)?,
        growth: match word_of(&f, "growth", &["deciduous", "evergreen"])? {
            Some("evergreen") => Growth::Evergreen,
            Some(_) => Growth::Deciduous,
            None => d.growth,
        },
        levels: f.uint("levels", d.levels, 0, og_geom::plant::LEVELS as u32 - 1)?,
        level,
        force_direction,
        force_strength,
        leaves,
        flowers,
    };
    Ok((plant, look))
}
