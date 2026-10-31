---
name: og-anim
description: Animation on Origin Engine — skinned glTF and .vox clips played by og.AnimationPlayer (cross-fades, stop motion on twos, root motion), blend trees and layered state machines as `type animgraph` text with masks and sync groups, IK chains, look-at, joint limits, foot locking, spring bones and their colliders, clip markers, retargeting at cook time and at run time across Mixamo, VRM, Unity, Rigify and Unreal rigs (fingers and twist bones included), mirroring and compression, 2D skeletons as `type rig2d` text (meshes, IK, skins, DragonBones import), and cutscenes as `type timeline` files with paths, camera rails, captions, sounds and events. Use when a character or prop must move by clips, when a clip plays wrong on another rig, or when a scripted sequence or cutscene is needed.
---

Animation on Origin Engine is components and text: a skeleton and a player
or a graph on the model's entity, procedural layers (IK, look-at, foot
locks, springs) on entities of their own, and the graphs, 2D rigs and
timelines as `.ogt` files the cook checks. Flipbooks (sprite sheets) are
og-2d §2; ragdolls and cloth are og-physics §2.

- §1 Skeletal animation: loading clips, the player, the graph, root
  motion, look-at and joint limits.
- §2 Foot locking and spring bones (and their colliders).
- §3 Clips by name, the third-person pattern, `.vox` clips and markers.
- §4 Retargeting at cook and run time, hands and twist bones, mirroring,
  compression, and the model's `animation` import block.
- §5 Blend trees: every line of a `type animgraph`.
- §6 2D skeletons: every line of a `type rig2d`, IK targets, skins.
- §7 Cutscenes: `type timeline` tracks, playing and seeking, paths.

## 1. Skeletal animation: players, graphs, root motion, look-at and joint limits

Skinned glTF models animate through `og.anim` (`og api show anim`):
`og.load.animation(world, "models/walker.glb")` answers the rig's skeleton
and clips (`og.load.gltfInfo(world, model).animations` lists the clip
names too); on the instantiated model's entity, `og.Skeleton { asset }`
poses it and `og.AnimationPlayer { clip, speed, loop, fade }` plays a clip
(changing `clip` cross-fades over `fade`; `stepFps` holds each pose for
1/stepFps seconds: 12 is stop motion on twos, 8 on threes, for clay and
puppet looks, while events and root motion stay smooth).
`og.AnimationGraph` runs a blend
tree with layered state machines registered by `og.anim.graph` (parameters
such as speed pick the motion; its grammar is §5), `og.IkChain`
turns a chain of bones to a target after the clips, `og.Blendshapes` weighs
morph targets, and `og.AnimationEvent` reports a clip's markers (below).
An `og.AnimationPlayer` beside a graph plays over it by its `weight`: a
strike, a bite or a fall at weight 1, back to 0 when `finished`.
**Root motion** is `og.Skeleton.rootMotion`: `planar` moves the entity's
Transform by the root joint's xz travel and yaw, `full` by all of it,
`off` (the default) leaves the hips travelling inside the model; switch it
to `planar` for a lunge's clip and back to `off` after.
**Look-at and joint limits.** `og.LookAt { skeleton, joint, target }`
on its own entity turns a joint (a head, an eye, a turret's barrel) so its
`axis` (default +Z, as three.js `lookAt`) points at `target` after the
clips and IK, its `upAxis` (+Y) rolling towards `up` (world +Y; zero
keeps the animated roll). `maxYaw` and `maxPitch` (radians from the
animated forward) hold a head from turning round, `clamped` reads back
when they did; `bones: 3` spreads the turn over neck, chest and spine;
`weight` fades it in and out. `og.JointLimit { skeleton, joint, axis,
swing, twistMin, twistMax }` (radians from the rest pose) limits what
`og.IkChain` and `og.LookAt` may turn a joint to: a knee is a hinge,
`axis` +X, `swing: 0.15`, `twistMin: 0`, `twistMax: 2.6`, so a
foot IK target never bends it backwards; a shoulder is a cone (`swing`).
The defaults limit nothing, and the clip's own pose is never clamped.

```ts
const look = must(world.spawn({ name: "hero-look" }));
must(world.insert(look, og.LookAt, { skeleton: body, joint: HEAD, bones: 2, maxYaw: 1.2, maxPitch: 0.5 }));
const knee = must(world.spawn({ name: "hero-knee-l" }));
must(world.insert(knee, og.JointLimit, { skeleton: body, joint: SHIN_L, axis: { x: 1, y: 0, z: 0 }, swing: 0.15, twistMin: 0, twistMax: 2.6 }));
// each tick, from a system that writes og.LookAt: world.set(look, og.LookAt, { target: playerPosition })
```

## 2. Foot locking and spring bones

**Foot locking.** `og.FootLock { skeleton, foot }` on its own entity, one
per foot, keeps a foot the clips plant (within 5 mm of its lowest in the
clip, the same test retargeting uses) where it touched the ground while
root motion carries the body on and clips cross-fade, bending the leg
(two-bone IK, `og.JointLimit`s respected); when the clips lift it the
foot rejoins its animation over `release` (0.15 s). `planted` and
`locked` read back each tick; `maxDistance` (0.5 m) lets go before a leg
would stretch. It needs clips whose feet actually rest: a foot that only
grazes its lowest for a frame is held for that frame only.

**Spring bones.** `og.SpringBone { skeleton, joint, bones }` on its own
entity, one per chain (a tail, ears, a ponytail, a single jiggling node),
swings `joint` and its `bones` − 1 parents after their parents' motion:
each joint's tail is a damped spring (`frequency` 3 Hz, `damping` 0.3;
1 settles without overshoot) towards where the animation puts it, under
`gravity` (`{ x: 0, y: -9.8, z: 0 }` droops it), held at the bone's
length. `space` world (default) lags behind the entity's motion; model
ignores it (a teleport does not whip the tail). Applied last, after IK
and look-at; `lag` reads back how far the chain trails. Deterministic and
saved with the world. `tipFrequency` and `tipDamping` (1) scale the last
joint's frequency and damping, the joints between in proportion: 0.25 is
a strand soft at its tip on a stiff root.

Keep hair out of the head with `og.SpringCollider { skeleton, joint,
shape, offset, tail, radius }` on its own entity, a sphere about `offset`
or a capsule from `offset` to `tail` in the joint's frame: every tail of
that skeleton's springs is pushed out of it to its `radius` plus the
spring's `radius`, keeping its bone's length. A collider's `layers` (1)
must meet the spring's `colliders` (all) to count.

```ts
const tail = must(world.spawn({ name: "hero-tail" }));
must(world.insert(tail, og.SpringBone, { skeleton: body, joint: TAIL_TIP, bones: 3, gravity: { x: 0, y: -4, z: 0 }, tipFrequency: 0.5, radius: 0.02 }));
const hips = must(world.spawn({ name: "hero-hips-collider" }));
must(world.insert(hips, og.SpringCollider, { skeleton: body, joint: HIPS, shape: "capsule", offset: { x: -0.15, y: 0, z: 0 }, tail: { x: 0.15, y: 0, z: 0 }, radius: 0.12 }));
```

## 3. Clips by name and markers

`templates/archetypes/third-person` (`src/anim.ts`: a 1D blend by speed,
a hero retargeted at cook time and sentinels at run time) is the pattern,
and for a 3D action game as a whole: a CharacterController over a level,
clips retargeted at cook and at run time, blend trees, a navmesh and
behaviour machines (og-ai).
`og.anim.clip` finds a clip by name, a model's as a flipbook's (og-2d §2):

```ts
const anim = must(og.load.animation(world, "models/slime.vox"));
const chase = og.anim.clip(anim, "chase"); // a ClipAsset; 0 when the model has none
world.insert(slime, og.Skeleton, { asset: anim.skeleton });
world.insert(slime, og.AnimationPlayer, { clip: chase, loop: true, stepFps: 12 });
```

A `.vox` with frames (og-game §5) is played the same way: its clips (`frames`, or
the `voxClip` names) swap the frames' models on the frame boundaries.

**Markers** are a clip's named times: the frame a strike lands, a
footstep. The cook reads them from the glTF, in each animation's extras
(`animations[i].extras."og.events"`, no other key):

```json
{ "name": "attack", "extras": { "og.events": [{ "name": "hit", "time": 0.42 }] } }
```

or from the model's import settings, `event "attack" "hit" 0.42` (clip,
name, seconds; the `animation` block below). A name is 1 to 64 bytes, a
time inside the clip and an `event` line's clip one of the glTF's own, or
the cook refuses it naming the clip; a library's clips bring the markers
the library carries (in its glTF or its own import settings).
`ClipInfo.markers` lists a clip's marker names in time order (an empty
list: the glTF carried none). On the tick playback passes one, through a
player or a graph, `og.AnimationEvent { entity, clip, name, time, weight }`
fires; a system that lists `og.AnimationEvent` in its `reads` reads them
with `world.events(og.AnimationEvent)`. Markers survive retargeting, at
cook time and at run time, so a library's `hit` lands on every character.

## 4. Retargeting, mirroring and compression

**Retargeting** — one clip on characters of other proportions and bone
names. Keep shared clips in an animation library (a glTF with a skeleton
and clips, no mesh). At cook time, a character's import settings
(`models/hero.glb.ogt`) pull them in. A model's `animation` block, every
line with its default:

```text
type import
version 1
---
animation {
  rate 30                              # resampling rate of every clip, 1 to 120 Hz (default: the source's key rate)
  event "attack" "hit" 0.42            # a marker on a clip of this glTF's own: clip, name, seconds (repeatable; or glTF extras."og.events")
  library "anims/humanoid-clips.glb"   # its clips land in this model's #anim, retargeted (repeatable)
  rig "rigs/hero.ogt"                  # how this model's bones are named (`type rig`); default: detected from the node names
  footIk true                          # retargeting solves the legs so planted feet stay planted (default true)
  mirror "turnLeft" "turnRight"        # also write a clip mirrored to the other side, under that name (default "<clip>.mirrored"; repeatable)
  compress false                       # quantised samples and keyframe reduction (default false: f32)
  tolerance 0.0001                     # the most compression may move a sampled value (default 0.0001, at most 0.1)
}
```

A clip the model has of its own wins over a library's of the same name.

At run time, `og.anim.retarget(world, { clip, source, target })` answers a
new clip for `target` (both skeletons from `og.load.animation`). The bones
are found from the node names (Mixamo, VRM, Unity, Rigify, Unreal, Blender);
when they are not, write a `type rig` .ogt (`scheme mixamo`, or
`hips "Root_Pelvis"` per bone), load it with `og.load.rig` and pass it as
`sourceRig`/`targetRig`. `og.anim.humanoid(world, { skeleton })` says which
bones were found. The target keeps its bone lengths, its hips travel the
source's distance scaled by the leg ratio (so root motion matches its
stride), and its legs are solved so planted feet stay on the ground
(`footIk`, on by default). The fx-showcase `retarget` station is the
pattern.

Hands retarget too: the fingers are humanoid bones (VRM 1.0's
`leftThumbMetacarpal` … `leftLittleDistal`, read from Mixamo's
`LeftHandIndex1`, VRoid's `J_Bip_L_Index1`, Rigify's `DEF-f_index.01.L`,
Unreal's `index_01_l`, plain `index.01.L`; Unity's legacy `Left Thumb
Proximal` is the metacarpal), each turning from its hand as
the source's does, so a fist or a grip holds on an A-posed arm whatever
the arm does. The target's twist bones (unmapped nodes named `twist`, such
as Unreal's `lowerarm_twist_01_l`, or Rigify's `DEF-forearm.L.001`) take a
share of their segment's roll by where they sit along it: none at the
segment's head, the next bone's at its end, so a forearm does not
candy-wrap. A clip made for the rig itself that leaves its twist bones
still gets the same shares each tick (after IK); a clip that keys them
is played as keyed. A character whose rest pose does not stand on the ground (an
armature lifted off it, a crouched rest) says how high its ankles stand
in its `type rig` file, `footHeight 0.08` (or `footHeight <left>
<right>`); planted feet then land on that ground, not at the rest pose's
foot height.

**Mirroring.** `og.anim.mirror(world, { clip, skeleton })` answers a new
clip that plays `clip` on the other side (a left-handed swing, a limp on
the other leg, a turn the other way): left and right nodes are paired by
the side tokens every naming scheme writes (`LeftArm`/`RightArm`,
`thigh.L`/`thigh.R`, `J_Bip_L_`/`J_Bip_R_`, `calf_l`/`calf_r`), each
node takes its counterpart's motion reflected across the skeleton's
x = 0 plane, root motion's sideways travel and turns flip. It assumes a
symmetric rest pose; mirror a retargeted clip on the skeleton it was
retargeted for.

**Compression** shrinks a model's clips in the bundle (not in memory: they
are rebuilt at load). In the same `animation` block, `compress true`
quantises rotations to 48 bits and the rest to 16 and drops the keyframes
interpolation rebuilds within `tolerance` (default 0.0001: about 0.1 mm
and 0.01°); a channel 16 bits cannot hold within it stays exact. Turn it on
for a game with many clips; leave it off while a clip's motion is still
being judged.

## 5. Blend trees: the graph grammar

A graph is `.ogt` text of `type animgraph` passed to `og.anim.graph(world,
{ text, clips })`; each `clip` line names a clip by `ClipInfo.name` among
`clips` (a model's `og.load.animation(...).clips`, or retargeted ones).
Motions (clips and blends of them) form a tree; layers run state machines
over them. Every line, with its default:

```text
type animgraph
version 1
---
param speed                 # a parameter: og.AnimationGraph.p0, the next p1, ... (at most 8)
param wave
param x
param y
clip idle "idle"            # clip <motion> "<clip name>"
clip walk "walk" {
  speed 1                   # duration multiplier, above 0 (default 1)
  loop true                 # false: holds its last frame (default true)
}
clip walkOther "walk" {
  mirror true               # plays the clip on the other side of the skeleton (default false): no og.anim.mirror needed
}
clip run "run"
clip waving "wave"
blend1d move speed {        # blend1d <motion> <param>: the two children around the value blend linearly, clamped at the ends
  child idle 0              # child <motion> <threshold>; any order (at least one)
  child walk 1.5
  child run 4
}
blend2d strafe x y {        # blend2d <motion> <param x> <param y>: children at points, gradient-band weights
  child walk 0 1            # child <motion> <x> <y>
  child idle 0 0
}
layer base {                # the first layer is the base: root motion and og.AnimationGraph.state come from it
  state still idle          # state <name> <motion>
  state move move {
    speed 1                 # playback rate, 0 or more (default 1)
    sync gait               # a sync group: states of one group play in step across layers (default none)
  }
  entry still               # the state it starts in (default: the first)
  transition still move {   # transition <from | any> <to>; in file order, the first that holds fires
    when speed gt 0.2       # when <param> gt | ge | lt | le | eq | ne <number>; every `when` must hold
    exit 0.9                # and the source has played this many cycles (a `when` or an `exit` is required)
    fade 0.25               # cross-fade seconds (default 0.2)
    interrupt any           # what may cut this fade: any (default) | none (it completes first) | source (only the state being left) | next (only the new state's own)
  }
  transition any still {    # any: from every state but the target
    when speed le 0.2
  }
}
layer upper {
  mode additive             # override (default): replaces the pose below | additive: adds each clip's difference from its first frame
  weight wave               # a number 0 to 1, or a parameter clamped to it (default 1)
  mask "spine" 0.5          # mask "<node>" [weight 0 to 1, default 1]: that node and every node under it (default: no mask, all 1)
  mask "arm.L"              # a deeper line wins for its branch; nodes under no line take 0
  state wave waving {
    sync gait
  }
}
```

Each state advances one phase by `dt × AnimationGraph.speed × state speed
/ duration`, the duration weighted across a blend's clips, so blended
clips stay in step. Per tick and layer the transitions are checked first,
then the phases advance, then the poses blend; a transition during a fade
replaces it unless its `interrupt` says otherwise (`none` for a
committed attack or landing, `source` to let an idle re-target a start
that has just begun). A layer's `mask` lines confine it to branches of the
skeleton (by node name, the nearest named ancestor's weight; an upper-body
layer is `mask "spine"`), so a wave or an aim plays over a walk without
freezing the legs. In a sync group the first layer (file order) whose
current state is in it leads: every later layer's state of that group
plays at the leader's phase, whatever its clip's length, and a transition
between two states of one group keeps the phase (a walk-to-run blend
keeps the feet in step). `og.anim.graph` refuses a parse error naming the line, a
clip name `clips` lacks, a parameter not declared and a motion that
contains itself. Write the parameters from a system every tick
(`world.set(body, og.AnimationGraph, { p0: speed })`); `state` and
`stateTime` read back the base layer's.

The third-person archetype's hero (`src/anim.ts`): idle, walk and run by
the speed across the ground, and the run held mid-stride, slowed, in the
air:

```text
type animgraph
version 1
---
param speed
param air
clip idle "idle"
clip walk "walk" {
  speed 1.1
}
clip run "run"
blend1d move speed {
  child idle 0
  child walk 2.2
  child run 5.6
}
layer base {
  state ground move
  state air run {
    speed 0.3
  }
  entry ground
  transition ground air {
    when air gt 0.5
    fade 0.15
  }
  transition air ground {
    when air lt 0.5
    fade 0.1
  }
}
```

```ts
const anim = must(og.load.animation(world, "models/strider.glb"));
const graph = must(og.anim.graph(world, { text: HERO_GRAPH, clips: anim.clips }));
world.insert(body, og.Skeleton, { asset: anim.skeleton });
world.insert(body, og.AnimationGraph, { asset: graph });
// every tick: world.set(body, og.AnimationGraph, { p0: groundSpeed, p1: airborne ? 1 : 0 });
```

## 6. 2D skeletons: the rig2d grammar

A cut-out character (bones, a cape that bends, feet that stay on a slope)
is an `.ogt` of `type rig2d` beside its PNG, loaded with
`og.load.rig2d(world, "rigs/hero.ogt")` (`og api show anim`). Pixels,
+y up, degrees counter-clockwise, `unit` pixels per world unit; bones
parent first, slots draw in file order:

```text
type rig2d
version 1
---
texture "hero.png"
filter nearest
unit 20
bone root {
}
bone hip {
  parent root
  at 0 22
  rotation 90
  length 16
}
slot torso {
  bone hip
  attachment torso
  color 1 1 1 1
  offset 0 0 0 0
}
region torso {
  slot torso
  rect 0 0 12 20
  at 8 0
  rotation -90
  size 12 20
}
mesh cape {
  slot cape
  vertex 0 32 hip 16 9 1
  vertex 6 40 hip 8 4 0.4 cape-1 7 0 0.6
  triangles 0 1 2
}
box hurt {
  slot torso
  polygon -6 0 6 0 6 20 -6 20
}
child flame {
  slot lantern
  armature fire
  clip burn
  at 15 0
}
ik leg {
  bones thigh shin
  target foot
  bend clockwise
}
armature fire {
  bone base {
  }
  # slots, regions, meshes, boxes, ik and clips, as at the top
}
clip idle {
  duration 2
  loop true
  bone hip {
    translate 0 0 0 curve 0.42 0 0.58 1
    translate 1 0 -1.5
    skew 0 0
  }
  slot torso {
    offset 0 0 0 0 0
    offset 1 0.2 0.1 0 0
  }
  deform cape {
    key 0 0 0 0 0
    key 1 0 -3 0 -6
  }
  order 1.5 cape torso
  ik leg {
    key 0 1 clockwise
    key 1 0.5 clockwise
  }
  event 0 "inhale"
}
```

`rect` is pixels from the image's top left; a mesh `vertex` is `u v` in
pixels then, per bone, `bone x y weight` in that bone's space (weights
summing to 1), or plain `x y` in the slot's bone. A bone's or region's
`skew` turns its y axis that many degrees past its x axis. A slot draws
`texel × color + offset`. Bone keys add to the setup pose (`rotate` and
`skew` degrees, `translate` pixels, `scale` multiplies); a `rotate` key
turns the shorter way to the next, or `rotate 0 0 clockwise 1` (or
`counterclockwise n`) that way through n more whole turns; every key eases
linearly to the next, holds with `step`, or follows `curve x1 y1 x2 y2`
(a cubic Bézier from 0,0 to 1,1, x time and y value; more segments as
`x y x1 y1 x2 y2`). A `deform` key offsets every influence in order, one
`x y` pair each; `order <time> <every slot>` redraws back to front from
then (alone, the file's order); an `ik` key sets the constraint's weight
and bend. An `ik`'s `bones` are a chain of any length, root first: one
bone points, two bend at the joint, three or more (a tail, a tentacle, a
three-joint leg) are solved by FABRIK and curl to the `bend` side. After the IK, a
`transform t { bones a; target b; mix 1 1 1 1 }` pulls bones towards
another bone's world rotation, origin, scale and shear by its mix, and an
`along s { path rail; bones s1 s2; position 0 percent; spacing 0 length;
rotate tangent 0; mix 1 1 }` lays bones along a `path` attachment (a
cubic spline, `vertex x y` three per knot, drawn as nothing) while its
slot shows it. A `skin armor { swap torso torso-armor; swap hat none }`
block shows other attachments of the same slots wherever the default ones
would show; `og.Skeleton2D.skin` picks it (`rig.skins.indexOf("armor")`,
`default` is 0). A `box` draws nothing: `og.anim.skeleton2dBoxes(world, hero)`
answers its polygon (in the Transform's frame) while its slot shows it,
for hit and hurt boxes a clip switches. A `child` shows an `armature`
block (a rig of its own on the same texture) in its slot; it plays the
parent's clip by name when it has one, else its `clip`, at the parent's
time, forwards or backwards; its events arrive as og.Skeleton2DEvent with
`armature` its name (empty for the skeleton's own clip). An `armature`
block takes `skin` blocks of its own, and wears the one named as the
skin the rig wears (else its default). A DragonBones 5.5 export is imported instead of written: a rig2d
`.ogt` with `dragonbones "hero_ske.json"` and `atlas "hero_tex.json"`
(skew, colour offsets, zOrder and IK timelines, armature displays,
bounding boxes, path displays and constraints, every skin, curves and `clockwise`
turns included; Spine's files are not read, convert
them to this text).

```ts
const rig = must(og.load.rig2d(world, "rigs/hero.ogt"));
const hero = must(world.spawn({ name: "hero", position: { x, y, z: 0 } }));
must(world.insert(hero, og.Skeleton2D, { rig: rig.rig, clip: og.anim.clip(rig, "idle"), layer: 1 }));
// A foot planted on the ground: one og.IkTarget2D per constraint, moved every tick.
const foot = must(world.spawn({ name: "hero-foot" }));
must(world.insert(foot, og.IkTarget2D, { skeleton: hero, constraint: rig.constraints.indexOf("leg"), target: { x, y: groundY } }));
```

`og.Skeleton2D` plays `clip` at `speed` (`time` is written each tick;
setting it seeks), wraps when `loop`, mirrors with `flipX` and draws at
sprite `layer`, lit by `og.Lighting2D` like a sprite there. Clip events
arrive as `og.Skeleton2DEvent` (`entity`, `clip`, `name`); an
`og.IkTarget2D` writes `reached` and `error` (its `weight` multiplies the
constraint's own). The pose is a function of the component, the
Transform and the targets, so a snapshot carries only the playback
position and the boxes. `templates/game-2d`'s lamp keeper
(`assets/rigs/keeper.ogt`, `spawnKeeper`, `Walk`, `Plant`) shows all of
it. A walk is the walk clip played while a system moves the entity, with
the feet's targets stepping in time: each planted for half the cycle
(sliding back as fast as the body goes), then swung a stride ahead, so
the feet stay on the ground the IK finds.

## 7. Cutscenes: timelines and paths

A cutscene, an intro, a scripted beat or a fly-through is a `type timeline`
file, not code (`og api show timeline`, `og api show path`;
`templates/game`'s `assets/timelines/intro.ogt` and `tests/intro.og.ts`
are the pattern). Tracks of keys at seconds: between two keys the later
key's `ease` (og.tween's names: `cubicInOut`, `quadIn`, ...) mixes them,
outside them the value is held. Every value is evaluated at the
timeline's time, never accumulated, so a seek lands exactly where playing
would (a playtest or a video can start at any second).

```text
type timeline
version 1
---
name "Intro"
duration 10
path "flyby" {
  kind catmullRom          # or bezier (p0 c0 c1 p1 ...) or linear; closed true wraps
  point -5 3.2 9
  point -1.5 2.4 7.5
  point 2 2 7
  point 5 2.8 8.5
}
track camera {
  rail "flyby"             # or position/target keys, or follow "<entity>"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 9.5 {
    at 1
    target 1.5 1 0
    fov 42
    ease cubicInOut
  }
}
track transform "trickster" {
  key 2 {
    position 2 0 0
  }
  key 5 {
    position 0.5 0 1.5
    rotation 0 90 0
    ease cubicInOut
  }
}
track animation "trickster" {     # the entity's og.Model's clips (or `model "<glb>"`)
  key 0 {
    clip "idle"
    loop true
  }
  key 2 {
    clip "walk"
    loop true
    blendIn 0.3
  }
}
track caption {
  captions "narration.srt"   # beside the timeline; or word timestamps .json
  style lg                   # a UI type step
  anchor bottom
}
track audio {
  key 1 {
    sound "audio/blip.wav"
    volume 0.6
    bus sfx
  }
}
track music {
  key 0 {
    sound "audio/theme.ogg"
    fadeIn 1
    fadeOut 2
    end 9
  }
}
track event {
  key 5.5 {
    do "spawn_wave:2"
    retroactive true
  }
}
track transition {
  key 0 {   # cut, fade, wipe, dissolve, iris, slide
    kind fade
    mode in
    duration 1
  }
  key 9 {
    kind fade
    mode out
    duration 1
  }
}
track marker {
  key 5.5 {
    name "drop"
  }
}
```

The cook checks
the sounds and models it names and imports the captions, so an `.srt`
edit recooks the timeline. Play it from code or from the command channel:

```ts
const intro = must(og.load.timeline(world, "timelines/intro.ogt"));
const e = world.spawn({ name: "intro" });
world.insert(e, og.Timeline, { asset: intro });
og.timeline.play(world, e);         // pause, seek(world, e, 4), stop
// a system reading og.TimelineMarker (name "drop") or og.TimelineEnded reacts to it
```

`og run . --commands "0:timeline:play:intro;4:screenshot;8:screenshot"`
plays `timelines/intro.ogt` on an entity named `timeline-intro`;
`timeline:seek:4`, `timeline:pause`, `timeline:resume`, `timeline:stop`
take an optional `:<name>`. While a timeline with a camera track plays
(or is paused) it owns `og.Camera`; it hands back the camera it found
when it stops or ends. Event keys run their action at the start of the
next tick; a seek skips the events it passes unless they are
`retroactive`, and markers are sent only by playing past them. A capture
at `t:` shows the world after the ticks before t: the play's first tick
is the timeline's time 0, so the frame at `4:` shows 3.983 s at 60 Hz.

Paths on their own: `og.path.create(world, { points, kind, closed,
tension })` or `og.path.find(world, "<name>")` for one a scene declares
(the same `path` block in a `type scene`); `og.path.sample(world, p, t)`
gives position, tangent and up at arc-length t 0..1, `length` and
`nearest` the rest. `og.PathFollower { path, speed, loop, align, offset }`
moves an entity along one each tick (`distance` is its own field, so a
snapshot keeps it); `og.CameraRail { path }` rides one instead of its
inline points.
