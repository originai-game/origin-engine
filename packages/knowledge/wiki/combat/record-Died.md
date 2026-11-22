<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Died"></a>

### `og.Died` (event)

A target's og.Health reached 0: sent once, the tick the hit was read (og.Health.dead is set).

See `combat §2`.

Also known as: on death, killed, health reached zero, death event

Token `og.Died`, registered under the schema name `og.Died`; its value type is `Died`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Health. |
| `source` | `Entity` | `0` | — | — | — | The attacker whose hit it was. |

```ogt
event Died {
  aliases "on death, killed, health reached zero, death event"
  doc "A target's og.Health reached 0: sent once, the tick the hit was read (og.Health.dead is set)."
  guide "combat §2"
  field entity Entity "The entity with the og.Health."
  field source Entity "The attacker whose hit it was."
}
```

<sub>Module `combat` line 144</sub>
