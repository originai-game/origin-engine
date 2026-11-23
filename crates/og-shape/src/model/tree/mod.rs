//! The model: a tree of named parts and groups in the world, the joints
//! between them, their materials and pose, built into an export scene.

use std::collections::{BTreeMap, BTreeSet};

use crate::bevel::{Chamfer, Fillet, Soften};
use crate::boolean::{BoolOp, BoolReport, Boolean};
use crate::edit::{Dent, EditReport, Hole, Keep, Pocket, Shell, Subdivide};
use crate::math::{DVec3, Frame};
use crate::mesh::{Measures, Solid};
use crate::tools::{
    Image, Interfering, JointKind, Material, Mimic, Part, Placement, Preview, Scene,
};

use crate::model::error::ModelError;
use crate::model::place::{self, Path};

mod build;
mod combine;
mod joints;
mod materials;
mod nodes;
mod skin;
pub use build::*;
pub use combine::*;
pub use joints::*;

/// A node of a [`Model`]: a part or a group. Ids are given in order of
/// creation and never reused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub u32);

/// The length unit of a program. Tolerances stay in millimetres whatever
/// the unit; export writes metres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Unit {
    /// Millimetres (the default).
    #[default]
    Millimetre,
    /// Centimetres.
    Centimetre,
    /// Metres.
    Metre,
}

impl Unit {
    /// Millimetres per unit.
    pub fn millimetres(self) -> f64 {
        match self {
            Unit::Millimetre => 1.0,
            Unit::Centimetre => 10.0,
            Unit::Metre => 1000.0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub(crate) name: String,
    /// The solid in the part's own space; `None` for a group.
    pub(crate) solid: Option<Solid>,
    /// Own space to world, at rest.
    pub(crate) placement: Frame,
    /// The group it is in.
    pub(crate) parent: Option<Id>,
    /// Its own material, if one was set on it, with the size (in the
    /// model's unit) its map spans.
    pub(crate) material: Option<(Material, f64)>,
    /// Named frames an assembly publishes, in the world.
    pub(crate) anchors: BTreeMap<String, Frame>,
    pub(crate) used_up: bool,
    /// Removed by [`Model::delete`] (also `used_up`).
    pub(crate) deleted: bool,
}

/// A model: named parts (solids with a placement and a material), groups
/// and assemblies above them, and joints between them (`docs/design.md`
/// §4).
///
/// - Every placement, frame, pivot and axis the API takes or gives is in
///   the world at rest (every joint at 0), in the model's unit.
/// - A part's solid stays in its own space; [`Model::at`] and
///   [`Model::transform`] move the part (and everything under it, with
///   the pivots of joints it carries) without touching the solid.
/// - Edits take their frames in the world and run in the part's space.
///   A boolean uses its tool up: the tool's id answers
///   [`ModelError::UsedUp`] after.
/// - Joints hang a child (a part or a group) on a parent part. Several on
///   one child compose in declaration order, the first nearest the parent.
///   [`Model::pose`] sets a value. Frames are read at rest only: a read
///   of a part a non-zero joint value moves ([`Model::bbox`],
///   [`Model::face`], [`Model::edge`], [`Model::snap`],
///   [`Model::placement`], [`Model::local`], [`Model::anchor`]) is
///   refused with [`ModelError::Posed`]. Export, preview and
///   [`Model::measure_posed`] show the pose.
#[derive(Clone, Debug, Default)]
pub struct Model {
    pub(crate) name: String,
    pub(crate) unit: Unit,
    pub(crate) nodes: Vec<Node>,
    pub(crate) joints: Vec<JointRec>,
    pub(crate) booleans: Vec<BoolRecord>,
    pub(crate) uvs: bool,
    pub(crate) clips: Vec<Clip>,
}

pub(crate) fn finite_frame(f: &Frame, what: &str) -> Result<(), ModelError> {
    if f.is_rigid(1e-9) {
        Ok(())
    } else {
        Err(ModelError::BadValue(format!(
            "{what} (a frame must be finite, orthonormal and right-handed)"
        )))
    }
}

/// The rigid motion taking `from` onto `to`: `to ∘ from⁻¹`.
pub(crate) fn motion(from: &Frame, to: &Frame) -> Frame {
    to.compose(&from.inverse())
}

/// A solid mapped by a placement (rigid: no mirror).
pub(crate) fn placed(mut s: Solid, f: &Frame) -> Solid {
    s.map_positions(|p| f.point_to_world(p), false);
    if let Some(skin) = &mut s.skin {
        for bone in &mut skin.bones {
            bone.rest = f.compose(&bone.rest);
            bone.inverse_bind = crate::mesh::skin::frame_inverse_matrix(&bone.rest);
        }
    }
    rounded(s)
}

/// A solid whose positions were just rounded by a motion or a scale, made
/// canonical and kept from crossing itself (`docs/research.md` §26).
///
/// - Rounding can put a needle a rounding error wide onto one line
///   (§15.4): such slivers are flipped away, exactly.
/// - It can turn a needle between two points a rounding error apart over
///   its neighbours (§25.3, §26): where the solid has an edge shorter
///   than the boolean's snap distance or a corner that close to the
///   opposite edge, and only then, its crossing pairs are counted, and
///   any are repaired by the boolean's repair (`uncross_output`, §23.2),
///   every point free to move. A solid it cannot improve is returned as
///   it is.
pub(crate) fn rounded(mut s: Solid) -> Solid {
    let _ = s.canonicalize();
    if crate::boolean::drop_slivers(&mut s) {
        let _ = s.canonicalize();
    }
    if thin(&s) && crate::boolean::crossing_pairs(&s) > 0 {
        s = uncrossed(s);
    }
    s
}

impl Model {
    /// An empty model named `name` (the scene's name on export), in
    /// millimetres.
    pub fn new(name: &str) -> Model {
        Model {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// The model's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The program's length unit, set before the first part.
    pub fn units(&mut self, unit: Unit) -> Result<(), ModelError> {
        if !self.nodes.is_empty() && unit != self.unit {
            return Err(ModelError::Units(
                "set the unit before the first part".into(),
            ));
        }
        self.unit = unit;
        Ok(())
    }

    /// The unit in force.
    pub fn unit(&self) -> Unit {
        self.unit
    }

    // ── nodes ────────────────────────────────────────────────────────────────

    // ── placement ────────────────────────────────────────────────────────────

    // ── copies ───────────────────────────────────────────────────────────────

    // ── materials ────────────────────────────────────────────────────────────

    // ── edits ────────────────────────────────────────────────────────────────

    // ── joints ───────────────────────────────────────────────────────────────

    // ── building ─────────────────────────────────────────────────────────────
}
