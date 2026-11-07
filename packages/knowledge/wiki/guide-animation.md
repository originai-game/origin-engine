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
| samples faster than 0.1 m/s | a foot creeping | under 5% (the engine shows 0.5–2%) |
| samples faster than 0.3 m/s outside a takeoff | a foot gliding | under 1% |
| floating frames of a held pose | a foot hovering | none |

The engine gets there without the model's help: a walk or run is played
at the speed the simulation moves the Transform (stride warping in time),
the drawn hips follow the generated hips' travel within 0.15 m of the
Transform, a foot is planted where the pose shows it low and slow (in a
held pose wherever it is under 0.35 m) and put down on the ground, a
planted foot left behind, let go or out of the leg's reach steps (lifted
7 cm on its arc, one foot at a time), a swing over the ground is lifted
clear of it, and the hips lower for a planted foot out of reach. A
generated jump that lands and takes off again unasked is cut at its
landing's lowest point and the model starts again from there. Queue walks with a speed for each gait:
a walkTo gains speed at its `acceleration` and carries it into the next
walk or jump, so the generated stride has a start and a run to follow.

<a id="guide-animation-3"></a>
### `animation §3` Clips and graphs

- A graph's parameters are set from systems in the tick. Gameplay that
  needs a joint (a hand for a sword's hitbox) reads the tick's pose
  through og.anim's joint queries, never the drawn pose.
- Retarget a clip library to the game's rig once, at load (og.load.rig
  and og.anim.retarget), never per frame.
- Hitboxes and damage windows come from the tick: mark them with clip
  events or tick counts, not with how far the drawn animation has played.

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
  reduced-motion setting replaces movement with a fade.
- **Coyote time** about 100 ms and a jump buffer of about 100 ms for a
  platformer; a jump without them feels broken at any polish.
- Ease every tween; linear motion reads as mechanical except for
  constant-speed things (a conveyor, a projectile).
