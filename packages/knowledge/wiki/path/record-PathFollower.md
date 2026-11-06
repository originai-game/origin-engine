<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="record-PathFollower"></a>

### `og.PathFollower` (component)

Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there.

Token `og.PathFollower`, registered under the schema name `og.PathFollower`; its value type is `PathFollower`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `Path` | `0` | — | — | — | The path (og.path.create or og.path.find); 0 does nothing. |
| `speed` | `number` | `1` | — | `mps` | tunable | Metres per second along the path; negative goes backwards. |
| `distance` | `number` | `0` | — | `m` | — | How far along the path it is, from its start; written every tick, set it to place the entity. |
| `loop` | `boolean` | `true` | — | — | — | Wrap at the end (and the start, going backwards); otherwise stop there and set `finished`. |
| `align` | `boolean` | `false` | — | — | — | Turn og.Transform.quaternion to face along the tangent (local -Z forward, +Y the path's up). |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Added to the path's point: along the path's right, up and backward axes with align, else world x, y and z. |
| `playing` | `boolean` | `true` | — | — | — | Moves while true. |
| `finished` | `boolean` | `false` | — | — | read only | Written: a path without loop reached its end. |

```ogt
component PathFollower {
  doc "Moves the entity along a path at `speed`, by the engine's og.path system after the game's systems and before physics: og.Transform.position is the path's point at `distance` plus `offset` (in the path's frame with align, else in world axes). Advanced by scaled time (slow motion, hit stop). The distance is the component's own field, so a snapshot, a restore and the world hash carry it, and writing it moves the entity there."
  field path Path "The path (og.path.create or og.path.find); 0 does nothing."
  field speed f32 "Metres per second along the path; negative goes backwards." {
    default 1
    unit mps
    tunable true
  }
  field distance f32 "How far along the path it is, from its start; written every tick, set it to place the entity." {
    unit m
  }
  field loop bool "Wrap at the end (and the start, going backwards); otherwise stop there and set `finished`." {
    default true
  }
  field align bool "Turn og.Transform.quaternion to face along the tangent (local -Z forward, +Y the path's up)."
  field offset Vector3 "Added to the path's point: along the path's right, up and backward axes with align, else world x, y and z." {
    unit m
  }
  field playing bool "Moves while true." {
    default true
  }
  field finished bool "Written: a path without loop reached its end." {
    readonly true
  }
}
```

<sub>`api.ogt` line 26078</sub>
