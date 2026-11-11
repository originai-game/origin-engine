<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-Fill2dDesc"></a>

### `Fill2dDesc` (desc struct)

A round patch of ground for og.destruction.fill2d.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Its centre, in world space (its xy). |
| `radius` | `number` | `1` | ≥ 0 | `m` | — | Its radius: every cell whose centre is within it is made solid. |
| `bedrock` | `boolean` | `false` | — | — | — | Fill with bedrock (never carved) instead of ground. |

```ogt
struct Fill2dDesc {
  doc "A round patch of ground for og.destruction.fill2d."
  desc true
  field point Vector3 "Its centre, in world space (its xy)." {
    unit m
  }
  field radius f32 "Its radius: every cell whose centre is within it is made solid." {
    default 1
    min 0
    unit m
  }
  field bedrock bool "Fill with bedrock (never carved) instead of ground."
}
```

<sub>Module `destruction` line 431</sub>
