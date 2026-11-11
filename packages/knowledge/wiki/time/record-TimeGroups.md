<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

<a id="record-TimeGroups"></a>

### `og.TimeGroups` (resource)

Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). Registered when a game first names og.TimeGroup or og.TimeGroups; a world that never names them keeps its saves and hash. Physics stays on the global scale (one solver steps every body together): slow a body with its velocity, or move it kinematically.

Token `og.TimeGroups`, registered under the schema name `og.TimeGroups`; its value type is `TimeGroups`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scales` | `readonly number[]` | `[]` | — | — | a list of at most 16 `f32` (`List.f32.16`): reads a frozen copy, writes a whole array | Per group, its scale (0 to 10); past the list's length 1. |

```ogt
resource TimeGroups {
  doc "Each time group's scale on top of the global one, set with og.time.setGroupScale (every group starts at 1). Registered when a game first names og.TimeGroup or og.TimeGroups; a world that never names them keeps its saves and hash. Physics stays on the global scale (one solver steps every body together): slow a body with its velocity, or move it kinematically."
  field scales List.f32.16 "Per group, its scale (0 to 10); past the list's length 1."
}
```

<sub>Module `time` line 50</sub>
