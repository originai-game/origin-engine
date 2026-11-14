<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Contact"></a>

### `og.Contact` (event)

One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.

Token `og.Contact`, registered under the schema name `og.Contact`; its value type is `Contact`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `a` | `Entity` | `0` | — | — | — | The entity with the lower id. |
| `b` | `Entity` | `0` | — | — | — | The entity with the higher id. |
| `sensor` | `number` | `0` | — | — | — | 0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are. |

```ogt
event Contact {
  doc "One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now."
  field a Entity "The entity with the lower id."
  field b Entity "The entity with the higher id."
  field sensor u32 "0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are."
}
```

<sub>Module `physics` line 341</sub>
