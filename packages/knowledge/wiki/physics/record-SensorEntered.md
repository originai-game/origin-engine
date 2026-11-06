<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-SensorEntered"></a>

### `og.SensorEntered` (event)

A collider started overlapping a sensor.

Token `og.SensorEntered`, registered under the schema name `og.SensorEntered`; its value type is `SensorEntered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sensor` | `Entity` | `0` | — | — | — | The entity whose collider is the sensor. |
| `other` | `Entity` | `0` | — | — | — | The entity that entered it. |

```ogt
event SensorEntered {
  doc "A collider started overlapping a sensor."
  field sensor Entity "The entity whose collider is the sensor."
  field other Entity "The entity that entered it."
}
```

<sub>`api.ogt` line 8697</sub>
