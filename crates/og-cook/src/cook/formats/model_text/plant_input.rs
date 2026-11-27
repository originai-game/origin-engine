//! Native plant choices preserve the existing growth and foliage kernels.
use super::*;
use crate::cook::{formats::geometry, source_schemas::world::model::plant as input};
use og_shape::generators::plant::{Branch, Flowers, Growth, LeafShape, Leaves, Level, Plant};

pub(super) struct Look {
    pub wood: Option<String>,
    pub bark_color: DVec3,
    pub leaf: Option<String>,
    pub translucency: f64,
    pub dome_normals: f64,
    pub season: og_render_data::foliage::LeafSeason,
}

fn level(d: &input::Level, l: &mut Level) {
    l.length = d.length.unwrap_or(l.length);
    l.radius = d.radius.unwrap_or(l.radius);
    l.sections = d.sections.unwrap_or(l.sections);
    l.segments = d.segments.unwrap_or(l.segments);
    l.taper = d.taper.unwrap_or(l.taper);
    l.gnarliness = d.gnarliness.unwrap_or(l.gnarliness);
    l.twist = d.twist.unwrap_or(l.twist);
    l.children = d.children.unwrap_or(l.children);
    l.angle = d.angle.unwrap_or(l.angle);
    l.start = d.start.unwrap_or(l.start);
}

pub(super) fn plant(
    d: &input::Plant,
    models: &geometry::Models<'_>,
) -> Result<(Plant, Look), String> {
    let (mut p, mut look) = match &d.kind {
        Some(kind) => {
            let text = match kind {
                input::PlantKind::Oak => include_str!("plants/oak.json"),
                input::PlantKind::Birch => include_str!("plants/birch.json"),
                input::PlantKind::Willow => include_str!("plants/willow.json"),
                input::PlantKind::Pine => include_str!("plants/pine.json"),
                input::PlantKind::Palm => include_str!("plants/palm.json"),
                input::PlantKind::Bamboo => include_str!("plants/bamboo.json"),
                input::PlantKind::Cactus => include_str!("plants/cactus.json"),
                input::PlantKind::Shrub => include_str!("plants/shrub.json"),
                input::PlantKind::Fern => include_str!("plants/fern.json"),
                input::PlantKind::Reed => include_str!("plants/reed.json"),
                input::PlantKind::Daisy => include_str!("plants/daisy.json"),
                input::PlantKind::Tulip => include_str!("plants/tulip.json"),
            };
            plant(
                &serde_json::from_str(text).expect("native plant defaults"),
                models,
            )?
        }
        None => (
            Plant {
                leaves: None,
                flowers: None,
                ..Default::default()
            },
            Look {
                wood: None,
                bark_color: DVec3::new(0.13, 0.085, 0.05),
                leaf: None,
                translucency: 0.5,
                dome_normals: 0.6,
                season: og_render_data::foliage::LeafSeason::EVERGREEN,
            },
        ),
    };
    for (d, l) in [
        d.trunk.as_ref(),
        d.branch1.as_ref(),
        d.branch2.as_ref(),
        d.branch3.as_ref(),
    ]
    .into_iter()
    .zip(&mut p.level)
    {
        if let Some(d) = d {
            level(d, l);
        }
    }
    if let Some([x, y, z, strength]) = d.force {
        p.force_direction = DVec3::new(x, y, z);
        p.force_strength = strength;
    }
    if let Some(c) = d.bark_color {
        look.bark_color = DVec3::from_array(c);
    }
    if look.bark_color.min_element() < 0.0 {
        return Err("plant.barkColor must be nonnegative".into());
    }
    look.wood = d.material.clone();
    if let Some(d) = &d.leaves {
        let l = p.leaves.take().unwrap_or_default();
        look.leaf = d.material.clone();
        if look.leaf.is_some()
            && (d.translucency.is_some()
                || d.dome_normals.is_some()
                || d.season.is_some()
                || d.autumn_color.is_some()
                || d.autumn_color2.is_some()
                || d.season_shift.is_some())
        {
            return Err(
                "named leaf materials own translucency, domeNormals and seasonal properties".into(),
            );
        }
        look.translucency = d.translucency.unwrap_or(look.translucency);
        look.dome_normals = d.dome_normals.unwrap_or(look.dome_normals);
        look.season = og_render_data::foliage::LeafSeason {
            season: d.season.unwrap_or(look.season.season),
            autumn_color: d.autumn_color.unwrap_or(look.season.autumn_color),
            autumn_color2: d.autumn_color2.unwrap_or(look.season.autumn_color2),
            shift: d.season_shift.unwrap_or(look.season.shift),
        };
        let start = d.start.unwrap_or(l.start);
        let end = d.end.unwrap_or(l.end);
        if end < start {
            return Err("plant.leaves.end is before start".into());
        }
        p.leaves = Some(Leaves {
            shape: match d.shape {
                Some(input::LeafShape::Card) => LeafShape::Card,
                Some(input::LeafShape::Cross) => LeafShape::Cross,
                _ => LeafShape::Blade,
            },
            count: d.count.unwrap_or(l.count),
            start,
            end,
            angle: d.angle.unwrap_or(l.angle),
            facing: d.facing.unwrap_or(l.facing),
            rows: d.rows.unwrap_or(l.rows),
            size: d.size.unwrap_or(l.size),
            width: d.width.unwrap_or(l.width),
            size_variance: d.size_variance.unwrap_or(l.size_variance),
            color: d.color.unwrap_or(l.color),
            color2: d.color2.or(d.color).unwrap_or(l.color2),
            color_variation: d.color_variation.unwrap_or(l.color_variation),
        });
    }
    if let Some(d) = &d.flowers {
        let l = p.flowers.take().unwrap_or_default();
        p.flowers = Some(Flowers {
            share: d.share.unwrap_or(l.share),
            count: d.count.unwrap_or(l.count),
            petals: d.petals.unwrap_or(l.petals),
            size: d.size.unwrap_or(l.size),
            width: d.width.unwrap_or(l.width),
            cup: d.cup.unwrap_or(l.cup),
            color: d.color.unwrap_or(l.color),
            color2: d.color2.or(d.color).unwrap_or(l.color2),
            center_color: d.center_color.unwrap_or(l.center_color),
        });
    }
    p.seed = d.seed.unwrap_or(p.seed);
    p.levels = d.levels.unwrap_or(p.levels);
    p.growth = match d.growth {
        Some(input::Growth::Evergreen) => Growth::Evergreen,
        Some(input::Growth::Deciduous) => Growth::Deciduous,
        None => p.growth,
    };
    p.branches = d
        .branches
        .iter()
        .map(|b| Branch {
            parent: b.parent.unwrap_or(-1),
            path: b.path.iter().copied().map(DVec3::from_array).collect(),
            radii: b.radii.clone(),
            segments: b.segments.unwrap_or(6),
        })
        .collect();
    p.bounds = d.bounds.map(|b| b.map(DVec3::from_array));
    p.path = d.path.iter().copied().map(DVec3::from_array).collect();
    p.leaf_mesh = d
        .leaf_geometry
        .as_ref()
        .map(|g| geometry::build(g, models))
        .transpose()?;
    p.flower_mesh = d
        .flower_geometry
        .as_ref()
        .map(|g| geometry::build(g, models))
        .transpose()?;
    Ok((p, look))
}
