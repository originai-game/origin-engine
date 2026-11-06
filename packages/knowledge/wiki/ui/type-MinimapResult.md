<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MinimapResult"></a>

### `MinimapResult` (struct)

What a minimap answered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `boolean` | `false` | — | — | — | It was pressed this tick: the map point is the press's position (og.Pointers) inside og.ui.bounds of <id>/Map, at the zoom and centre the game gave. |

```ogt
struct MinimapResult {
  doc "What a minimap answered."
  field clicked bool "It was pressed this tick: the map point is the press's position (og.Pointers) inside og.ui.bounds of <id>/Map, at the zoom and centre the game gave."
}
```

<sub>`api.ogt` line 22008</sub>
