<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-FlowDesc"></a>

### `FlowDesc` (desc struct)

A flow field over a grid.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `grid` | `Navigation` | `0` | — | — | — | The grid. |
| `targets` | `readonly Vector3[]` | `[]` | — | — | — | Where agents go: each point's cell is a target. |

```ogt
struct FlowDesc {
  doc "A flow field over a grid."
  desc true
  field grid Navigation "The grid."
  field targets List.Vector3 "Where agents go: each point's cell is a target."
}
```

<sub>Module `nav` line 425</sub>
