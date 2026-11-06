<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiVariant"></a>

### `UiVariant` (struct)

A prop override under a condition (docs/ui.md §6.2: breakpoint variants and container queries). Later variants win.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `when` | `UiQuery` | `{}` | — | — | — | When it applies. |
| `set` | `UiPatch` | `{}` | — | — | — | What it changes. |

```ogt
struct UiVariant {
  doc "A prop override under a condition (docs/ui.md §6.2: breakpoint variants and container queries). Later variants win."
  field when UiQuery "When it applies."
  field set UiPatch "What it changes."
}
```

<sub>`api.ogt` line 20430</sub>
