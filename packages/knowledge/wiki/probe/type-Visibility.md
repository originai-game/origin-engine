<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Visibility"></a>

### `Visibility` (struct)

One entity's verdict in the main camera's cull of the last frame drawn, as the culls that drew it decided (the GPU's with occlusion, or the CPU's). Natively the frame drawn last; on the web the newest whose GPU readback has landed, a frame or two before.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean \| null` | `null` | — | — | — | Whether the camera draws it: true for drawn and hlod, null for untracked. |
| `reason` | `DrawReason` | `"drawn"` | — | — | — | Why. |
| `detail` | `string` | `""` | — | — | optional | What the reason names: the hiding field, the loading asset, the untracked component; absent for none. |
| `drawn` | `number` | `0` | — | — | — | Its drawables (instances, skinned instances, sprites, terrains) that survived the cull. |
| `culled` | `number` | `0` | — | — | — | Its drawables culled by the layers, the view or the occluders. |

```ogt
struct Visibility {
  doc "One entity's verdict in the main camera's cull of the last frame drawn, as the culls that drew it decided (the GPU's with occlusion, or the CPU's). Natively the frame drawn last; on the web the newest whose GPU readback has landed, a frame or two before."
  field visible Option.bool "Whether the camera draws it: true for drawn and hlod, null for untracked."
  field reason DrawReason "Why."
  field detail String "What the reason names: the hiding field, the loading asset, the untracked component; absent for none." {
    optional true
  }
  field drawn u32 "Its drawables (instances, skinned instances, sprites, terrains) that survived the cull."
  field culled u32 "Its drawables culled by the layers, the view or the occluders."
}
```

<sub>Module `probe` line 493</sub>
