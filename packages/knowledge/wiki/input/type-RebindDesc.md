<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-RebindDesc"></a>

### `RebindDesc` (desc struct)

A player's rebinding of one slot of an action.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | Which binding: 0 the first; one past the last adds one. |
| `part` | `string` | `""` | — | — | — | A composite's part to change (`up`, `down`, `left`, `right`, `negative`, `positive`); empty sets the whole binding. |
| `binding` | `InputBinding` | `{}` | — | — | — | The new binding; for a part, its control is the part's. |
| `policy` | `ConflictPolicy` | `"refuse"` | — | — | — | What to do with a control another action holds. |

```ogt
struct RebindDesc {
  doc "A player's rebinding of one slot of an action."
  desc true
  field action String "The action."
  field slot u32 "Which binding: 0 the first; one past the last adds one."
  field part String "A composite's part to change (`up`, `down`, `left`, `right`, `negative`, `positive`); empty sets the whole binding."
  field binding InputBinding "The new binding; for a part, its control is the part's."
  field policy ConflictPolicy "What to do with a control another action holds."
}
```

<sub>`api.ogt` line 2161</sub>
