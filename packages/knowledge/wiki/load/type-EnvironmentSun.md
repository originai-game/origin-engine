<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentSun"></a>

### `EnvironmentSun` (struct)

The sun a cooked environment took out of its lighting image, for the DirectionalLight that puts it back. LDR panoramas extract by default; their intensity is a relative-radiance estimate from the clipped disc and circumsolar gradient, not measured lux. HDR/bracketed sources integrate measured radiance.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The unit direction towards the sun, in the map's own frame: the cook's `rotation` is applied, Environment.rotation is not (turn it about +Y by the same angle). |
| `color` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Its linear colour, the largest channel 1: DirectionalLight.color. |
| `intensity` | `number` | `0` | — | — | — | The light it gave a surface facing it: DirectionalLight.intensity. 0 when the environment has no sun to give (`sun keep`, an image with no sun, or an uncooked map). |
| `angularSize` | `number` | `0` | — | — | — | The apparent disc diameter in radians (solid-angle equivalent diameter of the extracted region); 0 when none was found. An estimate for clipped LDR images. |

```ogt
struct EnvironmentSun {
  doc "The sun a cooked environment took out of its lighting image, for the DirectionalLight that puts it back. LDR panoramas extract by default; their intensity is a relative-radiance estimate from the clipped disc and circumsolar gradient, not measured lux. HDR/bracketed sources integrate measured radiance."
  field direction Vector3 "The unit direction towards the sun, in the map's own frame: the cook's `rotation` is applied, Environment.rotation is not (turn it about +Y by the same angle)."
  field color Color "Its linear colour, the largest channel 1: DirectionalLight.color."
  field intensity f32 "The light it gave a surface facing it: DirectionalLight.intensity. 0 when the environment has no sun to give (`sun keep`, an image with no sun, or an uncooked map)."
  field angularSize f32 "The apparent disc diameter in radians (solid-angle equivalent diameter of the extracted region); 0 when none was found. An estimate for clipped LDR images."
}
```

<sub>Module `load` line 306</sub>
