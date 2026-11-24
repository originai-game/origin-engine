# Cameras

Which camera to drive the view with and how: an orbit or follow controller for a quick start, virtual cameras and the brain for a 3D game with shots, Follow2D with zones and rails for 2D, and shake, punch and kick for feel. Every camera here is visual only: it writes og.Camera, which the simulation never reads unless the game does, and a replay moves it the same on every host.

The API: `og api show camera`. Cutscene shots on a timeline are in
`animation §13`; dialogue shots come from og.story (`yarn §8`).

## 1. Choosing a camera

| The game | Use | Why |
|---|---|---|
| a new 3D game, any genre | og.VirtualCamera shots and og.CameraBrain | shots with priorities, blends, collision and focus; cutscenes and dialogue switch them |
| a model viewer, a board, an editor-like scene | og.OrbitCamera | three.js OrbitControls: drag turns, the wheel zooms, autoRotate |
| a quick third-person chase | og.FollowCamera | eases to an offset and looks at the entity; a vcam with `follow` and `collide` replaces it |
| a 2D game | og.Follow2D, og.CameraZone, og.CameraRail | dead zone, look-ahead, rooms and tracks |
| a shot list or a reference clip's framing | og.FramingCamera | the subject's screen height, place, yaw and pitch per beat; distance solved at any fov and aspect (§5) |

The controllers are game-level systems the engine ships in TypeScript; list
them first:

```ts
export default og.game({ systems: [...og.systems.cameras, ...yours], /* ... */ });
```

- Give the camera one driver. The first entity carrying a controller, in
  query order, drives it; a virtual camera drives og.Camera after the
  game's systems, so a world with vcams has no og.OrbitCamera or
  og.FollowCamera.
- og.Camera keeps three.js's parameter names (`fov`, `near`, `far`,
  `zoom`). Keep `near` as large as the game allows (0.1 m and up); depth
  precision follows `far / near`.
- Rendering reads og.Camera after the tick. A system that needs where the
  player looks reads the simulated pose that drove it, never og.Camera
  (`determinism §1`).

## 2. Virtual cameras and the brain

Each og.VirtualCamera entity is a shot; og.CameraBrain makes the
highest-priority enabled one live and blends og.Camera to it.

```ts
const play = world.spawn({ name: "play-cam" }).value!;
world.insert(play, og.VirtualCamera, { follow: hero, lookAt: hero, collide: true, priority: 10 });
const closeUp = world.spawn({ name: "close-up" }).value!;
world.insert(closeUp, og.VirtualCamera, { follow: npc, followOffset: { x: 0, y: 1.6, z: 1.2 }, lookAt: npc, fov: 35, enabled: false });

// a system that writes og.CameraBrain
const brain = world.resource(og.CameraBrain);
if (brain.ok) brain.value.solo = talking ? closeUp : 0;   // 0 releases it
```

- Read the brain as any resource: `brain.value.live` is the live shot. A
  system that reads it lists og.CameraBrain among its reads; one that sets
  `solo` among its writes.
- Switch shots by priority for gameplay states (aiming, a vehicle) and by
  `solo` for cutscenes and dialogue: a solo shot outranks every priority
  and is live even when not enabled. og.story sets `solo` for a dialogue
  line's `shot` (with `soloBlend` -1), and a timeline's shot track sets it
  for its keys; while a timeline's camera track plays, the timeline drives
  og.Camera and the brain waits.
- Of equal priorities the one enabled last wins, then the first in query
  order. Disabling the live shot blends to the next one.
- A blend starts from the previous camera still moving; a change during a
  blend starts from the blended pose, so the view never jumps. The first
  live camera of a world cuts.
- The brain's state is world state: a replay or a restore mid-blend
  continues the same blend. A world that never names a virtual camera
  keeps its hash.
- `focus` drives og.PostEffects' depth of field while the shot is live and
  hands the old values back when no live shot has focus.
- `collide` pulls the shot in at once when a collider comes between it and
  the look point and eases back out over `recoverTime`. It queries the
  colliders as the tick started and never moves a body.

| Blend | Value | Source |
|---|---|---|
| og.CameraBrain.blendTime | 1 s | engine default |
| og.VirtualCamera.blendTime | -1: the brain's | engine default |

## 3. 2D cameras: follow, zones and rails

- og.Follow2D moves the camera only when the entity leaves the dead zone,
  eases by `smoothTime` (a platformer follows jumps more lazily: set
  `smoothTimeY` above `smoothTime`) and leads the entity by its velocity
  with `lookAhead`. It keeps the camera's z, rotation and projection.
- og.CameraZone entities keep the view inside the zone the followed entity
  is in: rooms, arenas, a level's bounds. Overlaps go to the higher
  priority, then the first in query order. Entering another zone scrolls
  across over the new zone's `scrollTime` (0 jumps); a zone smaller than
  the view centres it. The engine's og.camera system picks the zone after
  the game's systems, into og.Camera's zone fields.
- og.CameraRail moves the camera along a spline to the point nearest its
  entity: corridors, a side-scroller's track, a 3D chase on rails. For
  more than eight points give it a `path` (og.path.create).

| Field | Value | Source |
|---|---|---|
| Follow2D `smoothTime` | 0.15 s; 0 snaps | engine default |
| Follow2D `lookAheadMax` | 3, 1.5 | engine default |
| Follow2D `deadZone`, `lookAhead` | 0: none | engine default |
| CameraZone `scrollTime` | 0: the view jumps | engine default |

## 4. Shake, punch and kick

All three are visual only, run unscaled (a hit stop still shakes) and
follow smooth noise of the tick, so a replay shakes the same. Systems may
call them; they take effect at the engine's og.camera system the same
tick.

Motion blur is separate: it integrates along the prepass's camera/object
velocity for the authored shutter fraction. A nearby moving surface may
contribute only where its shutter sweep covers the pixel and is not behind
it. A stationary subject outside that sweep keeps its own zero velocity;
putting it near a fast-moving object does not attach the two.

| Field | Value | Source |
|---|---|---|
| og.Camera.traumaDecay | 1.5 per second | engine default |
| trauma | clamped to 1 | engine limit |
| og.camera.shake trauma, recommended | 0.1 to 0.2 for a light hit, 0.25 to 0.4 a medium one, 0.5 to 0.8 an explosion or a boss slam; 1 is almost never right | engine guidance (IDL at 9ba5b0957) |
| og.camera.punch amount, recommended | 0.05 for a tap, 0.15 for a slam | engine guidance (IDL at 9ba5b0957) |

- Trauma adds up and clamps to 1; the shake grows with trauma squared.
  1 is almost never right: it is nauseating.
- Tune the feel on og.Camera (`traumaDecay`, `shakeOffset`, `shakeRoll`,
  `punchDecay`, `kickDecay`), not by calling shake every tick.
- Give the player a setting that scales trauma down; some players need
  shake off.

## 5. Framing a subject by beats

og.FramingCamera frames one subject the way a shot list or a reference
clip states it. At each beat it takes the subject's height as a fraction of
the frame's, where its centre sits in the frame, and the view's yaw and
pitch. It solves the camera's distance and place from og.Camera's `fov`
and the play surface's aspect, so a table measured on a 16:9 clip frames
the same on a phone or at another field of view. Nothing is placed by eye.

- Copy the shot list into `beats` row for row: time, screen height, x, y,
  yaw and pitch, six numbers a beat, at most 16 beats. x and y are
  fractions of the frame from its top left, as an image editor measures
  them. Yaw is in degrees about +Y from +Z towards +X; pitch is degrees
  above the horizon.
- Give `height`, the length the screen height measures (a character's
  standing height), and `centre`, the point x and y place, relative to
  the entity's position: half the height up for a character standing on
  its origin.
- Between beats every number eases (smoothstep). The screen height eases
  through its reciprocal, so the distance changes evenly. Yaw eases as
  written: -90 to -160 turns 70 degrees, and to go the short way round
  you write the short way.
- The clock is `time`: each tick frames the time it holds, then adds
  `speed` seconds, or ticks with `ticks`. A game whose shot follows its
  own states sets `time` itself (such as `SWING_AT + t` during a swing)
  with `speed` 0.
- The screen height is measured at the subject's centre. A subject seen
  from well above or below spans a little more or less than h from its
  top to its feet.
- Use a perspective og.Camera. Keep pitch inside ±90 degrees.

```ts recipe ticks=130
import { defineSystem, og } from "@origingame/engine";
import type { Result, Vector3, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const HEIGHT = 1.8;
// The shot list, one row a beat: tick, screen height, x, y, yaw (deg), pitch (deg).
const BEATS = [
  0, 0.7, 0.5, 0.45, 0, 0,
  60, 0.3, 0.33, 0.6, -90, 12,
  120, 0.5, 0.66, 0.4, -180, 0,
];

// He runs along +Z; the camera frames him wherever he is.
const Run = defineSystem("Run", { reads: [], writes: [og.Transform] }, (_q, time, world) => {
  const hero = world.find("hero")!;
  const t = must(world.get(hero, og.Transform));
  must(world.set(hero, og.Transform, { ...t, position: { ...t.position, z: t.position.z + 4 * time.dt } }));
});

// On each beat's tick, project him and compare with the row.
const Check = defineSystem("Check", { reads: [og.Camera, og.Transform, og.FramingCamera], writes: [] }, (_q, _time, world: World) => {
  const hero = world.find("hero")!;
  // The clock has moved on past the tick this frame was read at.
  const at = must(world.get(hero, og.FramingCamera)).time - 1;
  const row = BEATS.findIndex((v, i) => i % 6 === 0 && v === at);
  if (row < 0) return;
  const [, h, x, y, , pitch] = BEATS.slice(row, row + 6) as [number, number, number, number, number, number];
  const camera = must(world.resource(og.Camera));
  const view = og.ui.viewport(world);
  const p = must(world.get(hero, og.Transform)).position;
  const screen = (v: Vector3) => camera.worldToScreen(v)!;
  const centre = screen({ x: p.x, y: p.y + HEIGHT / 2, z: p.z });
  og.assert.always(Math.abs(centre.x / view.width - x) < 0.002 && Math.abs(centre.y / view.height - y) < 0.002, "his centre sits where the row says", { at, centre });
  if (pitch === 0) {
    const span = (screen(p).y - screen({ x: p.x, y: p.y + HEIGHT, z: p.z }).y) / view.height;
    og.assert.always(Math.abs(span - h) < 0.01, "level, he is the row's height of the frame", { at, span });
  }
  if (row === 12) og.assert.reachable("the last beat was framed", {});
});

export default og.game({
  systems: [Run, ...og.systems.cameras, Check],
  start(world) {
    const hero = must(world.spawn({ name: "hero" }));
    must(world.insert(hero, og.FramingCamera, { height: HEIGHT, centre: { x: 0, y: HEIGHT / 2, z: 0 }, beats: BEATS, ticks: true }));
  },
});
```

### Framing a level area

A shooter's playfield, an arena or a board is framed whole, not by a
subject's height: give `area`, its width across the view and its depth
along the ground ahead, lying level through `centre`. Each beat's screen
height is then the fraction of the frame the area's on-screen bounds fill
in their tighter direction (0.94 leaves 3% each side), and x and y place
the centre of those bounds. Under perspective the near edge spans wider
than the far one; the system solves that, so the same beats fit the field
on a 9:19.5 phone and at 16:9, at any pitch and fov. A vertical shooter
keeps one beat; an arena intro eases the pitch over a few. Put the
component on the field's own entity (it never moves) rather than the
player, and drive the follow and lead with og.VirtualCamera if the view
should track the ship.

```ts recipe ticks=40
import { defineSystem, og } from "@origingame/engine";
import type { Result, Vector3 } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

// The playfield: 18 m across, 32 m deep, seen from 40 degrees above.
const W = 18;
const D = 32;
const FILL = 0.94;
const BEATS = [0, FILL, 0.5, 0.5, 180, -40];

// Project the four corners and compare their bounds with the beat.
const Check = defineSystem("Check", { reads: [og.Camera, og.FramingCamera], writes: [] }, (_q, _time, world) => {
  const camera = must(world.resource(og.Camera));
  const view = og.ui.viewport(world);
  const xs: number[] = [];
  const ys: number[] = [];
  for (const [x, z] of [[-W / 2, -D / 2], [W / 2, -D / 2], [-W / 2, D / 2], [W / 2, D / 2]] as const) {
    const s = camera.worldToScreen({ x, y: 0, z } as Vector3);
    if (!s) return;
    xs.push(s.x / view.width);
    ys.push(s.y / view.height);
  }
  const w = Math.max(...xs) - Math.min(...xs);
  const h = Math.max(...ys) - Math.min(...ys);
  const cx = (Math.max(...xs) + Math.min(...xs)) / 2;
  const cy = (Math.max(...ys) + Math.min(...ys)) / 2;
  og.assert.always(Math.abs(Math.max(w, h) - FILL) < 0.002, "the field's bounds fill the beat's share in their tighter direction", { w, h });
  og.assert.always(w <= FILL + 0.002 && h <= FILL + 0.002, "the whole field is in frame", { w, h });
  og.assert.always(Math.abs(cx - 0.5) < 0.002 && Math.abs(cy - 0.5) < 0.002, "the bounds sit centred", { cx, cy });
  og.assert.reachable("the field was framed", {});
});

export default og.game({
  systems: [...og.systems.cameras, Check],
  start(world) {
    must(world.resource(og.Camera)).fov = 50;
    const field = must(world.spawn({ name: "field" }));
    must(world.insert(field, og.FramingCamera, { area: { x: W, y: D }, beats: BEATS }));
  },
});
```

## 6. Pitfalls

- Two drivers fighting: a vcam and an og.FollowCamera, or two follow
  controllers on two entities. The view jitters or ignores one; keep one.
- Reading og.Camera in gameplay: it is presentation, interpolated and
  shaken. Read the pose the game drives it from.
- Shaking from a system during a rollback re-simulation shakes twice:
  check og.net.resimulating first (`networking §4`).
- A cutscene that leaves `solo` set keeps the shot after it ends: release
  it with 0.

## 7. Physical lenses

A physical perspective camera takes its angle from an explicit focal
length and sensor, rather than `fov`. Millimetres describe the lens and
sensor; scene distances remain metres. A 50 mm lens on a 24 mm high gate
has a vertical field of view of about 26.99147°. `zoom` crops that gate.
All new numbers start at zero: an incomplete lens falls back to `fov`,
never to an engine-selected filmback. Orthographic cameras retain their
existing height and do not become thin-lens cameras. `lensStatus` reports
artistic, physical, invalid or orthographic after the camera system runs.

```ts recipe ticks=2
import { og } from "@origingame/engine";

export default og.game({
  start(world) {
    const result = world.resource(og.Camera);
    if (!result.ok) throw new Error(result.reason);
    const camera = result.value;
    camera.position = { x: 0, y: 0, z: 0 };
    camera.quaternion = { x: 0, y: 0, z: 0, w: 1 };
    camera.lensMode = "physical";
    camera.focalLength = 50;
    camera.sensorSize = { x: 36, y: 24 };
    camera.gateFit = "vertical";
    camera.lensShift = { x: 0.2, y: -0.1 };
    const axis = camera.worldToScreen({ x: 0, y: 0, z: -5 })!;
    const view = og.ui.viewport(world);
    og.assert.always(Math.abs(axis.x / view.width - 0.3) < 0.001
      && Math.abs(axis.y / view.height - 0.4) < 0.001,
      "positive shift moves the viewed window, not the camera", { axis });
  },
});
```

Vertical gate fitting keeps sensor height; horizontal keeps sensor width.
Fill uses the smaller fitted height, cropping the gate; overscan uses the
larger, showing the whole gate. Neither stretches pixels. Shift is a
fraction of each fitted dimension. Positive x/y moves the viewed window
right/up, so a subject on the optical axis moves left/down. Screen queries,
picking, shadows and the rendered view use the same shifted projection.

Virtual cameras carry the same authored lens. Between two physical shots
with the same gate fit, the brain blends focal length and sensor size,
then derives the projection; it does not blend an unrelated fov. Different
gate fits and physical/artistic transitions crossfade focal factors and
shift. An interrupted transition holds those resolved factors in the
brain's snapshot-owned state, at the last evaluated aspect. The held view
therefore remains continuous even if its outgoing entity disappears.
The old artistic-to-artistic fov blend is unchanged. A timeline camera
track still owns the camera while it plays.

The aperture, focus distance, shutter and exposure fields reserve the
physical post channels; projection alone does not enable DoF or motion
blur. Keep the corresponding PostEffects switches explicit. The artistic
post settings and Camera.exposure remain independent authored inputs.
