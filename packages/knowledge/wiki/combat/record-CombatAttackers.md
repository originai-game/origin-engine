<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-CombatAttackers"></a>

### `og.CombatAttackers` (resource)

Every og.Attack entity in entity order, written by og.systems.combat's first system each tick for its second, which reads the others' records entity by entity (at most 1,024 attackers).

Token `og.CombatAttackers`, registered under the schema name `og.CombatAttackers`; its value type is `CombatAttackers`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `attackers` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The attackers (og.systems.combat writes them; a game reads them). |

```ogt
resource CombatAttackers {
  doc "Every og.Attack entity in entity order, written by og.systems.combat's first system each tick for its second, which reads the others' records entity by entity (at most 1,024 attackers)."
  field attackers List.Entity.1024 "The attackers (og.systems.combat writes them; a game reads them)."
}
```

<sub>Module `combat` line 172</sub>
