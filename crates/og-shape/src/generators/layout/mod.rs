//! A town's ground plan from rules and a seed: streets, the blocks between
//! them, the lots along each block's street edges and the places on the
//! pavement where props stand. What stands on a lot is a grammar's
//! (`generators::grammar`); this module decides only where.
//!
//! - **Blocks.** A region is cut by a grid of streets (Parish and Müller,
//!   *Procedural Modeling of Cities*, 2001, its grid pattern), turned and
//!   jittered line by line, and by named streets; or a path is lined on
//!   both sides by strips mitred at its bends. A block is the ground
//!   between street corridors (half the street's width plus its pavement
//!   from its centre line), each edge knowing the street it fronts.
//! - **Lots.** A block (cut into convex parts first) is given a depth
//!   drawn for it; each street edge at least a lot wide takes the ground
//!   within that depth of its line that is nearer its line than any other
//!   front's (so two strips meet on the bisector at a corner), and the
//!   strip is cut across into lots of widths drawn between the least and
//!   the greatest; what lies behind the ring is the block's yard. Every lot fronts a street by construction, the access rule of
//!   Vanegas, Kelly, Weber, Halatsch, Aliaga and Müller, *Procedural
//!   Generation of Parcels in Urban Modeling* (Eurographics 2012), whose
//!   offset subdivision this follows.
//! - **Sites.** Each prop stands along the pavement of the streets it is
//!   given, its own distance from the kerb, at its own spacing, and at the
//!   corners where two streets meet.
//!
//! Coordinates are the world's x and z, in metres. Every polygon is
//! counter-clockwise in those axes (x right, z up the page). A lot's
//! footprint starts with its front: the edge from its first corner to its
//! second lies on the street.
//!
//! Every draw comes from a stream keyed by the seed, what it decides and
//! where (a grid line by its index, a block by its front, a lot by its
//! front's middle), so changing one rule moves only what it decides: a
//! prop's spacing never moves a lot, and a street added at one end of a
//! town leaves the lots at the other end as they were. f64 throughout,
//! basic arithmetic, angles through `og_core::math::double`.

mod blocks;
mod geom;
mod lots;
mod props;

pub use geom::Poly;

use glam::DVec2;
use og_core::hash::{hash_bytes, hash_words, splitmix64, splitmix64_next};

/// The most grid cells a region may span.
pub const MAX_CELLS: i64 = 1_000_000;

/// A street: its centre line, how wide its roadway is and the pavement on
/// each side of it.
#[derive(Clone, Debug, PartialEq)]
pub struct StreetDesc {
    /// What props and rules name it by (a grid's streets have none).
    pub name: String,
    pub points: Vec<DVec2>,
    pub width: f64,
    pub sidewalk: f64,
}

/// A grid of streets over a region.
#[derive(Clone, Debug, PartialEq)]
pub struct GridDesc {
    /// The distance between street centre lines along the grid's x and z.
    pub size: DVec2,
    /// The grid's turn, degrees counter-clockwise from the world's x.
    pub turn: f64,
    /// How far each street line moves from its place, as a share of the
    /// spacing, 0 to 0.45.
    pub jitter: f64,
    pub width: f64,
    pub sidewalk: f64,
}

/// Where the town is.
#[derive(Clone, Debug, PartialEq)]
pub enum Region {
    /// An area (its outline), cut by the grid and the named streets.
    Area(Vec<DVec2>),
    /// Both sides of one street, `depth` metres deep from its pavement.
    Along { street: StreetDesc, depth: f64 },
}

/// How lots are cut: each one's width along its street and the depth of
/// the ring of lots round a block, least and greatest.
#[derive(Clone, Debug, PartialEq)]
pub struct LotsDesc {
    pub width: [f64; 2],
    pub depth: [f64; 2],
}

/// Where one kind of prop stands.
#[derive(Clone, Debug, PartialEq)]
pub struct PropDesc {
    pub name: String,
    /// The spacing along a pavement (0: none along it).
    pub every: f64,
    /// How far each one may move along the pavement from its place.
    pub jitter: f64,
    /// Its distance from the kerb (the roadway's edge), at most the
    /// pavement's width.
    pub kerb: f64,
    /// One more at each corner where two streets meet.
    pub corners: bool,
    /// Only along the streets of this name (`None`: every street).
    pub street: Option<String>,
}

/// A whole ground plan's rules.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutDesc {
    pub seed: u64,
    pub region: Region,
    pub grid: Option<GridDesc>,
    /// Named streets; they cut the blocks the region and grid make.
    pub streets: Vec<StreetDesc>,
    pub lots: LotsDesc,
    pub props: Vec<PropDesc>,
}

/// A street of the plan: the named ones first, in their order, then the
/// path a region runs along, then the grid's.
#[derive(Clone, Debug, PartialEq)]
pub struct Street {
    pub name: String,
    pub points: Vec<DVec2>,
    pub width: f64,
    pub sidewalk: f64,
}

/// A block: its ground, convex and counter-clockwise (each edge naming the
/// street it fronts), and its pavement, the ground out to the roadways'
/// edges.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub poly: Poly,
    pub pavement: Vec<DVec2>,
}

/// A lot.
#[derive(Clone, Debug, PartialEq)]
pub struct Lot {
    pub block: usize,
    /// The street its front faces.
    pub street: usize,
    /// Its corners, counter-clockwise, its front edge first.
    pub footprint: Vec<DVec2>,
    /// Its front edge's length.
    pub frontage: f64,
    /// The depth of its block's ring of lots.
    pub depth: f64,
    /// What a grammar draws its choices from.
    pub seed: u64,
}

/// Where a prop stands.
#[derive(Clone, Debug, PartialEq)]
pub struct Site {
    /// Its prop, by its place in [`LayoutDesc::props`].
    pub prop: usize,
    /// The block whose pavement it stands on.
    pub block: usize,
    pub street: usize,
    pub position: DVec2,
    /// Its front's direction: towards the roadway.
    pub facing: DVec2,
    pub corner: bool,
}

/// A ground plan, with the facts worth a note (a block a street could not
/// cut, a block too shallow for its lots).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub streets: Vec<Street>,
    pub blocks: Vec<Block>,
    pub lots: Vec<Lot>,
    pub sites: Vec<Site>,
    pub notes: Vec<String>,
}

/// The stream keyed by the seed, what it decides and where.
pub(super) fn key(seed: u64, what: &str, at: DVec2) -> u64 {
    let q = |x: f64| (x * 1000.0).round() as i64 as u64;
    splitmix64(hash_words([
        seed,
        hash_bytes(what.as_bytes()),
        q(at.x),
        q(at.y),
    ]))
}

/// Draws from a key.
pub(super) struct Stream(u64);

impl Stream {
    pub fn new(key: u64) -> Stream {
        Stream(key)
    }

    /// In [0, 1).
    pub fn unit(&mut self) -> f64 {
        (splitmix64_next(&mut self.0) >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }
}

fn check(ok: bool, what: impl FnOnce() -> String) -> Result<(), String> {
    if ok { Ok(()) } else { Err(what()) }
}

fn finite(points: &[DVec2]) -> bool {
    points.iter().all(|p| p.is_finite())
}

fn street(s: &StreetDesc) -> Result<Street, String> {
    let named = if s.name.is_empty() {
        "a street".to_string()
    } else {
        format!("street `{}`", s.name)
    };
    check(s.points.len() >= 2 && finite(&s.points), || {
        format!("{named} has two points or more")
    })?;
    check(s.width >= 0.0 && s.width.is_finite(), || {
        format!("{named}'s width is 0 or more")
    })?;
    check(s.sidewalk >= 0.0 && s.sidewalk.is_finite(), || {
        format!("{named}'s sidewalk is 0 or more")
    })?;
    check(
        s.points.windows(2).all(|w| w[0].distance(w[1]) > geom::EPS),
        || format!("{named} has two points at one place"),
    )?;
    Ok(Street {
        name: s.name.clone(),
        points: s.points.clone(),
        width: s.width,
        sidewalk: s.sidewalk,
    })
}

fn validate(d: &LayoutDesc) -> Result<(), String> {
    let range = |r: [f64; 2], what: &str| {
        check(r[0] > 0.0 && r[0] <= r[1] && r[1].is_finite(), || {
            format!(
                "lots' {what} is a least and a greatest above 0, the least first, not {} {}",
                r[0], r[1]
            )
        })
    };
    range(d.lots.width, "width")?;
    range(d.lots.depth, "depth")?;
    if let Some(g) = &d.grid {
        check(
            g.size.x > 0.0 && g.size.y > 0.0 && g.size.is_finite(),
            || "a grid's size is above 0".into(),
        )?;
        check(g.turn.is_finite(), || "a grid's turn is a number".into())?;
        check((0.0..=0.45).contains(&g.jitter), || {
            format!("a grid's jitter is 0 to 0.45, not {}", g.jitter)
        })?;
        check(
            g.width >= 0.0 && g.sidewalk >= 0.0 && (g.width + g.sidewalk).is_finite(),
            || "a grid's width and sidewalk are 0 or more".into(),
        )?;
    }
    for p in &d.props {
        check(p.every >= 0.0 && p.jitter >= 0.0 && p.kerb >= 0.0, || {
            format!("prop `{}`: every, jitter and kerb are 0 or more", p.name)
        })?;
        check((p.every + p.jitter + p.kerb).is_finite(), || {
            format!("prop `{}`: a number is not finite", p.name)
        })?;
    }
    Ok(())
}

/// The ground plan `desc` lays out.
pub fn layout(desc: &LayoutDesc) -> Result<Layout, String> {
    validate(desc)?;
    let mut streets = desc
        .streets
        .iter()
        .map(street)
        .collect::<Result<Vec<_>, _>>()?;
    let named = streets.len();
    let mut notes = Vec::new();
    let mut polys = match &desc.region {
        Region::Area(points) => {
            check(finite(points), || {
                "the region's points are not finite".into()
            })?;
            let region = Poly::new(points.clone(), None).ccw().tidy();
            check(region.len() >= 3 && region.area() > geom::EPS, || {
                "the region has no area".into()
            })?;
            match &desc.grid {
                Some(g) => blocks::grid(&region, g, desc.seed, &mut streets)?,
                None => vec![region],
            }
        }
        Region::Along { street: s, depth } => {
            check(desc.grid.is_none(), || {
                "a region along a street takes no grid".into()
            })?;
            check(*depth > 0.0 && depth.is_finite(), || {
                "a region along a street is deeper than 0".into()
            })?;
            let id = streets.len();
            streets.push(street(s)?);
            blocks::along(&streets[id], *depth, id)
        }
    };
    for (id, s) in streets.iter().enumerate().take(named) {
        polys = blocks::cut(polys, s, id, &mut notes);
    }
    let blocks: Vec<Block> = polys
        .into_iter()
        .flat_map(geom::convex_parts)
        .map(|poly| {
            let out: Vec<f64> = poly
                .streets
                .iter()
                .map(|s| s.map_or(0.0, |i| streets[i].sidewalk))
                .collect();
            let pavement =
                geom::grown(&poly.points, &out).map_or_else(|| poly.points.clone(), |p| p.points);
            Block { poly, pavement }
        })
        .collect();
    let mut lots = Vec::new();
    for (b, block) in blocks.iter().enumerate() {
        notes.extend(lots::lots(b, &block.poly, &desc.lots, desc.seed, &mut lots));
    }
    let sites = props::sites(&blocks, &streets, &desc.props, desc.seed)?;
    Ok(Layout {
        streets,
        blocks,
        lots,
        sites,
        notes,
    })
}
