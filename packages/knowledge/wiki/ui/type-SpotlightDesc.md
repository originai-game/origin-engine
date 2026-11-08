<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SpotlightDesc"></a>

### `SpotlightDesc` (desc struct)

A spotlight: the surface dimmed except a hole around a target.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `padding` | `number` | `0` | ≥ 0 | `cssPx` | — | Room around the target; 0 is space.sm. |

```ogt
struct SpotlightDesc {
  doc "A spotlight: the surface dimmed except a hole around a target."
  desc true
  field padding f32 "Room around the target; 0 is space.sm." {
    unit cssPx
    min 0
  }
}
```

<sub>Module `ui` line 924</sub>
