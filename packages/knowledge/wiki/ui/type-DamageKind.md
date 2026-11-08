<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DamageKind"></a>

### `DamageKind` (enum)

What a damage number is.

A string: one of the variants below. The zero value is `"normal"`.

| Variant | Doc |
|---|---|
| `"normal"` | A hit. |
| `"critical"` | A critical hit: larger, in gold, popping past its size. |
| `"heal"` | Healing: success, with a +. |
| `"dot"` | Damage over time: smaller and dimmer. |
| `"miss"` | A miss, a dodge, a block or immune: the word, grey. |
| `"shield"` | Absorbed: the info colour. |
| `"self"` | Damage the player took: danger. |

```ogt
enum DamageKind {
  doc "What a damage number is."
  variant normal "A hit."
  variant critical "A critical hit: larger, in gold, popping past its size."
  variant heal "Healing: success, with a +."
  variant dot "Damage over time: smaller and dimmer."
  variant miss "A miss, a dodge, a block or immune: the word, grey."
  variant shield "Absorbed: the info colour."
  variant self "Damage the player took: danger."
}
```

<sub>Module `ui` line 6297</sub>
