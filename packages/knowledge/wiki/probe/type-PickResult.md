<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-PickResult"></a>

### `PickResult` (struct)

What a screen point resolved to.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `string` | `""` | — | — | — | The probe id of the entity under the point. |
| `world` | `[number, number] \| [number, number, number]` | — | — | — | — | Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero. |
| `source` | `string \| null` | `null` | — | — | — | Where a person would change it, as `player.ts::jump`. |

```ogt
struct PickResult {
  doc "What a screen point resolved to."
  field entity String "The probe id of the entity under the point."
  field world OneOf.Array.f32.2.Array.f32.3 "Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero."
  field source Option.String "Where a person would change it, as `player.ts::jump`."
}
```

<sub>`api.ogt` line 24038</sub>
