<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `animation` guide: Animation and og.motion

Motion in an Origin Engine game. Clips and animation graphs, og.motion's generated humanoid motion and why it is presentation only, tweens and timelines, and the timings that make actions read.

The API: `og api show anim` (clips, graphs, IK, retargeting, 2D
skeletons, flipbooks), `og api show motion` (og.motion),
`og api show path`, `og api show timeline` and `og api show tween`.

<a id="guide-animation-1"></a>
### `animation §1` Choosing the tool

| Need | Use |
|---|---|
| A humanoid walking, running, standing, gesturing between goals | og.motion, with a clip fallback |
| Authored character actions (attack, hit, death) | clips on og.AnimationPlayer or og.AnimationGraph |
| A 2D character | a flipbook sheet (from an atlas, or from generated GIF, WebP or APNG `source` files), or a 2D skeleton for smooth limbs |
| A value over time (a door, a UI panel, a pickup bob) | a tween |
| A scripted sequence (cutscene, intro) | a timeline |

<a id="guide-animation-2"></a>
### `animation §2` og.motion

og.motion generates humanoid motion on the GPU between goals the game
queues (walk to a point, turn, a library pose, a jump).

- **Presentation only.** The generated pose is drawn, never simulated:
  no system may read it, and no gameplay may wait on it. og.Transform is
  the character's root; with `MotionAgent.drive` the engine moves it to
  the goals with fixed rules inside the tick, otherwise the game moves
  it and the motion follows.
- **Branch on og.motion.status, never on og.motion.presentation.** Status
  answers which goal is current by fixed rules and is the same on every
  machine; presentation reports visual readiness for a host's cover and
  differs between GPUs.
- **Always give a fallback.** Until the weights are resident, or on a
  device without the backend (`og.motion.capabilities` answers backend
  none), the agent is drawn by its og.AnimationPlayer or
  og.AnimationGraph: give every motion agent at least an idle and a walk
  clip, or it stands in its bind pose.
- The weights (35 MB) are cooked only for a game that uses og.motion and
  load in the background after the boot bundle; do not add og.motion to
  a small game for one character's idle.
- Generation runs in GPU slices of about 3 ms a frame for all agents at
  once. A crowd of motion agents waits longer for each goal's motion; keep
  og.motion for the characters the player watches and clips for the rest.

### 2.1 Feet on the ground

The drawn feet are judged by where they are drawn, never by a contact
signal: `dump_motion`'s `footing` measures every shown frame. A foot is on
the ground when its ankle is within 3 cm of its rest height over the
Transform; a foot on the ground in two frames running is a sample of its
speed across the ground, counted by what the agent is doing (`walk`, `run`,
`turn`, `takeoff`, `landing`, `hold`; `air` has none). In a held pose
(`hold`), a frame floats when a foot has stayed over those 3 cm without
moving across for 0.1 s (longer than a step's rise; a stepping foot is
lifted by its step).

What a motion agent should show over a course of walks, a run, jumps and
held poses (the `motion-feet` fixture holds the engine to it):

| Measure | Skating looks like | Hold it to |
|---|---|---|
| samples faster than 0.1 m/s | a foot creeping | under 5% (the engine shows 0.2–0.5%) |
| samples faster than 0.3 m/s outside a takeoff | a foot gliding | none |
| floating frames of a held pose | a foot hovering | none |

The engine gets there without the model's help: a walk or run is played
at the speed the simulation moves the Transform (stride warping in time),
the drawn hips follow the generated hips' travel within 0.15 m of the
Transform, a foot is planted where the pose shows it low and slow (in a
held pose wherever it is under 0.35 m) and put down on the ground, a
planted foot left behind, let go or out of the leg's reach steps (lifted
7 cm on its arc, one foot at a time), a swing over the ground is lifted
clear of it (a stepping foot too, however it comes to move across), and
the hips lower for a planted foot out of reach. A generated jump that
lands and takes off again unasked is cut at its landing's lowest point
and the model starts again from there. Queue walks with a speed for each
gait:
a walkTo gains speed at its `acceleration` and carries it into the next
walk or jump, so the generated stride has a start and a run to follow.

<a id="guide-animation-3"></a>
### `animation §3` Clips and graphs

- A graph's parameters and IK targets are set from game systems in the
  tick. Base gameplay on the tick's simulation pose, never the drawn pose.
  Animation runs after game systems, so the pose they read is one tick
  behind their new inputs; allow for that latency when designing grabs
  and hit detection.
- Query with og.anim.getNodeMatrix when no attached entity is needed;
  use a BoneAttachment socket for a prop. Resolve names at load: node
  indices and a skin's joint indices are different; Animation.skinJoints
  bridges them. Read `og api show anim` for pose and frame semantics.
- og.Skeleton's `rootMotion` decides where a clip's travel goes. `off`
  (the default) holds the root joint's xz travel at its rest place: a run
  or a swing on a rope plays on the spot, and the game moves the entity.
  `planar` and `full` move og.Transform by the travel instead. Never
  overwrite og.Transform each tick to cancel `full`; use `off`.
- Retarget a clip library to the game's rig once, at load (og.load.rig
  and og.anim.retarget), never per frame.
- Hitboxes and damage windows come from the tick: mark them with clip
  events or tick counts, not with how far the drawn animation has played.
- During a cross-fade or a blend every playing clip fires its markers.
  Each og.AnimationEvent says which graph `state` and `layer` played it
  and whether its clip was `dominant` (weighed most of its layer that
  tick: in a cross-fade the state left until the fade passes half, then
  the state entered; in a player's fade its current clip from half on).
  Act on the markers your rule names: a hit window that only the
  attack being entered may open reads `state`; one the heavier attack
  owns reads `dominant`; footsteps that should sound from both clips of a
  walk-to-run fade read neither, and scale the volume by `weight`.

<a id="guide-animation-4"></a>
### `animation §4` Timing that reads

The thresholds actions are held to:

- **Input to visible response** within 100 ms (6 ticks at 60 Hz). A jump
  or attack whose wind-up delays the first visible change past that feels
  unresponsive; start the motion on the press and put the anticipation in
  a squash or pose change.
- **Anticipation, action, recovery**: an attack's anticipation 3 to 8
  ticks, the hit on a single tick, recovery 8 to 20 ticks; the player may
  cancel recovery into movement after about half of it.
- **Hit stop** 2 to 6 ticks for a normal hit and 8 to 12 for a heavy or
  finishing one; freeze presentation, keep the tick's clock rules
  (determinism §1).
- **Camera shake** on hits, decaying within 150 to 300 ms, a few
  centimetres or pixels, never on every frame of continuous fire.
- **UI motion** 150 to 300 ms for panels and screens, 80 to 150 ms for
  button feedback; entrances ease out, exits ease in and are shorter. The
  reduced-motion setting replaces movement with a fade; a transition a
  screen writes under `@media (prefers-reduced-motion: reduce)` plays as
  written instead, so keep motion that carries meaning small there.
- **Coyote time** about 100 ms and a jump buffer of about 100 ms for a
  platformer; a jump without them feels broken at any polish.
- Ease every tween; linear motion reads as mechanical except for
  constant-speed things (a conveyor, a projectile).

<a id="guide-animation-5"></a>
### `animation §5` GSAP names, engine targets and capture

Use `og.tween.to`, `og.tween.from` and `og.tween.fromTo` with an entity,
component and numeric field, not a live JavaScript object or DOM selector.
`repeat` counts extra cycles: 0 plays once, 1 twice, -1 forever. `yoyo`
reverses alternate cycles. Eases use GSAP names: `none`, `power1.out`,
`power2.inOut`, `back.out`, `expo.in`, `elastic.out`, `bounce.out`.
The named curves follow GSAP 3.15.0 maths in deterministic f32 arithmetic;
there are no custom JS ease functions or callbacks in saved world state.
The same `Ease` string accepts `cubic-bezier(0.17,0.85,0.63,0.08)` and
`steps(4,jump-start)`. The shared parser validates four finite Bézier
controls with x in [0,1], or a positive integer step count and known jump
position (jump-none needs at least 2). Invalid tween text returns an error
value naming the text and constraints; invalid timeline text fails the cook.
Camera `blendEase`, `soloEase` and `blendCurve` accept the same timing values.
An `Ease` field occupies 20 reflected bytes: a tag plus four payload words,
including all Bézier controls or the step count and jump position. Its
parameters are world state: hashed, snapshotted and restored on both hosts.

UI transitions take their curve from the element's `ease-*` class:
`ease-linear`, `ease-in`, `ease-out`, `ease-in-out`,
`ease-[cubic-bezier(x1,y1,x2,y2)]` or `ease-spring-[stiffness,damping]`.
An element without one runs on CSS's `ease`. The old UI curves `out`,
`exit` and `inOut` were custom cubic Béziers, not GSAP curves: preserve
them with `ease-[cubic-bezier(0.23,1,0.32,1)]`,
`ease-[cubic-bezier(0.4,0,1,1)]` and `ease-[cubic-bezier(0.65,0,0.35,1)]`,
respectively. Style roles such as `enter` and `exit` select curves; they
are not alternative curve names. A style's steps stay stepped in UI motion.
CSS accepts CSS keywords and timing functions, not GSAP strings. Its
keywords use the exact standard Béziers, not similarly named GSAP eases.

Mass-element keys also take `Ease`; `defineMass` returns a result and rejects
invalid timing text. Replace `linear`, `out`, `in`, `back` and `step` with
`none`, `power1.out`, `power1.in`, `back.out` and `steps(1,end)`.
Those curves have shader opcodes. Other curves use a deduplicated 4,097-sample
CPU-baked lookup table. Continuous curves interpolate; steps quantize time
before lookup so jumps never become ramps. Metal readback's maximum error
over the tested named curves is 0.000187039, the asymmetric Bézier
`cubic-bezier(0.17,0.85,0.63,0.08)` is within 0.000000119, and
`steps(4,start)` matches exactly. These are measured sample-sweep errors,
not universal bounds for arbitrary Bézier controls. The GPU approximation
is presentation-only; simulation uses the shared CPU evaluator.

Capture is explicit. `to` without `from` reads the starting field after
its delay. `from` captures its destination at the call, before the delay;
`fromTo` requires both endpoints. An explicit start is written on the
first `og.tween` tick and held through the delay, not synchronously at
the call as GSAP's immediateRender can do. A system calling `from` must
declare read access to the target component. Captured spans survive
snapshot/restore and repeats; a later write cannot change the destination.

<a id="guide-animation-6"></a>
### `animation §6` GSAP timelines

`og.tween.timeline(world, vars)` answers a timeline whose builders chain as
GSAP's do. Targets are entities, and each call tweens one numeric field
(`field` and `to`, as `og.tween.to`):

```ts
const tl = og.tween.timeline(world, { defaults: { duration: 0.4, ease: "power2.out" }, onComplete: "intro.done" });
tl.from(title, { component: og.Sprite, field: "opacity", from: [0] })
  .to(cards, { field: "position.y", to: [0], stagger: { each: 0.06, from: "center" } }, "<0.1")
  .addLabel("settled")
  .call("sfx.whoosh", "settled-=0.2")
  .to(door, { field: "position.x", keyframes: [{ to: [1] }, { to: [3], ease: "back.out" }] }, "settled");
```

- Position is GSAP's: seconds, `"<"` and `">"` (the start and end of what
  was added last), `"+=1"` and `"-=0.5"` (from the end), a label or
  `"label+=0.2"`, `"<50%"` (half way through the last addition) and
  `"-=25%"` (a quarter of what is inserted). None appends.
- `stagger` is seconds or `{ each | amount, from, grid, ease }`; `from` is
  `start`, `center`, `end`, `edges`, `random` or an index. `random` is a
  seeded shuffle of the world's og.Rng, so replays agree.
- `keyframes` replace `to` with steps; repeat and yoyo apply to the whole.
- Controls: `play`, `pause`, `resume`, `reverse`, `restart`, `seek`,
  `progress`, `timeScale`, `time`, `duration`, `isActive`, `kill`. They move
  the playhead; the values are written in the next `og.tween` run.
- Callbacks are event names, not functions: `onStart`, `onUpdate`,
  `onComplete`, `onRepeat` and `.call(name)` send og.TweenEvent, which a
  system reads in the next tick. A seek sends none.
- A timeline renders from its playhead: reverse writes every field back to
  its start, and a seek reaches the values playing there would. A child
  is written only when its time changes, so a finished tween hands its
  field back to the game.
- A refused builder call adds nothing and its reason is `tl.error`; the
  call still answers `tl`. A completed timeline stays until `kill()`
  (or `autoKill: true`), so it can be reversed or restarted.
- `tl.add(handle)` nests an `og.tween.to`, a sequence, a parallel group
  or another timeline; it stops running on its own.

The cutscene timeline stays a multi-track asset, not a GSAP root timeline:
camera, captions, audio, animation, events and nested tracks retain their
ownership and seek rules. It shares the engine's easing curves, but keeps
absolute-time evaluation; a segment uses the later key's ease and events
are not retroactive on seek unless authored that way.

<a id="guide-animation-7"></a>
### `animation §7` Rigs, retargeting, mirroring and crowds

- A model's glTF skins and clips are cooked into its `<model>#anim` blob,
  which og.load.animation loads.
- The cook probes skinned triangles in bind-joint stress poses (humanoids:
  both arms forward and a stride; other rigs: topology-derived branches).
  A `skin stretch` warning names source triangle IDs whose longest edge
  exceeds both 2.5 times its bind length and 0.08 model units. To explicitly
  repair those regions, set `model { relaxSkin true }` in the model's import
  sidecar (`<model>.glb.ogt`, `type import`, version 1). It defaults to false.
  Probes and position welding use model units after the skin bind transform,
  including quantised meshes' dequantisation. The cook welds positions to
  five decimal places for weight averaging, blends each flagged region with
  its neighbours 16 times per pass, widens it each pass, and retains four
  normalized influences with joint-index tie breaks. It stops at zero probe
  stretch or 12 passes, keeps the best result that worsens no probe pose,
  and reports changed source vertices and remaining stretched triangles.
  Only affected regions change;
  positions, UV seams, clips and the source file stay untouched. Inspect
  the character in its real animations too: stress probes are not animations
  and zero reported stretch does not guarantee a good deformation.
- Clips bind to a skeleton's nodes by name, so a clip of one rig plays on
  another with the same node names. A rig with other names or proportions
  plays it retargeted: at cook time with a model's `animation { library }`
  import setting (the usual way, for a library known ahead), at run time
  with og.anim.retarget for clips chosen while playing.
- A source that faces another way than the target (a rig authored
  facing +X or -Z) is turned first: by the quarter turn about +Y that
  best carries its mapped rest joints onto the target's, each about its
  centroid seen from above. A cook note says how far a clip was turned;
  a skeleton too narrow to tell (a spine alone) is not turned.
- Retargeting corrects each rig's rest pose first, so a T-pose drives an
  A-pose and differing bone frames agree; fingers turn from their hand, so
  a hand pose holds whatever the arm does. Name `footHeight` on each rig
  when the feet do not land on the ground of the rest pose.
- Twist bones (unmapped nodes on a limb named `twist`, or Rigify's
  `DEF-….001`) take a share of the limb's twist by their position along
  it.
- A mirrored clip known ahead needs no call: a graph's `clip` line with
  `mirror true` plays it mirrored, and the cook's
  `animation { mirror "<clip>" }` writes the mirrored clip into the model.
  og.anim.mirror is for clips chosen at run time; markers keep their
  names.
- IK targets in `world` space are in the frame the skeleton entity's
  og.Transform is written in: a body under a character capsule takes
  targets relative to the capsule. Transform world points by the parent's
  inverse yourself.
- og.CrowdAnimation draws thousands of walkers at no CPU animation cost.
  Give each member its own `offset` to keep them out of step. Use it for
  background characters only: nothing reads its pose and a clip change
  cuts. Members are culled and drawn on the GPU (RenderCapabilities.gpuCrowds), each at
  the model's cooked level of detail for its distance: keep the model's
  `lods` (the default cooks three), since a crowd of full meshes costs
  several times as much far away.

<a id="guide-animation-8"></a>
### `animation §8` Procedural bones: foot locks and springs

- og.FootLock keeps a planted foot still through blends and root motion.
  A foot counts as planted within 5 mm of its lowest in each clip (the
  notion retargeting uses), weighed across blends and cross-fades: it is
  planted while the planting clips hold at least half the pose. Put each
  lock on its own entity, one per foot; locks apply in query order, and
  og.JointLimit clamps the leg.
- A lock point stays in world space, including when a Parent moves.
  Set groundReach to raycast the ground, aligning the ankle to its normal;
  pelvisDrop permits lowering the pelvis to reach it. maxDistance releases
  an old anchor immediately on a teleport rather than pulling the leg back.
- og.SpringBone swings a tail, an ear, a strand of hair or a single
  jiggling node after the body's motion; put each chain on its own entity.
  It is deterministic under the fixed tick, and a restore continues a
  swing exactly.
- Author stiffness and drag independently: zero stiffness still permits
  gravity, wind and damping. tailOffset gives a leaf an explicit local tail.
  World-space chains include all Parent transforms; teleportDistance resets
  a displaced root, and maxSpeed caps tail speed. wind couples to the wind
  field; worldColliders selects optional physics collider layers.
- og.SpringCollider keeps those chains out of the body: a sphere or
  capsule on the head, the chest or an upper arm, or a plane (tail is its
  normal). Collision tests the full bone segment, not just its tip. Collider
  radii scale in the joint's frame. One entity per shape.
- Extracted motion appears in Skeleton.rootDelta and rootYawDelta.
  For a colliding character choose rootMotionMode controller and supply
  a CharacterController and Collider: the controller sweeps the displacement
  instead of animation moving the Transform through obstacles.
- og.Hang hangs a character from a rope. og.anim places the skeleton's
  og.Transform so the `grip` joint (the hand) sits at `point`, its +Y runs
  from the grip towards `anchor` by `lay`, and its front turns towards
  `facing` (0 -1 0 lays the chest to the ground). Each of its `limbs` (an
  og.IkChain on the free hand or a foot) reaches for the rope `reach`
  metres from the grip, upward above 0. Copy the rope in each tick: for an
  og.Rope, its first and last og.physics.ropePoints. For a physics swing
  (og.Joint of kind rope), copy the joint's two anchors.
- Play the swing clip with `rootMotion: "off"`: the clip poses the limbs,
  the hang places the body, and nothing else moves the Transform. Ease
  `lay` from 0 to 1 as he takes the rope and back to 0 before he lands, so
  he lands on his feet. Ease `weight` to blend the hang in or out of the
  entity's own place.
- The order each tick: root motion, hangs, foot locks, og.IkChain,
  og.LookAt, then spring bones top joint first.

A rope swinging from a beam, its end the hang's grip point. The game eases
the body onto the rope over its first second; og.anim places a skeleton
that carries the hang (none here, so the recipe checks what it feeds in).

```ts recipe ticks=150
import { defineSystem, og } from "@origingame/engine";
import type { Result, Vector3 } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const BEAM: Vector3 = { x: 0, y: 8, z: 0 };

// Each tick the hang takes the rope as the last physics step left it.
const Swing = defineSystem("Swing", { reads: [og.Rope], writes: [og.Hang] }, (_q, time, world) => {
  const rope = world.find("rope")!;
  const hang = world.find("hang")!;
  // Refused until the rope's first physics step.
  const got = og.physics.ropePoints(world, rope);
  if (!got.ok || got.value.length === 0) return;
  const points = got.value;
  const h = must(world.get(hang, og.Hang));
  const lay = Math.min(1, h.lay + time.dt);
  must(world.set(hang, og.Hang, { ...h, anchor: points[0]!, point: points[points.length - 1]!, lay }));
});

const Check = defineSystem("Check", { reads: [og.Hang], writes: [] }, (_q, _time, world) => {
  const h = must(world.get(world.find("hang")!, og.Hang));
  if (h.lay === 0) return;
  const dx = h.point.x - h.anchor.x, dy = h.point.y - h.anchor.y;
  og.assert.always(Math.abs(Math.hypot(dx, dy) - 4) < 0.3, "the grip is the rope's end, a rope's length from the beam", { dx, dy });
  og.assert.sometimes(h.point.y < 4.5, "the rope swings the grip down under the beam", {});
  if (h.lay === 1) og.assert.reachable("he lies along the rope", {});
});

export default og.game({
  systems: [Swing, Check],
  start(world) {
    // The rope starts out level, so it swings down and back.
    const rope = must(world.spawn({ name: "rope", position: BEAM }));
    must(world.insert(rope, og.Rope, { length: 4, segments: 8, damping: 0 }));
    const hang = must(world.spawn({ name: "hang" }));
    // On a character: skeleton (the og.Skeleton entity), grip (its hand joint), limbs and reach.
    must(world.insert(hang, og.Hang, { anchor: BEAM, point: { x: 4, y: 8, z: 0 }, facing: { x: 0, y: -1, z: 0 }, lay: 0 }));
  },
});
```

<a id="guide-animation-9"></a>
### `animation §9` 2D animation: flipbooks and skeletons

- og.Flipbook plays a sheet's clip on the entity's og.Sprite; the sprite
  keeps its size, colour, flips and layer. It runs on scaled time, so slow
  motion and hit stop slow it.
- og.Skeleton2D draws a rig's region attachments as quads and its mesh
  attachments weighted to bones and deformed by the clip, lit by
  og.Lighting2D when its layer is in the lights' range. A skin swaps
  attachments: find one by name with `rig.skins.indexOf("armor")`.
- One sprite graph (GraphDesc.sprite) serves every sheet or rig whose
  clips have the names its `clip` lines use. On a flipbook its transitions
  cut, never fade, and wait for the frame on screen to end, by the sheet's
  clip names. On a 2D skeleton it drives Skeleton2D's `clip`, `blendClip`,
  their times and `blend`, and fires both clips' events. Graph weight is
  not used on sprites; the machines are world state (the flipbook and 2D skeleton
  state blocks).

<a id="guide-animation-10"></a>
### `animation §10` og.motion goals, drive and backends

The shortest flow:

```ts
world.insert(hero, og.MotionAgent, { drive: true });
og.motion.walkTo(world, hero, { position: { x: 4, y: 0, z: 2 }, speed: 1.4 });
og.motion.jump(world, hero, { height: 0.5, distance: 1.5 });
og.motion.pose(world, hero, { pose: "wave" });
const s = og.motion.status(world, hero);
if (s.ok && s.value.goal === "none") nextBeat();
```

- A humanoid rig is one og.anim.humanoid recognises, or one
  og.MotionAgent.rig names. Until the weights are resident, or when they
  fail to load, og.motion.capabilities answers backend none.
- The model is a 17.6M-parameter student distilled from NVIDIA
  MotionBricks. Its library: idle, walk, run, jump, crouch,
  wave, cheer (og.motion.poses).
- Goals end by fixed rules: walkTo when the mover arrives, pose, place and
  face after their duration; then the next queued goal starts. With
  `drive` the og.motion system moves the Transform with fixed maths inside
  the tick (walkTo at its speed along a straight line, place over its
  duration, face at turnSpeed) and the generated pose is re-rooted onto
  the Transform, its planted feet absorbing the difference. Without
  `drive` the game moves the Transform (its own physics, path or
  controller) and the motion follows by how fast it moves. No model root
  motion ever reaches the simulation.
- Walks carry speed: a walk starts from the speed the last walk or jump
  left, so a walk queued after another keeps moving through the first
  target (braking only to the next walk's speed, less by the corner it
  turns), and a jump queued after a walk takes off at the speed it arrives
  with. An `acceleration` of 1000 starts at full speed in the first tick.
- A jump's arc is the simulation's. With an og.CharacterController the
  system sets the controller's jumpSpeed for `height` under its gravity
  and holds `jump` on the takeoff tick, so a ceiling cuts the rise and the
  landing is wherever the controller reaches ground; without one it flies
  the fixed ballistic arc back to the takeoff height. The generated jump is
  drawn in place on that flight, never adding the model's own vertical
  travel.
- The weights (35 MB in f16) ship inside @origingame/engine. `og cook`
  writes them as the optional file cooked/motion/model.ogb only for a
  project that uses og.motion (the manifest names it); the engine uploads
  them in the background as soon as the bundle is open (natively on a
  thread of its own, on the web a fetch after the boot bundle, never in
  it). One batch serves every agent, submitted in slices of about 3 ms of
  GPU time a frame so a batch never costs a frame its vsync, and decoded a
  few characters a frame.
- Results are applied when they come back, as of the tick they were due,
  so live frames may vary run to run. A capture, an `og test` screenshot,
  a reference frame or a fork check waits for every result due by its
  tick, and reproduces.
- og.motion.presentation is for a host's cover only: keep the page or
  shell cover up while the agent warms, lift it on ready or failed (showing
  the reason). Never write a component, gate a goal or advance a tick from
  it; a development build warns when a system reads it on the gpu backend.
  The probe's `dump_motion` reports readiness, generated frames, batches
  and timings.

| Backend | Cost | Use |
|---|---|---|
| gpu (default) | slices of about 3 ms GPU a frame for all agents | most games |
| cpu | about 21 ms of CPU a generation natively, 23 ms in wasm, spread over the ticks before it is due | a few characters, or no GPU backend |

  Set the backend before the first agent: a later change loads the model
  again and every agent's generated motion starts over. It is an engine
  setting, outside the world; a snapshot does not keep it.

<a id="guide-animation-11"></a>
### `animation §11` Motion reactions

A reaction (og.motion.react: a brace against wind, a flinch) plays an
authored clip over locomotion: entry, a held interval, exit. Its clip is a
cooked `type motion-action` asset:

```text
type motion-action
version 1
---
source "anims/brace.glb"
clip "brace"
rig "rigs/meshy.ogt"
holdStart 0.5
holdEnd 1.5
hand right
handTarget 0.1 -0.05 0.25
```

- Paths are relative to assets/; `rig` is optional. The hold lies inside
  the clip (0 <= holdStart < holdEnd < its duration) with matching poses at
  both ends, so it loops.
- Author it with a stationary root, +Z facing the reaction's source, entry
  before the hold and recovery after it. 30 Hz clips are recommended; the
  cook takes other rates.
- `hand` and `handTarget` attach that wrist to an offset from the head
  during the hold (a hood held against the wind): metres along the source
  head bone's local axes, independent of the GLB's unit scale.
  Retargeting adjusts it to the target's proportions, and the IK follows
  the final head pose after the look.
- The clip owns the stance step and lean: the engine synthesises no
  untrained action. Keep provenance and licensing beside the source asset.

<a id="guide-animation-12"></a>
### `animation §12` Tweens: state, timelines and feel presets

- Running tweens and timelines are world state (the og.tween state block):
  snapshots, restore and the world hash cover the whole timeline, its
  playhead, labels and captured values. og.TweenCompleted reports a tween,
  group or timeline that ends on its own; a cancelled one, or one whose
  entity or component went away, ends silently.
- A timeline's controls take effect in the next og.tween run: this tick
  when called from a system before it. A timeline's `from` and `fromTo`
  write their start values on its first render, before their time.
- A vector field takes one value per lane (`to: [0, 2, 0]`). A position
  naming a label that does not exist adds the label at the end.
- A `random` stagger shuffles from a fork of og.Rng's seed, so the main
  stream does not move; a system that builds one lists og.Rng among its
  reads.
- og.tween.squash and og.tween.flash are the feel presets: squash and
  stretch that recovers, a hit flash that fades.

| Squash `impact` | x | y | z |
|---|---|---|---|
| squash (a landing, a hit) | 1.25 | 0.8 | 0 (a sprite) |
| stretch (a jump) | 0.8 | 1.25 | 0 |
| a 3D squash, bulging in x and z | 1.15 | 0.85 | 1.15 |

  A component left out takes its default (x 1.25, y 0.8, z 0); z 0 keeps
  the rest scale, so a sprite's `{ x, y }` leaves z alone. Under a
  project's art contract ArtMotion.squash scales the impact: 0.5 keeps it,
  1 is rubber hose (x 1.5, y 0.6). Keep the recovery short (0.12 to 0.25
  s); back.out overshoots a little, elastic.out wobbles.

<a id="guide-animation-13"></a>
### `animation §13` Cutscene timelines

A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }`
blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }`
blocks; `og cook` cooks it (importing caption files) and og.load.timeline
loads it. An og.Timeline component plays it on its entity.

| Track | Keys |
|---|---|
| `camera` | position and target, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; `roll` (degrees about the view direction); `focus` (a distance) or `focusOn "<entity>"` and `focusRange`, which set og.PostEffects' depth of field while the track owns the camera; eased. `blendIn` and `blendOut` (seconds, on the track) ease from the camera it took at the timeline's start and back to it before the end |
| `transform "<entity>"` | position, rotation in degrees or a quaternion, scale; eased |
| `animation "<entity>"` | clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model |
| `audio` | sound, volume, playbackRate, bus |
| `music` | sound, volume, fadeIn, fadeOut, end, loop, bus |
| `caption` | text from `key` to `end`; `captions "narration.srt" [offset]` (or `.vtt` WebVTT with `<v Speaker>`, `<i>`, `<b>`, `<font color>`, and top/bottom positioning, or word timestamps `.json`), and its translations beside it (`narration.zh.srt`, `narration.zh.vtt`, `narration.pt-BR.srt`, each with its own times) cook with it: the track says the one of the locale the player asked for (og.i18n.setLocale), its language's for a regional tag, else its own; `locale "zh" { captions "…" }` names one elsewhere. A caption track draws nothing: og.timeline.captions(world) returns the keys showing, each `{ id, text, speaker, spans, position }`, which the built-in `Subtitles` screen shows after the game's own lines (mount it from `@origingame/engine/ui/screens/subtitles.tsx` and pass that list as its `captions` each tick) and any screen can iterate; where and how they are drawn is that screen's and the style pack's |
| `event` | `do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it |
| `transition` | kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from; over og.ui's screen cover |
| `marker` | `name`, sending og.TimelineMarker when played past |
| `property ["<entity>"]` | `field "og.PointLight.distance"` and keys of `value`: any reflected field of a component on the entity, or of a resource without one |
| `light "<entity>"` | its og.DirectionalLight, og.PointLight, og.SpotLight, og.RectLight or og.Light2D: color, intensity, distance, angle, ... |
| `visibility "<entity>"` | `visible true` or `false` of whatever it draws, stepped |
| `post` | og.PostEffects' fields |
| `blendshape "<entity>"` | og.Blendshapes w0 to w7; its source becomes component |
| `material "<entity>"` | its og.Mesh's material, or its og.Model's material `index N`: StandardMaterialDesc fields by name, other names the material shader's parameters |
| `timeline` | `play "<timeline>"`, speed, loop: from its key until the next, another timeline plays nested at (time − key) × speed, its markers sent as `<file>/<marker>`, at most four deep |
| `shot` | `camera "<entity>"` (a virtual camera's name) or `none`; `blend` seconds and `ease`: from its key until the next, that og.VirtualCamera is og.CameraBrain.solo, blended over the key's blend by its ease or by the camera's own; `none` and the timeline's end hand back the solo it found |
| `vfx` | `effect "<vfx>"` (a composed effect, `type vfx`), `at "<entity>"` (played as its child; none: in the world), `position` (offset), `scale`, `seed`: from its key the effect plays its length (a looping one until the timeline stops) at the world's tick; a seek into it joins it there without the events, shakes and sounds already passed; a seek out of it, a stop and the end take it |
| `video ["<entity>"]` | `video` (a `type video` asset), `from` (seconds into it, 0) and `speed` (0 to 4, 1): from its key until the next, the og.VideoPlayer of the entity (without a name: the og.Timeline's own) plays that video at `from + (t − key) · speed`, paused while the timeline is; loading the timeline loads its videos |
| `hotspot` | `id` (1 to 63 bytes), optional `kind` (`choice` or `inspect`), `shape` (`rect` or `circle`), `label`, `ease`, and `rect x y width height` (fractions of the video frame from its top left), or `off true`: from its key the hotspot covers the rect, easing to the same id's next key and held after its last until a key `off`. Nothing draws it: og.flow offers choices tagged `#hotspot:<id>` and non-cutting `inspect` hotspots (`flow §3`) |
| `scene` | `scene "<scene>"` (a `type scene` asset) or `scene none`: from its key until the next, that scene is in the world, its entities under a root that is a child of the og.Timeline's entity |

- The cook checks every field against the engine's records (a field the
  record lacks, one a timeline cannot drive, a value of the wrong shape),
  refuses nesting in a cycle or deeper than four, and refuses a track kind
  this engine does not read, naming it and the engine's version.
- Between two keys the later key's `ease` (og.tween's Ease names, linear
  when it gives none) mixes them; before the first and after the last the
  value is held. Values are evaluated at the timeline's time, kept exactly
  in steps of a thousandth of a tick, so seeking to a time and playing to
  it give the same world: a seek is how a video render starts at any
  frame.
- og.timeline runs after the game's systems and tweens, before og.anim and
  physics, so what it sets is what animation, physics and the frame see
  that tick. Event keys' actions run at the start of the next tick, as
  og.ai's commands do.
- While a timeline with a camera track plays or is paused it owns
  og.Camera and the camera brain waits; it hands back the camera it found
  when it stops or ends.
- A cutscene that opens on footage hitches at its first frame unless the
  decoder already has it. Set `preroll` on the og.Timeline while it is
  stopped, some ticks before you play it: nothing of it is shown, heard or
  taken (no camera, caption, sound or event), the entity's og.VideoPlayer
  holds the frame its own video track starts with, the scenes its scene
  tracks name are loaded, and the engine modules it needs are fetched.
  Then play it; `preroll` does nothing to a timeline
  that plays, is paused or was sought. og.flow pre-rolls the segments a
  choice may lead to this way (`flow` §6).
- A `scene` track is how a cutscene, a level or a minigame brings its
  own world: the scene is placed when the timeline's time enters its key
  and goes when it leaves it (the next key, `scene none`, a seek out) or
  the timeline stops. A seek inside a key keeps the scene that is there,
  with whatever the game did to it; og.timeline.stop then play places it
  anew. A timeline that ends keeps its last scene, as footage holds its
  last frame, until it is stopped or its entity goes. Move the
  og.Timeline's entity to move the scene. Entity names are unique in the
  world, so two timelines cannot have the same scene in at once.
- The cut happens between ticks: the tick whose time reaches the key
  decides, and the next tick starts with the old scene gone and the new
  one in place, on every host and in a replay, so no frame shows neither
  or both. The decision is saved with the world.
- The scenes a timeline names are loaded with what they spawn with (their
  models, textures and pipelines, their sub-scenes and prefabs) from the
  tick it plays or enters pre-roll, as og.scene.preload does
  (`loading` §12), and never placed before their key: a hidden scene
  would still be seen by physics and the game's systems. Give a timeline
  `preroll` a second or two before its cut and the cut costs only the
  spawn. If a scene is still being read or prepared at its cut, the clock
  holds and the world before the cut stays on screen until it is, as
  og.VideoPlayer.wait holds it for footage; no tick is lost.
- Loading a timeline (og.load.timeline, outside a system) grants
  og.timeline the records its tracks write. Values it wrote stay when it
  stops; the camera, its depth of field and the materials go back. A
  material track is presentation, applied every frame from the timeline's
  time, so a restored save draws the right material on its next frame.
- The command channel plays them: `timeline:play:<name>`
  (`timelines/<name>.ogt`, or an asset id), `timeline:seek:<seconds>[:<name>]`,
  `timeline:pause[:<name>]`, `timeline:resume[:<name>]`,
  `timeline:stop[:<name>]`.

<a id="guide-animation-14"></a>
### `animation §14` Auto-rigging layered 2D art (`type rig2d`)

A `type rig2d` `.ogt` file can auto-rig layered 2D artwork (a multi-layer PSD or a folder of PNG parts) into a full `og.Skeleton2D` rig plus packed texture atlas (`<rig>#atlas`) at cook time, with no external skeletal editor:

```text
type rig2d
version 1
---
parts "knight.psd"
template humanoid-side
pivot hand-r 212 340
unit 200
filter nearest
```

- **`parts`**: path to a layered `.psd` file or a folder of `.png` files (`"knight/"`). Each layer or PNG is named for a template part and drawn where it sits in the setup pose, overlapping its parent where they join.
- **`template`**: one of the four built-in rig templates — `humanoid-side`, `humanoid-front`, `quadruped`, or `creature`. The cook holds the parts to it: a layer or PNG named for no part of the template (one whose name starts with `_` is left out), a required part missing, two layers of one part, a part with no pixels, or a PNG not at the canvas size is refused, and the message names the parts the template expects and the fix.
- **`pivot <part> <x> <y>`**: optional manual joint override in canvas pixels.
- **`unit <px>`**: pixels per world unit; **`filter nearest`**: crisp pixel-art sampling.
- Load the cooked rig with `og.load.rig2d` and play its generated clips on `og.Skeleton2D` like any other 2D skeleton (§9).
