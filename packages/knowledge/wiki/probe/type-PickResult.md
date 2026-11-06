<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-PickResult"></a>

### `PickResult` (struct)

What a screen point resolved to in the scene.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `string` | `""` | — | — | — | The probe id of the entity under the point. |
| `world` | `[number, number] \| [number, number, number]` | — | — | — | — | Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero. |
| `source` | `string \| null` | `null` | — | — | — | Where a person would change it: the scene line that declares the entity (`scenes/main.ogt:12`), or a code site as `player.ts::jump`. |
| `ui` | `UiPickElement` | `{ id: "", path: "", kind: "", text: "", rect: { x: 0, y: 0, width: 0, height: 0 }, layer: null, source: null, part: null, rule: null, rules: [] }` | — | — | optional | The UI element over the point, as pickUi answers: what the player sees there instead of the entity. |

```ogt
struct PickResult {
  doc "What a screen point resolved to in the scene."
  field entity String "The probe id of the entity under the point."
  field world OneOf.Array.f32.2.Array.f32.3 "Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero."
  field source Option.String "Where a person would change it: the scene line that declares the entity (`scenes/main.ogt:12`), or a code site as `player.ts::jump`."
  field ui UiPickElement "The UI element over the point, as pickUi answers: what the player sees there instead of the entity." {
    optional true
  }
}
```

<sub>Module `probe` line 14</sub>
