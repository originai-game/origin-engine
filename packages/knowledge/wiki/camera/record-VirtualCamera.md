<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-VirtualCamera"></a>

### `og.VirtualCamera` (component)

A shot og.Camera can take, as Cinemachine's virtual cameras: a position, what it looks at, a field of view, roll and focus. og.CameraBrain makes the enabled one with the highest priority live and blends og.Camera to it over the incoming shot's blendTime by its blendEase.

Registered when a game first names it (with og.CameraBrain and og.CameraCut), so a world without one keeps its hash. Its pose is tick state, drawing on no randomness; it changes only og.Camera and, with focus, og.PostEffects' depth of field. While a timeline's camera track plays, the timeline drives og.Camera; the brain waits.

See `camera §2`.

Token `og.VirtualCamera`, registered under the schema name `og.VirtualCamera`; its value type is `VirtualCamera`.

#### Pick

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `priority` | `number` | `0` | — | — | tunable | Of the enabled virtual cameras the highest priority is live; of equal ones, the one enabled last (`activated`), then the first in query order. |
| `enabled` | `boolean` | `true` | — | — | — | Takes part in the brain's pick. Disabling the live camera blends to the next one. |

#### Position

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `follow` | `Entity` | `0` | — | — | — | The entity it follows: it sits at followOffset from it, easing by smoothTime. 0: it sits at its own entity's og.Transform position. |
| `followOffset` | `Vector3` | `{ x: 0, y: 2, z: 6 }` | — | `m` | tunable | Where it sits relative to the followed entity, in world axes; with followRotation, x to that entity's right, y up and z behind it. |
| `followRotation` | `boolean` | `false` | — | — | — | Turn followOffset with the followed entity's rotation, so it stays behind it: an entity faces its local +Z, and the offset's z counts backwards from there. |
| `smoothTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | Roughly how long it takes to catch up with the followed entity; 0 snaps every tick. It snaps when it is enabled. |

#### Aim

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lookAt` | `Entity` | `0` | — | — | — | The entity it looks at, at lookOffset from its position. 0: it keeps its own entity's og.Transform rotation. |
| `lookOffset` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | `m` | tunable | The point it looks at, relative to the lookAt entity's position (and the point collision keeps clear from). |

#### Lens

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fov` | `number` | `50` | 1 … 179 | `deg` | tunable | The vertical field of view it gives og.Camera. |
| `roll` | `number` | `0` | — | `deg` | tunable | A turn about the view direction, applied after the look (a Dutch angle). |
| `focus` | `number` | `0` | ≥ 0 | `m` | tunable | Depth of field: the focus distance it gives og.PostEffects (depthOfField on), handed back when no live camera has one. 0 leaves og.PostEffects alone. |
| `focusRange` | `number` | `1` | ≥ 0 | `m` | tunable | Depth of field: the range around the focus distance that stays sharp. |

#### Blend

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `blendTime` | `number` | `-1` | — | `s` | — | Seconds the brain takes to blend to it when it goes live (0 cuts); below 0 uses og.CameraBrain's blendTime and blendEase. |
| `blendEase` | `Ease` | `"power2.inOut"` | — | — | — | The curve of the blend to it: position lerped, rotation slerped, fov, roll and focus lerped by the curve's value at the blend's time. |

#### Collision

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `collide` | `boolean` | `false` | — | — | — | Keep a sphere of `radius` clear of colliders between the look point and where it sits: it pulls in towards the look point at once and eases back out over damping. Against the colliders as the tick started (og.physics' query view): it never moves a body. |
| `radius` | `number` | `0.2` | ≥ 0 | `m` | — | The sphere collision keeps clear. |
| `damping` | `number` | `0.4` | ≥ 0 | `s` | tunable | Roughly how long it takes to ease back out once the way is clear; 0 snaps. |
| `collisionMask` | `number` | `0` | — | — | — | The physics layers collision considers (a collider counts when its layers meet these); 0 every layer. |

#### Controller state

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `eased` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Controller state: the followed point after smoothing, before collision, in world space (a floating-origin rebase shifts it). |
| `pull` | `number` | `0` | — | `m` | read only | Controller state: how far collision has pulled it in towards the look point. |
| `activated` | `number` | `0` | — | — | read only | Controller state: og.CameraBrain.activations when it was last enabled (the later wins a priority tie). |
| `primed` | `boolean` | `false` | — | — | read only | Controller state: it was evaluated last tick (enabled, live or blended from); one that was not snaps to its goal and, if enabled, takes the next activation. |

```ogt
component VirtualCamera {
  doc "A shot og.Camera can take, as Cinemachine's virtual cameras: a position, what it looks at, a field of view, roll and focus. og.CameraBrain makes the enabled one with the highest priority live and blends og.Camera to it over the incoming shot's blendTime by its blendEase."
  doc "Registered when a game first names it (with og.CameraBrain and og.CameraCut), so a world without one keeps its hash. Its pose is tick state, drawing on no randomness; it changes only og.Camera and, with focus, og.PostEffects' depth of field. While a timeline's camera track plays, the timeline drives og.Camera; the brain waits."
  guide "camera §2"
  group "Pick"
  field priority i32 "Of the enabled virtual cameras the highest priority is live; of equal ones, the one enabled last (`activated`), then the first in query order." {
    tunable true
  }
  field enabled bool "Takes part in the brain's pick. Disabling the live camera blends to the next one." {
    default true
  }
  group "Position"
  field follow Entity "The entity it follows: it sits at followOffset from it, easing by smoothTime. 0: it sits at its own entity's og.Transform position."
  field followOffset Vector3 "Where it sits relative to the followed entity, in world axes; with followRotation, x to that entity's right, y up and z behind it." {
    default 0 2 6
    unit m
    tunable true
  }
  field followRotation bool "Turn followOffset with the followed entity's rotation, so it stays behind it: an entity faces its local +Z, and the offset's z counts backwards from there."
  field smoothTime f32 "Roughly how long it takes to catch up with the followed entity; 0 snaps every tick. It snaps when it is enabled." {
    default 0.25
    min 0
    unit s
    tunable true
  }
  group "Aim"
  field lookAt Entity "The entity it looks at, at lookOffset from its position. 0: it keeps its own entity's og.Transform rotation."
  field lookOffset Vector3 "The point it looks at, relative to the lookAt entity's position (and the point collision keeps clear from)." {
    default 0 1 0
    unit m
    tunable true
  }
  group "Lens"
  field fov f32 "The vertical field of view it gives og.Camera." {
    default 50
    min 1
    max 179
    unit deg
    tunable true
  }
  field roll f32 "A turn about the view direction, applied after the look (a Dutch angle)." {
    unit deg
    tunable true
  }
  field focus f32 "Depth of field: the focus distance it gives og.PostEffects (depthOfField on), handed back when no live camera has one. 0 leaves og.PostEffects alone." {
    min 0
    unit m
    tunable true
  }
  field focusRange f32 "Depth of field: the range around the focus distance that stays sharp." {
    default 1
    min 0
    unit m
    tunable true
  }
  group "Blend"
  field blendTime f32 "Seconds the brain takes to blend to it when it goes live (0 cuts); below 0 uses og.CameraBrain's blendTime and blendEase." {
    default -1
    unit s
  }
  field blendEase Ease "The curve of the blend to it: position lerped, rotation slerped, fov, roll and focus lerped by the curve's value at the blend's time." {
    default power2.inOut
  }
  group "Collision"
  field collide bool "Keep a sphere of `radius` clear of colliders between the look point and where it sits: it pulls in towards the look point at once and eases back out over damping. Against the colliders as the tick started (og.physics' query view): it never moves a body."
  field radius f32 "The sphere collision keeps clear." {
    default 0.2
    min 0
    unit m
  }
  field damping f32 "Roughly how long it takes to ease back out once the way is clear; 0 snaps." {
    default 0.4
    min 0
    unit s
    tunable true
  }
  field collisionMask u32 "The physics layers collision considers (a collider counts when its layers meet these); 0 every layer."
  group "Controller state"
  field eased Vector3 "Controller state: the followed point after smoothing, before collision, in world space (a floating-origin rebase shifts it)." {
    readonly true
    worldspace world
    unit m
  }
  field pull f32 "Controller state: how far collision has pulled it in towards the look point." {
    readonly true
    unit m
  }
  field activated u32 "Controller state: og.CameraBrain.activations when it was last enabled (the later wins a priority tie)." {
    readonly true
  }
  field primed bool "Controller state: it was evaluated last tick (enabled, live or blended from); one that was not snaps to its goal and, if enabled, takes the next activation." {
    readonly true
  }
}
```

<sub>Module `camera` line 373</sub>
