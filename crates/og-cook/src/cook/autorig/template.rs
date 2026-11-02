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

/// A leg the cook gives two-bone IK: its upper and lower parts, the part
/// at its end (a foot, a paw), and the way a knee that is straight in the
/// art bends.
#[derive(Clone, Copy, Debug)]
pub struct Leg {
    pub upper: &'static str,
    pub lower: &'static str,
    pub end: &'static str,
    /// og-anim's counterclockwise: towards +x for a leg hanging down
    /// (forwards for a character facing right).
    pub counterclockwise: bool,
}

#[derive(Debug)]
pub struct Template {
    pub name: &'static str,
    pub parts: &'static [Spec],
    pub legs: &'static [Leg],
    /// How tall the character stands in world units by default.
    pub height: f32,
    /// What its `-l` and `-r` parts are, for the art contract.
    pub sides: &'static str,
    /// The bones its clips scale (a creature's body squashing, a front
    /// view's thigh foreshortened as the knee lifts): the rigid parts
    /// below them keep their shape.
    pub scaled: &'static [&'static str],
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

const fn leg(
    upper: &'static str,
    lower: &'static str,
    end: &'static str,
    counterclockwise: bool,
) -> Leg {
    Leg {
        upper,
        lower,
        end,
        counterclockwise,
    }
}

use End::*;
use Shape::*;

/// The humanoid parts both humanoid templates share; `far` and `near`
/// name the two sides for a refusal.
macro_rules! humanoid {
    ($l:literal, $r:literal) => {
        &[
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
            spec(
                "ear-l",
                &["head"],
                &[],
                false,
                Near,
                Rigid,
                concat!("the ", $l, " ear"),
            ),
            spec(
                "ear-r",
                &["head"],
                &[],
                false,
                Near,
                Rigid,
                concat!("the ", $r, " ear"),
            ),
            spec(
                "arm-upper-l",
                &["torso"],
                &["arm-lower-l"],
                true,
                Top,
                Limb,
                concat!("the ", $l, " upper arm"),
            ),
            spec(
                "arm-lower-l",
                &["arm-upper-l"],
                &["hand-l"],
                true,
                Top,
                Limb,
                concat!("the ", $l, " forearm"),
            ),
            spec(
                "hand-l",
                &["arm-lower-l"],
                &[],
                true,
                Top,
                Rigid,
                concat!("the ", $l, " hand"),
            ),
            spec(
                "arm-upper-r",
                &["torso"],
                &["arm-lower-r"],
                true,
                Top,
                Limb,
                concat!("the ", $r, " upper arm"),
            ),
            spec(
                "arm-lower-r",
                &["arm-upper-r"],
                &["hand-r"],
                true,
                Top,
                Limb,
                concat!("the ", $r, " forearm"),
            ),
            spec(
                "hand-r",
                &["arm-lower-r"],
                &[],
                true,
                Top,
                Rigid,
                concat!("the ", $r, " hand"),
            ),
            spec(
                "leg-upper-l",
                &["pelvis"],
                &["leg-lower-l"],
                true,
                Top,
                Limb,
                concat!("the ", $l, " thigh"),
            ),
            spec(
                "leg-lower-l",
                &["leg-upper-l"],
                &["foot-l"],
                true,
                Top,
                Limb,
                concat!("the ", $l, " shin"),
            ),
            spec(
                "foot-l",
                &["leg-lower-l"],
                &[],
                true,
                Top,
                Rigid,
                concat!("the ", $l, " foot"),
            ),
            spec(
                "leg-upper-r",
                &["pelvis"],
                &["leg-lower-r"],
                true,
                Top,
                Limb,
                concat!("the ", $r, " thigh"),
            ),
            spec(
                "leg-lower-r",
                &["leg-upper-r"],
                &["foot-r"],
                true,
                Top,
                Limb,
                concat!("the ", $r, " shin"),
            ),
            spec(
                "foot-r",
                &["leg-lower-r"],
                &[],
                true,
                Top,
                Rigid,
                concat!("the ", $r, " foot"),
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
                concat!("the ", $l, " wing"),
            ),
            spec(
                "wing-r",
                &["torso"],
                &[],
                false,
                Near,
                Chain(2),
                concat!("the ", $r, " wing"),
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
        ]
    };
}

/// A humanoid seen from the side, facing right: `-l` is the far side
/// (drawn behind), `-r` the near side.
pub const HUMANOID_SIDE: Template = Template {
    name: "humanoid-side",
    parts: humanoid!("far", "near"),
    legs: &[
        leg("leg-upper-l", "leg-lower-l", "foot-l", true),
        leg("leg-upper-r", "leg-lower-r", "foot-r", true),
    ],
    height: 2.0,
    sides: "`-l` is the far side (drawn behind the body), `-r` the near side; the character faces right",
    scaled: &[],
};

/// A humanoid seen from the front or three-quarters: `-l` is the part on
/// the image's left, `-r` on its right; straight knees bend outwards.
pub const HUMANOID_FRONT: Template = Template {
    name: "humanoid-front",
    parts: humanoid!("image-left", "image-right"),
    legs: &[
        leg("leg-upper-l", "leg-lower-l", "foot-l", false),
        leg("leg-upper-r", "leg-lower-r", "foot-r", true),
    ],
    height: 2.0,
    sides: "`-l` is the part on the image's left, `-r` on its right; the character faces the viewer",
    scaled: &["leg-upper-l", "leg-upper-r"],
};

/// A four-legged animal seen from the side, facing right: `-l` is the
/// far side, `-r` the near side.
pub const QUADRUPED: Template = Template {
    name: "quadruped",
    parts: &[
        spec("body", &[], &[], true, Centre, Limb, "the body"),
        spec("neck", &["body"], &["head"], false, Near, Limb, "the neck"),
        spec(
            "head",
            &["neck", "body"],
            &[],
            true,
            Near,
            Rigid,
            "the head",
        ),
        spec("jaw", &["head"], &[], false, Near, Rigid, "a lower jaw"),
        spec("ear-l", &["head"], &[], false, Near, Rigid, "the far ear"),
        spec("ear-r", &["head"], &[], false, Near, Rigid, "the near ear"),
        spec(
            "mane",
            &["neck", "head"],
            &[],
            false,
            Near,
            Chain(2),
            "a mane",
        ),
        spec(
            "leg-front-upper-l",
            &["body"],
            &["leg-front-lower-l"],
            true,
            Top,
            Limb,
            "the far front leg's upper part",
        ),
        spec(
            "leg-front-lower-l",
            &["leg-front-upper-l"],
            &["paw-front-l"],
            true,
            Top,
            Limb,
            "the far front leg's lower part",
        ),
        spec(
            "paw-front-l",
            &["leg-front-lower-l"],
            &[],
            true,
            Top,
            Rigid,
            "the far front paw",
        ),
        spec(
            "leg-front-upper-r",
            &["body"],
            &["leg-front-lower-r"],
            true,
            Top,
            Limb,
            "the near front leg's upper part",
        ),
        spec(
            "leg-front-lower-r",
            &["leg-front-upper-r"],
            &["paw-front-r"],
            true,
            Top,
            Limb,
            "the near front leg's lower part",
        ),
        spec(
            "paw-front-r",
            &["leg-front-lower-r"],
            &[],
            true,
            Top,
            Rigid,
            "the near front paw",
        ),
        spec(
            "leg-back-upper-l",
            &["body"],
            &["leg-back-lower-l"],
            true,
            Top,
            Limb,
            "the far back leg's upper part",
        ),
        spec(
            "leg-back-lower-l",
            &["leg-back-upper-l"],
            &["paw-back-l"],
            true,
            Top,
            Limb,
            "the far back leg's lower part",
        ),
        spec(
            "paw-back-l",
            &["leg-back-lower-l"],
            &[],
            true,
            Top,
            Rigid,
            "the far back paw",
        ),
        spec(
            "leg-back-upper-r",
            &["body"],
            &["leg-back-lower-r"],
            true,
            Top,
            Limb,
            "the near back leg's upper part",
        ),
        spec(
            "leg-back-lower-r",
            &["leg-back-upper-r"],
            &["paw-back-r"],
            true,
            Top,
            Limb,
            "the near back leg's lower part",
        ),
        spec(
            "paw-back-r",
            &["leg-back-lower-r"],
            &[],
            true,
            Top,
            Rigid,
            "the near back paw",
        ),
        spec("tail", &["body"], &[], false, Near, Chain(4), "a tail"),
        spec(
            "wing-l",
            &["body"],
            &[],
            false,
            Near,
            Chain(2),
            "the far wing",
        ),
        spec(
            "wing-r",
            &["body"],
            &[],
            false,
            Near,
            Chain(2),
            "the near wing",
        ),
    ],
    legs: &[
        leg(
            "leg-front-upper-l",
            "leg-front-lower-l",
            "paw-front-l",
            true,
        ),
        leg(
            "leg-front-upper-r",
            "leg-front-lower-r",
            "paw-front-r",
            true,
        ),
        leg("leg-back-upper-l", "leg-back-lower-l", "paw-back-l", false),
        leg("leg-back-upper-r", "leg-back-lower-r", "paw-back-r", false),
    ],
    height: 1.2,
    sides: "`-l` is the far side (drawn behind the body), `-r` the near side; the animal faces right",
    scaled: &[],
};

/// A simple creature: a body, perhaps a head, and up to eight limbs that
/// swing as chains (legs, arms, tentacles), seen from the side or front.
pub const CREATURE: Template = Template {
    name: "creature",
    parts: &[
        spec("body", &[], &[], true, Centre, Limb, "the body"),
        spec("head", &["body"], &[], false, Near, Rigid, "a head"),
        spec(
            "eye-l",
            &["head", "body"],
            &[],
            false,
            Near,
            Rigid,
            "the left eye",
        ),
        spec(
            "eye-r",
            &["head", "body"],
            &[],
            false,
            Near,
            Rigid,
            "the right eye",
        ),
        spec(
            "ear-l",
            &["head", "body"],
            &[],
            false,
            Near,
            Rigid,
            "the left ear",
        ),
        spec(
            "ear-r",
            &["head", "body"],
            &[],
            false,
            Near,
            Rigid,
            "the right ear",
        ),
        spec("limb-1", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-2", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-3", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-4", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-5", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-6", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-7", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("limb-8", &["body"], &[], false, Near, Chain(3), "a limb"),
        spec("tail", &["body"], &[], false, Near, Chain(4), "a tail"),
        spec(
            "wing-l",
            &["body"],
            &[],
            false,
            Near,
            Chain(2),
            "the left wing",
        ),
        spec(
            "wing-r",
            &["body"],
            &[],
            false,
            Near,
            Chain(2),
            "the right wing",
        ),
    ],
    legs: &[],
    height: 1.0,
    sides: "`-l` is the part on the image's left, `-r` on its right; limbs are numbered from the front (`limb-1`)",
    scaled: &["body"],
};

pub const ALL: [&Template; 4] = [&HUMANOID_SIDE, &HUMANOID_FRONT, &QUADRUPED, &CREATURE];

pub const NAMES: [&str; 4] = ["humanoid-side", "humanoid-front", "quadruped", "creature"];

pub fn get(name: &str) -> Option<&'static Template> {
    ALL.iter().copied().find(|t| t.name == name)
}
