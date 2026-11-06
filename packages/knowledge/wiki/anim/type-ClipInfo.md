<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-ClipInfo"></a>

### `ClipInfo` (struct)

One clip of a loaded animation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clip` | `ClipAsset` | `0` | — | — | — | Its handle, what AnimationPlayer.clip takes. |
| `name` | `string` | `""` | — | — | — | The glTF animation's name. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `markers` | `readonly string[]` | `[]` | — | — | — | Its markers' names, in time order. |

```ogt
struct ClipInfo {
  doc "One clip of a loaded animation."
  field clip ClipAsset "Its handle, what AnimationPlayer.clip takes."
  field name String "The glTF animation's name."
  field duration f32 "Its length." {
    unit s
  }
  field markers List.String "Its markers' names, in time order."
}
```

<sub>`api.ogt` line 12024</sub>
