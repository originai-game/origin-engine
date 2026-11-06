<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-DestructibleInfo"></a>

### `DestructibleInfo` (struct)

What a destructible holds and how much of it stands (og.destruction.info).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chunks` | `number` | `0` | — | — | — | Its support chunks. |
| `children` | `number` | `0` | — | — | — | Their children, all told. |
| `bonds` | `number` | `0` | — | — | — | The bonds of its support graph. |
| `intact` | `number` | `0` | — | — | — | The bonds not yet broken. |
| `islands` | `number` | `0` | — | — | — | Its islands now. |
| `anchored` | `number` | `0` | — | — | — | Its support chunks still held to the world. |
| `health` | `number` | `0` | 0 … 1 | — | — | Its bonds' remaining health over their total, 1 intact to 0 rubble. |
| `debris` | `number` | `0` | — | — | — | Its fine pieces on the GPU now (og.Destructible.debris): broken off within og.Destruction.debrisLifetime, whether or not this device draws them. |
| `crumbs` | `number` | `0` | — | — | — | Its broken joints shedding grit on the GPU now (og.Destructible.debris): a few clay and mortar chips each, broken within og.Destruction.debrisLifetime (at most 48 joints a tick, spread over all that broke), whether or not this device draws them. |

```ogt
struct DestructibleInfo {
  doc "What a destructible holds and how much of it stands (og.destruction.info)."
  field chunks u32 "Its support chunks."
  field children u32 "Their children, all told."
  field bonds u32 "The bonds of its support graph."
  field intact u32 "The bonds not yet broken."
  field islands u32 "Its islands now."
  field anchored u32 "Its support chunks still held to the world."
  field health f32 "Its bonds' remaining health over their total, 1 intact to 0 rubble." {
    min 0
    max 1
  }
  field debris u32 "Its fine pieces on the GPU now (og.Destructible.debris): broken off within og.Destruction.debrisLifetime, whether or not this device draws them."
  field crumbs u32 "Its broken joints shedding grit on the GPU now (og.Destructible.debris): a few clay and mortar chips each, broken within og.Destruction.debrisLifetime (at most 48 joints a tick, spread over all that broke), whether or not this device draws them."
}
```

<sub>`api.ogt` line 27264</sub>
