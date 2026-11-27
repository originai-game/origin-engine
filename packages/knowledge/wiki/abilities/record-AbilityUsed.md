<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-AbilityUsed"></a>

### `og.AbilityUsed` (event)

An og.Ability was used: sent the tick it was.

Also known as: on cast, ability cast, skill used, spell cast

Token `og.AbilityUsed`, registered under the schema name `og.AbilityUsed`; its value type is `AbilityUsed`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ability` | `Entity` | `0` | — | — | — | The og.Ability. |
| `owner` | `Entity` | `0` | — | — | — | Its owner. |

```ogt
event AbilityUsed {
  aliases "on cast, ability cast, skill used, spell cast"
  doc "An og.Ability was used: sent the tick it was."
  field ability Entity "The og.Ability."
  field owner Entity "Its owner."
}
```

<sub>Module `abilities` line 67</sub>
