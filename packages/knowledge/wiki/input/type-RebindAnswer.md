<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-RebindAnswer"></a>

### `RebindAnswer` (struct)

What a rebinding did.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `applied` | `boolean` | `false` | — | — | — | The binding changed (false: refused over conflicts). |
| `conflicts` | `readonly BindingConflict[]` | `[]` | — | — | — | The bindings it shared a control with. |

```ogt
struct RebindAnswer {
  doc "What a rebinding did."
  field applied bool "The binding changed (false: refused over conflicts)."
  field conflicts List.BindingConflict "The bindings it shared a control with."
}
```

<sub>Module `input` line 196</sub>
