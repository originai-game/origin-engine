<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavAreaDesc"></a>

### `NavAreaDesc` (desc struct)

A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `NavShape` | `{}` | — | — | — | Where. |
| `area` | `number` | `0` | ≤ 63 | — | — | The area id, 0 to 63 (0 is ground). |

```ogt
struct NavAreaDesc {
  doc "A run-time area: the spans under `shape` (their floors from the agent's climb below its bottom to its top) take area `area`, over the baked areas and over run-time areas of lower ids."
  desc true
  field shape NavShape "Where."
  field area u32 "The area id, 0 to 63 (0 is ground)." {
    max 63
  }
}
```

<sub>Module `nav` line 340</sub>
