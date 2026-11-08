<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SkeletonDesc"></a>

### `SkeletonDesc` (desc struct)

A skeleton: the neutral fill standing in for content that is loading.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | A line's height; 0 is the body size. |
| `lines` | `number` | `0` | — | — | — | Stacked lines; 0 or 1 is one, the last of several is shorter. |

```ogt
struct SkeletonDesc {
  doc "A skeleton: the neutral fill standing in for content that is loading."
  desc true
  field width Option.Size "Its width; null grows."
  field height f32 "A line's height; 0 is the body size." {
    unit cssPx
    min 0
  }
  field lines u32 "Stacked lines; 0 or 1 is one, the last of several is shorter."
}
```

<sub>Module `ui` line 1073</sub>
