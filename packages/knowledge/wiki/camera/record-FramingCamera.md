<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-FramingCamera"></a>

### `og.FramingCamera` (component)

Frames the entity carrying it with og.Camera by beats, as a shot list states them: per beat the subject's height as a fraction of the frame's, where its centre sits, and the view's yaw and pitch. It solves the camera's distance and place from og.Camera's fov and the play surface's aspect, easing between beats (smoothstep); before the first the first holds, after the last the last. Perspective only. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.

See `camera §5`.

Also known as: framing camera, shot list, composition, rule of thirds, Cinemachine framing transposer, screen position

Token `og.FramingCamera`, registered under the schema name `og.FramingCamera`; its value type is `FramingCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0` | ≥ 0 | `m` | tunable | The subject's height, the length a beat's screen height measures (a character's standing height); 0 frames nothing. |
| `centre` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The point a beat's x and y place, relative to the entity's position, in world axes: the centre of the subject's box, such as half its height up for a character standing on its origin. |
| `beats` | `readonly number[]` | `[]` | — | — | a list of at most 96 `f32` (`List.f32.96`): reads a frozen copy, writes a whole array | Up to 16 beats in rising time, six numbers each: time (on `time`'s clock), screen height (above 0), x and y (the centre's place, fractions of the frame from its top left), yaw (degrees about +Y from +Z towards +X, eased as written) and pitch (degrees above the horizon). Empty frames nothing. |
| `area` | `Vector2` | `{ x: 0, y: 0 }` | ≥ 0 | `m` | worldspace `none` | A level rectangle framed whole instead of `height`: its width across the view and depth along the ground ahead, through `centre`. A beat's screen height is then the share of the frame its on-screen bounds fill in their tighter direction; x and y centre those bounds. Zero uses `height`. |
| `ticks` | `boolean` | `false` | — | — | — | Beat times count ticks instead of seconds. |
| `time` | `number` | `0` | — | — | tunable | The clock the beats are read on: a tick frames the time it holds, then advances it by `speed` seconds (ticks with `ticks`), so the first tick after adding the component frames time 0. A game may set it, such as to the second of a clip a state stands for. |
| `speed` | `number` | `1` | — | — | tunable | How fast `time` advances: 1 in step with the world's time, 0 holds it for the game to set. |

```ogt
component FramingCamera {
  doc "Frames the entity carrying it with og.Camera by beats, as a shot list states them: per beat the subject's height as a fraction of the frame's, where its centre sits, and the view's yaw and pitch. It solves the camera's distance and place from og.Camera's fov and the play surface's aspect, easing between beats (smoothstep); before the first the first holds, after the last the last. Perspective only. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera."
  guide "camera §5"
  aliases "framing camera, shot list, composition, rule of thirds, Cinemachine framing transposer, screen position"
  field height f32 "The subject's height, the length a beat's screen height measures (a character's standing height); 0 frames nothing." {
    min 0
    unit m
    tunable true
  }
  field centre Vector3 "The point a beat's x and y place, relative to the entity's position, in world axes: the centre of the subject's box, such as half its height up for a character standing on its origin." {
    worldspace none
    unit m
  }
  field beats List.f32.96 "Up to 16 beats in rising time, six numbers each: time (on `time`'s clock), screen height (above 0), x and y (the centre's place, fractions of the frame from its top left), yaw (degrees about +Y from +Z towards +X, eased as written) and pitch (degrees above the horizon). Empty frames nothing."
  field area Vector2 "A level rectangle framed whole instead of `height`: its width across the view and depth along the ground ahead, through `centre`. A beat's screen height is then the share of the frame its on-screen bounds fill in their tighter direction; x and y centre those bounds. Zero uses `height`." {
    worldspace none
    unit m
    min 0
  }
  field ticks bool "Beat times count ticks instead of seconds."
  field time f32 "The clock the beats are read on: a tick frames the time it holds, then advances it by `speed` seconds (ticks with `ticks`), so the first tick after adding the component frames time 0. A game may set it, such as to the second of a clip a state stands for." {
    tunable true
  }
  field speed f32 "How fast `time` advances: 1 in step with the world's time, 0 holds it for the game to set." {
    default 1
    tunable true
  }
}
```

<sub>Module `camera` line 390</sub>
