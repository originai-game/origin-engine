<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="type-ModifierOp"></a>

### `ModifierOp` (enum)

How an og.Modifier changes its og.Stat: all adds, then the percents summed, then each multiply.

A string: one of the variants below. The zero value is `"add"`.

| Variant | Doc |
|---|---|
| `"add"` | Adds its amount to the base. |
| `"percent"` | Its amount joins one sum: the value is times 1 + the sum (0.1 and 0.2 make 1.3). |
| `"multiply"` | Times 1 + its amount, one by one (0.1 and 0.2 make 1.32). |

```ogt
enum ModifierOp {
  doc "How an og.Modifier changes its og.Stat: all adds, then the percents summed, then each multiply."
  variant add "Adds its amount to the base."
  variant percent "Its amount joins one sum: the value is times 1 + the sum (0.1 and 0.2 make 1.3)."
  variant multiply "Times 1 + its amount, one by one (0.1 and 0.2 make 1.32)."
}
```

<sub>Module `abilities` line 18</sub>
