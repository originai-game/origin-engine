<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-SensorExited"></a>

### `og.SensorExited` (event)

A collider stopped overlapping a sensor, or one of them went away.

Token `og.SensorExited`, registered under the schema name `og.SensorExited`; its value type is `SensorExited`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sensor` | `Entity` | `0` | — | — | — | The entity whose collider is the sensor. |
| `other` | `Entity` | `0` | — | — | — | The entity that left it. |

```ogt
event SensorExited {
  doc "A collider stopped overlapping a sensor, or one of them went away."
  field sensor Entity "The entity whose collider is the sensor."
  field other Entity "The entity that left it."
}
```

<sub>Module `physics` line 228</sub>
