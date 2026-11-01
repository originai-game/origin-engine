//! 2D skeletal animation: the `type rig2d` asset (bones, slots in draw
//! order, region, weighted-mesh, bounding-box and child-armature
//! attachments, IK constraints, child armatures, clips with bone, slot,
//! deform, draw-order and IK keys and events), its pose at a clip time, the
//! textured triangles that pose draws ([`Rig2dAsset::triangles`]) and the
//! bounding boxes it holds ([`Rig2dAsset::boxes`]).
//!
//! **The text** (`.ogt`, `type rig2d`, `version 1`; the cook stores the
//! canonical form [`Rig2dAsset::to_text`] writes). Rig coordinates are
//! pixels with +y up, angles degrees counter-clockwise; `unit` pixels make
//! one world unit. Bones come parent first; slots draw in file order.
//!
//! ```text
//! texture "hero.png"          # the image the attachments cut from
//! size 128 128                # its pixels (the cook fills it in)
//! filter linear               # or nearest
//! unit 32                     # pixels per world unit
//! bone hip { parent root; at 0 40; rotation 90; skew 0; scale 1 1; length 20 }
//! slot torso { bone hip; attachment torso; color 1 1 1 1; offset 0 0 0 0 }
//! region torso { slot torso; rect 0 0 24 40; at 10 0; rotation -90; skew 0; size 24 40 }
//! mesh cape {
//!   slot cape
//!   vertex u v x y                      # unweighted: x y in the slot's bone
//!   vertex u v bone x y w [bone x y w]  # weighted: x y in each bone, weights summing to 1
//!   triangles 0 1 2 2 3 0
//! }
//! box hurt { slot torso; polygon -8 0 8 0 8 30 -8 30 }
//! child flame { slot lamp; armature fire; clip burn; at 0 6; rotation 0; skew 0; scale 1 1 }
//! ik leg { bones thigh shin; target foot-target; bend clockwise; weight 1 }
//! path rail {                  # a cubic spline, drawn as nothing
//!   slot rail; closed false; constant true   # constant: placed by arc length
//!   lengths 20 45                            # constant false: each curve's end along it
//!   vertex x y                               # three per knot (handle in, knot, handle out),
//!   vertex bone x y w [bone x y w]           # in the slot's bone or weighted
//! }
//! transform follow { bones a b; target t; rotation 0; at 0 0; scale 0 0; shear 0; mix 1 1 1 1 }
//! along spine { path rail; bones s1 s2 s3; position 0 percent; spacing 0 length; rotate tangent 0; mix 1 1 }
//! armature fire {               # a child armature: bone, slot, region, mesh,
//!   bone base { }               # box, child, ik and clip lines as above,
//!   ...                         # sharing the rig's texture and unit
//! }
//! clip walk {
//!   duration 0.8
//!   loop true
//!   bone thigh { rotate 0 20 counterclockwise 1 curve 0.25 0 0.75 1; rotate 0.4 -20; skew 0 0; translate 0 0 0; scale 0.2 1 1.1 step }
//!   slot eye { attachment 0.5 eye-shut; attachment 0.6 eye-open; color 0 1 1 1 1; offset 0 0.2 0 0 0 }
//!   deform cape { key 0 0 0 0 0; key 0.4 0 -3 0 -6 }
//!   order 0.3 cape torso eye    # every slot, back to front; `order <time>` alone is the setup order
//!   ik leg { key 0 1 clockwise; key 0.4 0 counterclockwise }
//!   event 0.2 "step"
//! }
//! ```
//!
//! (Entries are one per line in a file; `;` above only saves space.)
//!
//! **Bones.** `skew` turns the bone's y axis that many degrees further
//! counter-clockwise than its x axis (`rotation`): a bone's x axis is
//! (cos r, sin r) times `scale` x and its y axis (-sin (r + k), cos (r + k))
//! times `scale` y. Regions and child attachments take `skew` the same way.
//!
//! **Slots.** A slot's colour is `color` (linear RGBA multiplied with the
//! texel) and `offset` (RGBA added after it, each -1..1): a drawn texel is
//! `texel * color + offset`, clamped to 0..1. A child armature's slots
//! compose with the slot showing it: the parent's multiplier and offset
//! apply to the child's result.
//!
//! **Keys.** A bone key is relative to the setup pose: `rotate` and `skew`
//! add degrees, `translate` adds pixels, `scale` multiplies. A `rotate`
//! key turns to the next key's angle the shorter way round, unless its
//! angle is followed by `clockwise <n>` or `counterclockwise <n>`: that
//! way round, through n more whole turns (`clockwise 0` from 0 to 30 turns
//! 330 degrees, `counterclockwise 2` from 0 to 30 turns 750). Every key
//! eases towards the next one: linearly by default, `step` holds the value
//! until the next key, and `curve x1 y1
//! x2 y2` plays a cubic Bézier from (0, 0) through those control points to
//! (1, 1), where x is the fraction of time between the keys and y the
//! fraction of the value; more segments follow as `x y x1 y1 x2 y2` (an
//! anchor then its two control points), every x in 0..1 and the anchors in
//! order. A `deform` key offsets every influence of the mesh's vertices in
//! order (one x y pair per influence; an unweighted vertex has one), in
//! that influence's bone space. An `order` key holds a draw order (every
//! slot once, back to front) until the next. An `ik` key sets the
//! constraint's weight (eased) and its bend (held until the next key).
//!
//! **IK.** An `ik` constraint's `bones` are a chain, root first, each the
//! previous one's child. One bone turns to point at the target bone's
//! origin; two (a parent and its child) are solved exactly so the child's
//! tip reaches it, the joint bending clockwise or counter-clockwise; three
//! or more are solved by FABRIK (at most 64 passes, to 1e-4 pixels) from
//! the animated pose, its joints first put on the `bend` side of the line
//! from the root to the target, so a tail or a tentacle curls the asked
//! way. Out of reach the chain points straight at the target. A game moves
//! the target with og.IkTarget2D, whose weight multiplies the constraint's
//! (its `weight` line, or its clip's key).
//!
//! **Child armatures.** A `child` attachment shows an `armature` in its
//! slot, placed by `at`, `rotation`, `skew` and `scale` in the slot's bone
//! and drawn at the slot's place in the draw order. It plays the clip of
//! the armature named as the parent's playing clip when there is one (as
//! DragonBones' inherited animation does), else its `clip` line, else its
//! first clip, at the parent's clip time (wrapped by its own duration when
//! it loops, held at its end otherwise); its IK reaches its own target
//! bones; its clip's events fire as og.Skeleton2DEvent tagged with the
//! armature's name, as its parent's time passes them forwards or
//! backwards. An armature never shows itself, directly or through another.
//!
//! **Transform and path constraints**, applied after the IK, each in
//! file order, to the world transforms (a constrained bone's children
//! follow it). A `transform` moves each of its `bones` `mix` (rotate,
//! translate, scale, shear, each 0..1) of the way to the `target` bone's
//! world transform: its rotation plus `rotation` degrees, its origin at
//! the point `at` in the target, its axes' lengths plus `scale`, the angle
//! between its axes plus `shear` (Spine's absolute world mode; DragonBones
//! has none). An `along` places its `bones` on a `path` attachment while
//! its slot shows it (DragonBones' and Spine's path constraint): from
//! `position` (`percent` of the path's length or `fixed` pixels), `spacing`
//! apart (`length`: each bone's posed length plus that many pixels;
//! `fixed` pixels; `percent` of the length), turned along the path's
//! tangent (`tangent`) or towards the next bone's place (`chain`, the next
//! bone then starting at this one's tip; `chainscale`, the bone stretched
//! to reach it), plus `rotate`'s degrees; `mix` is rotate then translate.
//! A closed path wraps; an open one runs on past its ends along its end
//! handles, as Spine's runtime does. DragonBones' `_computeBezierCurve`
//! instead skips a bone placed past an open path's end (`continue`, a
//! TODO there), leaving whatever its reused positions array held from an
//! earlier frame: state outside the pose, which a pose that is a function
//! of its clip time cannot read, so that behaviour is not followed (the
//! two agree on every bone placed within the path). A `path` draws
//! nothing and a `deform` key offsets its points.
//!
//! **Skins.** The rig's own attachments are its `default` skin. A `skin
//! armor { swap torso torso-armor; swap hat none }` block (at the top
//! level, after the attachments it names) shows, wherever the setup pose
//! or a clip shows the first attachment, the second (in the same slot) or
//! nothing; og.Skeleton2D.skin picks one (Rig2DInfo.skins, `default`
//! first). An `armature` block takes `skin` blocks of its own over its
//! attachments; a child armature wears its skin of the name the rig wears
//! (as it plays its parent's clip by name), else its default.
//!
//! **Bounding boxes.** A `box` attachment draws nothing: its `polygon` (at
//! least three x y points in the slot's bone) is data a game queries
//! (og.anim.skeleton2dBoxes) while its slot shows it.
//!
//! **Converting another format.** DragonBones JSON is imported by the cook
//! (og-cook's `rig2d.rs`). Spine's format is not imported (its licence);
//! a converter maps it so: Spine `bones` (`x`, `y`, `rotation`, `scaleX`,
//! `scaleY`, `length`, y up, degrees counter-clockwise; `shearY` minus
//! `shearX` as `skew`, with `shearX` added to `rotation`) to `bone` blocks;
//! `slots` (`bone`, `attachment`, `color` hex to linear 0..1 after sRGB;
//! `dark` has no line) to `slot` blocks in the same order; a `region`
//! attachment's `x`, `y`, `rotation`, `width`, `height` to `at`,
//! `rotation`, `size` with the atlas region's rectangle (pixels from the
//! top left) as `rect`; a `mesh` attachment's `uvs` (times the region's
//! size, plus its corner) and `vertices` (unweighted pairs, or Spine's
//! `count, bone, x, y, weight` groups) to `vertex` lines and `triangles` as
//! they are; a `boundingbox` to `box` with its `vertices` as `polygon`;
//! `ik` with its `bones` (any number), `target` and `bendPositive` true as
//! `counterclockwise`; an animation's `bones` timelines (`rotate` `value`,
//! `translate` and `scale` `x` `y`, `shear` as `skew`, `time` in seconds;
//! `curve: "stepped"` as `step`, a Bézier `curve` c1 c2 c3 c4 as `curve`),
//! `slots` `attachment` and `rgba`, `deform` `offset` and `vertices` per
//! influence, `drawOrder` offsets resolved into `order` keys, `ik`
//! `mix` and `bendPositive` as `ik` keys, and `events`; a `path`
//! attachment to `path`, a path constraint to `along` and a transform
//! constraint (absolute, world) to `transform`. Clipping has no line.
//!
//! **Determinism.** Every angle goes through [`sincos`] and [`atan2`],
//! written with `+ - * /` and `sqrt` in f64, and a Bézier is solved by
//! bisection, so a pose is the same bits on every host.

mod asset;
mod constraints;
mod math;
mod parse;
mod text;
mod types;

use constraints::{fabrik, path_constraint, sq, transform_constraint, world_of};
pub use math::{Affine2, atan2, bezier, check_curve, sincos};
use math::{DEG, wrap};
use parse::{BODY_KEYS, block, name_of, num, parse_body, parse_err, parse_skins};
use text::{body_text, num_text, quote, skins_text};
use types::Body;
pub use types::*;

use og_core::ogt::{Block, Document, Entry, Number, Value};
use og_core::{Error, ErrorKind, Result};

use crate::SheetFilter;

/// The value `type` names in a rig's text.
pub const RIG2D_TYPE: &str = "rig2d";

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const ARM: &str = r#"type rig2d
version 1
---
texture "arm.png"
size 64 32
unit 10
bone root {
}
bone upper {
  parent root
  rotation 0
  length 20
}
bone lower {
  parent upper
  at 20 0
  length 20
}
bone hand {
  parent root
  at 30 10
}
slot skin {
  bone upper
  attachment skin
}
slot plate {
  bone lower
  attachment plate
  color 1 0.5 0.5 1
}
mesh skin {
  slot skin
  vertex 0 0 0 -2
  vertex 64 0 upper 20 -2 0.5 lower 0 -2 0.5
  vertex 64 32 lower 20 2 1
  triangles 0 1 2
}
region plate {
  slot plate
  rect 0 0 10 4
  at 10 0
  rotation 90
}
ik reach {
  bones upper lower
  target hand
  bend counterclockwise
}
clip wave {
  duration 1
  loop false
  bone upper {
    rotate 0 0
    rotate 1 90
  }
  slot plate {
    attachment 0.5 none
    color 0 1 1 1 1
    color 1 0 0 0 1
  }
  deform skin {
    key 0 0 0 0 0 0 0 0 0
    key 1 0 2 0 0 0 0 0 4
  }
  event 0.25 "lift"
}
"#;

    fn rig() -> Rig2dAsset {
        Rig2dAsset::parse(ARM).expect("the arm parses")
    }

    fn close(a: [f32; 2], b: [f32; 2], eps: f32) -> bool {
        (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
    }

    #[test]
    fn trig_from_arithmetic_matches_the_library() {
        for i in -40..40 {
            let x = f64::from(i) * 0.37;
            let (s, c) = sincos(x);
            assert!(
                (s - x.sin()).abs() < 1e-14 && (c - x.cos()).abs() < 1e-14,
                "{x}"
            );
            assert!(
                (atan2(s * 3.0, c * 3.0) - x.sin().atan2(x.cos())).abs() < 1e-14,
                "{x}"
            );
        }
    }

    #[test]
    fn the_text_parses_back_equal() {
        let r = rig();
        assert_eq!(r.slots[1].attachment, Some(1));
        assert_eq!(r.attachments[0].influences(), 4);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn clips_turn_bones_relative_to_setup_and_switch_slots() {
        let r = rig();
        let c = &r.clips[0];
        // Half way: the upper arm turned 45 degrees; its tip is at 20 (cos, sin) 45.
        let p = r.pose(Some(c), 0.5, &[(0, [100.0, 100.0], 0.0)]);
        let tip = p.world[1].apply(20.0, 0.0);
        let h = 20.0 * std::f32::consts::FRAC_1_SQRT_2;
        assert!(close(tip, [h, h], 1e-4), "{tip:?}");
        // The lower bone's origin is the upper's tip.
        assert!(close([p.world[2].tx, p.world[2].ty], [h, h], 1e-4));
        // At 0.5 the plate's attachment key hides it; the colour runs from white to black.
        assert_eq!(p.slots[1], None);
        assert_eq!(r.pose(Some(c), 0.25, &[]).slots[1], Some(1));
        assert_eq!(p.colors[1], [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn two_bone_ik_reaches_and_bends_the_asked_way() {
        let r = rig();
        // The hand bone at (30, 10) is the target: within reach (40).
        let p = r.pose(None, 0.0, &[]);
        assert!(p.ik_error[0] < 1e-3, "{:?}", p.ik_error);
        let tip = p.world[2].apply(20.0, 0.0);
        assert!(close(tip, [30.0, 10.0], 1e-3), "{tip:?}");
        // Counter-clockwise: the elbow lies left of the line from shoulder to target.
        let elbow = [p.world[2].tx, p.world[2].ty];
        assert!(30.0 * elbow[1] - 10.0 * elbow[0] > 0.0, "{elbow:?}");
        // A target out of reach: the arm points at it, straight.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 1.0)]);
        let tip = p.world[2].apply(20.0, 0.0);
        assert!(close(tip, [0.0, -40.0], 1e-2), "{tip:?}");
        assert!((p.ik_error[0] - 60.0).abs() < 1e-2);
        // Weight 0 keeps the animated pose.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 0.0)]);
        assert!(close([p.world[2].tx, p.world[2].ty], [20.0, 0.0], 1e-4));
    }

    #[test]
    fn weighted_and_deformed_vertices_follow_their_bones() {
        let r = rig();
        let c = &r.clips[0];
        // At the end the upper arm is turned 90 degrees and the deform at full.
        let p = r.pose(Some(c), 1.0, &[(0, [0.0, 40.0], 0.0)]);
        let (v, i) = r.triangles(&p);
        // The mesh first (slot order); the plate is hidden at 1.
        assert_eq!(i, [0, 1, 2]);
        // Vertex 0: (0, -2) + (0, 2) in the upper arm turned 90: the origin.
        assert!(close(v[0].position, [0.0, 0.0], 1e-5), "{:?}", v[0]);
        // Vertex 1: half of upper (20, -2) and half of lower (0, -2), both
        // turned: (2, 20) in pixels, a tenth in world units.
        assert!(close(v[1].position, [0.2, 2.0], 1e-5), "{:?}", v[1]);
        // Vertex 2: lower (20, 2 + 4) turned: (-6, 40).
        assert!(close(v[2].position, [-0.6, 4.0], 1e-5), "{:?}", v[2]);
        assert_eq!(v[2].uv, [1.0, 1.0]);
    }

    #[test]
    fn a_region_is_a_quad_of_its_rect() {
        let r = rig();
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 1.0)]);
        let (v, i) = r.triangles(&p);
        assert_eq!(i.len(), 9);
        // The plate: 10 x 4 at (10, 0) of the lower bone (from 20), turned 90.
        let quad = &v[3..7];
        assert!(close(quad[0].position, [3.2, -0.5], 1e-3), "{quad:?}");
        assert_eq!(quad[0].uv, [0.0, 4.0 / 32.0]);
        assert_eq!(quad[2].uv, [10.0 / 64.0, 0.0]);
        assert_eq!(quad[0].color, [1.0, 0.5, 0.5, 1.0]);
    }

    /// A rig of every DragonBones feature the text carries beyond the
    /// arm: a child armature in a slot (with a box of its own), a box,
    /// colour offsets, a draw-order key.
    pub(crate) const NEST: &str = r#"type rig2d
version 1
---
texture "parts.png"
size 4 4
unit 1
bone root {
}
bone hand {
  parent root
  at 10 0
  rotation 90
  length 4
}
slot body {
  bone root
  attachment body
}
slot hand {
  bone hand
  attachment lamp
  color 0.5 1 1 1
  offset 0 0.25 0 0
}
slot top {
  bone root
  attachment top
}
region body {
  slot body
  rect 0 0 2 2
}
child lamp {
  slot hand
  armature flame
  clip burn
  at 4 0
  scale 2 2
}
box reach {
  slot top
  polygon 0 0 4 0 4 2
}
region top {
  slot top
  rect 2 2 2 2
}
clip wave {
  duration 1
}
clip idle {
  duration 4
  slot top {
    attachment 0 top
    attachment 3.5 reach
  }
  order 1 "top" "hand" "body"
  order 3
}
armature flame {
  bone base {
  }
  slot fire {
    bone base
    attachment fire
    color 0.5 0.5 0.5 1
    offset 0.1 0 0 0
  }
  slot heat {
    bone base
    attachment heat
  }
  region fire {
    slot fire
    rect 0 0 2 2
    at 1 0
  }
  box heat {
    slot heat
    polygon 0 0 1 0 1 1
  }
  clip burn {
    duration 1
    bone base {
      rotate 0 0
      rotate 1 90
    }
  }
  clip wave {
    duration 2
    bone base {
      rotate 0 0
      rotate 2 -90
    }
  }
}
"#;

    /// An armature's own skins: the flame swaps its fire for an ember in
    /// its skin `gold`, which it wears while the rig wears its `gold`; in
    /// the rig's default skin, or one the flame has none of (`silver`), it
    /// wears its default. Before, an armature took no `skin` block and a
    /// child always wore its default.
    #[test]
    fn a_child_armature_wears_its_skin_of_the_name_the_rig_wears() {
        let text = NEST
            .replace(
                "clip wave {\n  duration 1\n}\n",
                "skin gold {\n}\nskin silver {\n}\nclip wave {\n  duration 1\n}\n",
            )
            .replace(
                "  box heat {\n",
                "  region ember {\n    slot fire\n    rect 2 0 2 2\n  }\n  skin gold {\n    swap fire ember\n  }\n  box heat {\n",
            );
        let r = Rig2dAsset::parse(&text).expect("an armature's skin");
        let flame = &r.armatures[0];
        assert_eq!(flame.skins.len(), 1);
        let (fire_slot, fire, ember) = (
            flame.slots.iter().position(|s| s.name == "fire").unwrap(),
            flame
                .attachments
                .iter()
                .position(|a| a.name == "fire")
                .unwrap(),
            flame
                .attachments
                .iter()
                .position(|a| a.name == "ember")
                .unwrap(),
        );
        let shown = |skin: u32| {
            let p = r.pose_in_skin(Some(&r.clips[1]), 0.5, &[], skin);
            assert_eq!(p.children.len(), 1, "the lamp shows the flame");
            p.children[0].1.slots[fire_slot]
        };
        assert_eq!(shown(0), Some(fire), "the default");
        assert_eq!(shown(1), Some(ember), "gold");
        assert_eq!(shown(2), Some(fire), "silver: the flame has none");
        // The canonical text keeps the armature's skin.
        assert_eq!(Rig2dAsset::parse(&r.to_text()).unwrap(), r);
        let bad = Rig2dAsset::parse(&text.replace("swap fire ember", "swap fire ash"))
            .unwrap_err()
            .reason()
            .to_string();
        assert!(
            bad.contains("armature flame") && bad.contains("ash"),
            "{bad}"
        );
    }

    /// The centre of quad `q` (four corners from `first`).
    fn centre(v: &[RigVertex], first: usize) -> [f32; 2] {
        let q = &v[first..first + 4];
        [
            q.iter().map(|c| c.position[0]).sum::<f32>() / 4.0,
            q.iter().map(|c| c.position[1]).sum::<f32>() / 4.0,
        ]
    }

    #[test]
    fn skew_turns_a_bones_y_axis_apart_from_its_x_axis() {
        let text = ARM
            .replace("bone hand {\n  parent root\n  at 30 10\n}", "bone hand {\n  parent root\n  at 30 10\n}\nbone tilt {\n  parent root\n  at 10 0\n  rotation 10\n  skew 30\n  scale 2 3\n}")
            .replace("  rect 0 0 10 4\n  at 10 0\n  rotation 90\n", "  rect 0 0 10 4\n  at 0 0\n  rotation 0\n  skew 30\n  size 2 2\n")
            .replace("bone lower\n  attachment plate", "bone tilt\n  attachment plate")
            .replace("  bone upper {\n    rotate 0 0", "  bone tilt {\n    skew 0 0\n    skew 1 20\n  }\n  bone upper {\n    rotate 0 0");
        let r = Rig2dAsset::parse(&text).expect("skewed");
        let tilt = r.bone("tilt").unwrap();
        assert_eq!(r.bones[tilt].skew, 30.0);
        let d = |deg: f64| deg.to_radians();
        let near = |a: f32, b: f64| (f64::from(a) - b).abs() < 1e-5;
        // Setup: x axis at 10 degrees times 2, y axis at 40 degrees times 3.
        let m = r.pose(None, 0.0, &[(0, [40.0, 0.0], 0.0)]).world[tilt];
        assert!(
            near(m.a, 2.0 * d(10.0).cos()) && near(m.b, 2.0 * d(10.0).sin()),
            "{m:?}"
        );
        assert!(
            near(m.c, -3.0 * d(40.0).sin()) && near(m.d, 3.0 * d(40.0).cos()),
            "{m:?}"
        );
        // Half way through the clip the skew key adds 10: the y axis at 50.
        let p = r.pose(Some(&r.clips[0]), 0.5, &[(0, [40.0, 0.0], 0.0)]);
        let m = p.world[tilt];
        assert!(near(m.a, 2.0 * d(10.0).cos()), "{m:?}");
        assert!(
            near(m.c, -3.0 * d(50.0).sin()) && near(m.d, 3.0 * d(50.0).cos()),
            "{m:?}"
        );
        // The plate region's own skew 30: its corner (1, 1) is (1 - sin 30, cos 30) in the bone.
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 0.0)]);
        let (v, _) = r.triangles(&p);
        let want = p.world[tilt].apply(1.0 - 0.5, 3f32.sqrt() / 2.0);
        assert!(
            close(v[5].position, [want[0] / 10.0, want[1] / 10.0], 1e-5),
            "{:?} {want:?}",
            v[5]
        );
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn bezier_keys_follow_their_curve_not_the_line() {
        let text = ARM
            .replace("    rotate 0 0\n    rotate 1 90", "    rotate 0 0 curve 0.42 0 1 1\n    rotate 1 90")
            .replace(
                "  slot plate {",
                "  bone hand {\n    translate 0 0 0 curve 0.1 0.6 0.3 0.8 0.5 0.8 0.7 0.8 0.9 1\n    translate 1 100 0\n  }\n  slot plate {",
            );
        let r = Rig2dAsset::parse(&text).expect("curves");
        let c = &r.clips[0];
        let p = r.pose(Some(c), 0.5, &[(0, [0.0, 0.0], 0.0)]);
        // Ease-in (0.42 0 1 1) at half time is 0.3153568 of the way
        // (solved independently): 28.38211 degrees, not the line's 45.
        let angle = f64::from(p.world[1].b)
            .atan2(f64::from(p.world[1].a))
            .to_degrees();
        assert!((angle - 90.0 * 0.315_356_81).abs() < 1e-3, "{angle}");
        // Two segments through the anchor (0.5, 0.8).
        let hand = r.bone("hand").unwrap();
        for (t, f) in [(0.25, 0.673_215_3), (0.75, 0.879_110_6), (0.5, 0.8)] {
            let x = r.pose(Some(c), t, &[(0, [0.0, 0.0], 0.0)]).world[hand].tx;
            assert!((x - (30.0 + 100.0 * f)).abs() < 1e-3, "{t}: {x}");
        }
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let bad = |curve: &str| {
            Rig2dAsset::parse(
                &ARM.replace("    rotate 0 0\n", &format!("    rotate 0 0 {curve}\n")),
            )
            .expect_err(curve)
            .reason()
            .to_string()
        };
        assert!(bad("curve 0.5 0 1").contains("x1 y1 x2 y2"));
        assert!(bad("curve 1.5 0 1 1").contains("0..1"));
        assert!(bad("smooth").contains("step"));
    }

    #[test]
    fn rotate_keys_turn_the_asked_way_through_whole_turns() {
        // The bone's angle at `t` of a 1 s turn from 0 to 30 degrees; each
        // expected angle is the turn's total (worked out by hand) times t.
        let angle_at = |turn: &str, t: f32| -> f64 {
            let text = ARM.replace(
                "    rotate 0 0\n    rotate 1 90",
                &format!("    rotate 0 0{turn}\n    rotate 1 30"),
            );
            let r = Rig2dAsset::parse(&text).expect(turn);
            assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
            let p = r.pose(Some(&r.clips[0]), t, &[(0, [0.0, 0.0], 0.0)]);
            f64::from(p.world[1].b)
                .atan2(f64::from(p.world[1].a))
                .to_degrees()
        };
        let same = |a: f64, b: f64| (a - b).rem_euclid(360.0).min((b - a).rem_euclid(360.0)) < 1e-3;
        for (turn, total) in [
            ("", 30.0),
            (" counterclockwise 0", 30.0),
            (" counterclockwise 2", 750.0),
            (" clockwise 0", -330.0),
            (" clockwise 1", -690.0),
        ] {
            for t in [0.1f32, 0.25, 0.6] {
                let a = angle_at(turn, t);
                assert!(
                    same(a, total * f64::from(t)),
                    "{turn} at {t}: {a}, not {}",
                    total * f64::from(t)
                );
            }
        }
        // The turn's own arithmetic: equal angles turn 0 the nearest way, a whole turn per count.
        assert_eq!(turn_delta(10.0, 10.0, Turn::Clockwise(0)), 0.0);
        assert_eq!(turn_delta(10.0, 10.0, Turn::Counterclockwise(1)), 360.0);
        assert_eq!(turn_delta(170.0, -170.0, Turn::Shorter), 20.0);
        let bad =
            Rig2dAsset::parse(&ARM.replace("    rotate 0 0\n", "    rotate 0 0 clockwise 1.5\n"))
                .expect_err("a fraction of a turn");
        assert!(bad.reason().contains("whole number"), "{}", bad.reason());
    }

    #[test]
    fn colour_offsets_add_after_the_multiplier_and_animate() {
        let text = ARM
            .replace(
                "  color 1 0.5 0.5 1\n}",
                "  color 1 0.5 0.5 1\n  offset 0.5 0 0 0\n}",
            )
            .replace(
                "    color 1 0 0 0 1\n",
                "    color 1 0 0 0 1\n    offset 0 0 0.2 0 0\n    offset 1 0 0.6 0 0\n",
            );
        let r = Rig2dAsset::parse(&text).expect("offsets");
        assert_eq!(r.slots[1].offset, [0.5, 0.0, 0.0, 0.0]);
        let p = r.pose(None, 0.0, &[(0, [40.0, 0.0], 1.0)]);
        let (v, _) = r.triangles(&p);
        assert_eq!(
            (v[3].color, v[3].offset),
            ([1.0, 0.5, 0.5, 1.0], [0.5, 0.0, 0.0, 0.0])
        );
        let p = r.pose(Some(&r.clips[0]), 0.25, &[(0, [40.0, 0.0], 1.0)]);
        assert!(
            close([p.offsets[1][0], p.offsets[1][1]], [0.0, 0.3], 1e-6),
            "{:?}",
            p.offsets
        );
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    fn ik_keys_animate_the_weight_and_bend_and_the_target_weight_multiplies() {
        let text = ARM.replace(
            "  event 0.25 \"lift\"",
            "  ik reach {\n    key 0 1 counterclockwise\n    key 0.5 0 clockwise\n    key 1 1 clockwise\n  }\n  event 0.25 \"lift\"",
        );
        let r = Rig2dAsset::parse(&text).expect("ik keys");
        let c = &r.clips[0];
        let elbow_side = |p: &Pose2d| 30.0 * p.world[2].ty - 10.0 * p.world[2].tx;
        // At 0 the key's weight 1, counter-clockwise: the hand reached, the elbow left.
        let p = r.pose(Some(c), 0.0, &[]);
        assert!(close(p.world[2].apply(20.0, 0.0), [30.0, 10.0], 1e-3) && elbow_side(&p) > 0.0);
        // At 0.5 the weight is 0: the clip's arm (upper turned 45, straight),
        // even with a target of weight 1 (it multiplies the clip's).
        let h = 40.0 * std::f32::consts::FRAC_1_SQRT_2;
        for targets in [&[][..], &[(0, [30.0, 10.0], 1.0)][..]] {
            let p = r.pose(Some(c), 0.5, targets);
            assert!(
                close(p.world[2].apply(20.0, 0.0), [h, h], 1e-3),
                "{:?}",
                p.world[2]
            );
        }
        // At 1 weight 1, clockwise: reached with the elbow right of the line.
        let p = r.pose(Some(c), 1.0, &[]);
        assert!(close(p.world[2].apply(20.0, 0.0), [30.0, 10.0], 1e-3) && elbow_side(&p) < 0.0);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
    }

    #[test]
    // Test reference values: libm checks the result, never the tick.
    #[allow(clippy::disallowed_methods)]
    fn a_child_armature_is_posed_and_drawn_in_its_slot() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        assert_eq!(r.armatures.len(), 1);
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        // The hand bone at (10, 0) points up; the child sits 4 along it at
        // scale 2: its origin (10, 4), its x axis (0, 2), its y axis (-2, 0).
        let place = |cx: f32, cy: f32| [10.0 - 2.0 * cy, 4.0 + 2.0 * cx];
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let idle = &r.clips[1];
        // idle has no child clip of its name: the child plays burn (its
        // `clip` line) at the parent's time, 0.5 (base turned 45), and wraps it at 2.5.
        for t in [0.5, 2.5] {
            let p = r.pose(Some(idle), t, &[]);
            let (v, i) = r.triangles(&p);
            assert_eq!((v.len(), i.len()), (12, 18), "body, the flame, top");
            assert!(
                close(centre(&v, 4), place(s, s), 1e-4),
                "{t}: {:?}",
                centre(&v, 4)
            );
        }
        // wave: the child has a wave of its own (2 s, to -90) and plays it.
        let p = r.pose(Some(&r.clips[0]), 0.5, &[]);
        let (v, _) = r.triangles(&p);
        let (sn, cs) = (22.5f32.to_radians().sin(), 22.5f32.to_radians().cos());
        assert!(
            close(centre(&v, 4), place(cs, -sn), 1e-4),
            "{:?}",
            centre(&v, 4)
        );
        // Colours compose: the hand slot's map over the fire's.
        assert_eq!(v[4].color, [0.25, 0.5, 0.5, 1.0]);
        assert!(
            close([v[4].offset[0], v[4].offset[1]], [0.05, 0.25], 1e-6),
            "{:?}",
            v[4].offset
        );
        // A child that shows its parent is refused.
        let cycle = NEST.replace(
            "  clip burn {",
            "  slot again {\n    bone base\n  }\n  child again {\n    slot again\n    armature flame\n  }\n  clip burn {",
        );
        let e = Rig2dAsset::parse(&cycle)
            .expect_err("a cycle")
            .reason()
            .to_string();
        assert!(e.contains("shows itself"), "{e}");
        let e = Rig2dAsset::parse(&NEST.replace("  clip burn\n", "  clip smoulder\n"))
            .expect_err("no such clip")
            .reason()
            .to_string();
        assert!(e.contains("smoulder"), "{e}");
    }

    #[test]
    fn order_keys_redraw_the_slots_back_to_front() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        let idle = &r.clips[1];
        let first_u = |t: f32| {
            let (v, _) = r.triangles(&r.pose(Some(idle), t, &[]));
            v[0].uv[0]
        };
        // Setup order: body (u from 0) first; from 1 s top (u from 0.5);
        // `order 3` alone is the setup order again.
        assert_eq!((first_u(0.5), first_u(1.5), first_u(3.2)), (0.0, 0.5, 0.0));
        let p = r.pose(Some(idle), 1.5, &[]);
        assert_eq!(p.order, [2, 1, 0]);
        let e = Rig2dAsset::parse(
            &NEST.replace("order 1 \"top\" \"hand\" \"body\"", "order 1 top hand"),
        )
        .expect_err("two of three")
        .reason()
        .to_string();
        assert!(e.contains("each of the 3 slots"), "{e}");
    }

    #[test]
    fn bounding_boxes_are_the_polygons_the_pose_shows() {
        let r = Rig2dAsset::parse(NEST).expect("nested");
        let idle = &r.clips[1];
        let s = std::f32::consts::FRAC_1_SQRT_2;
        // At 0.5 the top slot shows its region, the child's heat box shows
        // in the hand: (1, 0) in the base turned 45, then placed.
        let b = r.boxes(&r.pose(Some(idle), 0.5, &[]));
        assert_eq!(b.len(), 1, "{b:?}");
        assert_eq!(
            (b[0].slot.as_str(), b[0].name.as_str()),
            ("hand/heat", "heat")
        );
        assert!(
            close(b[0].points[1], [10.0 - 2.0 * s, 4.0 + 2.0 * s], 1e-4),
            "{b:?}"
        );
        // From 3.5 the top slot shows the reach box, in the root bone (back
        // in the setup order, after the hand).
        let b = r.boxes(&r.pose(Some(idle), 3.6, &[]));
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].slot, "hand/heat");
        assert_eq!(
            (b[1].slot.as_str(), b[1].points.clone()),
            ("top", vec![[0.0, 0.0], [4.0, 0.0], [4.0, 2.0]])
        );
        let (v, i) = r.triangles(&r.pose(Some(idle), 3.6, &[]));
        assert_eq!((v.len(), i.len()), (8, 12), "a box draws nothing");
    }

    /// The one- and two-bone solutions as they were before chains of any
    /// length (the numbers the depth stream's solver gave, to the bit).
    #[test]
    fn one_and_two_bone_ik_give_the_poses_they_always_gave() {
        let a = |a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32| Affine2 { a, b, c, d, tx, ty };
        let r = rig();
        let cases = [
            (
                [30.0, 10.0],
                1.0,
                a(0.5563508, 0.8309475, -0.8309475, 0.5563508, 0.0, 0.0),
                a(
                    0.9436492,
                    -0.33094752,
                    0.33094752,
                    0.9436492,
                    11.127016,
                    16.61895,
                ),
                0.0,
            ),
            (
                [-12.5, 21.25],
                1.0,
                a(-0.9912518, 0.13198434, -0.13198434, -0.9912518, 0.0, 0.0),
                a(
                    0.36625195,
                    0.93051577,
                    -0.93051577,
                    0.36625195,
                    -19.825037,
                    2.6396868,
                ),
                2.6973983e-6,
            ),
            (
                [0.0, -100.0],
                1.0,
                a(0.00022357237, -1.0, 1.0, 0.00022357237, 0.0, 0.0),
                a(-5.5893106e-5, -1.0, 1.0, -5.5893106e-5, 0.0044714473, -20.0),
                60.0,
            ),
            (
                [25.0, -5.0],
                0.4,
                a(0.96298677, 0.26954868, -0.26954868, 0.96298677, 0.0, 0.0),
                a(
                    0.9656971,
                    -0.25967127,
                    0.25967127,
                    0.9656971,
                    19.259735,
                    5.3909736,
                ),
                14.534759,
            ),
        ];
        for (t, w, upper, lower, err) in cases {
            let p = r.pose(None, 0.0, &[(0, t, w)]);
            assert_eq!(
                (p.world[1], p.world[2], p.ik_error[0]),
                (upper, lower, err),
                "{t:?}"
            );
        }
        let r = Rig2dAsset::parse(&ARM.replace("bones upper lower", "bones upper")).unwrap();
        for (t, w, upper, err) in [
            (
                [30.0, 10.0],
                1.0,
                a(0.9486833, 0.31622776, -0.31622776, 0.9486833, 0.0, 0.0),
                2.9802322e-7,
            ),
            (
                [-12.5, -21.25],
                0.5,
                a(0.4964775, -0.86804956, 0.86804956, 0.4964775, 0.0, 0.0),
                21.400766,
            ),
        ] {
            let p = r.pose(None, 0.0, &[(0, t, w)]);
            assert_eq!((p.world[1], p.ik_error[0]), (upper, err), "{t:?}");
        }
    }

    /// A tail of four 10-pixel bones, straight along x in its setup pose.
    const TAIL: &str = r#"type rig2d
version 1
---
texture "tail.png"
size 8 8
bone root {
}
bone t1 {
  parent root
  length 10
}
bone t2 {
  parent t1
  at 10 0
  length 10
}
bone t3 {
  parent t2
  at 10 0
  length 10
}
bone t4 {
  parent t3
  at 10 0
  length 10
}
bone goal {
  parent root
  at 25 15
}
ik curl {
  bones t1 t2 t3 t4
  target goal
  bend counterclockwise
}
"#;

    #[test]
    // Test reference values: libm checks the result, never the tick.
    #[allow(clippy::disallowed_methods)]
    fn a_chain_of_any_length_reaches_and_curls_the_asked_way() {
        let r = Rig2dAsset::parse(TAIL).expect("a four-bone chain");
        assert_eq!(r.iks[0].bones, [1, 2, 3, 4]);
        let joints = |p: &Pose2d| -> Vec<[f32; 2]> {
            (1..=4)
                .map(|b| [p.world[b].tx, p.world[b].ty])
                .chain(std::iter::once(p.world[4].apply(10.0, 0.0)))
                .collect()
        };
        // Left of the line from the root to the target is counter-clockwise.
        let sides = |p: &Pose2d, t: [f32; 2]| -> Vec<f32> {
            joints(p)[1..4]
                .iter()
                .map(|q| t[0] * q[1] - t[1] * q[0])
                .collect()
        };
        for (bend, sign) in [("counterclockwise", 1.0), ("clockwise", -1.0)] {
            let r = Rig2dAsset::parse(&TAIL.replace("counterclockwise", bend)).unwrap();
            // From the straight setup pose, to the goal bone (25, 15): 29.15 of 40 away.
            let p = r.pose(None, 0.0, &[]);
            let j = joints(&p);
            assert!(close(j[4], [25.0, 15.0], 1e-3), "{bend}: {j:?}");
            assert!(p.ik_error[0] < 1e-3, "{:?}", p.ik_error);
            for w in j.windows(2) {
                let l = ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
                assert!((l - 10.0).abs() < 1e-3, "{bend}: a bone stretched to {l}");
            }
            assert!(
                sides(&p, [25.0, 15.0]).iter().all(|s| s * sign > 0.0),
                "{bend}: {j:?}"
            );
        }
        // A target behind the root, closer than the chain is long: the tail folds back to it.
        let p = r.pose(None, 0.0, &[(0, [-12.0, -6.0], 1.0)]);
        assert!(
            close(joints(&p)[4], [-12.0, -6.0], 1e-3) && p.ik_error[0] < 1e-3,
            "{:?}",
            joints(&p)
        );
        assert!(sides(&p, [-12.0, -6.0]).iter().all(|s| *s > 0.0));
        // Out of reach: straight at the target, 60 short.
        let p = r.pose(None, 0.0, &[(0, [0.0, -100.0], 1.0)]);
        assert!(close(joints(&p)[4], [0.0, -40.0], 1e-3), "{:?}", joints(&p));
        assert!((p.ik_error[0] - 60.0).abs() < 1e-3);
        // Weight 0 keeps the setup pose.
        let p = r.pose(None, 0.0, &[(0, [25.0, 15.0], 0.0)]);
        assert!(close(joints(&p)[4], [40.0, 0.0], 1e-4));
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        // A chain with a gap is refused.
        let e = Rig2dAsset::parse(&TAIL.replace("bones t1 t2 t3 t4", "bones t1 t3 t4"))
            .expect_err("t3 is not t1's child")
            .reason()
            .to_string();
        assert!(e.contains("previous one's child"), "{e}");
    }

    fn near(m: &Affine2, want: [f32; 6], eps: f32) -> bool {
        [m.a, m.b, m.c, m.d, m.tx, m.ty]
            .iter()
            .zip(want)
            .all(|(g, w)| (g - w).abs() < eps)
    }

    #[test]
    fn a_transform_constraint_moves_its_bones_by_the_mix_towards_the_target() {
        let text = r#"type rig2d
version 1
---
texture "t.png"
bone root {
}
bone target {
  parent root
  at 10 5
  rotation 90
  scale 2 1
}
bone arm {
  parent root
}
bone hand {
  parent arm
  at 10 0
}
transform follow {
  bones arm
  target target
  at 2 0
  mix 0.5 0.5 1 0
}
"#;
        let r = Rig2dAsset::parse(text).expect("a transform constraint");
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let p = r.pose(None, 0.0, &[]);
        // Worked by hand: half of the target's 90 degrees (45); half way to
        // the target's point (2, 0), which is (10, 5) + 2 * (0, 2) = (10, 9),
        // from (0, 0): (5, 4.5); the x axis's length all the way to the
        // target's 2, the y axis's kept at the target's 1; no shear mix.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let arm = p.world[2];
        assert!(
            near(&arm, [2.0 * h, 2.0 * h, -h, h, 5.0, 4.5], 1e-5),
            "{arm:?}"
        );
        // The child follows: (10, 0) in the arm, 20 along the 45-degree axis.
        let hand = p.world[3];
        assert!(
            (hand.tx - (5.0 + 20.0 * h)).abs() < 1e-4 && (hand.ty - (4.5 + 20.0 * h)).abs() < 1e-4,
            "{hand:?}"
        );
        let bad = Rig2dAsset::parse(&text.replace("bones arm", "bones target"))
            .expect_err("its own target");
        assert!(
            bad.reason().contains("not one of its bones"),
            "{}",
            bad.reason()
        );
    }

    /// A path of one curve from (0, 0) through the handles (0, 10) and
    /// (10, 10) to (10, 0) on bone `rail`, two bones placed on it.
    fn path_rig(rail: &str, along: &str) -> Rig2dAsset {
        Rig2dAsset::parse(&format!(
            r#"type rig2d
version 1
---
texture "t.png"
bone root {{
}}
bone rail {{
  parent root
  {rail}
}}
bone a {{
  parent root
  at -40 3
  rotation 120
  length 8
}}
bone b {{
  parent a
  at 8 0
  length 8
}}
slot rail {{
  bone rail
  attachment rail
}}
path rail {{
  slot rail
  {along}
}}
along follow {{
  path rail
  bones a b
}}
"#
        ))
        .expect("a path constraint")
    }

    #[test]
    fn a_path_constraint_places_its_bones_along_the_curve() {
        // By the curve's parameter (`constant false`): spacing 25 percent of
        // its length 20 puts a at 5 (t 0.25) and b at 10 (t 0.5). Worked by
        // hand from the Bernstein form: B(0.25) = (1.5625, 5.625) with the
        // tangent (3.75, 5), 53.130102 degrees; B(0.5) = (5, 7.5) with the
        // tangent (5, 0), 0 degrees.
        let curve = "constant false\n  lengths 20\n  vertex 0 -10\n  vertex 0 0\n  vertex 0 10\n  vertex 10 10\n  vertex 10 0\n  vertex 10 -10";
        let mut r = path_rig("", curve);
        r.paths[0].spacing = 0.25;
        r.paths[0].spacing_mode = PathSpacing::Percent;
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let p = r.pose(None, 0.0, &[]);
        // (3.75, 5) over its length 6.25.
        let (s, c) = (0.8f32, 0.6f32);
        assert!(
            near(&p.world[2], [c, s, -s, c, 1.5625, 5.625], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [1.0, 0.0, 0.0, 1.0, 5.0, 7.5], 1e-4),
            "{:?}",
            p.world[3]
        );

        // By arc length on a straight path turned 30 degrees with its bone:
        // `length` spacing puts a at position 3 and b one bone length plus
        // spacing 2 further, at 13; both along the line.
        let line = "vertex -10 0\n  vertex 0 0\n  vertex 10 0\n  vertex 20 0\n  vertex 30 0\n  vertex 40 0";
        let mut r = path_rig("rotation 30", line);
        r.paths[0].position = 0.1;
        r.paths[0].spacing = 2.0;
        let p = r.pose(None, 0.0, &[]);
        let (s, c) = (0.5f32, 0.75f32.sqrt());
        assert!(
            near(&p.world[2], [c, s, -s, c, 3.0 * c, 3.0 * s], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [c, s, -s, c, 13.0 * c, 13.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );

        // Chain: each bone turns towards the next place and the next bone
        // starts at its tip (3 + 8 = 11); chainscale stretches a to reach
        // 13 from 3 (10 of its 8 pixels, 1.25) and b to reach 23 from 13.
        r.paths[0].rotate = PathRotate::Chain;
        let p = r.pose(None, 0.0, &[]);
        assert!(
            near(&p.world[3], [c, s, -s, c, 11.0 * c, 11.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );
        r.paths[0].rotate = PathRotate::ChainScale;
        let p = r.pose(None, 0.0, &[]);
        let k = 1.25;
        assert!(
            near(&p.world[2], [k * c, k * s, -s, c, 3.0 * c, 3.0 * s], 1e-4),
            "{:?}",
            p.world[2]
        );
        assert!(
            near(&p.world[3], [k * c, k * s, -s, c, 13.0 * c, 13.0 * s], 1e-4),
            "{:?}",
            p.world[3]
        );

        // Translate mix 0.5: half way from a's own origin (-40, 3).
        r.paths[0].rotate = PathRotate::Tangent;
        r.paths[0].mix = [1.0, 0.5];
        let a = r.pose(None, 0.0, &[]).world[2];
        assert!(
            (a.tx - (-40.0 + 3.0 * c) / 2.0).abs() < 1e-4
                && (a.ty - (3.0 + 3.0 * s) / 2.0).abs() < 1e-4,
            "{a:?}"
        );
        // A path its slot does not show places nothing.
        r.slots[0].attachment = None;
        let a = r.pose(None, 0.0, &[]).world[2];
        assert!(
            (a.tx + 40.0).abs() < 1e-6 && (a.ty - 3.0).abs() < 1e-6,
            "{a:?}"
        );
    }

    #[test]
    fn a_skin_swaps_the_default_attachments_wherever_they_show() {
        let text = ARM.replace(
            "ik reach {",
            "region plate-gold {\n  slot plate\n  rect 10 0 10 4\n  at 10 0\n  rotation 90\n}\nskin gold {\n  swap plate plate-gold\n  swap skin none\n}\nik reach {",
        );
        let r = Rig2dAsset::parse(&text).expect("a skin");
        assert_eq!(Rig2dAsset::parse(&r.to_text()).expect("canonical"), r);
        let gold = r
            .attachments
            .iter()
            .position(|a| a.name == "plate-gold")
            .unwrap();
        let plate = r
            .attachments
            .iter()
            .position(|a| a.name == "plate")
            .unwrap();
        // Default: the mesh and the plate; gold: no mesh and the gold plate.
        let pose = |skin: u32, t: f32| {
            r.pose_in_skin(Some(&r.clips[0]), t, &[(0, [0.0, 0.0], 0.0)], skin)
                .slots
        };
        assert_eq!(pose(0, 0.2), [Some(0), Some(plate)]);
        assert_eq!(pose(1, 0.2), [None, Some(gold)]);
        // A skin index past the rig's is the default.
        assert_eq!(pose(7, 0.2), [Some(0), Some(plate)]);
        // The clip hides the plate at 0.5: in every skin.
        assert_eq!(pose(1, 0.6), [None, None]);
        // The gold plate's texels: u from 10 of 64.
        let p = r.pose_in_skin(None, 0.0, &[(0, [0.0, 0.0], 0.0)], 1);
        let (v, _) = r.triangles(&p);
        assert_eq!(v.len(), 4);
        assert!((v[0].uv[0] - 10.0 / 64.0).abs() < 1e-6, "{:?}", v[0]);
        let bad = |swap: &str| {
            Rig2dAsset::parse(&text.replace("  swap skin none", swap))
                .expect_err(swap)
                .reason()
                .to_string()
        };
        assert!(bad("  swap skin plate").contains("stays in its slot"));
        assert!(bad("  swap skin cape").contains("no attachment"));
    }

    #[test]
    fn mistakes_are_refused_with_their_line() {
        let bad = |from: &str, to: &str| {
            Rig2dAsset::parse(&ARM.replace(from, to))
                .expect_err(to)
                .reason()
                .to_string()
        };
        assert!(bad("  parent upper\n  at 20 0", "  parent nobody\n  at 20 0").contains("nobody"));
        assert!(bad("upper 20 -2 0.5 lower", "upper 20 -2 0.6 lower").contains("sum"));
        assert!(bad("key 1 0 2 0 0 0 0 0 4", "key 1 0 2 0 0").contains("offsets"));
        assert!(bad("bones upper lower", "bones lower upper").contains("child"));
        assert!(bad("duration 1", "duration 0").contains("duration"));
    }
}
