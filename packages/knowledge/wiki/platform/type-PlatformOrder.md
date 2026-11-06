<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PlatformOrder"></a>

### `PlatformOrder` (enum)

How a leaderboard sorts; the first submission to a board fixes it.

A string: one of the variants below. The zero value is `"desc"`.

| Variant | Doc |
|---|---|
| `"desc"` | Higher is better (scores). |
| `"asc"` | Lower is better (times). |

```ogt
enum PlatformOrder {
  doc "How a leaderboard sorts; the first submission to a board fixes it."
  variant desc "Higher is better (scores)."
  variant asc "Lower is better (times)."
}
```

<sub>Module `platform` line 9</sub>
