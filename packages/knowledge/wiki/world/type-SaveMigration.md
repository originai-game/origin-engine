<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SaveMigration"></a>

### `SaveMigration` (struct)

A game's explicit save edit (GameDesc.migrations): run on a current-engine save of version `from` on its way to GameDesc.saveVersion. Engine records and state are never upgraded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | — | — | The save version it migrates from, to from + 1; from 1 to GameDesc.saveVersion - 1, each at most once. |
| `migrate` | `MigrateFn` | — | — | — | — | The step. |

```ogt
struct SaveMigration {
  doc "A game's explicit save edit (GameDesc.migrations): run on a current-engine save of version `from` on its way to GameDesc.saveVersion. Engine records and state are never upgraded."
  field from u32 "The save version it migrates from, to from + 1; from 1 to GameDesc.saveVersion - 1, each at most once."
  field migrate MigrateFn "The step."
}
```

<sub>Module `world` line 101</sub>
