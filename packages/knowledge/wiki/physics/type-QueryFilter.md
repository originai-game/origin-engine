<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-QueryFilter"></a>

### `QueryFilter` (desc struct)

Which colliders a scene query considers. The zero value considers every solid collider on every layer.

See `physics §4`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mask` | `number` | `0` | — | — | — | Layers to consider: a collider counts when its layers meet this; 0 means every layer. |
| `includeSensors` | `boolean` | `false` | — | — | — | Also report sensors. |
| `exclude` | `Entity \| null` | `null` | — | — | — | An entity to ignore, such as the caster itself. |

```ogt
struct QueryFilter {
  doc "Which colliders a scene query considers. The zero value considers every solid collider on every layer."
  guide "physics §4"
  desc true
  field mask u32 "Layers to consider: a collider counts when its layers meet this; 0 means every layer."
  field includeSensors bool "Also report sensors."
  field exclude Option.Entity "An entity to ignore, such as the caster itself."
}
```

<sub>Module `physics` line 1817</sub>
