<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GraphNode"></a>

### `GraphNode` (desc struct)

A node of a graph, at a position in the graph's own units.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id; links name it. |
| `x` | `number` | `0` | — | — | — | Across, in graph units (CSS px at zoom 1). |
| `y` | `number` | `0` | — | — | — | Down. |
| `label` | `string` | `""` | — | — | — | Its name, under it. |
| `icon` | `string` | `""` | — | — | — | Its icon; empty shows the label's first letters. |
| `state` | `GraphNodeState` | `"available"` | — | — | — | Its state. |
| `shape` | `GraphNodeShape` | `"circle"` | — | — | — | Its outline. |
| `kind` | `string` | `""` | — | — | — | A selector flag a style dresses (`boss`, `shop`). |
| `disabled` | `boolean` | `false` | — | — | — | Shown but not pressable. |
| `team` | `number \| null` | `null` | — | — | — | Its side on a map: 0 the player's (ink, so it differs from danger in every style, a gold primary too), 1 an enemy's (danger), more the style's team colours; the node is filled in that colour, its ring still its state's. Null is no side (the state's ink alone). Pair a side with a shape (triangle, diamond) so sides are not told by colour alone. |

```ogt
struct GraphNode {
  doc "A node of a graph, at a position in the graph's own units."
  desc true
  field id String "Its id; links name it."
  field x f32 "Across, in graph units (CSS px at zoom 1)."
  field y f32 "Down."
  field label String "Its name, under it."
  field icon String "Its icon; empty shows the label's first letters."
  field state GraphNodeState "Its state."
  field shape GraphNodeShape "Its outline."
  field kind String "A selector flag a style dresses (`boss`, `shop`)."
  field disabled bool "Shown but not pressable."
  field team Option.u32 "Its side on a map: 0 the player's (ink, so it differs from danger in every style, a gold primary too), 1 an enemy's (danger), more the style's team colours; the node is filled in that colour, its ring still its state's. Null is no side (the state's ink alone). Pair a side with a shape (triangle, diamond) so sides are not told by colour alone."
}
```

<sub>`api.ogt` line 18434</sub>
