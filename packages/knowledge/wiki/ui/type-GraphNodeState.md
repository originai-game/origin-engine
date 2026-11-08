<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GraphNodeState"></a>

### `GraphNodeState` (enum)

A graph node's state, shown by its ring and a mark as well as its colour.

A string: one of the variants below. The zero value is `"available"`.

| Variant | Doc |
|---|---|
| `"available"` | Can be taken (the default). |
| `"locked"` | Not yet: faint, with a lock's bar. |
| `"active"` | In progress: the primary ink and a glow. |
| `"done"` | Taken: a tick; links from it are solid. |
| `"current"` | Where the player is: the highlight ink and a dot. |

```ogt
enum GraphNodeState {
  doc "A graph node's state, shown by its ring and a mark as well as its colour."
  variant available "Can be taken (the default)."
  variant locked "Not yet: faint, with a lock's bar."
  variant active "In progress: the primary ink and a glow."
  variant done "Taken: a tick; links from it are solid."
  variant current "Where the player is: the highlight ink and a dot."
}
```

<sub>Module `ui` line 1657</sub>
