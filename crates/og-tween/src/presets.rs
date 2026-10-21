//! Game-feel presets built from tweens: squash and stretch (an impact scale
//! that recovers to rest) and the hit flash (a flash amount that fades out).

use crate::{Ease, GroupKind, Leaf, Node, Target};

/// Squash and stretch: the scale jumps to `rest × impact` and eases back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Squash {
    /// The x, y and z factors at the impact: (1.25, 0.8) squashes (a
    /// landing), (0.8, 1.25) stretches (a jump). z is for 3D models (a
    /// squash bulges in x and z); 0 leaves z at rest, as a sprite's squash.
    pub impact: [f32; 3],
    /// The scale it recovers to.
    pub rest: [f32; 3],
    /// Seconds to recover.
    pub duration: f32,
    pub ease: Ease,
}

impl Default for Squash {
    fn default() -> Self {
        Squash {
            impact: [1.25, 0.8, 0.0],
            rest: [1.0, 1.0, 1.0],
            duration: 0.18,
            ease: Ease::BackOut,
        }
    }
}

/// The squash of `scale` (a Transform.scale target: three lanes).
pub fn squash(scale: Target, s: &Squash) -> Node {
    let z = if s.impact[2] == 0.0 { 1.0 } else { s.impact[2] };
    let from = vec![
        s.rest[0] * s.impact[0],
        s.rest[1] * s.impact[1],
        s.rest[2] * z,
    ];
    Node::Leaf(
        Leaf::new(scale, s.rest.to_vec())
            .from(from)
            .duration(s.duration)
            .ease(s.ease),
    )
}

/// The hit flash: the colour set at once, the amount from `amount` to 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flash {
    pub color: [f32; 3],
    pub amount: f32,
    pub duration: f32,
    pub ease: Ease,
}

impl Default for Flash {
    fn default() -> Self {
        Flash {
            color: [1.0, 1.0, 1.0],
            amount: 1.0,
            duration: 0.12,
            ease: Ease::QuadIn,
        }
    }
}

/// The flash of `amount` (an f32 flash field) and, when given, its colour
/// field `color` (three lanes).
pub fn flash(amount: Target, color: Option<Target>, f: &Flash) -> Node {
    let fade = Leaf::new(amount, vec![0.0])
        .from(vec![f.amount])
        .duration(f.duration)
        .ease(f.ease);
    match color {
        None => Node::Leaf(fade),
        Some(c) => {
            let set = Leaf::new(c, f.color.to_vec())
                .from(f.color.to_vec())
                .duration(0.0);
            Node::group(GroupKind::Parallel, vec![set, fade], 0)
        }
    }
}
