<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-CameraZone"></a>

### `og.CameraZone` (component)

A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it.

Token `og.CameraZone`, registered under the schema name `og.CameraZone`; its value type is `CameraZone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The bottom-left corner. |
| `max` | `Vector2` | `{ x: 10, y: 10 }` | — | `m` | worldspace `world` | The top-right corner. |
| `priority` | `number` | `0` | — | — | — | Higher wins where zones overlap. |
| `scrollTime` | `number` | `0` | ≥ 0 | `s` | — | Seconds the view takes to scroll from the zone the entity left into this one (a room transition, eased in and out); 0 jumps at once. Visual only: the camera resource records the transition, so replays scroll the same. |

```ogt
component CameraZone {
  doc "A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it."
  field min Vector2 "The bottom-left corner." {
    worldspace world
    unit m
  }
  field max Vector2 "The top-right corner." {
    worldspace world
    default 10 10
    unit m
  }
  field priority i32 "Higher wins where zones overlap."
  field scrollTime f32 "Seconds the view takes to scroll from the zone the entity left into this one (a room transition, eased in and out); 0 jumps at once. Visual only: the camera resource records the transition, so replays scroll the same." {
    unit s
    min 0
  }
}
```

<sub>`api.ogt` line 3525</sub>
