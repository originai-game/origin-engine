<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SkeletonGroupDesc"></a>

### `SkeletonGroupDesc` (desc struct)

A skeleton standing in for content that is loading.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `SkeletonShape` | `"lines"` | — | — | — | What it stands in for. |
| `lines` | `number` | `0` | — | — | — | Lines, for lines; 0 is 3. |
| `count` | `number` | `0` | — | — | — | Repeats (rows of a list); 0 is 1. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | A block's or a card's art height; 0 is 120. |
| `name` | `string` | `"Loading"` | — | — | — | What a screen reader says while it loads. |

```ogt
struct SkeletonGroupDesc {
  doc "A skeleton standing in for content that is loading."
  desc true
  field shape SkeletonShape "What it stands in for."
  field lines u32 "Lines, for lines; 0 is 3."
  field count u32 "Repeats (rows of a list); 0 is 1."
  field width Option.Size "Its width; null grows."
  field height f32 "A block's or a card's art height; 0 is 120." {
    unit cssPx
    min 0
  }
  field name String "What a screen reader says while it loads." {
    default "Loading"
  }
}
```

<sub>Module `ui` line 5365</sub>
