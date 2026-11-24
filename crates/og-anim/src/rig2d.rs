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
