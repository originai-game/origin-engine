<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Declutter"></a>

### `Declutter` (enum)

How an anchored element that overlaps one of higher priority (or as high and nearer) resolves.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | It overlaps. |
| `"shift"` | Moved the shortest way clear (else stacked above). |
| `"stack"` | Moved above what it overlaps, one over another (a crowd's nameplates). |
| `"hide"` | Faded out while it overlaps. |

```ogt
enum Declutter {
  doc "How an anchored element that overlaps one of higher priority (or as high and nearer) resolves."
  variant none "It overlaps."
  variant shift "Moved the shortest way clear (else stacked above)."
  variant stack "Moved above what it overlaps, one over another (a crowd's nameplates)."
  variant hide "Faded out while it overlaps."
}
```

<sub>Module `ui` line 7316</sub>
