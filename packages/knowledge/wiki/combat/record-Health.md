<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Health"></a>

### `og.Health` (component)

What a target has left. og.systems.combat takes a hit's damage from `value` (never below 0) and sets `dead` when it reaches 0, sending og.Died once; it never heals. A dead target takes no more damage.

See `combat §2`.

Also known as: hit points, HP, life, health bar value, damageable

Token `og.Health`, registered under the schema name `og.Health`; its value type is `Health`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | ≥ 0 | — | tunable | What is left. |
| `dead` | `boolean` | `false` | — | — | — | Set when `value` reached 0; write false (and a value) to revive it. |

```ogt
component Health {
  aliases "hit points, HP, life, health bar value, damageable"
  doc "What a target has left. og.systems.combat takes a hit's damage from `value` (never below 0) and sets `dead` when it reaches 0, sending og.Died once; it never heals. A dead target takes no more damage."
  guide "combat §2"
  field value f32 "What is left." {
    min 0
    tunable true
  }
  field dead bool "Set when `value` reached 0; write false (and a value) to revive it."
}
```

<sub>Module `combat` line 107</sub>
