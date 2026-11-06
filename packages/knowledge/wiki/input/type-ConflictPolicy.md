<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-ConflictPolicy"></a>

### `ConflictPolicy` (enum)

What a rebinding does with a control another action holds.

A string: one of the variants below. The zero value is `"refuse"`.

| Variant | Doc |
|---|---|
| `"refuse"` | Change nothing and answer the conflicts (a controls screen asks the player). |
| `"swap"` | The other action takes what this slot had. |
| `"replace"` | The other action loses the control (its slot is left unbound). |
| `"allow"` | Bind it anyway: both actions read it. |

```ogt
enum ConflictPolicy {
  doc "What a rebinding does with a control another action holds."
  variant refuse "Change nothing and answer the conflicts (a controls screen asks the player)."
  variant swap "The other action takes what this slot had."
  variant replace "The other action loses the control (its slot is left unbound)."
  variant allow "Bind it anyway: both actions read it."
}
```

<sub>Module `input` line 139</sub>
