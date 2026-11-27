//! Buildings from split rules: a shape grammar in the manner of CGA shape
//! (Müller, Wonka, Haegler, Ulmer and Van Gool, *Procedural Modeling of
//! Buildings*, SIGGRAPH 2006) over the split grammars of Wonka, Wimmer,
//! Sillion and Ribarsky (*Instant Architecture*, 2003). Written here from
//! the papers; no code is taken from an implementation.
//!
//! A [`Grammar`] is a set of named rules. A rule takes one planar shape, a
//! [`face`] with its own frame, and either ends it (a `label`, an
//! `insert`, a `drop`) or hands its parts to other rules: `extrude` makes
//! a prism and names its faces, `split` cuts a face into strips along one
//! of its axes, `inset` cuts a border off it, `roof` builds a roof on it,
//! `pick` and `case` choose a rule. `recess` and `detail` change the face
//! before the rule's last operation. The first shape is a lot's footprint,
//! a horizontal face facing up.
//!
//! What it makes ([`Built`]):
//!
//! - one mesh per label and detail level, every triangle facing out of
//!   its face (counter-clockwise from the front), with flat normals;
//! - texture coordinates in metres, continuous across every split: a wall
//!   reads `u` along the building's perimeter, counter-clockwise seen from
//!   above from the corner where the walls start (the seam), and `v` as the height above the footprint, so a pattern runs
//!   on through window jambs and round corners. A horizontal face reads
//!   the world's x and z. `v` is written negated, as og-render's uvs run
//!   down the image: a texture's top is up the wall;
//! - [`Insert`]s, the scopes a model is to be fitted into, each by its
//!   asset id, for the caller to draw instanced.
//!
//! Every choice draws from a stream seeded by the shape's own id (the
//! lot's seed and the path of splits that reached it), so the same lot and
//! seed give the same bytes on every host, and changing one rule changes
//! only the shapes it reaches. f64 throughout, basic arithmetic, `tan`
//! through `og_core::math::double`.

mod eval;
mod face;
mod roof;
mod split;
mod validate;

pub use eval::build;
pub use validate::validate;

use std::collections::BTreeMap;

use glam::{DVec2, DVec3};

use crate::generators::Mesh;

/// The most shapes one lot may make before it is refused.
pub const MAX_SHAPES: usize = 1_000_000;

/// The most rules a chain may pass through before it is refused (a rule
/// that reaches itself without making its shape smaller never ends).
pub const MAX_DEPTH: usize = 256;

/// A number in a rule: written, or a parameter's name.
#[derive(Clone, Debug, PartialEq)]
pub enum Num {
    Lit(f64),
    Param(String),
}

/// A split's direction, along the face's own axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Along the face (a wall: along the street).
    X,
    /// Up the face (a wall: up the building).
    Y,
}

/// How a split part takes its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// Exactly its size (`part`).
    Abs,
    /// Its size, stretched with the others to fill what the absolute parts
    /// leave (`fill`, CGA's `~`).
    Fill,
    /// As many copies of its size as fit, stretched to fill (`repeat`,
    /// CGA's `*`): at least one.
    Repeat,
}

/// One part of a split.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitPart {
    pub size: Size,
    pub length: Num,
    pub rule: String,
}

/// Which roof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoofKind {
    /// The face itself.
    Flat,
    /// Two slopes along the face's longer side and a gable at each end; a
    /// face that is not a rectangle takes a hip roof.
    Gable,
    /// A slope rising from every edge (the straight skeleton of a convex
    /// face).
    Hip,
}

/// What `extrude` names: the rule each face of the prism takes. A face
/// named by none is not made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Faces {
    /// The face moved out by the depth.
    pub top: Option<String>,
    /// The face itself, turned to face back.
    pub bottom: Option<String>,
    /// The side on the face's first edge (a lot's: its street edge).
    pub front: Option<String>,
    /// The sides facing away from the front.
    pub back: Option<String>,
    /// The other sides.
    pub side: Option<String>,
    /// Every side the three above do not name.
    pub sides: Option<String>,
}

/// A condition of `case`: the first that holds picks its rule.
#[derive(Clone, Debug, PartialEq)]
pub enum Cond {
    /// The face is at least this wide.
    Wider(Num),
    /// The face is at least this tall.
    Taller(Num),
    /// The parameter is at least this.
    Param(String, Num),
}

/// One operation of a rule.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    /// The face is pushed in along its normal; a jamb on each edge joins
    /// it to where it was, each given to the `jamb` rule, or labelled
    /// `jamb` without one.
    Recess { depth: Num, jamb: Option<String> },
    /// What this rule and every rule it reaches makes is left out of a
    /// model's levels of detail coarser than this (0: always drawn).
    Detail(u32),
    /// The face is made, with this label.
    Label(String),
    /// A prism of the face.
    Extrude { depth: Num, faces: Faces },
    /// Strips of the face.
    Split { axis: Axis, parts: Vec<SplitPart> },
    /// A border of this width and the face inside it.
    Inset {
        width: Num,
        border: String,
        inner: String,
    },
    /// A roof on the face.
    Roof {
        kind: RoofKind,
        /// Degrees from level.
        angle: Num,
        /// How far the eaves stand out past the face's edges.
        overhang: Num,
        slope: String,
        gable: Option<String>,
        /// The slopes' undersides where they overhang (default: `slope`).
        eave: Option<String>,
    },
    /// A model fitted to the face.
    Insert(String),
    /// One rule by weight.
    Pick(Vec<(f64, String)>),
    /// The first rule whose condition holds, else `otherwise`.
    Case {
        arms: Vec<(Cond, String)>,
        otherwise: Option<String>,
    },
    /// Nothing is made.
    Drop,
}

impl Op {
    /// Whether it ends a rule (every rule ends in exactly one).
    fn ends(&self) -> bool {
        !matches!(self, Op::Recess { .. } | Op::Detail(_))
    }
}

/// A named rule: its operations in order, the last one ending it.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub name: String,
    /// Its line in the source (0 when built in memory).
    pub line: usize,
    pub ops: Vec<Op>,
}

/// A grammar: its parameters' defaults, its rules and the rule a lot
/// starts at.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grammar {
    pub params: BTreeMap<String, f64>,
    pub rules: BTreeMap<String, Rule>,
    pub start: String,
}

/// What a grammar builds on: a footprint on level ground.
#[derive(Clone, Debug, PartialEq)]
pub struct Lot {
    /// Its corners in x and z (metres), either way round; the edge between
    /// the first corner and the second is its front. Its walls are laid
    /// counter-clockwise seen from above from the front's left end (seen
    /// from outside), where their texture's `u` starts.
    pub footprint: Vec<DVec2>,
    /// The height (y) it stands at.
    pub base: f64,
    /// Its seed: every choice in it draws from streams on this.
    pub seed: u64,
}

/// A scope a model is fitted into: its corner, its axes (right, up and
/// out of the face) and its width and height along the first two.
#[derive(Clone, Debug, PartialEq)]
pub struct Insert {
    pub asset: String,
    pub origin: DVec3,
    pub x: DVec3,
    pub y: DVec3,
    pub z: DVec3,
    pub size: DVec2,
    pub detail: u32,
}

/// One label's triangles at one detail level.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub label: String,
    pub detail: u32,
    pub mesh: Mesh,
}

/// What a lot built: its parts by label then detail, and its inserts in
/// the order they were made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Built {
    pub parts: Vec<Part>,
    pub inserts: Vec<Insert>,
    /// The shapes the rules were applied to.
    pub shapes: usize,
}

impl Built {
    pub fn triangles(&self) -> usize {
        self.parts.iter().map(|p| p.mesh.triangle_count()).sum()
    }
}
