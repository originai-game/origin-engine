<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentSun"></a>

### `EnvironmentSun` (struct)

The sun a cooked environment took out of its lighting image, for the DirectionalLight that puts it back. LDR panoramas extract by default; their intensity is a relative-radiance estimate from the clipped disc and circumsolar gradient, not measured lux. HDR sources integrate measured radiance.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The world-space unit direction towards the sun: both the cook's rotation and the runtime rotation argument are applied. |
| `color` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | Its linear colour, the largest channel 1: DirectionalLight.color. |
| `intensity` | `number` | `0` | — | — | — | DirectionalLight.intensity units, including cooked and runtime intensity: assign directly. 0 for sun keep, no detected sun or an uncooked map. LDR estimates share the sky's relative linear-radiance scale, not calibrated lux. |
| `angularSize` | `number` | `0` | — | — | — | The apparent disc diameter in radians (solid-angle equivalent diameter of the extracted region); 0 when none was found. An estimate for clipped LDR images. |

```ogt
struct EnvironmentSun {
  doc "The sun a cooked environment took out of its lighting image, for the DirectionalLight that puts it back. LDR panoramas extract by default; their intensity is a relative-radiance estimate from the clipped disc and circumsolar gradient, not measured lux. HDR sources integrate measured radiance."
  field direction Vector3 "The world-space unit direction towards the sun: both the cook's rotation and the runtime rotation argument are applied."
  field color Color "Its linear colour, the largest channel 1: DirectionalLight.color."
  field intensity f32 "DirectionalLight.intensity units, including cooked and runtime intensity: assign directly. 0 for sun keep, no detected sun or an uncooked map. LDR estimates share the sky's relative linear-radiance scale, not calibrated lux."
  field angularSize f32 "The apparent disc diameter in radians (solid-angle equivalent diameter of the extracted region); 0 when none was found. An estimate for clipped LDR images."
}
```

<sub>Module `load` line 318</sub>
