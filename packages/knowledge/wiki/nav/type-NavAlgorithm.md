<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavAlgorithm"></a>

### `NavAlgorithm` (enum)

The grid search a path query runs.

A string: one of the variants below. The zero value is `"jps"`.

| Variant | Doc |
|---|---|
| `"jps"` | Jump point search: the same optimal paths as A* with fewer nodes; on a grid with weighted cells it runs A*. |
| `"astar"` | A* over every cell. |

```ogt
enum NavAlgorithm {
  doc "The grid search a path query runs."
  variant jps "Jump point search: the same optimal paths as A* with fewer nodes; on a grid with weighted cells it runs A*."
  variant astar "A* over every cell."
}
```

<sub>`api.ogt` line 13556</sub>
