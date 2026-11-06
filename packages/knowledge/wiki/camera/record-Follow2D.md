<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-Follow2D"></a>

### `og.Follow2D` (component)

Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in.

Also known as: follow camera, 2D camera, side-scroller camera, Cinemachine follow, Godot Camera2D

Token `og.Follow2D`, registered under the schema name `og.Follow2D`; its value type is `Follow2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where the camera centre sits relative to the entity. |
| `deadZone` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Half the width and height of the box around the camera centre inside which the entity moves without moving the camera. |
| `smoothTime` | `number` | `0.15` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up; 0 snaps it every tick. |
| `smoothTimeY` | `number` | `-1` | — | `s` | tunable | The same for y alone (a platformer follows jumps more lazily than runs); below 0 uses smoothTime. |
| `lookAhead` | `number` | `0` | ≥ 0 | `s` | tunable | Lead the entity by its velocity times this many seconds (0: no look-ahead). |
| `lookAheadMax` | `Vector2` | `{ x: 3, y: 1.5 }` | ≥ 0 | `m` | — | The largest lead along x and y. |
| `lookAheadSmoothing` | `number` | `0.4` | ≥ 0 | `s` | tunable | Roughly how long the lead takes to turn when the entity does. |
| `lead` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | read only | Controller state: the current lead. |
| `last` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | read only, worldspace `root` | Controller state: the entity's position on the previous tick. |
| `clamp` | `boolean` | `false` | — | — | — | Keep the camera centre between boundsMin and boundsMax. |
| `boundsMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The smallest x and y the camera centre takes with clamp. |
| `boundsMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The largest x and y the camera centre takes with clamp. |

```ogt
component Follow2D {
  doc "Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in."
  aliases "follow camera, 2D camera, side-scroller camera, Cinemachine follow, Godot Camera2D"
  field offset Vector2 "Where the camera centre sits relative to the entity." {
    unit m
  }
  field deadZone Vector2 "Half the width and height of the box around the camera centre inside which the entity moves without moving the camera." {
    unit m
  }
  field smoothTime f32 "Roughly how long the camera takes to catch up; 0 snaps it every tick." {
    default 0.15
    min 0
    unit s
    tunable true
  }
  field smoothTimeY f32 "The same for y alone (a platformer follows jumps more lazily than runs); below 0 uses smoothTime." {
    default -1
    unit s
    tunable true
  }
  field lookAhead f32 "Lead the entity by its velocity times this many seconds (0: no look-ahead)." {
    min 0
    unit s
    tunable true
  }
  field lookAheadMax Vector2 "The largest lead along x and y." {
    default 3 1.5
    min 0
    unit m
  }
  field lookAheadSmoothing f32 "Roughly how long the lead takes to turn when the entity does." {
    default 0.4
    min 0
    unit s
    tunable true
  }
  field lead Vector2 "Controller state: the current lead." {
    readonly true
    unit m
  }
  field last Vector2 "Controller state: the entity's position on the previous tick." {
    worldspace root
    readonly true
    unit m
  }
  field clamp bool "Keep the camera centre between boundsMin and boundsMax."
  field boundsMin Vector2 "The smallest x and y the camera centre takes with clamp." {
    worldspace world
    unit m
  }
  field boundsMax Vector2 "The largest x and y the camera centre takes with clamp." {
    worldspace world
    unit m
  }
}
```

<sub>Module `camera` line 240</sub>
