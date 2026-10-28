---
name: component-forge
description: Producing a deliberately procedural entity, screen element or effect for an Origin Engine game from measured references — choosing between finished assets, generated assets and building it as code, decomposing it into named parts, measuring the reference instead of eyeballing it, gating on the executed result (silhouette inside the outline, flat colours, no intersections, chirality, tapers), and assembling it so it spawns by name and tunes live. Use when a unit must be parametric (dialled under Tune, repeated at scale) or when nothing finished carries the chosen direction.
---

A game is built out of units that stand on their own: an entity, a screen,
an effect, a system. This is how one is produced when the reference is a
picture and the output has to be code whose numbers someone can dial.

The 3D build itself (the generators, the eight passes, the review frames
and `og compare`) is og-model's; the generators' API is og-game §18; a
screen is og-ui's. This skill is the decision to forge, and the discipline
that holds for every forged unit, 2D or 3D, model or screen.

## 1. Choose the route before you build

Where the game's plan already names a route for the unit, that route wins;
this skill is the procedural craft, not a reason to replace specified
generated or imported work. Where the route is open, compare:

1. **Finished assets first.** A licensed model, texture, material or HDRI
   (CC0 or a licence the game may ship) is finished work someone else
   already paid for. Import it under `assets/` (og-game §5), reference it
   by relative path and keep its licence and source beside it.
2. **Generate.** An asset made by a generative tool you have, through that
   tool's own interface, requested with the art brief (og-taste §1: the
   look compiled into the prompt, references and parameters for one asset)
   and cooked under the look's contract (og-build §8). A route the game's
   owner asked for is kept; saving effort is not permission to replace the
   outcome they chose.
3. **Forge third**, and deliberately: build the unit as code when it must
   be *parametric*: when someone will move its proportions under Tune,
   when it repeats at scale as instances, or when nothing finished carries
   the direction the game settled on.

Say which route you took and why. A hero silhouette forged out of
primitives because nobody looked for a finished asset reads as primitives,
and players will see it before you do.

## 2. Decompose before you build

Name the parts and their relationships first, in words: what is a body,
what is an appendage, what repeats, what attaches to what and where. A
unit built as one mesh cannot be animated, cannot be tuned and cannot be
reviewed part by part.

Write the decomposition once, in the unit's spec (og-model §1 keeps it in
`review/<model>/spec.md`), and refer to it from the module; never keep a
second, competing description beside the source. A part you did not name
is a part no gate can check.

## 3. Measure the reference; do not eyeball it

Take proportions off the reference instead of guessing them: relative
lengths, widths at named stations, where a feature sits along an axis.
Convert them into the unit's parameters. The point of forging is that the
numbers came from somewhere, and a number you invented has no defence when
someone says it looks wrong.

Where the reference cannot answer (a hidden side, an occluded joint),
mirror what is visible and **say that it is a mirror**, or leave it out.
Inventing an unseen side with confidence is the failure this route is most
prone to. Inferred is not measured: keep the two apart in what you report.

## 4. The route on Origin Engine

- **3D**: the parts are generators (lathe, extrude, sweep, csg, merge, the
  polyhedra) built in a factory function, one entity per part with
  `og.Mesh` and `og.Transform` under one root (og.Parent), each named as
  the spec names it; og-model §4 and §5 are the passes and the recipes.
  Materials go by role (`og.render.standardMaterial` once per role, shared
  by every part that has it), and repeats share their geometry and
  material so the engine instances them. A unit whose parts depend on
  nothing the game computes becomes a `type model` asset once its passes
  close (og-model §5). Build in the look's shape language from the first
  pass (og-model §5's recipe for faceted, voxel, clay, paper or wireframe
  styles), under its `ArtShape` budget, not restyled at the end.
- **The look's contract** holds a forged unit as it holds an asset: its
  colours come from the look's palette (`ArtPalette`), a 2D unit's parts
  stand on its pixel grid (`ArtPixels`), and a material's outline follows
  `ArtLine`. Read the numbers from the art brief of the unit's kind
  (og-taste §1) and put them in the spec.
- **2D**: the same rule draws flat parts as sprites or quads in the play
  camera, one entity per part; draw order is checked, not assumed
  (og-feel §10).
- **Parametric**: what is dialled is what the parts' transforms read. A
  limb's length is its part's scale and a stance is the parts' offsets,
  both computed in a system from `tunable` fields every tick, so the unit
  moves under Tune while it stands.
- **Chirality** is one generator with a sign (og-model §5's `mirror: "x"`,
  a scale of -1), in the factory and in the system that places the parts.

## 5. Gate on the executed result, never on the code you wrote

Run the unit, render it from the shipping camera and judge the frame
(visual-acceptance). The gates that matter here:

- **Silhouette agreement, measured inside the outline.** An outline-only
  comparison is nearly blind: a model with its whole face missing scores
  about the same as the finished one, because the face is not on the
  outline. Compare the interior too (`og compare` with a region `--mask`,
  per region), or the measurement approves work that is not there.
- **Flat regions keep their colour.** A region the reference holds as one
  flat colour comes back as one flat colour, within a stated ΔE00, and on
  the look's palette when it has one.
  Gradient creep across a region that should be flat is the usual sign a
  material was chosen by feel.
- **No self-intersection.** Parts that pass through each other read as one
  broken shape from most angles and cannot be rigged; check the side, top
  and orbit views, not only the front. In 2D the same fault is a part
  whose quad hides another.
- **Chirality is code, not luck.** Left and right come from the same
  generator with a sign and are checked as a pair. A hand-placed mirror
  drifts, and a unit whose two sides disagree looks wrong without anyone
  being able to say why. A 2D sprite's facing is one scale sign on its
  transform, never a second drawing.
- **A taper must actually taper.** A sweep whose stations all end at the
  same radius is a valid sweep and a wrong shape. Read the generated
  geometry back (`og.render.geometryInfo`, the bounds of each part) at
  each end instead of trusting the parameters you passed.

Record every refuted attempt with the project's notes. A gate that failed
and was then loosened is the one thing that must never happen quietly.

## 6. Assemble through the seams that already exist

A produced unit is finished when it is *assembled*, not when it renders:

- its parts are entities with stable names, spawned from the scene or the
  game's `start`, so the probe can pick, read and move them;
- every tunable number it has is a `tunable` field, so the probe's
  `bounds` lists it and Tune moves it (architecture-discipline §5);
- it stores what the dice decided (the seeded `Rng`'s draws) and resolves
  its dimensions every tick, so it can be dialled while it stands;
- a screen is the game's UI, separate from the world it draws over;
- `src/main.ts` assembles and nothing more.

A unit that cannot be spawned by name, tuned by parameter or reviewed from
the view that ships is not done, however good the still looks.

## 7. Say what the route cannot do

One image cannot reveal a hidden side, and a stylised reconstruction is not
a likeness. Hard-surface objects come out strongest; characters come out
stylised. **"This cannot reach the asked-for fidelity from this
reference"** is a complete and acceptable answer: give it early, with what
would be needed instead, rather than spending ten correction rounds
arriving at it.
