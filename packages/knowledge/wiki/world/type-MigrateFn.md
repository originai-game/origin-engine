<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-MigrateFn"></a>

### `MigrateFn` (callback)

One step of a game's save migration: changes the save from SaveMigration.from to the next version.

Game code supplies this function.

```ts
type MigrateFn = (save: SaveEdit) => Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `SaveEdit` | no | The save, open for editing. |

```ogt
callback MigrateFn {
  doc "One step of a game's save migration: changes the save from SaveMigration.from to the next version."
  param save SaveEdit "The save, open for editing."
  returns Result.Void "Refused (or a throw): the restore is refused with the reason, and the world is left as it was."
}
```

<sub>Module `world` line 94</sub>
