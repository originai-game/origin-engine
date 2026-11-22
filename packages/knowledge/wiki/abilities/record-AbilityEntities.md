<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-AbilityEntities"></a>

### `og.AbilityEntities` (resource)

Every og.Ability, og.Stat, og.Status and og.Modifier in entity order, written by og.systems.abilities' first systems each tick for its last.

Token `og.AbilityEntities`, registered under the schema name `og.AbilityEntities`; its value type is `AbilityEntities`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `abilities` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The abilities (at most 1,024). |
| `stats` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The stats (at most 1,024). |
| `statuses` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The statuses (at most 1,024). |
| `modifiers` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The modifiers (at most 1,024). |

```ogt
resource AbilityEntities {
  doc "Every og.Ability, og.Stat, og.Status and og.Modifier in entity order, written by og.systems.abilities' first systems each tick for its last."
  field abilities List.Entity.1024 "The abilities (at most 1,024)."
  field stats List.Entity.1024 "The stats (at most 1,024)."
  field statuses List.Entity.1024 "The statuses (at most 1,024)."
  field modifiers List.Entity.1024 "The modifiers (at most 1,024)."
}
```

<sub>Module `abilities` line 118</sub>
