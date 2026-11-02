//! The templates: each one's parts, which part each hangs from, where its
//! joint is when it overlaps nothing, and how it moves.

/// Where a part's joint is by convention, and the end the overlap with
/// its parent is looked for in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum End {
    /// The top of the part (a shoulder at the top of the upper arm).
    Top,
    /// The bottom (a neck at the bottom of the head).
    Bottom,
    /// Its middle (the pelvis, the body).
    Centre,
    /// Where it meets its parent: the overlap anywhere, else the end
    /// nearest the parent (a tail, a weapon's grip).
    Near,
}

/// How a part moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// One bone, drawn as it is (a hand, a head).
    Rigid,
    /// One bone that bends at its joints (an arm, a leg, the torso).
    Limb,
    /// A chain of this many bones (a cape, a tail), lagging in motion.
    Chain(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub name: &'static str,
    /// The part it hangs from: the first of these the art has (none: the root).
    pub parents: &'static [&'static str],
    /// The parts whose joint is its tip, the first the art has (none: its far end).
    pub tip: &'static [&'static str],
    pub required: bool,
    pub end: End,
    pub shape: Shape,
    /// What the part is, for a refusal that asks for it.
    pub what: &'static str,
}

#[derive(Debug)]
pub struct Template {
    pub name: &'static str,
    pub parts: &'static [Spec],
    /// Two-bone IK chains: (upper, lower, the part at the end).
    pub legs: &'static [[&'static str; 3]],
}

impl Template {
    pub fn spec(&self, name: &str) -> Option<&Spec> {
        self.parts.iter().find(|p| p.name == name)
    }
}

const fn spec(
    name: &'static str,
    parents: &'static [&'static str],
    tip: &'static [&'static str],
    required: bool,
    end: End,
    shape: Shape,
    what: &'static str,
) -> Spec {
    Spec {
        name,
        parents,
        tip,
        required,
        end,
        shape,
        what,
    }
}

use End::*;
use Shape::*;

/// A humanoid seen from the side, facing right: `-l` is the far side
/// (drawn behind), `-r` the near side.
pub const HUMANOID_SIDE: Template = Template {
    name: "humanoid-side",
    parts: &[
        spec("pelvis", &[], &["torso"], true, Centre, Rigid, "the hips"),
        spec(
            "torso",
            &["pelvis"],
            &["neck", "head"],
            true,
            Bottom,
            Limb,
            "the chest and belly",
        ),
        spec(
            "neck",
            &["torso"],
            &["head"],
            false,
            Bottom,
            Limb,
            "the neck",
        ),
        spec(
            "head",
            &["neck", "torso"],
            &[],
            true,
            Bottom,
            Rigid,
            "the head",
        ),
        spec(
            "hair",
            &["head"],
            &[],
            false,
            Top,
            Chain(2),
            "hair that swings",
        ),
        spec("ear-l", &["head"], &[], false, Bottom, Rigid, "the far ear"),
        spec(
            "ear-r",
            &["head"],
            &[],
            false,
            Bottom,
            Rigid,
            "the near ear",
        ),
        spec(
            "arm-upper-l",
            &["torso"],
            &["arm-lower-l"],
            true,
            Top,
            Limb,
            "the far upper arm",
        ),
        spec(
            "arm-lower-l",
            &["arm-upper-l"],
            &["hand-l"],
            true,
            Top,
            Limb,
            "the far forearm",
        ),
        spec(
            "hand-l",
            &["arm-lower-l"],
            &[],
            true,
            Top,
            Rigid,
            "the far hand",
        ),
        spec(
            "arm-upper-r",
            &["torso"],
            &["arm-lower-r"],
            true,
            Top,
            Limb,
            "the near upper arm",
        ),
        spec(
            "arm-lower-r",
            &["arm-upper-r"],
            &["hand-r"],
            true,
            Top,
            Limb,
            "the near forearm",
        ),
        spec(
            "hand-r",
            &["arm-lower-r"],
            &[],
            true,
            Top,
            Rigid,
            "the near hand",
        ),
        spec(
            "leg-upper-l",
            &["pelvis"],
            &["leg-lower-l"],
            true,
            Top,
            Limb,
            "the far thigh",
        ),
        spec(
            "leg-lower-l",
            &["leg-upper-l"],
            &["foot-l"],
            true,
            Top,
            Limb,
            "the far shin",
        ),
        spec(
            "foot-l",
            &["leg-lower-l"],
            &[],
            true,
            Top,
            Rigid,
            "the far foot",
        ),
        spec(
            "leg-upper-r",
            &["pelvis"],
            &["leg-lower-r"],
            true,
            Top,
            Limb,
            "the near thigh",
        ),
        spec(
            "leg-lower-r",
            &["leg-upper-r"],
            &["foot-r"],
            true,
            Top,
            Limb,
            "the near shin",
        ),
        spec(
            "foot-r",
            &["leg-lower-r"],
            &[],
            true,
            Top,
            Rigid,
            "the near foot",
        ),
        spec(
            "cape",
            &["torso"],
            &[],
            false,
            Top,
            Chain(3),
            "a cape from the shoulders",
        ),
        spec("tail", &["pelvis"], &[], false, Near, Chain(4), "a tail"),
        spec(
            "wing-l",
            &["torso"],
            &[],
            false,
            Near,
            Chain(2),
            "the far wing",
        ),
        spec(
            "wing-r",
            &["torso"],
            &[],
            false,
            Near,
            Chain(2),
            "the near wing",
        ),
        spec(
            "weapon",
            &["hand-r", "hand-l"],
            &[],
            false,
            Near,
            Rigid,
            "a weapon, its grip under the hand",
        ),
        spec(
            "shield",
            &["hand-l", "hand-r"],
            &[],
            false,
            Near,
            Rigid,
            "a shield, its grip under the hand",
        ),
    ],
    legs: &[
        ["leg-upper-l", "leg-lower-l", "foot-l"],
        ["leg-upper-r", "leg-lower-r", "foot-r"],
    ],
};

pub const ALL: [&Template; 1] = [&HUMANOID_SIDE];

pub const NAMES: [&str; 1] = ["humanoid-side"];

pub fn get(name: &str) -> Option<&'static Template> {
    ALL.iter().copied().find(|t| t.name == name)
}
