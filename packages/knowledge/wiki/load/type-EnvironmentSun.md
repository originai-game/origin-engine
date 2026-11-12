<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentSun"></a>

### `EnvironmentSun` (struct)

The sun a cooked environment's `sun extract` took out of its image, for the DirectionalLight that puts it back.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction towards the sun, in the map's own frame: the cook's `rotation` is applied, Environment.rotation is not (turn it about +Y by the same angle). |
| `color` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Its linear colour, the largest channel 1: DirectionalLight.color. |
| `intensity` | `number` | `0` | — | — | — | The light it gave a surface facing it: DirectionalLight.intensity. 0 when the environment has no sun to give (`sun keep`, an image with no sun, or an uncooked map). |

```ogt
struct EnvironmentSun {
  doc "The sun a cooked environment's `sun extract` took out of its image, for the DirectionalLight that puts it back."
  field direction Vector3 "The unit direction towards the sun, in the map's own frame: the cook's `rotation` is applied, Environment.rotation is not (turn it about +Y by the same angle)."
  field color Color "Its linear colour, the largest channel 1: DirectionalLight.color."
  field intensity f32 "The light it gave a surface facing it: DirectionalLight.intensity. 0 when the environment has no sun to give (`sun keep`, an image with no sun, or an uncooked map)."
}
```

<sub>Module `load` line 282</sub>
