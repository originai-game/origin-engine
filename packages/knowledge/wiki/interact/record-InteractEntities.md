<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-InteractEntities"></a>

### `og.InteractEntities` (resource)

Every og.Interactor, og.Toggle and og.Door in entity order, written by og.systems.interact's first systems each tick for its last, which reads the others' records entity by entity.

Token `og.InteractEntities`, registered under the schema name `og.InteractEntities`; its value type is `InteractEntities`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `interactors` | `readonly Entity[]` | `[]` | — | — | a list of at most 256 `Entity` (`List.Entity.256`): reads a frozen copy, writes a whole array | The interactors (at most 256). |
| `toggles` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The toggles (at most 1,024). |
| `doors` | `readonly Entity[]` | `[]` | — | — | a list of at most 1024 `Entity` (`List.Entity.1024`): reads a frozen copy, writes a whole array | The doors (at most 1,024). |

```ogt
resource InteractEntities {
  doc "Every og.Interactor, og.Toggle and og.Door in entity order, written by og.systems.interact's first systems each tick for its last, which reads the others' records entity by entity."
  field interactors List.Entity.256 "The interactors (at most 256)."
  field toggles List.Entity.1024 "The toggles (at most 1,024)."
  field doors List.Entity.1024 "The doors (at most 1,024)."
}
```

<sub>Module `interact` line 108</sub>
