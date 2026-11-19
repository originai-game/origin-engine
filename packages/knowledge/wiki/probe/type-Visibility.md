<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Visibility"></a>

### `Visibility` (struct)

One entity's verdict in the main camera's cull of the frame at the current tick (the frame is extracted as the renderer draws it; no GPU needed).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean \| null` | `null` | — | — | — | Whether the camera draws it: true for drawn and hlod, null for untracked. |
| `reason` | `DrawReason` | `"drawn"` | — | — | — | Why. |
| `detail` | `string` | `""` | — | — | optional | What the reason names: the hiding field, the loading asset, the untracked component; absent for none. |
| `drawn` | `number` | `0` | — | — | — | Its drawables (instances, skinned instances, sprites, terrains) that survived the cull. |
| `culled` | `number` | `0` | — | — | — | Its drawables culled by the layers or the view. |

```ogt
struct Visibility {
  doc "One entity's verdict in the main camera's cull of the frame at the current tick (the frame is extracted as the renderer draws it; no GPU needed)."
  field visible Option.bool "Whether the camera draws it: true for drawn and hlod, null for untracked."
  field reason DrawReason "Why."
  field detail String "What the reason names: the hiding field, the loading asset, the untracked component; absent for none." {
    optional true
  }
  field drawn u32 "Its drawables (instances, skinned instances, sprites, terrains) that survived the cull."
  field culled u32 "Its drawables culled by the layers or the view."
}
```

<sub>Module `probe` line 478</sub>
