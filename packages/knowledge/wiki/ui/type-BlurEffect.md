<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BlurEffect"></a>

### `BlurEffect` (desc struct)

The element and its subtree blurred (drawn offscreen).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `4` | ≥ 0 | `cssPx` | — | The blur. |

```ogt
struct BlurEffect {
  doc "The element and its subtree blurred (drawn offscreen)."
  desc true
  field radius f32 "The blur." {
    default 4
    min 0
    unit cssPx
  }
}
```

<sub>Module `ui` line 3116</sub>
