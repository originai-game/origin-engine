<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-SensorEntered"></a>

### `og.SensorEntered` (event)

A collider started overlapping a sensor: something walked into a trigger area (a pressure plate, a pickup, a zone that starts a cutscene). A sensor is an og.Collider with `sensor` set; it reports overlaps and pushes nothing.

Also known as: trigger enter, OnTriggerEnter, Godot Area2D body_entered, Unreal OnComponentBeginOverlap, trigger volume, zone entered

Token `og.SensorEntered`, registered under the schema name `og.SensorEntered`; its value type is `SensorEntered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sensor` | `Entity` | `0` | — | — | — | The entity whose collider is the sensor. |
| `other` | `Entity` | `0` | — | — | — | The entity that entered it. |

```ogt
event SensorEntered {
  doc "A collider started overlapping a sensor: something walked into a trigger area (a pressure plate, a pickup, a zone that starts a cutscene). A sensor is an og.Collider with `sensor` set; it reports overlaps and pushes nothing."
  aliases "trigger enter, OnTriggerEnter, Godot Area2D body_entered, Unreal OnComponentBeginOverlap, trigger volume, zone entered"
  field sensor Entity "The entity whose collider is the sensor."
  field other Entity "The entity that entered it."
}
```

<sub>Module `physics` line 224</sub>
