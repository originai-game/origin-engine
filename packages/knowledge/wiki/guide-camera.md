<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` guide: Cameras

Which camera to drive the view with and how: an orbit or follow controller for a quick start, virtual cameras and the brain for a 3D game with shots, Follow2D with zones and rails for 2D, and shake, punch and kick for feel. Every camera here is visual only: it writes og.Camera, which the simulation never reads unless the game does, and a replay moves it the same on every host.

The API: `og api show camera`. Cutscene shots on a timeline are in
`animation §13`; dialogue shots come from og.story (`yarn §8`).

<a id="guide-camera-1"></a>
### `camera §1` Choosing a camera

| The game | Use | Why |
|---|---|---|
| a new 3D game, any genre | og.VirtualCamera shots and og.CameraBrain | shots with priorities, blends, collision and focus; cutscenes and dialogue switch them |
| a model viewer, a board, an editor-like scene | og.OrbitCamera | three.js OrbitControls: drag turns, the wheel zooms, autoRotate |
| a quick third-person chase | og.FollowCamera | eases to an offset and looks at the entity; a vcam with `follow` and `collide` replaces it |
| a 2D game | og.Follow2D, og.CameraZone, og.CameraRail | dead zone, look-ahead, rooms and tracks |

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

<a id="guide-camera-2"></a>
### `camera §2` Virtual cameras and the brain

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
  the look point and eases back out over `damping`. It queries the
  colliders as the tick started and never moves a body.

| Blend | Seconds | Ease |
|---|---|---|
| a cut (a hard scene change, a dialogue reverse shot) | 0 | — |
| gameplay state (aim in, a vehicle) | 0.25 to 0.5 | power2.inOut |
| shot to shot in a cutscene | 1 to 2 | power2.inOut, sine.inOut |
| a slow reveal | 3 and up | sine.inOut |

<a id="guide-camera-3"></a>
### `camera §3` 2D cameras: follow, zones and rails

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

| Follow2D | Typical |
|---|---|
| `smoothTime` | 0.1 to 0.2 s; 0 snaps |
| `deadZone` | a quarter to a third of the view's half size |
| `lookAhead` | 0.2 to 0.5 s, `lookAheadMax` within a third of the view |
| zone `scrollTime` | 0.4 to 0.8 s for a room transition |

<a id="guide-camera-4"></a>
### `camera §4` Shake, punch and kick

All three are visual only, run unscaled (a hit stop still shakes) and
follow smooth noise of the tick, so a replay shakes the same. Systems may
call them; they take effect at the engine's og.camera system the same
tick.

| Effect | Light | Medium | Heavy |
|---|---|---|---|
| og.camera.shake trauma | 0.1 to 0.2 (a hit) | 0.25 to 0.4 | 0.5 to 0.8 (an explosion, a boss slam) |
| og.camera.punch amount | 0.05 (a tap) | 0.1 | 0.15 (a slam) |
| og.camera.kick distance | 0.05 m (recoil) | 0.15 m | 0.3 m (an impact) |

- Trauma adds up and clamps to 1; the shake grows with trauma squared.
  1 is almost never right: it is nauseating.
- Tune the feel on og.Camera (`traumaDecay`, `shakeOffset`, `shakeRoll`,
  `punchDecay`, `kickDecay`), not by calling shake every tick.
- Give the player a setting that scales trauma down; some players need
  shake off.

<a id="guide-camera-5"></a>
### `camera §5` Pitfalls

- Two drivers fighting: a vcam and an og.FollowCamera, or two follow
  controllers on two entities. The view jitters or ignores one; keep one.
- Reading og.Camera in gameplay: it is presentation, interpolated and
  shaken. Read the pose the game drives it from.
- Shaking from a system during a rollback re-simulation shakes twice:
  check og.net.resimulating first (`networking §4`).
- A cutscene that leaves `solo` set keeps the shot after it ends: release
  it with 0.
