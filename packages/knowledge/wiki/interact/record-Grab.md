<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Grab"></a>

### `og.Grab` (component)

A body an og.Interactor picks up when it uses it: while carried it is made kinematic and moved each tick to `carry` in the carrier's frame; the carrier's next press throws it, dynamic again, along the carrier's forward at `throwSpeed`. Put it on a dynamic og.RigidBody.

Also known as: pick up, carry, throw object, hold item, grab physics object, Half-Life gravity gun

Token `og.Grab`, registered under the schema name `og.Grab`; its value type is `Grab`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `carry` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | Where it is carried, in the carrier's frame (forward +Z). |
| `throwSpeed` | `number` | `0` | ≥ 0 | `mps` | tunable | The speed it leaves the carrier at; 0 drops it. |
| `heldBy` | `Entity` | `0` | — | — | read only | Written: the og.Interactor carrying it (0: none). |

```ogt
component Grab {
  aliases "pick up, carry, throw object, hold item, grab physics object, Half-Life gravity gun"
  doc "A body an og.Interactor picks up when it uses it: while carried it is made kinematic and moved each tick to `carry` in the carrier's frame; the carrier's next press throws it, dynamic again, along the carrier's forward at `throwSpeed`. Put it on a dynamic og.RigidBody."
  field carry Vector3 "Where it is carried, in the carrier's frame (forward +Z)." {
    unit m
    worldspace none
  }
  field throwSpeed f32 "The speed it leaves the carrier at; 0 drops it." {
    min 0
    unit mps
    tunable true
  }
  field heldBy Entity "Written: the og.Interactor carrying it (0: none)." {
    readonly true
  }
}
```

<sub>Module `interact` line 91</sub>
