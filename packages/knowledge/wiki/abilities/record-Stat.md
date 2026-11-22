<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-Stat"></a>

### `og.Stat` (component)

A number something has, on an entity of its own: `value` is `base` with every og.Modifier whose stat it is applied: adds, then percents, then multiplies, each kind in ascending order of what it adds, so the order modifiers came in never changes it. Written by og.systems.abilities each tick.

See `abilities §2`.

Also known as: attribute, stat, character stats, speed stat, strength, armor value, mana pool, stamina, Unreal attribute set

Token `og.Stat`, registered under the schema name `og.Stat`; its value type is `Stat`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `base` | `number` | `0` | — | — | — | Its own number: a pool's amount (og.Ability.cost takes from it), a level's strength. |
| `value` | `number` | `0` | — | — | read only | Written: the base with its modifiers applied, each tick. |

```ogt
component Stat {
  aliases "attribute, stat, character stats, speed stat, strength, armor value, mana pool, stamina, Unreal attribute set"
  doc "A number something has, on an entity of its own: `value` is `base` with every og.Modifier whose stat it is applied: adds, then percents, then multiplies, each kind in ascending order of what it adds, so the order modifiers came in never changes it. Written by og.systems.abilities each tick."
  guide "abilities §2"
  field base f32 "Its own number: a pool's amount (og.Ability.cost takes from it), a level's strength."
  field value f32 "Written: the base with its modifiers applied, each tick." {
    readonly true
  }
}
```

<sub>Module `abilities` line 77</sub>
