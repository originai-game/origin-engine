<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-Modifier"></a>

### `og.Modifier` (component)

A change to an og.Stat, on an entity of its own: a child of an og.Status counts its amount once per stack and goes with it; one without counts once until it is despawned.

See `abilities §2`.

Also known as: stat modifier, buff value, debuff value, bonus, multiplier, flat bonus, percent bonus

Token `og.Modifier`, registered under the schema name `og.Modifier`; its value type is `Modifier`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `stat` | `Entity` | `0` | — | — | — | The og.Stat it changes. |
| `op` | `ModifierOp` | `"add"` | — | — | — | How it changes it. |
| `amount` | `number` | `0` | — | — | — | What it adds (add), joins to the percent sum (percent) or multiplies by 1 + (multiply); 0 changes nothing. |

```ogt
component Modifier {
  aliases "stat modifier, buff value, debuff value, bonus, multiplier, flat bonus, percent bonus"
  doc "A change to an og.Stat, on an entity of its own: a child of an og.Status counts its amount once per stack and goes with it; one without counts once until it is despawned."
  guide "abilities §2"
  field stat Entity "The og.Stat it changes."
  field op ModifierOp "How it changes it."
  field amount f32 "What it adds (add), joins to the percent sum (percent) or multiplies by 1 + (multiply); 0 changes nothing."
}
```

<sub>Module `abilities` line 87</sub>
