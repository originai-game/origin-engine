<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphState"></a>

### `GraphState` (struct)

A state in a graph layer.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Unique within the layer. |
| `motion` | `string` | `""` | — | — | — | A key in GraphDesc.motions. |
| `speed` | `number` | `0` | — | — | optional | Nonnegative playback multiplier; omitted is one. |
| `sync` | `string` | `""` | — | — | optional | Phase synchronization group; omitted means independent. |

```ogt
struct GraphState {
  doc "A state in a graph layer."
  field name String "Unique within the layer."
  field motion String "A key in GraphDesc.motions."
  field speed f32 "Nonnegative playback multiplier; omitted is one." {
    optional true
  }
  field sync String "Phase synchronization group; omitted means independent." {
    optional true
  }
}
```

<sub>Module `anim` line 733</sub>
